# 00-ingest: ingest

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

The finding set is loaded and refused if it is untyped, if ids are not
unique, or if the intent does not name a human authorization for acting
on it.

## What must become true here (durable outcome)

The finding set is either accepted for processing — typed per §2.7 of
the design record, every id unique, a human authorization named in the
intent — or explicitly refused with the specific reason.

## Behavior contract

- **Refuse a finding set missing any required §2.7 column** (`id`,
  `axis`, `claim`, `evidence`, `severity`, `status`, `refutation`). An
  untyped set is not repaired by this stage; it is refused back to
  whoever produced it.
  (trigger: loading the finding set; outcome: only a genuinely typed set
  proceeds past this stage)
- **Refuse a finding set whose ids are not unique.** A collision makes
  the completeness proof `40-disposition-matrix` exists for unenforceable.
  (trigger: loading the finding set; outcome: every id downstream is
  known to be unambiguous)
- **Refuse to proceed unless the intent itself names the human
  authorization for acting on this finding set.** This workflow does not
  self-authorize from a review's output — the authorization must be
  stated, not inferred from the finding set's mere existence.
  (trigger: the finding set is otherwise well-formed; outcome: scope
  change never transfers silently from a review to a remediation)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
None beyond mechanical validation: typed-column presence, id uniqueness,
and authorization presence are checks, not judgment calls.

### J1 — local choices allowed
Exact wording of the refusal message.

### J0 — must become `needs_input`
The finding set is untyped, has non-unique ids, or the intent names no
human authorization — refuse and state exactly which condition failed,
rather than proceeding on an assumed authorization or a best-effort
repair of a malformed set.

### Completion boundary
This stage may complete only once the finding set is confirmed typed,
uniquely-identified, and authorized — or explicitly refused.

### Decision evidence
`output/ingest.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
