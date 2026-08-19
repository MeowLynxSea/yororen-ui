# `upstream/` — public-API baselines

This directory contains `cargo public-api` baselines for the
6 published crates in the workspace. They are the source of
truth for what the public surface of each crate looks like.

## Format

One `.api.txt` per published crate:

| File | Crate |
| --- | --- |
| `yororen_ui_core.api.txt` | `yororen_ui_core` (`yororen-ui-core`) |
| `yororen_ui_default_renderer.api.txt` | `yororen_ui_default_renderer` (`yororen-ui-default-renderer`) |
| `yororen_ui_brutalism_renderer.api.txt` | `yororen_ui_brutalism_renderer` (`yororen-ui-brutalism-renderer`) |
| `yororen_ui_winui_renderer.api.txt` | `yororen_ui_winui_renderer` (`yororen-ui-winui-renderer`) |
| `yororen_ui_xml.api.txt` | `yororen_ui_xml` (`yororen-ui-xml`) |
| `yororen_ui.api.txt` | `yororen_ui` (`yororen-ui` meta-crate) |

The `yororen_ui_xml_macro` proc-macro crate is **not** tracked by a
baseline: its only public surface is a handful of `proc macro`
declarations, so a baseline would add no diff signal. If it ever gains
real public items (re-exported types or functions), add it here.

Each file is the `--simplified` output of `cargo public-api` —
flat, one-public-item-per-line, no type bodies. That makes
diffs human-reviewable and easy to skim in a PR.

## When to update

Update the baseline when you intentionally ship a breaking
change to the public surface. Concretely:

1. **Run** the local baseline generation command (see below).
2. **Diff** the new baseline against the old one. Every
   removed / changed line is a breaking change.
3. **Update** `MIGRATION.md` (or `./tmp/MIGRATION_*.md` until
   the wiki process picks it up) with the migration recipe
   for every changed item.
4. **Commit** the new baseline alongside the code change
   that caused it. One commit, atomic — never leave the
   baseline out of sync with the code.
5. **Tag** the release (the next published version will
   advertise the breaking change in its CHANGELOG).

## Local baseline generation

The fastest path is `scripts/regen-public-api.sh --all`, which updates
every tracked baseline in one pass and is what the pre-commit hook and
CI both call. To regenerate the baseline for one crate by hand:

```bash
cargo public-api -p yororen_ui_winui_renderer --simplified >| upstream/yororen_ui_winui_renderer.api.txt
```

Note: the `>|` redirect (not `>`) is required when `setopt
NO_CLOBBER` is enabled in your shell — otherwise the
`public-api` regeneration silently no-ops against the existing
file and you'll think you've refreshed the baseline when you
haven't.

## CI

The `Public API Diff` job in `.github/workflows/public-api.yml`
generates every baseline via `scripts/regen-public-api.sh --all` on
every push to `main` / `master` and every PR, then runs
`git diff --exit-code -- upstream/` against the committed baseline. A
non-empty diff fails the job. The error message tells the contributor
to either fix the code or refresh `upstream/*.api.txt` via
`scripts/regen-public-api.sh --all`.

Locally, the pre-commit hook (`scripts/hooks/pre-commit`) runs the same
regeneration for any modified crate before a commit lands, so a stale
baseline fails the commit before it ever reaches CI.

## What counts as "public"?

- Every `pub fn`, `pub struct`, `pub enum`, `pub trait`,
  `pub type`, `pub const` re-exported from the crate root.
- Re-exports (`pub use foo::Bar`) — these are how
  third-party crates import items, so they count.
- Trait impls are not listed (they're implicit).

## What does NOT count?

- Items in `#[cfg(test)]` modules.
- Items behind non-default feature flags (the public-api
  binary runs with default features only).
- Doc-only items (`#[doc(hidden)]`).

## Limitations

- The public-api binary requires the crate to compile in the
  default feature configuration. Crates that fail to build
  with default features will show up as "diff = full API
  missing", which is a false positive. Always keep the default
  build green before regenerating baselines.
- `--simplified` does not surface trait implementations
  separately. If your breaking change is "this type no longer
  implements Trait X", the baseline diff won't catch it.
  Combine the public-api diff with a downstream-app smoke
  test in CI if you need that level of coverage.