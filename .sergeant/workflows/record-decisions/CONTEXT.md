# Record Decisions
Layer 1 orientation only — it is never delivered as a stage's
instructions; each stage's own `CONTEXT.md` (Layer 2) is the actor's
contract (`docs/icm/convention.md` §1a rule 5).

## Purpose

Transcribe decisions already made — a brief carrying an in-session
grilling's outcome, each decision with its alternatives and rejection
reasons — into ADR/glossary material, with fidelity to that brief as the
review's top-weighted axis.

## Trigger

An in-session grilling (`skills/grilling`) has produced decisions that
need to become durable ADR/glossary material, and the transcription/
write-up is delegated rather than done in the grilling session itself.

## Stages

| Stage | Ladder rung | Durable outcome |
|---|---|---|
| `10-transcribe-decisions` | actor-stage (judgment) | Every decision in the brief is transcribed with its alternatives and rejection reasons; a decision whose rationale the brief does not carry is logged as a gap, never invented. |
| `20-fidelity-review` | actor-stage (judgment) | Every axis named in the brief's authoritative list runs as independent parallel review, with fidelity to the brief weighted above every other axis; outputs unblended. |

## Authority envelope

This workflow receives an already-admitted Work whose intent names a
brief: the record of decisions an in-session grilling already made, with
slots for each decision's alternatives and rejection reasons.

### Workflow may decide
- How to phrase each transcribed decision, alternative, and rejection
  reason, without changing what the brief actually says.
- The ADR/glossary document's own structure and placement, where the
  brief and the repository's own convention leave it open.

### Workflow may not decide
- Make a new decision, resolve an open question, or pick between
  alternatives the brief itself left unresolved — this workflow
  transcribes and reviews decisions already made; it does not make them.
- Invent a rationale, alternative, or rejection reason the brief does
  not carry, to make a decision's record look complete.
- Narrow the fidelity review's axis list below what the brief itself
  names.

### Human or Captain gates
- The brief does not clearly carry a decision's rationale, alternatives,
  or rejection reason (`10-transcribe-decisions`) — logged as a gap, not
  escalated by default, unless the missing material makes the decision
  itself unidentifiable.
- The fidelity review finds the transcription materially diverges from
  the brief (`20-fidelity-review`).

### Decision record
The transcribed ADR/glossary material and the fidelity review's per-axis
findings are this workflow's own durable output; this two-stage workflow
declares no separate decision-log file.
