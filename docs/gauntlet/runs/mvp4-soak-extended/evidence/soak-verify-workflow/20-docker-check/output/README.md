# Output — `20-docker-check`

Layer 4 (per-run artifact). This directory is empty in the authored tree
apart from this file; a run of this stage writes `check.txt` here,
directly by the stage's own container (`kind = "execute"`, `workflow.toml`)
rather than by an actor.

**Expected artifact:** `check.txt` — the line count of `notes/status.md` as
seen inside the container, plus a trailing `checked-ok` line, proving the
container could read the actor stage's edit from `10-touch` and write back
into the mounted worktree.

**Disposition:** `evidence`

Per-run mechanical evidence for `30-confirm` to read and act on with its
own judgment — sergeant itself only reads this stage's container exit code
to decide the stage outcome (§11.2), never this file's content.
