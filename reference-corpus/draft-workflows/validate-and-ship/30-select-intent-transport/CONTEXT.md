# 30-select-intent-transport: select intent transport

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-reserve-isolated-snapshot/output/README.md | L4 | upstream artifact produced by `20-reserve-isolated-snapshot` |

## Purpose

The transport is probed against the installed build's real capability, decided once with explicit consent for the exposing option, recorded twice for audit, and re-checked before the run.

Trigger (workflow-level): Implementation, native tests, lint and independent review are complete and the coordinator has reached the approved shipping boundary.

## What must become true here (durable outcome)

The transport is probed against the installed build's real capability, decided once with explicit consent for the exposing option, recorded twice for audit, and re-checked before the run.

## Behavior contract

- **Whether a validation run's canonical intent is passed to the shipping-gate tool via a private file path or via argv is decided once, probed against the actually-installed build's real capability (not assumed), and requires explicit operator consent to select the argv transport at all, because argv exposes intent content through process listings and /proc.**
  (trigger: a validation launch is about to decide how to pass canonical intent; outcome: intent content is never exposed via argv without the operator's explicit, per-run consent, and the transport decision is grounded in what the installed tool actually supports)
  — `BU-P6-134`, `reference/sergeant-upstream/bin/sgt-validate` (L274-281)
- **Canonical intent content must never appear in process arguments (readable by any local process via ps or /proc/<pid>/cmdline); sgt-validate therefore probes no-mistakes' own --help output before creating a run and requires an --intent-file capability, delivering intent through a path instead of argv.**
  (trigger: the coordinator is about to launch the final validation run; outcome: the intent's content is never exposed to any other local process through argv, by construction)
  — `BU-P8-085`, `reference/sergeant-upstream/docs/using-sergeant.md` (L333-339 (Intent transport and the argv consent gate))
- **The transport actually used for a validation launch is recorded twice for durable audit: a current-run marker cleared only on retry-reset, and an append-only log of timestamp/transport/HEAD/intent-revision for every committed launch, so the privacy decision stays auditable across retries; the validation worker also re-checks the recorded transport against the build that will actually run, so a no-mistakes binary swapped between launch and run can neither silently downgrade a private transport into argv nor invoke a flag that build no longer accepts.**
  (trigger: a validation run's launch-time transport decision must still hold true at actual run time; outcome: a TOCTOU-style swap of the no-mistakes binary between launch and run can never silently weaken the privacy guarantee already recorded)
  — `BU-P8-087`, `reference/sergeant-upstream/docs/using-sergeant.md` (L348-355)
- **sgt-validation-worker must pass the canonical intent to no-mistakes via an `--intent-file` path rather than inline content in argv (so intent content never appears in process arguments/listings), and capability-probing invocations of no-mistakes (--version, `axi run --help`) must not count as a validation run.**
  (trigger: sgt-validation-worker launches no-mistakes with the canonical intent; outcome: intent content (which may include sensitive implementation detail) never transits argv where it would be visible to other users on the same host via process listings)
  — `BU-P7-105`, `reference/sergeant-upstream/tests/sgt-validation-worker-test.sh` (lines 37-38)
- **Before invoking the shipping-gate tool, the validation worker re-reads and re-hashes the canonical intent file and refuses to proceed if its content changed since the coordinator's own initial verification, so a content edit made in the gap between launch and run is caught rather than silently validated.**
  (trigger: the validation worker is about to invoke no-mistakes; outcome: the intent that gates a validation run is guaranteed byte-identical to the intent verified at launch, with no window for a late substitution)
  — `BU-P6-042`, `reference/sergeant-upstream/bin/sgt-validation-worker` (L172-183)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
