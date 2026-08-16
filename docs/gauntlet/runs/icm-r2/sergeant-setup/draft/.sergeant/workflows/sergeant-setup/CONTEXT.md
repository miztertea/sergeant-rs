# Sergeant Setup (retired)

Retired package. This file is preserved as a pointer, not workflow
orientation — there is no admitted workflow left to orient an actor into.

## Why this retired

See `docs/gauntlet/runs/icm-r2/sergeant-setup/adjudication-draft.md` in
full. Summary: every surviving behavior unit in this package's two stages
(`05-file-capability-gaps`, `30-project-interview`) and its standing
constraints classified to PL-0 (already absorbed by `AGENTS.md` or current
`sgt` CLI verbs), PL-1 (a stable invariant that belongs in `AGENTS.md`, not
here), or PL-2 (live Captain-session judgment that belongs in
`skills/estate-navigation/SKILL.md`) — none survive at PL-4/PL-5 (workflow/
stage). A remaining fragment of the project-registration interview cites
fields (`agent_instructions` free text, per-repo role, project-level GitHub
identity, Graphify output path) absent from the current `sergeant.toml`
schema; that fragment is parked, not translated, pending a schema decision
this docs-only pass has no authority to make.

## Destinations

| Behavior | Moved to |
|---|---|
| Unsupported capability → tracked td issue, y/yes gated | `skills/estate-navigation/SKILL.md` |
| Interactive repo/group registration (name, origin, membership) | `skills/estate-navigation/SKILL.md` |
| Write only to Sergeant-owned paths; never touch other tools' config | `AGENTS.md` Guardrails (already present, `BU-1263`/`BU-1264`/`BU-1295`) |
| td/Graphify/Treehouse consent-gated auto-init | `AGENTS.md` Guardrails (new bullet, was orphaned) |
| Per-repo role, free-text agent instructions, GitHub identity, Graphify path | Parked — no current schema field |
| Preview-before-write, timestamped backup | Absorbed — no whole-file write moment remains under the incremental `sgt repo add`/`sgt group add` model; `AGENTS.md`'s destructive-state guardrail (`BU-0050`) and `sgt init`'s no-op-not-reset behavior already cover the protective intent |

## Retired (from the prior MVP-5 F2 pass, unchanged)

`00-detect-prerequisites`, `10-install-commands`, `20-global-config`,
`40-repair-existing`, `60-task-tracking-init`, `70-optional-capabilities` —
see `docs/icm/re-homing-record-2026-08-12.md`. Unaffected by this retirement;
already absorbed by `sgt init`/`sgt doctor` before this pass began.

## Provenance

`docs/gauntlet/promoted-provenance/sergeant-setup.md` (archived, all eight
original stages). This package's own `CONTEXT.md` historically cited a
`provenance.md` alongside it that was never actually created — noted in this
pass's adjudication record's Validation evidence section.
