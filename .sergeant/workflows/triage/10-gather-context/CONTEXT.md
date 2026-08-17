# 10-gather-context: gather context

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only, since `00-show-attention` was demoted into this stage — N1 adjudication A4) |

## Purpose

Three fixed attention buckets are shown, oldest first; the item and prior notes are read; an already-implemented check and out-of-scope-KB concept match are run.

Trigger (workflow-level): An item is at the front of one of the three fixed attention buckets, oldest first.

## What must become true here (durable outcome)

Three fixed attention buckets are shown, oldest first; the item and prior notes are read; an already-implemented check and out-of-scope-KB concept match are run.

## Behavior contract

- **Triaging a specific item begins by fully reading the item and prior triage notes, exploring the codebase via its domain glossary and ADRs, and running two checks: whether the behavior is already implemented (by domain concept, not literal wording) and whether the request resembles a prior recorded out-of-scope rejection.**
  (trigger: a specific issue or PR is being triaged; outcome: the actor has full context plus a redundancy verdict and a prior-rejection match (if any) before recommending)
- **Matching a new issue against the out-of-scope KB is done by concept similarity rather than literal keyword overlap.**
  (trigger: gather-context's prior-rejection check runs; outcome: a conceptually similar but differently-worded request is still recognized as a match)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Judging whether the behavior is already implemented (by domain concept, not literal wording) and whether the request conceptually matches a prior out-of-scope rejection.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers.

### Completion boundary
This stage may complete only when the item and prior notes are fully read, the already-implemented check has run, and the out-of-scope-KB match check has run.

### Decision evidence
The redundancy verdict and any prior-rejection match are this stage's own durable output.

## Helper invocation: show attention

Demoted from a standalone stage (`00-show-attention`) at N1 adjudication A4: its only stage-level justification was the §6.5 deterministic-machinery boilerplate, with no additional checkpoint argument, so it folds into this stage as a helper invocation performed before gathering context on the selected item. No `kind = "execute"` stage exists in the current engine, so the acting harness performs the query itself:

- **When asked what needs attention, the workflow queries the tracker and presents three fixed buckets ordered oldest-first.**
  (trigger: the maintainer asks what needs attention; outcome: three ordered buckets of attention-worthy items are shown)
- **The third discovery bucket is needs-info items where the reporter has posted activity since the last triage notes, signaling they need re-evaluation.**
  (trigger: an item is in needs-info and the reporter has replied; outcome: the item surfaces in the attention list for re-evaluation)
- **The discovery bucket filter excludes non-external PRs, but this filter applies only to unprompted discovery — an explicitly named PR is triaged regardless of who authored it.**
  (trigger: PRs are included in the attention buckets; outcome: internal PRs never appear via discovery, but can always be triaged by explicit request)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
