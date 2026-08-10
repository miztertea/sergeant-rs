# 20-reserve-isolated-snapshot: reserve isolated snapshot

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-acquire-launch-reservation/output/README.md | L4 | upstream artifact produced by `10-acquire-launch-reservation` |

## Purpose

Validation runs against an isolated snapshot pinned at the reviewed commit with a clean tree, re-verified immediately before invocation.

Trigger (workflow-level): Implementation, native tests, lint and independent review are complete and the coordinator has reached the approved shipping boundary.

## What must become true here (durable outcome)

Validation runs against an isolated snapshot pinned at the reviewed commit with a clean tree, re-verified immediately before invocation.

## Behavior contract

- **Validation runs against a code-cloned isolated snapshot, not the worker's live worktree — created via a shared, no-checkout local clone then checked out at the exact reviewed commit, with an owner marker recorded inside the clone's own git directory — so a shipping-gate run can never observe a worktree the worker continues to mutate concurrently.**
  (trigger: a validation run needs a code snapshot to validate against; outcome: a shipping-gate run's verdict is always about a genuinely frozen snapshot of exactly the reviewed commit, never a moving worktree)
  — `BU-P6-133`, `reference/sergeant-upstream/bin/sgt-validate` (L829-845, L855-858)
- **The isolated validation code snapshot's identity is re-verified against the reviewed commit immediately before invoking the shipping-gate tool — the snapshot must still be at the exact reviewed HEAD and have a clean tree — so validation can never silently run against code that changed after review.**
  (trigger: the validation worker is about to invoke no-mistakes against the isolated snapshot; outcome: the exact code that was reviewed is the exact code that gets validated — no substitution window)
  — `BU-P6-044`, `reference/sergeant-upstream/bin/sgt-validation-worker` (L123-129)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
