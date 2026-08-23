# 05-baseline: baseline

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-orient/output/orientation.md | L4 | the pinned revision and change boundary this baseline is recorded against |

## Purpose

The pre-change state is recorded: which tests exist, which pass, which
command runs them, and what the change is expected to move.

## What must become true here (durable outcome)

A recorded baseline — the discovered test command, which tests currently
pass, and which behavior the change is expected to move — exists before
any implementation commit, so `15-validate` and the test-honesty axis have
something checkable to compare against rather than a retrospective guess.

## Behavior contract

- **Discover and record the command that runs the relevant tests, and run
  it once before any change, recording the real pass/fail state
  verbatim.**
  (trigger: the change's boundary is stated; outcome: a real, re-runnable
  baseline exists rather than an assumed "tests currently pass")
- **State what behavior the change is expected to move**: which currently
  failing or absent behavior should exist afterward, or which currently
  passing behavior must remain passing.
  (trigger: the baseline test run completes; outcome: `15-validate` has a
  concrete target to check the change against, not just "run the tests
  again")

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Discovering the correct test command when the repository does not name
  one obviously (checking `docs/DEVELOPMENT.md`, CI configuration, or
  existing suite conventions).

### J1 — local choices allowed
- Exact wording of the baseline record.

### J0 — must become `needs_input`
- No test command can be discovered anywhere in the repository *and* the
  intent does not name one — stop and ask rather than inventing a
  validation target.

### Completion boundary
This stage may complete only once the test command is recorded, the
baseline run's real output is captured, and the expected-to-move behavior
is stated.

### Decision evidence
The discovered command, the baseline run's verbatim output, and the
expected-to-move statement are recorded in `output/baseline.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
