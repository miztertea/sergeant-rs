# 10-acquire-launch-reservation: acquire launch reservation

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-verify-readiness/output/README.md | L4 | upstream artifact produced by `00-verify-readiness` |

## Purpose

An identity-checked reservation for the exact task/repo pair; concurrent attempts fail closed until the owner exits or stale ownership is proven.

Trigger (workflow-level): Implementation, native tests, lint and independent review are complete and the coordinator has reached the approved shipping boundary.

## What must become true here (durable outcome)

An identity-checked reservation for the exact task/repo pair; concurrent attempts fail closed until the owner exits or stale ownership is proven.

## Behavior contract

- **Before cloning the validation checkout or publishing any launch state, the coordinator must acquire an identity-checked validation-launch reservation for that exact task/repository pair, and concurrent launch attempts fail closed until the recorded owner exits or stale-ownership recovery proves the reservation abandoned.**
  (trigger: two launches of sgt-validate for the same task/repository could race; outcome: only one validation run per task/repository can ever be in flight, and a concurrent second attempt fails closed rather than double-launching)
  — `BU-P8-084`, `reference/sergeant-upstream/docs/using-sergeant.md` (L328-331)
- **Launching the shipping-gate validator is a bounded, gated procedure: a coordinator-owned validation run requires a worker-published readiness marker proving the exact reviewed commit and every review axis passed, an isolated validation code snapshot preserving that exact commit, and either the original dispatching coordinator or an explicitly-claimed replacement to own it — with every precondition checked before any state is committed.**
  (trigger: a worker's code is ready for final shipping-gate validation; outcome: a validation run is only ever launched against exactly the reviewed commit, owned by exactly one accountable coordinator, with an isolated snapshot proving nothing changed underneath it)
  — `BU-P6-129`, `reference/sergeant-upstream/bin/sgt-validate` (L2)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
