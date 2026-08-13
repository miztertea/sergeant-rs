# Estate manifest design — capture of in-session decisions, 2026-08-11

Transcribed from the owner grill; input to the MVP-1 contract. **v2 same
day:** read against `src/` by the review pipeline. Corrections carry
**[v2]** with a cite and their rationale in the bucketing plan's
dispositions; the owner's decisions stand except one ESCALATED fork.

## Shape

Extends the estate root's `sergeant.toml` (reuse the discovered config
file, R1): `[estate]` (name, `data_dir` defaulting `.sergeant/data`),
**[v3] `data_dir` was never carried into implementation and this line is
stale.** R-MVP1-1 (the MVP-1 contract's own ruling on `[estate]`) named
only `surfaces_dir`; `EstateSection` (`workspace.rs`) is
`deny_unknown_fields` over `name`/`default_backend`/`default_workflow`/
`surfaces_dir`, and `resolve_data_dir` (`cli.rs`) hardcodes
`estate_root.join(DEFAULT_ESTATE_DATA_DIR)` reading no manifest field at
all — so a hand-written `[estate] data_dir = "..."` following this line
literally fails closed with a `deny_unknown_fields` parse refusal rather
than overriding anything. Flagged, not implemented, at the MVP-3 fixer
pass (invariants finding MVP3-C4; GAUNTLET.md backlog entry B5):
per-Work-item scope, this is R-NS-4 new engine/config surface and wants
its own ratification, not a silent addition inside a bug-fix pass.
`[[profile]]` (existing; **[v2]** array-of-tables per `workspace.rs:179`,
unchanged here — the earlier `[profile.*]` shorthand was wrong),
`[[repo]]` entries, `[group.<name>]` tables.

**[v2] The migration is a decision, not a free extension.**
`WorkspaceFile` is `deny_unknown_fields` over exactly
`workspace`/`repository`/`profile` (`workspace.rs:172-198`), so this
design's own vocabulary is a hard `Malformed` today. Working position:
the new tables are the estate vocabulary, `[workspace]`/`[[repository]]`
stay accepted as the legacy single-repo shape, both resolving into
`Workspace`, and mixing them fails closed naming both. **ESCALATED** —
rename-with-refusal vs. coexistence is the owner's call, and MVP-1 cannot
name serde fields until it is made.

Per `[[repo]]`: `name` (mount at `repos/<name>`), `origin`
(populate/verify only — canonical clone locations are not sergeant's
business; an existing entry need only be a git repo), `base` (worktree
cut point), `instructions = "local" | "suppress"` (the trust knob:
local = actor consumes the repo's own AGENTS.md natively in its
worktree; suppress = the old `--setting-sources` behavior for foreign
repos — the adapter TRANSLATES this policy, never defines it),
`brief` (one orientation line, AI-facing string value).

**[v2]** None of the four exist, and two are more than fields.
`RepositorySpec` is `{name, path}` (`workspace.rs:33-39`) and a declared
non-repo fails closed `RepositoryNotFound` (`:274-281`) — so
populate-from-`origin` is new code, and fail-closed stays the behavior
when `origin` is absent. `base` has no plumbing: `materialize_one` cuts
unconditionally from `HEAD` (`surface.rs:340`) and must learn to resolve
a ref, HEAD when absent. And `instructions` cannot
be per-repo as written: one `--setting-sources` flag rides one `Command`
per turn (`claude.rs:874-881`) while multi-repo work executes at the
shared surface root, in no worktree (`surface.rs:152-161`). Rule:
`instructions` resolves per-Work at bind; disagreeing repos fail closed
naming them and the remedy. What `local` translates to is **unmeasured**
— L1 binds, so MVP-2 measures it before the semantics are fixed.

Per `[group.<name>]`: `repos` list (members must be declared repos,
fail closed with remedy) + optional `brief`. Consumption:
`sgt run "..." --group <name>` expands to the group's repos as Work
scope — **[v2] provisional**, not settled: NORTH-STAR.md's gaps section
lists cross-repo Work "unimplemented and uncontracted" pending the
group-expansion ruling MVP-1 owes. Until then membership needs no new
surface: `--repo` is repeatable (`cli.rs:75-77`), so a harness reading
`[group.*]` expands it itself.

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
time), on-demand (doctor/list). No hot reload, no daemon-resident cache;
the file is tiny, read fresh per operation. **[v2]** But discovery never
reaches the estate root from inside a member repo: `Workspace::discover`
anchors on `git rev-parse --show-toplevel` and reads `sergeant.toml` only
there (`workspace.rs:206-231`), and `--show-toplevel` never crosses an
inner `.git`. Estate discovery therefore walks upward from cwd *past* git
boundaries for a `sergeant.toml` carrying `[estate]`, keeping the
git-toplevel path as the zero-config single-repo fallback; #22 owns the
fixtures.

Torn-read defense in three layers: sgt's pens write atomically; strict
fail-closed parse is the tripwire for foreign pens (refused operation
names the defect, retry after save); **pin-at-bind** caps the blast
radius — a Work's binding snapshots the manifest policy it launched
under, so mid-flight edits affect the next Work, never a running one.
**[v2]** That *extends* the `workflow.bound` precedent, it does not reuse
it: the payload pins `Profile` and `stage_bindings` whole but the
workspace only as a bare `plan.workspace.name` string
(`engine.rs:1044-1051`), so per-repo policy means widening it to the
resolved `Vec<RepositorySpec>` — new fields in an immutable event,
additive-only thereafter.

## Wrongness contract

Binary parses fail-closed with line/key/expected named (#47 precedent);
wrongness scoped per-entry (a broken repo blocks works targeting it,
not the estate); doctor is the shared human/AI validation loop — every
failing check names a remedy. Manifest changes are not journaled (git
versions the file); each Work's journaled binding records the policy
snapshot it ran under.

**[v3]** The manifest *edit* pens (`sgt init`/`repo add`/`repo remove`/
`group add`/`group remove`) violated the per-entry half of this
contract from MVP-3 through the fixer pass that found it (invariants
finding MVP3-C1): `domain::manifest::validate` round-tripped every edit
through the strict, on-disk-resolving parser, which fails at the *first*
declared repository missing from disk — so one broken repo blocked
*every* manifest edit, not just works targeting it, and a freshly `git
clone`d estate (which gitignores `repos/`) could not be edited at all
until every declared repository was manually re-cloned. Fixed by adding
`Workspace::from_config_structural` (schema-level only, no git
resolution) as the edit pens' validator; `sgt run --group`'s
client-side group expansion (MVP3-C2) had the same coupling and the same
fix (`Workspace::declared_groups_scoped`). Pinned by
`domain::manifest::tests::a_missing_unrelated_repo_does_not_block_edits_
that_do_not_touch_it`.

## Deliberately absent (Ponytail — wait for measured need)

Per-repo backends/models, write-protection flags, path overrides,
per-repo workflow defaults, per-group instruction files, a third
`instructions` value.
