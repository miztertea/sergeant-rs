# 10-confirm-understanding: confirm understanding

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-interview-loop/output/README.md | L4 | upstream artifact produced by `00-interview-loop` |

## Purpose

An explicit user confirmation gate before any action; decisions landed during the interview are then captured as ADRs/glossary entries per domain-modeling conventions.

Trigger (workflow-level): A plan or design needs interview-style stress-testing that should also produce durable domain artifacts.

## What must become true here (durable outcome)

An explicit user confirmation gate before any action; decisions landed during the interview are captured as ADRs/glossary entries per domain-modeling conventions.

## Behavior contract

No behavior units are cited directly against the confirmation-gate part of this stage; its content is wholly delegated (see Delegation below) or is the workflow's own structural connective tissue. This is recorded explicitly rather than invented to fill the section.

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Delegation

This stage's outcome is produced by running **grilling** to its own completion (context composition today — see `docs/icm/convention.md` §4 on `@@name` versus true nested-workflow invocation, which does not exist yet).

## Helper invocation: capture decisions

Demoted from a standalone stage (`20-capture-decisions`) at N1 adjudication A4: its only stage-level justification was the §6.5 deterministic-machinery boilerplate, with no additional checkpoint argument, so it folds into this stage as a helper invocation performed once understanding is confirmed. No `kind = "execute"` stage exists in the current engine, so the acting harness performs the capture operation itself using domain-modeling's document conventions:

- **grill-with-docs is a workflow that sharpens a plan or design through interview while producing durable design docs (ADRs, glossary) as a side effect.**
  (trigger: user invokes grill-with-docs (or its trigger phrase) on a plan/design; outcome: the plan/design reaches shared understanding and ADR/glossary docs exist for the decisions made)
  — `BU-P3-001`, `reference/sergeant-upstream/.agents/skills/grill-with-docs/SKILL.md` (frontmatter: description)
- **grill-with-docs is defined by composing two other procedures: it runs the grilling interview loop while using the domain-modeling skill to capture ADRs/glossary entries as decisions land.**
  (trigger: grill-with-docs invocation; outcome: a grilling interview occurs and its outputs are captured via domain-modeling's document conventions)
  — `BU-P3-003`, `reference/sergeant-upstream/.agents/skills/grill-with-docs/SKILL.md` (body line 7)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
