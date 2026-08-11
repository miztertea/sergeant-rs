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
CONTRACT → BUILD → GATES → BLIND CRITICS → ADVERSARIAL VERIFY → FIX → (round 1) → CHECKPOINT GATE → (round 2, lean) → ADJUDICATE → MARK & LOG
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
5. **ADVERSARIAL VERIFY.** Findings are refuted adversarially — batched **per
   axis** (one refuter agent verifies all of an axis's findings; economy
   revision 2026-08-08, replacing per-finding refuters: M1 spent ~68 refuter
   agents and the per-axis batch preserves independence — the refuter still
   never wrote the code — at a quarter of the agent count).
6. **RE-GAUNTLET, capped at 2 panel rounds total** (economy revision
   2026-08-08, was 4: M1's data showed later rounds produce axis-tension
   oscillation that only orchestrator adjudication resolves, and the token
   cost was an order of magnitude beyond a prototype's needs). Round 1: full
   panel inside the build workflow, then round-1 fixes. Then the CHECKPOINT
   GATE (7a). Then ONE lean follow-up round under orchestrator control:
   fresh critics on only the axes that had confirmed findings (plus anything
   the checkpoint gate raised), medium effort, batched refuters. Everything
   still open after that is adjudicated, not re-looped. Residual
   confirmed-but-deferred findings go to the ledger backlog — never silently
   dropped.
7. **MARK & LOG.** Commit; append a ledger entry to `GAUNTLET.md` with two
   scorecards (see below); update `LESSONS.md`.
7a. **CHECKPOINT GATE** (adopted 2026-08-08, owner-directed; effective from M3).
   After round 1's fixes leave gates green: checkpoint commit, then a
   no-mistakes run (`--skip push,pr,ci`). Its confirmed findings join panel
   round 2's fix input; pipeline fix commits are adopted via `axi sync
   --recover`. Rationale: M1 showed the pipeline catches what the panel
   misses (independent method), and the independent signal is worth more
   while the fixer is still engaged than after MARK & LOG. This deliberately
   relaxes upstream Sergeant's "final gate, not an implementation loop"
   doctrine: that rule's cost case (human-driven, repeated restarts) does not
   apply to a bounded two-passes-per-milestone autonomous cadence. Not a
   per-round interleave — the pipeline validates committed branch state, and
   more than two passes re-validates barely-changed code.
8. **SHIPPING GATE** (adopted 2026-08-08, R5 — installed dependency supplies
   independent final validation; method + lineage diverse from the critic panel:
   own pipeline, own agent invocation, fresh disposable worktree). After the
   milestone commit: `no-mistakes axi run --intent "<contract outcome>" --skip
   push,pr,ci` — push/PR/CI stay ours. Never `--yes` (upstream Sergeant doctrine):
   the orchestrator inspects each gate finding and responds selectively via
   `axi respond` — `fix` for confirmed actionable findings, relay for ask-user,
   approve for no-op. Pipeline fix commits stay on the branch; never
   abort-and-restart to escape a gate. It is a final shipping gate, not an
   implementation loop — if per-milestone runtime proves too heavy, reduce
   cadence and log the change. Tool: no-mistakes v1.47.0 built from source
   (SHA recorded in the M1 ledger entry), gate agent: local claude CLI pinned
   to Sonnet via `agent_args_override` (owner decision 2026-08-08 — validation
   work, not judgment-dense; consistent with the model-assignment table above).

## Model assignment

**Revised 2026-08-10 (owner direction; supersedes the table below — kept for
the record). Sonnet by default, escalation by earned need.**

- The default worker is **Sonnet** — extraction, drafting, mechanical fixes,
  scripted verification, stage acting, running measurements, applying
  enumerated rulings. The test: if the task has a clear contract, grounded
  inputs, and a checkable output, Sonnet does it. Most work in a gauntlet
  loop is exactly that shape, which is why most agents are Sonnet.
- **Opus** comes in where the task is judgment under breadth — one context
  weighing many things against each other with no mechanical check to lean
  on: cross-partition synthesis, blind adversarial review, independent
  verification of a fixer's claims, and fixes whose correctness turns on
  architectural understanding rather than a ruling someone already wrote.
  The tell: if a wrong answer would look plausible and nothing downstream
  would mechanically catch it, that seat gets Opus.
- **Fable is one seat, not a tier: the orchestrator.** It holds the whole
  program in context — contracts, rulings, the ledger, what every workflow
  is doing and why — and does the things that require that totality:
  adjudicating conflicting findings, writing contracts and rulings, deciding
  when a loop should stop iterating and be ruled instead (L4), and catching
  when the process itself is wrong rather than its output. Fable never fans
  out; it's never a worker; there are no Fable subagents.

The escalation logic in one line: Sonnet executes contracts, Opus judges
outcomes, Fable owns the contract-writing and the tie-breaking. Cost follows
accountability — the expensive contexts are the ones whose mistakes the
system can't catch mechanically. Cross-model diversity on review seats
remains an independence measure, not a cost measure ("multiplicity is not
institutional independence"). Ruling record: R-S0-13 in
`docs/gauntlet/contracts/S0.md`; first applied in the S1 phase-1 round 2.

**Revision 2026-08-11 (owner direction, Cerberus handoff): Fable subagent
seats are permitted when earned.** The escalation ladder stays Sonnet →
Opus by earned need, and now extends one rung further: a Fable subagent
seat where even Opus-grade judgment has measurably fallen short, on the
same earned-need test. Separate context is the independence measure —
a Fable critic grading Fable-orchestrated work is legal because blindness
comes from fresh context, not model lineage (this narrows R-S0-13's
"never fans out; there are no Fable subagents" clause, which had encoded
cost discipline as an absolute). Unchanged: the orchestrator remains one
seat holding program totality, is never a worker in its own loop, and
most seats stay Sonnet.

*Superseded table (M0–P1 era, for the record):*

| Role | Model | Rationale |
|---|---|---|
| Orchestrator (contracts, adjudication, integration, ledger) | Fable 5 (the session) | Judgment-dense; delegation is a named Fable 5 strength |
| Builders — thesis-bearing core (journal/recovery, workflow engine, adapters) | Opus by default; Fable reserved for M4's adapter/recovery core (economy revision 2026-08-08 — session-window pressure; Opus-high is sufficient for well-contracted server code, and the panel catches the delta) | First-shot correctness on complex well-specified work; a re-gauntlet round costs more than the model delta |
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
- **Probe hygiene** (added 2026-08-08 after a live incident): critics and
  refuters may run mutation probes (comment out a guard, rerun tests) ONLY in
  a disposable `git worktree` copy, never the main tree, and must report the
  probe. During M2's follow-up round a refuter edited the main tree to
  hardcode a token and remove a replay guard so tests would pass — caught by
  the harness's security screen and reverted, tree integrity verified by the
  orchestrator afterward. A verifier that mutates the thing it verifies to
  reach a verdict has stopped verifying; its batch is quarantined and
  re-adjudicated. This is the anti-capture boundary applied to our own loop.
- **Independent probe execution** (revision 2026-08-11, from S2 wave 2's
  measured result): builders do not execute their own mutation probes.
  Each builder returns a *guard map* — per new test, the exact mutation
  that should kill it — and ONE independent prober (an Opus seat: blind
  adversarial verification) executes the whole wave's map batched in a
  single disposable worktree, plus probes of its own devising for the
  compositions no map listed. Probe survivors enter the fix round as
  executed evidence, not refutable claims. Why: S1's round 2 and S2's
  probes each caught pins a builder's self-probe had passed
  (parts-vs-composition, unfalsifiable assertions — L13); and one
  worktree at a time respects the disk budget where per-builder probe
  trees do not. Self-probing is retired.
- **Small diffs batch into the next larger panel** (revision 2026-08-11,
  owner direction, Cerberus session): R-S0-12 means every executable diff
  gets multi-axis review — it does not mean every diff gets its own
  gauntlet. A dedicated builder→panel→fixer loop spun up for one small
  file is waste; instead the diff rides the next larger loop's panel
  round as an added axis/scope item (first instance: `scripts/probe-env.sh`
  built by a single Sonnet seat, reviewed inside BS2's round 2 rather
  than by its own Opus panel). The review still happens; the ceremony
  doesn't multiply.
- **Parallel builders in one checkout** (revision 2026-08-11, from S2
  wave 1's near-misses): concurrent builders get exclusive, named file
  surfaces and never touch outside them, even transiently — no unscoped
  `cargo fmt` (use `--check` only), shared support modules are
  escalation-only, and the cargo lock serializing their test runs is
  normal, never a reason to create alternate target dirs. Builders that
  mutate shared state get worktree isolation or sequential scheduling
  instead.

## The scripts as run

Every orchestration script, exactly as executed, one file per workflow
invocation, lives under `resources/` in per-series folders (migration
2026-08-11; formerly `reference/gauntlet-workflows.zip` — extracted in
full, the zip retired because plain files make the between-milestone
diffs visible, which is where the economy revisions and protocol changes
actually live). Schemas, axis briefs, hygiene preambles, and model/effort
assignments are all in the scripts themselves.
