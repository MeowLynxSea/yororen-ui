---
name: yororen-ui-app-core
description: App bootstrap and core architecture for end users building a gpui desktop app with Yororen UI (yororen_ui). Use when generating or refactoring main.rs, window setup, global theme, global i18n, assets (UiAsset/CompositeAssetSource), or when creating a new Yororen UI app crate. Not for developing yororen-ui itself.
---

# Yororen UI App Core

Build the foundation of a Yororen UI app: application bootstrap, global registrations, window creation, and a maintainable module layout.

## Requirements to Enforce

- Use **Rust edition 2024** (match Yororen UI and the demo apps).
- Pin **gpui** to a single git revision across your whole dependency graph.

## Canonical Bootstrap (copy this shape)

Follow this order:
1) `Application::new().with_assets(UiAsset)`
2) `component::init(cx)`
3) `cx.set_global(GlobalTheme::new(cx.window_appearance()))`
4) `cx.set_global(I18n::with_embedded(Locale::new("en")?))` (or demo-specific loader)
5) `cx.set_global(AppState::default())`
6) `cx.open_window(options, |_, cx| cx.new(|cx| Root::new(cx)))`

Use the demos as the ground truth for structure:
- `demo/counter/src/main.rs`
- `demo/todolist/src/main.rs`
- `demo/file_browser/src/main.rs`

## Dependency Setup

If the user's app does not include Yororen UI yet, add it first.

Preferred setup (GitHub git + stable tag):

```toml
[dependencies]
yororen_ui = { git = "https://github.com/MeowLynxSea/yororen-ui.git", tag = "v0.1.0" }
```

Notes:

- Prefer `tag = "..."` for reproducible builds.
- Avoid local `path = "../yororen-ui"` dependencies for end users unless they explicitly want to track local, potentially-unstable changes.

Then run a build once so Cargo fetches the dependency source checkout (needed for reading demos/docs from the dependency).

### New project rule (important)

When scaffolding a brand-new app:

- Pin `yororen_ui` by **tag** (stable API snapshot).
- Pin `gpui` by **rev** (git commit), and keep it consistent with the `gpui` rev used by `yororen_ui`.

## Pinning `gpui` (required)

Yororen UI depends on `gpui` via a git dependency pinned to a specific `rev`.
If your app pulls a different `gpui` revision, you will likely hit errors like:

- "multiple different versions of crate `gpui` in the dependency graph"
- trait/type mismatch errors

Do this in the end-user app repo:

1) Read the pinned `gpui` revision from the `yororen_ui` dependency source checkout
- Use `cargo metadata --format-version 1` to locate the `yororen_ui` checkout path.
- Open `Cargo.toml` in that checkout and copy its `gpui = { git = "...", rev = "..." }`.

2) Pin `gpui` to the same `rev` in your application workspace

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "<same rev as yororen_ui>" }
```

Prefer this over local path hacks; keep the graph single-version and reproducible.

## Project Structure (recommended)

- `main.rs`: bootstrap only (assets/theme/i18n/state/window)
- `state.rs`: global state (`Arc<Mutex<...>>`) + `Global`
- `*_app.rs`: root component implementing `Render`
- `components/`: reusable UI components (pure UI)
- `model/` or `domain/`: app logic + data types (no gpui dependencies)

## Rules

- Keep `main.rs` minimal; avoid business logic there.
- Store a notify target `EntityId` in global state if other components need to trigger a root re-render (see `demo/todolist/src/todo_app.rs`).
- Keep lock scopes short; never hold a lock while doing expensive computation or building a huge UI subtree.

## Docs to Read (Dependency source, read-only)

Do not assume the user has the repo wiki checked out.
Read from the `yororen_ui` dependency source checkout (via `cargo metadata`).

Recommended entry points in that checkout:
- `README.md`
- `demo/counter/src/main.rs`
- `demo/todolist/src/main.rs`
