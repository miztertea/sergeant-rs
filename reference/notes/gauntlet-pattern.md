# The Gauntlet Loop

The development method for this prototype. One gauntlet unit per milestone. The
orchestrator (a Fable 5 session) runs the loop; builders and critics are subagents
dispatched through ultracode workflows.

Grounding: the internet "gauntlet loop" pattern (blind critics, fresh instance per
retry, evidence-only adjudication, human gates outrank the loop), Anthropic's
"Prompting Claude Fable 5" guidance (fresh-context verifier subagents outperform
self-critique; ground progress claims in tool evidence; memory file across runs;
outcome prompts, not step-walking), and the IdeaOS operating contract
(Plan → Act → Verify at every scale; see `ideaos-agent-contract.md`).

## The loop

```
CONTRACT → BUILD → GATES → BLIND CRITICS → ADVERSARIAL VERIFY → FIX → (re-gauntlet ≤4) → MARK & LOG
```

1. **CONTRACT** (Plan). The orchestrator writes `docs/gauntlet/contracts/M<N>.md`:
   bounded outcome, acceptance tests, proposal sections in scope (cited by number,
   never restated), explicit non-goals, and an **Unknowns** section naming what is
   genuinely unresolved rather than forcing builders to fake certainty.
2. **BUILD** (Act). Builder subagents implement the contract. Prompts state the
   outcome and constraints, not procedures. Builders never grade their own work.
3. **DETERMINISTIC GATES.** `cargo fmt --check`, `cargo clippy -- -D warnings`,
   `cargo test`, plus the contract's acceptance tests. Red → back to BUILD.
   Critics never see a red build.
4. **BLIND CRITIC PANEL.** Fresh-context subagents that did not build, one axis
   each, grading the actual code/diff/test output — never a builder summary:
   - **spec-fidelity** — implementation vs. proposal + contract; deviations must be
     justified in the ledger or flagged.
   - **invariants** — the proposal §40 principles: one owner, work state ≠ process
     state, durable trajectory, disposable projections, idempotency, fail-closed
     ambiguity.
   - **simplicity** — the Ponytail Minimality Ladder is the grading rubric
     (`ideaos-agent-contract.md`): every addition should sit on its lowest
     viable rung; unjustified R7s and skipped rungs are findings.
   - **test-honesty** — do the tests verify the claims made, or mirror the
     implementation? Evidence-only.
5. **ADVERSARIAL VERIFY.** Each finding gets an independent refuter prompted to
   kill it. Confirmed findings are ranked; the builder fixes the largest gaps.
6. **RE-GAUNTLET** with fresh critics, capped at 4 iterations per milestone.
   Residual confirmed-but-deferred findings go to the ledger backlog — never
   silently dropped. (Diminishing returns: iterations 1–2 find architecture,
   3–4 find calibration, 5+ hit bedrock.)
7. **MARK & LOG.** Commit + push; append a ledger entry to `GAUNTLET.md` with two
   scorecards (see below); update `LESSONS.md`.

## Model assignment

Capability goes where judgment lives; diversity goes where independence matters.

| Role | Model | Rationale |
|---|---|---|
| Orchestrator (contracts, adjudication, integration, ledger) | Fable 5 (the session) | Judgment-dense; delegation is a named Fable 5 strength |
| Builders — thesis-bearing core (journal/recovery, workflow engine, adapters) | Fable (inherit) | First-shot correctness on complex well-specified work; a re-gauntlet round costs more than the model delta |
| Builders — mechanical work (scaffold, vendoring, CI, plumbing, UI polish) | Sonnet, low/medium effort | Well-defined execution |
| Critics — spec-fidelity, invariants | Fable or Opus, high effort | Verification rigor scales with effort |
| Critics — simplicity, test-honesty; refuters | Opus | Strong enough to refute; **a different model grading Fable-built code is an independence measure, not a cost measure** — same-lineage reviewers share blind spots ("multiplicity is not institutional independence") |

## Rules that outrank the loop

- Human gates outrank the loop; "keep going until perfect" never self-approves.
- Progress claims must trace to tool output from this session. Unverified means
  saying so.
- One responsibility per context surface: the proposal is the spec, contracts cite
  it, the ledger records what happened, LESSONS.md records what was learned.
  Refer, don't copy.
- Builders and fixers return `design_decisions` with a Ponytail rung per entry
  (`R3: seq gap detection via iterator adapter, stdlib only`); the ledger
  preserves them. An R7 names the lower rungs that failed.
- The ledger records **mission outcome** (contract met, gates green) and
  **environmental behavior** (iterations used, findings by axis and disposition,
  escalations, evidence completeness) separately.
- Ambiguity fails closed. A critic finding that cannot be confirmed or refuted is
  recorded as `PLAUSIBLE`, not dropped.
