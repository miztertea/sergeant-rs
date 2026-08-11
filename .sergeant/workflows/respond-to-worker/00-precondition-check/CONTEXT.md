# 00-precondition-check: precondition check

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Exact question read, only genuinely missing decisions asked, decision recorded in tracked work, no unconsumed generation already pending.

Trigger (workflow-level): A worker has published an escalation and a human decision exists.

## What must become true here (durable outcome)

Exact question read, only genuinely missing decisions asked, decision recorded in tracked work, no unconsumed generation already pending.

## Behavior contract

- **Before responding to a worker, the operator must: read the exact finding/question and recommendation; ask only for missing product, risk, security, privacy, destructive, or irreversible decisions; record the decision in the owning td task; verify no unconsumed response generation already exists; and after sending, require the matching worker to acknowledge/consume it.**
  (trigger: a worker enters needs_input and an operator is about to respond; outcome: a response is only ever sent after this exact precondition checklist, and only ever asks about the categories of decision that actually require a human)
  — `BU-P8-079`, `reference/sergeant-upstream/docs/using-sergeant.md` (L253-262 (Respond to a worker))

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
