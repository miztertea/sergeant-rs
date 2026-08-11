# Contract — repo-to-icm run

## 1. Subject repository and revision

**Subject:** `reference/sergeant-upstream` (a named subtree of this outer
worktree).

**Case:** vendored subtree, not a live checkout. Checked with
`git -C reference/sergeant-upstream rev-parse --is-inside-work-tree`, which
does not report a subject-owned repository — there is no `.git` under
`reference/sergeant-upstream` (`ls reference/sergeant-upstream/.git` →
"No such file or directory"). This directory is tracked as ordinary files
inside the outer repository. `git -C reference/sergeant-upstream rev-parse
HEAD`, if run, would resolve the *outer* repository's own moving HEAD
(currently `54ed8243f880fc8b073d86e1ef89765b6590bc1b`), not a fact about the
subject — that value is not used below.

**Resolution method:** provenance document, not `git rev-parse`. The
subject's pinned revision is recorded in `reference/UPSTREAM.md`, the
provenance file alongside it:

> `sergeant-upstream/` | https://github.com/miztertea/sergeant (fork of
> https://github.com/callmeradical/sergeant) | `f430cfd4f90174a98adbd7abebbece6303817929`
> (main, includes merged PR #2 — Claude background harness) | 2026-08-08

**Pinned revision:** `f430cfd4f90174a98adbd7abebbece6303817929` (full SHA).

**Discrepancy check:** the Work's initiating task named this same SHA,
`f430cfd4f90174a98adbd7abebbece6303817929`, verbatim. It matches
`reference/UPSTREAM.md` exactly — no discrepancy to record. Re-verification
downstream should re-read `reference/UPSTREAM.md`, not run `git rev-parse`
inside `reference/sergeant-upstream` (there is no such object there to
resolve against).

## 2. Scope

The Work's task bounds this run to exactly two named partitions of the
subject, not "everything under the subject repository's root minus
exclusions":

1. **Root operating instructions** —
   `reference/sergeant-upstream/AGENTS.md` and
   `reference/sergeant-upstream/README.md`.
2. **The `bin/` fleet dispatch partition** —
   everything under `reference/sergeant-upstream/bin/`, subject to the
   build-artifact exclusion in §3.

`10-inventory` enumerates only within these two partitions. Every other
top-level entry under `reference/sergeant-upstream/` (`.agents/`, `.claude/`,
`docs/`, `schema/`, `scripts/`, `skills/`, `templates/`, `tests/`,
`Dockerfile.test`, `LICENSE`, `.gitignore`, `mise.toml`, `opencode.json`) is
out of scope for this run by explicit contract, per the Work's task
("treat all other partitions as out of scope by contract") — not because
those files are unreadable or uninteresting, but because this is a bounded
measurement run and the task named exactly these two scopes.

## 3. Exclusions, with reasons

- **`.git/`** (outer worktree's VCS internals) — not authored procedural
  content. (The subject itself has no `.git/` of its own; see §1.)
- **Build/dependency output within the in-scope `bin/` partition:**
  `reference/sergeant-upstream/bin/__pycache__/` (contains
  `sgt-callbackcpython-312.pyc`) — compiled Python bytecode cache, not
  authored source; excluded on the same basis as `target/`, `node_modules/`,
  `dist/`, and vendored lock caches per this stage's own contract.
- **Every partition outside the two named in §2** — excluded by the scope
  restriction itself (§2), not independently re-listed here.
- **Reference/"gold" decomposition directory:** none was named in the
  Work's task. A directory-name search of this worktree (`find . -iname
  '*reference-corpus*'`, no content read) turned up nothing named
  `reference-corpus/` or similar. Per this stage's own instruction, that is
  as far as the search goes absent the task naming one — recorded here as
  "none was named," not as a guarantee none exists elsewhere. If this run is
  in fact graded against such a directory that the task did not name, every
  stage downstream still owes it the blindness discipline in
  `../_config/run-discipline.md` §1 the moment it becomes known; nothing in
  this contract should be read as license to go looking for it later.
- **Nothing else** was explicitly excluded by the Work's task beyond the
  scope restriction itself.

## 4. Output paths

Each downstream stage (`10-inventory` through `90-reconcile`) writes its own
declared artifact(s) to its own `output/` directory, per that stage's own
`output/README.md`. This contract does not introduce any new output path or
override any stage's own convention — it only confirms the convention
applies unchanged to this run.

## 5. Success criteria

The Work's task names one explicit criterion narrower than the workflow's
own default: this is a **bounded measurement run**, and it is bounded to the
two partitions named in §2 — root operating instructions
(`AGENTS.md`, `README.md`) and the `bin/` fleet dispatch partition, at
revision `f430cfd4f90174a98adbd7abebbece6303817929`. No other partition of
`reference/sergeant-upstream` is in scope for this run's coverage, and
absence of downstream artifacts for any other partition means "excluded by
contract," not "missed."

Beyond that bound, this run carries no success criteria narrower than the
workflow's own chained outcome: each stage's `CONTEXT.md` "durable outcome,"
`00-contract` through `90-reconcile`, applied only to the in-scope material
established above.

## Note on this stage's own inability to ask

No ambiguity was found in the subject, revision, or scope — the task named
all three unambiguously and the SHA cross-checked cleanly against
`reference/UPSTREAM.md`, so this run did not need to invoke the fail-closed
`# AMBIGUOUS — NOT RESOLVED` path. Recorded here only because
`../CONTEXT.md` asks this stage to note, as a meta-level grammar-pressure
observation, that the engine gives an actor stage no mid-turn
actor-initiated way to ask a human a clarifying question
(`docs/gauntlet/notes/n2-fake-backend-semantics.md`) — this run simply did
not land in the situation where that gap would have mattered.
