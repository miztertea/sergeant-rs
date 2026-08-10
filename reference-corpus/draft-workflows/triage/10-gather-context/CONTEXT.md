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
  — `BU-P3-065`, `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 70)
- **Matching a new issue against the out-of-scope KB is done by concept similarity rather than literal keyword overlap.**
  (trigger: gather-context's prior-rejection check runs; outcome: a conceptually similar but differently-worded request is still recognized as a match)
  — `BU-P3-089`, `reference/sergeant-upstream/.agents/skills/triage/OUT-OF-SCOPE.md` (line 75)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Helper invocation: show attention

Demoted from a standalone stage (`00-show-attention`) at N1 adjudication A4: its only stage-level justification was the §6.5 deterministic-machinery boilerplate, with no additional checkpoint argument, so it folds into this stage as a helper invocation performed before gathering context on the selected item. No `kind = "execute"` stage exists in the current engine, so the acting harness performs the query itself:

- **When asked what needs attention, the workflow queries the tracker and presents three fixed buckets ordered oldest-first.**
  (trigger: the maintainer asks what needs attention; outcome: three ordered buckets of attention-worthy items are shown)
  — `BU-P3-062`, `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 58)
- **The third discovery bucket is needs-info items where the reporter has posted activity since the last triage notes, signaling they need re-evaluation.**
  (trigger: an item is in needs-info and the reporter has replied; outcome: the item surfaces in the attention list for re-evaluation)
  — `BU-P3-063`, `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 62)
- **The discovery bucket filter excludes non-external PRs, but this filter applies only to unprompted discovery — an explicitly named PR is triaged regardless of who authored it.**
  (trigger: PRs are included in the attention buckets; outcome: internal PRs never appear via discovery, but can always be triaged by explicit request)
  — `BU-P3-064`, `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 64)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
