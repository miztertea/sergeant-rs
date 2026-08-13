# 05-file-capability-gaps: file capability gaps

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/standing-constraints.md | L3 | constraints binding every stage of this workflow |

## Purpose

Each unsupported capability becomes an approved tracked issue, or is reported as an unfilled gap.

Trigger (workflow-level): First install, a new project/repository to register, a broken or incomplete installation, or a verification request.

## What must become true here (durable outcome)

Each unsupported capability becomes an approved tracked issue, or is reported as an unfilled gap.

## Behavior contract

- **For each unsupported prerequisite, sergeant-setup drafts a td issue (title, description, acceptance criteria) and shows it for explicit y/yes approval before creating it; on decline it reports the gap in the summary and creates no tracked work.**
  (trigger: a required or optional prerequisite is classified unsupported; outcome: either a tracked issue exists or the gap is explicitly reported, never silently dropped or silently created)
  — `BU-P5-012`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 77-89)

## Helper: detect prerequisites (retired MVP-5 F2, 2026-08-12)

This stage previously consumed `00-detect-prerequisites`'s L4 artifact.
That stage retired: the execution-surface re-triage
(`docs/icm/retriage-2026-08-11.md`) and the upstream function map
(`docs/gauntlet/notes/upstream-core-function-map-2026-08-11.md`) both found
its prerequisite-detection job absorbed by shipped `sgt doctor` (git, the
`claude` CLI + version gate, Docker, data dir, journal, projection, daemon,
per-profile permission mode, and — inside an estate — manifest health and
disk pressure; every failing check names its own remedy). This stage now
runs `sgt doctor` itself as its own precondition step (a helper invocation
per `docs/icm/convention.md` §5, not a declared L4 input — `sgt doctor`'s
output is a live command result, not an upstream workflow artifact) and
works from its findings directly: any failing check `sgt doctor` cannot
itself remedy is this stage's "unsupported capability" to file or report.

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
