# Normalization method

Layer 3 (stable across runs), local to `30-normalize`.

Normalization rewrites `statement` (and, where they leaked implementation
detail, `scope`/`trigger`/`outcome`) so the unit reads as true regardless of
which specific tool, script, filename, or old mechanism the subject
repository currently happens to use — without touching the evidence that
anchors it to source.

## What changes, what never does

**Never touches:** `id`, `source.path`, `source.locator`, `source.quote`,
`source.quote_hash`. These are the evidence. Rewriting a citation to make
the prose read better breaks the one property `../_config/evidence-policy.md`
exists to guarantee — that the record can be re-anchored to real source.
If normalization makes you want to change the quoted span, you are not
normalizing, you are re-extracting; that means going back to what
`20-harvest` should have captured, not silently editing its citation here.

**Rewritten when needed:** `statement`, and `scope`/`trigger`/`outcome` if
they currently only make sense in terms of a specific mechanism. Two moves,
applied together:

1. **Extract the durable behavior.** Ask: would this statement still be
   true if the subject repository swapped its specific tool/script for a
   different one that achieved the same effect? If not, the mechanism has
   leaked into the statement — restate the behavior in terms of what must
   durably be true, not how the current repository happens to achieve it.
2. **Preserve the mechanism as a note, not delete it.** The old mechanism is
   often useful context for whoever later decides how to represent the
   behavior (helper? shared context? obsolete already?). Move it to
   `notes` rather than throwing it away.

## Re-checking "one behavior per unit"

Sometimes a unit that looked atomic at extraction time turns out, once you
try to state it independent of mechanism, to actually be two behaviors that
happened to share one mechanism. If normalizing a unit forces you to write
"and" to keep it accurate, split it: assign a new `id` to the second half,
have both halves cite the same `source.quote`/`quote_hash` (the span
supports both), and note the split in each unit's `notes` (e.g. "split from
EX-0042 at normalization; see EX-0042a/b").

## What you are not doing here

You are not deciding whether a unit becomes a workflow, a stage, an
`AGENTS.md` invariant, a helper, or an engine-gap claim. A normalized
`statement` that already reads like a stage boundary or a workflow name is
a smell worth flagging in `notes` for the classification stage, not an
invitation to add `representation`/`workflow`/`stage` fields yourself —
manufacturing a unit's shape to match an anticipated classification is the
violation `docs/icm/record-shapes.md` §3 rule 2 names.

## Re-assessing confidence honestly

If rewriting a unit for implementation-independence reveals that the
statement leans more on inference than the source directly supports, lower
`confidence` to reflect that. Keeping it `high` out of habit after the
rewrite exposed a gap is not honest bookkeeping.
