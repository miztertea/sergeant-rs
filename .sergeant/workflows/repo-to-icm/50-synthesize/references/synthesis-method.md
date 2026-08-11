# Synthesis method — clustering classified units into candidates

Layer 3 (stable across runs), local to `50-synthesize`. Turns a flat file of
per-unit classification records into named, describable candidates a human
(and `60-draft`) can act on.

## The seven buckets

Every classification record from `../40-classify/output/classifications.ndjson`
lands in exactly one of these buckets, by its `representation` field. Work
bucket by bucket; within a bucket, group by shared `workflow`/`stage` values
before naming anything.

1. **Workflow candidates** (`representation: workflow` records, plus the
   `stage`/`stage-context`/`helper` records whose `workflow` field names
   the same candidate — `docs/icm/record-shapes.md` §4 requires every
   `stage`/`stage-context`/workflow-local-`helper` record to carry a
   non-empty `workflow` field, so this sweep should never meet one without
   it; if you do find a `helper` (or `stage`/`stage-context`) record with no
   `workflow` value, that is a `40-classify`-stage defect surfacing here,
   not something to silently resolve — record it under the same
   `## Unattached records` heading bucket 3 defines below, rather than
   dropping it or inventing a `workflow` value for it). For each distinct
   `workflow` value seen across the
   corpus, produce one candidate: a short kebab-case name (unique within
   this run's output — check against every other candidate name you are
   about to mint, and against `.sergeant/workflows/` and
   `.sergeant/drafts/workflows/` as they exist right now), a description
   that names a recognizable trigger, a bounded outcome, and a completion
   condition (the same bar `docs/icm/record-shapes.md` §1 sets for
   `index.md`'s own `description` field — restating the name is a
   violation there and is one here too), and an ordered list of its member
   stage candidates (below).
2. **Stage candidates** (`representation: stage` records grouped by
   `workflow`+`stage`). Order stages within a workflow candidate by the
   sequence implied by chaining `trigger`→`outcome` across the member
   behavior units in `../30-normalize/output/behavior-units.normalized.ndjson`
   (a unit whose `trigger` names a condition another unit's `outcome`
   establishes comes after it). Where the ordering is genuinely ambiguous,
   pick a defensible order and say why in one line — this is a judgment
   call `80-adversarial-review` gets to challenge, not a decision to hide.
3. **Stage-context attachments** (`representation: stage-context` records).
   These do not create new stage boundaries. Attach each to the stage
   candidate its `workflow`+`stage` fields name. **If no stage candidate
   with that `workflow`+`stage` exists yet, that is a synthesis-time
   defect** — a stage-context unit implying a checkpoint no one classified
   as a stage. Do not silently invent a stage to hang it on and do not
   silently drop the unit; record it under a `## Unattached records`
   heading naming the gap plainly (the same heading bucket 1 uses for a
   workflow-local `helper`/`stage` record missing its required `workflow`
   field — both are the same class of synthesis-time defect: a record this
   stage cannot place without inventing a fact `40-classify` should have
   supplied).
4. **Permanent-instruction candidates** (`representation: agents-invariant`
   records). List them; do not draft them into any workflow package (this
   workflow does not publish, and `AGENTS.md` changes are the promotion
   reviewer's call, not this run's).
5. **Shared helper/context candidates** (`representation: shared-helper` /
   `shared-context` records). For each, name the contract (inputs, output
   shape, meaning) and list which candidate workflows would consume it. If
   a `.sergeant/common/contexts/` or `.sergeant/common/scripts/` entry with
   that name already exists, say so and note whether this candidate's
   contract actually matches it (a same-name mismatch is worth flagging,
   not silently assumed to be the same thing).
6. **Obsolete-mechanism findings** (`representation: obsolete-mechanism`
   records). Name the mechanism, the settled fact that already replaces it
   (carried from the behavior unit's `notes`, per the ladder's own
   requirement that obsolescence cites a specific settled fact — see
   `../_config/icm-ladder.md`, "One more disposition"), and where any
   surviving policy re-homes (usually into a permanent-instruction or
   stage-context candidate you are also producing — cross-reference it).
7. **Engine-pressure candidates** (`representation: engine-gap` records).
   Carry the record's `engine_gap` object through **unchanged** — this
   stage clusters and lists, it does not edit engine-gap evidence. Where
   two or more engine-gap records plausibly require the *same* minimum
   runtime capability, you may note the overlap, but do not force a merge
   without stating the shared capability explicitly.

## What must not happen

- A classification record silently absent from every bucket. Every
  `behavior_id` in `../40-classify/output/classifications.ndjson` appears
  in exactly one bucket appearance across all seven buckets: buckets 1–3
  count as one appearance for a `stage`/`stage-context` record (the unit is
  a stage or an attachment to one, not both); a record landing in the
  `## Unattached records` heading (bucket 1 or 3) also counts as its one
  appearance — it is accounted for, just not attached to anything; buckets
  4–7 (`agents-invariant`, `shared-helper`/`shared-context`,
  `obsolete-mechanism`, `engine-gap`) are otherwise one record = one direct
  appearance, keyed by `representation` alone.
- A candidate name invented with no member records citing it — every
  candidate you write down traces back to at least one `behavior_id`.
- Manufacturing a workflow/stage candidate's boundary to look tidy rather
  than reporting what the classification records actually said. If the
  classified corpus produces an awkward, unevenly-sized, or
  single-behavior "workflow," write that down as the candidate — reshaping
  it into something that reads better is inventing scope this stage was
  not given.
