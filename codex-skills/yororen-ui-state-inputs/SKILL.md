---
name: yororen-ui-state-inputs
description: State management and input/form best practices for end users building Yororen UI apps with gpui. Use when implementing TextInput/TextArea/SearchInput/ComboBox/Form/Modal, when wiring on_change/on_submit handlers, or when diagnosing typing lag, cursor jumps, or render loops caused by controlled inputs. Not for developing yororen-ui itself.
---

# Yororen UI State + Inputs

Implement UI state safely and efficiently in gpui/Yororen UI apps, with special focus on avoiding controlled-input feedback loops.

## Mental Model (gpui + Yororen UI)

- gpui rebuilds UI trees often; your `render(...)` must be fast and side-effect-free.
- Yororen UI inputs (e.g., `TextInput`) own internal cursor/selection state via keyed state.
- Your app state should store *business data* and *small UI flags*, not the entire UI tree.

## State Pattern (recommended)

Use a global state struct with `Arc<Mutex<T>>` fields and implement `Global`.

When multiple components must trigger a root re-render:
- Store the root component's `EntityId` into global state during `Root::new(cx)`.
- After state mutation, call `cx.notify(entity_id)`.

Reference implementation:
- `demo/todolist/src/state.rs`
- `demo/todolist/src/todo_app.rs`

## Input Rule: Avoid Render-Driven Feedback Loops

### The problem

If you do this:
- In `render`, set `text_input().content(state_value)`
- In `on_change`, write back `state_value = value`

...you create a loop: typing updates state -> render updates content -> input syncs content -> more updates. Symptoms include typing lag, cursor jumps, and high CPU.

### The rule

- Prefer **uncontrolled inputs**: do not set `.content(...)` during normal typing.
- Use `.on_change(...)` to update your state.
- When you need to programmatically set the input (initial load, reset, "open edit modal"), use **one-time** set: `.set_content(...)`.

Concrete reference:
- In the dependency source checkout, read `Component-TextInput.md` if present.
- Always verify behavior in `src/component/text_input.rs`.

### Patterns that work

1) New-item form (uncontrolled)
- `text_input("new-todo").on_change(...)`
- On submit/click: mutate data, clear the buffer in state, and optionally clear the input using `.set_content("")` if the component supports it in your version.

2) Edit modal (one-time initialization)
- Maintain `edit_needs_init: bool` in state.
- When opening the modal, set buffers (`edit_title`, `edit_category`) and set `edit_needs_init = true`.
- In modal render, do:
  - `when(edit_needs_init, |this| this.set_content(edit_title.clone()))`
  - in `on_change`, update `edit_title`.

Reference implementation:
- `demo/todolist/src/components/todo_modal.rs` (uses `edit_needs_init` + `.set_content(...)`)

## Mutex + Lock Ordering

- Keep lock scopes minimal.
- When you need multiple values, read/clone them first, then drop locks, then do work.
- Never hold two locks longer than necessary; follow a consistent ordering if unavoidable.

See:
- `demo/todolist/src/components/todo_modal.rs` (save handler explicitly clones then updates)

## Docs to Read (Dependency source, read-only)

Read docs from the `yororen_ui` dependency source checkout (via `cargo metadata`).

Minimum set:
- `src/component/text_input.rs` (authoritative for `.content()` / `.set_content()` behavior)
- `demo/todolist/src/components/todo_modal.rs` (one-time init flag pattern)
