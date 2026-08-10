# 10-write-findings: write findings

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-investigate/output/README.md | L4 | upstream artifact produced by `00-investigate` |

## Purpose

One Markdown file, every claim cited, placed per the repo's convention or an explicitly stated choice.

Trigger (workflow-level): A topic needs to be researched, or docs/API facts need gathering, and reading legwork is delegated.

## What must become true here (durable outcome)

One Markdown file, every claim cited, placed per the repo's convention or an explicitly stated choice.

## Behavior contract

- **The investigation's output is a single Markdown file where every claim carries a source citation.**
  (trigger: investigation is complete; outcome: a single cited Markdown findings file exists)
  — `BU-P3-043`, `reference/sergeant-upstream/.agents/skills/research/SKILL.md` (item 2, line 11)
- **The findings file is placed according to the repository's existing note-keeping convention, or in a sensible location (with the choice explicitly stated) if no convention exists.**
  (trigger: the findings file is being saved; outcome: the file lands in a discoverable, convention-consistent (or explicitly justified) location)
  — `BU-P3-044`, `reference/sergeant-upstream/.agents/skills/research/SKILL.md` (item 3, line 12)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
