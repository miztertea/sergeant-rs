# 00-name-destination: name destination

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

The destination is named via a grilling session; scope is settled first.

Trigger (workflow-level): A destination is named that requires mapping fog before it can be reached.

## What must become true here (durable outcome)

The destination is named via a grilling session; scope is settled first.

## Behavior contract

- **Charting a wayfinder map first names the destination via a grilling session (settling scope first), then maps the frontier breadth-first across the whole space rather than deep on one thread.**
  (trigger: a loose, oversized idea is presented to be charted; outcome: the destination is fixed before any tickets are drafted)
  — `BU-P4-094`, `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Invocation / Chart the map, L111).
  Upstream names this a "grilling/domain-modeling session"; no
  `domain-modeling` skill package exists in this repo yet (only frozen
  upstream evidence — see `docs/icm/agents-invariant-dispositions.md`
  BU-1064), so sharpening domain terminology folds into the `grilling`
  session below rather than a second invocation.

## Bounded judgment

Apply `@@bounded-judgment`.

### J5 — governing constraint
- The destination is named by running **grilling** live, in-session — never by dispatching a Work item (R-NS-6: conversation is the harness's job, never engine work).

### J2 — delegated to this stage
- None beyond the delegated `grilling` session itself, which carries its own bounded judgment.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers; a `grilling` session's own J0 escalation applies while it runs.

### Completion boundary
This stage may complete only once the destination is named and scope is settled via the grilling session.

### Decision evidence
The named destination is this stage's own durable output, recorded per `output/README.md`.

## Delegation

This stage's outcome is produced by running the **grilling** operator skill
(`skills/grilling/SKILL.md`) to completion, live in this session — not by
dispatching a Work item. `grilling` retired as a `.sergeant/workflows/`
package at the MVP-5 F2 execution-surface re-triage (North Star ruling
R-NS-6: conversation is the harness's job, never engine work; see
`docs/icm/re-homing-record-2026-08-12.md`), which also resolves the E3
dependency this stage previously inherited from the retired package's
WORKFLOW-IF-E3 classification — there is no engine hold to wait on here;
the interview happens directly, in-session, before this stage's durable
outcome (the named destination) is recorded.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
