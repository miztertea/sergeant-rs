# 03-record-canonical-intent: Record Canonical Intent

## Inputs

| File | Layer | Why |
|---|---|---|
| ../02-create-td-tasks/output/ | L4 | upstream artifact from this candidate's own prior stage (create-td-tasks); shape to be fixed at promotion review, see ../02-create-td-tasks/output/README.md |

## Purpose

A dispatch's canonical intent is recorded at dispatch-creation time (BU-0135's trigger: 'a dispatch is created'), after the two preconditions above and before spawn; that same intent then stays stable and governs every later dispatched action -- implementation, review, PR, successor, recovery, shipping-gate (BU-0040, BU-0303).

Source evidence: `BU-0040, BU-0135, BU-0303` -- see `../provenance.md`.

## Output

Declared in `output/README.md` (Layer 4). Not populated at draft time (`../../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
