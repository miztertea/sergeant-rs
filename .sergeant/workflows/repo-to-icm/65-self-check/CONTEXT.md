# 65-self-check: this workflow's own structural self-check, mechanically

## Inputs

| File | Layer | Why |
|---|---|---|
| ../scripts/validate-structure.py | L3 | the validator this stage's container runs, unmodified, against this workflow's own admitted tree |

## Purpose

This is a `kind = "execute"` stage (`workflow.toml`), not an actor turn —
sergeant launches a pinned container directly, no harness/model involved,
per N4 (`docs/gauntlet/contracts/N4.md` §11.2/§12.3). Its job is exactly
what `70-lint`'s own "How to do it" step 6 used to ask an actor to do by
hand: run `../scripts/validate-structure.py` with no path argument (the
admitted-mode self-check, validating this workflow's own tree as it sits
in the current run's worktree) and record the result. Moving it here
instead of leaving it inside `70-lint`'s actor turn is the point: whether
this workflow's own structure is clean is a **mechanical**, deterministic
question with one right answer per run — exactly the kind of step N4's
adjudication (2026-08-12) asks the first real execute stage to
demonstrate, and exactly the kind of judgment an actor turn should not be
spending tokens re-deriving every run.

## What must become true here (durable outcome)

`output/self-check-result.txt` exists in the materialized worktree,
holding the container's captured stdout+stderr verbatim: the validator's
PASS/FAIL result for this workflow's own tree, as of this run's worktree
contents.

The stage's own completion is mechanical (§11.2): exit 0 (validator PASS)
completes the stage, exit 1 (validator FAIL) fails it. Sergeant reads
only the container's exit code to decide the stage outcome — it never
parses `output/self-check-result.txt` to decide anything itself. That
file exists for the next actor stage, `70-lint`, to read and act on with
its own judgment (`docs/icm/convention.md` §5) — this is N4's
actor → execute → actor proof shape: the execute stage's output evidence
is available to the following actor without sergeant interpreting it.

A FAIL here is real signal, not a workflow bug: it means this run's own
worktree — after `40-classify` has just written a fresh
`classifications.ndjson` into it — has a structural defect the validator
can catch mechanically (front matter, stage order, Inputs tables,
`@@` references, path traversal, engine-gap record completeness). That is
exactly the check `70-lint`'s "How to do it" step 6 already asked a human
actor to run by hand; `70-lint` still owns the judgment about what a FAIL
here means for the rest of the run (repair it, or carry it forward as
substantive signal for `80-adversarial-review`/`90-reconcile`) — this
stage only owns running the check and handing the result forward
mechanically.

## The pinned container (`workflow.toml`)

- **Image:** `python:3.13` (not `-slim`) — chosen because
  `validate-structure.py`'s `[S15]` check (admitted mode, and this
  workflow ships its own `scripts/finalize.py`) shells out to `git` inside
  a disposable sandbox directory, and this stage's `network = "none"`
  policy (§16.7 — the only legal value in this schema) rules out
  installing anything at run time. `python:3.13` ships both a `git` on
  `PATH` and the `tomllib` stdlib module the validator needs (Python
  3.11+) with no extra install step, so the container is fully offline-
  capable and needs no custom-built image. Measured warm (image already
  pulled) runtime on Cerberus, 2026-08-12: **0.64s** for a full PASS run
  including the S15 git-sandbox round trip — well inside "cheap, fast,
  runnable constantly" (N4 adjudication, 2026-08-12). `python:3.13-slim`
  was measured first and is smaller (178MB vs 1.62GB) but has no `git`,
  which makes `[S15]` fail closed with "git not found on PATH" under
  `network = "none"` — ruled out for that reason, not for size.
- **Command:** runs the validator with no path argument (admitted-mode
  self-check) and redirects its combined stdout/stderr into this stage's
  own `output/self-check-result.txt` inside the mounted worktree; the
  shell's exit status is the validator's own exit code (redirection does
  not change `$?`), which is what sergeant reads to decide the stage
  outcome.
- **Workspace access:** `read_write` — the container must write the
  output artifact into the worktree.
- **Network:** `none` — the validator never needs it (S15's `git` sandbox
  is entirely local), and this schema accepts no other value.
