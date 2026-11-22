#!/bin/sh
# Give a worktree its own cargo target, warm from a snapshot of origin/main.
#
#   scripts/seed-target.sh [DIR]               seed DIR's target/ (default: this
#                                              checkout) before anything builds
#   scripts/seed-target.sh --background [DIR]  replace a link now, copy in the
#                                              background (the git hooks)
#   scripts/seed-target.sh --refresh [--force] [--log FILE]
#                                              rebuild and republish the snapshot
#                                              (the primary watcher runs this)
#
# The seeding was the earlier development program's `workflow seed-target`; no
# maintained program has it since 2026-11-22 (the move of development code to
# Susi Tools decides whether it returns). This file asks $SUSI_WORKFLOW_BIN and
# `susi-tools` for the verb and execs the first that has it, holding none of the
# caller's descriptors (a lock on fd 9, a pipe a hook reads to its end). With
# none, it still does the one thing that cannot wait: a worktree's
# target/ that is a link to the clone's old shared target is unlinked (the
# link only), and a missing one is made, empty, so neither the cargo the
# caller runs next nor v0.21.23's `workflow start` (which links a target/ that
# does not exist) builds where a sibling's build can stand in for this
# branch's. Nothing is copied: that first build is cold. It says so and exits
# 0: the seeding never fails its caller. Nothing here needs more than sh and git.
set -u
top=$(git rev-parse --show-toplevel 2>/dev/null) || top=
# Only SUSI_WORKFLOW_BIN and susi-tools are asked (2026-11-22): no other program
# carries the verb any more, and an older build that does must not act here.
for bin in "${SUSI_WORKFLOW_BIN:-}" "$(command -v susi-tools 2>/dev/null)"; do
    [ -n "$bin" ] && [ -x "$bin" ] || continue
    if "$bin" workflow seed-target --help >/dev/null 2>&1 </dev/null; then
        exec "$bin" workflow seed-target "$@" 3>&- 4>&- 5>&- 6>&- 7>&- 8>&- 9>&-
    fi
done
echo "target: no program with \`workflow seed-target\` found (SUSI_WORKFLOW_BIN, susi-tools): nothing is copied, and the first build compiles everything." >&2
# The worktree named (the last argument that is no flag), else this one;
# --refresh has nothing to fall back on.
dir=
for arg in "$@"; do
    case "$arg" in
    --refresh) exit 0 ;;
    --log) dir=-- ;;
    -*) ;;
    *) [ "$dir" = -- ] && dir= || dir=$arg ;;
    esac
done
[ "$dir" != -- ] || dir=
[ -n "$dir" ] || dir=$top
[ -n "$dir" ] && [ -d "$dir" ] || exit 0
top=$(git -C "$dir" rev-parse --show-toplevel 2>/dev/null) || exit 0
# Never the primary checkout: its target is the one worktrees still link.
gitdir=$(git -C "$top" rev-parse --path-format=absolute --git-dir 2>/dev/null) || exit 0
common=$(git -C "$top" rev-parse --path-format=absolute --git-common-dir 2>/dev/null) || exit 0
[ "$gitdir" != "$common" ] || exit 0
t=$top/target
if [ -L "$t" ]; then
    rm -f -- "$t" 2>/dev/null
    if [ -L "$t" ]; then
        # Git for Windows' rm may refuse a junction; cmd's rmdir without /s
        # removes the junction only. The path is its own argument.
        case "$(uname -s 2>/dev/null)" in
        MSYS* | MINGW* | CYGWIN*) MSYS2_ARG_CONV_EXCL='*' cmd.exe /d /c rmdir "$(cygpath -aw "$t" 2>/dev/null || printf '%s' "$t")" >/dev/null 2>&1 ;;
        esac
    fi
    if [ -L "$t" ]; then
        echo "target: could not remove the link at $t; it still points at a shared target" >&2
        exit 0
    fi
    echo "target: replaced the link to a shared target with this worktree's own ($t)" >&2
fi
[ -e "$t" ] || mkdir "$t" 2>/dev/null
exit 0
