# Provenance — Diagnose Bug

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W20** `diagnose-bug`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P2-019` | The diagnosing-bugs workflow triggers when the user says 'diagnose'/'debug this', or reports something broken, throwing, failing, or slow, and phases may be skipped only when explicitly justified. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (front matter description, lines 3-3) |

## Stages

### `10-build-feedback-loop`

| Unit | Statement | Source |
|---|---|---|
| `BU-P2-021` | A tight pass/fail signal that goes red specifically on the bug in question is the entire skill; with it, bisection/hypothesis-testing/instrumentation all just consume it, and without it no code-reading will substitute. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 1 core principle, lines 14-14) |
| `BU-P2-022` | Disproportionate effort should be spent building the feedback loop; the actor should be aggressive, creative, and refuse to give up at this stage. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 1, lines 16-16) |
| `BU-P2-023` | Feedback loops should be attempted in roughly this priority order: a failing test at the seam that reaches the bug; a curl/HTTP script against a running dev server; a CLI invocation diffed against a known-good snapshot; a headless-browser script (Playwright/Puppeteer); replaying a captured trace; a throwaway minimal harness; a property/fuzz loop for 'sometimes wrong output' bugs; a bisection harness automatable via `git bisect run`; a differential loop comparing old vs new versions or configs. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 1: loop-construction ladder items 1-9, lines 20-28) |
| `BU-P2-025` | Once a loop exists it should be tightened by asking whether it can be made faster (cache setup, skip unrelated init, narrow scope), sharper (assert on the specific symptom, not merely 'didn't crash'), and more deterministic (pin time, seed RNG, isolate filesystem, freeze network). | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 1: Tighten the loop, lines 35-39) |
| `BU-P2-026` | A 30-second flaky loop is barely better than no loop; a 2-second deterministic loop is treated as a debugging superpower. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 1: Tighten the loop, lines 41-41) |
| `BU-P2-027` | For non-deterministic bugs the goal is a higher reproduction rate, not a clean repro: loop the trigger ~100x, parallelize, add stress, narrow timing windows, inject sleeps; a 50%-flake bug is debuggable, a 1% one is not, so the actor keeps raising the rate until it is debuggable. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 1: Non-deterministic bugs, lines 45-45) |
| `BU-P2-028` | If no loop can genuinely be built, the actor must stop and say so explicitly, list what was tried, and ask the user for environment access, a captured artifact (HAR, log dump, core dump, timestamped screen recording), or permission to add temporary production instrumentation — and must not proceed to hypothesize without a loop. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 1: When you genuinely cannot build a loop, lines 49-49) |
| `BU-P2-029` | Phase 1 is complete only when the actor can name one already-run command (script path, test invocation, curl) that is red-capable (drives the actual bug path and asserts the user's exact symptom), deterministic (or, for flaky bugs, a pinned high reproduction rate), fast (seconds not minutes), and agent-runnable (unattended, human only via the HITL script). | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 1: Completion criterion, lines 53-58) |
| `BU-P2-030` | If the actor catches themselves reading code to build a theory before the red-capable command exists, that is the exact failure the skill prevents, and no red-capable command means no Phase 2. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 1 closing rule, lines 60-60) |

### `20-reproduce-and-minimize`

| Unit | Statement | Source |
|---|---|---|
| `BU-P2-031` | Phase 2 begins by running the loop and watching it go red, confirming the bug appears. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 2 start, lines 64-64) |
| `BU-P2-032` | The actor must confirm the loop reproduces the specific failure mode the user described (not a coincidentally nearby different failure), that it reproduces across multiple runs (or at a high enough rate for non-deterministic bugs), and that the exact symptom has been captured for later verification. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 2 confirm checklist, lines 68-70) |
| `BU-P2-033` | Once red, the repro is shrunk to the smallest scenario that still goes red by cutting inputs, callers, config, data, and steps one at a time, re-running the loop after each cut, keeping only what is load-bearing for the failure. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 2: Minimise, lines 74-74) |
| `BU-P2-034` | Minimizing the repro shrinks the hypothesis space for Phase 3 and produces the clean regression test used later in Phase 5. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 2: Minimise rationale, lines 76-76) |
| `BU-P2-035` | Minimization is done when every remaining element is load-bearing — removing any one of them makes the loop go green. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 2: Minimise completion, lines 78-78) |
| `BU-P2-036` | The actor must not proceed past Phase 2 until the bug has been both reproduced and minimized. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 2 closing rule, lines 80-80) |

### `30-hypothesize`

| Unit | Statement | Source |
|---|---|---|
| `BU-P2-037` | Before testing any of them, the actor generates 3-5 ranked hypotheses, because generating only one hypothesis anchors reasoning on the first plausible idea. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 3, lines 84-84) |
| `BU-P2-038` | Each hypothesis must be falsifiable, stated in the form 'If <X> is the cause, then <changing Y> will make the bug disappear / <changing Z> will make it worse'; if no prediction can be stated, the hypothesis is a vibe to be discarded or sharpened. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 3: falsifiability, lines 86-88) |
| `BU-P2-039` | The ranked hypothesis list should be shown to the user before testing begins, since users often re-rank instantly from domain knowledge or already-ruled-out hypotheses; this is a cheap checkpoint that should not block progress if the user is away. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 3: user checkpoint, lines 92-92) |

### `40-instrument`

| Unit | Statement | Source |
|---|---|---|
| `BU-P2-040` | Each instrumentation probe in Phase 4 must map to a specific prediction from Phase 3, changing exactly one variable at a time. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 4, lines 96-96) |
| `BU-P2-041` | Instrumentation tool preference is ordered: debugger/REPL inspection where the environment supports it (one breakpoint beats ten logs), then targeted logs at boundaries that distinguish hypotheses, and never 'log everything and grep'. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 4: tool preference, lines 100-102) |
| `BU-P2-042` | Every debug log must be tagged with a unique prefix (e.g. `[DEBUG-a4f2]`) so cleanup at the end becomes a single grep; untagged logs survive cleanup, tagged ones die. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 4: log tagging, lines 104-104) |
| `BU-P2-043` | For performance regressions, logs are usually the wrong tool; instead the actor establishes a baseline measurement (timing harness, `performance.now()`, profiler, query plan) and then bisects — measure first, fix second. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 4: performance branch, lines 106-106) |

### `50-fix-with-regression-test`

| Unit | Statement | Source |
|---|---|---|
| `BU-P2-044` | The regression test is written before the fix, but only if there is a correct seam for it. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 5, lines 110-110) |
| `BU-P2-045` | A correct seam is one where the test exercises the real bug pattern as it occurs at the call site; a too-shallow seam (single-caller test for a multi-caller bug, a unit test that can't replicate the triggering chain) gives false confidence. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 5: correct seam definition, lines 112-112) |
| `BU-P2-046` | If no correct seam exists, that absence is itself the finding: it must be noted as evidence that the codebase architecture is preventing the bug from being locked down, and flagged for the next phase. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 5: no seam finding, lines 114-114) |
| `BU-P2-047` | When a correct seam exists, the procedure is: turn the minimized repro into a failing test at that seam, watch it fail, apply the fix, watch it pass, then re-run the Phase 1 feedback loop against the original un-minimized scenario. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 5: procedure when a seam exists, lines 116-122) |

### `60-cleanup-and-postmortem`

| Unit | Statement | Source |
|---|---|---|
| `BU-P2-048` | Before declaring the diagnosis done, the actor must confirm: the original repro no longer reproduces, the regression test passes (or the seam absence is documented), all `[DEBUG-...]` instrumentation is removed via a prefix grep, throwaway prototypes are deleted or clearly marked, and the correct hypothesis is stated in the commit/PR message for the next debugger. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 6: required checklist, lines 128-132) |
| `BU-P2-049` | After the fix is in, the actor asks what would have prevented the bug; if the answer involves architectural change (no good test seam, tangled callers, hidden coupling) the actor hands off to the `/improve-codebase-architecture` skill with specifics, making the recommendation only after the fix — not before, since more is known by then. | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md` (Phase 6: architecture handoff, lines 134-134) |

## Notes

**Synthesis notes:** Proposal §8.2's "strong low-ambiguity reference workflow" assessment holds — all six stages survive the §6.3 reimplementation test.

