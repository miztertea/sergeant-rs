# 06-choose-intent-transport

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the project-validation step is about to create a validation run

**Outcome:** the default transport path never exposes intent content via process argv

**Statement (the operative rule):** Canonical intent must not appear in process arguments, where any local process can read it from `ps` or `/proc/<pid>/cmdline`; before creating a validation run, the project-validation step probes the validation pipeline and requires `--intent-file`, which delivers the intent through a path instead of argv.

## What must become true here (durable outcome)

The default transport path never exposes intent content via process argv — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0164`: When the installed validation pipeline does not offer `--intent-file`, the launch fails closed and names the required capability, the observed version, the observed flag surface, and the operator's options; no run, marker, or state change is created.
- `BU-0165`: `--allow-argv-intent` consents, for that invocation only, to delivering the intent through `--intent`, accepting the exposure; consent is a flag rather than an environment variable so it cannot be exported once and silently reapplied to later runs.
- `BU-0166`: The transport actually launched is recorded twice — `validation_intent_transport` for the current run (cleared on retry-reset) and an append-only owner-only `validation_transport.log` of every committed launch — and the validation worker re-checks the recorded transport against the build that will actually run, so the validation pipeline binary replaced between launch and run can neither downgrade the private transport into argv nor invoke a flag that build rejects.
- `BU-0324`: The set of flags the validation pipeline's own `run` subcommand accepts is discovered by parsing its own --help output, not by inferring capability from a version number.
- `BU-0325`: The intent transport is resolved by preferring the private --intent-file flag when the installed validation pipeline build supports it; the argv --intent flag is only selected when --intent-file is unavailable AND the operator has explicitly consented (allow_argv=true); otherwise resolution fails.
- `BU-0375`: The intent transport (private intent-file vs. consented argv) is resolved and validated before any validation run, marker, or state change exists, specifically so that an incompatible validation pipeline build cannot record a failed run.
- `BU-0384`: The intent transport actually used for a validation run (intent-file or consented argv) is recorded to an append-only audit log as the last publication step of a committed launch; the comment notes over-recording is the conservative direction for this privacy-relevant decision even though a failure at this step still rolls the whole launch back.
- `BU-0389`: The validation worker re-checks the coordinator's recorded intent transport decision against the actually-installed validation pipeline build's real capability, so a binary swapped between launch time and run time can neither downgrade the private intent-file transport into argv nor invoke a flag the installed build rejects; the recorded decision is honored exactly and never re-optimized here.
- `BU-0394`: With the intent-file transport only the intent's file PATH reaches the validation pipeline's argv, so intent content never appears in ps or /proc/<pid>/cmdline; the argv transport is only reachable at all because the coordinator passed --allow-argv-intent, and it does expose the full intent content through those same surfaces.

