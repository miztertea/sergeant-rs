# Model assignment

Resolved as `@@model-assignment` from
`.sergeant/common/contexts/model-assignment.md` per
`docs/icm/convention.md` §4. Policy context (Placement Ladder PL-3). The
homeless policy, homed at last (F.11 of the distro content rebuild's
design record): this file and `@@test-first` are the two canonical homes
this wave gives content that previously had none. Self-contained —
`@@` resolves only inside an active stage's context (ADR 0014 decision
10), so this file states its own rule in full rather than pointing
elsewhere for it.

## The rule

**Sonnet by default.** A heavier model is assigned only where the work
earns it, named and justified in the same place the assignment is made —
not by default, not by habit. The captain's own tier is never assigned
below the captain: a delegated actor's model is never heavier authority
than the session directing it, only potentially more expensive compute
for a bounded task.

## Seat-tier guidance for panel stages

Panel and refuter seats (`@@panel`, `@@refute`, `@@fan-out-evidence`) are
sub-agents of the stage's own actor. They inherit the calling actor's
model assignment. **A stage cannot assign a model to its seats**, and no
shipped text may imply it can — there is no mechanism in this engine for
a stage to select a different model tier for a sub-agent it spawns, and a
package that writes as if there were is describing a capability that does
not exist.

## What this context contributes when loaded inside a stage

- **J0 the caller must honor:** a task appears to need a heavier model
  than the default and no named justification is available — state the
  need and the gap rather than silently assigning up or silently staying
  at default.
- **J2 the caller retains:** whether this specific task's own work
  justifies a heavier assignment, named and justified inline.
- **J1 the caller retains:** none — the default is sonnet unless justified,
  full stop.

There is no stage library in this engine. This file is shared text pulled
into a stage's own `CONTEXT.md` by `@@` reference. A change here must be
hand-propagated to every narrowing consumer — drift by construction,
named rather than hidden.
