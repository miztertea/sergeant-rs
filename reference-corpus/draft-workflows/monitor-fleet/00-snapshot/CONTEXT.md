# 00-snapshot: snapshot

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

A bounded, constant-size, versioned, strictly read-only snapshot; `busy:true` only with a verified witness, otherwise `busy:null`.

Trigger (workflow-level): An operator or another workflow (dispatch's `80-monitor`) needs a live view of the fleet.

## What must become true here (durable outcome)

A bounded, constant-size, versioned, strictly read-only snapshot; `busy:true` only with a verified witness, otherwise `busy:null`.

## Behavior contract

- **A bounded, side-effect-free activity snapshot answers exactly one narrow question — is Sergeant verifiably doing work right now — as constant-size versioned JSON, and reports busy:true only when ALL of a stable in_progress status, an exact live worker pane identity match, and recent progress attributable to that exact pane hold together; every other outcome is busy:null, because absence of a verified witness is never treated as proof of idleness.**
  (trigger: an external coordinator or bridge needs a side-effect-free, bounded answer to 'is Sergeant busy right now'; outcome: a caller can trust a busy:true result as verified evidence, and never mistakes an unverifiable observation for a confirmed idle state)
  — `BU-P6-101`, `reference/sergeant-upstream/bin/sgt-watch` (L36-49)
- **`sgt-watch --snapshot` must be strictly read-only, constant-size, and versioned, and must only report the fleet as busy when it has a verified active witness — unlike `--list` (human-oriented, embeds free-form brief text) or `--sync`/`--sync-all` (which mutate lifecycle state), giving a coordinator or bridge a safe machine-readable answer to 'is Sergeant verifiably doing work right now?'.**
  (trigger: an external coordinator or automation needs to know whether Sergeant is currently active, without triggering side effects; outcome: a machine-consumable, side-effect-free, versioned snapshot answer exists, distinct from both the human-oriented listing and the mutating sync commands)
  — `BU-P7-101`, `reference/sergeant-upstream/tests/sgt-watch-snapshot-test.sh` (lines 1-12)

> **Read `pane`/`tmux` above as this project's durable execution/session identity, not literally.** Old Sergeant's tmux pane is obsolete here (deviation register D2; `reference-corpus/synthesis.md` §4 clusters M1-M4) — `BU-P6-101` carry a durable identity/liveness/ownership policy that survives the pane; the pane itself does not.

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
