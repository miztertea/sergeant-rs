# 01-launch-validation-run

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** a dispatched worker reaches readiness

**Outcome:** exactly one validation-only boundary runs, in a split pane, with redundant stages skipped by default

**Statement (the operative rule):** After readiness, the coordinator uses the project-validation step to launch the single validation-only boundary in a split pane of the worker's tmux window; its default medium profile skips the redundant validation pipeline `review` and `document` stages.

## What must become true here (durable outcome)

Exactly one validation-only boundary runs, in a split pane, with redundant stages skipped by default — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0043`: Remediation that changes HEAD still requires independent rereview before updating the readiness marker, but must not trigger repeated validation pipeline review cycles.
- `BU-0079`: The validation pipeline is used as a final shipping gate, not an implementation loop: implementation, focused repository-native tests, lint, and independent review must be complete before starting it.
- `BU-0080`: Before starting the validation pipeline run: finish and commit on a feature branch, ensure the validation pipeline is healthy, and check the validation pipeline for an already-active matching run to reattach to rather than creating a duplicate.
- `BU-0081`: `--yes` is never used when starting the validation pipeline; `--skip=<steps>` is used only for stages already proven irrelevant, since skipping is not a substitute for checks that were never performed.
- `BU-0082`: Routine dispatched workers do not invoke the validation pipeline for ordinary completion, prototypes, investigations, documentation drafts, intermediate commits, or remediation loops; the coordinator starts a single run only after the implementation branch is committed and native validation is complete.
- `BU-0161`: The project-validation step splits the worker's existing tmux window, renames it to `validation-<repo>-<task>`, and runs the validation pipeline interactively in the new coordinator-owned pane with the canonical intent; it never uses `--yes`, and its default medium profile skips `review`/`document` (already covered by required independent reviews and readiness evidence) unless an explicit `--skip` replaces the default list.
- `BU-0170`: Validation is treated as validation-only: each actionable finding is routed into separate, deduplicated owning-repository task tracker work, source is never modified inside the retained validation run, and high-risk findings are escalated rather than approved.
- `BU-0181`: If shared daemon credentials cannot access one repository, the global GitHub account is never switched while unrelated runs are active; an approved repo-scoped method, waiting, or an explicit manual-shipping override is used instead.
- `BU-0182`: A one-shot `GH_TOKEN` for `gh` and a one-shot credential helper for Git are preferred, and the global GitHub account is never switched while other workers may invoke GitHub operations.
- `BU-0376`: For a worker whose recorded status is not done or needs_input (i.e. it is expected to still be running), validation launch requires the recorded worker pane identity to still match a live pane; a done or needs_input worker is exempt from this check since its pane is expected to already be dead.
- `BU-0382`: The --skip value is interpolated unquoted into the constructed validation-worker launch command deliberately, because printf %q would escape a comma (a brace-expansion special character) in a way that would not match the literally-recorded pane_start_command later compared against it; SKIP is independently constrained to a safe [a-z]+(,[a-z]+)* pattern beforehand.
- `BU-0383`: If splitting a new validation pane beside the worker's own pane fails and the worker's own status is done (its pane expectedly already dead), the project-validation step falls back to opening a new tmux window instead, rather than failing the validation launch outright.
- `BU-1197`: The validation pipeline gate validates code changes through a pipeline (intent, rebase, review, test, document, lint, push, PR, CI) before they reach the configured push target, driven through the validation pipeline command family which prints machine-readable TOON to stdout and progress to stderr.
- `BU-1198`: When the user invokes `/no-mistakes`, the agent must report the pipeline's outcome at the end.
- `BU-1199`: If the user asks for something specific (e.g. "skip the lint step"), the agent must translate that request into the matching pipeline-automation tool flags itself, such as `--skip=lint`.
- `BU-1200`: In validate-only mode (bare `/no-mistakes`), the user's code changes are already committed; the agent validates them as-is and reports the outcome, without doing any task work first.
- `BU-1201`: In task-first mode, before changing or committing anything the agent must inspect `git status`, preserve unrelated pre-existing uncommitted changes, and when committing include only the changes belonging to the user's task.
- `BU-1202`: In task-first mode the agent must commit its work on a feature branch; if the user is currently on the repository's default branch, the agent must create a feature branch first, because the gate validates committed history on a non-default branch.
- `BU-1203`: In task-first mode, once the work is committed the agent validates it by passing the user's task text as `--intent`, enriched with the decisions and tradeoffs made while doing the work.
- `BU-1204`: The gate validates committed history, not the uncommitted working tree, so the work being validated must already be committed on a branch.
- `BU-1205`: The user must be on a feature branch, not the repository's default branch, for the validation pipeline run to proceed.
- `BU-1206`: The repository must already be initialized with the validation pipeline before a run can proceed.
- `BU-1207`: The daemon must have a runnable configured pipeline agent (a supported native agent binary, the `agent: cursor` ACP alias, or an explicit `acp:<target>` through `acpx`); the invoking agent is the AXI driver, not an implicit pipeline-agent backend, and if none is available the run fails before its first step, with the validation pipeline reporting the configuration problem.
- `BU-1208`: If any precondition is not met, the pipeline-automation tool returns an `error:` with the exact command needed to fix it, which the agent must read and act on (commit the work, or create a branch).
- `BU-1209`: If the repository is not initialized, the agent must run the validation pipeline first.
- `BU-1210`: If the validation pipeline command itself is missing or misbehaving, the validation pipeline reports what is wrong.
- `BU-1211`: Before starting a new run, the agent must run the validation pipeline (the home view).
- `BU-1212`: If the home view shows an active run on the current branch, the agent inspects it with the validation pipeline.
- `BU-1213`: If the active run is parked at a gate, the agent drives it with the validation pipeline.
- `BU-1214`: The agent may reattach an in-flight run by re-running the validation pipeline when it still matches the current `HEAD`, either as the submitted head or as the current pipeline head.
- `BU-1215`: The validation pipeline is only for discarding a run before starting over; it is a between-runs action and must never be used to take over or bypass a gate while a run is still going.
- `BU-1216`: If the home view shows an active run on another branch, the agent leaves that run alone and starts validation for its own current branch with the validation pipeline.
- `BU-1217`: Starting a run requires `--intent`, describing what the user set out to accomplish (not a description of the diff), passed verbatim from what the agent knows from the conversation rather than left for the validation pipeline to infer from local agent transcripts.
- `BU-1218`: The agent must err on the side of completeness rather than brevity in `--intent`, capturing the user's goal, the specific decisions and tradeoffs made, constraints ruled in or out, and anything explicitly requested that might otherwise look surprising in the diff, because the review step uses `--intent` to tell a deliberate decision apart from a mistake.
- `BU-1219`: Starting the run with the validation pipeline blocks until the first decision point or the run's end.

