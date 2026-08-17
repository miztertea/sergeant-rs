# Diagnose Bug
Draft workflow package — candidate **W20** `diagnose-bug` from the N1
manual reference-corpus decomposition (`sergeant-rs-workspace/knowledge/evidence/gauntlet/contracts/N1.md`).
This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Reproduce, isolate, prove, remediate and verify a defect.

## Trigger

"Diagnose"/"debug this", or something reported broken, throwing, failing, slow.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `10-build-feedback-loop` | actor-stage (§6.4, judgment) | A named, already-run, red-capable, deterministic, fast, agent-runnable command exists, or the run stops and asks for access/artifacts. |
| `20-reproduce-and-minimize` | actor-stage (§6.4, judgment) | The loop goes red on the user's exact symptom and every remaining element is load-bearing. |
| `30-hypothesize` | actor-stage (§6.4, judgment) | 3-5 ranked falsifiable hypotheses are shown to the user. |
| `40-instrument` | actor-stage (§6.4, judgment) | One probe per prediction, one variable at a time, tagged logs. |
| `50-fix-with-regression-test` | actor-stage (§6.4, judgment) | A test exists at a correct seam before the fix, or the seam's absence is recorded as the finding. |
| `60-cleanup-and-postmortem` | actor-stage (§6.4, judgment) | Repro gone, test passing, instrumentation removed, hypothesis recorded, architectural hand-off if warranted. |

## Authority envelope

This workflow receives an already-admitted Work intent (a reported defect to diagnose).

### Workflow may decide
- Construction strategy, minimization cuts, hypothesis ranking, instrumentation tooling, and seam adequacy (all J2, per each stage's own Bounded judgment section).
- Whether the fix implicates an architectural finding worth recording (`60-cleanup-and-postmortem`).

### Workflow may not decide
- Proceeding past `10-build-feedback-loop` without a red-capable loop, or asking the user instead — J0.
- Proceeding past `20-reproduce-and-minimize` without both reproduction and minimization — J5 gate.
- Skipping a phase without explicit justification (corrected 2026-08-16, ICM-R3: the prior citation for this constraint bundled two non-contiguous upstream spans — front-matter line 3 and body line 8 — under one locator; the phase-skipping discipline itself is on line 8 only).

### Human or Captain gates
- Phase 3's ranked-hypothesis display — advisory and non-blocking by the package's own text.
- `10-build-feedback-loop`'s J0 escalation when no loop can be built.

### Decision record
Material decisions are recorded per-stage in each stage's own output artifact.

## Notes for reviewers

Proposal §8.2's "strong low-ambiguity reference workflow" assessment holds — all six stages survive the §6.3 reimplementation test.

## Provenance

See `sergeant-rs-workspace/knowledge/evidence/gauntlet/promoted-provenance/diagnose-bug.md` for the complete stage-to-behavior-unit mapping and workflow-level citations. (ICM-R3 correction: the prior text pointed at a workflow-local `provenance.md` that does not exist under `.sergeant/workflows/diagnose-bug/`.)
