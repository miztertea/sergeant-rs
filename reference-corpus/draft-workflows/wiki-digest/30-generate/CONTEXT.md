# 30-generate: generate

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-inspect-preview/output/README.md | L4 | upstream artifact produced by `20-inspect-preview` |

## Purpose

Synthesis, never a transcript; collected from every configured source with unavailable ones silently skipped.

Trigger (workflow-level): A digest is due (scheduled) or explicitly requested; or the schema/logic changed and needs a dry run first.

## What must become true here (durable outcome)

Synthesis, never a transcript; collected from every configured source with unavailable ones silently skipped.

## Behavior contract

- **The non-dry-run digest command is only run once the dry-run preview satisfies the schema.**
  (trigger: a dry-run preview has been reviewed; outcome: curated pages are only ever written from a preview that already passed review)
  — `BU-P5-140`, `reference/sergeant-upstream/skills/wiki/SKILL.md` (line 53)
- **The digest must synthesize outcomes, decisions, blockers, and next state; it must never reproduce the conversation as a transcript.**
  (trigger: digest content is being generated; outcome: curated pages stay synthesized and durable, never a raw transcript dump)
  — `BU-P5-143`, `reference/sergeant-upstream/skills/wiki/SKILL.md` (lines 57-58)
- **Synthesizing a daily activity digest is a bounded procedure that collects session content from every configured AI-agent history source for one day (silently skipping any source that is unavailable), enriches it with merged pull requests and completed tracked-work items for that day, and produces one synthesized markdown page per day rather than a raw log dump.**
  (trigger: operator or a scheduled job wants a synthesized record of a day's work; outcome: one durable, synthesized (not raw) markdown page per day, cross-linked into the wiki index)
  — `BU-P6-092`, `reference/sergeant-upstream/bin/wiki-daily-digest` (L1-7)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
