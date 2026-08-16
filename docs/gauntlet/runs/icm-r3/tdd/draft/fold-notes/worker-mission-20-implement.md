# Proposed fold: `.sergeant/workflows/worker-mission/20-implement/CONTEXT.md`

Not a live edit (`docs/adr/0013-icm-r0-owner-rulings.md` decision 6) — the
`worker-mission` package is not itself part of this ICM-R3 pass's assigned
scope (`tdd` only); this note is the concrete diff a later pass over
`worker-mission` should apply once the `tdd` REHOME is accepted.

## Current `## Delegation` section

```markdown
## Delegation

This stage's outcome is produced by running **diagnose-bug, prototype,
tdd, implement, or deepen-module (whichever 10-triage-and-route selected)**
to its own completion (context composition today — see `docs/icm/
convention.md` §4 on `@@name` versus true nested-workflow invocation,
which does not exist yet).
```

## Proposed replacement (the `tdd` branch only — the other four selections
are unaffected by this ICM-R3 pass and keep their current wording)

```markdown
## Delegation

This stage's outcome is produced by running **diagnose-bug, prototype,
implement, or deepen-module** (whichever `10-triage-and-route` selected) to
its own completion (context composition today — see `docs/icm/
convention.md` §4 on `@@name` versus true nested-workflow invocation, which
does not exist yet); **or**, when `10-triage-and-route` selected the TDD
discipline directly rather than routing through `implement`, by applying
`@@tdd` and `@@test-quality` in place — `tdd` is a shared technique
context, not a dispatchable workflow, as of ICM-R3.

### J0 — must become `needs_input` (added when the `tdd` branch applies)
- Seam confirmation per `@@tdd`: no test is written at an unconfirmed
  seam.
```

Same underlying finding as the `implement` fold note: the seam-confirmation
requirement was previously only visible by following the delegation into
`tdd`'s own `00-agree-seams` stage in full.
