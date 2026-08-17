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
- **Reaching the needs-info outcome requires posting triage notes using the workflow's template.**
  (trigger: the outcome is needs-info; outcome: templated triage notes exist recording what is known and what is still needed)
- **When wontfix is reached because the behavior is already implemented, the closing comment points to where it lives and the out-of-scope knowledge base is explicitly not written to, since that store is reserved for rejected, not built, requests.**
  (trigger: wontfix is reached because the item is already implemented; outcome: the closing comment cites the existing implementation and no out-of-scope record is created)
- **When wontfix is reached because an enhancement request is rejected, a record is written to the out-of-scope knowledge base, linked from the closing comment, before the item is closed.**
  (trigger: wontfix is reached as a rejected enhancement; outcome: an out-of-scope record exists and is linked before closing)
- **Anything resolved during a grilling session must be captured under the needs-info template's 'established so far' section, and outstanding questions must be specific and actionable rather than generic.**
  (trigger: a needs-info comment is being posted after some grilling occurred; outcome: no resolved information from grilling is lost, and outstanding questions are concrete)
- **If the maintainer confirms a KB match applies, the new issue is appended to that file's prior-requests list and closed.**
  (trigger: the maintainer confirms a surfaced KB match; outcome: the new issue is recorded against the existing concept and closed)
- **If the maintainer reconsiders a previously rejected concept, the KB file is deleted or updated and the new issue proceeds through normal triage instead of being auto-closed.**
  (trigger: the maintainer reconsiders a surfaced KB match; outcome: the stale rejection record is removed/updated and the item re-enters the normal state machine)
- **The KB is written to only for rejected enhancements (never bugs), and this applies identically whether the rejected item is an issue or a PR.**
  (trigger: an item reaches wontfix; outcome: only enhancement rejections, whether issue or PR, produce a KB record)
- **Closing an item as already-implemented must never produce a KB record, because doing so would corrupt future deduplication checks with a false rejection; the closing comment instead points to the existing implementation.**
  (trigger: an item reaches wontfix because it is already implemented; outcome: no KB record is created, and the correctness of future dedup matching is preserved)
- **When a rejected concept is reconsidered and its KB file removed, previously closed issues that cited the old rejection are not reopened, since they remain valid historical records.**
  (trigger: a maintainer reconsiders a previously rejected concept; outcome: old closed issues stay closed even though the KB record that justified closing them is gone)

## Bounded judgment

Apply `@@bounded-judgment`.

### J5 — fixed rules, no interpretation
- Wontfix because already-implemented: closing comment cites the existing implementation, and the out-of-scope KB is never written to.
- Wontfix because a rejected enhancement: an out-of-scope KB record is written and linked from the closing comment before the item is closed.
- The KB is written to only for rejected enhancements, never bugs, regardless of whether the item is an issue or a PR.
- Maintainer confirms a surfaced KB match applies: the new issue is appended to that file's prior-requests list and closed.
- Maintainer reconsiders a previously rejected concept: the KB file is deleted or updated, the new issue proceeds through normal triage instead of auto-closing, and previously closed issues that cited the old rejection are left closed, not reopened.

### J2 — delegated to this stage
- Drafting the structured agent brief (ready-for-agent), the templated triage notes (needs-info), and the closing comment's specific wording.
- Folding anything resolved during grilling into the needs-info template's "established so far" section, and keeping outstanding questions specific and actionable.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers — the KB write/no-write and reconsideration rules above are J5, not judgment calls.

### Completion boundary
This stage may complete only when the terminal disposition's required artifact (agent brief, triage notes, or closing comment with correct KB handling) has been produced and applied.

### Decision evidence
The applied artifact and any KB record change are this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
