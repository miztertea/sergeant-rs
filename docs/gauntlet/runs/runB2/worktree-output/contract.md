# Contract — repo-to-icm run (bounded measurement run, runB2)

Layer 4 artifact of `00-contract`, disposition `promote`. Established from
the Work's initiating task, `reference/UPSTREAM.md`, and direct inspection
of this worktree. No ambiguity was encountered; this run did **not** fail
closed (see §6).

## 1. Subject repository and revision

**Subject:** the vendored subtree `reference/sergeant-upstream/`, relative
to this worktree's root.

**Case determination:** the subject is a **vendored subtree**, not a live
checkout. Verified: `reference/sergeant-upstream/.git` does not exist;
`git -C reference/sergeant-upstream rev-parse --git-dir` resolves to the
*outer* repository's worktree git-dir
(`.../runB2/subject/.git/worktrees/runb2-subject`), i.e. any `rev-parse
HEAD` inside the subtree reports the outer repository's moving HEAD, not a
revision of the subject. The subject's revision is therefore a **recorded
fact**, not something derivable with `git` inside the subtree.

**Pinned revision (verbatim from `reference/UPSTREAM.md`):**

```
f430cfd4f90174a98adbd7abebbece6303817929
```

Source row: `reference/UPSTREAM.md` pins `sergeant-upstream/` from
`https://github.com/miztertea/sergeant` (fork of
`https://github.com/callmeradical/sergeant`) at that SHA ("main, includes
merged PR #2 — Claude background harness"), vendored 2026-08-08.

**Resolution method used, and how to re-verify:** re-read
`reference/UPSTREAM.md` and use its recorded SHA verbatim. Do **not**
attempt `git -C reference/sergeant-upstream ...` — that is the live-checkout
procedure and here it silently answers about the wrong repository.
The Work's task claimed the same SHA; the recorded value agrees, so there
is no discrepancy to note. For additional context: the outer worktree's
HEAD at contract time was `6c87cfb8d8adf5d1b5f8988ceb950f13bdf43eb3`
(branch `sergeant/01KZRBQF79YND346STVPWVVE5S`), which is the revision of
the outer repository whose tree contains the vendored files and
`UPSTREAM.md` as read.

## 2. Scope

This is a **bounded measurement run**. In scope for `10-inventory` to
enumerate are exactly two partitions of the subject, named verbatim by the
Work's task:

1. **Root operating instructions:** `reference/sergeant-upstream/AGENTS.md`
   and `reference/sergeant-upstream/README.md` (both verified present).
2. **The `bin/` fleet-dispatch partition:**
   `reference/sergeant-upstream/bin/` (verified present; authored shell
   entry points `sgt-*`, `_sgt-*.sh` helpers, and `wiki-daily-digest`).

Nothing else in the subject is in scope. This is a deliberate narrowing of
the workflow's normal "everything under the subject's root" default.

## 3. Exclusions, each with a reason

1. **All other partitions of the subject — out of scope by contract.** The
   Work's task states: "treat all other partitions as out of scope by
   contract." Concretely (as of this worktree): `tests/`, `templates/`,
   `skills/`, `scripts/`, `schema/`, `docs/`, `.claude/`, `.agents/`,
   `opencode.json`, `mise.toml`, `LICENSE`, `Dockerfile.test`,
   `.gitignore`, and any other path under `reference/sergeant-upstream/`
   not named in §2. Reason: bounded measurement run; the bound is the
   contract itself. Downstream absence of artifacts from these partitions
   means "excluded by contract," not "not present in the subject."
2. **VCS internals (`.git/`).** The subject has none of its own; the outer
   repository's `.git` is not authored procedural content of the subject.
3. **Build/dependency/generated output.** Not authored procedural content.
   Within the in-scope partitions this concretely includes
   `reference/sergeant-upstream/bin/__pycache__/` (compiled bytecode
   cache). The general class (`target/`, `node_modules/`, `dist/`,
   vendored lock caches, and the like) is excluded wherever encountered.
4. **`reference-corpus/` — the blindness boundary.** This is a measurement
   run, so per `../_config/run-discipline.md` §1 every stage's actor is
   blind to `reference-corpus/` for the entire run: never open it, never
   grep it, never let it enter a prompt, never let a helper's output
   surface its contents. It is the graders' answer key; reading it
   invalidates the measurement this run exists to produce. Stages checking
   their own output's shape use `../scripts/validate-structure.py`, never
   anything under `reference-corpus/`. Note on provenance of this
   exclusion: the Work's task did not itself name a reference/"gold"
   decomposition directory; the name `reference-corpus/` comes from the
   run-discipline file's own description of exactly this measurement (the
   N2 run against `reference/sergeant-upstream`). Per this stage's
   contract, no search for such a directory was performed to confirm or
   locate it — it is excluded by name, sight unseen.

## 4. Output paths (convention restated for the record)

Each downstream stage — `10-inventory`, `20-harvest`, `30-normalize`,
`40-classify`, `50-synthesize`, `60-draft`, `70-lint`,
`80-adversarial-review`, `90-reconcile` — writes its declared artifact(s)
to its own `output/` directory under
`.sergeant/workflows/repo-to-icm/<stage>/`, exactly as declared by that
stage's `output/README.md` (Layer 4). `60-draft` additionally materializes
draft package(s) under `.sergeant/drafts/workflows/` per its own
declaration. No new paths are invented by this contract; the convention
applies to this run unchanged.

## 5. Success criteria

The Work's task bounds this run as follows: decompose **only** the two
partitions in §2 into **draft** ICM workflows per
`.sergeant/workflows/repo-to-icm`, as a bounded measurement run. It names
no success criteria narrower than the workflow's own. Therefore: this run
is done when each stage's `CONTEXT.md`-declared durable outcome has been
made true in sequence, `00-contract` through `90-reconcile`, over the §2
scope and nothing more, ending with `90-reconcile`'s final measurement
package and its run of `scripts/finalize.py`. Nothing this run writes is
runnable procedure until a human crosses the publication boundary; the
deliverable is reviewable evidence (draft packages + report), not
published workflows.

## 6. Fail-closed status

Not triggered. Subject, revision, scope, and exclusions were all
unambiguous after reading the Work's task, `reference/UPSTREAM.md`, and
the worktree. No `# AMBIGUOUS — NOT RESOLVED` marker applies to this run's
`contract.md`, and downstream stages may treat this document as settled
fact per `../_config/run-discipline.md` §2.
