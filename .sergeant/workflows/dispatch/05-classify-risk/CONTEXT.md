# 05-classify-risk: classify risk

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-check-queue-and-plan/output/README.md | L4 | upstream artifact produced by `00-check-queue-and-plan` |

## Purpose

The objective is routed to the standard-isolated path or forced onto an explicit intent-file path by a fixed safety-sensitive keyword set.

Trigger (workflow-level): Work spans repositories, contains two or more independent repository-owned tasks, needs an isolated review worker, or the user asks for workers.

## What must become true here (durable outcome)

The objective is routed to the standard-isolated path or forced onto an explicit intent-file path by a fixed safety-sensitive keyword set.

## Behavior contract

**Engine gap (skew-check-2026-08-17 finding 1, [issue #166](https://github.com/miztertea/sergeant-rs/issues/166)):** the three bullets below describe the upstream tool's `--intent-file` mechanism, which does not exist in the shipped `sgt` binary. `sgt run --help`'s full option list is `--data-dir`, `--workflow`, `--backend`, `--json`, `--profile`, `--repo`, `--group`, `--workspace`, `--turns`, `--ceiling-secs` — no intent-transport flag of any kind, and no other verb accepts one either. The only channel that reaches a Work at all today is `sgt run`'s plain-text positional `<INTENT>` argument, which the engine does not validate for required sections, path traversal, symlinks, or size. Until #166 is closed, this stage cannot structurally force a safety-sensitive objective through a validated intent path — the actor can only fold the eight required sections into that plain-text `<INTENT>` string by convention and record, in this stage's own output, that the safety-sensitive path was warranted even though nothing in the CLI enforced it. The keyword-match routing decision itself (below) still stands; what is missing is engine-side enforcement of where that routing leads.

- **An objective whose text matches a fixed set of safety-sensitive or stateful keywords (auth, security, secrets, payments, databases, migrations, production, destructive, persistent state, state transitions) cannot proceed on the standard-isolated intent path and must instead be given an explicit --intent-file.**
  (trigger: a work objective is being auto-converted into a minimal standard-isolated intent; outcome: risky-sounding work can never proceed on the lightweight auto-generated intent path; it must be given an explicit, fuller intent document)
- **The dispatch skill must document a `standard-isolated` execution path and name specific trigger keywords (auth, OAuth, security, secret, credential, payment, database, migration, stateful, production, destructive) that route work away from it, and must warn against mutation happening before validation, and must bound remediation to at most two cycles before escalating.**
  (trigger: a task is about to be dispatched to a worker; outcome: safety-sensitive work is routed through a different, more conservative path than routine implementation, and remediation loops are bounded rather than open-ended)
- **--intent-file is mandatory whenever the objective names auth/OAuth, security, secrets or credentials, payments, databases or migrations, stateful/production work, destructive work, persistent state, or state transitions; the intent file must contain the eight required sections, and malformed, missing, path-traversing, symlinked, or oversized input fails before any dispatch mutation, while every other objective uses the lighter standard-isolated path.**
  (trigger: sgt-dispatch is about to launch a worker for a stated objective; outcome: high-risk objectives are structurally forced through a stricter, validated intent path before any state is created; low-risk objectives use a lighter path)
  — **not implemented by `sgt run`; see engine-gap note above ([#166](https://github.com/miztertea/sergeant-rs/issues/166)).**

## Bounded judgment

Apply `@@bounded-judgment`.

### J5 — governing constraint
- **An objective matching the fixed safety-sensitive keyword set (auth, security, secrets, payments, databases, migrations, production, destructive, persistent state, state transitions) cannot proceed on the standard-isolated path — it must be given an explicit `--intent-file`.** Not a delegated judgment call; the keyword match is fixed. **`--intent-file` itself does not exist in `sgt run` today** ([issue #166](https://github.com/miztertea/sergeant-rs/issues/166), see the engine-gap note under "Behavior contract" above) — until it does, satisfy this constraint by folding the required content into `sgt run`'s plain-text `<INTENT>` argument and recording, in this stage's output, that the safety-sensitive path applied even though the CLI could not structurally enforce it.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers.

### Completion boundary
This stage may complete only when the objective is routed to exactly one path (standard-isolated or explicit-intent-file), per the fixed keyword classification.

### Decision evidence
The routing decision is this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
