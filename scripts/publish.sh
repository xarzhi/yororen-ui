#!/usr/bin/env bash
# Publish every yororen-ui crate to crates.io in dependency order.
#
# Usage:
#   scripts/publish.sh                 # interactive: confirms before each upload
#   scripts/publish.sh --no-confirm    # CI / trusted: skip confirmations
#   scripts/publish.sh --dry-run       # package + verify, but never upload
#   scripts/publish.sh --allow-dirty   # pass --allow-dirty to cargo publish
#   scripts/publish.sh <crate-name>    # publish only one crate (used by retry)
#
# Crate package names use the cargo `name` field (underscored), NOT the
# directory name. For example `yororen_ui_core`, not `yororen-ui-core`.
#
# Why the sleep: crates.io indexes new versions asynchronously. The index
# entry for `yororen_ui_core@0.3.0` may take 10–30 s to propagate before
# downstream crates can resolve it. The 60 s sleep below is conservative.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Crate publish order, leaf → root, matching the workspace dependency graph.
PUBLISH_ORDER=(
    yororen_ui_core
    yororen_ui_default_renderer
    yororen_ui_brutalism_renderer
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
for arg in "$@"; do
    case "$arg" in
        --no-confirm) CONFIRM=0 ;;
        --dry-run)    DRY_RUN=1 ;;
        --allow-dirty) ALLOW_DIRTY=1 ;;
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
    cargo_publish "$pkg"
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
if [[ "$DRY_RUN" -eq 0 ]]; then
    if ! cargo whoami >/dev/null 2>&1; then
        echo "[publish] not logged in to crates.io; run \`cargo login <token>\` first" >&2
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
        echo "[publish] sleeping 60 s for crates.io index propagation..."
        sleep 60
    fi
done

echo
echo "[publish] done."