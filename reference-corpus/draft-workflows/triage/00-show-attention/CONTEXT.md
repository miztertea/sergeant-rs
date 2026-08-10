# 00-show-attention: show attention

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Three fixed buckets, oldest first.

Trigger (workflow-level): An item is at the front of one of the three fixed attention buckets, oldest first.

## What must become true here (durable outcome)

Three fixed buckets, oldest first.

## Behavior contract

- **When asked what needs attention, the workflow queries the tracker and presents three fixed buckets ordered oldest-first.**
  (trigger: the maintainer asks what needs attention; outcome: three ordered buckets of attention-worthy items are shown)
  — `BU-P3-062`, `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 58)
- **The third discovery bucket is needs-info items where the reporter has posted activity since the last triage notes, signaling they need re-evaluation.**
  (trigger: an item is in needs-info and the reporter has replied; outcome: the item surfaces in the attention list for re-evaluation)
  — `BU-P3-063`, `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 62)
- **The discovery bucket filter excludes non-external PRs, but this filter applies only to unprompted discovery — an explicitly named PR is triaged regardless of who authored it.**
  (trigger: PRs are included in the attention buckets; outcome: internal PRs never appear via discovery, but can always be triaged by explicit request)
  — `BU-P3-064`, `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 64)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
