# Yororen UI

A reusable GPUI component library.

## Keyed State (`key(...)`)

GPUI stores many pieces of UI state (text cursor, selection, open/closed menus, toggles, etc.)
against an element identifier. In Yororen UI, any component that owns internal UI state exposes
`key(...)` as an alias for `id(...)`.

- Use `key(...)` when you want to emphasize **state identity**.
- In lists/virtualized UIs (e.g. `VirtualList`), always provide a stable key derived from your
  data model (id/uuid/path), not from call sites.

## Virtualization (`VirtualList` / `VirtualRow`)

For long, scrollable content, use Yororen UI's virtualization primitives:

- `VirtualList` (widget): a thin wrapper around `gpui::list(ListState, ...)`
- `VirtualRow` (component): a virtualization-safe row shell

`VirtualRow` responsibilities:

1) Stable row key: required, prevents state bleed when rows are recycled.
2) Row-local element namespace: isolates `Location::caller()`-based ids inside each row.
3) Row spacing/dividers belong to the shell: callers should only render content.

If row height can change (expand/collapse, async-loaded content), notify the list via
`VirtualListController.reset(...)` or `VirtualListController.splice(...)`.

## ListItem layout rule

`ListItem` is a row content container. By default it **does not stretch** child components
horizontally; children keep their intrinsic widths unless you explicitly opt into flex growth.
