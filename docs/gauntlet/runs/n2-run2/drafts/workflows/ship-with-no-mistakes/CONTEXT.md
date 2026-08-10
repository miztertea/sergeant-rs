# ship-with-no-mistakes — workflow orientation

Layer 1. This file orients an actor entering this **draft** workflow for
the first time; it is not stage instruction.

## What this workflow is for

A shipping-gate run is about to be started, is active, or reaches an end
state (checks-passed, failed, cancelled, or an actionable finding).
Shipping-gate runs are launched under strict invocation discipline (never
auto-approved; started only by the coordinator, exactly once, after native
validation) and driven under strict handling rules while active (ask-user
gates always reach a human decision; the pipeline-owned worktree and
commit history are never touched; driving stops at checks-passed rather
than polling to merge). On failure or cancellation, recovery follows the
reported `branch_sync` action exactly (no improvised git surgery);
actionable findings are routed out to task-tracker tasks rather than fixed
inline by the run itself.

## Why this package has no stages

Unlike `dispatch-mode` and `standard-task-workflow`, this candidate is
visible in the source corpus **only through seven unattached
`stage-context` records** (`BU-0028`–`BU-0034`) — no `representation:
workflow` record and no `representation: stage` record cites
`ship-with-no-mistakes` at all. Zero `stage` candidates exist for it, so
this package has zero `NN-<stage-name>/` directories: nothing in
`references/../60-draft/references/draft-package-template.md`'s "Required
contents, per candidate package" list is a fixed minimum count of stages,
and inventing stage boundaries this corpus never classified would be
exactly the "unsupported invention" that file's `provenance.md` section
names as a defect for `80-adversarial-review` to catch — the wrong way to
paper over an honest gap. See `provenance.md` for the full accounting.

This is the finding `50-synthesize`'s own candidates.md called out for
this candidate — recorded there as "the corpus's most awkward workflow
candidate: a workflow visible only through orphaned judgment-content, with
zero classified checkpoints of its own" — carried forward here rather than
smoothed over.

## What a human reviewer needs to do before promotion

This package is not promotable as-is: an engine `workflow.toml` with an
empty `stages` list cannot run. Before promotion, a human reviewer needs
to either (a) read the underlying source material behind `BU-0028`–
`BU-0034` and define real stage candidates (a `start-run`, `drive-gates`,
`finish-run`, and `route-findings` stage each look plausible from the
`stage` names those unattached records cite, per `provenance.md`, but this
run has no direct `stage`-representation evidence for any of them — that
would be a fresh classification decision, not something `60-draft` can
make on this run's evidence), or (b) reconsider whether
`ship-with-no-mistakes` is better represented some other way entirely.

## Status

`status: draft`. This package lives under `.sergeant/drafts/workflows/`
and is not runnable procedure until a human promotes it into
`.sergeant/workflows/` (`docs/icm/convention.md` §2).
