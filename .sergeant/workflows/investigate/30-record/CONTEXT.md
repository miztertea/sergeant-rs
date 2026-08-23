# 30-record: record

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-synthesize/output/synthesis.md | L4 | the document this stage records durably |
| ../25-challenge/output/challenge.md | L4 | the challenge verdicts folded into the recorded artifact |

## Purpose

The durable repo artifact exists at a named path.

## What must become true here (durable outcome)

The synthesized, challenged document is written to a named, durable path
in the repository — the one place in this workflow that writes outside
`output/`.

## Behavior contract

- **The artifact's path is stated in the intent, or chosen here and
  recorded explicitly** — following the repository's own note-keeping
  convention where one exists, or a sensible, explicitly justified
  location where none does.
  (trigger: the synthesis and challenge are complete; outcome: the
  finished investigation lands in a discoverable location a future reader
  can find without having watched this Work run)
- **The recorded artifact folds in `25-challenge`'s verdicts**: a
  conclusion presented as standing carries that status, and an overturned
  conclusion is corrected or removed rather than left presented as if it
  still held.
  (trigger: writing the durable artifact; outcome: the artifact a future
  reader finds reflects the challenged state, not the pre-challenge draft)
- **This is the one place this workflow writes outside `output/`.** Every
  other stage's output stays Layer 4, Work-branch evidence; this artifact
  is the workflow's actual deliverable to the wider repository.
  (trigger: this stage completes; outcome: exactly one durable,
  repo-visible artifact exists per investigation, at a stated path)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
Choosing the artifact's placement when no repository convention exists
and the intent does not name one.

### J1 — local choices allowed
Mechanical formatting of the recorded artifact.

### J0 — must become `needs_input`
Any unexpected file, path, or worktree state — a surface that looks
unfamiliar or inconsistent with what this stage was told to expect — is a
stop-and-ask condition, never a reason to infer or relocate a write
target outside this stage's own assigned surface.

### Completion boundary
This stage may complete only once the artifact exists at a named,
recorded path and reflects `25-challenge`'s verdicts.

### Decision evidence
The recorded artifact's own path, named in `output/README.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
