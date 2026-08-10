# 10-await-convergence: await convergence

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-set-drain/output/README.md | L4 | upstream artifact produced by `00-set-drain` |

## Purpose

A bounded wait; a worker counts as drained only when its exit is provable; timeout leaves the drain active, exits non-zero, and names the unresolved.

Trigger (workflow-level): An operator needs to freeze new stage/turn admission — globally or for one project — before a disruptive operation.

## What must become true here (durable outcome)

A bounded wait; a worker counts as drained only when its exit is provable; timeout leaves the drain active, exits non-zero, and names the unresolved.

## Behavior contract

- **A worker is only ever counted as having genuinely finished draining when its recorded process is provably gone; absence of recorded identity is explicitly not treated as proof of exit, so an unverifiable worker blocks a drain wait rather than being silently counted as resolved.**
  (trigger: a bounded drain wait is evaluating whether the scope is fully drained; outcome: a drain wait can never falsely report success because a worker's identity happened to be unrecordable)
  — `BU-P6-064`, `reference/sergeant-upstream/bin/sgt-drain` (L147-152)
- **A drain refuses new worker starts within its scope while still storing incoming responses generation-safely for later delivery, --wait activates the drain and then waits for in-scope live workers to finish their current turn and exit, and on timeout it leaves the drain active, exits nonzero, and names the unresolved workers without terminating any of them.**
  (trigger: an operator wants to pause admission of new work, optionally waiting for a graceful stop; outcome: admission is refused immediately, in-flight responses are never lost, and a timed-out graceful wait never silently force-stops anything)
  — `BU-P8-077`, `reference/sergeant-upstream/docs/using-sergeant.md` (L231-243 (Pause admission with a drain))

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
