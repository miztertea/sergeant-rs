# 35-challenge: challenge

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-verify-fidelity-and-facts/output/verification.md | L4 | the verified draft's findings this challenge attacks alongside |
| ../20-draft/output/draft.md | L4 | the draft this stage reads adversarially |

## Purpose

Adversarial read: what does this document fail to accomplish for its
audience?

## What must become true here (durable outcome)

The draft has been read adversarially against its named audience and
purpose; every challenge raised either stands (the draft does not answer
it) or is answered by the draft, per a default-refuted posture.

## Behavior contract

- **Read the draft as its audience would, looking for what it fails to
  accomplish** — not fidelity or facts again (that is `30`'s job), but
  whether the document actually serves the reader it was written for.
  (trigger: `30-verify-fidelity-and-facts` has completed; outcome: a
  distinct, adversarial pass runs after the fidelity/fact check, not
  instead of it)
- **Default-refuted applies: a challenge stands only if the draft does not
  answer it.** Re-read the draft specifically to see whether it already
  addresses the challenge before recording it as open.
  (trigger: a challenge is raised; outcome: only challenges the draft
  genuinely fails to answer survive into the record)
- **This stage does not rewrite the draft.** A surviving challenge is
  recorded for `40-finalize` to act on; rewriting here would collapse the
  adversarial-read stage into the drafting stage it is meant to check.
  (trigger: a challenge survives; outcome: the record stays a record, not
  a silent edit)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
Generating challenges from the named audience's likely perspective, and
judging whether the draft already answers a given challenge.

### J1 — local choices allowed
Formatting of the challenge record.

### J0 — must become `needs_input`
A challenge turns on a question about the audience or purpose itself
that the intent does not resolve.

### Completion boundary
This stage may complete only once every raised challenge has a verdict —
answered by the draft, or standing.

### Decision evidence
`output/challenge.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
