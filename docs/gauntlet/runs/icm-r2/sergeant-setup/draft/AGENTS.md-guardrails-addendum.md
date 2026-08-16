# AGENTS.md Guardrails addendum (draft)

Proposed new bullet for `AGENTS.md`'s `## Guardrails` section (after the
existing "`sgt init`/`sgt repo add`/`sgt group add` write only within the
estate they scaffold..." bullet, `AGENTS.md` lines 191-195), carrying
`BU-P5-006` forward now that its originally-owning stages
(`60-task-tracking-init`, `70-optional-capabilities`) are retired
(`docs/icm/re-homing-record-2026-08-12.md`) and it was never carried into
that re-homing record — it currently constrains no live behavior anywhere
in the corpus:

```markdown
- `td`, Graphify, and Treehouse are never auto-initialized without an
  explicit per-tool confirmation prompt; a declined prompt leaves state
  unchanged and reports the skip, never a silent init and never a silent
  drop. <!-- BU-P5-006 -->
```

This is a proposed addition only — not applied to the live `AGENTS.md` by
this pass. `AGENTS.md` itself is out of this ICM-R2 producer pass's file
scope beyond recording this one candidate bullet for the reconcile-and-
publish step to accept or reject.
