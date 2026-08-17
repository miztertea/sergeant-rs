# 10-design-it-twice: design it twice

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-classify-dependencies/output/README.md | L4 | upstream artifact produced by `00-classify-dependencies` |

## Purpose

At least 3 independently generated, structurally different designs, each under a distinct constraint, compared on depth/locality/seam placement, ending in an opinionated recommendation.

Trigger (workflow-level): A module's interface needs redesign, or a port/adapter decision needs to be made deliberately rather than by default.

## What must become true here (durable outcome)

At least 3 independently generated, structurally different designs, each under a distinct constraint, compared on depth/locality/seam placement, ending in an opinionated recommendation.

## Behavior contract

- **When exploring alternative interfaces for a chosen deepening candidate, run a parallel-sub-agent pattern that produces several radically different designs before picking one, on the premise that a single first idea is unlikely to be the best.**
  (trigger: a deepening candidate has been chosen and alternative interface shapes should be explored before committing; outcome: multiple independently-produced interface designs exist and are compared before one is chosen)
- **Before spawning parallel design sub-agents, first produce and show the user a framing of the problem space (constraints, dependency category, an illustrative sketch), then proceed immediately to spawning sub-agents without waiting for a reply — this is an unconditional discipline stated with no hedge or case-by-case carve-out, not a local implementation choice.**
  (trigger: framing has been produced for a design-it-twice pass; outcome: the user is informed of the problem framing concurrently with sub-agent work starting, rather than gating on their reply)
  **Corrected 2026-08-16, ICM-R3**: this unit's J-rung was previously stated as J1 (local, reversible); the independent reviewer found that inconsistent with this same table's own J5 treatment of the testing stage's equally unconditional, no-hedge testing directives, and noted that whether this stage gates on a live reply is exactly the evidence the package's own placement argument (PL-4/PL-5, not PL-2) depends on — not a cosmetic choice. Re-rung to **J5**.
- **Produce at least three independently-generated, radically different interface designs for the same deepening candidate, each under an explicit distinguishing design constraint (e.g. minimal interface, maximal flexibility, optimize the common case, ports-and-adapters).**
  (trigger: the problem space has been framed for a design-it-twice pass; outcome: three or more genuinely distinct candidate interface designs exist for comparison)
- **Each sub-agent's design brief must include both the `codebase-design` technique vocabulary and the project's own domain vocabulary (from the project's own `CONTEXT.md`), so a generated design speaks the codebase's actual language rather than a generic one.**
  (trigger: briefing a parallel design sub-agent; outcome: every sub-agent's brief carries both the technique's own vocabulary and the target project's domain vocabulary)
  **Added 2026-08-16, ICM-R3**: extracted at N1, classified workflow-local, never actually written into this stage's contract until now.
- **Present the several generated interface designs to the user one at a time so each can be absorbed, then compare them explicitly by depth (leverage at the interface), locality (where change concentrates), and seam placement, ending with an opinionated recommendation (including a hybrid if warranted) rather than a menu.**
  (trigger: multiple candidate designs have been produced; outcome: the user receives a structured comparison and a concrete recommendation, not a raw dump of options)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- How to explore alternative designs, which constraints to assign each sub-agent, and how many designs to produce beyond the minimum three.
- How to compare the produced designs and what to recommend.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J5 — governing constraints
- **Framing must be shown to the user, then sub-agents spawn immediately without waiting for a reply** — unconditional, not a case-by-case gate.
- **Every sub-agent brief must include both the technique vocabulary and the project's own domain vocabulary**.

### Completion boundary
This stage may complete only when at least three structurally distinct designs exist, each under an explicit constraint, presented sequentially and compared by depth/locality/seam placement, ending in an opinionated recommendation.

### Decision evidence
The generated designs and the final recommendation are this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
