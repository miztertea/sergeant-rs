# T-SERIES-1 — blind critic: invariants

Axis: does any section of `reference/proposal-tui-t-series.md` §1–§25 violate
`NORTH-STAR.md`'s ownership boundaries or its Never list, the R-NS-* rulings
(R-NS-6 especially — grading §5.6's own restatement rather than accepting
it), the architecture invariants in `docs/DEVELOPMENT.md`, or the
equal-client boundary `tests/m6_surfaces.rs` t5 enforces today, including
the Ponytail Minimality Ladder's rung-logging convention applied to §22's
decision register, per `docs/gauntlet/contracts/T-SERIES-1.md`.

Context read: `reference/proposal-tui-t-series.md` (full, §1–§25),
`NORTH-STAR.md`, `docs/DEVELOPMENT.md`, `reference/notes/ideaos-agent-contract.md`,
`docs/gauntlet/notes/north-star-draft-2026-08-11.md` (source of R-NS-1..5,
which `NORTH-STAR.md` only cites as "as drafted"), `tests/m6_surfaces.rs`
(t5, t5b), `src/lib.rs` (module list), `src/cli.rs` (repo/group/Doctor
implementation location), `docs/gauntlet/runs/foundation-1/critics/invariants.md`
(precedent shape).

Two findings, both landing on the extract-on-contact design for Estate/Doctor
(§5.3, §12, §16.2–16.3) and the Ponytail register (§22). One additional item
considered and filed at lower confidence on §5.6's restatement of R-NS-6,
per the contract's specific instruction to grade that restatement rather
than accept it.

---

## Finding 1: `inv-estate-doctor-breaks-t5`

**Severity:** error

**Section:** §5.3 ("Estate-local behavior is shared, not tunneled through
the daemon", Decision T2-14), §12 (Estate), §16.2 (Estate operations,
Decision T2-57), §16.3 (Doctor report, Decision T2-58), §19.5/§19.6
(Estate/Doctor parity tests), §22 (T2-14, T2-43, T2-44, T2-45, T2-57, T2-58)

**Claim at issue:** §5.3, verbatim: "The TUI consumes repo/group and Doctor
behavior through narrow typed local operations extracted from the current
CLI implementation... This is an explicit refinement of the old 'TUI imports
only ApiClient' source-scan rule: `daemon-owned facts → ApiClient only`,
`estate manifest edits → shared local Estate operations`, `installation
checks → shared local Doctor report`." §16.2/T2-57: "Extract on contact...
Illustrative, nonbinding shape: `list_repositories(...)`, `add_repository(...)`
[...]." §16.3/T2-58: "Extract Doctor's structured `Check`/`Report` result
from CLI formatting and let both CLI and TUI consume it."

**What I checked:** The actual boundary this "refinement" claims to revise.
`docs/DEVELOPMENT.md`'s "Clients are equal" invariant, verbatim: "The CLI
and TUI reach state only through the loopback HTTP/SSE API (`src/api.rs`)
via `ApiClient`. This is enforced by tests, not convention:
`tests/m6_surfaces.rs` t5 scans `tui.rs` for internals imports — widening
that reach fails the test by design." I then read t5 itself
(`tests/m6_surfaces.rs:2308`): `crate_paths(&source)` collects every
`crate::`/`super::`/`self::`-rooted path `tui.rs` names, and the test
asserts `paths == vec!["api".to_string()]` exactly — not "no forbidden
tokens," an exact-equality allowlist of one entry. `NORTH-STAR.md`'s
Ownership section states the same rule with no estate/Doctor carve-out:
"Surfaces (CLI, TUI, harnesses) own presentation and steering through the
API only." The draft source for R-NS-4 (`docs/gauntlet/notes/north-star-draft-2026-08-11.md:98`)
is more explicit still: "**R-NS-4 Statelessness rule (surfaces).** A surface
may never be the only place a fact exists, and reaches state only through
the API (`tests/m6_surfaces.rs` t5 already enforces this)." — the draft
names t5 as the rule's own enforcement mechanism, by name.

I then checked where the behavior §16.2/§16.3 propose to extract actually
lives today. `crate::domain::manifest` already holds `add_repo`,
`remove_repo`, `add_group`, `remove_group` (`src/cli.rs:945,957,1002,1011`
call them directly) — a pure manifest edit with no daemon involved, per the
CLI's own doc comments at those call sites ("a pure manifest edit/read, no
daemon"). Doctor's `Check`/`Report` types are defined inline inside
`src/cli.rs` (`src/cli.rs:1635,1677`), not in a separate module today. There
is no existing module that already sits outside `tui.rs`'s current `["api"]`
allowlist and also outside `cli.rs` that the TUI could import from without
either (a) widening `crate_paths(tui.rs)` past `["api"]`, which t5 fails on
by construction, or (b) importing from `crate::cli` itself, which would
invert "clients are equal" into "TUI depends on CLI" — a hierarchy neither
`DEVELOPMENT.md` nor the proposal's own "CLI and TUI format the same
outcomes differently" (§5.3) framing endorses.

**What I found:** §5.3 correctly distinguishes *daemon-owned* facts (which
stay `ApiClient`-only) from *estate-local* facts (repo/group manifest,
Doctor), and the underlying design judgment — Doctor must work with no
daemon running, so routing it through a new daemon API would be wrong, a
point §23.3 ("New daemon APIs for repo/group/Doctor — Rejected") makes
explicitly and I did not find fault with. But calling this "an explicit
refinement of the old... rule" understates what actually happens
mechanically: it is a widening of `tui.rs`'s import surface past the exact
allowlist t5 currently enforces, and R-NS-4's own draft text names t5 as
the concrete definition of "reaches state only through the API" for
surfaces. Nothing in §5.3, §12, §16.2, §16.3, §19.5, or §19.6 states that
t5 itself must change, names what its new assertion should be (e.g. `paths
⊆ {"api", "estate"}` instead of `paths == ["api"]`), or acknowledges that
the exact-equality form of the test — the shape that caught a regression
hidden behind a syntactically-different-but-semantically-identical import
in t5b's own regression note — needs deliberate loosening rather than
silent breakage. §19.5 (Estate parity tests) pins output *parity* between
the CLI and the shared function; it never mentions the import-boundary test at
all. This is the same failure mode FOUNDATION-1 Finding 1 named: a section
asserts an invariant is preserved or merely "refined" where the proposal's
own cited mechanism does not establish that a currently-enforced,
by-name-cited test keeps passing.

**Does the section survive the correction?** Yes, but not as currently
worded. §5.3 (or §19.5/§19.6, wherever the proposal specifies its test
program) needs an explicit line stating that t5's assertion is revised as
part of T3 (§20.4, Estate) to allow the specific new shared-operations
module(s) `tui.rs` will import, naming that module, and confirming the
revision keeps the forbidden-token half of t5 (`ApiState`, `registry`,
`Journal`, `Analytics`, `Engine`, `blocking_lock`) intact — because that
half is what actually stops the TUI from reaching daemon internals, while
the `paths == ["api"]` half is what the extract-on-contact design
necessarily breaks. As written, the proposal both cites t5's governing rule
by name (indirectly, via `DEVELOPMENT.md`) and designs directly through it
without saying so.

---

## Finding 2: `inv-t2-40-unjustified-r7`

**Severity:** warning

**Section:** §11.2 (Read-only catalog route, Decision T2-40), §22 (register
row T2-40), cross-referencing §8.7 (Decision T2-31) as the proposal's own
comparison case

**Claim at issue:** §22's register preamble, verbatim: "Any implementation
decision not represented here is logged in the milestone report. **Every
new R7 names failed lower rungs.**" §11.2, Decision T2-40 in full: "**(R2/R6/R7):**
Add one workflow catalog projection because the TUI must not privately
reinterpret executable procedure." No further text in §11.2 names which of
R1–R6 were checked or why they failed.

**What I checked:** `reference/notes/ideaos-agent-contract.md`'s rung table
and its binding "Rung logging convention (this repo)": "An `R7` entry must
name which lower rungs were checked and why they failed." I then found
every R7-tagged decision in the document (`grep -n "R7"` across the full
text): exactly two — T2-31 and T2-40. T2-31 (§8.7) is the proposal's own
model citizen for this convention: "Failed lower rungs: R1 fails: multiline
deliberate input is settled. R2 fails: the current one-line `String` buffer
cannot satisfy it. R3/R4 fail: standard Rust and terminal events do not
provide editor behavior. R5 fails: the currently installed dependency set
has no editor. R6 fails: correct wrapping, visual-row cursor movement,
paste, delete, and scrolling are not a tiny composition." T2-40 has no
equivalent trace anywhere in §11 — only a list of what the new route
*reuses* (estate/workspace discovery, workflow loader, root publication
boundary, embedded fallback, "existing Axum and `ApiClient` patterns"),
which argues for R2/R5 reuse, not for why R7 (new machinery) was reached
despite that reuse.

**What I found:** T2-40 is exactly the shape the convention's R7 clause
exists to police: genuinely new code (a new authenticated HTTP route) that
also reuses a great deal of existing machinery, which is precisely the
situation where a reader cannot tell, without the trace, whether R7 was
actually necessary or whether a lower rung (R5 — the route is just a
handler over Axum, already an installed dependency; or R6 — a tiny
composition wiring existing loader/validation functions to an existing
router) would have sufficed. The register's own preamble commits to every
R7 naming failed lower rungs; this is the second of exactly two R7 entries
in the whole document, and it is silent. This is the same gap FOUNDATION-1
Finding 3 found in the predecessor proposal-grading unit (there, four of
seven changes were unlogged against the ladder; here the document is far
more disciplined — 62 of 64 decisions carry a rung and T2-31 sets a genuine
standard — which makes the one silent R7 more conspicuous, not less,
precisely because the document proves elsewhere that it knows how to do
this correctly.

**Does the section survive the correction?** Yes — this is an addition, not
a correction of anything currently wrong in the route's design itself. §11.2
needs the same four-to-five-line trace T2-31 already models: state why R2
(a fully existing route already does this — none does), R5 (an installed
crate already provides catalog projection — none does, Axum only supplies
the routing primitive, not the projection), and R6 (a one-line/tiny
composition over an existing endpoint — no existing endpoint returns
workflow definitions) each fail, before landing on R7 for the new handler
itself.

---

## What I considered and did not file

- **§5.6's restatement of R-NS-6.** The contract specifically asks this
  restatement be graded rather than accepted, so I traced it closely. §5.6,
  verbatim: "R-NS-6 distinguishes execution mechanics from the harness-owned
  conversation. `respond` answers a parked request. It is not a generic
  message operation." The actual ruling (`NORTH-STAR.md`) is an ownership
  assignment, not a bare distinction — "sgt owns message *mechanics* to a
  running execution (`needs_input`/`respond`, journaled); the harness owns
  the *conversation*" — and it carries two clauses §5.6 never restates:
  "Nothing conversational is ever engine work" (the ruling's actual
  falsifiable test) and "whether a transport's actor can ask mid-run is a
  measured per-transport capability with runtime withdrawal, never new hold
  machinery" (the clause that would govern exactly the kind of feature
  T-Series must not add). I checked whether the proposal's actual behavior
  violates either dropped clause and found it does not: §6.2 explicitly
  excludes "interrupt behavior added merely because the backend trait
  contains an interrupt capability," which is the "never new hold
  machinery" clause applied correctly — just uncited back to R-NS-6, three
  sections after §5.6 restates the rule incompletely. Per the same standard
  FOUNDATION-1 Finding 2 used (an argument citing a mechanism it does not
  fully have standing to cite is a defect in the argument independent of
  whether the conclusion holds), this is arguably still a finding. I am
  naming it here rather than filing it as a numbered finding because,
  unlike Finding 1, no test or downstream section is left unreconciled by
  the gap — §6.2's non-goals list already carries the substance the
  restatement omits, so a reader who reads past §5.6 does not end up
  misled about what T-Series actually does. A future critic revisiting this
  axis on a later revision should still consider filing it if §6.2 and §5.6
  ever drift apart.
- **NORTH-STAR's Never list ("fleet as a domain object," "reconstructed
  tmux-era supervision," etc.).** Checked all six items against every
  section that touches Fleet (§10), Estate (§12), and the composer (§15).
  Fleet remains a client-side view over `GET /v1/work` with no persisted
  identity of its own (§10.3's filters and §7.3's drawer both explicitly
  "store no notification state" / read live from Fleet); PTY/harness
  supervision is explicitly excluded (§6.2, §23.3 "Embedded harness/PTTY —
  Rejected. ADR 0006 deliberately chooses `exec`, never supervise."). No
  finding.
- **"One owner" (daemon exclusively owns the data dir and process
  handles).** The estate manifest is Estate-owned config, not daemon-owned
  durable state (`NORTH-STAR.md`'s Ownership section draws this line
  explicitly), and the CLI already writes it directly with no daemon
  involved today (`src/cli.rs:945` etc., doc-commented "a pure manifest
  edit... no daemon"). §16.2's manifest locking/atomic-writes list matches
  that existing scheme. No finding — this is Finding 1's territory
  (equal-client import boundary), not a "one owner" violation.
- **Register rung-to-decision consistency.** Cross-checked all 64 inline
  `Decision T2-NN (Rn...)` tags against their §22 register row; every rung
  combination matches exactly (e.g. T2-14 R2/R6 both places, T2-31 R7 both
  places, T2-56 R2/R5/R6 both places). No mismatch found beyond Finding 2's
  missing justification text, which is a content gap, not a rung
  disagreement.
