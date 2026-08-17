# Prototype
Draft workflow package — candidate **W21** `prototype` from the N1
manual reference-corpus decomposition (`sergeant-rs-workspace/knowledge/evidence/gauntlet/contracts/N1.md`).
This is Layer 1 orientation only — it is never delivered as a stage's
instructions; each stage's own `CONTEXT.md` (Layer 2) is the actor's
contract (`docs/icm/convention.md` §1a rule 5).

## Purpose

Build a throwaway prototype to answer a design question, branching between logic and UI questions.

## Trigger

The user wants to sanity-check whether a state model or logic feels right, or explore what a UI should look like.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-select-branch` | actor-stage (§6.4, judgment) | Which question type (logic vs. UI) is decided; a heuristic fallback is recorded when the user is unreachable. |
| `10-record-question` | actor-stage (§6.4, judgment) | The design question the prototype must answer is recorded. |
| `20L-build-logic` | actor-stage (§6.4, judgment) | A logic prototype is built to answer the recorded question. |
| `20U-build-variants` | actor-stage (§6.4, judgment) | UI variants are built to answer the recorded question. |
| `30-hand-off` | actor-stage (§6.4, judgment) | The prototype and its answer are handed off. |
| `40-capture` | actor-stage (§6.4, judgment) | A validated decision is folded into real code and rewritten to production standards; the throwaway is preserved on a throwaway branch. |

## Authority envelope

This workflow receives an already-admitted Work intent (a design question to sanity-check).

### Workflow may decide
- Which branch (logic vs. UI) applies, and a heuristic fallback when ambiguous and the user is unreachable (`00-select-branch`).
- Sub-shape, variant count, and structural divergence within the UI branch (`20U-build-variants`); interface/module design within the logic branch (`20L-build-logic`).

### Workflow may not decide
- That a prototype has answered its question — this is always confirmed by the user, never inferred (`40-capture`, corrected at ICM-R3).
- Whether to perform a real mutation in a UI variant, or expose the variant switcher in production (`20U-build-variants`, J5 constraints).

### Human or Captain gates
- Confirming the prototype's answer before capture.
- The branch-selection heuristic, when the user is reachable to ask directly.

### Decision record
Material decisions are recorded per-stage in each stage's own output artifact.

## Notes for reviewers

The A/U branch at `20L`/`20U` is the corpus's cleanest evidence for *conditional* procedure. It is representable today as one selection stage (`00-select-branch`) plus mutually-exclusive downstream stages — recorded as grammar pressure for a future conditional-stage schema extension, not an engine gap (the current linear `workflow.toml` requires both stage directories to exist; the non-selected one is a documented no-op for that run).

## Provenance

See `sergeant-rs-workspace/knowledge/evidence/gauntlet/promoted-provenance/prototype.md` for the complete stage-to-behavior-unit mapping and workflow-level citations. (ICM-R3 correction: the prior text pointed at a workflow-local `provenance.md` that does not exist under `.sergeant/workflows/prototype/`.)
