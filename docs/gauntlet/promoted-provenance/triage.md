# Provenance — Triage

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W30** `triage`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-051` | triage is a workflow that moves issues and qualifying external PRs through a fixed state machine of category and state roles, ending in an agent-ready brief or another terminal disposition. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (frontmatter: description) |
| `BU-P3-052` | The triage workflow treats a qualifying external PR as an issue with attached code, reusing the same roles and state machine with a small number of documented deltas. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (body line 11) |
| `BU-P3-054` | needs-triage is a state in the triage state machine meaning the maintainer still needs to evaluate the item. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 33) |
| `BU-P3-055` | needs-info is a state meaning the item is waiting on the reporter to supply more information before triage can proceed. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 34) |
| `BU-P3-056` | ready-for-agent is a state meaning the item is fully specified and ready for an autonomous agent to act on. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 35) |
| `BU-P3-057` | ready-for-human is a state meaning the item needs a human to implement it rather than an agent. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 36) |
| `BU-P3-058` | wontfix is a state meaning the item will not be actioned, closed either because it is already implemented, a rejected bug, or a rejected enhancement. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 37) |
| `BU-P3-060` | The triage state machine's transition graph starts at needs-triage, fans out to needs-info/ready-for-agent/ready-for-human/wontfix, loops needs-info back to needs-triage on reporter reply, and allows the maintainer to override any transition at will, with unusual transitions flagged for confirmation first. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 45) |
| `BU-P3-061` | The workflow's trigger is a natural-language maintainer request, interpreted by the actor to select which triage action to take. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 49) |
| `BU-P3-073` | An explicit maintainer directive to move an item to a specific state is trusted and applied directly, skipping gather-context/recommend/grill, after confirming the intended action; if the target is ready-for-agent without grilling, the actor separately asks whether a brief is wanted. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 90) |
| `BU-P3-075` | When resuming triage on an item with prior notes, the actor checks whether outstanding questions have been answered and presents an updated picture, never re-asking already-resolved questions. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 112) |

## Stages

### `10-gather-context`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-065` | Triaging a specific item begins by fully reading the item and prior triage notes, exploring the codebase via its domain glossary and ADRs, and running two checks: whether the behavior is already implemented (by domain concept, not literal wording) and whether the request resembles a prior recorded out-of-scope rejection. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 70) |
| `BU-P3-089` | Matching a new issue against the out-of-scope KB is done by concept similarity rather than literal keyword overlap. | `reference/sergeant-upstream/.agents/skills/triage/OUT-OF-SCOPE.md` (line 75) |
| `BU-P3-062` (helper invocation, folded from demoted `00-show-attention`) | When asked what needs attention, the workflow queries the tracker and presents three fixed buckets ordered oldest-first. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 58) |
| `BU-P3-063` (helper invocation, folded from demoted `00-show-attention`) | The third discovery bucket is needs-info items where the reporter has posted activity since the last triage notes, signaling they need re-evaluation. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 62) |
| `BU-P3-064` (helper invocation, folded from demoted `00-show-attention`) | The discovery bucket filter excludes non-external PRs, but this filter applies only to unprompted discovery — an explicitly named PR is triaged regardless of who authored it. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 64) |

### `20-verify`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-067` | Before grilling, the actor verifies the claim empirically — reproducing a bug or checking out and testing a PR's diff — and reports one of confirmed, failed, or insufficient-detail, where confirmation strengthens the eventual agent brief. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 74) |

### `30-recommend`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-066` | The actor proposes a category/state recommendation with reasoning and a relevant codebase summary, then waits for the maintainer's direction before proceeding. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 72) |

### `40-grill-if-underspecified`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-068` | If the item is underspecified after verification, the actor invokes the grilling and domain-modeling procedures together to sharpen it into shape. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 76) |

### `50-apply-outcome`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-069` | Reaching the ready-for-agent outcome requires posting a structured agent brief comment. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 79) |
| `BU-P3-070` | Reaching the needs-info outcome requires posting triage notes using the workflow's template. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 81) |
| `BU-P3-071` | When wontfix is reached because the behavior is already implemented, the closing comment points to where it lives and the out-of-scope knowledge base is explicitly not written to, since that store is reserved for rejected, not built, requests. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 83) |
| `BU-P3-072` | When wontfix is reached because an enhancement request is rejected, a record is written to the out-of-scope knowledge base, linked from the closing comment, before the item is closed. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 85) |
| `BU-P3-074` | Anything resolved during a grilling session must be captured under the needs-info template's 'established so far' section, and outstanding questions must be specific and actionable rather than generic. | `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 108) |
| `BU-P3-090` | If the maintainer confirms a KB match applies, the new issue is appended to that file's prior-requests list and closed. | `reference/sergeant-upstream/.agents/skills/triage/OUT-OF-SCOPE.md` (line 80) |
| `BU-P3-091` | If the maintainer reconsiders a previously rejected concept, the KB file is deleted or updated and the new issue proceeds through normal triage instead of being auto-closed. | `reference/sergeant-upstream/.agents/skills/triage/OUT-OF-SCOPE.md` (line 81) |
| `BU-P3-092` | The KB is written to only for rejected enhancements (never bugs), and this applies identically whether the rejected item is an issue or a PR. | `reference/sergeant-upstream/.agents/skills/triage/OUT-OF-SCOPE.md` (line 86) |
| `BU-P3-093` | Closing an item as already-implemented must never produce a KB record, because doing so would corrupt future deduplication checks with a false rejection; the closing comment instead points to the existing implementation. | `reference/sergeant-upstream/.agents/skills/triage/OUT-OF-SCOPE.md` (line 88) |
| `BU-P3-096` | When a rejected concept is reconsidered and its KB file removed, previously closed issues that cited the old rejection are not reopened, since they remain valid historical records. | `reference/sergeant-upstream/.agents/skills/triage/OUT-OF-SCOPE.md` (line 104) |

## Notes

**Synthesis notes:** `resume` and `quick-override` (BU-P3-075, BU-P3-073) are documented re-entry variants of this same stage sequence, not separate stage directories. BU-P3-060's transition graph is explicitly non-linear (loops, maintainer override at any point) — the source extractor considered and rejected an engine-gap claim for it, and that rejection is upheld here: each transition is a fresh invocation of a stage, not a control-flow construct the runtime must own.

## Adjudication A4

- **`00-show-attention` — DEMOTED.** Its CONTEXT.md carried only the §6.5 deterministic-machinery boilerplate ("candidate execute-stage workload") with no additional checkpoint argument (no "Additional note" section). Per A4's default rule, folded into `10-gather-context` as a helper invocation; `BU-P3-062`/`BU-P3-063`/`BU-P3-064` and the stage's citations move with it. The stage directory is removed. No renumbering needed: the remaining ordinals `10-50` are already in correct order without a leading `00`.

## Promotion note (`docs/icm/promotion-spec-2026-08-11.md`)

`50-apply-outcome`, this package's true (and only) closing stage, declares
a `promote` output disposition with no finalize step — one of the 30 of 34
N1 packages in that shape, not one of the 3 (`drain-fleet`,
`respond-to-worker`, `to-spec`) that name one. Recorded here per the
spec's finalize-gap rule rather than silently promoted; disposition is
left to human review at merge time, not applied mechanically by this
curation act.

**NEEDS-JUDGMENT resolution (§5):** this package's classification rests on
two signals, both confirmed rather than re-adjudicated. (1) `## Delegation`
in `40-grill-if-underspecified/CONTEXT.md` names **grilling** as the
target that produces this stage's outcome; `grilling` is present in this
library under `.sergeant/workflows/grilling/` at the time of this
promotion, so the reference resolves. Because `grilling`'s own G5 case
(re-enterable `needs_input` interview loop) lives inside `grilling`, not
inside any of `triage`'s own five stages, `triage` inherits that
caveat only by composition through delegation — it is not a second, live
G5 case of `triage`'s own, and the "needs a scripted or real-backend
acceptance pass before trusted" obligation attaches to `grilling`'s own
promotion record, not repeated as a gate requirement here. (2) The
`## Notes` section above (and `CONTEXT.md`'s "Notes for reviewers")
separately records that BU-P3-060's non-linear transition graph
(needs-triage fanning out to needs-info/ready-for-agent/ready-for-human/
wontfix, looping, maintainer override at any point) was considered for an
engine-gap claim by the source extractor and the claim was **rejected,
not left open** — each transition is packaged as a fresh stage invocation,
not a control-flow construct the engine must own. This is distinct from
`grilling`'s live G5 and is not re-litigated here; it is carried across
unedited as the adjudicated call it already is. §3's engine-acceptance
gate exercised `triage`'s own five stages as ordinary sequential
completions (none of which is itself a `needs_input` stage), so the clean
gate pass is full mechanical confirmation of `triage`'s own packaging.

