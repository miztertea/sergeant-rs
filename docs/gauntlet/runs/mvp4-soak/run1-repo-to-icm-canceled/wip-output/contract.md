# Contract — `repo-to-icm` run against `vendor/tiny-deploy-tool`

## 1. Subject repository and revision

**Subject:** `vendor/tiny-deploy-tool/` — a subtree of this outer repository
(the repository this workflow run itself executes inside).

**Case determination:** this is the **vendored-subtree case**, not the
live-checkout case. Checked directly rather than assumed:

- `vendor/tiny-deploy-tool/` contains no `.git` of its own (confirmed by
  directory listing: only `README.md`, `UPSTREAM.md`, `docs/`, `scripts/`,
  `tests/`).
- `git -C vendor/tiny-deploy-tool rev-parse --is-inside-work-tree` returns
  `true` — but per this stage's own contract, that is expected and does
  **not** indicate a live checkout: it reports that the path sits inside
  the *outer* repository's work tree, not that the path has its own `.git`.
  `git -C vendor/tiny-deploy-tool rev-parse HEAD` would resolve to the
  outer repository's own moving `HEAD`, which is not this subject's
  revision, and was not used for that reason.

**Pinned revision (resolution method: read the subtree's own provenance
document, `vendor/tiny-deploy-tool/UPSTREAM.md`, verbatim — not derived
from any `git rev-parse` inside the subtree, since there is no such git
object to resolve there):**

```
90aa2c01bd647020383d91ad662c294fae5c2aa3
```

Per `UPSTREAM.md`: "Not vendored from an external upstream — authored
directly as a scratch fixture for the MVP-4 real-Claude soak (issue #19)
... Pinned revision: `90aa2c01bd647020383d91ad662c294fae5c2aa3` — the
commit in this outer repository that added this subtree
(`vendor/tiny-deploy-tool/`) in its initial, complete form."

**Cross-check (not the resolution method, corroboration only):** the outer
repository's own commit log for this path shows
`90aa2c0 soak fixture: add tiny-deploy-tool vendored subtree for
repo-to-icm MVP-4 soak (#19)` as the commit that introduced the path, and
`2f3f288 soak fixture: record tiny-deploy-tool provenance (pinned to
90aa2c01bd647020383d91ad662c294fae5c2aa3)` as a follow-up that recorded the
same SHA in `UPSTREAM.md`. The full SHA in the log (`90aa2c01bd647020383d91ad662c294fae5c2aa3`)
agrees exactly with `UPSTREAM.md`'s recorded value — no discrepancy to
record.

## 2. Scope

Everything under `vendor/tiny-deploy-tool/`, i.e. all five files present in
the subtree at the pinned revision:

- `vendor/tiny-deploy-tool/README.md`
- `vendor/tiny-deploy-tool/UPSTREAM.md`
- `vendor/tiny-deploy-tool/docs/runbook.md`
- `vendor/tiny-deploy-tool/scripts/deploy.sh`
- `vendor/tiny-deploy-tool/tests/test_deploy.sh`

This is the entirety of the subtree's working-tree contents as it currently
sits (the vendored-subtree case: the working tree itself *is* the pinned
snapshot, per this stage's own instructions), confirmed by a recursive file
listing (`find vendor/tiny-deploy-tool -type f`) returning exactly these
five paths, no more.

## 3. Exclusions

None narrow the scope above — this is a small, wholly in-scope subtree with
no VCS internals, build/dependency output, or vendored lock caches inside
it (`vendor/tiny-deploy-tool/.git` does not exist; there is no `target/`,
`node_modules/`, `dist/`, or similar generated-output directory inside the
subtree itself — `dist/` is referenced by the runbook/script/test as an
artifact *produced at deploy time in whatever repository consumes this
tool*, not a directory present in this subject).

**Reference/gold decomposition:** this run's own initiating task states
explicitly that no reference or gold decomposition exists for this subject
and that none should be looked for. Consistent with that: this is **not** a
measurement run (`../_config/run-discipline.md` §1's blindness rule is
therefore vacuous for this run, though its underlying citation discipline —
cite only the target repository under decomposition — still holds
throughout). No directory holding such a reference was named in the
initiating task, and none was gone looking for, per this stage's own
instruction not to hunt for one unless told it exists. This outer
repository does contain `reference/` and `reference-corpus/` directories at
its root, but neither is inside `vendor/tiny-deploy-tool/` and neither was
named as a gold decomposition of *this* subject by the initiating task —
they are simply outside scope for this run, not excluded-with-reason
within it.

**Other exclusions named by the Work's task:** none beyond the standard
VCS/build-output exclusions already covered above (the initiating task
states "no exclusions beyond the standard VCS/build-output ones").

## 4. Output paths

Each downstream stage of this `repo-to-icm` run (`10-inventory` through
`90-reconcile`) writes its declared artifact(s) to its own
`.sergeant/workflows/repo-to-icm/<stage>/output/` directory in this run's
materialized worktree, per that stage's own `output/README.md`. This
contract does not invent any new output location — it restates that the
existing per-stage convention applies unchanged to this run.

## 5. Success criteria

The initiating task names no success criteria narrower than this
workflow's own declared outcome, chained end to end: this run is complete
once every stage from `10-inventory` through `90-reconcile` has met the
durable outcome its own `CONTEXT.md` declares, culminating in
`90-reconcile` adjudicating `80-adversarial-review`'s findings, emitting
the measurement package and grammar-pressure report, and running
`scripts/finalize.py` to close the run. No narrower or additional
criterion was given.
