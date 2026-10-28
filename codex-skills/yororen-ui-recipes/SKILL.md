---
name: yororen-ui-recipes
description: End-to-end recipe patterns for end users building gpui apps with Yororen UI (yororen_ui). Use when the user asks for a complete working example, wants a screen layout pattern, or needs guidance composing components, modals, forms, list rendering, virtualized lists, notifications, theme switching, or i18n. Triggers include "show me a full example", "copy from the gallery", "how do I do theme switching", "how do I compose a modal + form", "how do I virtualize a long list". Not for contributing to yororen-ui itself.
---

# Yororen UI Recipes

Prefer copying and adapting proven patterns from the demos rather than
inventing new architectures. The five live demos converge on a small
set of working patterns; this skill points to the right demo for each
need and distills the cross-cutting composition rules.

## 1. The five demos

All five are under `crates/yororen-ui-demos/`. The first four are
members of the workspace `Cargo.toml`; the fifth is excluded but
still builds in isolation.

| Demo | What it shows | Copy from when you want… |
|---|---|---|
| `counter` | minimal bootstrap, single `Entity<T>` global, three buttons, `cx.notify()` | a starter template for any app |
| `layers_demo` | the 3 render pathways (headless / default-render / caller-custom Material ripple) side by side | to understand the visual flexibility of headless props |
| `inputs_demo` | all 7 text inputs wired with `cx.entity().clone()` `on_change` closures | an input form, a settings page, anything input-heavy |
| `gallery_demo` | the full 58-component showcase, theme switching, i18n, notification host, virtualized list | a kitchen-sink reference for any pattern |
| `theme_showcase` | per-render `theme::install(cx, theme)` for live theme switching | a "Next theme" toolbar, A/B theme testing |

The gallery is the most complete — start there if you're building a
real app. The counter is the smallest — start there if you're new.

## 2. Counter — the minimal template

Files: `crates/yororen-ui-demos/counter/src/`.

```rust
// main.rs
use gpui::{App, AppContext, Application, WindowBounds, WindowOptions, px, size};
use yororen_ui::assets::UiAsset;
use yororen_ui::locale_en;
use yororen_ui::renderer;

mod counter_app;
mod state;

fn main() {
    let app = Application::new().with_assets(UiAsset);
    app.run(|cx: &mut App| {
        renderer::install(cx, cx.window_appearance());
        locale_en::install(cx);
        cx.set_global(state::AppState::new(cx));

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(
                gpui::Bounds::centered(None, size(px(400.0), px(300.0)), cx),
            )),
            ..Default::default()
        };
        let _ = cx.open_window(options, |_, cx| cx.new(|_cx| counter_app::CounterApp));
    });
}
```

```rust
// state.rs
use gpui::{App, AppContext, Entity, Global};
#[derive(Default)] pub struct Counter { pub value: i32 }
pub struct AppState { pub counter: Entity<Counter> }
impl AppState {
    pub fn new(cx: &mut App) -> Self { Self { counter: cx.new(|_| Counter::default()) } }
}
impl Global for AppState {}
```

```rust
// counter_app.rs — three buttons, each with its own Entity clone.
use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div};
use yororen_ui::headless::button::button;
use yororen_ui::headless::label::label;
use crate::state::AppState;

pub struct CounterApp;
impl Render for CounterApp {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let count = cx.global::<AppState>().counter.read(cx).value;
        let inc = cx.global::<AppState>().counter.clone();
        let dec = cx.global::<AppState>().counter.clone();
        let reset = cx.global::<AppState>().counter.clone();

        div().size_full().flex().flex_col().items_center().justify_center().gap_3().p_4()
            .child(label("count", count.to_string(), cx).render(cx).text_size(gpui::px(20.)))
            .child(
                div().flex().gap_2()
                    .child(button("dec", cx).on_click(move |_, _, cx| {
                        dec.update(cx, |c, cx| { c.value -= 1; cx.notify(); });
                    }).render(cx).child("-"))
                    .child(button("reset", cx).on_click(move |_, _, cx| {
                        reset.update(cx, |c, cx| { c.value = 0; cx.notify(); });
                    }).render(cx).child("Reset"))
                    .child(button("inc", cx).on_click(move |_, _, cx| {
                        inc.update(cx, |c, cx| { c.value += 1; cx.notify(); });
                    }).render(cx).child("+")),
            )
    }
}
```

Three takeaways from the counter:

- The state is one `Entity<Counter>` inside a `Global` wrapper. The
  global is registered with `cx.set_global`, read with
  `cx.global::<AppState>()`.
- Each button's `move` closure clones the `Entity<Counter>` once at
  construction. The closure owns the clone.
- Mutate the value, then `cx.notify()`. gpui-ce 0.3 does not auto-notify
  on `Entity::update`.

## 3. Layers demo — the 3 render pathways

Files: `crates/yororen-ui-demos/layers_demo/src/`.

One window, three columns:

| Column | Render pathway | What it shows |
|---|---|---|
| 1 | `headless::button(id, cx).on_click(...).apply(div().bg(…).p_2().child("click me"))` | A11y only — caller owns every visual. The button does not respond to hover/press; the cursor changes to a pointer. |
| 2 | `headless::button(id, cx).variant(ActionVariantKind::Neutral).on_click(...).render(cx).child("Click me")` | The default renderer paints bg / border / padding / radius / hover / active from the theme JSON. |
| 3 | `material_button(id, "Click me".into(), cx, window)` (a custom Material-ripple painter defined in `material_button.rs`) | Caller writes a bespoke `gpui::Element` for the ripple animation. Headless `apply` is still called for focus + click. |

Copy from this demo when you need to:

- Understand what `.apply` does and doesn't do (it doesn't paint).
- See a hand-rolled `Element` (the ripple) that uses
  `window.request_animation_frame` to advance an `f32` progress and
  `PathBuilder` to draw a true circle.
- See how a custom renderer is wired into the headless flow (the
  `material_button` function calls `headless::button(id, cx).on_click(...).apply(div()...)`
  internally — the focus + click is the headless layer's job; the
  visual is the bespoke painter's job).

The demo also has a fourth panel: a `text_input` with `.render(cx, window)`
to prove the headless / renderer split works the same for inputs, not
just buttons.

## 4. Inputs demo — all 7 text inputs wired

Files: `crates/yororen-ui-demos/inputs_demo/src/`.

Seven panels, one per text input. The pattern is the same for all:

```rust
text_input("demo-text-input")
    .placeholder("Type here…")
    .on_change({
        let entity = cx.entity();           // <-- the canonical pattern
        move |new: &str, _window, cx| {
            entity.update(cx, |s, _cx| s.text_value = new.to_string());
        }
    })
    .render(cx, window)
```

Plus a status line that reads the value back from state:

```rust
fn status_line(text: &str) -> Div {
    div().text_color(hsla(0.0, 0.0, 0.4, 1.0)).text_size(px(12.)).child(text.to_string())
}
// ...
.child(status_line(&format!("text_input value: {:?}", self.text_value)))
```

Two non-obvious things the demo shows:

- **`number_input` has three callbacks** — `on_change(f64, …)` for
  the typed value, `on_increment(f64, …)` and `on_decrement(f64, …)`
  for the stepper buttons. All three write to the same field.
- **`keybinding_input` has its own `mode: KeybindingInputMode`**
  (`Idle` or `Capturing`). The renderer drives the mode transitions
  but you wire the `on_start_capture` / `on_cancel_capture` hooks to
  update your app state so the status line shows "Capturing…" vs "Idle".

## 5. Gallery demo — the kitchen sink

Files: `crates/yororen-ui-demos/gallery_demo/src/`.

By far the largest demo. It is the canonical reference for:

- The full component surface (58 components, all in one window).
- `cell()` and `input_cell()` helper functions that wrap a single
  component in a labelled card with a status line — the fastest way
  to compose a "compare every variant" page.
- A working theme switcher in the toolbar (default vs brutalism, light
  vs dark) using `cx.renderer_arc` and per-render
  `theme::install(cx, theme)`.
- i18n with multiple locales + app-specific translations layered on
  top of the framework strings, via
  `locale::install_with_translations(cx, "en"|"zh-CN"|"ar", app_map)`.
- The `NotificationCenter` host rendered with
  `gpui::deferred(...).with_priority(3)` so toasts float above
  modals.
- The modal scrim and Esc handling using `modal_state.set_on_close` +
  `ModalCloseReason::Escape | ScrimClick | Programmatic`.
- A virtualized list with infinite loading, using
  `on_visible_range_change` to bump the controller's count.

When the user asks "how do I do X", the answer is almost always in
the gallery. Open it before writing new code.

## 6. Theme showcase — live theme switching

Files: `crates/yororen-ui-demos/theme_showcase/src/` (excluded from
workspace but still builds; check the file before reaching for it).

Two-window pattern for theme switching:

```rust
fn main() {
    let app = Application::new().with_assets(UiAsset);
    app.run(|cx: &mut App| {
        // Register the 58 default renderers ONCE (no theme yet).
        default_renderer::install(cx, cx.window_appearance());

        let app_entity = cx.new(|_cx| theme_app::ThemeApp::new());
        let _ = cx.open_window(options, |_, _cx| app_entity);
    });
}

impl Render for ThemeApp {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Re-install the theme on every render — cheap, and lets a
        // toolbar button advance `self.current` and retheme the window.
        theme::install(cx, self.current_theme());
        let surface = cx.theme().get_color("surface.base").unwrap_or_default();
        // ... layout ...
    }
}
```

The four themes are usually `system-light`, `system-dark`, and two
inline `const CATPPUCCIN: &str = r##"{...}"##;` JSON strings. Bump
`self.current = (self.current + 1) % themes.len()` in a click handler,
`cx.notify()`, and the next render re-themes the window.

## 7. Composition rules

These are the rules the demos converge on. Violate them and the UI
becomes hard to maintain or visibly broken.

- **Stable identity.** Every stateful child in a list, virtualized
  view, or any tree that can be reordered needs `.id(...)`. Use the
  row index as the id when the data is append-only; use the data's
  primary key when the data is mutable.
  Example: `text_input(format!("row-email-{ix}"))` in a virtual list.

- **Render overlays at the scroll root, not inside a section.** A
  modal that lives inside a scrollable column will be clipped and
  won't receive Escape reliably. Build the scroll root, then
  `.child(content).child(modal).child(toast_host)` as siblings, with
  the modal and toast wrapped in `gpui::deferred(...).with_priority(N)`.

- **`&mut **cx` for minting entities from `Context<T>`.**
  `cx.focus_handle()` and `XxxState::new(cx)` need `&mut App`. Inside
  a `Render::render` closure you have `&mut Context<MyApp>`;
  `Context<T>: DerefMut<Target = App>`, so `&mut **cx` is a
  `&mut App`. Use it inline at the call site, never stored in a `let`.

- **Stable identifier scope.** Two inputs with the same `id` share
  the same keyed state. If you render the same logical input twice
  (in a list of email rows, say), include the row index or some
  unique key in the id: `format!("email-{row_id}")`.

- **No `render()` side effects.** Render closures must be pure with
  respect to gpui's redraw cycle. Spawning tasks, calling `cx.spawn`,
  doing file I/O — all of these go in event handlers, not in
  `Render::render`.

- **One renderer per `(Marker, dyn Trait)` slot.** If you register
  `BrutalButtonRenderer` for the `Button` marker, the previous
  `TokenButtonRenderer` is overwritten. There's no layering. If you
  want a hybrid, write a new renderer that does both.

## 8. Choosing a renderer

| Renderer | When to use | Install |
|---|---|---|
| **default** (`yororen-ui-default-renderer`) | Standard look, full coverage, well-tested | `yororen_ui::renderer::install(cx, cx.window_appearance())` |
| **brutalism** (`yororen-ui-brutalism-renderer`, feature-gated) | Sharp corners, hard shadows, monospace, high-contrast | `yororen_ui::brutalism_renderer::install(cx)` |
| **custom** (your own crate) | Brand identity, accessibility contrast requirements, animation customization | Write `cx.register_renderer_arc::<m::X, dyn XRenderer>(Arc::new(MyRenderer))` for each marker you implement |

The same `headless::button("save", cx).on_click(...).render(cx)` works
in all three. The visual is what changes.

To make the choice at runtime (a "Switch theme" button), see
`$yororen-ui-app-core` § 11 and the `theme_switcher.rs` file in
`gallery_demo`.

## 9. Related skills

- `$yororen-ui-user` — entry point, hard rules
- `$yororen-ui-app-core` — bootstrap, theme authoring, state, i18n
- `$yororen-ui-state-inputs` — inputs, forms, modals, composites

## 10. References

- `references/overlay-z-order.md` — paint priority for popovers, modals, toasts
- `references/custom-renderer.md` — writing your own 58-marker renderer crate
