# Deepen Module
Draft workflow package — candidate **W25** `deepen-module` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`).
This is Layer 1 orientation only — it is never delivered as a stage's
instructions; each stage's own `CONTEXT.md` (Layer 2) is the actor's
contract (`docs/icm/convention.md` §1a rule 5).

## Purpose

Turn a shallow module into a deep one at a deliberately chosen seam.

## Trigger

A module's interface needs redesign, or a port/adapter decision needs to be made deliberately rather than by default.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-classify-dependencies` | actor-stage (§6.4, judgment) | A four-way classification determines whether a port is needed at all. |
| `10-design-it-twice` | actor-stage (§6.4, judgment) | At least 3 independently generated, structurally different designs, each under a distinct constraint, compared on depth/locality/seam placement, ending in an opinionated recommendation. |
| `20-test-at-new-interface` | actor-stage (§6.4, judgment) | Old shallow-module tests are deleted; new tests assert through the interface only. |

## Authority envelope

This workflow receives an already-admitted Work intent (a module's interface needs redesign, or a deliberate port/adapter decision is needed).

### Workflow may decide
- Dependency classification and the matching adapter strategy, including the two-adapter threshold for exposing a port (`00-classify-dependencies`).
- Which constraints to assign design sub-agents, how to compare designs, and what to recommend (`10-design-it-twice`).

### Workflow may not decide
- Whether to delete old shallow-module tests or keep testing internal state — both fixed, unconditional disciplines (`20-test-at-new-interface`).
- Whether to wait for the user's reply before spawning design sub-agents — framing is shown, then sub-agents spawn immediately, unconditionally (`10-design-it-twice`).

### Human or Captain gates
- None — every checkpoint in this workflow resolves via J2/J5, not a live human gate; the user receives framing and sequential design presentations but the workflow does not block on a reply.

### Decision record
Material decisions are recorded per-stage in each stage's own output artifact.

## Provenance

See `docs/gauntlet/promoted-provenance/deepen-module.md` for the complete stage-to-behavior-unit mapping and workflow-level citations. (ICM-R3 note: the prior text pointed at a workflow-local `provenance.md` that does not exist — the same class of defect found in 19 of 23 packages' own `CONTEXT.md` files, systemic rather than specific to this package, corrected here incidentally while this file was already being amended for other reasons.)
