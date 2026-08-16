# Engine-gap claim: real nested/child workflow invocation

Filed per `docs/icm/record-shapes.md` §5's six-field template, as part of
the ICM-R3 `implement` package adjudication
(`docs/gauntlet/runs/icm-r3/implement/adjudication-draft.md`, unit
`BU-IMPL-09`). This is evidence for the reference corpus's own
`engine-pressure.md`, per §5 rule 6 — **it is not itself authorization for
any engine change.** `reference/proposal-icm-r-procedure-authority.md`'s
hard boundary and `docs/adr/0013-icm-r0-owner-rulings.md` decision 10 (the
runtime freeze through ICM-R4) both apply: no `src/` change is proposed by
this record or by this pass.

```json
{
  "behavior": "Two of implement's own two stages each need to invoke another already-checkpointed, multi-stage workflow to its own terminal completion (10-implement-with-tdd invokes tdd's seam-agreement/red-green-cycle discipline; 30-review invokes code-review's four-stage, review-independence-bearing procedure), not just pull in reference text.",
  "source_evidence": [
    "BU-P2-052 (implement/10-implement-with-tdd delegates to tdd, docs/gauntlet/promoted-provenance/implement.md)",
    "BU-P2-054 (implement/30-review delegates to code-review, docs/gauntlet/promoted-provenance/implement.md)",
    "tdd's own two current, independently-checkpointed stages (00-agree-seams, 10-red-green-cycle), each a fresh execution today when tdd is dispatched directly",
    "code-review's own four current, independently-checkpointed stages (00-pin-fixed-point, 10-identify-spec-source, 20-30-parallel-review, 40-aggregate), each a fresh execution today, plus its own settled Authority envelope",
    "the identical 'context composition today ... which does not exist yet' hedge already present, unaddressed, at every current call site: implement/10-implement-with-tdd/CONTEXT.md, implement/30-review/CONTEXT.md (prior revision), and worker-mission/20-implement/CONTEXT.md",
    "docs/gauntlet/runs/icm-r3/tdd/review.md's independent finding that tdd's own REHOME producer draft never weighed this exact alternative, for the identical underlying shape",
    "worker-mission/10-triage-and-route/CONTEXT.md's own self-flagged 'Additional note', independent of this claim: 'This is the branching point that raises engine-gap G6 (child-procedure invocation with its own checkpoints) ... It survives partially: representable today only by inlining the chosen discipline's stages, losing independent parent/child checkpoint and recovery visibility' -- generalizing this claim's shape across five delegation targets (diagnose-bug, prototype, tdd, implement, deepen-module) rather than implement's own two (ICM-R3, BU-WM-12)"
  ],
  "lower_rungs_attempted": [
    "shared context (@@tdd, as drafted for tdd's disputed REHOME)",
    "context composition prose without @@ syntax (implement's own current, pre-revision wording for both delegations)",
    "ad-hoc dispatch of a separate Work via sgt run (implement/30-review's corrected wording in this pass's draft)",
    "workflow-local duplication (writing tdd's or code-review's own stage content directly into implement's two stages)"
  ],
  "why_each_fails": {
    "shared context (@@tdd, as drafted for tdd's disputed REHOME)": "Pulls text into the current actor's single turn; produces no independent durable checkpoint, retry, or measurement of its own. tdd's own 00-agree-seams and 10-red-green-cycle are today two distinct fresh executions with per-seam retry; collapsing them into one caller turn absorbs an unbounded, per-seam-repeating sub-procedure into that turn's private judgment (record-shapes.md §5's own worked example, applied directly).",
    "context composition prose without @@ syntax (implement's own current, pre-revision wording for both delegations)": "Same mechanics as the @@ case, minus even the textual pin: 'running code-review to its own completion' cannot be produced by reading code-review's CONTEXT.md into implement/30-review's turn, because code-review's actual behavior (asking the user when its own fixed point or spec source is missing, spawning two isolated sub-agents in one message, aggregating without merging) depends on code-review running as its own execution with its own Authority envelope intact, not as borrowed prose inside a different workflow's stage.",
    "ad-hoc dispatch of a separate Work via sgt run (implement/30-review's corrected wording in this pass's draft)": "proposal-next-iteration-icm-workflows.md §7.7, verbatim: an agent 'could even submit another sgt run, but neither behavior creates a real child workflow inside the parent's state machine' — this preserves code-review's own checkpoints and envelope (an improvement over context composition) but still loses parent/child trajectory identity, deterministic retry and cancellation semantics coordinated with the parent, parent-aware recovery, per-subworkflow telemetry visible in the parent's own trajectory, and a completion contract the parent's own engine state understands rather than one the actor must track and report itself.",
    "workflow-local duplication (writing tdd's or code-review's own stage content directly into implement's two stages)": "Recreates the exact duplication docs/icm/convention.md §1 rule 2 and §5 rule 3 exist to prevent — two independent current consumers of each delegate (implement and worker-mission, for tdd; a real second path exists once worker-mission or any other package calls code-review directly) would each carry their own private copy, and drift between the copies would be undetectable and untracked, exactly as record-shapes.md §5's own worked example names for this failure mode."
  },
  "minimum_runtime_capability_required": "A stage kind (or workflow.toml field on an actor stage) that binds and executes another pinned workflow as a real child, retaining parent/child trajectory identity, coordinated retry and cancellation, parent-aware recovery, and per-subworkflow telemetry visible in the parent's own journal/trajectory — the same six losses proposal-next-iteration-icm-workflows.md §7.7 already names, now evidenced by two concrete, currently-promoted consumer packages (implement, worker-mission) rather than a hypothetical.",
  "observable_acceptance_test": "implement/10-implement-with-tdd invokes tdd and implement/30-review invokes code-review as real children; each child's own stage-level checkpoints, retries, and measurements (tdd's per-seam red-green cycles; code-review's four stages and its own Authority-envelope-driven user questions) are visible in implement's own trajectory without implement re-implementing or silently absorbing them, and a second parent (worker-mission, for tdd; any future direct caller, for code-review) invoking the same child identity shares that one identity rather than a duplicated copy."
}
```

## Notes

- This claim generalizes the shape `docs/gauntlet/runs/icm-r3/tdd/
  review.md` identified for `tdd` specifically (there, from the delegate's
  own side) to both of `implement`'s delegations (here, from the caller's
  side) and to `code-review`'s settled case, which the `tdd` dispute never
  covered. It is filed once, at `implement`, rather than duplicated into a
  parallel claim under `tdd`'s own draft tree, since the underlying gap
  and its evidence are the same runtime capability — a future pass may
  merge this into `tdd`'s own record or into a corpus-level
  `engine-pressure.md` once one exists; neither exists yet under
  `.sergeant/` or `docs/gauntlet/` at the time of this pass (grepped
  directly).
- Filing this claim does not settle `tdd`'s own ICM-R3 placement dispute.
  It is compatible with any of that dispute's possible outcomes: if `tdd`
  ultimately REHOMEs to a shared context after all (e.g., because a
  reviewer judges the per-seam checkpoint granularity it currently has is
  itself over-engineered for a short technique, as the `tdd` reviewer's
  own recommendation floats as one acceptable resolution), this claim's
  `code-review` evidence alone still stands on its own and the claim is
  simply narrower, not invalid.
- Per record-shapes.md §5 rule 6, this claim is evidence, not a mandate.
  `implement`'s corrected `30-review` delegation (this pass's draft) does
  not wait on this claim being accepted — it uses the best-available
  lower-rung mechanism (separate-Work dispatch) today and names this claim
  as the eventual correct fix.
