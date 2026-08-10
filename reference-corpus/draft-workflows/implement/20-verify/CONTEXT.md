# 20-verify: verify

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-implement-with-tdd/output/README.md | L4 | upstream artifact produced by `10-implement-with-tdd` |

## Purpose

Typecheck and focused tests run during implementation; the full suite runs once at the end.

Trigger (workflow-level): Explicitly invoked to implement a defined piece of work (never auto-loaded).

## What must become true here (durable outcome)

Typecheck and focused tests run during implementation; the full suite runs once at the end.

## Behavior contract

- **During implementation, typechecking and single test files should be run regularly, with the full test suite run once at the end.**
  (trigger: implementation work is underway; outcome: fast, frequent local checks are interleaved with work, with one full-suite pass at the close)
  — `BU-P2-053`, `reference/sergeant-upstream/.agents/skills/implement/SKILL.md` (body, lines 11-11)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
