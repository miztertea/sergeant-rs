# 10-identify-spec-source: identify spec source

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-pin-fixed-point/output/README.md | L4 | upstream artifact produced by `00-pin-fixed-point` |

## Purpose

The spec source is identified via a fixed priority order ending in asking the user.

Trigger (workflow-level): A diff needs review before merge (invoked directly or delegated from `worker-mission`/`implement`).

## What must become true here (durable outcome)

The spec source is identified via a fixed priority order ending in asking the user.

## Behavior contract

- **The spec source for the Spec axis is located in a fixed priority order: issue references in commit messages, then a path the user passed as an argument, then a PRD/spec file under docs/, specs/, or .scratch/ matching the branch or feature name, then — if nothing is found — asking the user; if the user says no spec exists, the Spec sub-agent is skipped and reports 'no spec available'.**
  (trigger: identifying what the Spec axis should compare against; outcome: a spec source is found, or the Spec review is explicitly skipped with a reason)
  — `BU-P2-007`, `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Step 2: Identify the spec source, lines 27-32)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Walk the fixed priority order (commit-message issue references, user-supplied path, matching PRD/spec file under `docs/`, `specs/`, or `.scratch/`) and select the first match (`BU-P2-007`).

### J1 — local choices allowed
- The exact matching heuristic (e.g. glob pattern) used when scanning for a PRD/spec file by branch or feature name.

### J0 — must become `needs_input`
- No spec source is found by any of the three automated steps: ask the user where the spec is (`BU-P2-007`). If the user confirms none exists, record "no spec available" — that recorded answer is what lets `20-30-parallel-review` skip the Spec sub-agent at J4, not a fresh J0 there.

### Completion boundary
This stage may complete only when a spec source is identified, or the user has explicitly confirmed none exists.

### Decision evidence
Write the identified spec source (or the user's "no spec" answer) to `output/README.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
