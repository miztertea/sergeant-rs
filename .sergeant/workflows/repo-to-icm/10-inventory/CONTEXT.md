# 10-inventory: produce a deterministic source inventory

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-contract/output/contract.md | L4 | upstream artifact produced by `00-contract` — subject repository, pinned revision, scope, and exclusions this stage enumerates within |
| references/dispositions.md | L3 | the four-way disposition legend this stage applies, and how to apply it |
| ../_config/run-discipline.md | L3 | the blindness rule, and the `# AMBIGUOUS — NOT RESOLVED` propagation this stage must honor if `contract.md` reports itself unresolved |

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

0. If `contract.md` opens with `# AMBIGUOUS — NOT RESOLVED`, do not proceed
   — follow `../_config/run-discipline.md` §2 instead of the steps below.

1. Enumerate the subject repository's files at the pinned revision,
   restricted to `contract.md`'s scope and exclusions, using whichever
   enumeration method matches how `contract.md` says the revision was
   resolved: if it recorded a **live checkout** (its own `.git`), the
   repository's own tooling is more trustworthy than walking the working
   tree by hand (`git -C <subject> ls-files`, or `git -C <subject> ls-tree
   -r --name-only <SHA>` if the pinned SHA is not the checkout's current
   `HEAD`). If it recorded a **vendored subtree** (no live checkout at that
   revision exists in this worktree — there is no git object to list
   against), enumerate the directory as it currently sits, since the
   working tree itself *is* the pinned snapshot in that case (an ordinary
   recursive file listing, e.g. `find <subject> -type f`, restricted the
   same way).
   If this run's purpose is measurement (`../_config/run-discipline.md`
   §1), this enumeration itself must not cross into `reference-corpus/` —
   confirm it is excluded per `contract.md` before or while enumerating,
   not after.
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

**On volume.** Like `20-harvest`, this stage runs as one actor turn on this
workflow's current grammar (proposal §9.2) — there is no fan-out across
files or partitions. If the volume in scope genuinely cannot be fully
dispositioned within one turn, do not silently truncate the inventory and
do not invent an ad hoc sub-procedure to work around it: finish as much as
the turn allows, in a stated deterministic order (e.g. directory-listing
order), and record plainly — in `output/inventory.md` itself — exactly
which paths were not reached. That gap is real signal for this workflow's
grammar-pressure report (`../_config/run-discipline.md`;
`90-reconcile/references/reconciliation-method.md` §3), not a defect to
hide, and it means the "every file in scope" durable outcome above was not
fully met this run — say so, rather than rounding an incomplete inventory
up to "done."

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its disposition.
