---
name: yororen-ui-recipes
description: End-to-end recipe patterns for end users building gpui apps with Yororen UI (yororen_ui). Use when the user asks for a complete working example (counter/todolist/file browser/toast), wants a screen layout pattern, or needs guidance composing components, modals, forms, list rendering, keyed state, virtualization, notifications, or i18n. Not for developing yororen-ui itself.
---

# Yororen UI Recipes

Prefer copying and adapting proven patterns from the demos rather than inventing new architectures.

## Primary References

Use these as "known-good" implementations:
- `demo/todolist/` (most complete: components + state + modal + form + i18n)
- `demo/counter/` (minimal bootstrap + global state + notify)
- `demo/file_browser/` (tree + context menu + empty state patterns)
- `demo/toast_notification/` (toast + NotificationCenter patterns)

## Recipe Index

### 1) Counter app (small)

Use when the user wants a minimal example with a polished UI.
Start from `demo/counter/src/main.rs` and its `counter_app` module.

### 2) Todo app (full reference)

Use when the user wants:
- Forms (`TextInput` + `ComboBox`)
- A modal editor (`Modal`)
- Derived UI state (filtering/search/category)
- i18n via `cx.t(...)`
- Correct input state patterns (avoid `.content(...)` loops)

Key files:
- `demo/todolist/src/main.rs` (bootstrap)
- `demo/todolist/src/state.rs` (global state layout)
- `demo/todolist/src/todo_app.rs` (root render + derived state)
- `demo/todolist/src/components/todo_form.rs` (form pattern)
- `demo/todolist/src/components/todo_modal.rs` (modal + one-time input init)

### 3) File browser (tree + actions)

Use when building hierarchical UIs and contextual actions.
Start from `demo/file_browser/src/main.rs` and follow the module structure.

### 4) Toasts and notifications

Use when the user wants in-app feedback.
Start from `demo/toast_notification/src/toast_demo_app.rs`.

## Composition Rules

- Prefer the "toolbar + content" and "list rows" patterns from the docs in the `yororen_ui` dependency source checkout.
- If the user has a local clone of the Yororen UI repository (including the wiki), you may also consult the wiki page `Guide-Composing-UI.md`.
- Always attach stable keys/ids to rows and to stateful inputs in lists.
- When asked for performance improvements, check for:
  - controlled input sync loops
  - missing stable keys
  - rendering huge non-virtualized lists

## Docs to Read (Dependency source, read-only)

Read from the `yororen_ui` dependency source checkout (via `cargo metadata`).

Minimum set:
- `demo/todolist/`
- `demo/file_browser/`
- `demo/toast_notification/`
