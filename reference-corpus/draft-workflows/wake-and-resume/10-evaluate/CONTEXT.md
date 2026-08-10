# 10-evaluate: evaluate

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-validate-condition/output/README.md | L4 | upstream artifact produced by `00-validate-condition` |

## Purpose

One of six typed condition kinds is evaluated; external checks bind to the worker's own recorded remote.

Trigger (workflow-level): A worker is in the `waiting` state with a recorded wake condition.

## What must become true here (durable outcome)

One of six typed condition kinds is evaluated; external checks bind to the worker's own recorded remote.

## Behavior contract

- **Evaluating whether a waiting worker's durable wake condition is satisfied recognizes a fixed set of condition kinds — a timestamp, a GitHub check, a sibling fleet task's completion, a tracked-work task's completion, a deployment (unsupported today), or an explicit human response (never auto-evaluable) — and every kind not recognized is treated as needing human input rather than silently ignored.**
  (trigger: a waiting worker's condition is due for evaluation; outcome: every wake condition either resolves to met, unmet, an adapter error, permanently unsatisfiable (escalate), or explicitly unsupported — never silently stuck)
  — `BU-P6-096`, `reference/sergeant-upstream/bin/sgt-wake` (L9-16)
- **Evaluating an external GitHub check status always binds the query to the worker's own recorded worktree's remote, never to whatever repository the scheduler process happens to be running from, because the scheduler is normally invoked from somewhere other than the worker's own repository.**
  (trigger: a github_check wake condition is evaluated; outcome: a resolution failure is never confused with a genuinely still-pending check)
  — `BU-P6-100`, `reference/sergeant-upstream/bin/sgt-wake` (L284-291)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Additional note

This is the direct source of engine-gap **G1** (survives): the *scheduling* of this stage — periodic re-evaluation without a live process burning a billed turn — is exactly what no lower rung can own. See `reference-corpus/synthesis.md` §5.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
