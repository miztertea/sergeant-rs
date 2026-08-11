# 30-start-run: start run

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-select-intent-transport/output/README.md | L4 | upstream artifact produced by `20-select-intent-transport` |

## Purpose

A run exists on a feature branch with committed history, a verbatim intent, an initialized repo and a runnable pipeline agent; an in-flight matching run is reattached, never duplicated.

Trigger (workflow-level): Implementation, native tests, lint and independent review are complete and the coordinator has reached the approved shipping boundary.

## What must become true here (durable outcome)

A run exists on a feature branch with committed history, a verbatim intent, an initialized repo and a runnable pipeline agent; an in-flight matching run is reattached, never duplicated.

## Behavior contract

- **The actor then validates, passing the user's task text as `--intent` verbatim, enriched with the decisions and tradeoffs made while doing the work.**
  (trigger: the task is committed on a feature branch; outcome: validation begins with an intent string that captures the user's goal plus the actor's own tradeoff decisions)
  — `BU-P2-062`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Task-first mode step 3, lines 43-47)
- **The work to be validated must already be committed on a branch; the gate validates committed history, not the uncommitted working tree.**
  (trigger: preparing to run the pipeline; outcome: uncommitted changes are never what gets validated)
  — `BU-P2-063`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Before you start: committed work, lines 54-55)
- **The actor must be on a feature branch, not the repository's default branch, before running the pipeline.**
  (trigger: preparing to run the pipeline; outcome: the pipeline is never run directly against the default branch)
  — `BU-P2-064`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Before you start: feature branch, lines 56-56)
- **The repository must already be initialized with `no-mistakes init` before the pipeline can be run.**
  (trigger: preparing to run the pipeline; outcome: an uninitialized repository cannot be validated until `no-mistakes init` has been run)
  — `BU-P2-065`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Before you start: initialized repo, lines 57-57)
- **The daemon must have a runnable configured pipeline agent — a supported native agent binary, the `agent: cursor` ACP alias, or an explicit `acp:<target>` via `acpx` — since the invoking actor is the AXI driver, not an implicit pipeline-agent backend; without one, the run fails before its first step and `no-mistakes doctor` reports the configuration problem.**
  (trigger: preparing to run the pipeline; outcome: the run fails fast with a diagnosable configuration error rather than partway through, if no pipeline agent is configured)
  — `BU-P2-066`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Before you start: pipeline agent, lines 58-62)
- **If any precondition is unmet, `axi run` returns an `error:` with the exact command to fix it, which the actor reads and acts on (committing work, or creating a branch); an uninitialized repo needs `no-mistakes init` first, and a broken `no-mistakes` command itself needs `no-mistakes doctor`.**
  (trigger: a precondition check fails; outcome: the actor is given, and follows, a concrete remediation command rather than guessing)
  — `BU-P2-067`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Precondition failure handling, lines 64-68)
- **Before starting, the actor runs `no-mistakes axi` (home view); if it shows an active run on the current branch, the actor inspects it with `axi status`, and if it is parked at a gate, drives it with `axi respond`.**
  (trigger: about to start a new validation; outcome: an existing in-flight run on the same branch is discovered and handled instead of a redundant new run being started)
  — `BU-P2-068`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Home-view check, lines 69-71)
- **An in-flight run is reattached by re-running `no-mistakes axi run` when it still matches the current HEAD, either as the submitted head or the current pipeline head.**
  (trigger: an active run exists on the current branch and matches HEAD; outcome: the same run is continued rather than a new one started)
  — `BU-P2-069`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Reattach rule, lines 72-72)
- **`axi abort` is only used when the actor means to discard the current run before starting over; it is a between-runs action, never a way to take over or bypass a gate while a run is still going.**
  (trigger: considering whether to abort an active run; outcome: abort is used only to intentionally discard a run, never to sidestep an active gate)
  — `BU-P2-070`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Abort semantics, lines 73-73)
- **If the home view shows an active run on another branch, that run is left alone and the actor starts validation for the current branch independently.**
  (trigger: an active run exists but on a different branch than the current one; outcome: the other branch's run is left untouched while the current branch's validation proceeds independently)
  — `BU-P2-071`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Other-branch runs, lines 74-74)
- **Starting a run requires `--intent`: what the user set out to accomplish, in their own terms — not a description of the diff or changed files — since no-mistakes uses it verbatim rather than inferring it from local agent transcripts, which the source calls slower and flakier.**
  (trigger: starting a validation run; outcome: the run is anchored to a stated human goal rather than an inference over the diff)
  — `BU-P2-072`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Intent is required, lines 78-83)
- **The intent should err toward completeness, not brevity, since the review step uses it to tell a deliberate decision from a mistake: it should capture the user's goal, the specific decisions and tradeoffs made, constraints ruled in or out, and anything explicitly requested that might otherwise look surprising in the diff.**
  (trigger: composing the `--intent` text; outcome: a rich enough intent is provided that the review step can distinguish deliberate choices from mistakes)
  — `BU-P2-073`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Intent completeness, lines 85-92)
- **Starting the run (`no-mistakes axi run --intent "..."`) blocks until the first decision point or the end.**
  (trigger: the run is started; outcome: the actor's call does not return until a gate or a terminal outcome is reached)
  — `BU-P2-074`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Validate-and-decide step 1, lines 98-101)
- **A clean no-mistakes run takes several minutes; invoking it during development or repeatedly restarting it multiplies that cost.**
  (trigger: considering whether to (re)start a no-mistakes run; outcome: runs are started deliberately, not repeatedly, to avoid multiplying cost)
  — `BU-P1-069`, `reference/sergeant-upstream/README.md` (README.md L264, restart cost)
- **Before starting a run: finish and commit on a feature branch, ensure no-mistakes doctor is healthy, and check no-mistakes axi for an already-active matching run — reattach rather than create a duplicate.**
  (trigger: about to start a no-mistakes run; outcome: no duplicate concurrent run is created and preconditions are verified)
  — `BU-P1-070`, `reference/sergeant-upstream/README.md` (README.md L268, start preconditions)
- **sgt-validate's default medium profile skips the redundant no-mistakes review and document stages.**
  (trigger: a validation boundary is launched with the default profile; outcome: review/document stages are not duplicated when the coordinator already covered them)
  — `BU-P1-042`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L154-155)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Additional note

Re-rung from a §6.5 deterministic-machinery classification to an actor stage per N1 adjudication A5 (finding N1-BH-04). The original extraction classified this as boilerplate machinery, but its own behavior contract carries judgment the boilerplate frame denied: discovering, inspecting, and correctly handling an already-active in-flight run — reattach on the current branch, leave alone on another branch, never bypass a gate via `abort` (`BU-P2-068`–`BU-P2-071`) — requires reading actual state and choosing among genuinely different responses; and composing an intent string rich enough for the downstream review step to distinguish a deliberate decision from a mistake (`BU-P2-073`) is exactly the kind of judgment §6.4 names. (The upstream invocation-mode judgment — distinguishing validate-only from task-first — is `00-check-scope`'s own checkpoint, `BU-P2-059`; it is not re-cited here, per record-shapes.md §3's one-unit-one-stage rule. This stage takes that decision as already settled by the time either entry path reaches it.) This survives the §6.3 reimplementation test in the other direction from most of this package's demoted stages: replacing the deterministic precondition checks (repo initialized, feature branch, pipeline agent configured) with different tooling tomorrow would leave this checkpoint's outcome unchanged, but replacing the actor's judgment about in-flight-run handling and intent composition would not — those decisions are the checkpoint.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
