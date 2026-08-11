# Contract — repo-to-icm run 01KZQ32J2BAD4P8WJA9SWXRMZ9

## 1. Subject repository and revision

**Subject:** `reference/sergeant-upstream` — a subtree of this outer
repository, named explicitly in the Work's initiating task:

> Decompose the repository subtree reference/sergeant-upstream — pinned
> per reference/UPSTREAM.md at upstream SHA
> f430cfd4f90174a98adbd7abebbece6303817929 — into draft ICM workflows per
> .sergeant/workflows/repo-to-icm. Scope: the subtree only; exclude
> reference/UPSTREAM.md itself, .sergeant/, and AGENTS.md.

**Case determination (checked before resolving anything):**

```text
$ git -C reference/sergeant-upstream rev-parse --is-inside-work-tree
true
$ ls reference/sergeant-upstream/.git
ls: cannot access 'reference/sergeant-upstream/.git': No such file or directory
$ git -C reference/sergeant-upstream rev-parse HEAD
e7ef97cc465bc38ce713766fe2258dcf1fb53930
$ git rev-parse HEAD   # run at the outer worktree root
e7ef97cc465bc38ce713766fe2258dcf1fb53930
```

`reference/sergeant-upstream` has no `.git` of its own, and the SHA its
`rev-parse HEAD` reports is identical to the *outer* worktree's own HEAD —
exactly the false signal `00-contract/CONTEXT.md` warns about. This is the
**vendored-subtree case**: an ordinary tracked directory inside this outer
repository, not a live checkout. Its revision is not something to
(re)derive with `git rev-parse` inside it; there is no such object to
resolve against.

**Provenance document used:** `reference/UPSTREAM.md`, its
`sergeant-upstream/` row:

| Item | Source | Pinned at | Date vendored |
|---|---|---|---|
| `sergeant-upstream/` | https://github.com/miztertea/sergeant (fork of https://github.com/callmeradical/sergeant) | `f430cfd4f90174a98adbd7abebbece6303817929` (main, includes merged PR #2 — Claude background harness) | 2026-08-08 |

**Resolved revision:** `f430cfd4f90174a98adbd7abebbece6303817929` (40 hex
characters — a full SHA), taken verbatim from `reference/UPSTREAM.md`.

**Resolution method:** vendored-subtree provenance-document lookup (read
`reference/UPSTREAM.md`'s row for this subject), *not* `git -C <subject>
rev-parse`. A later re-verification of this fact must re-read the same row
in `reference/UPSTREAM.md`, not run `git` inside `reference/sergeant-upstream`.

**Discrepancy check:** the Work's task text claims the same SHA,
`f430cfd4f90174a98adbd7abebbece6303817929`. The resolved value (from
`reference/UPSTREAM.md`, the authoritative provenance record for a
vendored subtree) agrees with the task's claim exactly — no discrepancy to
record.

## 2. Scope

Everything under `reference/sergeant-upstream/` (the subtree root and all
its contents), per the task's explicit "Scope: the subtree only" —
narrowed only by the exclusions below. Nothing outside
`reference/sergeant-upstream/` is in scope for `10-inventory` to enumerate.

## 3. Exclusions

| Path | Reason |
|---|---|
| `reference/sergeant-upstream/bin/__pycache__/` (e.g. `sgt-callbackcpython-312.pyc`) | Build/interpreter output, not authored procedural content. |
| `reference/UPSTREAM.md` | Explicitly named for exclusion by the Work's task. (Also outside the in-scope subtree, so redundant with §2 — recorded here verbatim per the task's own wording.) |
| `.sergeant/` | Explicitly named for exclusion by the Work's task. (Outside the in-scope subtree; this is the outer repo's own Sergeant workflow tree, including this very run's artifacts — not part of the subject being decomposed.) |
| `AGENTS.md` | Explicitly named for exclusion by the Work's task. (Outside the in-scope subtree; this is the outer repo's own root-level agent instructions file, not part of the subject.) |
| VCS internals (`.git/`) | Not authored procedural content. None found inside the subject subtree itself (`reference/sergeant-upstream` has no `.git` of its own — see §1); the outer repository's own `.git` is outside scope regardless. |
| Other build/dependency output (`target/`, `node_modules/`, `dist/`, vendored lock caches) | Not authored procedural content. Checked for inside the subject subtree — none found beyond the `__pycache__` entry above. |
| A reference/"gold" decomposition of this corpus, measured against this run | **None named.** The Work's task does not identify such a directory, and none was found anywhere in this worktree (checked: no `reference-corpus/` or similarly named directory exists under this worktree's root, `.sergeant/`, or `reference/`). Per `_config/run-discipline.md` §1, this is accordingly not a measurement run against a pre-existing reference corpus — the blindness rule is vacuous for this run (there is nothing to be blind to), though its underlying discipline (cite only the subject subtree, never any pre-existing answer key) still holds for every downstream stage. |

## 4. Output paths

Each downstream stage (`10-inventory` through `90-reconcile`) writes its
declared artifact(s) to its own `output/` directory, per that stage's own
`output/README.md` — this contract does not invent new paths; it confirms
the existing per-stage convention (`../CONTEXT.md` "How the stages hand
off") applies unmodified to this run.

## 5. Success criteria

The Work's task names no criteria narrower than the workflow's own chained
stage outcomes. Per `../CONTEXT.md`'s stage table, this run is bounded by,
and done when, each stage's own `CONTEXT.md`-declared durable outcome has
been reached in sequence, ending with `90-reconcile` emitting the
adjudicated findings and final measurement package and running
`scripts/finalize.py` as its closing act. No task-specific success
criteria beyond that chain were given.

Resolving this contract required no ambiguous judgment calls — the Work's
task named the subject, its provenance document, and its exclusions
explicitly, and the vendored-subtree case (§1) applied cleanly, so the
fail-closed `# AMBIGUOUS — NOT RESOLVED` path was not triggered.
