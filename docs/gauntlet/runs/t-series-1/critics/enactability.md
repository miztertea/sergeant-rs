# T-SERIES-1 — enactability critic

Axis: can each section actually be executed as dispatched work? Graded
against `reference/proposal-tui-t-series.md` §1–§25, checked against the
repository as it stands (`Cargo.toml`/`Cargo.lock`, `tests/`,
`reference/notes/gauntlet-pattern.md`, `.sergeant/workflows/`) and
`docs/gauntlet/contracts/T-SERIES-1.md`. Special attention to §20's T0–T4
program shape and §21's acceptance contract (can T0 be dispatched as the
first `sgt run` without further judgment calls?), and to §8/§19 for claims
about what `ratatui`/`crossterm` can render or what a pure-state test can
assert without a live daemon.

Non-goals observed: no finding argues an absent implementation ("this isn't
built yet" is the whole premise of a proposal), re-litigates the North Star
gate amendment, proposes a different build than the one the proposal
states, or extends scope into an adjacent problem the proposal doesn't
claim to solve.

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
`cargo add ratatui-textarea --dry-run` resolves cleanly against the current
tree. Producing that evidence — a resolved version graph with "no direct
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

## Finding 3 — §20.1's "validate workflow catalog route" is ambiguous between a paperwork task and a coding task, and either reading conflicts with something else in the proposal

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

## Finding 4 — Acceptance item §21.57 depends on an external gate-defect fix that no phase of §20 owns or sequences

**Severity:** warning
**Section:** §21 item 57, cross-checked against §3.3, §19.12, and §20

**The claim:** §21 item 57: "The shipping gate actually executes and
passes; a skipped false-green is failure." §19.12: "The gate defect in PR
#111 must be resolved before its result is trusted." §3.3: "the shipping
gate produced false passed verdicts and #120 remains open."

**What I checked:** I searched all of §20 (T0 through T4) and the rest of
the proposal for any task that fixes the shipping-gate defect or names an
owner/timeline for issue #120's resolution. None of T0's re-audit/spike/
contract tasks, T1–T3's feature work, or T4's "close-out and polish" list
include gate-defect remediation; §6.1/§6.2 don't scope it in or out either.

**What I found:** Acceptance item 57 makes T-Series's own completion
contingent on a fix that lives entirely outside T-Series's stated scope,
with no phase of §20 responsible for it and no stated dependency ordering
relative to T4. A Work executing T4 close-out has no way to satisfy item 57
if issue #120 is still open at that point — it can neither fix the gate
itself (out of scope, unauthorized by any Decision in §20 or §22) nor wait
on a schedule the proposal never names.

**Survives the correction?** Yes — naming this as an explicit external
dependency ("T4 cannot close until #120 is independently resolved") would
make the acceptance item honestly conditional instead of silently assuming
a fix arrives in time.

---

## Finding 5 — the PR #111 disposition is pinned once at T0 but the program runs through T4, with no named re-check point

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

## Finding 6 — §19's test list and §21's acceptance list are both flat and undifferentiated by T-phase, leaving each phase's Work to infer its own subset of "done"

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

## What I checked and found nothing on

- §8.3's claim that Ratatui 0.30.2 already supplies `Tabs`, `Table`,
  `List`, `Paragraph`, `Scrollbar`, `Block`, `Clear`, `Gauge`/`LineGauge`,
  and styled `Span`/`Line`/`Text`: `Cargo.lock` confirms `ratatui 0.30.2`
  is the resolved version; these are genuinely built-in Ratatui widgets, not
  aspirational ones. Not a finding.
- §8.4's claim that every state glyph "is tested for one-cell width":
  `unicode-width 0.2.2` is already resolved in `Cargo.lock` (pulled
  transitively), so this is checkable with an already-available crate, not
  a dependency that needs to be invented. Not a finding.
- §8.7's crossterm-conflict concern itself: verified directly (see
  Finding 1) that `cargo add ratatui-textarea --dry-run` resolves cleanly
  against the current lockfile, and that the repository already tolerates
  two coexisting crossterm versions (0.28.1 via `duckdb`, 0.29.0 via
  `ratatui`) without incident. The *substance* of the spike's admission
  criteria is plausible to satisfy; only the "no product code" framing
  around it (Finding 1) is the problem.
- §19.7's "Live daemon tests ... Using fake backend": grepped
  `src/backend/fake.rs` and `tests/m7_docker_executor.rs`,
  `tests/m8_estate_cli.rs`, `tests/m9_watch.rs` — `FakeBackend` is an
  established, already-used pattern in this repository's integration tests
  (a real daemon process with a scripted fake actor backend), not a
  fictional harness the proposal invents. The section name and content are
  consistent with existing convention. Not a finding.
- §19.1/§19.10's claim that Ratatui `TestBackend` supports geometry testing
  at arbitrary sizes (80x24, 120x36, 180x48) with buffer assertions: this is
  a real, documented `TestBackend` capability (cited directly in §8.3's own
  references), and the fixture list in §19.10 gives each geometry a
  concrete set of screens to render — a Work has an actual checkable target
  here, not just a philosophy statement. Not a finding.
- §8.8's Kitty keyboard protocol claims (`PushKeyboardEnhancementFlags`/
  `PopKeyboardEnhancementFlags`, "nonfatal" failure): matches Crossterm's
  documented API shape as cited; the "integrated into the existing terminal
  lifecycle guard" claim has a concrete existing target (§17.6's list of
  guarantees already shipped) to extend rather than invent from nothing.
  Not a finding.
- §6.1/§6.2's in-scope/non-goal lists against §20's phases: every in-scope
  item traces to a specific T1–T3 deliverable and every non-goal is a
  negative (nothing to dispatch), so neither list itself hides an
  undecided question. Not a finding.
