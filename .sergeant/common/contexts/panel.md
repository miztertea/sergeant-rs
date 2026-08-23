# Panel

Resolved as `@@panel` from `.sergeant/common/contexts/panel.md` per
`docs/icm/convention.md` §4. Shared stage context, two or more consumers:
`implement-change/20-panel`, `fix-defect/50-panel`,
`review-change/20-panel`. Specializes `@@fan-out-evidence`'s general N-seat
mechanism at exactly four fixed seats.

The four-axis attack a change or a diff is judged against, fixed in order
and named identically everywhere this wave ships it:

| Axis | The question it and only it asks |
|---|---|
| `spec-fidelity` | Does the change do what the intent/spec/acceptance actually asked — no less, and nothing unasked-for? |
| `invariants` | What must remain true across this change, and does it? Compatibility, replay, persisted state, error paths, concurrency. |
| `simplicity` | Is this the smallest faithful change? Dead branches, needless abstraction, duplicated logic, unnecessary surface. |
| `test-honesty` | Do the tests prove what they claim? Tautological assertions, tests that pass against the unchanged code, coverage claimed but not run, a green run that never executed the new path. |

## Contract

- **Four seats, one message.** Spawned together, each seeing only its own
  brief — the pinned revision and diff command, the spec/acceptance
  source, this axis's own definition verbatim, the finding-record columns
  (below), and a 400-word cap.
- **Never merged, never re-ranked, never traded off.** A finding belongs to
  exactly one axis — the seat that raised it. Two axes are never invited
  to re-litigate the same finding against each other.
- **Every finding enters at `status: raised`.** This is the panel's whole
  job: raise, don't confirm. Confirmation belongs to `@@refute` alone.
- **A dead or missing seat degrades the panel to fewer axes**, named in
  the calling stage's own output and carried into the close packet — never
  silently absorbed as four when only three ran.
- **The typed finding set** — one Markdown table, per §2.7 of the design
  record, columns `id`, `axis`, `claim`, `evidence`, `severity`, `status`,
  `refutation` — is the panel's one representation. `id` is
  `F-<axis-initials>-<nn>`, unique in the set.

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

Never oversold: no shipped sentence describes these seats as "blind
seats", "independent reviewers", or "separate reviews".

## What this context contributes when loaded inside a stage

- **J0 the caller must honor:** a seat cannot complete — the panel
  degrades and names the missing axis; a review that ran three axes and
  reported four is precisely what the test-honesty axis exists to catch
  in a change, and precisely what this panel must not itself do.
- **J2 the caller retains:** how to phrase each seat's brief within these
  bounds, and which of a seat's remarks are findings rather than
  commentary.
- **J1 the caller retains:** exact invocation wording, so long as all four
  seats are spawned in one message.

There is no stage library in this engine. This file is shared text pulled
into a stage's own `CONTEXT.md` by `@@` reference. A change here must be
hand-propagated to every narrowing consumer — drift by construction,
named rather than hidden.
