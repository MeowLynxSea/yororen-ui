# Yororen UI

<p align="center">
  <a href="README_zh_CN.md">中文版</a> · <strong>English</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/license-Apache%202.0-blue.svg" alt="License">
  <img src="https://img.shields.io/badge/rust-edition%202024-yellow.svg" alt="Rust Edition">
  <img src="https://img.shields.io/badge/gpui-based-2a2a2a.svg" alt="Powered by gpui">
  <img src="https://img.shields.io/badge/version-0.3.0-orange.svg" alt="Version">
</p>

**Yororen UI** is a headless-first Rust UI library built on top of [`gpui`](https://github.com/zed-industries/zed) (via [`gpui-ce`](https://crates.io/crates/gpui-ce)). A button is not a visual — it is a focusable, clickable thing with a label and an optional icon. The visual is a plug-in.

```text
theme JSON  ─▶  renderer (XxxRenderer)  ─▶  headless (XxxProps)  ─▶  gpui-ce
```

- **Headless** ([`yororen-ui-core`](https://crates.io/crates/yororen-ui-core)) — data, state, a11y, i18n, RTL, animation, assets. No visual decisions.
- **Renderer** ([`yororen-ui-default-renderer`](https://crates.io/crates/yororen-ui-default-renderer) · [`yororen-ui-brutalism-renderer`](https://crates.io/crates/yororen-ui-brutalism-renderer)) — turns props into a styled div. 55 trait slots, filled with `Token*` or `Brutal*` impls. Swap the renderer, change the entire look.
- **Theme** — a JSON file. The renderer reads paths like `action.primary.bg`; missing paths fall back to renderer defaults.

The meta-crate [`yororen-ui`](https://crates.io/crates/yororen-ui) re-exports core + the default renderer + three bundled locales, so most apps need a single dependency. Add the `brutalism` or `xml` feature to opt into the alternative renderer or the XML DSL.

---

## Features

<table>
  <tr>
    <th width="30%">Capability</th>
    <th>What you get</th>
  </tr>
  <tr>
    <td><strong>55 components</strong></td>
    <td>Buttons, inputs, badges, tooltips, modals, popovers, selects, lists, virtualised lists, trees, tables, and more</td>
  </tr>
  <tr>
    <td><strong>Three-layer architecture</strong></td>
    <td>Headless primitives + JSON themes + swappable visual renderers (default + brutalism)</td>
  </tr>
  <tr>
    <td><strong>JSON themes</strong></td>
    <td>Swap palettes at runtime with one <code>install()</code> call</td>
  </tr>
  <tr>
    <td><strong>Animation system</strong></td>
    <td><code>AnimatedVisibility</code> on every stateful composite, plus presets and easing functions</td>
  </tr>
  <tr>
    <td><strong>Internationalization</strong></td>
    <td><code>en</code>, <code>zh-CN</code>, <code>ar</code> bundled; <code>ar</code> flips layout to RTL via <code>cx.i18n().text_direction()</code></td>
  </tr>
  <tr>
    <td><strong>Accessibility</strong></td>
    <td>Focus management, keyboard navigation, click-outside guards, scroll-lock counter, focus trap</td>
  </tr>
  <tr>
    <td><strong>Embedded assets</strong></td>
    <td>20+ SVG icons bundled via <code>rust-embed</code></td>
  </tr>
  <tr>
    <td><strong>Notification system</strong></td>
    <td><code>NotificationCenter</code> global with auto-dismiss timers, sticky flag, and persistence</td>
  </tr>
  <tr>
    <td><strong>Optional XML DSL</strong></td>
    <td>Declarative screens via <code>xml!</code> / <code>xml_file!</code> macros</td>
  </tr>
</table>

---

## Quick start

```rust
use gpui::{App, Application, Bounds, WindowBounds, WindowOptions, px, size};
use yororen_ui::assets::UiAsset;
use yororen_ui::locale_en;
use yororen_ui::renderer;

fn main() {
    let app = Application::new().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        // 1) Renderer + theme — picks system-light or system-dark by OS appearance,
        //    installs the global Theme, and registers 55 default XxxRenderer impls.
        renderer::install(cx, cx.window_appearance());

        // 2) Text-input keymap (idempotent).
        yororen_ui::headless::text_input::init(cx);

        // 3) Locale.
        locale_en::install(cx);

        // 4) Main window.
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(
                Bounds::centered(None, size(px(800.), px(600.)), cx),
            )),
            ..Default::default()
        };
        cx.open_window(options, |_, cx| cx.new(|_| my_app::MyApp));
    });
}
```

Inside your `Render::render`:

```rust
use yororen_ui::headless::button::button;
use yororen_ui::headless::label::label;

fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    div().size_full().flex().items_center().justify_center().gap_2()
        .child(label("count", "0", cx).render(cx))
        .child(button("inc", cx).caption("+").render(cx))
}
```

Every headless factory exposes both `.apply(div)` (a11y only) and `.render(cx)` (full visual via the registered renderer). For text inputs, `.render(cx, window)` is two-arg and registers an IME handler.

<details>
<summary><strong>Swap renderers or themes (optional)</strong></summary>

### Brutalism renderer

```rust
use yororen_ui::brutalism_renderer;

brutalism_renderer::install(cx);   // sharp corners, hard shadows, monospace
```

### Custom JSON theme

```rust
use yororen_ui_default_renderer::{Theme, install_with};

const MY_THEME: &str = include_str!("../themes/my-brand.json");
install_with(cx, Theme::from_json(MY_THEME).expect("valid JSON"));
```

### Live theme switching

Call `yororen_ui::theme::install(cx, new_theme)` inside a `Render` impl — every frame, or when the user picks a new palette. Idempotent and cheap.

```rust
impl Render for MyApp {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        yororen_ui::theme::install(cx, self.current_theme());
        // … rest of render …
    }
}
```

</details>

---

## Demos

<!-- ====================================================================== -->
<!-- To regenerate a screenshot: run the demo and capture a PNG into          -->
<!-- screenshots/<name>.png with the matching filename.                    -->
<!-- See screenshots/README.md for the convention.                         -->
<!-- ====================================================================== -->

<table>
  <tr>
    <td width="50%" valign="top">

**`counter`** — minimal bootstrap, single `Entity<T>` global, three buttons.

<p><img src="https://raw.githubusercontent.com/MeowLynxSea/yororen-ui/refs/heads/main/screenshots/counter.png" width="480" alt="Counter demo"></p>

```
cargo run -p counter-demo
```

</td>
    <td width="50%" valign="top">

**`gallery_demo`** — kitchen-sink reference: every component, theme switching, i18n, notifications, virtualised list.

<p><img src="https://raw.githubusercontent.com/MeowLynxSea/yororen-ui/refs/heads/main/screenshots/gallery-demo.png" width="480" alt="Gallery demo"></p>

```
cargo run -p gallery-demo
```

</td>
  </tr>
  <tr>
    <td width="50%" valign="top">

**`inputs_demo`** — all seven text inputs wired with `cx.entity().clone()`. Handles <kbd>Tab</kbd> / <kbd>Enter</kbd> / <kbd>Esc</kbd> / <kbd>⌘V</kbd>.

<p><img src="https://raw.githubusercontent.com/MeowLynxSea/yororen-ui/refs/heads/main/screenshots/inputs-demo.png" width="480" alt="Inputs demo"></p>

```
cargo run -p inputs-demo
```

</td>
    <td width="50%" valign="top">

**`layers_demo`** — the three render pathways side by side + a hand-rolled Material ripple.

<p><img src="https://raw.githubusercontent.com/MeowLynxSea/yororen-ui/refs/heads/main/screenshots/layers-demo.png" width="480" alt="Layers demo"></p>

```
cargo run -p layers-demo
```

</td>
  </tr>
  <tr>
    <td width="50%" valign="top">

**`theme_showcase`** — live theme switching via `theme::install` per render. Cycles four themes.

<p><img src="https://raw.githubusercontent.com/MeowLynxSea/yororen-ui/refs/heads/main/screenshots/theme-showcase.png" width="480" alt="Theme showcase demo"></p>

```
cargo run -p theme-showcase-demo
```

</td>
    <td width="50%" valign="top">

**`variant_showcase`** — `ActionVariantKind` comparison (`Neutral` / `Primary` / `Danger`) with the same `headless::button` factory.

<p><img src="https://raw.githubusercontent.com/MeowLynxSea/yororen-ui/refs/heads/main/screenshots/variant-showcase.png" width="480" alt="Variant showcase demo"></p>

```
cargo run -p variant-showcase-demo
```

</td>
  </tr>
  <tr>
    <td width="50%" valign="top">

**`gallery_xml`** — the gallery written with the XML DSL instead of the Rust builder API.

<p><img src="https://raw.githubusercontent.com/MeowLynxSea/yororen-ui/refs/heads/main/screenshots/gallery-xml.png" width="480" alt="Gallery XML demo"></p>

```
cargo run -p gallery-xml-demo
```

</td>
    <td width="50%" valign="top">

**`showcase_xml`** — smaller XML-DSL smoke test.

<p><img src="https://raw.githubusercontent.com/MeowLynxSea/yororen-ui/refs/heads/main/screenshots/showcase-xml.png" width="480" alt="Showcase XML demo"></p>

```
cargo run -p showcase-xml-demo
```

</td>
  </tr>
</table>

---

## What's inside

<table>
  <tr>
    <th>Crate</th>
    <th>Role</th>
  </tr>
  <tr>
    <td><code>yororen-ui-core</code></td>
    <td>Headless primitives, theme JSON access, i18n, a11y, RTL, animation, assets, notification center</td>
  </tr>
  <tr>
    <td><code>yororen-ui-default-renderer</code></td>
    <td>55 <code>TokenXxxRenderer</code> impls + bundled <code>system-light.json</code> / <code>system-dark.json</code> themes + <code>renderer::install</code> bootstrap</td>
  </tr>
  <tr>
    <td><code>yororen-ui-brutalism-renderer</code><br><sub><em>(optional, feature <code>brutalism</code>)</em></sub></td>
    <td>Sharp corners, thick black borders, hard offset shadows, monospace typography</td>
  </tr>
  <tr>
    <td><code>yororen-ui-xml</code> + <code>yororen-ui-xml-macro</code><br><sub><em>(optional, feature <code>xml</code>, default-on)</em></sub></td>
    <td>XML DSL: <code>xml!</code>, <code>xml_file!</code>, <code>register_xml_component!</code>, <code>bind</code></td>
  </tr>
  <tr>
    <td><code>yororen-ui-locale-{en, zh-CN, ar}</code></td>
    <td>Bundled JSON translation catalogs</td>
  </tr>
  <tr>
    <td><code>yororen-ui</code><br><sub><em>(meta-crate)</em></sub></td>
    <td>Re-exports the above. Most apps need only this.</td>
  </tr>
</table>

### Three-layer architecture

```text
theme JSON  ─▶  renderer (XxxRenderer)  ─▶  headless (XxxProps)  ─▶  gpui-ce
```

- **Headless** — data + control + a11y. No visual.
- **Renderer** — a per-component trait that reads the theme and produces visual divs.
- **Theme** — a single `serde_json::Value` you can swap at runtime.

The 55 component markers (<code>yororen-ui-core::renderer::markers</code>) are the keys into the global <code>RendererRegistry</code>. The default and brutalism renderers each implement all 55 trait slots.

A custom renderer only needs to implement the 55 <code>XxxRenderer</code> traits &mdash; it doesn't touch the headless layer.

---

## Installation

<details>
<summary><strong>From crates.io (recommended)</strong></summary>

```toml
[dependencies]
yororen_ui = "0.3"
```

The <code>xml</code> feature is enabled by default so <code>xml!</code> / <code>xml_file!</code> work out of the box. To opt out:

```toml
[dependencies]
yororen_ui = { version = "0.3", default-features = false }
```

To enable specific features:

```toml
[dependencies]
yororen_ui = { version = "0.3", default-features = false, features = ["xml", "brutalism"] }
```

</details>

<details>
<summary><strong>From GitHub (latest development build)</strong></summary>

```toml
[dependencies]
yororen_ui = { git = "https://github.com/MeowLynxSea/yororen-ui.git", tag = "v0.3.0" }
```

</details>

<details>
<summary><strong>From a local path (development)</strong></summary>

```toml
[dependencies]
yororen_ui = { path = "../yororen-ui" }
```

</details>

<details>
<summary><strong><code>gpui</code> dependency</strong></summary>

<code>gpui</code> is provided via the <a href="https://crates.io/crates/gpui-ce"><code>gpui-ce</code></a> crate on crates.io. Make sure your application uses a compatible version:

```toml
[dependencies]
gpui = { package = "gpui-ce", version = "0.3" }
```

In this repository, <code>gpui-ce</code> is specified in <code>Cargo.toml</code>.

</details>

---

## Requirements

- **Rust edition:** 2024
- **Platforms:** macOS, Linux, Windows (whatever <code>gpui-ce</code> supports)

---

## License

Licensed under the **Apache License, Version 2.0**. Built on top of <code>gpui</code> (Zed Industries), also Apache-2.0. See <code>LICENSE</code> and <code>NOTICE</code> for attribution.

---

## Contributing

Issues and PRs are welcome. When changing visuals:

- Include screenshots or a short recording
- Keep changes <code>rustfmt</code>-clean

---

## Wiki

See the [Yororen UI Wiki](https://github.com/MeowLynxSea/yororen-ui/wiki) for guides, recipes, and per-component references.

---

## Star History

<a href="https://star-history.dera.page/#MeowLynxSea/yororen-ui&type=date&legend=top-left">
  <img src="https://star-history.dera.page/svg?repos=MeowLynxSea/yororen-ui&type=date&legend=top-left" alt="Star History Chart">
</a>

---

## Maintained by [Yoro.ren](yoro.ren)

<p align="center">
  <img src="https://raw.githubusercontent.com/MeowLynxSea/yororen-ui/refs/heads/main/screenshots/yororen-brand.png" width="560" alt="Yororen — Maintained with care">
</p>