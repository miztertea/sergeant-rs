# 00-release-verification: release verification

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

The drain suite must pass before every push; missing tooling fails closed rather than silently skipping.

Trigger (workflow-level): A push to the source repository is about to happen.

## What must become true here (durable outcome)

The drain suite must pass before every push; missing tooling fails closed rather than silently skipping.

## Behavior contract

- **Before every git push, the drain test suite must run and pass; the push is blocked on failure unless the operator explicitly opts out with git push --no-verify.**
  (trigger: operator runs git push; outcome: push proceeds only after the drain suite passes, or the operator explicitly consents to skipping validation)
  — `BU-P6-007`, `reference/sergeant-upstream/scripts/hooks/pre-push` (L2-11)
- **If the tooling required to run the pre-push validation (mise, docker) is unavailable, the hook fails closed with exit 1 and an actionable message, rather than silently skipping validation and letting the push through.**
  (trigger: required validation tooling is missing on push; outcome: a push with unrunnable validation is blocked with a diagnosis, never silently allowed through)
  — `BU-P6-008`, `reference/sergeant-upstream/scripts/hooks/pre-push` (L29-33, L35-39)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
