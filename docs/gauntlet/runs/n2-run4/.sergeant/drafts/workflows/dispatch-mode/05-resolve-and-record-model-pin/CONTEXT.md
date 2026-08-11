# 05-resolve-and-record-model-pin

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** a dispatch is being created

**Outcome:** the model is resolved deterministically by explicit precedence, and unpinned dispatches are explicitly recorded as such

**Statement (the operative rule):** The dispatch step or `SERGEANT_MODEL` pins the harness model as `provider/model[:variant]`, resolved with precedence `--model` > `SERGEANT_MODEL` > the harness's ambient default; an unpinned dispatch is recorded as `unpinned` rather than left blank, with no project-level model default.

## What must become true here (durable outcome)

The model is resolved deterministically by explicit precedence, and unpinned dispatches are explicitly recorded as such — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0059`: A model tuple the selected harness cannot honor fails before any intent file, the task tracker task, worktree, or fleet state is created; a worker already handed an unhonorable tuple fails terminally instead of inheriting the ambient default.
- `BU-0067`: Every model transport (opencode/oc argv, goose env vars, claude unmeasured) is measured on a host with that harness installed, never inferred from documentation, and Sergeant validates against it before creating any state.
- `BU-0068`: For opencode, Sergeant writes a generated agent definition pinning both model and variant into fleet state, never the worktree, so a pinned variant cannot leave untracked files in the repository under review, and points the harness at it with `OPENCODE_CONFIG`.
- `BU-0069`: A model/variant pin fails closed in exactly two distinct situations — 'no known transport' (harness measured, no way to pin that axis) and 'unmeasured' (harness not installed here) — and the diagnostic states which one applies.
- `BU-0071`: `launch_state` is `intended` before the harness executes and becomes `confirmed` only once the harness reports itself ready, so a harness that rejects a pin and exits never leaves evidence claiming the model ran.
- `BU-0072`: `variant_verified` is always `false` today because no supported harness reports back which variant it resolved, so Sergeant records the pin without claiming it was honored.
- `BU-0074`: The model tuple is resolved from the flag, the environment, or the unpinned default, explicitly only — there is deliberately no project-level model default.
- `BU-0345`: When the resolved harness cannot honor the pinned model tuple, the worker writes the rejection reason to a diagnostic file and to both the fleet-state and worktree status sentinels as an explicit "failed: cannot honor pinned model tuple" before exiting, rather than launching with a substituted model.
- `BU-0346`: The launch record's launch_state field distinguishes intent from proof: it is written "intended" before the harness process is invoked and promoted to "confirmed" only once the harness is observably ready, so a harness that rejects the pin and exits never leaves evidence claiming the model actually ran; variant_verified is always recorded false because no supported harness reports back which variant it resolved.
- `BU-0361`: Before spending a real API call, the Claude harness pre-flight rejects any model value that does not match a bare known alias (sonnet/opus/haiku/fable) or a full claude-<family>-<version> ID, as defence-in-depth against a bypass of the normal tuple-parsing path.
- `BU-0880`: A pinned model is honored only on a launch transport Sergeant has actually measured for that harness; a harness whose model-launch transport is recorded as unmeasured fails closed rather than silently passing the pin through.
- `BU-0881`: A pinned model tuple whose provider does not match a harness's fixed provider scope is rejected before any launch is attempted.
- `BU-0882`: A pinned variant is honored only on a variant transport that is known and measured for the target harness; a harness whose variant transport is unknown or unmeasured fails closed on a variant pin.
- `BU-0883`: When a harness's model transport does not carry the provider segment in its own argv (argv-bare), the launch records the provider as unverified rather than treating it as confirmed, because the invocation itself does not prove which provider was used.
- `BU-0884`: A pinned variant carried via a generated harness agent-definition is written into Sergeant's own fleet state, never into the worktree under review, so pinning a variant cannot leave untracked files in the repository being worked on.
- `BU-0887`: A worker dispatch that cannot honor its requested harness/model pin dies with an actionable diagnostic before any intent file, the task tracker task, worktree, or fleet state is created for it.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0066`: The model tuple is written `provider/model` with an optional `:variant`, using a restricted charset (`[a-z0-9-]` for providers, `[A-Za-z0-9._-]` for models); a tuple outside that charset is rejected.
- `BU-0070`: Dispatch records the resolved model tuple in `agent_model` and its origin in `agent_model_source`; the worker records what it actually launched in `launch_record` (harness, model, provider, both transports, the generated definition, and exact argv/environment used).
- `BU-0197`: GitHub CLI identity for dispatch resolves in strict precedence: `repo.identity` → `project.identity` → `config.default_identity` → no-op.
- `BU-0885`: The generated harness agent-definition config file is written to a temporary file first and only then renamed into place, so a reader never observes a partially-written config; on any write failure the temporary file is removed.
- `BU-0886`: A pinned agent tuple (provider/model[:variant]) must match a deliberately narrow character set; a tuple containing a shell metacharacter, whitespace, or anything outside that charset is rejected rather than passed through into durable launch evidence.

