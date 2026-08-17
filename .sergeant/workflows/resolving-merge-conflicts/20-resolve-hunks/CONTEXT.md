# 20-resolve-hunks: resolve hunks

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-research-intent/output/README.md | L4 | upstream artifact produced by `10-research-intent` |

## Purpose

Both intents are preserved, or one is picked with the trade-off recorded; behavior is never invented; the merge is never aborted.

Trigger (workflow-level): A git merge or rebase is in a conflicted state.

## What must become true here (durable outcome)

Both intents are preserved, or one is picked with the trade-off recorded; behavior is never invented; the merge is never aborted.

## Behavior contract

- **Each conflicting hunk is resolved by preserving both sides' intent where possible, or by picking the side matching the merge's stated goal and recording the trade-off when incompatible; resolution must never invent new behavior, and the merge/rebase must always be carried to completion rather than aborted.**
  (trigger: conflict intents are understood; outcome: every hunk is resolved consistent with recorded intent, with no invented behavior, and the operation is never abandoned via --abort)

## Helper: validate and finish (folded from demoted `30-validate` and `40-finish`, N1 adjudication A4)

`30-validate` and `40-finish` were classified at extraction as deterministic machinery (ladder §6.5) with no checkpoint argument beyond the boilerplate; per adjudication A4 both are demoted and folded here, in sequence, as the concluding helper invocations of this checkpoint, subordinate to this stage's own judgment-bearing outcome:

- **After resolving conflicts, the actor discovers and runs the project's automated checks in the order typecheck, then tests, then format, fixing anything the merge broke.**
  (trigger: all hunks are resolved; outcome: the project's own automated checks pass and any merge-induced breakage is fixed)
- **The workflow concludes by staging and committing everything, and, if rebasing, continuing until every commit has been rebased.**
  (trigger: validation has passed; outcome: the merge or rebase is fully completed and committed)

  If continuing the rebase surfaces a new conflict, treat it as a return to
  this stage's own hunk-resolution behavior, not a fresh unaddressed state.

## Bounded judgment

Apply `@@bounded-judgment`.

### J5 — governing constraint
- Never invent new behavior; never abort the merge or rebase.

### J2 — delegated to this stage
- Preserve-both vs. pick-a-side, and which side matches the merge's stated goal.
- What counts as breakage the merge caused among failing automated checks, and how to fix it, bounded by the J5 never-invent constraint.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- The two sides are genuinely irreconcilable and no stated goal (from the Work intent, a commit message, or `10-research-intent`'s own findings) resolves which one governs: record both intents, state the trade-off, and ask the user to choose rather than resolving the tie unilaterally.
- An automated check fails and the actor cannot establish that the failure is attributable to this merge, or the correct fix itself requires a judgment call beyond mechanical repair (i.e. not obviously implied by the hunk resolution already recorded): record what was checked and ask the user rather than guessing at the correct fix.

### Completion boundary
This stage may complete only once every hunk is resolved consistent with recorded intent, the project's automated checks pass, and the merge/rebase is carried to completion and committed — never aborted — or the stage has stopped at one of the J0 cases above.

### Decision evidence
Each hunk's resolution and any recorded trade-off are this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
