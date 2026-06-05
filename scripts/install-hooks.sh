#!/usr/bin/env bash
# Install git hooks from scripts/hooks/ into .git/hooks/.
# Run once after cloning the repo, and again after pulling changes
# that added or modified hooks.
#
# Existing hooks of the same name in .git/hooks/ are overwritten
# without prompting. This is the intended behavior: scripts/hooks/
# is the source of truth, .git/hooks/ is a copy.

set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
hooks_src="${repo_root}/scripts/hooks"
hooks_dst="${repo_root}/.git/hooks"

if [ ! -d "$hooks_src" ]; then
    echo "[install-hooks] no ${hooks_src} directory; nothing to install" >&2
    exit 1
fi

installed=0
for hook in "$hooks_src"/*; do
    [ -f "$hook" ] || continue
    name="$(basename "$hook")"
    dest="${hooks_dst}/${name}"
    cp "$hook" "$dest"
    chmod +x "$dest"
    echo "[install-hooks] installed ${name}"
    installed=$((installed + 1))
done

if [ "$installed" -eq 0 ]; then
    echo "[install-hooks] no hook files found in ${hooks_src}" >&2
    exit 1
fi

echo "[install-hooks] done. ${installed} hook(s) installed."
echo "[install-hooks] to bypass a hook for a single commit, use: git commit --no-verify"
