# Architecture Decision Records

This directory records decisions that fix an architectural shape once and are
expensive to re-litigate casually — the platform boundary, the durability
promise, cross-platform development constraints — as opposed to
`GAUNTLET.md`, which is the append-only ledger of what was built and
measured against a milestone contract. Files are numbered sequentially,
`NNNN-kebab-case-title.md`, and are never renumbered or edited to reverse a
decision; a decision that changes gets a new, higher-numbered ADR that
supersedes the old one and says so, the same append-only discipline
`GAUNTLET.md` already uses elsewhere in this repo (unlike `LESSONS.md`,
which updates entries in place and deletes what proves wrong — a
different, non-append-only convention for a different kind of record).
Each ADR
carries: **Status** (accepted, with a date), **Context**, **Decision**,
**Alternatives considered**, **Consequences** (including the negative ones),
and **Open questions** for anything the record leaves genuinely unresolved
rather than papering over.
