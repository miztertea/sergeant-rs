# 15-validate: validate

## Inputs

| File | Layer | Why |
|---|---|---|
| ../05-baseline/output/baseline.md | L4 | the test command and pre-change baseline this run is compared against |
| ../10-implement/output/implementation.md | L4 | the commits this validation runs against |

## Purpose

The targeted validation named at `05` has been run against the change,
with its real output recorded — pass or fail.

## What must become true here (durable outcome)

The test command recorded at `05-baseline` has actually been run against
the implemented change, and its real, verbatim output is recorded —
whichever way it comes out.

## Behavior contract

- **Run the baseline's test command against the change and record the
  real output verbatim.** A failing validation is recorded and carried
  forward, never worked around or silently re-run until it passes by
  accident.
  (trigger: implementation is complete; outcome: an honest pass/fail
  record exists, tied to the specific behavior `05-baseline` said the
  change should move)
- **This stage does not fix a failure it finds.** A failing validation is
  this stage's own honest result; fixing it, if warranted, is
  `30-fix-confirmed`'s job once the panel has weighed in, not a reason to
  loop back here.
  (trigger: the test command fails; outcome: the failure is recorded and
  handed forward rather than patched in place to make this stage look
  clean)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- None beyond ordinary tool mechanics: running the recorded command and
  transcribing its real output is not a judgment call.

### J1 — local choices allowed
- None identified.

### J0 — must become `needs_input`
- The recorded test command from `05-baseline` no longer runs at all
  (missing tool, broken invocation) — stop and ask rather than silently
  substituting a different command.

### Completion boundary
This stage may complete once the test command has been run against the
change and its real output — pass or fail — is recorded.

### Decision evidence
The verbatim test-run output is recorded in `output/validation.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
