# 20-route-each: route each

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-retain-artifact/output/README.md | L4 | upstream artifact produced by `10-retain-artifact` |

## Purpose

Each finding is routed with a dedup marker scoped to axis+source+id+parent+branch; a divergent stored body is refused untouched.

Trigger (workflow-level): A review pass (worker-mission's `30-independent-review`, or code-review) has produced findings.

## What must become true here (durable outcome)

Each finding is routed with a dedup marker scoped to axis+source+id+parent+branch; a divergent stored body is refused untouched.

## Behavior contract

- **A finding is deduplicated against an existing tracked-work item using a marker scoped to the exact review axis, source, finding ID, parent mission, and branch — never axis/source/id alone — because reviewers emit generic finding IDs (e.g. 'spec-1') that would otherwise collide across unrelated review sessions and let one session's update silently overwrite another's evidence.**
  (trigger: a finding is being matched against previously routed findings; outcome: findings from genuinely different review sessions are never conflated into one tracked-work item just because they share a generic id)
  — `BU-P6-085`, `reference/sergeant-upstream/bin/sgt-review-findings` (L524-528)
- **A matched existing tracked-work item is only ever updated (priority, labels reopened-if-closed) when its stored content digest still matches the incoming finding's digest; a divergent stored body is refused and left completely untouched — no description, title, or status change — so nothing stored can ever be silently lost by a write that never happens.**
  (trigger: a deduplication match's stored content digest disagrees with the incoming finding; outcome: a mismatched match is refused explicitly rather than silently overwriting potentially human-edited or differently-sourced evidence)
  — `BU-P6-086`, `reference/sergeant-upstream/bin/sgt-review-findings` (L586-592)
- **Deduplication of routed findings must be scoped to the parent task and branch, not applied globally, so an identical-looking finding on a different branch is never silently treated as an already-seen duplicate.**
  (trigger: the same finding text is routed for two different missions or branches; outcome: duplicate findings are correctly deduplicated per mission/branch, not globally, which would hide a real repeat occurring on a different branch)
  — `BU-P7-063`, `reference/sergeant-upstream/tests/sgt-review-findings-test.sh` (line 492)
- **The router must refuse (not silently accept) findings whose review-artifact composition or comparison the owner explicitly ruled must be rejected, per an adjudicated decision rather than an inferred default.**
  (trigger: the router evaluates a findings artifact against its composition/comparison rules; outcome: specific refusal conditions are explicit, owner-adjudicated policy encoded in the router, not emergent or accidental behavior)
  — `BU-P7-064`, `reference/sergeant-upstream/tests/sgt-review-findings-test.sh` (line 541)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
