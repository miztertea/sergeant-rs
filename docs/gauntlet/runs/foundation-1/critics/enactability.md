# FOUNDATION-1 — enactability critic

Axis: can each section actually be executed as dispatched work? Graded
against `reference/proposal-foundation-rationalization.md` §1–§8, checked
against the repository, the seven ADRs (0005–0011), and
`docs/gauntlet/contracts/FOUNDATION-1.md`. Non-goals observed: no finding
argues an absent implementation, re-litigates a decision, or proposes a
different build than the one the proposal states.

Severity key: **error** = a dispatched Work would stall or produce an
unreviewable guess; **warning** = a Work could proceed but would have to
invent something the proposal should have supplied; **info** = worth
recording, doesn't block dispatch.

---

## Finding 1 — §6 sequences §5.3 before §5.2, but §5.3(a)'s own text makes the environment guarantee a product of §5.2

**Severity:** error
**Section:** §6, sequencing step 2 vs. step 3; interacts with §5.3(a) and §5.2

**The claim:** §6 orders "2. §5.3 (runtime contract) next... 3. §5.2
(passthrough) then, since §5.3(a)'s environment guarantee is what the
passthrough composes."

**What I checked:** §5.3(a)'s own definition: "Whatever composes an
actor's context states what it can rely on — its environment guarantee
(§5.2) and its execution model." The parenthetical attributes the
environment guarantee to §5.2. I then checked the ADR pair directly:
ADR 0007(a) says an actor needs to know, "alongside the environment
guarantee ADR 0006 establishes, what its own runtime model actually
permits" — ADR 0006 (§5.2) is explicitly named as the thing that
*establishes* the guarantee; ADR 0007 (§5.3) is the step that *states* it
*alongside* that establishment, not ahead of it.

**What I found:** §6's own justification for placing §5.2 third — "since
§5.3(a)'s environment guarantee is what the passthrough composes" — reads
the dependency backwards relative to §5.3(a)'s and ADR 0007(a)'s own
text: the guarantee is attributed to §5.2, not manufactured by §5.3.
A Work dispatched to do "§5.3 first" has two options neither of which the
proposal names: (a) write the context-composition text asserting an
environment guarantee that doesn't exist yet, which is false until §5.2
lands, or (b) ship only the execution-model half of §5.3(a) and defer the
environment-guarantee half until after §5.2 — a partial-§5.3 that the
proposal never describes as an option. The ordering section asserts a
justification that argues the opposite of what it orders.

**Survives the correction?** Not as written. The fix is cheap (state
which half of §5.3(a) can land before §5.2 and which cannot, or swap the
order), but as it stands a Work executing step 2 has no way to produce an
honest artifact without either lying about §5.2 or silently narrowing its
own scope.

---

## Finding 2 — §5.2 presents "composes the environment" as buildable while §8.1 admits its contents are unenumerated

**Severity:** error
**Section:** §5.2, cross-checked against §8.1

**The claim:** §5.2: "sgt composes the environment, binds the estate, and
**execs**." Presented as the concrete shape of the change, with a worked
consequence ("the harness, the daemon it spawns, and every actor beneath
inherit one deliberately-composed environment").

**What I checked:** §8.1 in the same document: "§5.2 composes 'the
environment'... The precise list — toolchain, estate binding, what else —
is not settled." I cross-checked against ADR 0006's own open questions:
"The exact set of environment variables and estate-binding facts the
passthrough must compose was not enumerated in the interview beyond the
general contract... this ADR records that the contract must exist and be
stated by the product, not its contents."

**What I found:** This is the exact "reads as decided but rests on an
unknown listed in §8" pattern the contract calls out. §5.2's own text
gives no list to build against — not PATH, not `CARGO_HOME`, not which
estate-binding facts (root only, or also `surfaces_dir`/`data_dir`?), not
whether secrets or credentials are ever composed. A Work dispatched
against §5.2 alone would have to invent that list itself, and nothing in
§5.2, §8.1, or the cited ADR gives it grounds to do so — §8.1 explicitly
says this is unresolved, not merely unstated shorthand for something
obvious. The "estate binding" half is narrower and closer to decidable
(ADR 0006 says binding is "decided once, explicitly, at launch time" —
i.e., the estate root resolved the way `resolve_data_dir`/estate discovery
already does), but the environment half has no enumerable content
anywhere in the proposal or its cited evidence.

**Survives the correction?** The "exec, not supervise" boundary (the part
§4.4/§4.7 grade) is unaffected — that's a shape constraint, not a content
list, and it's genuinely decided. But "compose the environment" as a
deliverable is not dispatchable as stated; a Work would need §8.1 resolved
first, or the section would need to scope itself to only the estate-binding
half and explicitly punt the environment-variable list to a named
follow-up (which is what §8.1 already functions as, but §5.2 doesn't say
so).

---

## Finding 3 — §5.4's "authority for both or neither" gives no precedence rung, and the surfaces_dir analogy it invokes points the opposite way from the code's stated `SGT_DATA_DIR` precedence

**Severity:** warning
**Section:** §5.4

**The claim:** "Add `[estate] data_dir`, for symmetry with `surfaces_dir`.
The manifest is authority for both or neither."

**What I checked:** Read `resolve_data_dir`'s doc comment
(`src/cli.rs:391-399`): "`--data-dir` flag, `SGT_DATA_DIR` — both
unchanged, unconditional precedence — then... an estate discovered by
walking upward." I then read how `surfaces_dir` actually resolves today:
`src/runtime/engine.rs:94-96` — "`workspace.surfaces_dir` when the
manifest declared one, else the engine's own default (`SGT_SURFACES_DIR`,
else `<data_dir>/surfaces`)." For `surfaces_dir`, the **manifest value
outranks the env var**. For `data_dir` as it stands today, the **env var
(`SGT_DATA_DIR`) has unconditional precedence** ahead of estate discovery
— the opposite relative ordering.

**What I found:** "Symmetry with `surfaces_dir`" is the only guidance §5.4
gives for where the new field slots into the resolution order, and taken
literally it would require `[estate] data_dir` to outrank `SGT_DATA_DIR` —
which contradicts §5.4(a)'s own adjacent claim, upheld from ADR 0008(a),
that "the existing rung order in `resolve_data_dir`... is upheld as-is."
Both cannot be true as written: either the rung order changes (env var no
longer unconditional) to achieve symmetry, or symmetry is not literal and
`data_dir` gets a different, unstated relationship to `SGT_DATA_DIR`. ADR
0008's own open questions say exactly this is unresolved ("whether it
slots in at the same rung as estate discovery currently does, or
somewhere else... is not specified"), but §5.4 doesn't carry that
qualifier forward — it states the symmetry claim as though it already
answers the question.

**Survives the correction?** The field addition itself (b) and the #64
re-ruling (c) are unaffected and enactable independent of this gap. Only
the precedence question is unenactable as stated; a Work would have to
either ask or guess which of the two contradictory readings to build.

---

## Finding 4 — §5.1's "rebuild only where matched" has no operational trigger, and the proposal doesn't carry forward the ADR's own admission of that gap

**Severity:** warning
**Section:** §5.1

**The claim:** "Stages get rebuilt only where we can show we have matched
them."

**What I checked:** ADR 0005's open questions, verbatim: "What
specifically counts as evidence that a rebuilt stage has 'matched' the
no-mistakes stage it would replace — a defect-count threshold, a
side-by-side run, an owner sign-off — was not specified in the interview.
Until that bar is named, 'rebuild only where we can show we have matched
them' has no operational trigger and risks never firing, or firing on an
ad hoc judgment call each time." §5.1 restates the rule near-verbatim but
omits this warning entirely.

**What I found:** The base dispatch mechanism for §5.1 is enactable — the
repository already has a published, actor-only `validate-and-ship`
workflow (`.sergeant/workflows/validate-and-ship/`, `status: published`,
stages `40-drive-gates`/`50-reconcile-custody` matching the ones §3.1
cites) that a Work can run today via the existing dispatch path, so
"gating becomes a dispatched Work" itself has a concrete thing to point
at. But the specific rule this finding is about — when a stage is
"rebuilt" versus kept as no-mistakes — has no acceptance criterion a Work
or its reviewer could check against, and the proposal presents it as a
settled operating rule rather than the open item its own source ADR
labeled it.

**Survives the correction?** Yes for the base dispatch mechanism; the
rebuild-trigger clause specifically needs either a named threshold or an
explicit "not yet specified" flag matching ADR 0005's own honesty about
it.

---

## Finding 5 — §5.7 elides ADR 0011's own flagged test-triage ambiguity; a concrete counterexample exists in the tree

**Severity:** warning
**Section:** §5.7

**The claim:** §5.7 describes deletion as a clean removal — "Remove
`src/web.rs`, `web/`, and the `sgt web` verb... and `web` leaves §5.5's
sweep list" — with no mention of which tests come with it.

**What I checked:** ADR 0011's own open questions: "The exact scope of
'm6's dashboard tests go with it' — which specific tests in
`tests/m6_surfaces.rs` are pure dashboard tests to delete outright versus
tests that exercise the three-clients-through-one-API invariant more
broadly and need rewriting rather than deletion — is not enumerated in
the interview; that triage is implementation work this ADR does not
perform." I then read `tests/m6_surfaces.rs` directly. `t5_the_tui_and_
the_dashboard_are_clients_like_any_other` (line 2460) is exactly the
ambiguous case: it loops `for module in ["tui.rs", "web.rs"]` and pins the
same "reaches state only through `ApiViews`" structural invariant
DEVELOPMENT.md cites as enforced by test, not convention. Deleting
`web.rs` breaks this test's compilation (it directly imports
`DASHBOARD_CSS, DASHBOARD_JS` at line 71 and reads `web.rs`'s source at
line 2491) — but the invariant it pins (clients reach state only through
the API) must survive for `tui.rs` alone, so this test needs rewriting to
drop `web.rs` from its loop, not outright deletion.

**What I found:** §5.7 doesn't surface this at all — it reads as though
"the dashboard's tests go" is one homogeneous, mechanical action. A Work
executing §5.7 naively (delete every `web`-touching test) risks deleting
the three-clients-equal invariant coverage along with the dashboard, which
would be a real regression against `docs/DEVELOPMENT.md`'s "Clients are
equal" invariant (out of scope for this axis to grade directly, but it is
the concrete cost of the missing acceptance criterion).

**Survives the correction?** Yes — this is a scoping gap, not a
contradiction. Naming the two categories (delete outright vs. rewrite to
drop `web.rs` from the loop) turns this into a checkable acceptance
criterion; §5.7 currently supplies neither category.

---

## Finding 6 — §6's "§5.4 is independent, slots anywhere after 1" permits landing it before §5.2, but §5.4's own rationale leans on §5.2 already existing

**Severity:** info
**Section:** §6, sequencing step 4, cross-checked against §5.4(a)

**The claim:** §6: "§5.4 (manifest authority) — independent, slots
anywhere after 1." §5.4(a) itself: "§5.2's explicit launch binding removes
most of the surprise that made [estate-first precedence] feel wrong in
the first place."

**What I checked:** Whether §5.4's actual deliverables (the `data_dir`
field addition, the #64 re-ruling) require §5.2 to exist first, versus
whether only the *rationale text* leans on it.

**What I found:** The coded deliverables of §5.4(b)/(c) don't need §5.2 —
adding a manifest field and closing an issue with a documentation ruling
are independent of the passthrough. But §5.4(a) is upheld, not built (per
ADR 0008(a): "upheld as-is"), and its stated justification for why the
status quo no longer feels wrong is explicitly conditioned on §5.2 having
landed. "Slots anywhere after 1" as written permits an ordering (§5.1,
§5.4, §5.3, §5.2) where §5.4 lands while citing a mitigation that doesn't
exist yet. This doesn't block dispatch of §5.4's actual work items, so it
doesn't rise above info, but the "independent" label overstates it
slightly against the section's own prose.

**Survives the correction?** Yes — narrowing "independent" to "the field
addition and re-ruling are independent; the precedence rationale assumes
§5.2" would resolve it without changing what any Work builds.

---

## What I checked and found nothing on

- §5.5 and §5.6's "no exceptions" and homepage-scope claims: both name
  their genuinely open sub-questions in-line (§5.6's estate-awareness
  fork) rather than hiding them, and both have a literal, buildable base
  deliverable (a static homepage; a no-spawn verb list) that doesn't
  depend on resolving the flagged open question. Not a finding.
- §6's claim that §5.5/§5.6/§5.7 share test blast radius: verified —
  `tests/m6_surfaces.rs:414` (`t1_bare_sgt_opens_the_tui_as_a_client`) is
  cited by both ADR 0009 and ADR 0010 as pinning behavior each of §5.5 and
  §5.6 reverses; `tests/m8_estate_cli.rs`'s auto-spawn assertion and
  `tests/m6_surfaces.rs`'s dashboard tests are in the same files. The
  "three passes over the same pinned tests" cost claim holds.
- §5.3(b) (the closing-stage dirty-worktree guard): has a concrete,
  checkable acceptance criterion as stated ("must not land in plain
  `completed` when the branch never advanced and the worktree is dirty")
  independent of §5.3(a)'s problems above — a Work could build and test
  this in isolation. Not a finding.
- §8.2, §8.3, §8.5: checked each against repository evidence for whether
  it's actually decidable now and left vague. All three turn on future
  measurement or product judgment (self-review independence under load,
  a UX preference, unmeasured user need) that no static evidence in this
  repo resolves. Genuinely unresolved, not merely unstated.
- §8.4: the `no-mistakes` skill (a directly-invoked `/no-mistakes` entry
  point) and the `validate-and-ship` workflow (a separately dispatchable,
  published workflow) both currently exist side by side, confirming the
  premise of the open question is live, not stale — but whether the
  skill entry is "dead weight" is a judgment about intended future usage,
  not something the repository's current state settles either way.
