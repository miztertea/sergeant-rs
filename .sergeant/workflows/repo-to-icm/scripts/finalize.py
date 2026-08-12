#!/usr/bin/env python3
"""finalize.py — repo-to-icm's own D9 disposition finalize step.

R-MVP1-2 (`docs/gauntlet/contracts/MVP-1.md`): promote/finalize EXECUTION is
workflow content invoking a *shared, deterministic* finalize helper — the
engine learns no output vocabulary, and no workflow keeps a private fork of
the disposition logic. That shared helper now lives at `.sergeant/lib/
finalize.py`; this file is repo-to-icm's own thin closing-stage wrapper
around it (this workflow was the helper's first consumer, before the
generalization — see the shared module's own docstring for the full
disposition-format and evidence-preservation-guard documentation).

Kept at this path, with this exact CLI, so `90-reconcile`'s existing
instructions (`../90-reconcile/CONTEXT.md`, `references/
reconciliation-method.md`) and `scripts/test-finalize-evidence-guard.py`
need no change: `python3 .sergeant/workflows/repo-to-icm/scripts/
finalize.py [--dry-run]` still finalizes *this* workflow's own tree, by
forwarding to the shared helper with this workflow's root supplied
explicitly (the shared helper has no workflow of its own to default to —
every caller must name its root).

USAGE
-----
    python3 finalize.py [--dry-run] [WORKFLOW_ROOT]

WORKFLOW_ROOT defaults to this script's own workflow directory (the parent
of `scripts/`) — the ordinary case when invoked by `90-reconcile` in the
materialized run worktree. An explicit argument is accepted for review or
testing against a copy, exactly as before the generalization.
"""

import os
import subprocess
import sys

_SHARED = os.path.join(
    os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "lib", "finalize.py"
)


def main():
    args = sys.argv[1:]
    positional = [a for a in args if a != "--dry-run"]

    if len(positional) > 1:
        print("usage: finalize.py [--dry-run] [WORKFLOW_ROOT]", file=sys.stderr)
        return 2

    if positional:
        forwarded = args
    else:
        # No explicit root: pin this workflow's own root, matching this
        # wrapper's pre-generalization default exactly (the shared helper
        # itself takes no default — it has no workflow of its own).
        own_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        forwarded = args + [own_root]

    if not os.path.isfile(_SHARED):
        print(f"error: shared finalize helper not found at {_SHARED}", file=sys.stderr)
        return 2

    return subprocess.call([sys.executable, _SHARED, *forwarded])


if __name__ == "__main__":
    sys.exit(main())
