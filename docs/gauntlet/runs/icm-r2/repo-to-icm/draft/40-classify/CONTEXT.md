# 40-classify: apply the ICM decomposition ladder

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/icm-ladder.md | L3 | the decomposition ladder (§6.1-6.7) this stage applies to every unit, in order, including the required-before-helper §6.3 answer and the over-promotion tell |
| ../_config/run-discipline.md | L3 | the blindness rule and the `# AMBIGUOUS — NOT RESOLVED` propagation rule |
| references/classification-record-shape.md | L3 | the exact record fields, conditional requirements, and the empty-`alternatives_considered` rule |
| ../20-harvest/references/consequence-class-checklist.md | L3 | the five safety/identity/recovery/delivery/human-decision hunt classes — a unit whose behavior falls into one of these gets heightened §6.3 scrutiny before being routed to `helper`/`shared-helper` (see step 2 below) |
| ../30-normalize/output/behavior-units.normalized.ndjson | L4 | upstream artifact produced by `30-normalize` — the normalized behavior units this stage classifies |

## Purpose

Apply `../_config/icm-ladder.md`'s decomposition ladder to every normalized
behavior unit, one classification record per unit, per
`references/classification-record-shape.md`. This stage assigns
representation; it does not draft, cluster, or materialize anything —
that is `50-synthesize`'s and `60-draft`'s job.

## Bounded judgment

Apply `@@bounded-judgment`.

A governing constraint (J5, from `../_config/icm-ladder.md` itself): the
ladder's rungs are asked **in order**, 6.1 through 6.7, stopping at the
first that answers yes — this stage may not skip ahead to a rung that
"feels right" before ruling out every lower one, and may not classify
`helper`/`shared-helper`/`shared-context` without first stating an explicit
§6.3 answer in `rationale`.

### J2 — delegated to this stage
- Answering each ladder question and selecting the rung, for every
  normalized unit, with `rationale` stating why that rung and not an
  adjacent one.
- Applying heightened scrutiny to a unit whose behavior falls into one of
  the five consequence classes before accepting a `helper`/`shared-helper`
  classification for it.
- Resolving a unit genuinely torn between two adjacent rungs to one rung,
  recording both in `alternatives_considered` and stating the tension
  honestly in `rationale` — this is a delegated call, not an escalation;
  `80-adversarial-review` exists to challenge it.
- Writing the full six-field `engine_gap` template for an `engine-gap`
  classification, or declining the classification outright if any field
  would read "would be convenient."
- Running this stage's own over-promotion self-check (step 7) on its
  `helper`/`shared-helper` output before handing it downstream.

### J1 — local choices allowed
- The order in which normalized units are processed, so long as the whole
  input file is processed and no unit is skipped or classified twice.

### J0 — must become `needs_input`
- `../30-normalize/output/behavior-units.normalized.ndjson` opens with
  `# AMBIGUOUS — NOT RESOLVED` — do not proceed; follow `../_config/
  run-discipline.md` §2.

### Completion boundary
This stage may complete only when `output/classifications.ndjson` classifies
every unit from the upstream file exactly once, each record meeting
`references/classification-record-shape.md`'s bar for `rationale` and
`alternatives_considered`, and the over-promotion self-check (step 7) has
been run against this run's own `helper`/`shared-helper` output.

### Decision evidence
`output/classifications.ndjson`'s own `rationale`/`alternatives_considered`
fields, per record, are this stage's decision record.

## What must become true here (durable outcome)

`output/classifications.ndjson` exists, one record per line, with every unit
from `../30-normalize/output/behavior-units.normalized.ndjson` classified
exactly once (no unit skipped, no unit classified twice). Every record
carries `rationale` and `alternatives_considered` meeting
`references/classification-record-shape.md`'s bar — not generic, not
copy-pasteable onto an adjacent rung.

## How to do it

0. If `../30-normalize/output/behavior-units.normalized.ndjson` opens with
   `# AMBIGUOUS — NOT RESOLVED`, do not proceed — follow
   `../_config/run-discipline.md` §2.

Go through `../30-normalize/output/behavior-units.normalized.ndjson` in
order, one unit at a time:

1. Ask `../_config/icm-ladder.md`'s questions **in order**, 6.1 through 6.7,
   and stop at the first one that answers yes. This ordering is itself the
   conservatism rule — do not skip ahead to a rung that "feels right" before
   ruling out every lower one.
2. **For any unit reaching 6.3 or later** (i.e., not already resolved at 6.1
   or 6.2), explicitly apply the reimplementation test and record the
   answer in `rationale` as its own sentence, **before** stating whatever
   rung the unit actually lands on: if this behavior's current mechanism
   were replaced tomorrow, would the checkpoint still exist? This is
   required even when the answer is "no" and the unit continues past 6.3 to
   `stage-context`, `helper`, or `shared-helper` — a rationale for `helper`/
   `shared-helper` that never states why the §6.3 question resolved "no"
   has not actually applied the ladder in order; it has jumped to §6.5's
   question without clearing §6.3's first, which is not legal (see
   `../_config/icm-ladder.md`'s "the question must actually be answered"
   note — this is the exact rung-ordering error N2 adjudicated at scale).
   If the honest §6.3 answer trends toward "yes, operators would want this
   measured independently," classify `stage` and stop there — do not keep
   walking the ladder to force a `helper` fit that feels tidier.
   **Heightened scrutiny:** if the unit's behavior falls into one of
   `../20-harvest/references/consequence-class-checklist.md`'s five classes
   (safety, identity, recovery, delivery, human-decision — check the
   originating unit's `notes`/`scope`/`trigger`/`outcome` for these, they
   are exactly the classes N2 run 2 silently misrouted), do not let a quick
   "this is just deterministic machinery" impression substitute for
   actually reasoning through the reimplementation test — these are
   precisely the behaviors most likely to look like machinery on a fast
   read while actually gating something operators would want measured or
   preserved independently.
3. For an `engine-gap` classification (6.7), this is the last rung — reached
   only after 6.1–6.6 have each been tried and failed for a rung-specific
   reason. Write the full six-field template from
   `../_config/icm-ladder.md`'s §6.7 section into `engine_gap`, verbatim
   field names, all six populated. If any field would read "would be
   convenient" or "could be more elegant" in any form, this is not an
   engine-gap unit — reclassify to the best-fitting lower rung instead and
   let the temptation itself go unrecorded (the classification record has
   no field for it; it is not evidence of anything).
4. For an `obsolete-mechanism` disposition, name the specific settled fact
   (a deviation-register ruling, an already-documented invariant) that
   already replaces the mechanism — carried from the unit's `notes` field.
   If you cannot name one, this is not `obsolete-mechanism`; classify by
   whatever surviving policy the behavior actually states.
5. Write the record per `references/classification-record-shape.md`. Do not
   copy the unit's evidence fields into it — only `behavior_id` refers back.
6. Move to the next unit only once its classification is written.

Process the entire input file. A classification you are genuinely torn on
between two adjacent rungs is still resolved to one — record both rungs in
`alternatives_considered` and let `rationale` state the tension honestly;
`80-adversarial-review` exists to challenge exactly this kind of call.

7. **Before writing `output/classifications.ndjson`, check your own
   `helper`/`shared-helper` records for the over-promotion tell**
   (`../_config/icm-ladder.md` §6.6): group them by `source.path` (from the
   originating behavior unit — you may need to glance back at
   `../30-normalize/output/behavior-units.normalized.ndjson` for this, it is
   already a named Input). If one source file's `helper`/`shared-helper`
   records, taken together, would obviously become their own
   single-file cluster once `50-synthesize` groups by contract — i.e.
   nothing about *what the behavior does* ties it to any other file's
   helper records — re-open each of those records' `rationale` and
   re-confirm its §6.3 answer (step 2) was actually reasoned about that
   specific behavior, not copy-pasted machinery language. This is a check
   this stage can run on its own output before handing it downstream; do
   not leave it for `80-adversarial-review` to discover what a look back at
   your own file groupings would have caught here.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its disposition.
