# 40-undrain: undrain

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-force-stop/output/README.md | L4 | upstream artifact produced by `30-force-stop` |

## Purpose

Undrain is idempotent, with mutually exclusive scopes.

Trigger (workflow-level): An operator needs to freeze new stage/turn admission — globally or for one project — before a disruptive operation.

## What must become true here (durable outcome)

Undrain is idempotent, with mutually exclusive scopes.

## Behavior contract

- **Removing a drain is explicitly idempotent: undraining a scope that is not currently drained still exits successfully, and --global and a named project are mutually exclusive scopes that cannot both be targeted in one invocation.**
  (trigger: operator runs sgt-undrain for a project or --global; outcome: admission for the given scope is restored, or was already restored, with the same successful outcome either way)
  — `BU-P6-015`, `reference/sergeant-upstream/bin/sgt-undrain` (L8-9, L47)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
