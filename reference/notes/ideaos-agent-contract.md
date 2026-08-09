# IdeaOS Operating Contract — Distillation

Source: "IdeaOS Agent Instructions" (owner's Notion workspace, fetched 2026-08-08).
What the prototype inherits from it; the full page remains authoritative.

## Plan → Act → Verify, recursively

Every Act originates from a Plan and terminates in Verification, at every scale —
program, work package, task, and individual write. A large loop is composed of
smaller loops; it is not one giant planning phase followed by unchecked activity.
The gauntlet (see `gauntlet-pattern.md`) is this contract applied to software
milestones.

## Invariants adopted here

- **Search before creating; prefer updating over duplicating.**
- **Readable bodies are required** — human-readable synthesis is primary;
  structure supports retrieval but never replaces it. (Ledger entries and
  contracts are prose first.)
- **Verify every write** — read back what was changed; confirm the intended state;
  repair before reporting completion. (For code: gates and critics; for documents:
  read-back.)
- **Preserve state and history** — do not silently reopen settled decisions or
  rewrite evolution as a cleaner story than the evidence supports. (The ledger is
  append-only; superseded decisions stay visible.)
- **Respect authority boundaries** — Git owns reproducible source; execution
  systems own detailed work state; Notion/IdeaOS owns meaning and continuity.
  This repo never writes to the owner's Notion; world-delta candidates accumulate
  in `LESSONS.md` for the owner to promote explicitly.
- **Ponytail Minimality Ladder** — a strict preference order for every addition
  (source: "Ponytail Minimality Ladder", owner's Knowledge base; upstream
  https://github.com/DietrichGebert/ponytail):

  | Rung | Question | Resolution |
  |---|---|---|
  | R1 | Does this need to exist? | No → skip it (YAGNI) |
  | R2 | Already in this codebase? | Reuse it, don't rewrite |
  | R3 | Stdlib does it? | Use it |
  | R4 | Native platform feature? | Use it |
  | R5 | Installed dependency? | Use it |
  | R6 | One line? | One line |
  | R7 | Only then | The minimum that works |

  The ordering is the point: it blocks the jump from "I understand the
  requirement" to "I should create a new abstraction." Minimality does not mean
  skipping tests, recovery, or necessary architecture — those are part of
  correctness ("correctness constrains the destination; expertise constrains
  the path").

  **Rung logging convention (this repo):** every design decision in a ledger
  entry, every deviation-register row, and every new dependency, file, trait,
  or store records the rung it resolved at (`R1`–`R7`). An `R7` entry must name
  which lower rungs were checked and why they failed. Critics on the simplicity
  axis grade rung-skipping as a finding.

## Lineage and boundary

firstmate → Sergeant (Bash/tmux, `reference/sergeant-upstream/`) → **sergeant-rs**
(this prototype). Adjacent but distinct: **Garrison**, the owner's conceptual
work-to-execution architecture (WorkPacket, Work Filesystem, Mission Record), is
deliberately unimplemented and behind decision gates. This prototype is a
Sergeant-lineage runtime; it must not quietly become a Garrison implementation.
Where the proposal's trajectory/evidence/workflow concepts rhyme with Garrison's,
that is recorded lineage, not license to import Garrison contracts.
