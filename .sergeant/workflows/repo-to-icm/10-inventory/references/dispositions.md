# Inventory disposition legend

Layer 3 (stable across runs), local to `10-inventory` (no other stage in
this workflow consults it — if that changes, this file moves to
`../_config/` per `docs/icm/convention.md` §1.3/§1a rule 2). Every file in
scope gets exactly one of these four dispositions. Silence — a file present
in the subject repository but absent from the inventory — is not a fifth
disposition; it is a defect in the inventory.

| Disposition | Meaning |
|---|---|
| **decompose** | Behavior-bearing. Goes to `20-harvest` for extraction. |
| **helper-evidence** | Deterministic mechanics (ladder §6.5/§6.6 in the proposal) that may inform a later helper/shared-context map but are not themselves a durable checkpoint or procedural outcome. Not sent to `20-harvest`. |
| **obsolete-candidate** | A mechanism that some other already-settled fact (an invariant, a deviation-register ruling, a design decision the subject repository itself documents) already replaces structurally. *Candidate* because the final ruling belongs to a later classification stage, not to inventory — name the specific settled fact in the row, don't just assert obsolescence. |
| **reference-only** | Excluded from extraction outright: license text, research notes, PRDs/audits/historical drafts, binary/compiled artifacts, generated lockfiles, and similar non-procedural material. State the reason per row. |

## How to apply it

Read the file (or, for a large uniform group — a directory of identically-
shaped generated artifacts, a set of compiled bytecode caches — read a
representative sample and say so) before assigning a disposition. A
disposition assigned from a filename pattern alone, without opening the
file, is a guess, not an inventory.

**decompose vs. helper-evidence** is the distinction that matters most and
is easiest to get wrong under time pressure: ask whether the file states or
implies a *procedural outcome someone follows or decides by*, versus
*mechanism that makes some other checkpoint work* (a Docker image
definition, a display-metadata file, a template script invoked by an actor
elsewhere). When genuinely unsure, prefer `decompose` — a unit extracted
from a helper-shaped file that turns out to carry no real behavior is a
cheap discard at `20-harvest`; a behavior silently never sent to extraction
because it was miscalled `helper-evidence` here is a missed behavior with no
downstream chance to recover it. This asymmetry is deliberate.

**obsolete-candidate** requires a citation, not a hunch. If you cannot name
the specific settled fact that already replaces the mechanism, the row is
`decompose` (let a later stage make the obsolescence ruling with the
behavior actually extracted) or `helper-evidence`, not `obsolete-candidate`.

**reference-only** is for material that is not behavior at all, not for
material you judge low-priority. A skippable-looking but genuinely
procedural file is still `decompose`; deprioritization is a later stage's
job (partitioning below, or classification after that), not inventory's.

## Partitioning `decompose` rows

Group the `decompose` rows into named batches sized for coherent, roughly
self-contained review — by subsystem, directory, or topic, not by an
arbitrary row count. A batch a reader could summarize in one sentence
("root-level agent instructions," "the diagnose-bug skill and its
templates") is well-formed; a batch that is just "files 1 through 40 in
directory order" is not — it gives `20-harvest` no coherent unit of work and
gives a reviewer no way to sanity-check completeness against a topic.
Record the partition name and its member paths in the inventory row (or a
partition-summary section) so `20-harvest` can work through partitions in a
declared order and a reader can verify every `decompose` row landed in
exactly one partition.

## Symlinks and duplicated trees

Where the subject repository contains symlinks (or otherwise-mirrored
duplicate trees) that point at content inventoried elsewhere under a
different path, do not re-disposition and re-partition the duplicate: note
it as pointing at its target's inventory row and give it the same
disposition and partition as that target, without duplicating extraction
work later.
