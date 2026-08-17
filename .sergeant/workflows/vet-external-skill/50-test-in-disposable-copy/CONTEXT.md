# 50-test-in-disposable-copy: test in disposable copy

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-verify-no-conflict/output/README.md | L4 | upstream artifact produced by `30-verify-no-conflict` (this stage absorbed the demoted `40-pin-source` stage — N1 adjudication A4) |

## Purpose

The external skill's source is pinned or locked where the installer supports it; the skill is then tested in a disposable repository or worktree before broad installation.

Trigger (workflow-level): Before adopting an external skill, or when an adopted skill needs updating.

## What must become true here (durable outcome)

The external skill's source is pinned or locked where the installer supports it; the skill is tested in a disposable repository or worktree before broad installation.

## Behavior contract

- **Test the external skill in a disposable repository or worktree before broad installation.**
  (trigger: source pinned; outcome: the skill is proven in an isolated environment before being broadly installed)

## Helper invocation: pin source

Demoted from a standalone stage (`40-pin-source`) at N1 adjudication A4: its only stage-level justification was the §6.5 deterministic-machinery boilerplate, with no additional checkpoint argument, so it folds into this stage as a helper invocation performed before testing (the source must be pinned before the pinned version is what gets tested).

**Rung-rationale correction (ICM-R3, 2026-08-16):** the prior text here claimed "no `kind = \"execute\"` stage exists in the current engine" as part of this fold's justification. That is false as of this branch: `.sergeant/workflows/repo-to-icm/workflow.toml`'s `65-self-check` is a live `kind = "execute"` stage. Whether this pin/lock fold should instead ride on a `kind = "execute"` stage is the same open question raised and parked at `research/00-investigate/CONTEXT.md`'s equivalent correction, not resolved here. Until that's decided, the acting harness performs the pin/lock operation itself:

- **Pin or lock the external skill's source where the installer supports it.**
  (trigger: no conflict found; outcome: the installed skill version is pinned wherever the tooling allows)

## Bounded judgment

Apply `@@bounded-judgment`.

### J5 — governing constraint
- No broad installation without a prior disposable-copy test.
- Pin/lock the source wherever the installer supports it (mechanical, no alternative to choose among).

### J2 — delegated to this stage
- Judging whether the disposable-copy run is representative enough to trust.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- The disposable-copy test fails: record what failed and ask the user (do not install broadly, re-vet, or reject) rather than proceeding to broad installation or an update stage on a failing test.

### Completion boundary
This stage may complete only once the source is pinned/locked where supported and the skill is proven in a disposable copy — or the stage has stopped at the J0 case above.

### Decision evidence
The test result and pin/lock outcome are this stage's own durable output, recorded per `output/README.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
