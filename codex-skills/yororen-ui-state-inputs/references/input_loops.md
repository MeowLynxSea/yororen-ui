# Avoiding input sync loops (gpui + Yororen UI)

## The failure mode

In gpui, `render(...)` is called frequently. If you wire a text input like this:

- `text_input().content(state_value)` in `render`
- `on_change` writes `state_value = value`

you create a feedback loop. The input keeps re-syncing its content from state while the user is typing.

Common symptoms:

- typing feels laggy
- cursor jumps to end / selection resets
- high CPU while typing

## Use the right API

For `TextInput`:

- Prefer *uncontrolled* usage during typing: do not set `.content(...)`.
- Use `.set_content(...)` to set value **once** (initial load, reset/clear, modal open init).

See the TextInput documentation in the Yororen UI repository (if available), and always verify behavior in `src/component/text_input.rs` from the `yororen_ui` dependency source checkout.

## Proven pattern: one-time init flag

When opening an edit modal:

1) Copy model data into edit buffers (`edit_title`, `edit_category`, ...)
2) Set `edit_needs_init = true`
3) In modal render:
   - `when(edit_needs_init, |this| this.set_content(edit_title.clone()))`
   - in `on_change`, keep updating `edit_title`

This pattern prevents the modal from re-initializing the input on every render.

Reference implementation:

- `demo/todolist/src/components/todo_modal.rs`
