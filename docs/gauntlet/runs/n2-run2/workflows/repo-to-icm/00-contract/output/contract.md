# Contract — repo-to-icm run

This document is the settled record for this run and this run alone, per
`../CONTEXT.md`'s "What must become true here." Every stage from
`10-inventory` onward treats this file as fact.

## 1. Subject repository and revision

**Subject:** `reference/sergeant-upstream` (a path inside this outer
worktree).

**Case:** vendored subtree, not a live checkout. Checked:
`reference/sergeant-upstream/.git` does not exist (`ls` reports "No such
file or directory"). Running `git -C reference/sergeant-upstream rev-parse
--is-inside-work-tree` from this worktree returns `true`, and `git -C
reference/sergeant-upstream rev-parse HEAD` would likewise return
*something* — but per this stage's own `CONTEXT.md`, that something is the
**outer** repository's moving HEAD (confirmed: it equals this worktree's
own `git rev-parse HEAD`, `85568a0f52500537826c7010756dc5bfa558d576`), not
a revision of the subject. The subject has no `.git` of its own, so there
is no such object to resolve against; its pinned revision is a recorded
fact, not something to (re)derive with `git rev-parse`.

**Resolution method used:** read the subject's own provenance document,
`reference/UPSTREAM.md`, and took the SHA recorded there verbatim (per this
stage's `CONTEXT.md`, the vendored-subtree case). That document records:

> `sergeant-upstream/` | https://github.com/miztertea/sergeant (fork of
> https://github.com/callmeradical/sergeant) | `f430cfd4f90174a98adbd7abebbece6303817929`
> (main, includes merged PR #2 — Claude background harness) | 2026-08-08

**Resolved revision:** `f430cfd4f90174a98adbd7abebbece6303817929` (40-hex
full SHA, upstream `main`, fork `miztertea/sergeant`).

**Discrepancy check:** the Work's initiating task also states this same
SHA (`f430cfd4f90174a98adbd7abebbece6303817929`) as the pin. The task's
stated value and the provenance document's recorded value are character-
for-character identical — no discrepancy to record.

**Re-verification procedure for a stranger:** re-read
`reference/UPSTREAM.md`'s `sergeant-upstream/` row; do not run `git -C
reference/sergeant-upstream rev-parse HEAD` expecting it to reflect the
subject's revision — it does not have one to report.

## 2. Scope

Everything under `reference/sergeant-upstream/` (the subtree root), as it
stands at the outer worktree's current commit
(`85568a0f52500537826c7010756dc5bfa558d576`), narrowed only by the
exclusions in §3. This is the full scope the Work's task grants: "the
subtree only."

## 3. Exclusions, with reasons

- **VCS internals / build-dependency output** (`.git/`, `target/`,
  `node_modules/`, `dist/`, vendored lock caches, and the like) — not
  authored procedural content. Checked: the subject subtree
  (`reference/sergeant-upstream/`) has no `.git/` of its own (see §1) and
  its top-level directories are `.agents/ .claude/ bin/ docs/ schema/
  scripts/ skills/ templates/ tests/` — no build/dependency-output
  directory is currently present under it, but this exclusion category
  still bounds scope for `10-inventory` should one appear (e.g. under a
  test fixture).
- **A reference/"gold" decomposition of this run's own generated
  corpus** — the Work's task does not name any such directory, and none
  was found under this outer worktree (checked: no directory named
  `reference-corpus` or similar exists anywhere under this worktree's
  tree). Per this stage's own `CONTEXT.md`, "if you are not told one
  exists, do not go looking for one to exclude — record that none was
  named": none was named, so none is recorded as excluded, and no such
  directory was opened, read, or otherwise inspected.
- **Explicit exclusions named verbatim by the Work's task:**
  - `reference/UPSTREAM.md` itself — the subject's own provenance record,
    not part of the subject subtree's procedural content.
  - `.sergeant/` — this workflow's own definition tree, not subject
    material.
  - `AGENTS.md` — this outer repository's own orientation file, not
    subject material.

## 4. Output paths

Confirmed: no new paths are being invented here. Each downstream stage
(`10-inventory` through `90-reconcile`) writes its declared artifact(s) to
its own `output/` directory, per that stage's own `output/README.md`, per
`../CONTEXT.md`'s stage table. This restates that the existing convention
applies to this run; it does not redefine it.

## 5. Success criteria

The Work's initiating task names no success criteria narrower than
completing the decomposition it describes. Per this stage's `CONTEXT.md`:
this run's success criteria are bounded by nothing narrower than what each
stage's own `CONTEXT.md` declares as its durable outcome, chained end to
end from `00-contract` through `90-reconcile`. No additional or
narrower bar is recorded here.

## Meta note (run-discipline, not part of the contract's substance)

This stage encountered no ambiguity — the subject, revision, and scope
were all resolvable from the Work's task plus `reference/UPSTREAM.md` — so
no `# AMBIGUOUS — NOT RESOLVED` marker applies and no actor-initiated
"ask a human mid-run" gap was hit this run.
