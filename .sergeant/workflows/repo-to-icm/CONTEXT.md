# repo-to-icm — workflow orientation

Layer 1 (`docs/icm/convention.md` §1a). This file orients an actor entering
this workflow for the first time; it is **not** stage instruction — no
stage may substitute reading this for its own `CONTEXT.md` contract, and
only the first stage lists this file in its Inputs table (§1a rule 5).

## What this workflow does

`repo-to-icm` (proposal §9.1) converts a repository's distributed
procedural knowledge — skills, agent instructions, scripts, docs, tests —
into **draft** ICM workflow packages plus an evidence-backed report. It
never publishes a workflow into `.sergeant/workflows/`, never edits the
engine, and never assumes every source artifact deserves an ICM artifact.
Its output is reviewable evidence for a human (or a later, separately
reviewed generator run) to promote — nothing this workflow writes is
runnable procedure until a human crosses the publication boundary
(`docs/icm/convention.md` §2).

## The blindness rule (read before touching anything)

This run's central safety constraint — for a **measurement** run, every
stage's actor is blind to `reference-corpus/`, full stop — is stage
instruction, not orientation, so per §1a rule 5 (Layer 1 "MUST NOT contain
stage instructions") its operative text lives in `_config/run-discipline.md`
(Layer 3), which every stage's own Inputs table names, not here. What
follows is orientation only: expect every stage to be bound by it for the
whole time it runs, and expect a fail-closed `# AMBIGUOUS — NOT RESOLVED`
marker (same file, §2) to propagate down the pipeline rather than be
silently papered over if `00-contract` ever has to fail closed instead of
guessing.

## How the stages hand off

Every stage is an ordinary actor stage (§9.2) — a fresh execution per
stage, given exactly that stage's own pinned context plus the worktree.
There is no shared conversation state between stages; everything one
stage needs from an earlier one is a **named Layer-4 artifact**: the
producing stage writes it under its own `output/`, and every consuming
stage names it explicitly in its own `CONTEXT.md` Inputs table
(`docs/icm/record-shapes.md` §1a). An artifact a later stage needs but
does not name in its Inputs table is not available to it by convention,
whatever a fresh agent might be tempted to go looking for in the
worktree.

```text
00-contract      → output/: repository revision, scope, exclusions,
                   output paths, success criteria
10-inventory     → output/: deterministic source inventory (decompose /
                   helper-evidence / obsolete-candidate / reference-only)
20-harvest       → output/: source-cited behavior units (NDJSON), no ICM
                   form assigned yet
30-normalize     → output/: the same units rewritten independent of old
                   filenames and mechanisms
40-classify      → output/: one classification record per behavior unit,
                   ladder rung + rationale + alternatives considered
50-synthesize    → output/: clustered workflow / stage / context / helper
                   / invariant candidates
60-draft         → output/: materialized draft workflow package(s) under
                   .sergeant/drafts/workflows/, plus provenance.md
70-lint          → output/: validator results + any mechanical repairs
80-adversarial-review → output/: challenge findings (coverage gaps,
                   over-staging, hidden translation, speculative gaps)
90-reconcile     → output/: adjudicated findings + final measurement
                   package; runs scripts/finalize.py as its closing act
```

Each stage's own `output/README.md` declares its expected artifact(s) and
a **disposition** — `promote` (survives this run's own Work-branch merge)
or `evidence` (Work-branch record only). This workflow's outputs are
per-run artifacts: the finalize convention (`docs/icm/convention.md` §1a,
D9) applies to *this workflow's own* stage outputs at the end of a run,
not to the generated draft packages' own (empty, templated) `output/`
directories, which describe artifact shape for that *candidate's* future
runs, not this one.

## Stages

| Stage | Durable outcome (§9.3) |
|---|---|
| `00-contract` | Repository revision, scope, exclusions, output paths, and success criteria are established. |
| `10-inventory` | A deterministic inventory of behavioral artifacts exists; unreadable/generated/vendor regions are identified and excluded. |
| `20-harvest` | Source-cited behavior units are extracted, without assigning ICM forms yet. |
| `30-normalize` | Behavior units are rewritten independently of source filenames and old-implementation mechanisms. |
| `40-classify` | The ICM decomposition ladder (`_config/icm-ladder.md`) is applied to every unit, with rationale and alternatives recorded. |
| `50-synthesize` | Classified units are clustered into workflow, stage, context, helper, and invariant candidates. |
| `60-draft` | Draft workflow package(s) and catalog entries are materialized under the draft namespace. |
| `70-lint` | `scripts/validate-structure.py` is run against the drafted package(s); malformed metadata, broken references, duplicate identities, and missing provenance are repaired. |
| `80-adversarial-review` | A fresh execution challenges coverage, over-staging, hidden file-shape translation, and speculative engine-gap claims. |
| `90-reconcile` | Findings are adjudicated, the final measurement package is emitted, and `scripts/finalize.py` applies this run's own disposition policy. |

## Shared config (`_config/`, Layer 3)

- `_config/run-discipline.md` — the blindness rule and the
  `# AMBIGUOUS — NOT RESOLVED` fail-closed propagation convention. Named in
  every stage's own Inputs table (not just `00-contract`'s), since it binds
  every stage for the whole run, not only the first one.
- `_config/evidence-policy.md` — the citation/quote/hash discipline every
  stage that touches a behavior unit (`20-harvest` onward) must follow.
- `_config/icm-ladder.md` — the §6 decomposition ladder distilled for
  `40-classify`, including the §6.3 reimplementation test and the full
  §6.7 engine-gap template.

## Helpers (`scripts/`)

- `scripts/validate-structure.py` — the §9.7 structural validator. Run with
  no arguments it checks this workflow's own tree; given a path, it checks
  a generated draft tree (used by `70-lint`).
- `scripts/finalize.py` — the D9 disposition finalize helper, run once by
  `90-reconcile` at the close of a run. It is deterministic machinery
  (`docs/icm/convention.md` §5): it reads dispositions, it does not decide
  whether an artifact should exist — that judgment belongs to the stage
  that wrote it.

Both are helpers subordinate to their invoking stage's judgment
(`docs/icm/convention.md` §5) — their exit status and structured output are
something the actor reviews and acts on, not something the engine
interprets on its own. A stage invoking either one states the exact
repository-root-relative invocation and this run's working-directory
convention in its own `CONTEXT.md` (see `70-lint` and `90-reconcile`) — this
orientation file does not itself hand a stranger an executable command.
