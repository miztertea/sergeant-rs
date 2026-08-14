# FOUNDATION-1 — blind critic: invariants

Axis: does any section of `reference/proposal-foundation-rationalization.md`
§1–§8 violate a principle this project has already committed to
(`NORTH-STAR.md`'s ownership boundaries, Never list, R-NS-1..6;
`docs/DEVELOPMENT.md`'s architecture invariants; ADRs 0001–0011; the
Ponytail Minimality Ladder)? Grading the proposal text itself, and grading
§4's seven preservation claims rather than accepting them, per
`docs/gauntlet/contracts/FOUNDATION-1.md`.

Context read: `NORTH-STAR.md`, `docs/DEVELOPMENT.md`,
`reference/notes/ideaos-agent-contract.md`, `LESSONS.md`, `AGENTS.md`,
`docs/adr/0005`–`0011`, `docs/gauntlet/notes/north-star-draft-2026-08-11.md`
(source of R-NS-1..5, which `NORTH-STAR.md` itself only cites as "as
drafted"), and `src/runtime/surface.rs` (the `WorkSurface` mechanism §4.2
and §5.1 invoke by name).

Three findings. All land on §4's preservation claims specifically, per the
contract's instruction to grade the claim rather than accept it. No
finding invokes an implementation that doesn't exist, and none
re-litigates a ruling — each is about whether the proposal's own stated
argument for preservation is actually load-bearing.

---

## Finding 1: `inv-one-owner-relocated`

**Severity:** error

**Section:** §4.2 ("One owner"), §4.6 ("Procedure is data"), §5.1 ("Gating
becomes a dispatched Work")

**Claim at issue:** §4.2, verbatim: "The daemon exclusively owns the data
dir. §5.1 does not weaken this — it replaces one ownership mechanism
(no-mistakes' repo-wide branch lock) with one sergeant already enforces (a
Work owns its surface)." §4.6, verbatim: "§5.1 moves gating into a
workflow package rather than a script, which strengthens this rather than
bending it." §5.1 itself: "The gate becomes a Work with its own surface."

**What I checked:** `docs/DEVELOPMENT.md`'s actual "One owner" invariant
("The daemon ... exclusively owns the data dir ... and all process
handles") and §3.1's own description of what no-mistakes actually locks
("`scripts/gate.sh` wraps no-mistakes, which takes ownership of *the
branch* for the duration of a run" — confirmed against
`docs/DEVELOPMENT.md`'s Shipping gate section: "While a run is active the
branch is pipeline-owned: don't commit locally until the run reaches an
outcome, then `no-mistakes axi respond` on gates and `no-mistakes axi sync
--recover` to take custody of pipeline commits"). Then I checked what "a
Work owns its surface" actually means mechanically, in
`src/runtime/surface.rs`: a `WorkSurface` is "one git worktree per
targeted repository, on its own branch, materialized under the daemon's
data dir" — the module docstring gives the concrete shape,
`<data-dir>/surfaces/<work-id>/<repo>/` checked out on branch
`sergeant/<work-id>`. `add_worktree`/`add_worktree_from`
(`src/runtime/surface.rs:642,660`) create that branch fresh via `git
worktree add -b <branch> <path> <start>`; `teardown` (line 491) "retain[s]
every branch" while removing the worktree; `rematerialize` (line 417)
re-attaches a worktree "to a surface's *retained* branches" — always the
same `sergeant/<work-id>` branch the surface was born with. There is no
code path by which a `WorkSurface` binds to a branch it did not itself
mint from a `work-id`.

**What I found:** These are two different ownership scopes, and the
proposal's own wording in §5.1 ("a Work *with its own surface*") confirms
which one it means. No-mistakes' branch lock is exclusive access to *the
branch already produced by the Work being shipped* — the artifact under
review. Sergeant's existing "a Work owns its surface" guarantee is
exclusive access to *a freshly-minted `sergeant/<work-id>` branch the gate
Work itself creates* — a different branch, by construction, from the one
it would need to review. For the gate Work to actually gate the target
branch under sergeant's existing surface semantics, one of two things has
to be true, and the proposal states neither: (a) the gate Work's surface
is somehow bound to the *actual* target branch rather than a fresh
`sergeant/<gate-work-id>` branch — which requires the target Work's own
worktree to already be torn down (git refuses two worktrees on one
branch), a sequencing constraint the proposal never states and that
constrains when in a Work's lifecycle gating can even be dispatched; or
(b) the gate Work reviews a *copy* (a new branch cut from the target's
tip), which breaks no-mistakes' own assumption that its auto-fix commits
land on the branch being shipped and are later recovered with `axi sync
--recover` onto that same branch — an extra reconciliation step the
proposal does not mention. Either way, "a Work owns its surface" does not
by itself establish what §4.2 and §4.6 claim it establishes; it is a true
fact about a different resource. This is not a corner I am inventing:
ADR 0005's own "Open questions" section says so directly — "The mechanics
of how a gate Work is specified — what workflow stage invokes
`scripts/gate.sh`/no-mistakes ... whether it is a new named ICM workflow
or a stage folded into an existing one — are not decided here." §4.2 and
§4.6 assert the preservation as settled fact; the ADR the proposal itself
cites as the ruling admits the mechanism is not decided.

**Does the section survive the correction?** Yes, but not as currently
worded. §4.2 and §4.6 should say what §8 already models honestly
elsewhere in this same document: that the *intent* is to fold gate
ownership into the existing surface model, and that the binding mechanism
— and its interaction with worktree-per-branch exclusivity — is open,
tracked the same way ADR 0005's own open questions track it. As written,
these two entries assert a mechanism that the proposal's own cited ADR
says is undecided; that is exactly the failure mode the contract asks
this axis to catch ("a section 4 entry that asserts preservation without
the mechanism that preserves it is a finding").

---

## Finding 2: `inv-section4-cites-unsettled-mechanisms`

**Severity:** warning

**Section:** §4.3 ("Ambiguity fails closed"), §4.4 ("A surface adds
usability, never functionality"), cross-referencing §5.3, §5.6, §8.3

**Claim at issue:** §4.3: "§5.3's safety net converts a silent success
into an honest non-terminal state." §4.4: "§5.6's homepage reads a
manifest and contacts nothing."

**What I checked:** Whether these are the actual, ruled mechanisms or the
proposal's own invented specifics. ADR 0007 (§5.3's ruling), Open
questions: "The exact detection logic for (b) ... and what state a
closing stage should report instead of plain `completed` is not specified
here." ADR 0010 (§5.6's ruling), Open questions, and the proposal's own
§5.6 text: "**Open, not decided:** whether the homepage is estate-aware
(reading `sergeant.toml` is not observing the daemon) or a static
banner" — restated again at §8.3: "The homepage's estate-awareness. §5.6,
unruled."

**What I found:** Both §4 entries state a specific implementation detail
as accomplished fact where the proposal's own later sections (and the
ADRs those sections summarize) mark the same detail explicitly open. §4.3
names the replacement state as "non-terminal" — but ADR 0007 says the
replacement state is not specified, only that it must not be plain
`completed`; a terminal-but-flagged state would equally satisfy ADR 0007's
ruling and would not be "non-terminal." §4.4 states the homepage "reads a
manifest" as a description of what it does — but §5.6 and §8.3, three and
seven paragraphs later in the same document, say this is a live option,
not a decision, and the other option ("a static banner") reads no
manifest at all. Neither slip changes whether the underlying invariant
(no second source of truth; usability not functionality) actually holds
— under either resolution, the closing-stage guard stays a
journal-recorded state and the homepage stays read-only and contacts
nothing — so this is not a violation of the invariant itself. It is §4
citing a specific mechanism as the *reason* an invariant holds, when that
mechanism is explicitly unruled elsewhere in the same proposal. Per the
contract's instruction to grade the claim rather than accept it, an
argument that names a mechanism it does not actually have standing to
name is a defect in the argument, independent of whether the conclusion
happens to be safe either way.

**Does the section survive the correction?** Yes, trivially — rephrase
both entries to state the invariant holds *regardless* of which unruled
option is eventually chosen (which is true and checkable), rather than
picking one unruled option and describing it as settled.

---

## Finding 3: `inv-ladder-incomplete-and-5.7-mislabeled`

**Severity:** warning

**Section:** §4.7 ("The Ponytail minimality ladder"), §5.7 ("The
dashboard is deleted")

**Claim at issue:** §4.7 in full: "Each change should sit on its lowest
viable rung. §5.7 is a deletion — the lowest rung available. §5.2 is an
`exec`, not a supervisor. §5.1 reuses `validate-and-ship` rather than
authoring a new procedure."

**What I checked:** `reference/notes/ideaos-agent-contract.md`'s rung
table (R1–R7) and its "Rung logging convention (this repo)": "every design
decision in a ledger entry, every deviation-register row, and every new
dependency, file, trait, or store records the rung it resolved at (R1–R7).
An `R7` entry must name which lower rungs were checked and why they
failed." I then checked how many of the proposal's seven changes §4.7
actually rung-logs, and re-read ADR 0011's own "Alternatives considered"
for §5.7 to see what "lowest rung" is being compared against.

**What I found:** Two related gaps.

First, §4.7 rung-logs only three of the seven changes (§5.1 as R2 reuse,
§5.2 as exec-not-supervisor, §5.7 as deletion). §5.3 (a new
context-composition addition plus a new closing-stage guard), §5.4 (a new
`[estate] data_dir` manifest field), §5.5 (removing auto-spawn from five
more verbs), and §5.6 (a wholly new verb, `sgt tui`, plus a new homepage
renderer — genuinely new code, not a reuse or a one-liner) have no stated
rung anywhere in the document. §5.6 in particular is the shape the
convention's R7 clause exists for — new bespoke code — and the convention
requires an R7 entry to "name which lower rungs were checked and why they
failed"; §5.6 names none. This is the repo's own binding logging
convention, cited by `docs/DEVELOPMENT.md` itself ("Design decisions log
their Ponytail rung"), and four of seven changes are silent against it.

Second, "§5.7 is a deletion — the lowest rung available" measures the
wrong thing. The ladder's rungs (R1–R7) order how much new machinery a
change *builds* — the point, per the source document, is "block[ing] the
jump from 'I understand the requirement' to 'I should create a new
abstraction.'" Deletion does not appear on that ladder because it is not
an addition; there is no rung for "remove code." ADR 0011's own
"Alternatives considered" section shows what is actually being compared:
a full removal (779 + 224 lines, plus `tests/m6_surfaces.rs`'s
dashboard-specific tests, per its own "Consequences" section) against a
smaller change — disabling the route and leaving the code in place. The
proposal's stated reason for preferring the larger change is "a stub
carrying two open issues indefinitely is a maintenance claim, and deletion
commits to less than that stub would" (ADR 0011) — an argument about
future maintenance commitment, not about which change is smaller or
reuses more existing machinery, which is what "rung" measures everywhere
else in this document. Calling that choice "the lowest rung" borrows the
ladder's vocabulary for an argument the ladder does not actually make;
the argument for deletion is sound on its own terms (stated and reasoned
in ADR 0011), it just is not a ladder argument, and the diff it produces
is the larger of the two options actually on the table, not the smaller
one.

**Does the section survive the correction?** Yes — the four missing rung
citations are additions to the document, not corrections of anything
currently wrong, and §5.7's own stated reasoning (avoiding an indefinite
maintenance claim) stands without needing the ladder's vocabulary at all;
dropping "the lowest rung available" and keeping the rest of the sentence
costs nothing.

---

## What I did not find

Checked and clean, no finding filed:

- **§5.2 exec-vs-supervision.** `NORTH-STAR.md`'s Never list ("reconstructed
  tmux-era supervision") is the specific risk named by the contract for
  this section. §5.2's own text is explicit and mechanistic ("`sgt` does
  not own the harness process ... **execs** — replacing itself with the
  harness," "exec'ing means there is no lifecycle to own") and ADR 0006
  argues the same boundary on the same grounds. `exec` genuinely has no
  process table entry, pid file, or restart policy by construction — the
  claim is not a naked assertion, it names the mechanism (the syscall
  itself) that makes the Never-list violation structurally impossible for
  this specific design. No drift toward supervision language found
  anywhere in §5.2's or ADR 0006's text.
- **R-NS-1 (durability test), R-NS-2 (regeneration test), R-NS-3
  (no-second-home), R-NS-5 (estate opacity).** Checked each of the seven
  changes against all four. §5.1 keeps judgment (Captain's adjudication)
  out of the engine and execution (Sgt's pipeline) in it, consistent with
  R-NS-1. §5.4(c)'s re-ruling of #64 explicitly upholds "machine-local
  truth is in-estate," reinforcing R-NS-3 rather than testing it. Nothing
  in the seven changes touches AGENTS.md generation or catalog content
  (R-NS-2). No finding on any of these four.
- **§5.5's "fail closed, not merely fails differently."** Checked the
  substance, not just the label: previously, an observation verb on a
  cold estate auto-spawned a daemon (took an action with a side effect to
  satisfy an ambiguous/absent precondition); after §5.5, it refuses and
  names `sgt doctor` as the remedy (declines the side-effecting action and
  reports state honestly instead). That is a substantive match for the
  fail-closed pattern this repo already ships and names elsewhere
  (R-WATCH-3, "observation must not materialize the thing observed —
  fail-closed at both ends of the process's life," which §5.5 explicitly
  extends). §4.3's citation of this under `docs/DEVELOPMENT.md`'s
  differently-scoped "ambiguity fails closed" bullet (which is textually
  about restart-reconciliation ambiguity, not CLI auto-spawn) is an
  imprecise citation, not a substantive violation — I considered filing it
  as a fourth finding and did not, because the behavior itself is
  genuinely fail-closed under the rule it actually extends (R-WATCH-3),
  and flagging a citation mismatch where the underlying claim holds would
  be exactly the manufactured-finding failure mode the contract warns
  against.
- **§5.4's estate-first precedence and the `[estate] data_dir` addition**
  against "the journal is the only truth" and "one owner." A manifest
  field is estate-owned config, not daemon-owned durable state; adding one
  does not create a second source of truth for anything the daemon
  tracks. No finding.
- **Adjacent-append crash-window hazard (LESSONS L6).** §5.3(b)'s
  closing-stage guard and §5.1's gate-Work dispatch both *could* eventually
  introduce a two-append journal sequence of the shape L6 warns about, but
  nothing is implemented yet and the proposal states no journal mechanics
  for either — checking this would mean designing the implementation to
  find a defect in it, which is out of scope by the contract's own
  non-goals ("Not designing the implementations"). Not filed.
