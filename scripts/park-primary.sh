#!/bin/sh
# Keep the primary checkout parked at origin/main so nobody works in it
# (Mandate 49): on `main`, fast-forwarded to origin/main; detached there only
# when another worktree holds `main`. It never undoes a human's own `git switch
# main`, and refuses, saying why, when the primary checkout has uncommitted
# changes, an operation in progress, or commits that are not in origin/main.
#
#   scripts/park-primary.sh              park (run from any worktree of the repo)
#   scripts/park-primary.sh --sync-only  keep an already-parked main current: fast-
#                                        forward it to origin/main, and do nothing
#                                        (silently) unless the primary checkout is on
#                                        main, clean and has no local commits. Safe
#                                        to run from every workflow check and hook.
#
# The parking was the earlier development program's `workflow park-primary`; no
# maintained program has it since 2026-11-22 (the move of development code to Susi
# Tools decides whether it returns). This file stays so that the hooks
# (.githooks/sync-primary) keep working: it asks $SUSI_WORKFLOW_BIN and
# `susi-tools` for the verb and execs the first that has it. With none,
# --sync-only is silent (it is best effort everywhere it runs) and a park says so
# and fails, as a park that could not move anything did. Nothing here needs more
# than sh and git.
set -u
# Only SUSI_WORKFLOW_BIN and susi-tools are asked (2026-11-22): no other program
# carries the verb any more, and an older build that does must not act here.
for bin in "${SUSI_WORKFLOW_BIN:-}" "$(command -v susi-tools 2>/dev/null)"; do
    [ -n "$bin" ] && [ -x "$bin" ] || continue
    if "$bin" workflow park-primary --help >/dev/null 2>&1 </dev/null; then
        exec "$bin" workflow park-primary "$@" 3>&- 4>&- 5>&- 6>&- 7>&- 8>&- 9>&-
    fi
done
case " $* " in
*" --sync-only "*) exit 0 ;;
esac
echo "park-primary: no program with \`workflow park-primary\` found (SUSI_WORKFLOW_BIN, susi-tools): not moving anything." >&2
exit 1
