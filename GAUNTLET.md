# Gauntlet Ledger — moved

This file's content — the Deviation register, Backlog, and all 45 Ledger
entries — has been decomposed into typed records in the
`sergeant-rs-workspace` repo's knowledge library (ADR 0014 decisions 9 and
17; Phase 4 step 3a of `sergeant-rs-workspace`'s
`knowledge/evidence/reference/proposal-product-workspace-split.md`, prior
to this file's own migration).

The filename stays here, unlinked from its old content, so that existing
references to `GAUNTLET.md` (e.g. `docs/DEVELOPMENT.md`) do not 404.

Where the record now lives, in `sergeant-rs-workspace`:

- Deviation register and Backlog → `knowledge/rulings/deviations/`
- Ledger entries → `knowledge/evidence/runs/` (`milestone-run-record` type)
- Owner rulings quoted verbatim inside ledger entries →
  `knowledge/rulings/owner-rulings/`
- Audit trail for the decomposition itself →
  `knowledge/evidence/decomposition-gauntlet-2026-08-17.md`

The per-milestone contracts this file's header pointed at
(`docs/gauntlet/contracts/`) moved with the rest of the bulk evidence
corpus to `knowledge/evidence/gauntlet/contracts/` in the same repo
(Phase 4 step 4).

Nothing here is append-only anymore — see `knowledge/rulings/index.md`
and `knowledge/evidence/index.md` for the two disciplines that replaced
this file's single one.
