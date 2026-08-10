# Classification record shape

Layer 3 (stable across runs), local to `40-classify`. This is the operative,
workflow-owned distillation of `docs/icm/record-shapes.md` §4 (classification
record); it does not amend that document, and if the two ever disagree,
`docs/icm/record-shapes.md` governs and this file is stale and should be
corrected (edit-source principle, `docs/icm/convention.md` §1a rule 3).

## The record

One JSON record per line, one record per behavior unit:

```json
{
  "behavior_id": "BU-0042",
  "representation": "stage-context",
  "workflow": "cross-repo-work",
  "stage": "00-establish-scope",
  "rationale": "The rule is needed only while establishing scope; it is not a reusable procedure or independent durable checkpoint.",
  "alternatives_considered": ["AGENTS.md invariant", "separate workflow", "helper"],
  "engine_gap": null
}
```

| Field | Required | Rule |
|---|---|---|
| `behavior_id` | yes | MUST reference an existing unit's `id` from `../30-normalize/output/behavior-units.normalized.ndjson`. Every normalized unit gets exactly one classification record — no omissions, no duplicates. |
| `representation` | yes | One of: `agents-invariant`, `workflow`, `stage`, `stage-context`, `helper`, `shared-helper`, `shared-context`, `engine-gap`, `obsolete-mechanism`. No other value — `references/../_config/icm-ladder.md`'s questions 6.1–6.7 plus the "one more disposition" section are the only source of this vocabulary. |
| `workflow` | conditional | Required for `stage`/`stage-context`/`helper`-with-a-single-owner records; omit for `agents-invariant`, `engine-gap`, or a `shared-*` record with no single owning workflow. |
| `stage` | conditional | Required for `stage`/`stage-context` records. Names the *intended* stage — it does not need to match a materialized directory yet (that happens at `60-draft`). |
| `rationale` | yes | States *why this rung and not an adjacent one*. A rationale that would read identically pasted onto a different representation choice does not discriminate and is a violation — see icm-ladder.md's reimplementation test for `stage` records specifically. |
| `alternatives_considered` | yes | The other rungs weighed and rejected, as a list of ladder rung names (6.1–6.7 names, not restatements). Empty is legal **only** where no adjacent rung was facially plausible. REQUIRED non-empty for every record carrying a `workflow` or `stage` value, and for every `engine-gap` record. |
| `engine_gap` | yes (nullable) | `null` unless `representation` is `engine-gap`, in which case it MUST be the full six-field template (verbatim field names) from `../_config/icm-ladder.md`'s §6.7 section — not a summary, not a partial object. A `representation: engine-gap` record with a `null` or partial `engine_gap` is a violation and is auto-rejected at lint (`record-shapes.md` §5 rule 1), not merely flagged. |

## What this stage does not do

This record does not touch or restate the behavior unit's own evidence
fields (`id` used only by reference as `behavior_id`, `source.*`,
`statement`, `scope`, `trigger`, `outcome`, `authority`, `confidence`,
`notes`) — those live only in `../30-normalize/output/behavior-units.normalized.ndjson`
and are never copied into a classification record. A classification record
that duplicates evidence fields instead of just citing `behavior_id` has
misunderstood the shape.

## Classification is a claim, not a fact

Every record written here is provisional until `80-adversarial-review` and
`90-reconcile` have had their turn at it (`record-shapes.md` §4 rule 1). Do
not soften a rationale or inflate `confidence`-adjacent language to make a
classification look more settled than it is — an honest, contestable
classification is the correct output of this stage.
