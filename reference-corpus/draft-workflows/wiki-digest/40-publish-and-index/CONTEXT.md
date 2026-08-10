# 40-publish-and-index: publish and index

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-generate/output/README.md | L4 | upstream artifact produced by `30-generate` |

## Purpose

The page exists and is linked, or the page is kept, its path reported, and the digest marked incomplete; an existing page is never overwritten with less information.

Trigger (workflow-level): A digest is due (scheduled) or explicitly requested; or the schema/logic changed and needs a dry run first.

## What must become true here (durable outcome)

The page exists and is linked, or the page is kept, its path reported, and the digest marked incomplete; an existing page is never overwritten with less information.

## Behavior contract

- **After a real digest run, ~/wiki/sessions/YYYY-MM-DD.md must exist and be linked from ~/wiki/index.md.**
  (trigger: a real digest run has completed; outcome: the digest's completion condition is a concrete, checkable pair of filesystem facts)
  — `BU-P5-141`, `reference/sergeant-upstream/skills/wiki/SKILL.md` (line 54)
- **An existing curated page is never overwritten with a version containing less information; the existing page is preserved and the rejected update is reported.**
  (trigger: a regenerated page would contain less information than the existing one; outcome: curated content is monotonically preserved, never regressed by an automatic overwrite)
  — `BU-P5-148`, `reference/sergeant-upstream/skills/wiki/SKILL.md` (line 74)
- **If the index update fails, the generated page itself is kept, its exact path is reported, and the digest is left explicitly marked incomplete.**
  (trigger: updating ~/wiki/index.md fails after a page was generated; outcome: a partial success (page written, index not updated) is reported precisely rather than silently swallowed or silently treated as full success)
  — `BU-P5-149`, `reference/sergeant-upstream/skills/wiki/SKILL.md` (line 75)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
