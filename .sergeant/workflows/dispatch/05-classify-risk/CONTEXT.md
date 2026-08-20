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

**`--intent-file` is real ([issue #166](https://github.com/miztertea/sergeant-rs/issues/166)).** `sgt run --intent-file <path>` reads a file's contents as the intent, verbatim, before any daemon contact — see `sgt run --help`'s own text for the flag (re-verified against the estate-root contract, 2026-08-20). It is pure-content transport: `sgt` validates only mechanics — the leaf must not be a symlink, must be a regular file, is capped at 1 MiB, and must be valid UTF-8 (`src/cli.rs`'s `read_intent_file`) — and neither parses the contents nor requires any particular section structure. The eight-dimension risk brief (Objective, Required Invariants, Approved Tradeoffs, Out Of Scope, State Transitions, Failure Windows, Negative Test Matrix, Validation Evidence) is Captain's own discipline, stated once in AGENTS.md's `### INTENT — Captain's intent discipline` — the one home for that list; this stage points at it rather than restating it. This stage's own obligation is narrower than composing that brief: match the objective's text against the fixed keyword set below, route accordingly, and record in this stage's own output that the safety-sensitive path applied.

- **An objective whose text matches a fixed set of safety-sensitive or stateful keywords (auth, security, secrets, payments, databases, migrations, production, destructive, persistent state, state transitions) cannot proceed on the standard-isolated intent path and must instead be given an explicit --intent-file.**
  (trigger: a work objective is being auto-converted into a minimal standard-isolated intent; outcome: risky-sounding work can never proceed on the lightweight auto-generated intent path; it must be given an explicit, fuller intent document)
- **The dispatch skill must document a `standard-isolated` execution path and name specific trigger keywords (auth, OAuth, security, secret, credential, payment, database, migration, stateful, production, destructive) that route work away from it, must warn against a worker being dispatched before this stage's routing decision has run, and must bound remediation to at most two cycles before escalating (the two-cycle bound is `80-monitor`'s own contract, not restated here).**
  (trigger: a task is about to be dispatched to a worker; outcome: safety-sensitive work is routed through a different, more conservative path than routine implementation, and remediation loops are bounded rather than open-ended)
- **--intent-file is mandatory whenever the objective names auth/OAuth, security, secrets or credentials, payments, databases or migrations, stateful/production work, destructive work, persistent state, or state transitions; `sgt run` refuses the file before any dispatch mutation on exactly four mechanical grounds (symlink, non-regular-file, oversized, non-UTF-8) — it does not check for the eight dimensions, a section structure, or any other content shape — and every other objective uses the lighter standard-isolated path.**
  (trigger: sgt-dispatch is about to launch a worker for a stated objective; outcome: high-risk objectives are routed onto the fuller `--intent-file` path before any worker starts; what actually fails before dispatch is limited to the four mechanical guards above, never a content or completeness check)

## Bounded judgment

Apply `@@bounded-judgment`.

### J5 — governing constraint
- **An objective matching the fixed safety-sensitive keyword set (auth, security, secrets, payments, databases, migrations, production, destructive, persistent state, state transitions) cannot proceed on the standard-isolated path — it must be given an explicit `--intent-file`.** Not a delegated judgment call; the keyword match is fixed. `--intent-file` exists in `sgt run` today ([issue #166](https://github.com/miztertea/sergeant-rs/issues/166)) as pure-content transport with mechanical guards only; composing the eight-dimension brief that content should carry is Captain's own discipline (AGENTS.md's `### INTENT — Captain's intent discipline`), not something this stage — or `sgt` — checks.

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
