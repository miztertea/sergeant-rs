# T-SERIES-1 — enactability critic

Axis: can each section actually be executed as dispatched work? Graded
against `reference/proposal-tui-t-series.md` §1–§25, checked against the
repository at `242abe3c4a889c2b666c7ce34b32812dd1ee8d61` (`Cargo.toml`/
`Cargo.lock`, `tests/m6_surfaces.rs`, `src/backend/fake.rs`,
`reference/notes/gauntlet-pattern.md`, `.sergeant/workflows/`), live crate
resolution (`cargo add --dry-run`, `cargo fetch`) against the vendored
`ratatui-0.30.2` and `crossterm-0.29.0` sources, GitHub issue #120, and
`docs/gauntlet/contracts/T-SERIES-1.md`. Special attention to §20's T0–T4
program shape and §21's acceptance contract (can T0 be dispatched as the
first `sgt run` without further judgment calls?), and to §8/§19 for claims
about what `ratatui`/`crossterm` can render or what a pure-state test can
assert without a live daemon.

Non-goals observed: no finding argues an absent implementation ("this
isn't built yet" is the whole premise of a proposal), re-litigates the
North Star gate amendment, proposes a different build than the one the
proposal states, or extends scope into an adjacent problem the proposal
doesn't claim to solve.

Severity key: **error** = a dispatched Work would stall or produce an
unreviewable guess; **warning** = a Work could proceed but would have to
invent something the proposal should have supplied; **info** = worth
recording, doesn't block dispatch.

---

## Finding 1 — §20.1 T0 says "No product code" while also requiring a dependency-resolution spike that cannot be proven without writing and compiling product code

**Severity:** error
**Section:** §20.1, cross-checked against §8.7

**The claim:** §20.1 lists T0's tasks, including "spike `ratatui-textarea`
dependency resolution," and closes the section with: "No product code."

**What I checked:** §8.7 defines exactly what the spike must prove: "one
resolved Ratatui version, one resolved Crossterm version, no direct
conflicting crossterm edge, no search/regex feature, no mouse requirement,
no editor-owned submit behavior, pure access to the local draft for
testing." I reproduced the check myself: `cargo tree -i crossterm@0.28.1`
and `cargo tree -i crossterm@0.29.0` against the current lockfile show the
repository already carries two crossterm versions side by side (0.28.1 via
`duckdb → comfy-table`, 0.29.0 via `ratatui-crossterm → ratatui`).
`cargo add ratatui-textarea --dry-run` resolves cleanly against the
current tree, and a full scratch-crate `cargo add ratatui@0.30.2
ratatui-textarea` + `cargo fetch` locks to a *single* crossterm 0.29.0.
Producing that evidence — a resolved version graph with "no direct
conflicting crossterm edge" — requires actually touching `Cargo.toml`
and/or running `cargo add`/`cargo tree`/`cargo build` against the real
dependency graph, which is a change to and execution of product build
files, not a design artifact.

**What I found:** §20.1's own two instructions are mutually exclusive as
written. A Work dispatched to "do T0" cannot both honor "No product code"
literally (skip the spike, in which case §8.7's admission gate for
`ratatui-textarea` is never actually cleared, and T1 inherits an unresolved
dependency question the proposal treats as settled by T0) and complete the
spike as §8.7 defines it (which requires a `Cargo.toml` edit and a build,
i.e. product code, even if the change is later reverted). Nothing in §20.1
or §8.7 says the spike's `Cargo.toml` edit is exempt from "no product
code," and nothing names which of the two instructions yields when they
conflict.

**Survives the correction?** Not as written. The fix is cheap — scope "no
product code" to mean "no application/UI code" and explicitly except the
dependency-resolution spike, or move the spike itself later once T0's
scoping is settled — but as it stands a T0 Work has no way to produce an
honest artifact without either skipping a task §20.1 lists for it or
violating the constraint §20.1 states for it.

---

## Finding 2 — §20.1's first T0 task names a process a dispatched Work cannot itself perform

**Severity:** error
**Section:** §20.1, cross-checked against `reference/notes/gauntlet-pattern.md` and `docs/gauntlet/contracts/T-SERIES-1.md`

**The claim:** §20.1 lists T0's first task as: "review this proposal
through the repository's proposal gauntlet."

**What I checked:** `reference/notes/gauntlet-pattern.md` describes what
"the repository's proposal gauntlet" actually is: a loop of
`CONTRACT → BUILD → GATES → BLIND CRITICS → ADVERSARIAL VERIFY → FIX →
CHECKPOINT GATE → ADJUDICATE → MARK & LOG`, run by "the orchestrator (a
Fable 5 session)" dispatching builder and critic subagents through
ultracode workflows — not a task a single `sgt run` Work performs from
inside itself. `docs/gauntlet/contracts/T-SERIES-1.md` confirms this
proposal is already inside exactly that process right now, and states
plainly: "Build against the graded proposal is a separate, later contract
(§20's T0–T4 program shape, sequenced only after this unit closes)." I
checked `.sergeant/workflows/` for a workflow that encodes "run the
proposal gauntlet" as something a Work could dispatch against — none of
the 23 admitted workflows (`code-review`, `research`, `implement`, `tdd`,
`wayfinder`, etc.) is that process; the gauntlet loop runs at the
orchestrator level, above individual dispatched Works.

**What I found:** As a literal T0 action item, this bullet has no
dispatchable referent. Either it describes something that, by the
contract's own text, must already be true before T0 can exist at all (in
which case it is dead, redundant prose inside T0 rather than a task T0
performs), or it is read as an instruction for the T0 Work itself to run a
blind-critic gauntlet on its own proposal — which is not a shape a single
dispatched Work can execute; that shape requires the orchestrating session,
multiple fresh-context subagents, and a capped multi-round adjudication
loop. A Work executing "T0" has no defined action to take for this line.

**Survives the correction?** Not as written. Naming this bullet as a
precondition ("T0 begins only once a proposal gauntlet unit like this one
has closed") rather than a task T0 itself performs would resolve it without
changing anything else in §20.

---

## Finding 3 — the T0–T4 program never assigns four of the five canonical Work tabs to a phase

**Severity:** error
**Section:** §20.2–§20.5, cross-checked against §13.2–§13.7 and §21 items 13, 24–26

**The claim:** §13.2 (Decision T2-48) requires "one canonical full-body
Work surface" with views `Thread Workflow Evidence Graph Details`. §21
holds this as binding acceptance: item 13 ("Work has Thread, Workflow,
Evidence, Graph, Details"), item 24 ("Workflow progression is ordinal
only" — the rail defined in §13.4), item 25 (Graph, §13.6), item 26
(Evidence, §13.5).

**What I checked:** Read every bullet in §20.2 (T1) through §20.5 (T4)
looking for the four tab names or their describing nouns ("stage rail",
"journal window"/Evidence, "relationship tree"/Graph, "progressive
disclosure"/Details). T1 names "canonical Work shell", "transcript-backed
Thread", and "output/envelope/completed-dirty" (covering §13.8/§13.9, not
a separate tab) as its Work-surface line items. T2 covers the top-level
Workflows *catalog* screen (§11) and Home's `@` chooser — a different
surface from the per-Work Workflow rail in §13.4. T3 covers Estate/Doctor
extraction. T4 is explicitly "close-out and polish" (fixtures, pre-flight,
screenshots, docs, ledger, handoff) — a phase whose own bullets read as
finishing already-built surfaces, not building new ones.

**What I found:** None of T1–T4's bullets name the Workflow rail,
Evidence, Graph, or Details tabs. This is ambiguous in a way that is
itself the defect: either they are silently meant to be bundled into T1's
"canonical Work shell" (in which case that phrase is confidently vague
about four separate, non-trivial deliverables — each with its own
rendering rules in §13.4–§13.7), or they are genuinely unassigned, in
which case §21 items 13/24/25/26 have no phase whose "done" bullet
delivers them. The backing data for all four already exists in the
shipped API today (§13.4–§13.7 each cite an "existing" or already-shipped
read: the Work read model, `GET /v1/events`, and "the existing one-Work
graph"), so bundling them into T1 is the more plausible reading and the
gap is closeable without new sequencing — but the proposal doesn't say
so, and "transcript-backed Thread" being called out as its own T1 bullet
right next to the vaguer "canonical Work shell" argues against treating
the two as synonymous.

**Survives the correction?** Yes, cheaply — naming the four tabs
explicitly under T1 (since none of them need new endpoints, unlike
Workflows/Estate) closes the gap without changing what any Work builds.
As written, a Work that completed T1–T4 exactly as scoped could pass its
own phase checklists while failing acceptance items 13/24–26, and nothing
in the program would have caught it earlier than final acceptance review.

---

## Finding 4 — §20.1's "validate workflow catalog route" is ambiguous between a paperwork task and a coding task, and either reading conflicts with something else in the proposal

**Severity:** warning
**Section:** §20.1, cross-checked against §11.2 and §20.3

**The claim:** §20.1 lists "validate workflow catalog route" as a T0 task,
under the same "No product code" close that governs Finding 1.

**What I checked:** §11.2 describes the workflow catalog route
(`GET /v1/workflows?cwd=...`) as not yet built: "The exact response is
contracted before implementation." §20.3 lists "workflow catalog endpoint"
as a T2 deliverable, not a T0 one.

**What I found:** "Validate ... route" reads naturally as exercising a real
HTTP route — request it, check the response — which requires the route to
exist, which requires writing it, which is product code T0 disclaims and
duplicates T2's stated deliverable. The alternative reading — "validate"
means finalizing/reviewing the JSON contract described in §11.2 without
implementing it — is compatible with "no product code" and with §11.2's own
"contracted before implementation" framing, but the proposal never says
which reading is intended, and "validate" is a stronger verb than "contract"
or "specify" would have been. A Work executing T0 has to guess which of the
two it's being asked to do, and guessing wrong either produces code T0
forbids or produces nothing checkable against the word "validate."

**Survives the correction?** Yes — replacing "validate" with "contract" (or
explicitly stating "no implementation, schema only") would align this bullet
with §11.2's own language and remove the ambiguity without changing scope.

---

## Finding 5 — Acceptance item §21.57 depends on a live, open, unowned gate-defect fix that no phase of §20 sequences

**Severity:** warning
**Section:** §19.12 and §21 item 57, cross-checked against §3.3 and §20

**The claim:** §21 item 57: "The shipping gate actually executes and
passes; a skipped false-green is failure." §19.12: "The gate defect in PR
#111 must be resolved before its result is trusted." §3.3: "the shipping
gate produced false passed verdicts and #120 remains open."

**What I checked:** `gh issue view 120` — confirmed **still open**, filed
this session, title: "no-mistakes resolves its diff base against a live
non-bare origin checkout, silently short-circuiting on empty diff." The
reproduction is concrete and current: when a dispatched Work's branch is
cut from a tip that produces an empty diff against the auto-detected base,
the gate takes an "empty diff after rebase, skipping remaining steps" path
straight to `outcome: passed`, skipping review/test/document/lint
entirely — exactly the "skipped false-green" §21 item 57 names, and #120's
own text confirms there is no CLI flag to pin an explicit diff base as a
workaround. I then searched all of §20 (T0 through T4) and the rest of the
proposal for any task that fixes the shipping-gate defect or names an
owner/timeline for #120's resolution: none of T0's re-audit/spike/contract
tasks, T1–T3's feature work, or T4's "close-out and polish" list include
gate-defect remediation; §6.1/§6.2 don't scope it in or out either.

**What I found:** Item 57 is self-aware — it explicitly anticipates and
rejects a false-green outcome rather than accepting whatever the gate
reports — but the proposal names no mechanism by which a T-Series Work
would detect a false green other than the gate itself reporting honestly,
which is precisely what #120 says it currently does not do under a
reproducible, still-open condition. A Work executing T4 close-out has no
operational path to satisfy item 57 other than an out-of-band manual check
the proposal never describes, and no T-phase is responsible for landing
the fix (or confirming it landed) before close-out.

**Survives the correction?** Yes — this doesn't block T0–T3, and it's
plausible T-Series's owner expects #120 to be fixed by unrelated infra
work before T4 closes. But as scoped, item 57 is a real acceptance
criterion with no assigned owner and a known-live counterexample; naming
either "T4 cannot close until #120 is independently resolved" or "T4
manually verifies a non-empty diff before trusting `passed`" would close
the gap.

---

## Finding 6 — the PR #111 disposition is pinned once at T0 but the program runs through T4, with no named re-check point

**Severity:** warning
**Section:** §12.4 and Decision T2-06 (§3.3), cross-checked against §20

**The claim:** Decision T2-06: "T0 pins the actual implementation base
after PR #111 is either merged or explicitly excluded. No T-Series screen
may claim an integration-only fact before that disposition." §12.4 gates
the entire retained/reap surface on "If the integration branch's
retained/reap surfaces merge."

**What I checked:** §20's T0–T4 phases are sequential, multi-slice work
("T3: Estate ... conditional retained/reap consumption"); nothing in §20
schedules a second check of PR #111's merge status between the T0 pin and
T3/T4, when the retained/reap UI is actually built and when acceptance
items 54–55 are checked.

**What I found:** If T0 pins the base with PR #111 "explicitly excluded"
(the state the proposal itself describes at audit time — "not ready to
merge"), and the PR merges later, during T1–T3, the proposal gives T3's
Work no instruction to re-open that pin and build the conditional surfaces
after all; conversely if T0 pins "merged" and something causes the branch
to be reverted before T3, there's equally no named re-check. The one-time
T0 pin and the multi-phase program duration are not reconciled.

**Survives the correction?** Yes — naming a re-check point (e.g., "T3
re-verifies PR #111's disposition before building §12.4" or "the T0 pin is
binding for the whole program; a changed PR #111 status requires a new
T-series decision, not a silent T3 re-check") would close the gap without
otherwise changing the section.

---

## Finding 7 — §19's test list and §21's acceptance list are both flat and undifferentiated by T-phase, leaving each phase's Work to infer its own subset of "done"

**Severity:** info
**Section:** §19 (all subsections) and §21, cross-checked against §20

**The claim:** §19 lists testing obligations (pure-state, composer,
catalog, Estate parity, Doctor parity, live-daemon, integration-conditional,
geometry) and §21 lists 58 acceptance items, neither annotated with which
T0–T4 phase they apply to.

**What I checked:** Cross-referenced items against §20's phase deliverables
— most items map cleanly by content (e.g. §21.29–31 clearly belong to T2's
"workflow catalog endpoint" work; §21.32–37 clearly belong to T3's Estate
work) but the mapping is inferred by the reader, not stated. §19.2's "pure
state tests" bullet "workflow live-versus-pinned labeling" specifically
requires the catalog endpoint that §20.3 assigns to T2, meaning a T1 Work
attempting the full §19.2 list as written would hit a test it cannot pass
yet.

**What I found:** This doesn't block dispatch of any individual phase — the
content-based mapping is inferable, and FOUNDATION-1 established that this
kind of legwork is a known, acceptable cost of a design document, not a
contradiction. It's recorded because a Work executing T1 in isolation,
handed §19/§21 verbatim as "the tests/acceptance for this phase," would
have to first work out for itself which subset is actually in scope yet —
exactly the kind of self-supplied judgment call the enactability axis is
meant to catch when it isn't named.

**Survives the correction?** Yes — tagging each §19/§21 item with its owning
T-phase (or adding one line per T0–T4 subsection in §20 pointing at its
relevant §19/§21 items) would remove the inference step entirely.

---

## Finding 8 — §8.7's T0 spike checklist omits the one check that actually gates the dependency

**Severity:** info
**Section:** §8.7, cross-checked against `tests/m6_surfaces.rs:2164`

**The claim:** §8.7 lists seven conditions a T0 spike must prove before
`ratatui-textarea` is admitted: one resolved Ratatui version, one resolved
Crossterm version, no direct conflicting crossterm edge, no search/regex
feature, no mouse requirement, no editor-owned submit behavior, pure
access to the local draft for testing.

**What I checked:** Whether these seven conditions are actually
sufficient, by reproducing the dependency addition and reading source, not
just resolving versions. `cargo add ratatui@0.30.2 ratatui-textarea` in a
scratch crate resolves to a **single** crossterm 0.29.0 in `Cargo.lock`
(no duplicate edge). I then fetched and read the vendored
`ratatui-textarea-0.9.2` source directly: its `crossterm` feature
(`src/input/crossterm.rs`) is implemented against
`ratatui_crossterm::crossterm::event::{KeyEvent, ...}` — the same
re-exported path `ratatui::crossterm` uses (`ratatui-0.30.2/src/lib.rs:483`:
`pub use ratatui_crossterm::crossterm;`) — not the bare `crossterm` crate.
Separately, this repository already pins that exact constraint as a test:
`tests/m6_surfaces.rs:2164`,
`the_tui_stack_is_ratatui_with_crossterm_reached_through_it`, which fails
the build if `crossterm` becomes a direct `Cargo.toml` dependency or if
`src/tui.rs` contains a bare `use crossterm` line.

**What I found:** The dependency genuinely clears the bar — this is the
one place I went looking for an unbuildable §8/§19 claim and didn't find
one. But §8.7's own checklist never mentions the repository's actual
gating mechanism (the pinned source-scan test), only generic
dependency-resolution properties. A Work could satisfy all seven listed
conditions and still be surprised by
`the_tui_stack_is_ratatui_with_crossterm_reached_through_it` failing for an
unrelated reason (e.g. a future `ratatui-textarea` release adding a direct
`crossterm` re-export), because the spike's own success criteria don't
name the test that actually decides admission.

**Survives the correction?** Yes — trivially. Adding "the pinned
`the_tui_stack_is_ratatui_with_crossterm_reached_through_it` test still
passes" as an eighth spike condition would close this without changing the
technical outcome (which, as verified, already holds).

---

## What I checked and found nothing on

- §8.3's claim that Ratatui 0.30.2 already supplies `Tabs`, `Table`,
  `List`, `Paragraph`, `Scrollbar`, `Block`, `Clear`, `Gauge`/`LineGauge`,
  and styled `Span`/`Line`/`Text`: `Cargo.lock` confirms `ratatui 0.30.2`
  is the resolved version and the vendored source contains every widget
  named; these are genuinely built-in, not aspirational. Not a finding.
- §8.4's claim that every state glyph "is tested for one-cell width":
  `unicode-width 0.2.2` is already resolved in `Cargo.lock` (pulled
  transitively), so this is checkable with an already-available crate, not
  a dependency that needs to be invented. Not a finding.
- §8.7's crossterm-conflict concern itself: verified directly (Findings 1
  and 8) that `ratatui-textarea` resolves cleanly against the current
  lockfile to a single crossterm version, reached through the same
  `ratatui::crossterm` re-export path the repository's pinned test already
  enforces. The *substance* of the spike's admission criteria is
  satisfiable; only the "no product code" framing around it (Finding 1)
  and the checklist's incompleteness (Finding 8) are the problems.
- §19.7's "Live daemon tests ... Using fake backend": grepped
  `src/backend/fake.rs` and `tests/m7_docker_executor.rs`,
  `tests/m8_estate_cli.rs`, `tests/m9_watch.rs`, `src/runtime/router.rs` —
  `FakeBackend` is an established, already-used pattern in this
  repository's integration tests (a real daemon process with a scripted
  fake actor backend), not a fictional harness the proposal invents. Not a
  finding.
- §19.1/§19.10's claim that Ratatui `TestBackend` supports geometry testing
  at arbitrary sizes (80x24, 120x36, 180x48) with buffer assertions:
  matches how `tests/m6_surfaces.rs` already exercises the current TUI —
  a real, documented `TestBackend` capability, and §19.10's fixture list
  gives each geometry a concrete set of screens to render. Not a finding.
- §8.8's Kitty keyboard protocol claims (`KeyEvent`,
  `PushKeyboardEnhancementFlags`/`PopKeyboardEnhancementFlags`,
  `KeyboardEnhancementFlags`, "nonfatal" failure): all present in the
  vendored `crossterm-0.29.0` source at the cited paths; the "integrated
  into the existing terminal lifecycle guard" claim has a concrete
  existing target (§17.6's list of guarantees already shipped) to extend
  rather than invent from nothing. Not a finding.
- §6.1/§6.2's in-scope/non-goal lists against §20's phases: every in-scope
  item traces to a specific T1–T3 deliverable and every non-goal is a
  negative (nothing to dispatch), so neither list itself hides an
  undecided question. Not a finding.
- §12.4 and the other PR #111-conditional sections (§10.5, §19.8): each
  states a clean either/or ("if it lands... if it does not...") with a
  named fallback (omit the control, no placeholder) rather than hiding the
  branch — enactable regardless of the PR's actual disposition, which is
  the assumptions axis's question, not this one's (the *re-check timing*
  gap across that same conditional is Finding 6).
