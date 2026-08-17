# 60-update-owned: update owned

## Inputs

| File | Layer | Why |
|---|---|---|
| — | — | no contract-bearing upstream dependency beyond this workflow's ordering |

## Purpose

For Sergeant-owned skills: update this repository through a reviewed PR and run the instruction-policy test plus the full test suite.

Trigger (workflow-level): Before adopting an external skill, or when an adopted skill needs updating.

## What must become true here (durable outcome)

For Sergeant-owned skills: update this repository through a reviewed PR and run the instruction-policy test plus the full test suite.

## Behavior contract

- **For Sergeant-owned skills, update this repository through a reviewed PR and run the repository's own instruction-policy test plus the full test suite.**
  (trigger: updating a Sergeant-owned skill; outcome: no Sergeant-owned skill changes ship without passing review and the full test suite, including the instruction-policy test)

  **Source-fidelity correction (ICM-R3, 2026-08-16):** the upstream text
  names a literal path, `tests/instruction-policy-test.sh`. This
  repository's own `tests/` does not contain that script (it contains
  `estate_routes.rs`, `m1_event_core.rs`, ... `t2_workflow_catalog.rs`);
  that literal path is frozen source-project evidence, not this
  repository's own live tooling. This package is generic per-repository
  guidance: run whichever instruction-policy check and full test suite the
  *target* repository (the one whose Sergeant-owned skill is being
  updated) actually names, not this literal upstream path.

## Bounded judgment

Reclassified from `stage (§6.3, deterministic-machinery candidate)` to actor-stage at N1 adjudication A4: the checkpoint here is not running the test suite (deterministic machinery) but the decision that gates it — updating only through a reviewed PR, i.e. human review plus a passing instruction-policy test and full suite before changes ship. That decision survives any reimplementation of how the tests themselves are run (§6.3's test), so it is genuine judgment (PL-5).

Apply `@@bounded-judgment`.

### J5 — governing constraint
- No Sergeant-owned skill change ships without a reviewed PR and a passing instruction-policy test plus full suite.

### J2 — delegated to this stage
- Identifying which instruction-policy check and test suite the target repository actually names, per the source-fidelity correction above.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers.

### Completion boundary
This stage may complete only once the change is submitted through a reviewed PR and the instruction-policy test plus full suite both pass.

### Decision evidence
The PR link and test results are this stage's own durable output, recorded per `output/README.md`.

## Additional note

Alternate entry: only reached when updating an already-adopted, Sergeant-owned skill, not during the initial `00`-`50` vetting sequence. Mutually exclusive with `60-update-managed`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
