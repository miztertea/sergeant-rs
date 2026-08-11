# Contract — `repo-to-icm` run 01KZQRGZE32RQ79KT82XTB9MV2

## 1. Subject repository and revision

**Subject:** `reference/sergeant-upstream` (a subtree of this outer worktree).

**Case:** vendored subtree, not a live checkout. Verified:
`reference/sergeant-upstream/.git` does not exist (`ls` fails with
"No such file or directory"). `git -C reference/sergeant-upstream
rev-parse --is-inside-work-tree` returns `true`, but per this stage's own
contract that result is a trap here, not evidence of a live checkout — it
is only reporting that the path sits inside the *outer* repository's work
tree (this outer worktree's own `.git`), not that the subject has one of
its own. The absence of `reference/sergeant-upstream/.git` is the
dispositive check.

**Revision:** `f430cfd4f90174a98adbd7abebbece6303817929`, taken verbatim
from the subject's own provenance document, `reference/UPSTREAM.md` (its
table row for `sergeant-upstream/`: "pinned at
`f430cfd4f90174a98adbd7abebbece6303817929` (main, includes merged PR #2 —
Claude background harness)"). No `git rev-parse` was run against the
subject to derive this value — there is no such object to resolve against
for a vendored subtree with no `.git` of its own.

**Resolution method used:** provenance-document read (vendored-subtree
case), not `git -C <subject> rev-parse HEAD`. Re-verification later means
re-reading `reference/UPSTREAM.md`'s table row for `sergeant-upstream/`,
not re-running a git command against the subject.

**Discrepancy check:** the Work's initiating task also states SHA
`f430cfd4f90174a98adbd7abebbece6303817929`. This agrees exactly with the
value recorded in `reference/UPSTREAM.md`. No discrepancy to record.

## 2. Scope

Everything under `reference/sergeant-upstream/` (the subtree root and all
its contents), and nothing outside it. Top-level contents at the time of
this contract: `.agents/`, `.claude/`, `.gitignore`, `AGENTS.md`,
`Dockerfile.test`, `LICENSE`, `README.md`, `bin/`, `docs/`, `mise.toml`,
`opencode.json`, `schema/`, `scripts/`, `skills/`, `templates/`, `tests/`.

## 3. Exclusions

- **VCS internals / build-dependency output.** Standing exclusion per
  `../CONTEXT.md`. Checked: no `.git/`, `node_modules/`, `target/`,
  `dist/`, or `vendor/` directory exists anywhere under
  `reference/sergeant-upstream/` (confirmed by `find`). The subtree's own
  `.gitignore` lists only `.DS_Store` and `.todos/` — neither is present
  either. This exclusion is therefore currently vacuous (nothing to
  exclude under it) but remains standing for any such artifact introduced
  later in this run's lifetime.
- **Reference/"gold" decomposition of this corpus (blindness rule,
  `../_config/run-discipline.md` §1).** The Work's task does not name any
  such directory, and none exists anywhere in this worktree — no path
  matching `reference-corpus` (or similar) was found by search of the
  full worktree. Per this stage's own contract, "if you are not told one
  exists, do not go looking for one to exclude" — recorded here only that
  none was named and none was found to exist; its contents were never
  opened. `run-discipline.md`'s canonical example (an "N2 measurement run
  against `reference/sergeant-upstream`, graded against
  `reference-corpus/`") superficially resembles this run's subject, but
  this run's task carries no measurement framing and no answer-key
  directory is present in this worktree, so that clause does not resolve
  to an actual exclusion here — recorded as a discrepancy worth a
  downstream reader's attention, not silently reconciled.
- **`reference/UPSTREAM.md` itself.** Explicit, verbatim from the Work's
  task ("exclude `reference/UPSTREAM.md` itself"). Outside the scoped
  subtree in any case (it lives at `reference/UPSTREAM.md`, a sibling of
  `reference/sergeant-upstream/`, not inside it), so this exclusion is
  belt-and-suspenders with scope itself, not an carve-out from within the
  subtree.
- **`.sergeant/`.** Explicit, verbatim from the Work's task. Also outside
  the scoped subtree (this outer worktree's own `.sergeant/`, not
  anything inside `reference/sergeant-upstream/`).
- **`AGENTS.md`.** Explicit, verbatim from the Work's task. This is this
  outer worktree's own root `AGENTS.md`, outside the scoped subtree. Note
  `reference/sergeant-upstream/AGENTS.md` (the subject's *own* AGENTS.md,
  inside the subtree) is a distinct file at a distinct path — the task's
  exclusion names the outer one specifically (no path qualifier was
  given, and the outer worktree's root `AGENTS.md` is the one a caller
  reading this outer worktree would mean by the bare name); the inner one
  remains in scope as ordinary subject content unless a downstream stage
  finds reason to think otherwise.

## 4. Output paths

Each downstream stage (`10-inventory` through `90-reconcile`) writes its
own declared artifact(s) to its own `output/` directory, per that stage's
`output/README.md`. This contract does not invent or redirect any output
path — it only confirms the convention applies to this run.

## 5. Success criteria

The Work's task ("Decompose the repository subtree
`reference/sergeant-upstream` ... into draft ICM workflows per
`.sergeant/workflows/repo-to-icm`. Scope: the subtree only; exclude
`reference/UPSTREAM.md` itself, `.sergeant/`, and `AGENTS.md`.") names a
target output shape (draft ICM workflows) and the scope/exclusions
already captured above, but no criteria narrower than "the workflow's own
outcome, chained end to end." This run therefore has no success criteria
narrower than what each stage's own `CONTEXT.md` declares as its durable
outcome, `00-contract` through `90-reconcile`.
