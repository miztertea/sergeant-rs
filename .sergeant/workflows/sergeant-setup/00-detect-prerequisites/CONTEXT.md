# 00-detect-prerequisites: detect prerequisites

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |
| ../_config/standing-constraints.md | L3 | constraints binding every stage of this workflow |

## Purpose

Every checked tool is classified present / installable / unsupported; required gaps stop the run unless the user accepts the risk.

Trigger (workflow-level): First install, a new project/repository to register, a broken or incomplete installation, or a verification request.

## What must become true here (durable outcome)

Every checked tool is classified present / installable / unsupported; required gaps stop the run unless the user accepts the risk.

## Behavior contract

- **Phase 1 classifies every checked prerequisite as present, installable (a supported bootstrap command exists), or unsupported (file a tracked issue).**
  (trigger: Phase 1 (prerequisite detection) runs; outcome: every prerequisite has one of three dispositions before Phase 2 begins)
  — `BU-P5-009`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 56-57)
- **Required prerequisites for Sergeant are: bash 3.2+, git, an authenticated gh, tmux, yq, lsof, a td implementation supporting --description/--json/--work-dir, and at least one interactive agent (opencode, goose, or claude).**
  (trigger: Phase 1 runs; outcome: the exact required-tool set for Sergeant to operate is enumerated)
  — `BU-P5-010`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 59-68)
- **Optional prerequisites (mise, treehouse, graphify, no-mistakes, node/npm) are skipped, not failed, when absent.**
  (trigger: Phase 1 runs; outcome: absence of an optional tool never blocks setup)
  — `BU-P5-011`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 70-75)
- **sergeant-setup does not continue past Phase 1 until every required prerequisite is present or the user explicitly accepts the risk of proceeding without it.**
  (trigger: Phase 1 completes its checks; outcome: Phase 2 never begins on an unresolved required-prerequisite gap without explicit risk acceptance)
  — `BU-P5-013`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 91-92)
- **Dependency checking distinguishes required tools, whose absence fails the check with a nonzero exit, from optional tools, whose absence only warns, so an operator can tell which gaps block use of Sergeant from which merely reduce functionality.**
  (trigger: operator runs the dependency check task; outcome: the operator sees a pass/fail per dependency with an explicit required-vs-optional distinction and an install hint)
  — `BU-P6-003`, `reference/sergeant-upstream/mise.toml` (tasks.check, L148-154)
- **The mise dependency check must fail (nonzero exit, mentioning the missing tool and 'MISSING') when `td` is absent, and separately must reject an installed `td` binary whose `create --help` output does not advertise `--work-dir` support as unsupported.**
  (trigger: a developer or CI runs `mise run check`; outcome: an actionable, specific failure is reported before any other setup or work proceeds, rather than a downstream cryptic error)
  — `BU-P7-025`, `reference/sergeant-upstream/tests/mise-check-test.sh` (lines 101-113)
- **The dependency check must accept any of OpenCode, Claude, or Goose as a satisfying agent harness, and must fail with a specific 'install OpenCode, Goose, or Claude' message when none is present.**
  (trigger: a developer or CI runs `mise run check`; outcome: at least one of a fixed, named set of agent harnesses is guaranteed present before any procedure that depends on an interactive agent can proceed)
  — `BU-P7-026`, `reference/sergeant-upstream/tests/mise-check-test.sh` (lines 115-140)
- **First-install requires a fixed set of tools (Bash>=3.2, Git, authenticated gh, tmux, yq, Python 3, lsof, Marcus td, and at least one of OpenCode/Goose/Claude Code as a persistent interactive worker terminal) plus optional tools (mise, Treehouse, Graphify, no-mistakes, Node/npm), and installation does not proceed past this check until an automated or manual verification confirms every required tool is present.**
  (trigger: a new local user begins installing Sergeant; outcome: installation cannot proceed until the required-tool set is confirmed present)
  — `BU-P8-041`, `reference/sergeant-upstream/docs/getting-started.md` (L6-27 (Prerequisites))
- **Installation must not continue past the prerequisite stage until `td create --help` proves Marcus td support for --description, --json, and --work-dir, and at least one supported agent executable resolves on PATH.**
  (trigger: the prerequisite verification step completes; outcome: installation only proceeds once the specific capability surface it depends on is proven present, not merely assumed from a tool's name being on PATH)
  — `BU-P8-042`, `reference/sergeant-upstream/docs/getting-started.md` (L51-53)

Per A11 and the workflow-level `CONTEXT.md`'s "Notes for reviewers" section: read `pane`/`tmux` in `BU-P5-010` and `BU-P8-041` above as this project's durable execution/session identity, not literally — old Sergeant's tmux pane is obsolete here (deviation register D2). That reading rule is stated once at the workflow level rather than repeated per stage.

## Judgment required (kept per N1 adjudication A4)

This stage was originally extracted as "Deterministic-machinery candidate" (§6.5), but that classification does not survive §6.3's reimplementation test: the behavior contract's actual gate — "does not continue past Phase 1 until every required prerequisite is present **or the user explicitly accepts the risk of proceeding without it**" (`BU-P5-013`) — requires an actor to ask the user and weigh a risk-acceptance exception, structurally identical to the confirm-then-act pattern this package's other stages (`05`, `10`, `20`, `30`, `40`, `60`, `70`) are already classified "Judgment required" for. Swapping the detection tool (a different shell probe, a different mise task) would leave that risk-acceptance gate completely unchanged — it is not incidental machinery, it is the checkpoint. **Kept as an actor stage; reclassified from §6.5 to §6.4.** See `provenance.md`'s "Adjudication A4" section.

The original Additional note is preserved unaffected by this reclassification: `BU-P7-026` is cross-referenced from the shared-context map (§3b `skill-discovery`, per synthesis.md's † marker) rather than owned here — it is repeated because the same probe rule (accept any of OpenCode/Claude/Goose as a satisfying harness) is this stage's own prerequisite check.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
