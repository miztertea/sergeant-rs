# 55-refute: refute

## Inputs

| File | Layer | Why |
|---|---|---|
| ../50-panel/output/findings.md | L4 | the typed finding set this stage attacks |

## Purpose

Every raised finding carries a refuter verdict.

**What "isolated" means here, exactly.** These seats are sub-agents
spawned inside one stage execution, not separate Works. Their isolation
is context isolation only: each seat sees the brief this stage hands it
and nothing of its siblings' reasoning. It is weaker than the isolation a
separate Work would give — the seats share this stage's single execution,
its journal entry, its usage window and its failure; there is no per-seat
journal, no per-seat recovery, and a stage that dies takes every seat's
unwritten output with it. `docs/icm/convention.md` §6.3 places review
independence in the execution boundary: this stage has one execution boundary, not four.
Report this panel as what it is — four isolated
briefs read by one stage — and never as four independent reviews.

## What must become true here (durable outcome)

Every finding in `output/findings.md` carries a final `status` —
`confirmed` only where a refuter seat attacked it and could not overturn
it, `refuted` in every other case.

## Behavior contract

Apply `@@refute`. Standard four-axis refutation over `50-panel`'s
findings: default refuted, one seat per axis spawned in one message
seeing only its own axis's findings, silence never confirms, no new
findings added here.

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
How to phrase each refuter's brief.

### J1 — local choices allowed
Exact invocation wording, so long as all refuters are spawned in one
message.

### J0 — must become `needs_input`
A refuter's verdict turns on a decision only the human can make.

### Completion boundary
Every finding in the set has a `status` and, where `refuted`, a
`refutation`.

### Decision evidence
`output/findings.md`, updated in place.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
