# 05-escalate-undecided-seam: Escalate Undecided Seam

## Inputs

| File | Layer | Why |
|---|---|---|
| ../04-spawn-worker/output/ | L4 | upstream artifact from this candidate's own prior stage (spawn-worker); shape to be fixed at promotion review, see ../04-spawn-worker/output/README.md |

## Purpose

Triggered while a worker is already running and needs to establish an undecided public behavioral seam -- necessarily after spawn.

Source evidence: `BU-0281` -- see `../provenance.md`.

## Output

Declared in `output/README.md` (Layer 4). Not populated at draft time (`../../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` "What never happens").
