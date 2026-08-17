# Ponytail Minimality Ladder — local specialization

The rung definitions (R1–R7) and the "what minimality does not mean"
clause are canonical in `AGENTS.md`'s CONSTRUCTION section — this file
does not repeat them. It carries only what's specific to how this repo
applies the ladder in its own artifacts, referenced as `@@ponytail`.

Source lineage: "Ponytail Minimality Ladder," owner's Knowledge base;
upstream https://github.com/DietrichGebert/ponytail.

## Rung-logging convention

Every design decision in a ledger entry, every deviation-register row, and
every new dependency, file, trait, or store records the rung it resolved
at (R1–R7). An R7 entry must name which lower rungs were checked and why
they failed. Critics on the simplicity axis grade rung-skipping as a
finding.

## Boundary

This ladder governs construction — *should this exist?* It answers a
different question from the Bounded-Judgment Ladder's *may I decide
this?* (`@@bounded-judgment`, also canonical in `AGENTS.md`). A change can
pass R1–R7 and still need a J-rung citation for who gets to make it, or
the reverse.
