# Research
Draft workflow package — candidate **W27** `research` from the N1
manual reference-corpus decomposition (`sergeant-rs-workspace/knowledge/evidence/gauntlet/contracts/N1.md`).
This is Layer 1 orientation only — it is never delivered as a stage's
instructions; each stage's own `CONTEXT.md` (Layer 2) is the actor's
contract (`docs/icm/convention.md` §1a rule 5).

## Purpose

Investigate a question against high-trust primary sources and capture the findings as a Markdown file in the repo.

## Trigger

A topic needs to be researched, or docs/API facts need gathering, and reading legwork is delegated.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-investigate` | actor-stage (§6.4, judgment) | Primary sources only, every claim traced; one Markdown file, every claim cited, placed per the repo's convention or an explicitly stated choice. |

## Authority envelope

This workflow receives an already-admitted Work intent: a research
question or documentation/API-fact request, typically delegated by a
Captain session that wants to keep working while sources are read.

### Workflow may decide
- Which primary sources are authoritative for the question, tracing every
  claim back to the source that owns it.
- Where the findings file is placed when the repository has no existing
  note-keeping convention, provided the choice is stated explicitly in the
  file itself.

### Workflow may not decide
- That a location outside its assigned Work surface is ever a valid write
  target, no matter how the surface appears from outside (git-ignored,
  unfamiliar, or otherwise "wrong"-looking) — see the stage's `J0` clause
  below.
- To answer from secondary summaries when primary sources are reachable.

### Human or Captain gates
- Any unexpected file, path, or worktree state is never resolved by the
  workflow alone; it is a stop-and-ask (`needs_input`) condition (see
  `00-investigate/CONTEXT.md`'s `## Bounded judgment`).

### Decision record
Material decisions (source selection, findings placement when no
convention exists, any `J0` stop) are recorded in the stage's own turn and
surfaced through `needs_input` where applicable; this single-stage
workflow declares no separate decision-log file.

## Notes for reviewers

Delegated to a background execution context — that delegation is a *scheduling* property of how research is invoked, not a stage of the procedure itself.

**N1 adjudication A4:** the former `10-write-findings` stage carried only the §6.5 deterministic-machinery boilerplate as its stage-level justification, with no additional checkpoint argument; it is demoted and folded into `00-investigate` as a helper invocation. See `provenance.md`'s "Adjudication A4" section.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
