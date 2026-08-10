# 20-capture-decisions: capture decisions

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-confirm-understanding/output/README.md | L4 | upstream artifact produced by `10-confirm-understanding` |

## Purpose

Decisions landed during the interview are captured as ADRs/glossary entries per domain-modeling conventions.

Trigger (workflow-level): A plan or design needs interview-style stress-testing that should also produce durable domain artifacts.

## What must become true here (durable outcome)

Decisions landed during the interview are captured as ADRs/glossary entries per domain-modeling conventions.

## Behavior contract

- **grill-with-docs is a workflow that sharpens a plan or design through interview while producing durable design docs (ADRs, glossary) as a side effect.**
  (trigger: user invokes grill-with-docs (or its trigger phrase) on a plan/design; outcome: the plan/design reaches shared understanding and ADR/glossary docs exist for the decisions made)
  — `BU-P3-001`, `reference/sergeant-upstream/.agents/skills/grill-with-docs/SKILL.md` (frontmatter: description)
- **grill-with-docs is defined by composing two other procedures: it runs the grilling interview loop while using the domain-modeling skill to capture ADRs/glossary entries as decisions land.**
  (trigger: grill-with-docs invocation; outcome: a grilling interview occurs and its outputs are captured via domain-modeling's document conventions)
  — `BU-P3-003`, `reference/sergeant-upstream/.agents/skills/grill-with-docs/SKILL.md` (body line 7)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
