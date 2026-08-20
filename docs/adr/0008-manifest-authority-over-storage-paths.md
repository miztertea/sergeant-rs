# ADR 0008: Manifest authority over storage paths

**Status:** Accepted, 2026-08-14. **Amended in part, 2026-08-20** — see
"Amended by the estate-root integration" below (gauntlet finding C7a).

## Context

`resolve_data_dir` (`src/cli.rs:406-419`) resolves the data dir through a
fixed rung order: `--data-dir`, then `SGT_DATA_DIR`, then estate discovery
walking up from `cwd`, then falls through to a pre-estate platform default
(`crate::platform::data_dir::fallback_dir`, which itself prefers
`XDG_DATA_HOME` over `HOME` on Linux). The doc comment on `resolve_data_dir`
already flagged this precedence — specifically, whether estate discovery
should come before the pre-estate fallback at all — as "an owner ruling
tracked separately in **#80** and is not decided here." Separately, the
manifest already has precedent for narrowing daemon-owned defaults: `[estate]
surfaces_dir` (`src/domain/workspace.rs:199-204`, overridable by
`SGT_SURFACES_DIR`) lets a manifest declare where surfaces live, resolved
to an absolute path when present and otherwise leaving the daemon's own
default in force. `resolve_data_dir` has no equivalent — it walks up to
*find* `sergeant.toml` and then ignores its contents entirely, hardcoding
`estate_root.join(DEFAULT_ESTATE_DATA_DIR)` (`src/domain/manifest.rs:85`).
Issue #64 is the third strand: it proposes flipping the surfaces default
outside every checkout via `XDG_STATE_HOME`, filed originally as a
self-hosting hazard — sergeant's own machine-local state living inside a
directory sergeant itself might operate on.

## Decision

Three parts, one ruling.

**(a) Estate-first precedence stands (D4).** The existing rung order in
`resolve_data_dir` — `--data-dir`, then `SGT_DATA_DIR`, then estate
discovery from `cwd`, then the pre-estate platform fallback
(`XDG_DATA_HOME`, then `HOME`) — is upheld as-is; estate discovery keeps
its position ahead of the XDG fallback. `XDG_DATA_HOME` is one global path
per machine, and estates are per-directory. If XDG precedence won instead,
every estate on a machine would collapse onto the same journal, the same
blob store, and the same exclusive daemon lock, with estate A's Works able
to reference estate B's repos — that is a collision, not a matter of taste
between two reasonable defaults. ADR 0006's explicit launch-time estate
binding removes most of the surprise that made estate-before-XDG feel
wrong in the first place: once a session is bound to its estate
deliberately at launch, silently falling back to a global path is no
longer the failure mode it would otherwise be.

**(b) The manifest gains `[estate] data_dir`, for symmetry with
`surfaces_dir` (D4).** `resolve_data_dir` walks up specifically to find
`sergeant.toml` and then discards it, hardcoding
`DEFAULT_ESTATE_DATA_DIR` regardless of what the manifest says. Today the
manifest is authority for repos, groups, profiles, and `surfaces_dir`, but
not for where its own state lives — an asymmetry with no principled
reason behind it. The manifest should be authority for both or neither;
this decision adds `[estate] data_dir` to close that gap.

**(c) Issue #64 is re-ruled, not implemented (D4).** #64's proposed default
flip — moving surfaces outside every checkout via `XDG_STATE_HOME` — is
rejected in its filed form. With the estate now a deliberate, explicitly-
bound directory (per ADR 0006), splitting sergeant's own state across
`~/.local/state` and the estate would make the architecture *less*
coherent, not more: `NORTH-STAR.md`'s own ownership section already states
the destination this decision holds to — "machine-local truth is in-estate
and gitignored." The hazard #64 was filed against is already avoided in
practice, not merely in theory: `sgt repo add` clones a declared repository
into `repos/<name>`, a sibling of `.sergeant/data` under the estate root,
never an ancestor of it (`src/cli.rs:233-239`, `src/domain/manifest.rs:463`),
and the WATCH pilot ran sergeant-on-sergeant cleanly against exactly this
layout.

The cost of re-ruling #64 rather than building it is recorded explicitly,
not softened: #64 is filed as a genuine self-hosting contradiction, and the
contradiction is real. Surfaces under `.sergeant/data/surfaces/` *do* sit
inside the sergeant-rs distro checkout when sergeant operates on itself.
It is invisible today only for two reasons, both incidental rather than
structural: `.sergeant/data` is one of the manifest's own gitignored
entries (`src/domain/manifest.rs:72-78`), so `git status` never surfaces
it; and the engine's own refusal checks (guarding against operating on an
estate with an uncloned repository, `src/domain/manifest.rs:322-323`) run
against the *target* repository being worked on, not the outer sergeant-rs
checkout the estate happens to be nested inside. Re-ruling #64 means
accepting that contradiction as permanent and saying so out loud, rather
than leaving it masked by two coincidences of implementation.

## Alternatives considered

**XDG-first precedence** (falling through to `XDG_DATA_HOME` ahead of, or
instead of, estate discovery) was the implicit alternative to (a), and is
rejected on the collision argument above: XDG is a single global path,
estates are not.

**Implementing #64 as filed** — the `XDG_STATE_HOME` default flip — was
considered as the straightforward resolution to the issue as written, and
rejected in favor of re-ruling it, per (c) above: it would make the
architecture less coherent under the estate-as-deliberate-directory model
ADR 0006 establishes, not more.

## Consequences

The manifest gains a new `[estate] data_dir` field, mirroring
`surfaces_dir`'s shape. Implemented; see GAUNTLET.md's "ADR 0008" ledger
entry for the shipped shape, the precedence rung it chose, and
verification.

#64 closes on this ruling rather than on the implementation it originally
proposed; the self-hosting contradiction it named stays real and is now an
accepted, explicit property of this architecture (surfaces nested inside
the very checkout sergeant might be operating on) rather than an open
hazard awaiting a fix. Anyone re-reading #64 after this ADR should not
expect the `XDG_STATE_HOME` flip it originally proposed to land.

## Amended by the estate-root integration (C7a, 2026-08-20)

Part (a) of this decision — "estate-first precedence stands" — was built on
a mechanism the estate-root contract removes. `resolve_data_dir`'s third rung
was *estate discovery walking up from `cwd`*: R-MVP1-12's ancestor walk,
filesystem-first, crossing Git boundaries, bounded at `$HOME`. Phase D
deletes that walk outright, along with the zero-configuration Git-toplevel
fallback beneath it (proposal §4.1: "Sergeant does not search parents and
does not use Git to infer an estate"). R-MVP1-12 is **superseded**.

What survives, and what changes:

- **The manifest keeps its storage-path authority.** (b) is untouched:
  `[estate] data_dir` is still read, still resolved the same way
  `surfaces_dir` is, still narrows the daemon's own default rather than
  outranking an explicit override. The *discovery* of the manifest changes;
  its *authority* does not.
- **The rung order is untouched.** `--data-dir`, then `SGT_DATA_DIR`, then
  the estate, then the pre-estate platform fallback. (a)'s collision
  argument — `XDG_DATA_HOME` is one global path per machine, estates are
  per-directory — is exactly as true under exact-root admission as it was
  under the walk, and is the reason the estate rung stays where it is.
- **What the estate rung *means* changes.** It is no longer "the nearest
  `[estate]`-bearing `sergeant.toml` at or above `cwd`" but "`cwd` itself, if
  `cwd` is an estate root" — one deterministic check against one directory
  (`Estate::is_estate_root`). From a `repos/<name>` mount one level down,
  the rung no longer matches at all and resolution falls through to the
  platform default, where before it would have found the estate above.
- **(a)'s own rationale gets stronger, not weaker.** It leaned on ADR 0006's
  explicit launch-time estate binding to argue that estate-before-XDG is not
  a silent surprise. Exact-root admission finishes that argument: every
  estate-scoped command now refuses outside a root with §4.4's loud
  diagnostic before it resolves a data dir at all (§4.3), so the case where
  a caller silently gets a data dir they did not intend has no path left.
- **(c) is untouched.** #64 stays re-ruled, and the self-hosting
  contradiction it named stays an accepted, explicit property.

The proposal's own disposition register carries the same ruling from the
other side: "R-MVP1-12 / `Workspace::discover_scoped`: upward estate
discovery across Git boundaries — **Superseded.** Estate-scoped commands
operate only when the current directory itself contains a valid
`sergeant.toml`."

## Open questions

Whether (c) closes #64 outright or leaves it open, relabeled to record the
accepted contradiction rather than a pending fix, was not specified in the
interview.

The precedence and override behavior of a manifest-declared `data_dir`
relative to `--data-dir` and `SGT_DATA_DIR` — whether it slots in at the
same rung as estate discovery currently does, or somewhere else in the
existing five-rung order — is not specified beyond "the manifest should be
authority for both or neither." The implementation picked a rung
(`SGT_DATA_DIR`/`--data-dir` still outrank a manifest `data_dir`) as a
recommendation, not an owner ruling — argued and left open for
adjudication in GAUNTLET.md's "ADR 0008" ledger entry.
