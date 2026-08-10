# Output — `50-test-in-disposable-copy`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory is empty in the authored tree; a run of this stage writes its artifact(s) here in the materialized work surface, Git-tracked on the Work branch and reviewable in the diff like any other change.

**Expected artifact:** a record of — The external skill's source is pinned or locked where the installer supports it (folded in from the demoted `40-pin-source` stage, N1 adjudication A4); the skill is tested in a disposable repository or worktree before broad installation.

**Disposition:** `promote`

This is a workflow deliverable: it survives into the merge under the finalize policy (`docs/icm/convention.md` §1a open question 1 — "silence promotes nothing"; a `promote` artifact is kept explicitly).
