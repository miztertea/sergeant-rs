# Fan out evidence

Resolved as `@@fan-out-evidence` from
`.sergeant/common/contexts/fan-out-evidence.md` per
`docs/icm/convention.md` §4. Shared stage context, two or more consumers:
`investigate/10-fan-out-evidence`, `implement-change/20-panel`,
`review-change/20-panel` (the general fan-out mechanism `panel.md` and
`refute.md` specialize for the four fixed axes).

The only proven mechanism in the library for isolated concurrent
sub-agent work, on the precedent of the retired `code-review`
package's own two-call parallel dispatch: N seats, spawned together in a
single message, each with a self-contained brief, each capped, reporting
back to a stage that collects rather than reviews.

## Contract

- **One message, N calls.** Every seat this stage spawns is spawned in one
  message. A seat spawned after reading another seat's output is not this
  mechanism; it is a chain, and it forfeits the only isolation property
  the mechanism has.
- **Each seat's brief is self-contained**: what it is being asked, the
  evidence or material it needs (paths, pasted contents, a diff command),
  a word cap, and nothing else — a seat has no other access to this Work.
- **A seat that cannot complete degrades the fan-out to fewer seats**, and
  the missing seat is named in the calling stage's own output. The stage
  never silently reports full coverage it did not get.
- **Caps are caps, not guarantees.** The word limit bounds what a seat may
  return; it does not certify that every seat which stayed under it
  produced something useful — that judgment belongs to the calling stage
  when it reads the reports back.

**What "isolated" means here, exactly.** These seats are sub-agents
spawned inside one stage execution, not separate Works. Their isolation
is context isolation only: each seat sees the brief this stage hands it
and nothing of its siblings' reasoning. It is weaker than the isolation a
separate Work would give — the seats share this stage's single execution,
its journal entry, its usage window and its failure; there is no per-seat
journal, no per-seat recovery, and a stage that dies takes every seat's
unwritten output with it. `docs/icm/convention.md` §6.3 places review
independence in the execution boundary: this stage has one execution boundary, not four.
Report this panel as what it is — four isolated
briefs read by one stage — and never as four independent reviews.

## What this context contributes when loaded inside a stage

- **J0 the caller must honor:** a seat cannot be spawned or dies with no
  output — the caller reports reduced coverage, never silently absorbs it
  as full coverage.
- **J2 the caller retains:** how many seats to spawn and how to bound each
  one's sub-question, within whatever N its own contract fixes.
- **J1 the caller retains:** exact invocation wording, so long as every
  seat is spawned in the one message.

There is no stage library in this engine. This file is shared text pulled
into a stage's own `CONTEXT.md` by `@@` reference. A change here does not
propagate to a consumer's own narrowing; each must be re-read by hand
against it — drift by construction, named rather than hidden.
