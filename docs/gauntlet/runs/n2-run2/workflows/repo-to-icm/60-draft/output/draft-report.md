# 60-draft report

Produced by `60-draft` from `../50-synthesize/output/candidates.md` (which
did not open with `# AMBIGUOUS — NOT RESOLVED`, so ordinary drafting
proceeded per `../_config/run-discipline.md` §2).

## Manifest — materialized draft workflow packages

All three of bucket 1's workflow candidates were materialized as packages
under `.sergeant/drafts/workflows/` (this run's own worktree), checked
before writing against `.sergeant/workflows/` — only `repo-to-icm` itself
exists there — and against each other: no name collisions, matching
`../50-synthesize/output/candidates.md` bucket 1's own preamble.

| Candidate | Path | Member stages | Notes |
|---|---|---|---|
| `dispatch-mode` | `.sergeant/drafts/workflows/dispatch-mode/` | 1 (`10-dispatch-worker`) | |
| `standard-task-workflow` | `.sergeant/drafts/workflows/standard-task-workflow/` | 5 (`10-load-context`, `20-check-queue`, `30-reconcile-existing-state`, `40-validate`, `50-reconcile-and-deliver`) | `40-validate`'s exact step-number placement (5, 6, 7, or 8) is an unresolved ordering call inherited verbatim from `50-synthesize` — see the package's own `CONTEXT.md`/`provenance.md`. |
| `ship-with-no-mistakes` | `.sergeant/drafts/workflows/ship-with-no-mistakes/` | 0 | See "Judgment call" below — materialized as a workflow shell with `stages = []`, not skipped. |

Each package's own `provenance.md` maps every stage (and the workflow as a
whole) to its member `behavior_id`(s), citing `../40-classify/output/
classifications.ndjson` directly — no invented citations, and every
stage/workflow with no direct `stage`- or `workflow`-representation
evidence is marked as such rather than left silent.

## Judgment call: materializing `ship-with-no-mistakes` with zero stages

`../50-synthesize/output/candidates.md` bucket 1 lists `ship-with-no-
mistakes` as a workflow candidate with "no member stage candidates" and
calls this "itself the finding worth surfacing for this candidate."
`../60-draft/CONTEXT.md`'s own instruction is unqualified: "Every workflow
candidate from `50-synthesize` is materialized as a package... Permanent-
instruction, obsolete-mechanism, and engine-pressure candidates... are
**not** materialized as packages (they are not workflows)." `ship-with-
no-mistakes` is a bucket 1 *workflow* candidate, not a member of any of
those three excluded categories, so it was materialized rather than only
carried through as a list entry.

Neither `CONTEXT.md` nor `references/draft-package-template.md` states a
minimum stage count for a package, so a workflow shell with `workflow.toml`
`stages = []` and no `NN-<stage-name>/` directories was written, with the
zero-stage situation stated plainly in the package's own `CONTEXT.md`,
`index.md` (`tags: [..., no-member-stages]`), and `provenance.md`, rather
than either (a) silently skipping this candidate's materialization, or (b)
inventing stage candidates from the four stage names visible only in its
unattached `stage-context` records (`start-run`, `drive-gates`,
`finish-run`, `route-findings`) — neither of which this run's evidence or
this stage's own contract supports. This reading is recorded here as an
ambiguity for `70-lint`/`80-adversarial-review`/`90-reconcile` to weigh:
the alternative reading — that a workflow with zero classified checkpoints
should be carried through in this report rather than materialized as a
package at all — is facially plausible and was not obviously foreclosed by
this stage's own contract.

## Carried through unchanged — not materialized as packages

Per `../50-synthesize/output/candidates.md`, verbatim from that stage
(this stage does not edit these lists).

### Bucket 4 — Permanent-instruction candidates (13 `agents-invariant` records, all sourced from `AGENTS.md`)

| ID | Statement |
|---|---|
| `BU-0001` | Before acting on a project, resolve its repositories, roles, inherited instructions, and configured paths through the project's context-resolution step, rather than inferring ownership from the current directory. |
| `BU-0002` | The primary session coordinates multi-repository work by default, and may implement directly only when the user explicitly asks to work in-session (or asks not to dispatch) and one repository owns the complete outcome. |
| `BU-0004` | In direct mode, the default branch is never edited; a feature branch is created or reused before the first implementation change. |
| `BU-0005` | Every direct-mode implementation requires opening a PR and satisfying required CI, review threads, and merge authorization before delivery is considered complete. |
| `BU-0006` | Every directive written into this instruction file must specify at least one of: a trigger/condition, a required or prohibited action, or evidence/a stop condition proving compliance. |
| `BU-0007` | A bare toolbelt command is used when it resolves on PATH; otherwise the matching local script is run instead. Manual operations are used only when no toolbelt command covers the operation or the command returns an explicit unsupported-case error, and that fallback plus the original error evidence must be reported. |
| `BU-0008` | Procedural skills are loaded only when their stated trigger condition applies. |
| `BU-0009` | For every listed skill trigger, the repository-local skill definition file is read directly; it is canonical and takes precedence over any same-named registry skill. |
| `BU-0010` | A harness registry's omission of a skill does not make the skill unavailable, and is not by itself grounds to ask the owner or stop; the actor stops and reports only the exact repository-local path when that file is itself absent or unreadable, and does not reconstruct a partial protocol from memory. |
| `BU-0017` | Deferred waits publish a durable wake condition and resume automatically once it is satisfied, while human decisions resume through an explicit response-delivery step; a waiting worktree is never cleaned, and an expected blocked exit is never rewritten as orphaned. |
| `BU-0018` | Every dispatched implementation, independent review, PR description, successor, recovery, and final shipping gate must use the same canonical intent revision from a shared intent file, and workers and remediation loops never run the shipping-gate tool themselves. |
| `BU-0020` | A completed, merged, blocked, or abandoned task is never left recorded as in_progress; the task tracker and fleet state are reconciled truthfully. |
| `BU-0021` | Tool absence produces an actionable fallback or explicit blocker, never a silent skip, false success, or indefinite wait. |

### Bucket 6 — Obsolete-mechanism findings

None. Zero `representation: obsolete-mechanism` records exist in
`../40-classify/output/classifications.ndjson` for this corpus. Empty for
this run, not skipped.

### Bucket 7 — Engine-pressure candidates

None. Zero `representation: engine-gap` records exist in this corpus, and
every record's `engine_gap` field is `null`. Empty for this run, not
skipped.

## Meta-level grammar pressure, for `90-reconcile`

The materialized packages under `.sergeant/drafts/workflows/` are this
run's principal deliverable, yet the D9 disposition/finalize mechanism
(`docs/icm/convention.md` §1a) only governs a stage's own `output/` — it
has no lower-rung way to give per-run content written elsewhere in the
worktree a disposition, or to bring it under `../scripts/finalize.py`'s
reach. This is stated here per `../60-draft/CONTEXT.md`'s own instruction,
for `90-reconcile` to write up fully against `../_config/run-discipline.md`
and `90-reconcile/references/reconciliation-method.md` §3.

## Structural note

Per `../60-draft/output/README.md`, the materialized packages themselves
are not this stage's own `output/` artifact — they live under
`.sergeant/drafts/workflows/`. This file (`draft-report.md`) is this run's
pointer and carry-through record, not a copy of the packages' content.
