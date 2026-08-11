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

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Helper invocation: write findings

Demoted from a standalone stage (`10-write-findings`) at N1 adjudication A4: its only stage-level justification was the §6.5 deterministic-machinery boilerplate, with no additional checkpoint argument, so it folds into this stage as a helper invocation performed while crossing this checkpoint. No `kind = "execute"` stage exists in the current engine, so the acting harness performs the write-and-place operation itself:

- **The investigation's output is a single Markdown file where every claim carries a source citation.**
  (trigger: investigation is complete; outcome: a single cited Markdown findings file exists)
  — `BU-P3-043`, `reference/sergeant-upstream/.agents/skills/research/SKILL.md` (item 2, line 11)
- **The findings file is placed according to the repository's existing note-keeping convention, or in a sensible location (with the choice explicitly stated) if no convention exists.**
  (trigger: the findings file is being saved; outcome: the file lands in a discoverable, convention-consistent (or explicitly justified) location)
  — `BU-P3-044`, `reference/sergeant-upstream/.agents/skills/research/SKILL.md` (item 3, line 12)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
