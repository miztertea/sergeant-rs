# Shared rules (both branches)

Layer 3 reference, stable across every run of this workflow — cited by
both `20L-build-logic` and `20U-build-variants`'s own Inputs tables and
Behavior contract sections, per `docs/icm/convention.md`'s four-layer
model. Carries forward four rules from the N1 harvest, each already
classified `representation: shared-context` at N1 but never actually
materialized anywhere in this package before ICM-R3 (corrected per the
independent reviewer from an initial "harvest gap" misclassification —
these four were already extracted, just never built; only a fifth item,
"skip the polish," remains a genuine unbuilt harvest gap, tracked
separately, not drafted here).

- **Throwaway from day one, and clearly marked as such.** Locate the
  prototype code close to where it will actually be used (next to the
  module or page it's prototyping for) so context is obvious — but name
  it so a casual reader can see it's a prototype, not production. For
  throwaway UI routes, obey whatever routing convention the project
  already uses; don't invent a new top-level structure.
- **One command to run.** Whatever the project's existing task runner
  supports — `pnpm <name>`, `python <path>`, `bun <path>`, etc. The user
  must be able to start it without thinking.
- **No persistence by default.** State lives in memory. Persistence is
  the thing the prototype is *checking*, not something it should depend
  on. If the question explicitly involves a database, hit a scratch DB
  or a local file with a clear "PROTOTYPE — wipe me" name.
- **Surface the state.** After every action (logic) or on every variant
  switch (UI), print or render the full relevant state so the user can
  see what changed.

## Not carried forward here

Item 4 ("skip the polish — no tests, minimal error handling, no
abstractions") has no corresponding extracted behavior unit anywhere in
the N1 harvest — a genuine harvest gap, not a placement question. Filing
its extraction and placement is left as a tracked, unresolved finding,
not fabricated here.
