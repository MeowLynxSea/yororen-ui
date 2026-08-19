#!/usr/bin/env bash
# Regenerate cargo public-api baselines for one or more published
# crates in this workspace. Used by both the pre-commit hook
# (scripts/hooks/pre-commit) and manual regeneration.
#
# Usage:
#   scripts/regen-public-api.sh --all                # regen every published crate
#   scripts/regen-public-api.sh yororen_ui_core ...  # regen specific crates
#
# Crate package names are the cargo names (e.g. `yororen_ui_core`),
# not the directory names (e.g. `yororen-ui-core`).
#
# On failure the script aborts with `set -e`, leaving the existing
# baseline file untouched (writes go to a temp file and are renamed
# atomically on success).

set -euo pipefail

if ! command -v cargo-public-api >/dev/null 2>&1; then
    echo "[regen-public-api] cargo-public-api is not installed; skipping" >&2
    echo "[regen-public-api] install with: cargo install cargo-public-api" >&2
    exit 0
fi

# Crate package name -> baseline filename. The published
# crates are listed here. Phase J crates (yororen-ui-{virtual,form,command,table})
# will be added when they ship. `yororen_ui_xml_macro` is intentionally
# absent: as a proc-macro crate its only public surface is a handful of
# `proc macro` declarations, so a baseline adds no signal.
declare -A pkg_to_baseline=(
    [yororen_ui_core]="upstream/yororen_ui_core.api.txt"
    [yororen_ui]="upstream/yororen_ui.api.txt"
    [yororen_ui_default_renderer]="upstream/yororen_ui_default_renderer.api.txt"
    [yororen_ui_brutalism_renderer]="upstream/yororen_ui_brutalism_renderer.api.txt"
    [yororen_ui_winui_renderer]="upstream/yororen_ui_winui_renderer.api.txt"
    [yororen_ui_xml]="upstream/yororen_ui_xml.api.txt"
)

if [ "$#" -eq 0 ] || [ "$1" = "--all" ]; then
    targets=("${!pkg_to_baseline[@]}")
else
    targets=("$@")
fi

for pkg in "${targets[@]}"; do
    if [ -z "${pkg_to_baseline[$pkg]:-}" ]; then
        echo "[regen-public-api] unknown crate package: $pkg (skip)" >&2
        continue
    fi
    baseline="${pkg_to_baseline[$pkg]}"
    echo "[regen-public-api] regenerating $pkg -> $baseline"
    tmp="$(mktemp)"
    trap "rm -f '$tmp'" EXIT
    if ! cargo public-api -p "$pkg" --simplified > "$tmp"; then
        rm -f "$tmp"
        echo "[regen-public-api] FAILED to regenerate $baseline; existing baseline left untouched" >&2
        exit 1
    fi
    mv "$tmp" "$baseline"
    trap - EXIT
done
