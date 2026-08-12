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
20-harvest       → output/: source-cited behavior units (NDJSON, no ICM
                   form assigned yet), a per-partition checkpoint ledger,
                   and a consequence-class sweep record
30-normalize     → output/: the same units rewritten independent of old
                   filenames and mechanisms
40-classify      → output/: one classification record per behavior unit,
                   ladder rung + rationale + alternatives considered
50-synthesize    → output/: clustered workflow / stage / context / helper
                   / invariant candidates
60-draft         → output/: materialized draft workflow package(s) under
                   .sergeant/drafts/workflows/, plus provenance.md
65-self-check    → output/: this workflow's own validate-structure.py
                   result (kind = "execute" — a pinned container, not an
                   actor turn; see its own CONTEXT.md)
70-lint          → output/: validator results against each candidate
                   package, plus any mechanical repairs; folds in
                   65-self-check's already-run self-check result rather
                   than re-running the validator against this workflow's
                   own tree itself
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
| `65-self-check` | (`kind = "execute"`, N4) `scripts/validate-structure.py` is run, mechanically, against this workflow's own tree; the result is written for `70-lint` to read. |
| `70-lint` | `scripts/validate-structure.py` is run against the drafted package(s); malformed metadata, broken references, duplicate identities, and missing provenance are repaired. `65-self-check`'s already-run result covers this workflow's own tree. |
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
  no arguments it checks this workflow's own tree — this is what
  `65-self-check`'s pinned container runs, mechanically, as a `kind =
  "execute"` stage (N4), not an actor turn; given a path, it checks a
  generated draft tree (used by `70-lint`, as an actor turn — an
  execute-stage container has no dynamic candidate list to loop over, so
  the per-candidate checks stay actor-driven).
- `scripts/finalize.py` — the D9 disposition finalize helper, run once by
  `90-reconcile` at the close of a run. It is deterministic machinery
  (`docs/icm/convention.md` §5): it reads dispositions, it does not decide
  whether an artifact should exist — that judgment belongs to the stage
  that wrote it. Before removing anything it now verifies every file it is
  about to remove is already reachable in a committed tree, and refuses
  outright (fail-closed, nothing modified) rather than deleting a file that
  was never actually committed — see the module docstring and
  `docs/gauntlet/runs/n2-run2/grammar-pressure-report.md` GP-5b.
- `scripts/test-finalize-evidence-guard.py` — a standalone sandbox test
  proving `finalize.py`'s evidence-preservation guard holds in both
  directions (refuses on an uncommitted file, proceeds on a committed one).
  Not invoked by any stage; run by a human or CI directly, and by
  `validate-structure.py`'s `[S15]` check against this workflow's own tree.

All three are helpers subordinate to their invoking stage's judgment
(`docs/icm/convention.md` §5) when an actor invokes them directly — their
exit status and structured output are something the actor reviews and
acts on, not something the engine interprets on its own. `65-self-check`
is the one exception, by design (N4, §11.2): as a `kind = "execute"`
stage it invokes `validate-structure.py` directly as a pinned container
command, and *there* the exit code genuinely is what the engine reads to
decide the stage outcome — the mechanical case §11.2 carves out on
purpose, distinct from every actor-invoked use of the same script
elsewhere in this workflow. A stage invoking one of the first two as an
actor states the exact repository-root-relative invocation and this run's
working-directory convention in its own `CONTEXT.md` (see `70-lint` and
`90-reconcile`; `65-self-check`'s own `CONTEXT.md` states its pinned
container's exact command instead) — this orientation file does not
itself hand a stranger an executable command.

## v2: how `20-harvest` handles volume (read before assuming one turn is enough)

N2 run 2 covered 16 of 136 `decompose`-dispositioned files in one `20-harvest`
turn before its context window closed (`docs/gauntlet/runs/n2-run2/
grammar-pressure-report.md` GP-1). The adjudicated fix is authoring
guidance, not an engine change — two shapes were weighed:

- **Rejected: split `20-harvest` into a fixed sequence of numbered
  partition-slice stages** (e.g. `20a-harvest-p1` … `20e-harvest-p5`, one
  per `10-inventory` partition, each a genuinely fresh actor turn). This
  requires committing, at workflow-*authoring* time, to a maximum partition
  count — but `10-inventory/CONTEXT.md` is explicit that this workflow "is
  not scoped to any one repository shape," so the partition count a future
  subject repository's inventory produces is unbounded and unknowable in
  advance. A repository whose inventory yields more partitions than the
  provisioned slots reproduces the identical volume-wall failure this fix
  exists to close, merely relocated to a higher, less visible threshold —
  and every slot a smaller repository doesn't need is a wasted stage
  directory, `CONTEXT.md`, Inputs table, and `output/README.md` to author
  and maintain. It also multiplies engine-visible stage identity for
  something that fails §6.3's own reimplementation test applied to the
  *stage boundary itself*: "harvest partition 3" is not a checkpoint
  operators would care about independently of "harvest partition 1" — it is
  the same checkpoint kind repeated, homogeneous iteration inside one
  parent, not distinct procedural structure (the same conclusion
  grammar-pressure-report.md's §21.8 section reaches: this is iteration, not
  workflow composition).
- **Chosen: keep `20-harvest` as the single stage it already is, and give it
  an explicit per-partition checkpoint-and-retry protocol** (below). This
  matches a mechanism the engine already ships and that run 2's own journal
  exercised (by accident, via a different bug) and proved works: a stage
  that cannot finish within one turn stops cleanly at a partition boundary,
  writes a durable, `promote`d partition ledger recording exactly which
  partitions are done and which remain, and is re-entered as a **fresh
  execution** — fresh actor turn, fresh context window — via `sgt work
  retry`, picking up from the ledger rather than starting over. This scales
  to however many partitions a given repository's inventory actually
  produces, with no author-time upper bound, at the cost of relying on a
  human (or an orchestrating caller) to notice an incomplete ledger and
  issue the retry — which is the same shape `00-contract`'s own fail-closed
  `# AMBIGUOUS — NOT RESOLVED` marker already relies on (a human or caller
  acting on a durable, explicit signal this workflow writes, not a new
  engine primitive). It does **not** depend on the actor-initiated
  mid-turn ask primitive GP-2 confirms is not yet wired to a real harness —
  the actor never tries to pause *inside* a turn; it simply ends its turn
  early and honestly, the same way `10-inventory`'s existing "On volume"
  section already does for a single-turn stage.

`20-harvest/CONTEXT.md` and `20-harvest/references/partition-checkpoint-
protocol.md` carry the operative version of this protocol; `_config/
icm-ladder.md` and this file are the record of the choice.
