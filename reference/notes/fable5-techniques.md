# Fable 5 Technique Notes

Distilled from Anthropic's "Prompting Claude Fable 5" (platform.claude.com docs,
fetched 2026-08-08) and the owner's 2026-07-31 Anthropic-convergence research
session. What the gauntlet applies, and why.

## Applied in the gauntlet

- **Fresh-context verifier subagents outperform self-critique.** Structurally
  enforced: builders never grade their own work; every re-gauntlet round gets
  fresh critics.
- **Ground progress claims in tool evidence.** Anthropic reports this nearly
  eliminates fabricated status reports on long runs. Ledger entries and critic
  verdicts must point at concrete test/tool output.
- **Outcome prompts, not step-walking.** Fable rewards being handed an outcome and
  left alone; over-prescription degrades output. Contracts state what must be
  true, not how to do it. Skills/prompts written for older models are often too
  prescriptive — audit before reuse.
- **Memory system.** `LESSONS.md` carries corrections and confirmed approaches
  across gauntlet units; one lesson per entry; update rather than duplicate;
  delete what proves wrong.
- **Parallel subagents, asynchronous.** Dispatch independent work concurrently;
  don't block on the slowest; keep long-lived subagents when cache reuse pays.
- **Effort is the second dial.** Within a model, low effort for routine stages,
  high/xhigh for verification and the hardest builds — often better than changing
  models.
- **Longer turns are normal.** Hard-task requests run minutes to hours; check on
  runs asynchronously rather than blocking; don't mistake a long quiet step for a
  stall.

## From the convergence research (owner's corpus)

- **Multiplicity is not institutional independence.** Same-model reviewers share
  lineage and assumptions; the critic panel mixes models and axes for genuine
  independence, not cost.
- **Unknowns are environmental objects.** Contracts name what is unknown, stale,
  or contradictory instead of forcing the actor to infer certainty (map vs.
  territory).
- **Context allocation over accumulation.** Excess and duplicated context degrades
  capable models; one responsibility per surface; refer rather than copy; thin
  universal instructions, discoverable specialized depth.
- **Hard governance belongs in enforceable structure.** Prompts orient; gates,
  identity, and deterministic checks enforce. (In the gauntlet: cargo gates and
  fail-closed rules are structure, not prompt requests.)
