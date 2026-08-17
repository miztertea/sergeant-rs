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

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Judging whether a candidate PRD/spec file under `docs/`, `specs/`, or `.scratch/` actually matches the branch or feature name closely enough to count as found, before falling through to the next step in the priority order.

### J1 — local choices allowed
- Exact wording of the question when asking the user for a spec source.

### J0 — must become `needs_input`
- The fixed priority order (issue references in commit messages, a user-passed path, a matching PRD/spec file) is exhausted with nothing found: ask the user rather than guessing or fabricating a spec source.

### Completion boundary
This stage may complete only once a spec source is identified, or the user has explicitly confirmed none exists (in which case the Spec sub-agent is skipped downstream in `20-30-parallel-review`).

### Decision evidence
The identified spec source (or the user's "no spec" answer) is recorded in this stage's own `output/README.md`-declared artifact.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
