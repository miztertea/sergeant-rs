# 40-pin-source: pin source

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-verify-no-conflict/output/README.md | L4 | upstream artifact produced by `30-verify-no-conflict` |

## Purpose

The external skill's source is pinned or locked where the installer supports it.

Trigger (workflow-level): Before adopting an external skill, or when an adopted skill needs updating.

## What must become true here (durable outcome)

The external skill's source is pinned or locked where the installer supports it.

## Behavior contract

- **Pin or lock the external skill's source where the installer supports it.**
  (trigger: no conflict found; outcome: the installed skill version is pinned wherever the tooling allows)
  — `BU-P1-124`, `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L130, vet step 5)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
