# 00-contract: establish the run's contract

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |
| ../_config/run-discipline.md | L3 | the blindness rule and the fail-closed `# AMBIGUOUS — NOT RESOLVED` propagation convention this stage's own "Fail closed" section uses |

You are the first stage of `repo-to-icm`. Nothing upstream has run yet — the
Work's initiating task (whatever the caller told Sergeant when this Work was
created) is your only other source, alongside this context and the worktree
itself. Read it before doing anything else; it is where the subject
repository, any explicit scope, and any explicit exclusions were named, if
they were named at all.

## What must become true here (durable outcome)

A `contract.md` exists in `output/` that unambiguously answers, for this run
and this run alone:

1. **Subject repository and revision.** Which repository (a path in this
   worktree, or a named subtree of it) and which exact Git revision — a full
   SHA, not a branch or tag that can move. Two genuinely different cases,
   and you must tell them apart before resolving anything:
   - **The subject is a live checkout** (it has its own `.git`, checkable
     with `git -C <subject> rev-parse --is-inside-work-tree`; a plain
     directory living inside this outer worktree with no `.git` of its own
     fails this check even if `git -C <subject> rev-parse HEAD` returns
     *something* — that something is the outer repository's own moving
     HEAD, not the subject's). Only here does "resolve `HEAD`" mean
     anything: `git -C <subject> rev-parse HEAD`.
   - **The subject is a vendored subtree** (no `.git` of its own — e.g.
     `reference/sergeant-upstream`, tracked as ordinary files inside this
     outer repository). Its pinned revision is not something to (re)derive
     from `git rev-parse` inside it — there is no such object to resolve
     against. It is a recorded fact: look for that subject's own provenance
     document (a file like `UPSTREAM.md` alongside it, or named in the
     Work's task) and use the SHA recorded there verbatim. If the Work's
     task names a subject with no such record and no `.git`, that is the
     ambiguity the next paragraph is about — do not invent a resolution
     method it doesn't have.

   If the Work's task does not name a revision, resolve it per whichever
   case above actually applies; if it does not name a subject at all, this
   is the ambiguity the "Fail closed" paragraph below covers.
2. **Scope.** What is in bounds for `10-inventory` to enumerate — normally
   "everything under the subject repository's root," narrowed only by
   exclusions named below.
3. **Exclusions, each with a reason.** At minimum:
   - VCS internals (`.git/`), build/dependency output (`target/`,
     `node_modules/`, `dist/`, vendored lock caches, and the like) — these
     are not authored procedural content.
   - **Any directory that holds a reference or "gold" decomposition of the
     very corpus this run is generating**, if one exists and this run will
     be measured against it. Reading it here would let the generator see
     the answer key; the measurement this workflow exists to support
     depends on that not happening. If the Work's task identifies such a
     directory, exclude it explicitly and name it in `contract.md`; if you
     are not told one exists, do not go looking for one to exclude — record
     that none was named.
   - Anything else the Work's task explicitly excludes, verbatim.
4. **Output paths.** Restate, for the record, that each downstream stage
   (`10-inventory` … the workflow's last stage) writes its declared artifact
   to its own `output/` directory, per that stage's `output/README.md` — you
   are not inventing new paths, just confirming the convention applies to
   this run.
5. **Success criteria.** What this run is bounded by and when it is done.
   If the Work's task names explicit criteria, record them. Otherwise this
   workflow's own outcome is bounded by what each stage's `CONTEXT.md`
   declares as its durable outcome, chained end to end — record that this
   run has no criteria narrower than that.

## How to do it

Work in the order above. Resolve the revision per the live-checkout-vs-
vendored-subtree distinction above rather than trusting a possibly-stale
value in the task text — if the resolved SHA disagrees with what the task
claimed, record the resolved value, note the discrepancy, and record which
resolution method you used (both because a stranger reading `contract.md`
later needs to know how to re-verify it, and because the two cases need
different verification procedures downstream: `git -C <subject> ...` for a
live checkout, re-reading the same provenance document for a vendored one).

**Fail closed, not by guessing.** If the subject repository, its revision,
or the scope is ambiguous after reading the Work's task and this worktree,
do not pick a plausible default. On the engine this workflow runs on today,
an actor stage has no way to pause its own turn and wait for a human's
answer mid-run — that is a `needs_input`/`waiting` transition the runtime
drives from *outside* the actor's turn, never something the turn itself can
request (`docs/gauntlet/notes/n2-fake-backend-semantics.md`). So the
fail-closed action actually available to you is: still write
`output/contract.md`, but make the ambiguity the document's own headline
rather than a fabricated answer —

```text
# AMBIGUOUS — NOT RESOLVED

What is ambiguous: <the specific missing/conflicting fact>
What was checked: <what you looked at before concluding it's ambiguous>
```

— and record **no** invented subject, revision, or scope in its place. Every
stage downstream of you treats `contract.md` as settled fact, with exactly
one exception: a `contract.md` opening with `# AMBIGUOUS — NOT RESOLVED` is
itself the fail-closed signal — see `../_config/run-discipline.md`, which
every downstream stage's Inputs table names, for what a stage receiving one
must do (stop, do not proceed on invented values). Also record this turn's
inability to ask as a meta-level grammar-pressure moment
(`../_config/run-discipline.md`; `90-reconcile/references/
reconciliation-method.md` §3) — the workflow's current grammar has no
actor-initiated "ask a human mid-run" primitive, which is real signal, not
something to paper over by pretending the ambiguity resolved itself.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its disposition.
