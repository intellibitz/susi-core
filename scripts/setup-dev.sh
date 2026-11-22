#!/usr/bin/env bash
# One-time per clone (shared by every worktree of it): enable the repo's git
# hooks.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
git config core.hooksPath .githooks
echo "hooks: $(git config core.hooksPath)"
