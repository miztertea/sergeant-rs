# 07-monitor: monitor

## Inputs

| File | Layer | Why |
|---|---|---|
| ../06-execute/output/README.md | L4 | upstream artifact produced by `06-execute` |

## Purpose

Progress is evidenced by recent meaningful events plus exact process identity.

Trigger (workflow-level): Any task the user brings.

## What must become true here (durable outcome)

Progress is evidenced by recent meaningful events plus exact process identity.

## Behavior contract

- **Monitor real progress: require recent meaningful events or an active child operation plus exact pane/process identity — parent-process liveness alone is insufficient; in OpenCode use a managed background watch and verify it started, falling back to bounded one-shot status checks rather than a blocking watch call when unavailable.**
  (trigger: execution has begun; outcome: progress claims are backed by verified, recent, identity-bound evidence)
  — `BU-P1-032`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L144, step 7)

> **Read `pane`/`tmux` above as this project's durable execution/session identity, not literally.** Old Sergeant's tmux pane is obsolete here (deviation register D2; `reference-corpus/synthesis.md` §4 clusters M1-M4) — `BU-P1-032` carry a durable identity/liveness/ownership policy that survives the pane; the pane itself does not.

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
