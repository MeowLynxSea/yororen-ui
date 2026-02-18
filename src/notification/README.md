# Notification Center

This crate includes a small, unified **Notification Center** that manages toast-like notifications.

It provides:

- **Queue management** (default `max_queue_len = 5`)
- **Persistence** for sticky notifications (survive window refresh / state restore)
- **Click callbacks** and **dismiss callbacks**
- **Dismiss strategies** (manual vs auto-dismiss)

## Quick start

1) Render the host overlay once in your window root:

```rust,ignore
use yororen_ui::notification::notification_host;

div()
  .child(app_content)
  .child(notification_host())
```

2) Push notifications from anywhere you have `&mut gpui::App`:

```rust,ignore
use yororen_ui::notification::{Notification, NotificationCenter};
use yororen_ui::component::ToastKind;

let center = cx.global::<NotificationCenter>().clone();
center.notify(Notification::new("Saved!").kind(ToastKind::Success), cx);
```

## Persistence behavior

- Only notifications with `sticky = true` are persisted.
- Callbacks are **not** persisted.
- `payload` is persisted for `sticky` notifications.
