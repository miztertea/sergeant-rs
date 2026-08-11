# 03-publish-spec

## Inputs

| File | Layer | Why |
|---|---|---|
| ../02-sketch-test-seams/output/outcome.md | L4 | upstream evidence produced by `sketch-test-seams` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** the spec has been written using the template

**Outcome:** the spec is immediately actionable in the tracker without a further triage pass

**Statement (the operative rule):** The finished spec is published to the project issue tracker with the `ready-for-agent` triage label applied, and no additional triage step is needed.

## What must become true here (durable outcome)

The spec is immediately actionable in the tracker without a further triage pass — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0994`: A published spec follows a fixed template, containing, in order: Problem Statement, Solution, User Stories, Implementation Decisions, Testing Decisions, Out of Scope, and Further Notes.
- `BU-0995`: The spec's User Stories section is an extremely extensive, LONG numbered list covering all aspects of the feature, each in the form 'As an <actor>, I want a <feature>, so that <benefit>'.
- `BU-0996`: The Implementation Decisions section does not include specific file paths or code snippets, since they may go outdated quickly.
- `BU-0997`: If a prototype produced a snippet (state machine, reducer, schema, type shape) that encodes a decision more precisely than prose can, the snippet is inlined within the relevant decision, noted as having come from a prototype, and trimmed to only its decision-rich parts rather than kept as a working demo.
- `BU-0998`: The spec's Testing Decisions section includes a description of what makes a good test (testing only external behaviour, not implementation details), which modules will be tested, and prior art (similar tests already in the codebase).

