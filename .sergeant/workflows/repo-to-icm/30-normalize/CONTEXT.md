# 30-normalize: rewrite units independent of source mechanism

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/evidence-policy.md | L3 | the record shape and citation fields that must survive this rewrite unchanged |
| ../_config/run-discipline.md | L3 | the blindness rule and the `# AMBIGUOUS — NOT RESOLVED` propagation rule |
| references/normalization-method.md | L3 | what changes, what never does, and how to re-check unit boundaries while rewriting |
| ../20-harvest/output/behavior-units.ndjson | L4 | upstream artifact produced by `20-harvest` — the raw extracted units this stage rewrites |

## Purpose

Rewrite each behavior unit from `20-harvest` so its `statement` (and, where
needed, `scope`/`trigger`/`outcome`) is independent of the subject
repository's specific filenames and old implementation mechanisms — while
leaving the evidence fields (`id`, `source.*`) untouched, per
`references/normalization-method.md`.

## What must become true here (durable outcome)

`output/behavior-units.normalized.ndjson` exists, one record per line, with
every unit from `20-harvest`'s output accounted for: either rewritten in
place (same `id`, evidence fields unchanged, `statement`/etc. normalized),
carried through unchanged with a note if it was already implementation-
independent, or split into two or more successor units (new `id`s, `notes`
recording the split, same source evidence) where normalization revealed a
conjoined behavior. No unit from the input silently disappears.

## How to do it

0. If `../20-harvest/output/behavior-units.ndjson` opens with `#
   AMBIGUOUS — NOT RESOLVED`, do not proceed — follow
   `../_config/run-discipline.md` §2.

Go through `../20-harvest/output/behavior-units.ndjson` in order, one
record at a time:

1. Read the `statement` and ask whether it would still hold if the subject
   repository's specific mechanism changed. If yes, carry it through with
   at most cosmetic cleanup. If no, rewrite it per
   `references/normalization-method.md` — extract the durable behavior,
   move the mechanism detail to `notes`.
2. Check `scope`/`trigger`/`outcome` for the same leak; rewrite only where
   needed.
3. Leave `source.path`, `source.locator`, `source.quote`, `source.quote_hash`
   exactly as they came in. Do not touch them.
4. If rewriting exposes a conjoined behavior, split per
   `references/normalization-method.md`'s guidance and record the split.
5. Re-assess `confidence` honestly if the rewrite changed how directly the
   source supports the statement.
6. Do not add `representation`, `workflow`, `stage`, `rationale`,
   `alternatives_considered`, or `engine_gap` — none of that is this
   stage's contract.

Process the whole input file; a unit you judge needs no change is still
written to the output file (unchanged, or with a `notes` addition saying
so) — the output is the complete normalized corpus, not a diff.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its disposition.
