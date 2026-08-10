# 00-contract: establish the run's contract

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

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
   SHA, not a branch or tag that can move. If the Work's task does not name
   a revision, resolve `HEAD` of the named subject and record the resulting
   SHA; if it does not name a subject at all, stop and ask rather than
   guessing which directory in the worktree is meant.
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

Work in the order above. Resolve the revision with the subject repository's
own tooling (e.g. `git -C <subject> rev-parse HEAD` or the named ref) rather
than trusting a possibly-stale value in the task text — if the resolved SHA
disagrees with what the task claimed, record the resolved value and note
the discrepancy; do not silently prefer one over the other.

**Fail closed, not by guessing.** If the subject repository, its revision,
or the scope is ambiguous after reading the Work's task and this worktree,
stop here and ask rather than picking a plausible default. Every stage
downstream of you treats `contract.md` as settled fact; an unresolved
ambiguity you paper over becomes their silent error, not yours to fix later
(this mirrors the reference corpus's own "pin the fixed point" discipline —
fail here, not inside a later stage).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its disposition.
