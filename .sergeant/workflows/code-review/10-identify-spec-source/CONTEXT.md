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
- Judging whether a candidate PRD/spec file under `docs/`, `specs/`, or `.scratch/` genuinely matches the branch or feature name — the priority order names the search locations, but recognizing a match among several plausible files is not mechanical.

### J1 — local choices allowed
- None identified: the priority order (commit-message issue refs, then user-supplied path, then a matching PRD/spec file, then asking the user) is fully specified and admits no equivalent local variants.

### J0 — must become `needs_input`
- No spec source is found anywhere in the priority order: ask the user whether one exists. If the user says no, the Spec sub-agent is skipped and reports "no spec available" rather than the stage inventing a source.

### Completion boundary
This stage may complete only once a spec source is identified, or the user's "no spec" answer is recorded and the Spec axis is marked skipped.

### Decision evidence
Record the identified spec source — or the skip and the user's reason — in `output/README.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
