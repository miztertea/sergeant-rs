# 50-apply-outcome: apply outcome

## Inputs

| File | Layer | Why |
|---|---|---|
| ../40-grill-if-underspecified/output/README.md | L4 | upstream artifact produced by `40-grill-if-underspecified` |

## Purpose

The terminal disposition is applied with its required artifact.

Trigger (workflow-level): An item is at the front of one of the three fixed attention buckets, oldest first.

## What must become true here (durable outcome)

The terminal disposition is applied with its required artifact.

## Behavior contract

- **Reaching the ready-for-agent outcome requires posting a structured agent brief comment.**
  (trigger: the outcome is ready-for-agent; outcome: an agent brief comment exists on the item)
  — `BU-P3-069`, `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 79)
- **Reaching the needs-info outcome requires posting triage notes using the workflow's template.**
  (trigger: the outcome is needs-info; outcome: templated triage notes exist recording what is known and what is still needed)
  — `BU-P3-070`, `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 81)
- **When wontfix is reached because the behavior is already implemented, the closing comment points to where it lives and the out-of-scope knowledge base is explicitly not written to, since that store is reserved for rejected, not built, requests.**
  (trigger: wontfix is reached because the item is already implemented; outcome: the closing comment cites the existing implementation and no out-of-scope record is created)
  — `BU-P3-071`, `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 83)
- **When wontfix is reached because an enhancement request is rejected, a record is written to the out-of-scope knowledge base, linked from the closing comment, before the item is closed.**
  (trigger: wontfix is reached as a rejected enhancement; outcome: an out-of-scope record exists and is linked before closing)
  — `BU-P3-072`, `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 85)
- **Anything resolved during a grilling session must be captured under the needs-info template's 'established so far' section, and outstanding questions must be specific and actionable rather than generic.**
  (trigger: a needs-info comment is being posted after some grilling occurred; outcome: no resolved information from grilling is lost, and outstanding questions are concrete)
  — `BU-P3-074`, `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 108)
- **If the maintainer confirms a KB match applies, the new issue is appended to that file's prior-requests list and closed.**
  (trigger: the maintainer confirms a surfaced KB match; outcome: the new issue is recorded against the existing concept and closed)
  — `BU-P3-090`, `reference/sergeant-upstream/.agents/skills/triage/OUT-OF-SCOPE.md` (line 80)
- **If the maintainer reconsiders a previously rejected concept, the KB file is deleted or updated and the new issue proceeds through normal triage instead of being auto-closed.**
  (trigger: the maintainer reconsiders a surfaced KB match; outcome: the stale rejection record is removed/updated and the item re-enters the normal state machine)
  — `BU-P3-091`, `reference/sergeant-upstream/.agents/skills/triage/OUT-OF-SCOPE.md` (line 81)
- **The KB is written to only for rejected enhancements (never bugs), and this applies identically whether the rejected item is an issue or a PR.**
  (trigger: an item reaches wontfix; outcome: only enhancement rejections, whether issue or PR, produce a KB record)
  — `BU-P3-092`, `reference/sergeant-upstream/.agents/skills/triage/OUT-OF-SCOPE.md` (line 86)
- **Closing an item as already-implemented must never produce a KB record, because doing so would corrupt future deduplication checks with a false rejection; the closing comment instead points to the existing implementation.**
  (trigger: an item reaches wontfix because it is already implemented; outcome: no KB record is created, and the correctness of future dedup matching is preserved)
  — `BU-P3-093`, `reference/sergeant-upstream/.agents/skills/triage/OUT-OF-SCOPE.md` (line 88)
- **When a rejected concept is reconsidered and its KB file removed, previously closed issues that cited the old rejection are not reopened, since they remain valid historical records.**
  (trigger: a maintainer reconsiders a previously rejected concept; outcome: old closed issues stay closed even though the KB record that justified closing them is gone)
  — `BU-P3-096`, `reference/sergeant-upstream/.agents/skills/triage/OUT-OF-SCOPE.md` (line 104)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
