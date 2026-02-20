---
name: yororen-ui-user
description: High-quality app code generation for end users building Rust desktop GUIs with gpui + Yororen UI (yororen_ui). Use when a user asks to build or modify an application using Yororen UI/gpui (e.g., "build a beautiful counter with yororen ui", "add a modal form", "add i18n", "use TextInput/SearchInput/ComboBox"), or when working in a Rust project that depends on yororen_ui. Not for contributing to yororen-ui itself.
---

# Yororen UI (End-User)

Generate application code that uses Yororen UI correctly (structure, state, inputs, theming, i18n) and avoids common gpui pitfalls (especially controlled-input feedback loops).

If the user is editing the Yororen UI library itself, stop and ask for an app repository instead.

## Workflow Decision Tree

1) Determine intent
- App bootstrap, assets, theme, i18n init, window creation: use `$yororen-ui-app-core`.
- Any inputs, forms, modals, editing buffers, typing lag, cursor jumps: use `$yororen-ui-state-inputs`.
- "Show me an example" / "copy a working pattern": use `$yororen-ui-recipes`.

2) Determine codebase context
- If the repository has an app crate that depends on `yororen_ui`: proceed.
- If the repository appears to be the `yororen-ui` library itself: do not proceed.

2.5) Ensure the dependency exists
- If the app does not depend on `yororen_ui` yet, add it to `Cargo.toml` first.
- Prefer a **stable git tag** (reproducible) rather than a branch.
- Avoid `path = "../yororen-ui"` style local dependencies for end users unless the user explicitly wants to develop against a local checkout.

2.6) Enforce toolchain + core dependency constraints
- Use **Rust edition 2024** for the app crate(s).
- Use **gpui-ce** (gpui community edition) from crates.io with the same version as `yororen_ui`.

2.7) New project rule
- For a brand-new project: pin `yororen_ui` by **tag**, and use gpui-ce version "0.3".

3) Always follow these safety rules
- Avoid render-driven feedback loops in inputs.
- Provide stable identity (`.key(...)` / `.id(...)`) for stateful components in lists and virtualized UIs.
- Keep mutex lock scopes short; clone data out before heavy work.

## Docs to Read (Dependency source, read-only)

Do not assume the user has a local `yororen-ui.wiki` checkout.
Most end users consume Yororen UI from crates.io:

```toml
[dependencies]
yororen_ui = "0.2"
```

Or from GitHub:

```toml
[dependencies]
yororen_ui = { git = "https://github.com/MeowLynxSea/yororen-ui.git", tag = "v0.2.0" }
```

So, read docs from the dependency's *source checkout* that Cargo already fetched:

1) Locate the `yororen_ui` source path
- Run `cargo metadata --format-version 1` in the app repo.
- Find the `yororen_ui` package and its `manifest_path`.

2) Read the project docs from that checkout (treat as read-only)
- `README.md` (quick start + boot sequence)
- `demo/todolist/` (end-to-end patterns)
- `src/component/text_input.rs` (input state + sync behavior)

If the dependency source is unavailable (offline / not fetched yet), fall back to the reference summaries under this skill's `references/` directories (and sibling skills' references).

## Installation (for the human user)

Point the user at the installation guide in the Yororen UI repo wiki (if they have the repo locally): `Agent-Skills.md`.

Otherwise provide short instructions:
- Locate Codex home (commonly `~/.codex`).
- Copy the skill folders into `~/.codex/skills/`.
- Restart Codex (or refresh skills) and invoke with `$yororen-ui-user` if auto-trigger does not happen.

## Output Standards

When generating code:
- Prefer complete, compilable modules over scattered snippets.
- Match patterns from the demos (especially `demo/todolist`).
- Keep UI logic inside gpui `Render` components; keep domain models UI-free.
- Explain non-obvious gpui/Yororen UI decisions briefly and concretely.
