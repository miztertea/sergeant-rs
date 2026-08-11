# Estate manifest design — capture of in-session decisions, 2026-08-11

Transcribed from the owner grill; input to the MVP-1 contract. Under
review — the four-reviewer pipeline attacks this alongside the
bucketing plan.

## Shape

Extends the estate root's `sergeant.toml` (reuse the discovered config
file, R1): `[estate]` (name, `data_dir` defaulting `.sergeant/data`),
`[profile.*]` (existing), `[[repo]]` entries, `[group.*]` tables.

Per `[[repo]]`: `name` (mount at `repos/<name>`), `origin`
(populate/verify only — canonical clone locations are not sergeant's
business; an existing entry need only be a git repo), `base` (worktree
cut point), `instructions = "local" | "suppress"` (the trust knob:
local = actor consumes the repo's own AGENTS.md natively in its
worktree; suppress = the old `--setting-sources` behavior for foreign
repos — the adapter TRANSLATES this policy, never defines it),
`brief` (one orientation line, AI-facing string value).

Per `[group.<name>]`: `repos` list (members must be declared repos,
fail closed with remedy) + optional `brief`. Consumption:
`sgt run "..." --group <name>` expands to the group's repos as Work
scope.

Field rule: **structure is for the binary, string values are for the
AI, comments are for the human.** The binary never parses prose.

## Edit semantics — three pens, one file

Hand-edit canonical (tracked TOML, git is history/rollback). CLI verbs
(`sgt repo add/remove/list`, `sgt group add/remove/list` with mkdir-p
member semantics) are conveniences editing the same file — validate at
the pen, atomic write (temp + rename), advisory lock around sgt's OWN
read-modify-write only (agent-race lost-update guard; foreign editors
stay last-write-wins). The harness edits it like any file, with doctor
as its feedback loop. TUI later = the same verbs with a screen.

## Read semantics

Read at discrete moments only — daemon start, work submission (bind
time), on-demand (doctor/list). No hot reload, no daemon-resident
cache; the file is tiny, read fresh per operation. Torn-read defense in
three layers: sgt's pens write atomically; strict fail-closed parse is
the tripwire for foreign pens (refused operation names the defect,
retry after save); **pin-at-bind** caps the blast radius — a Work's
binding snapshots the manifest policy it launched under (the
workflow.bound precedent), so mid-flight edits affect the next Work,
never a running one.

## Wrongness contract

Binary parses fail-closed with line/key/expected named (#47 precedent);
wrongness scoped per-entry (a broken repo blocks works targeting it,
not the estate); doctor is the shared human/AI validation loop — every
failing check names a remedy. Manifest changes are not journaled (git
versions the file); each Work's journaled binding records the policy
snapshot it ran under.

## Deliberately absent (Ponytail — wait for measured need)

Per-repo backends/models, write-protection flags, path overrides,
per-repo workflow defaults, per-group instruction files, a third
`instructions` value.
