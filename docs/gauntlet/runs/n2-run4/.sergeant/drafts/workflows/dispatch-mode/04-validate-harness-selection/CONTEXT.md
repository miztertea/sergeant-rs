# 04-validate-harness-selection

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** a dispatch is about to create worker state

**Outcome:** only a recognized persistent-interactive agent selection is accepted; anything else is rejected before worker state is created

**Statement (the operative rule):** `SERGEANT_AGENT` or the dispatch step may select `opencode`, `oc`, `goose`, `claude`, or an equivalent path whose basename is one of those names; dispatch uses only persistent interactive sessions and rejects every other agent and all non-interactive launch modes before creating worker state.

## What must become true here (durable outcome)

Only a recognized persistent-interactive agent selection is accepted; anything else is rejected before worker state is created — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0136`: Workers always run as persistent interactive TTY sessions; Sergeant never starts one-shot run, prompt, print, or automatic modes.
- `BU-0138`: OpenCode workers are launched with `--dangerously-skip-permissions` because workers run in an automated dispatch context (no operator at the keyboard) and operator trust is scoped at dispatch time by the reviewed intent file and worker brief, which bound what the agent may do.
- `BU-0139`: `--dangerously-skip-permissions` is not a capability grant; the actual trust boundary is the intent file content approved at dispatch time, the worker brief injected into the session, and the repository permissions of the worktree the worker checks out into.
- `BU-0294`: When switching gh auth identity for a repo fails, dispatch records both a `failed:` status and a diagnostic to the repo's fleet directory before dying, rather than exiting with only a terminal error message.
- `BU-0304`: Non-interactive agent launch modes are prohibited for a worker: it never launches `opencode run`, `goose run`, Claude print mode, `--auto`, `--prompt`, or an equivalent one-shot mode, since Sergeant delivers only a fixed ID-bearing notification through interactive terminal input.
- `BU-0321`: The opencode and oc harnesses are launched with --dangerously-skip-permissions by deliberate security-posture decision: the operator's consent is scoped to the intent file and worker brief reviewed and approved at dispatch time, not to per-action interactive confirmation, and the worker cannot escalate beyond that dispatch-time-approved scope.
- `BU-0343`: The harness capability/registry check runs before any model/variant resolution work or durable state creation for the worker, so an unregistered harness fails immediately instead of after state has already been created.
- `BU-0360`: The Claude harness pre-flight capability gate verifies the resolved $AGENT binary itself is present, resolved through the harness path this worker was invoked with (not a hardcoded literal claude) so a fake-CLI test can substitute its own binary the same way every other harness-launch test does.
- `BU-0879`: The interactive agent harness Sergeant launches defaults to auto-detection from the ambient environment (OpenCode markers, then Claude Code session markers, then a hardcoded opencode fallback) unless the operator explicitly overrides it with SERGEANT_AGENT.
- `BU-0894`: Dispatch to an interactive agent dies before proceeding if the configured agent name is not one of the supported harnesses, or if the resolved agent binary is not found on PATH.
- `BU-0895`: When the configured interactive agent is Goose, its support for interactive sessions is verified by live-probing 'goose session --help' rather than assumed; the check dies if the probe does not succeed.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0313`: One shared harness registry defines the accepted-harness capability gate, the readiness probe, and the launch invocation together, so the three can never drift out of sync with each other.
- `BU-0314`: Harness registry validation rejects a row that is malformed or does not declare both a harness name and a readiness probe, naming the offending row and the expected format in the error.
- `BU-0315`: Harness registry validation rejects a row that names a readiness probe not implemented by _sgt_harness_supported_probes, listing the supported probes in the error.
- `BU-0316`: Harness registry validation rejects a harness name declared more than once in the registry.
- `BU-0317`: The harness capability gate validates the entire registry before checking whether the specific requested harness is accepted, and when rejecting an unsupported harness it lists every currently accepted harness name.

