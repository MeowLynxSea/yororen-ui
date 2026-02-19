use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, Mutex},
    time::Duration,
};

use chrono::{DateTime, Utc};
use gpui::{AnyWindowHandle, AppContext, ClickEvent, Global, SharedString, Window};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::component::ToastKind;

/// How a notification should be dismissed.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DismissStrategy {
    /// Never dismiss automatically. User must explicitly dismiss.
    Manual,
    /// Dismiss after the given duration.
    After { duration_ms: u64 },
}

impl Default for DismissStrategy {
    fn default() -> Self {
        Self::After { duration_ms: 4000 }
    }
}

/// A single notification payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,

    pub title: Option<SharedString>,
    pub message: SharedString,
    pub kind: ToastKind,

    pub dismiss: DismissStrategy,

    /// Optional arbitrary payload for user handling.
    ///
    /// This is persisted for `sticky` notifications.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<JsonValue>,

    /// Optional label for the action that occurs on click.
    pub action_label: Option<SharedString>,

    /// If true, the notification is retained across persistence loads.
    /// Useful for long-running tasks or important messages.
    pub sticky: bool,
}

impl Notification {
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            title: None,
            message: message.into(),
            kind: ToastKind::Neutral,
            dismiss: DismissStrategy::default(),
            payload: None,
            action_label: None,
            sticky: false,
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn kind(mut self, kind: ToastKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn dismiss(mut self, dismiss: DismissStrategy) -> Self {
        self.dismiss = dismiss;
        self
    }

    pub fn action_label(mut self, label: impl Into<SharedString>) -> Self {
        self.action_label = Some(label.into());
        self
    }

    pub fn payload(mut self, payload: JsonValue) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn sticky(mut self, sticky: bool) -> Self {
        self.sticky = sticky;
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub(crate) struct PersistedState {
    // keep it intentionally small; callbacks/handles are not persisted
    items: Vec<Notification>,
}

type ClickCb = Arc<dyn Fn(&Notification, &ClickEvent, &mut Window, &mut gpui::App)>;
type DismissCb = Arc<dyn Fn(&Notification, &mut Window, &mut gpui::App)>;

#[derive(Clone)]
pub struct NotificationCenter {
    state: Arc<Mutex<State>>,
}

#[derive(Default)]
struct State {
    queue: VecDeque<Notification>,

    // config
    max_queue_len: usize,
    persist_enabled: bool,
    persist_key: SharedString,

    // host registration
    host_window: Option<AnyWindowHandle>,

    // persistence - owned by the host (created via window.use_keyed_state during render)
    persisted_state: Option<gpui::Entity<PersistedState>>,
    loaded_from_persisted: bool,

    // callbacks - not persisted
    on_click: HashMap<Uuid, ClickCb>,
    on_dismiss: HashMap<Uuid, DismissCb>,

    // used to avoid re-scheduling auto-dismiss for the same notification
    scheduled_auto_dismiss: HashSet<Uuid>,
}

impl Global for NotificationCenter {}

impl NotificationCenter {
    #[allow(clippy::arc_with_non_send_sync)]
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                max_queue_len: 5,
                persist_enabled: true,
                persist_key: "yororen_ui:notifications".into(),
                persisted_state: None,
                loaded_from_persisted: false,
                ..State::default()
            })),
        }
    }

    pub fn set_max_queue_len(&self, max: usize) {
        let mut state = self.state.lock().unwrap();
        state.max_queue_len = max.max(1);
        Self::trim_queue_locked(&mut state);
    }

    pub fn set_persistence(&self, enabled: bool, key: impl Into<SharedString>) {
        let mut state = self.state.lock().unwrap();
        state.persist_enabled = enabled;
        state.persist_key = key.into();
    }

    pub fn persistence_config(&self) -> (bool, SharedString) {
        let state = self.state.lock().unwrap();
        (state.persist_enabled, state.persist_key.clone())
    }

    pub fn register_host_window(&self, window: AnyWindowHandle) {
        let mut state = self.state.lock().unwrap();
        state.host_window = Some(window);
    }

    pub fn unregister_host_window(&self, window: AnyWindowHandle) {
        let mut state = self.state.lock().unwrap();
        if state.host_window == Some(window) {
            state.host_window = None;
        }
    }

    pub fn notify(&self, n: Notification, cx: &mut gpui::App) -> Uuid {
        let id = n.id;

        {
            let mut state = self.state.lock().unwrap();
            state.queue.push_back(n);
            Self::trim_queue_locked(&mut state);
        }

        self.persist(cx);
        self.refresh_host(cx);
        self.maybe_schedule_auto_dismiss(id, cx);
        id
    }

    pub fn notify_with_callbacks(
        &self,
        n: Notification,
        on_click: Option<ClickCb>,
        on_dismiss: Option<DismissCb>,
        cx: &mut gpui::App,
    ) -> Uuid {
        let id = self.notify(n, cx);
        let mut state = self.state.lock().unwrap();
        if let Some(cb) = on_click {
            state.on_click.insert(id, cb);
        }
        if let Some(cb) = on_dismiss {
            state.on_dismiss.insert(id, cb);
        }
        id
    }

    pub fn dismiss(&self, id: Uuid, cx: &mut gpui::App) {
        {
            let mut state = self.state.lock().unwrap();
            state.queue.retain(|n| n.id != id);
            state.on_click.remove(&id);
            state.on_dismiss.remove(&id);
            state.scheduled_auto_dismiss.remove(&id);
        }

        self.persist(cx);
        self.refresh_host(cx);
    }

    pub fn clear(&self, cx: &mut gpui::App) {
        {
            let mut state = self.state.lock().unwrap();
            state.queue.clear();
            state.on_click.clear();
            state.on_dismiss.clear();
            state.scheduled_auto_dismiss.clear();
        }
        self.persist(cx);
        self.refresh_host(cx);
    }

    pub fn items(&self) -> Vec<Notification> {
        let state = self.state.lock().unwrap();
        state.queue.iter().cloned().collect()
    }

    pub(crate) fn click(&self, id: Uuid, ev: &ClickEvent, window: &mut Window, cx: &mut gpui::App) {
        let (n, cb) = {
            let state = self.state.lock().unwrap();
            let n = state.queue.iter().find(|n| n.id == id).cloned();
            let cb = state.on_click.get(&id).cloned();
            (n, cb)
        };

        if let (Some(n), Some(cb)) = (n, cb) {
            cb(&n, ev, window, cx);
        }
    }

    pub(crate) fn dismiss_from_ui(&self, id: Uuid, window: &mut Window, cx: &mut gpui::App) {
        let (n, cb) = {
            let state = self.state.lock().unwrap();
            let n = state.queue.iter().find(|n| n.id == id).cloned();
            let cb = state.on_dismiss.get(&id).cloned();
            (n, cb)
        };

        if let (Some(n), Some(cb)) = (n, cb) {
            cb(&n, window, cx);
        }

        self.dismiss(id, cx);
    }

    /// Triggers a re-load from the persisted snapshot on the next host render.
    ///
    /// Note: the actual persisted state entity is owned by [`NotificationHost`], because
    /// `Window::use_keyed_state` can only be called during render/layout/paint.
    pub fn load_persisted(&self, cx: &mut gpui::App) {
        {
            let mut state = self.state.lock().unwrap();
            state.loaded_from_persisted = false;
        }
        self.refresh_host(cx);
    }

    pub(crate) fn bind_persisted_state(
        &self,
        entity: gpui::Entity<PersistedState>,
        cx: &mut gpui::App,
    ) {
        let (should_load, loaded_snapshot) = {
            let mut state = self.state.lock().unwrap();
            state.persisted_state = Some(entity.clone());
            if state.loaded_from_persisted {
                (false, PersistedState::default())
            } else {
                let snapshot = entity.read(cx).clone();
                state.loaded_from_persisted = true;
                (true, snapshot)
            }
        };

        if should_load {
            let ids_to_schedule = {
                let mut state = self.state.lock().unwrap();
                state.queue = loaded_snapshot.items.into_iter().collect();
                Self::trim_queue_locked(&mut state);
                state.queue.iter().map(|n| n.id).collect::<Vec<_>>()
            };

            for id in ids_to_schedule {
                self.maybe_schedule_auto_dismiss(id, cx);
            }
        }
    }

    pub(crate) fn unbind_persisted_state(&self) {
        let mut state = self.state.lock().unwrap();
        state.persisted_state = None;
    }

    fn maybe_schedule_auto_dismiss(&self, id: Uuid, cx: &mut gpui::App) {
        let (dismiss, host_window, already_scheduled) = {
            let mut state = self.state.lock().unwrap();
            let Some(n) = state.queue.iter().find(|n| n.id == id) else {
                return;
            };
            let dismiss = n.dismiss.clone();
            let host = state.host_window;
            let already = state.scheduled_auto_dismiss.contains(&id);
            if !already {
                state.scheduled_auto_dismiss.insert(id);
            }
            (dismiss, host, already)
        };

        if already_scheduled {
            return;
        }

        let DismissStrategy::After { duration_ms } = dismiss else {
            return;
        };

        // Require a host window for correctness: we don't want to spawn tasks in a context
        // where there is no window rendering to reflect the changes.
        if host_window.is_none() {
            return;
        }

        let this = self.clone();
        cx.spawn(async move |cx| {
            cx.background_executor()
                .timer(Duration::from_millis(duration_ms))
                .await;
            cx.update(|cx| {
                this.dismiss(id, cx);
            })
            .ok();
        })
        .detach();
    }

    fn refresh_host(&self, cx: &mut gpui::App) {
        let host = { self.state.lock().unwrap().host_window };
        if let Some(host) = host {
            cx.spawn(async move |cx| {
                cx.update(|app| {
                    app.update_window(host, |_, window, _cx| {
                        window.refresh();
                    })
                    .ok();
                })
                .ok();
            })
            .detach();
        }
    }

    fn persist(&self, cx: &mut gpui::App) {
        let (enabled, entity, snapshot) = {
            let state = self.state.lock().unwrap();
            let snapshot = PersistedState {
                items: state.queue.iter().filter(|n| n.sticky).cloned().collect(),
            };
            (
                state.persist_enabled,
                state.persisted_state.clone(),
                snapshot,
            )
        };

        if !enabled {
            return;
        }

        let Some(entity) = entity else {
            return;
        };

        entity.update(cx, |state, _| {
            *state = snapshot;
        });
    }

    fn trim_queue_locked(state: &mut State) {
        while state.queue.len() > state.max_queue_len {
            if let Some(removed) = state.queue.pop_front() {
                state.on_click.remove(&removed.id);
                state.on_dismiss.remove(&removed.id);
                state.scheduled_auto_dismiss.remove(&removed.id);
            }
        }
    }
}

impl Default for NotificationCenter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_queue_len_trims_oldest() {
        let center = NotificationCenter::new();
        center.set_max_queue_len(2);

        // We can exercise trimming without a GPUI App by accessing the internal lock.
        // (No persistence/auto-dismiss scheduling is involved here.)
        {
            let mut state = center.state.lock().unwrap();
            state.queue.push_back(Notification::new("1"));
            state.queue.push_back(Notification::new("2"));
            state.queue.push_back(Notification::new("3"));
            NotificationCenter::trim_queue_locked(&mut state);
        }

        let items = center.items();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].message.as_str(), "2");
        assert_eq!(items[1].message.as_str(), "3");
    }
}
