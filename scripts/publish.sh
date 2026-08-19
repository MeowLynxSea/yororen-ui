#!/usr/bin/env bash
# Publish every yororen-ui crate to crates.io in dependency order.
#
# Usage:
#   scripts/publish.sh                       # interactive: confirms before each upload
#   scripts/publish.sh --no-confirm            # CI / trusted: skip confirmations
#   scripts/publish.sh --dry-run               # package + verify, but never upload
#   scripts/publish.sh --allow-dirty           # pass --allow-dirty to cargo publish
#   scripts/publish.sh --wait 60               # sleep 60s between publishes (default 10)
#   scripts/publish.sh <crate-name>            # publish only one crate (used by retry)
#
# Crate package names use the cargo `name` field (underscored), NOT the
# directory name. For example `yororen_ui_core`, not `yororen-ui-core`.
#
# Why the sleep: crates.io indexes new versions asynchronously. The index
# entry for `yororen_ui_core@0.3.0` may take 10–30 s to propagate before
# downstream crates can resolve it. Set to 10 s by default; bump to 60
# (or use `--wait N`) if you observe "no matching package" failures
# between consecutive publishes.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Crate publish order, leaf → root, matching the workspace dependency graph.
PUBLISH_ORDER=(
    yororen_ui_core
    yororen_ui_default_renderer
    yororen_ui_brutalism_renderer
    yororen_ui_winui_renderer
    yororen_ui_locale_en
    yororen_ui_locale_zh_cn
    yororen_ui_locale_ar
    yororen_ui_xml
    yororen_ui_xml_macro
    yororen_ui
)

# Parse flags.
CONFIRM=1
DRY_RUN=0
ALLOW_DIRTY=0
SINGLE_CRATE=""
WAIT_SECS=10
for arg in "$@"; do
    case "$arg" in
        --no-confirm) CONFIRM=0 ;;
        --dry-run)    DRY_RUN=1 ;;
        --allow-dirty) ALLOW_DIRTY=1 ;;
        --wait)
            shift
            WAIT_SECS="${1:-10}"
            ;;
        --wait=*)
            WAIT_SECS="${arg#--wait=}"
            ;;
        --help|-h)
            sed -n '2,18p' "$0"
            exit 0
            ;;
        -*)
            echo "[publish] unknown flag: $arg" >&2
            exit 2
            ;;
        *)
            SINGLE_CRATE="$arg"
            ;;
    esac
done

# Build the cargo publish invocation as an array so quoting is safe.
cargo_publish() {
    local pkg="$1"
    local cmd=(cargo publish -p "$pkg")
    if [[ "$DRY_RUN" -eq 1 ]]; then
        cmd+=(--dry-run)
    fi
    if [[ "$ALLOW_DIRTY" -eq 1 ]]; then
        cmd+=(--allow-dirty)
    fi
    "${cmd[@]}"
}

confirm() {
    local pkg="$1"
    if [[ "$CONFIRM" -eq 0 ]]; then
        return 0
    fi
    local answer
    read -r -p "[publish] upload $pkg now? [y/N] " answer
    [[ "$answer" =~ ^[Yy]$ ]]
}

publish_one() {
    local pkg="$1"
    echo
    echo "============================================================"
    echo "[publish] $pkg"
    echo "============================================================"
    # Run cargo publish; capture stderr so we can detect the "already
    # exists" case and skip rather than abort. Without this guard, the
    # second invocation aborts on the first crate that is already on
    # crates.io (e.g. after a partial successful run).
    local log
    log="$(mktemp)"
    trap "rm -f '$log'" EXIT
    if cargo_publish "$pkg" 2>&1 | tee "$log"; then
        trap - EXIT
        return 0
    fi
    trap - EXIT
    if grep -q 'already exists on crates.io index' "$log"; then
        # Extract "crate@version" from the error line for a clearer log.
        local existing
        existing="$(grep -oE '[a-zA-Z0-9_-]+@[0-9]+\.[0-9]+\.[0-9]+' "$log" | head -1)"
        echo "[publish] $existing already on crates.io — skipping"
        return 0
    fi
    # Real failure: surface it.
    return 1
}

# Resolve which crates to publish.
if [[ -n "$SINGLE_CRATE" ]]; then
    targets=("$SINGLE_CRATE")
else
    targets=("${PUBLISH_ORDER[@]}")
fi

# Pre-flight: verify every target is actually in the workspace.
missing=()
for pkg in "${targets[@]}"; do
    if ! cargo metadata --no-deps --format-version=1 \
            | python3 -c "import json,sys; data=json.load(sys.stdin); sys.exit(0 if any(p['name']=='$pkg' for p in data['packages']) else 1)" \
            >/dev/null 2>&1; then
        missing+=("$pkg")
    fi
done
if [[ ${#missing[@]} -gt 0 ]]; then
    echo "[publish] the following crates are not in the workspace: ${missing[*]}" >&2
    exit 1
fi

# Pre-flight: confirm logged in (skip for --dry-run since no upload happens).
# Note: as of cargo 1.92 the built-in `cargo whoami` subcommand was
# removed; the token still lives at ~/.cargo/credentials.toml after a
# successful `cargo login`, so we just check for that file directly.
if [[ "$DRY_RUN" -eq 0 ]]; then
    cred_file="${CARGO_HOME:-$HOME/.cargo}/credentials.toml"
    if [[ ! -f "$cred_file" ]]; then
        echo "[publish] no credentials at $cred_file; run \`cargo login <token>\` first" >&2
        exit 1
    fi
fi

# Publish.
for pkg in "${targets[@]}"; do
    if ! confirm "$pkg"; then
        echo "[publish] skipping $pkg (user declined)"
        continue
    fi
    publish_one "$pkg"
    # Wait for crates.io index to update before next upload.
    if [[ "$DRY_RUN" -eq 0 ]]; then
        echo "[publish] sleeping ${WAIT_SECS}s for crates.io index propagation..."
        sleep "$WAIT_SECS"
    fi
done

echo
echo "[publish] done."