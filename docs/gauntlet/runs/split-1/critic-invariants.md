# SPLIT-1 — critic report, axis: invariants

Blind critic seat 2 of 4. Grades `reference/proposal-product-workspace-split.md`
§1–§13 against `NORTH-STAR.md` (ownership boundaries, Never list, R-NS-*
rulings), `docs/DEVELOPMENT.md`'s architecture invariants, ADRs 0001–0014,
BU-0109, and the Ponytail Minimality Ladder (`reference/notes/ideaos-agent-
contract.md`). Did not read any other file under `docs/gauntlet/runs/split-1/`.

## Hazards named in the contract, tested and cleared (not findings)

- **Embedding the distro in the binary (decision 1) vs. ADR 0002's platform
  boundary.** No collision. ADR 0002 governs `#[cfg]`-selected *platform*
  modules (compile-time Linux/macOS/WSL2 facts) vs. a `Platform` trait; it says
  nothing about doctrine/template content, and the proposal's embedding
  mechanism (§4.7, Phase 3) does not touch `src/platform/` or introduce
  per-platform dispatch. Different axis entirely.
- **Embedding the distro in the binary vs. ADR 0008's manifest authority over
  storage paths.** Partial concern — see Finding 4 below. There is no direct
  collision (the proposal doesn't contradict any of ADR 0008's three rulings),
  but the proposal is silent on a question ADR 0008 already settled the
  *principle* for.
- **The workspace estate mounting `sergeant-rs` vs. NORTH-STAR's "`repos/` is
  a mount, never a dev_root."** No violation. §3.1 and Phase 4 describe
  `sergeant-rs` as a working-set entry under the workspace's `repos/`, with
  the workspace's own tracked content (`sergeant.toml`, `.sergeant/local/`,
  `DEVELOPMENT.md`, the knowledge library) held separately at the estate
  root — exactly the shape NORTH-STAR's Ownership section describes, and the
  precedent ADR 0008 cites (the WATCH pilot's sergeant-on-sergeant layout)
  already validates it.

## Findings

### 1. [error] Phase 3 — the plan's only new engine work — carries no Ponytail rung

**Section:** §6, Phase 3 ("Override mechanism... Only new engine work in the
plan"); also §9's "Not authorized: Any Phase 3+ work" row and §13's decision
register.

**Quoted text:** "`.sergeant/local/workflows/` resolution — local shadows
stock by name. `sgt doctor` stock-drift and edition-drift detection. `sgt
workflow fork <name>` — copy stock to local with provenance. Embedding: stock
templates compiled into the binary, written by `sgt init`."

**Invariant violated:** `docs/DEVELOPMENT.md`'s rung-logging convention —
"Design decisions log their Ponytail rung (R1–R7 ladder in
`reference/notes/ideaos-agent-contract.md`)" — and that document's own text:
"every design decision in a ledger entry, every deviation-register row, and
every new dependency, file, trait, or store records the rung it resolved at
(R1–R7). An R7 entry must name which lower rungs were checked and why they
failed."

**Why it matters:** Phase 3 introduces four new pieces of engine surface — a
shadow-resolution mechanism, a `doctor` drift detector, a new `sgt workflow
fork` verb, and a binary-embedding pipeline — and the contract for this axis
asks explicitly whether it "sits on its lowest viable rung." The proposal
never says. No R1–R6 rejection is argued anywhere for any of the four items,
and none is logged as R7 with the required "which lower rungs were checked
and why they failed" justification. This is not a paperwork nit: §4.3 of this
same proposal is the one *inlining the Ponytail ladder into `AGENTS.md`* as
always-on doctrine, and §9 treats the ladder's own reachability as "the
recursion proof." A plan that elevates the ladder to Layer-0 status while
exempting its own only new engine work from the ladder's logging discipline
is internally inconsistent in exactly the way the ladder exists to prevent —
the jump from "I understand the requirement" straight to a new mechanism,
un-audited.

### 2. [warning] §4.7's edition marker, §3.3's retrieval partitioning, and Phase 5's validator are each ungraded against the ladder

**Sections:** §4.7 ("Templates carry an edition marker"), §3.3 ("Scoped
retrieval, not flat search"), §6 Phase 5 ("Validator (Gate F)").

**Quoted text:** §4.7 — "Each shipped template declares the distro version it
was forked from... there is no `workflow diff` verb." §3.3 — "Partition by
durable operator axis — host, repo, workflow, milestone — before any semantic
search... Host-environment facts get a hard-scoped partition of their own."
Phase 5 — "Extract every `sgt <verb> --<flag>` claim from doctrine; diff
against the built binary's actual `--help` surface."

**Invariant violated:** same rung-logging convention as Finding 1.

**Why it matters:** the contract asks whether each of these three is
"minimum-sufficient... or reaching past R6." That question cannot be answered
from the proposal text as written, because none of the three states a rung.
§4.7 plausibly *is* R6 (a one-line frontmatter field) — but §11 open question
2 leaves "frontmatter field, or a generated manifest?" unresolved, and a
generated manifest is not obviously R6. §3.3's partition-by-axis retrieval
model is new indexing architecture with no stated floor at all — it could be
R5 (an installed dependency, e.g. an existing embedding/index library) or a
bespoke R7 build; the proposal doesn't say which, so a reader cannot tell
whether cheaper options were considered. Phase 5's cross-repo `--help`-vs-doctrine
diff is workflow content (reuse of the existing workflow engine, arguably
R2), which is the least concerning of the three, but even that goes
unstated. Absent rung citations, "does it sit on its lowest viable rung" is
unanswerable, and the burden this axis was asked to discharge falls back on
the panel's guesswork rather than the proposal's own argument.

### 3. [note] §12's non-goals list claims restraint but does not test restraint where the ladder actually bites

**Section:** §12 ("Non-goals").

**Quoted text:** "No native Windows. No crates.io, Homebrew, or
package-manager publication. No self-update. No nightly CI. No automatic
releases from tags... No central mission authority, federated ownership
model, or organizational knowledge system. No `sgt workflow diff`. No OpenSSF
score chasing. No SLSA level claimed before it is verified."

**Invariant/rung violated:** the Ponytail ladder's own framing (minimality is
demonstrated per-decision, not asserted in aggregate) — not a hard rule
violation, hence "note," but the contract's instruction is explicit:
"§12 claims restraint via a non-goals list; grade that claim, do not accept
it."

**Why it matters:** every item on this list is either release/publishing
infrastructure the plan was never going to build (self-update, package-manager
publication, nightly CI) or a scope boundary already settled elsewhere (ADR
0014 decision 4 for `workflow diff`, BU-0109 for federated ownership). None of
them is the plan's own new engine surface — Phase 3, §3.3, §4.7, Phase 5 —
which is exactly the surface Finding 1 and 2 show is ungraded against R1–R6.
A non-goals list that is accurate about what won't be built is not the same
claim as "what is being built sits at its lowest viable rung," and §12 reads
as though it substitutes for the latter. The claim of restraint survives as
far as it goes; it does not go as far as the axis needs it to.

### 4. [warning] `sgt init`'s write path for the embedded distro is unaddressed against ADR 0008's manifest-authority ruling

**Section:** §6, Phase 3 ("Embedding: stock templates compiled into the
binary, written by `sgt init`"); also §4.7.

**Quoted text:** "Embedding: stock templates compiled into the binary,
written by `sgt init`."

**Invariant violated:** ADR 0008's ruling that "the manifest should be
authority for both or neither" regarding storage-path decisions the engine
makes on an estate's behalf — a principle it applied to add `[estate]
data_dir` for symmetry with `surfaces_dir` precisely because `resolve_data_dir`
walked up to find the manifest and then "discard[ed] it, hardcoding
`DEFAULT_ESTATE_DATA_DIR`."

**Why it matters:** Phase 3 introduces a materially identical shape — a new
disk-write the engine performs at `sgt init` time (writing embedded templates
into the estate) — without saying whether the destination path is
manifest-declarable or hardcoded. ADR 0008 exists precisely because this
repository has already been burned once by an engine write-path decision made
without manifest symmetry in mind. The proposal's Phase 3 bullet and §4.7
never mention `sergeant.toml`, `data_dir`, or `surfaces_dir` at all, so
there's no way to tell from the text whether this decision was made with ADR
0008's precedent in view or simply not considered. Given §11 open question 2
is already tracking edition-marker *format* as unresolved, the write-path
question deserves the same explicit "open decision" treatment rather than
silence.

### 5. [warning] GAUNTLET.md and LESSONS.md — DEVELOPMENT.md's own named development-record files — are never placed in either repository or substrate

**Sections:** §3.1 (the two repositories' file lists), §3.2 (the three
knowledge substrates), §6 Phase 4 ("`sergeant-rs` sheds `docs/gauntlet/`,
`reference/`, `reference-corpus/`, `resources/`").

**Quoted text:** §3.1's product-repo list — "`src/`, `tests/`, `scripts/`,
`Cargo.*`, `AGENTS.md`, `skills/`, `.sergeant/workflows/` (templates),
`docs/adr/`, `NORTH-STAR.md`, `docs/DEVELOPMENT.md`, `docs/environments/`,
`docs/glossary.md`, `SECURITY.md`, `CONTRIBUTING.md`, `CHANGELOG.md`." Phase
4's shed list — "`sergeant-rs` sheds `docs/gauntlet/`, `reference/`,
`reference-corpus/`, `resources/`."

**Invariant violated:** `docs/DEVELOPMENT.md`'s "The development record" —
which names `GAUNTLET.md` ("append-only ledger: the deviation register,
backlog rows with named triggers, per-milestone scorecards and adjudication
rulings... Append; never rewrite history") and `LESSONS.md` ("binding on
development here") as the two governing records a session must read before
changing method or scope.

**Why it matters:** neither file appears anywhere in the proposal — not in
either repository's file list in §3.1, not in the three-substrate table in
§3.2, and not in Phase 4's explicit shed list (verified by direct search of
the proposal text: zero matches for either filename). By §3.2's own
definitions both are unambiguously Evidence-substrate material (append-only,
never edited, superseded-is-marked) and should move to the workspace with
`docs/gauntlet/runs/` and the rest — but the proposal never says so. This
matters more than an ordinary omission because these are exactly the two
files `docs/DEVELOPMENT.md` singles out as the ones a session must consult
before it may change method or scope on this repository, and the plan is
proposing to change the repository's own topology without saying where the
record that governs such changes will live afterward.

## Summary

Five findings: one error (Phase 3's un-rung new engine work), three warnings
(the three named additions ungraded against the ladder; the ADR 0008
write-path silence; the missing placement of GAUNTLET.md/LESSONS.md), one
note (§12's non-goals list answering a narrower claim than the axis asks
for). Three hazards named in the contract were tested directly and cleared:
ADR 0002 platform-boundary collision, ADR 0008 direct collision (as opposed
to the silence in Finding 4), and the `repos/`-as-mount question.

No finding here invalidates the proposal's architecture. All five are
gaps in the proposal's own stated discipline (rung logging, manifest
symmetry, record placement) rather than contradictions of a settled
invariant.
