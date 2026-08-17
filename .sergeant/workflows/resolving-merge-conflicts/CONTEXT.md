# Resolving Merge Conflicts
Draft workflow package — candidate **W26** `resolving-merge-conflicts` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`).
This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Resolve an in-progress git merge/rebase conflict without inventing behavior or aborting.

## Trigger

A git merge or rebase is in a conflicted state.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `10-research-intent` | PL-5 (stage) | The intent behind each conflicting side is researched; folds the demoted `00-assess-state` checkpoint as a PL-6 helper (N1 adjudication A4). |
| `20-resolve-hunks` | PL-5 (stage) | Both intents are preserved, or one is picked with the trade-off recorded; behavior is never invented; the merge is never aborted; folds the demoted `30-validate` and `40-finish` checkpoints as PL-6 helpers (N1 adjudication A4). |

`00-assess-state`, `30-validate`, and `40-finish` were demoted per N1 adjudication A4 (finding N1-BH-02): each was classified at extraction as deterministic machinery (ladder §6.5) with no checkpoint argument beyond the boilerplate. Their behavior units survive, folded into the adjacent judgment-bearing stage as helper invocations — see each stage's own `CONTEXT.md` and `provenance.md`'s "Adjudication A4" section.

## Authority envelope

This workflow receives an already-conflicted git merge or rebase state as
its trigger.

### Workflow may decide
- Which primary sources (commit messages, PRs, issues/tickets) to inspect
  when tracing each side's original intent (`10-research-intent`).
- How to preserve both sides' intent where possible, or which side matches
  the merge's stated goal when a trade-off is required (`20-resolve-hunks`).
- What counts as breakage the merge caused, and how to fix it, when running
  the project's automated checks (`20-resolve-hunks`).

### Workflow may not decide
- Invent new behavior not implied by either side's traced intent.
- Abort the merge or rebase.
- Resolve a genuine tie between two irreconcilable sides, or fix an
  automated-check failure it cannot attribute to the merge, without asking.

### Human or Captain gates
- No primary source can be found for one side's intent (`10-research-intent`).
- The two sides are genuinely irreconcilable with no discoverable stated
  goal to break the tie (`20-resolve-hunks`).
- An automated check fails for a reason not attributable to the merge, or
  its correct fix itself requires a judgment call beyond mechanical repair
  (`20-resolve-hunks`).

### Decision record
Material decisions (traced intent, hunk resolutions and trade-offs, any J0
stop) are recorded in the stage's own turn and surfaced through
`needs_input` where applicable; this two-stage workflow declares no
separate decision-log file.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
