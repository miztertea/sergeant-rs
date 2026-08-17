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
- **Recognized dependency-edge evidence includes: a contract/schema producer preceding its consumers, infrastructure/config preceding runtime that requires it, independent implementations running in parallel once an approved contract exists, and deployment dependency recorded separately from code-merge dependency.**
  (trigger: an edge is being justified; outcome: dependency edges are drawn from a small, principled evidence vocabulary rather than guesswork)
- **Cycles are rejected before dispatch; if a cycle reflects a genuinely coupled contract, cross-repo-work instead defines the contract artifact or compatibility phase that breaks the cycle.**
  (trigger: the drawn dependency edges form a cycle; outcome: no cyclic dependency graph ever reaches dispatch)

## Bounded judgment

Apply `@@bounded-judgment`.

### J5 — governing constraint
- No cyclic dependency graph ever reaches dispatch.

### J2 — delegated to this stage
- Which evidence justifies a dependency edge, from the named vocabulary.
- How to break a genuinely coupled cycle by defining a contract artifact or compatibility phase.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers.

### Completion boundary
This stage may complete only once the dependency edge set is acyclic, drawn only from the named evidence vocabulary.

### Decision evidence
The dependency edge set is this stage's own durable output, recorded per `output/README.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
