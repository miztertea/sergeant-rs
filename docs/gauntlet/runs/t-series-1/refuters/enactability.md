# T-SERIES-1 — enactability refuter

Refuting `docs/gauntlet/runs/t-series-1/critics/enactability.md` against
`reference/proposal-tui-t-series.md`, the repository at
`ddc4719d9b30e5efa0a904505dfb5a8950b7c38d` (`242abe3c4a889c2b666c7ce34b32812dd1ee8d61`,
the critic's checked commit, is an ancestor; nothing in `Cargo.toml`,
`Cargo.lock`, `tests/m6_surfaces.rs`, `src/backend/fake.rs`,
`reference/notes/gauntlet-pattern.md`, or `.sergeant/workflows/` changed
between the two), and live GitHub via `gh`. I did not write the proposal or
the critic findings. Method per `docs/gauntlet/contracts/T-SERIES-1.md`:
re-verify each factual claim independently, re-derive rather than trust the
critic's own citations, check scope, check for style dressed as defect,
check severity.

---

## Finding 1 — §20.1's "No product code" vs. the T0 dependency-resolution spike

**Verdict: REFUTED as argued.**

**Re-verification.** I reproduced the critic's own checks against the live
repository: `cargo tree -i crossterm@0.28.1` and `crossterm@0.29.0` both
confirm the two-version state the critic describes (`duckdb → comfy-table`
pulls 0.28.1; `ratatui-crossterm → ratatui` pulls 0.29.0). I then ran
`cargo add ratatui-textarea --dry-run` against the real tree — it resolves
cleanly and reports "aborting add due to dry run." I built the critic's
"full scratch-crate" check myself in `/tmp/scratch-crate`
(`cargo add ratatui@0.30.2 ratatui-textarea && cargo fetch`): the lockfile
shows a single `crossterm 0.29.0`, no duplicate edge — matching the critic's
own figures exactly. `git status --short Cargo.toml Cargo.lock` in the real
repository after all of this: **empty**. Every fact the critic cites is
accurate.

**Where the critic overstates it.** The finding's core claim is that
producing §8.7's required evidence "requires actually touching `Cargo.toml`
and/or running `cargo add`/`cargo tree`/`cargo build` against the real
dependency graph, which is a change to and execution of product build
files" — and concludes "a T0 Work has no way to produce an honest artifact
without either skipping a task §20.1 lists for it or violating the
constraint §20.1 states for it." I just demonstrated the opposite: `cargo
tree` is read-only (no mutation of any file); `cargo add --dry-run` is
Cargo's own purpose-built no-op inspection mode, engineered specifically to
report a resolution without writing it; and a scratch crate outside the
product tree entirely is, definitionally, not the product's build files.
All three are exactly the tools the critic used to write this finding, and
all three leave the real `Cargo.toml`/`Cargo.lock` untouched — which I
verified directly, not by assumption. "No product code" reads naturally as
"no application/UI code"; read-only dependency inspection and a throwaway
scratch crate are not code the product ships, let alone code at all in the
scratch-crate case. The critic anticipated and rejected only the
"touch-then-revert" path ("even if the change is later reverted") — they
never considered the dry-run/scratch-crate path, despite using it
themselves one paragraph earlier in the same finding to gather their own
evidence. A path that produces the required evidence without touching a
single product file exists; the "no way" claim is factually wrong.

**Severity.** Refuted, not downgraded — the section survives without
correction. If anything remains, it's a documentation nicety (the proposal
could say "the spike uses dry-run/scratch-crate tooling, not a committed
edit"), but ordinary engineering practice for a timeboxed spike already
defaults to non-mutating investigation first, and a Work with normal
cargo-tooling competence — which is what the critic themselves
demonstrated while writing this very finding — resolves this without
inventing anything or stalling.

---

## Finding 2 — §20.1's "review this proposal through the repository's proposal gauntlet" names a process a Work can't perform

**Verdict: CONFIRMED, severity downgraded error → warning.**

**Re-verification.** I read `reference/notes/gauntlet-pattern.md` directly:
the loop is `CONTRACT → BUILD → GATES → BLIND CRITICS → ADVERSARIAL VERIFY
→ FIX → CHECKPOINT GATE → ADJUDICATE → MARK & LOG`, explicitly run by "the
orchestrator (a Fable 5 session)" dispatching builder/critic subagents
through ultracode workflows. I listed `.sergeant/workflows/` myself: 23
entries (`code-review`, `dispatch`, `implement`, `tdd`, `wayfinder`,
`validate-and-ship`, etc.) — none of them is "run a blind-critic gauntlet
against a proposal." `docs/gauntlet/contracts/T-SERIES-1.md` confirms this
proposal is inside exactly that process right now and states "Build
against the graded proposal is a separate, later contract... sequenced
only after this unit closes." All of the critic's supporting facts check
out.

**Where the critic overstates it.** The finding frames this as a bullet
with "no defined action to take," on the same footing as a genuine stall.
But T0, by the contract's own construction, can only be dispatched after
this exact gauntlet unit has closed — any orchestrator capable of
dispatching a T0 Work necessarily already knows the gauntlet ran and how it
resolved (validated / validated with findings / sent back), because that
outcome is what authorized T0 to exist at all. A T0 Work reading this
bullet in that context has an obvious, low-judgment resolution available:
cite the closed gauntlet's outcome (e.g., "T-SERIES-1 closed
validated-with-findings, per `GAUNTLET.md`; proceeding") and move to the
next bullet. That is not "no defined action" — it's a precondition
citation, not a re-run of the loop from inside itself. A dispatched Work
would not stall on this, and what it "invents" (a one-line citation of an
already-settled fact) is not a judgment call the proposal was obligated to
make for it.

**What survives.** The critic is right that the bullet, read cold and
literally as an *action T0 performs*, has no single crisp referent — it's
genuinely ambiguous between "dead/redundant precondition prose" and "an
instruction no single Work can execute," and the proposal doesn't
disambiguate. That's a real, cheap-to-fix wording gap, just not one that
blocks or misdirects a competently dispatched Work the way an "error" would.

**Severity.** Downgraded to **warning**: matching the axis's own bar, a
Work "could proceed but would have to invent" (here: infer that this is a
precondition citation, not a literal action) something the proposal should
have stated plainly — it would not stall or produce an unreviewable guess.

---

## Finding 3 — T0–T4 never assigns Workflow/Evidence/Graph/Details tabs to a phase

**Verdict: CONFIRMED as scored (error).**

**Re-verification.** I read §20.2–§20.5 and §13.2–§13.7 directly. §13.2
(Decision T2-48) requires one canonical Work surface with views `Thread
Workflow Evidence Graph Details`; §21 items 13, 24, 25, 26 bind all four
non-Thread tabs into acceptance. §20.2 (T1)'s own Work-surface bullets are
exactly "canonical Work shell," "transcript-backed Thread," and
"output/envelope/completed-dirty" — the last of these maps to §13.8/§13.9
(Output/Envelope), not §13.4–§13.7. I also checked §7.2 ("Canonical Work")
for a definition that might bundle the four tabs into "canonical Work
shell" explicitly — it doesn't; §7.2 only says "one canonical full-body
Work surface" and lists entry points, never naming the sub-tabs. T2 is
scoped to the Workflows catalog screen and Home's `@` chooser (a different
surface from the per-Work Workflow rail); T3 is Estate/Doctor; T4 is
explicitly close-out/polish bullets (fixtures, screenshots, docs, ledger,
handoff) that read as finishing already-built surfaces, not building new
ones. None of T1–T4's bullets name the Workflow rail, Evidence, Graph, or
Details tabs, or their describing nouns. This matches the critic's claim
exactly.

**Where the critic could be second-guessed, and why it doesn't hold.** One
could argue "canonical Work shell" is obviously meant to bundle all five
tabs and this is style-nitpicking. But the critic's own counter-evidence is
strong: "transcript-backed Thread" is called out as its *own* separate T1
bullet immediately next to "canonical Work shell" — if "shell" already
meant "all five views," naming Thread again separately would be redundant.
The proposal's own bullet structure argues against the generous reading.

**What I found independently.** A Work executing T1–T4 exactly as scoped
could pass every phase's own checklist while silently never building
Workflow/Evidence/Graph/Details, and nothing in §20 catches this before
final acceptance review against §21 items 13/24–26 — a genuinely late,
expensive discovery point for a gap that's cheap to close by naming the
four tabs under T1 (the data backing all four is already described as
existing).

**Severity.** No basis to downgrade. This is not a narrow sub-clause
blocked while the rest proceeds (the FOUNDATION-1 pattern that justified
downgrades on other findings in this run) — it's four full, non-trivial,
acceptance-bound deliverables with no owning phase at all, discoverable
only after the fact. **Error stands.**

---

## Finding 4 — "validate workflow catalog route" is ambiguous between paperwork and code

**Verdict: CONFIRMED as scored (warning).**

**Re-verification.** §11.2, read directly: "The exact response is
contracted before implementation" — the route does not exist yet at
proposal-text time. §20.3 lists "workflow catalog endpoint" as a T2
deliverable, not T0's. I grepped the entire proposal for "validate"
(case-insensitive): it occurs **exactly once**, in the T0 bullet itself —
there is no other usage anywhere in the document to establish a
disambiguating house convention. The critic's claim that "validate" has no
textual anchor to resolve toward either reading is accurate, not merely
plausible.

**Scope/severity check.** This doesn't stall T0 outright — a Work would
likely default to the schema-review reading given §11.2's own "contracted
before implementation" framing sits right next to it — but it is a real
case of the proposal using a stronger verb than its own adjacent section
supports, exactly the "invent something the proposal should have supplied"
shape the axis defines as warning. **Confirmed at warning, unchanged.**

---

## Finding 5 — Acceptance item 57 depends on an unowned, still-open gate defect

**Verdict: CONFIRMED as scored (warning).**

**Re-verification.** `gh issue view 120`, run independently: **state OPEN**,
title "no-mistakes resolves its diff base against a live non-bare origin
checkout, silently short-circuiting on empty diff," reproduction section
intact. I read §3.3, §19.12, and §21 item 57 directly — all three quotes
in the finding are verbatim. I grepped the full proposal for "#120",
"false green", "false pass", and "gate defect": the only three hits are
the ones the critic already cites (§3.3, §19.12, §21.57) — no fourth
location assigns remediation. I separately read all of §20.1–§20.5 myself
looking for any gate-fix task under any phase name (re-audit, spike,
contract, close-out): none exists. Every factual claim holds.

**Scope check.** Is this re-litigating the North Star gate-lift ruling or
extending scope into an adjacent problem the proposal doesn't claim to
solve? No — item 57 is the proposal's *own* acceptance criterion, and the
finding is narrowly about whether the proposal supplies any mechanism to
satisfy the criterion it itself states. In bounds for this axis.

**Severity.** Correctly scored — this doesn't block T0–T3 dispatch, and
it's plausible #120 gets fixed by unrelated infra work before T4, matching
the critic's own "warning, not error" reasoning. **Confirmed, unchanged.**

---

## Finding 6 — PR #111's disposition is pinned once at T0 with no named re-check point

**Verdict: CONFIRMED, severity downgraded warning → info.**

**Re-verification.** §12.4 and Decision T2-06 (§3.3), read directly, match
the critic's quotes. I checked §20 for a re-check point between the T0 pin
and T3/T4 (when retained/reap UI is actually built, per §20.4): none
exists, as claimed.

**What re-deriving the critic's own citation changes.** The contract
explicitly names PR #111's live merge status as something every axis must
re-verify against current GitHub rather than trust the proposal's own
audit-time text ("§12.4 treats PR #111's retained/reap surfaces as
conditional on that PR merging — `gh pr view 111` shows it is already
merged"). I ran `gh pr view 111 --json state,mergedAt,mergeCommit`
independently: **state MERGED**, `mergedAt: 2026-08-15T15:28:02Z`, merge
commit `3a46b87`. I confirmed `3a46b87` is an ancestor of the real
`github/main` (fetched live from `https://github.com/miztertea/sergeant-rs`),
which has moved well past it (`github/main` HEAD is `5d51f21`, PR #125).
The critic's Finding 6 quotes §3.3's audit-time description ("not ready to
merge") as "the state the proposal itself describes at audit time" and
builds its risk scenario on that being the live state T0 might still pin
("if T0 pins the base with PR #111 'explicitly excluded'... and the PR
merges later"). Unlike Finding 5, where the critic ran `gh issue view 120`
to re-check #120's live state, Finding 6 never runs `gh pr view 111` to
re-check PR #111's live state — it treats the proposal's own stale
audit-time text as still current, which is exactly the trap the contract's
assumptions-axis note warns against, and which I was told to re-derive
rather than trust.

**What this does to the finding's substance.** The critic's risk is
bidirectional: (a) T0 pins "excluded," then the PR merges during T1–T3 with
no re-check; (b) T0 pins "merged," then something reverts it later with no
re-check. Branch (a) is now foreclosed as a live risk — PR #111 has
already been merged for some time, well before any T-Series T0 Work would
be dispatched (T0 cannot start before this gauntlet unit closes, and the
merge predates the unit's own contract). Whenever T0 actually runs, it
will observe "merged" and pin that, once, permanently — there is no window
left in which "excluded, then merges later" can occur for this specific
proposal's program. Branch (b) (merge later reverted) remains
hypothetically possible but is unprecedented in this repository's history,
requires an active decision to unwind a shipped, already-built-upon merge,
and is not meaningfully different from the general regression risk any
already-shipped feature carries — treating it as a T-Series-specific
enactability gap overstates a generic risk no proposal specifically
defends against.

**What survives.** The literal textual gap the critic names — the proposal
never states a re-check mechanism, as a matter of process design — is
still true and still cheap to name explicitly. But the concrete, load-bearing
half of the critic's own risk scenario is moot against the real, current,
already-settled disposition, which the critic could have checked with the
same `gh` call they used one finding earlier.

**Severity.** Downgraded to **info**: worth recording as a documentation
nicety, but it no longer meets "a Work could proceed but would have to
invent something" — the practical scenario that would force an invention
has already been foreclosed by real-world events that predate T0's
earliest possible dispatch.

---

## Finding 7 — §19/§21 are flat and undifferentiated by T-phase

**Verdict: CONFIRMED as scored (info).**

**Re-verification.** Spot-checked the claimed mappings directly: §19.2's
"workflow live-versus-pinned labeling" bullet (confirmed present at that
location) requires the catalog endpoint, which §20.3 assigns to T2, not
T1 — matching the critic's example exactly. I confirmed §21 items are not
phase-tagged anywhere in §21's text. The critic's own severity reasoning
(citing FOUNDATION-1 precedent that this kind of reader-side legwork is a
known, acceptable cost, and that content-based mapping is inferable) is
sound and consistent with how the FOUNDATION-1 refuter treated
comparably-shaped findings. **Confirmed, unchanged.**

---

## Finding 8 — §8.7's spike checklist omits the actual gating test

**Verdict: CONFIRMED as scored (info).**

**Re-verification.** I independently fetched and read the vendored
`ratatui-textarea-0.9.2` source: `src/input/crossterm.rs` imports from
`ratatui_crossterm::crossterm::event::{...}` throughout, exactly as
claimed. I confirmed `ratatui-0.30.2/src/lib.rs:483` reads
`pub use ratatui_crossterm::crossterm;`, the same re-export path. I read
`tests/m6_surfaces.rs`'s `the_tui_stack_is_ratatui_with_crossterm_reached_through_it`
test directly: it asserts no direct `crossterm` line in `Cargo.toml` and no
bare `use crossterm` in `tui.rs` — matching the critic's description
exactly, at the location cited. Both technical claims check out to the
letter. The finding's own conclusion — the dependency clears the bar, but
§8.7's checklist should name the pinned test as an eighth condition — is
correctly scored as low-stakes, since (per Finding 1's refutation) the
substance is already satisfiable and this is purely a completeness nicety
for the checklist. **Confirmed, unchanged.**

---

## Finding 9 — T0 has no closing acceptance criterion of its own

**Verdict: CONFIRMED as scored (warning).**

**Re-verification.** §20.1 ends with "write T1 only after rulings" — no
further qualifier, confirmed by direct read. §20.5 (T4), by contrast, is
explicit: fixtures, pre-flight, screenshots, README/help updates,
ledger/lessons/ADR/proposal-supersession updates, explicit handoff. I
grepped the full proposal for "ruling" (case-insensitive): four hits, none
of which defines what a T0 "ruling" artifact is, where it's recorded, or
what a reviewer checks — matching the critic's claim that no elaboration
exists anywhere in the document.

**Scope check.** Not a request to design the artifact shape (which would
be out of scope per the contract's non-goals) — the finding only asks that
T0 name its own closure artifact the way every other phase does, using the
repository's existing conventions (ADRs, `GAUNTLET.md`, the ledger) that T4
itself already cites. In bounds. **Confirmed, unchanged.**

---

## Summary

| Finding | Critic severity | Refuter verdict | Refuter severity |
|---|---|---|---|
| 1 | error | REFUTED | — |
| 2 | error | CONFIRMED (framing refuted) | warning |
| 3 | error | CONFIRMED | error |
| 4 | warning | CONFIRMED | warning |
| 5 | warning | CONFIRMED | warning |
| 6 | warning | CONFIRMED (risk scenario largely refuted) | info |
| 7 | info | CONFIRMED | info |
| 8 | info | CONFIRMED | info |
| 9 | warning | CONFIRMED | warning |

One of nine findings is fully refuted on the facts: Finding 1's "mutually
exclusive as written" claim is disproven by the critic's own investigative
method, which already resolves the tension it describes without touching
any product file. Two findings (2, 6) survive on their narrower textual
point but lose the severity their headline framing claimed — Finding 2
because a competently dispatched Work has an obvious, non-stalling
resolution available from context; Finding 6 because the real, current,
already-settled disposition of PR #111 (re-derived live via `gh pr view
111`, not trusted from the critic's stale quote of the proposal's own
audit-time text) forecloses the concrete risk scenario the finding's
severity depended on. The remaining six findings (3, 4, 5, 7, 8, 9) hold
at the critic's original severity — each re-verified independently against
the live repository, live GitHub, or the proposal's own text, not assumed
from the critic's citations.
