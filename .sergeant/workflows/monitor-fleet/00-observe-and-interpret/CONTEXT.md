# 00-observe-and-interpret: observe and interpret fleet state

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

A bounded, read-only picture of the fleet's current state, interpreted rather than merely reported: whether Sergeant is verifiably busy right now, and whether each in-progress worker is actually healthy or has stalled past its grace window — with a stalled worker recorded as a non-terminal diagnostic, never acted on automatically. This is the workflow's sole checkpoint (N1 adjudication A4): the snapshot and liveness computations that used to be extracted as their own stages carried no judgment argument beyond the §6.5 boilerplate and fold in below as helper invocations; the judgment this stage actually performs is interpreting what those two mechanical readings mean and deciding how to report it.

Trigger (workflow-level): An operator or another workflow (dispatch's `80-monitor`) needs a live view of the fleet.

## What must become true here (durable outcome)

A bounded, constant-size, versioned, strictly read-only snapshot exists (`busy:true` only with a verified witness, otherwise `busy:null`), each in-progress worker's health is evaluated against identity plus recent meaningful progress with a defined fallback chain, and a worker whose evidence exceeds the grace window is left `in_progress` with a non-terminal "live worker stalled" diagnostic recorded — never an automatic kill — with the whole interpreted picture reported to the caller.

## Behavior contract

- **A bounded, side-effect-free activity snapshot answers exactly one narrow question — is Sergeant verifiably doing work right now — as constant-size versioned JSON, and reports busy:true only when ALL of a stable in_progress status, an exact live worker execution-instance identity match, and recent progress attributable to that exact instance hold together; every other outcome is busy:null, because absence of a verified witness is never treated as proof of idleness.**
  (trigger: an external coordinator or bridge needs a side-effect-free, bounded answer to 'is Sergeant busy right now'; outcome: a caller can trust a busy:true result as verified evidence, and never mistakes an unverifiable observation for a confirmed idle state)
  — `BU-P6-101`, `reference/sergeant-upstream/bin/sgt-watch` (L36-49)
- **`sgt-watch --snapshot` must be strictly read-only, constant-size, and versioned, and must only report the fleet as busy when it has a verified active witness — unlike `--list` (human-oriented, embeds free-form brief text) or `--sync`/`--sync-all` (which mutate lifecycle state), giving a coordinator or bridge a safe machine-readable answer to 'is Sergeant verifiably doing work right now?'.**
  (trigger: an external coordinator or automation needs to know whether Sergeant is currently active, without triggering side effects; outcome: a machine-consumable, side-effect-free, versioned snapshot answer exists, distinct from both the human-oriented listing and the mutating sync commands)
  — `BU-P7-101`, `reference/sergeant-upstream/tests/sgt-watch-snapshot-test.sh` (lines 1-12)
- **Worker health must never be equated with the in_progress status alone; it requires exact live-process identity plus recent, meaningful progress evidence, using a defined fallback chain (session activity, then recorded progress timestamp, then file mtime only as a last resort), and once that evidence exceeds the grace window the worker stays in_progress but a nonterminal 'live worker stalled' diagnostic is recorded rather than an automatic failure or kill.**
  (trigger: an operator or sgt-watch --sync evaluates whether an in_progress worker is actually healthy; outcome: a stalled worker is distinguished from a healthy one using layered evidence, and the distinction is recorded durably as a nonterminal diagnostic rather than acted on automatically)
  — `BU-P8-072`, `reference/sergeant-upstream/docs/using-sergeant.md` (L161-172 (Worker states))

## Judgment required

This is an actor stage (ladder §6.4): distinguishing a genuinely verified `busy:true` witness from an unverifiable observation, and distinguishing a worker that is actually healthy from one whose fallback-chain evidence has exceeded the grace window, both require inspecting the evidence the helpers below produce and choosing among alternatives — they are not mechanical lookups. The escalation choice itself is narrow and read-only: a stalled worker is never killed or otherwise acted on here, only recorded as a non-terminal diagnostic and surfaced to the caller (an operator, or dispatch's `80-monitor`, which owns any actual escalation decision). Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Helper invocations (folded stages, N1 adjudication A4)

The two operations below were extracted as their own candidate stages (ladder §6.5, "deterministic-machinery candidate") but carried no "Additional note" argument that survives §6.3's reimplementation test: swapping either operation's implementation tomorrow would leave this stage's checkpoint — a verified busy/idle answer, and a correctly distinguished healthy/stalled worker — unchanged. They fold in here as ordered helper invocations the acting harness performs (or invokes a script for) before interpreting the result:

1. **snapshot** (formerly `00-snapshot`) — produce the bounded, constant-size, versioned, strictly read-only activity snapshot; `busy:true` only with a verified witness, otherwise `busy:null`.
   - `BU-P6-101`, `BU-P7-101` (see behavior contract above).
2. **evaluate liveness** (formerly `10-evaluate-liveness`) — evaluate each in-progress worker's identity plus recent meaningful progress against the defined fallback chain (session activity, then recorded progress timestamp, then file mtime).
   - `BU-P8-072` (see behavior contract above).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
