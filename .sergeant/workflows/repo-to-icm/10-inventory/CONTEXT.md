# 10-inventory: produce a deterministic source inventory

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-contract/output/README.md | L4 | upstream artifact produced by `00-contract` — subject repository, pinned revision, scope, and exclusions this stage enumerates within |
| references/dispositions.md | L3 | the four-way disposition legend this stage applies, and how to apply it |

## Purpose

Produce a deterministic inventory of every file in scope (per
`contract.md`): what it is, which of four dispositions it gets, and — for
files headed to extraction — which named partition they belong to. This is
one of the workflow's own declared outputs (a "source inventory"), not
scratch work for a later stage to redo.

## What must become true here (durable outcome)

`output/inventory.md` exists and accounts for **every file in scope named or
implied by `contract.md`** — no file present in scope and absent from the
inventory, no disposition assigned without having actually looked at the
file (or, for a large uniform group, a representative sample — say so when
you do this). Every `decompose`-dispositioned file belongs to exactly one
named partition.

## How to do it

1. Enumerate the subject repository's files at the pinned revision,
   restricted to `contract.md`'s scope and exclusions (a repository's own
   file listing, e.g. `git ls-files`, run against the pinned SHA, is more
   trustworthy than walking the working tree by hand — the working tree
   could have drifted).
2. For each file, assign exactly one disposition from
   `references/dispositions.md` and record what the file actually is (a
   one-line description, not a category alone) alongside the disposition
   and the reason.
3. Group `decompose` rows into named partitions per
   `references/dispositions.md`'s partitioning guidance.
4. Total the count: every enumerated file appears in exactly one
   disposition row (directly, or via a symlink/duplicate note pointing at
   its target's row); the counts by disposition and by partition sum back
   to the total files enumerated in step 1. A mismatch here means step 1 or
   step 2 missed something — find it before finishing, not after.

This workflow is not scoped to any one repository shape (a library, an
infrastructure repo, a documentation repo may have very different file
mixes) — apply the legend fresh to what is actually in scope rather than
assuming a particular repository's shape.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its disposition.
