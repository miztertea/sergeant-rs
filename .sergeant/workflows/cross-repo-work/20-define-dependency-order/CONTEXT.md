# 20-define-dependency-order: define dependency order

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-assign-ownership/output/README.md | L4 | upstream artifact produced by `10-assign-ownership` |

## Purpose

An acyclic edge set in prerequisite>dependent form; cycles broken by a named contract artifact.

Trigger (workflow-level): Resolved project context shows more than one repository owns the requested outcome (not merely that the project has several repos).

## What must become true here (durable outcome)

An acyclic edge set in prerequisite>dependent form; cycles broken by a named contract artifact.

## Behavior contract

- **Dependency edges are created only when one repository's merged or deployed result is required by another, expressed in the prerequisite>dependent notation accepted by the dispatch command.**
  (trigger: ownership is assigned; outcome: the dependency graph contains only load-bearing edges, in a syntax the dispatch stage can consume directly)
  — `BU-P5-044`, `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 40-43)
- **Recognized dependency-edge evidence includes: a contract/schema producer preceding its consumers, infrastructure/config preceding runtime that requires it, independent implementations running in parallel once an approved contract exists, and deployment dependency recorded separately from code-merge dependency.**
  (trigger: an edge is being justified; outcome: dependency edges are drawn from a small, principled evidence vocabulary rather than guesswork)
  — `BU-P5-045`, `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 45-51)
- **Cycles are rejected before dispatch; if a cycle reflects a genuinely coupled contract, cross-repo-work instead defines the contract artifact or compatibility phase that breaks the cycle.**
  (trigger: the drawn dependency edges form a cycle; outcome: no cyclic dependency graph ever reaches dispatch)
  — `BU-P5-046`, `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 53-54)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
