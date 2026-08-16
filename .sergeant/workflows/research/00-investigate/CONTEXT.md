# 00-investigate: investigate

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Primary sources only, every claim traced; the investigation ends with one cited Markdown findings file written.

Trigger (workflow-level): A topic needs to be researched, or docs/API facts need gathering, and reading legwork is delegated.

## What must become true here (durable outcome)

Primary sources only, every claim traced; one Markdown file exists with every claim cited, placed per the repo's convention or an explicitly stated choice.

## Behavior contract

- **Research must be conducted against primary sources (official docs, source code, specs, first-party APIs) rather than secondary summaries, with every claim traced back to its owning source.**
  (trigger: the research workflow is investigating; outcome: every claim in the findings traces to a primary source)
  — `BU-P3-042`, `reference/sergeant-upstream/.agents/skills/research/SKILL.md` (item 1, line 10)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Choosing which primary sources are authoritative and tracing every claim
  to its owning source, per `BU-P3-042` above.
- Choosing where the findings file is placed and what "every claim carries
  a citation" means mechanically, per `BU-P3-043`/`BU-P3-044` in the
  Helper invocation section below (cited here by unit id, not restated —
  ICM-R2 pilot review flagged the earlier draft's verbatim duplication of
  this text in both sections as drift-prone).

### J1 — local choices allowed
- Mechanical formatting of the findings file (heading structure, citation
  style), as long as `BU-P3-043` holds.
- The findings file's own filename, chosen inside the stage's assigned
  work surface.

### J0 — must become `needs_input`
- **Any unexpected file, path, or worktree state** — a surface that looks
  git-ignored, missing, unfamiliar, or otherwise inconsistent with what
  this stage was told to expect. This is always a stop-and-ask condition,
  never a reason to search for, infer, or relocate a write target outside
  the stage's own assigned worktree — including into an orchestrating
  session's own live checkout. (Direct fix for the observed failure
  recorded as `GAUNTLET.md` backlog item B9 and ratified by
  `docs/adr/0013-icm-r0-owner-rulings.md` decision 8: a dispatched
  `research` Work once inferred its own surface was "the wrong (ignored)
  location" and wrote into the orchestrating session's active checkout
  instead of asking.)
- Primary sources conflict on a material fact and no higher rung resolves
  which one governs.
- No primary source can be found for a claim the requester needs answered.

### Completion boundary
This stage may complete only when a single Markdown findings file exists,
every claim in it carries a source citation, it has been placed per
`BU-P3-044`, and no `J0` condition above was encountered without first
being raised as `needs_input`.

### Decision evidence
Record material J2 decisions (source-selection rationale, findings
placement choice when no convention exists) in the findings file itself,
under a short "Sources and placement" note. A `J0` stop is recorded in the
turn's own `needs_input` question per `@@bounded-judgment`'s canonical
shape.

## Helper invocation: write findings

Demoted from a standalone stage (`10-write-findings`) at N1 adjudication A4: its only stage-level justification was the §6.5 deterministic-machinery boilerplate, with no additional checkpoint argument, so it folds into this stage as a helper invocation performed while crossing this checkpoint.

**Rung-rationale correction (ICM-R2 pilot review, 2026-08-16):** the prior text here claimed "no `kind = \"execute\"` stage exists in the current engine" as part of this fold's justification. That is false as of this branch: `.sergeant/workflows/repo-to-icm/workflow.toml`'s `65-self-check` is a live `kind = "execute"` stage. Whether "every claim carries a citation" should become a mechanical execute-stage check (analogous to `65-self-check`'s validator, riding after this actor stage) rather than trusted to this stage's own self-check is a real open question the pilot review raised but did not resolve — parked as a follow-on finding (not built here; adding it would be new workflow.toml/execute-stage content beyond this reconciliation pass's own scope), not silently re-asserted as settled. Until that's decided, the acting harness performs the write-and-place operation itself:

- **The investigation's output is a single Markdown file where every claim carries a source citation.**
  (trigger: investigation is complete; outcome: a single cited Markdown findings file exists)
  — `BU-P3-043`, `reference/sergeant-upstream/.agents/skills/research/SKILL.md` (item 2, line 11)
- **The findings file is placed according to the repository's existing note-keeping convention, or in a sensible location (with the choice explicitly stated) if no convention exists.**
  (trigger: the findings file is being saved; outcome: the file lands in a discoverable, convention-consistent (or explicitly justified) location)
  — `BU-P3-044`, `reference/sergeant-upstream/.agents/skills/research/SKILL.md` (item 3, line 12)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
