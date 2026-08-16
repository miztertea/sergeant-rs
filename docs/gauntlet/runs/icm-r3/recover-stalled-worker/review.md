# Independent adversarial review: recover-stalled-worker

ICM-R3, `reference/proposal-icm-r-procedure-authority.md` §8.11 (independent
adversarial review) and ADR 0013 decision 7 (a later, fresh-execution,
review-only stage in the same reconciliation workflow qualifies as
independent). Reviewer position: fresh read, no edit authority over the
live package (`.sergeant/workflows/recover-stalled-worker/`) or the
producer's draft, per this dispatch's own instructions. Every disposition
below was independently re-derived against the live package content —
`.sergeant/workflows/recover-stalled-worker/{CONTEXT.md,index.md,
workflow.toml,00-collect-signals,40-escalate-on-second-attempt,
50-escalate-undocumented}` — and against `reference/sergeant-upstream/
{bin/sgt-recover,bin/sgt-watch,docs/troubleshooting.md}`, not accepted from
the producer's own citations.

Checklist applied per §8.11: source fidelity; rung order (PL and J);
Captain/workflow boundary; stage/helper boundary; authority grants and
missing J0 cases; package identity/naming; duplicated or drift-prone
content; false pairing assumptions; unjustified engine gaps.

---

### BU-RSW-01 -- verdict: CONFIRMED

Producer: PL-4, J5 (contract-level: exactly one bounded attempt, ever),
STAND.

Independent re-derivation: `CONTEXT.md` Purpose line ("One bounded recovery
attempt ... converge on a replacement or escalate — never guess") is a
workflow-level durable outcome, not a Captain-dialogue behavior and not a
stage-scoped checkpoint — PL-4 per §5.6's test ("given an already-defined
intent ... execute this procedure durably from admission to a terminal
result"). No adjacent rung (PL-2/PL-3) is facially plausible: nothing here
converses with a user to decide whether Work should exist. J5 is the right
rung because "exactly one bounded attempt, ever" is a governing invariant
the package may not silently relax, not a delegated per-run choice.

### BU-P8-095 -- verdict: CONFIRMED

Producer: PL-5, J5+J2, STAND, `00-collect-signals`.

Independently re-verified against `reference/sergeant-upstream/docs/
troubleshooting.md` L52-68: the quote is faithful (four signals: fleet
status/log mtime, pane identity + `pane_activity`, `progress_ts`/stall
diagnostic, td handoff + branch/worktree state) and the source text itself
instructs reconciling a nonterminal stall diagnostic "through the progress
rules ... before killing or relaunching anything" — matching the stage's
own J2 delegation (interpret via the documented progress rules) bounded by
a J5 floor (no kill/relaunch on partial evidence). PL-5 is correct: this is
a genuine judgment checkpoint (reconciling a nonterminal diagnostic is not
mechanical), not PL-6.

### BU-P8-099 -- verdict: CONFIRMED

Producer: PL-5, J2+J5, STAND, `00-collect-signals`.

Verified against L96-100 of the same source file: quote is faithful. J2
(investigate the specific cause) plus a J5 floor (never produce a duplicate
task or response) matches the source's own imperative framing ("Do not
create duplicate tasks or send duplicate responses"). PL-5 is correct for
the same reason as BU-P8-095 — distinguishing "stale record" from
"unconsumed response" from "misreclassified expected-blocked worker" is
judgment, not mechanical dispatch.

### BU-P6-071 -- verdict: CONFIRMED (see also cross-package note below)

Producer: PL-5 (checkpoint) + PL-6 (stamping mechanism), J5, STAND,
`40-escalate-on-second-attempt`.

Verified against `bin/sgt-recover` L6-10 (quoted verbatim in the CONTEXT.md
comment header) and independently against the N1 provenance file's
Adjudication A4 table, which already ran the §6.3 reimplementation test on
the three folded stages before this ICM-R3 pass began. Re-running that
test myself: swapping the stamp-check implementation leaves the observable
checkpoint (an already-stamped worker refuses a second attempt) unchanged,
so the PL-6/folded-helper placement holds. The dual citation inside
`40-escalate-on-second-attempt/CONTEXT.md` (own checkpoint + Helper §1) is
correctly explained as one fact serving two checkpoints, not duplicated
evidence — confirmed by inspection, this is a single physical statement
appearing twice in one file, not drift-prone content split across files
that could diverge.

One gap this unit's disposition does not address, flagged below rather
than repeated per-unit: what a **first**, legitimately-blocked preflight
failure (drain active, unprovable lease owner — see BU-P7-092/BU-P7-093)
reports to a human. See "Cross-cutting finding" below.

### BU-P6-073 -- verdict: CONFIRMED

Producer: PL-6, J5, STAND, `40-escalate-on-second-attempt` (helper).

Verified against `bin/sgt-recover` L140-155 and
`tests/sgt-recover-lease-owner-test.sh`. The fail-closed default ("anything
else ... must fail closed") is correctly cited as J5 (a governing floor,
not a per-run delegated choice) — the source explicitly treats "merely
looks idle" as insufficient proof, which the package content preserves
verbatim in spirit. PL-6 (folded helper, not a standalone stage) is
consistent with N1 Adjudication A4's reimplementation test, independently
re-applied: swapping the liveness-adjudication implementation leaves the
"unfinished lease blocks recovery unless provably dead" checkpoint
unchanged.

### BU-P6-075 -- verdict: CONFIRMED

Producer: PL-6, J5, STAND, `40-escalate-on-second-attempt` (helper).

Verified against `bin/sgt-recover` L229-232, L260-264 (cited ranges
consistent with the stated content — every preflight validation completes
before the one-shot budget is consumed). J5 is correct: "the attempt is
'used up' only once the coordinator has actually committed to relaunching"
is a governing ordering constraint, not delegated judgment. This unit is
also the basis for the cross-cutting finding below: a failed preflight
explicitly does **not** consume the budget, so a retry remains legally
possible, but the package states nothing about whether the operator is
told the first attempt did nothing.

### BU-P7-092 -- verdict: CONFIRMED (see cross-cutting finding)

Producer: PL-6, J5, STAND, `40-escalate-on-second-attempt` (helper).

Verified against `tests/sgt-recover-drain-test.sh` (grep-confirmed the file
exists and the cited behavior — recovery refused during an active drain —
matches the file's evident purpose). PL-6/J5 placement is correct: this is
a governing admission-control boundary shared with ordinary dispatch/
respond relaunches, not a delegated choice. See cross-cutting finding
below — this is one of the two concrete preflight-failure paths that has
no stated human-visible outcome distinct from "second attempt" escalation.

### BU-P7-093 -- verdict: CONFIRMED (see cross-cutting finding)

Producer: PL-6, J5+J1, STAND, `40-escalate-on-second-attempt` (helper).

Verified against `tests/sgt-recover-lease-owner-test.sh`. J5 (fail-closed
on unprovable ownership, including the reused-identifier edge case) is
correctly the dominant rung; J1 for "no raw shell error leaked to stderr"
is a defensible local/cosmetic choice (error-message text quality does not
change scope, authority, or destructive effect) per §6.6's own examples.
This is the other concrete preflight-failure path referenced in the
cross-cutting finding below.

### BU-P6-072 -- verdict: CONFIRMED

Producer: PL-6, J5, STAND, `40-escalate-on-second-attempt` (helper).

Verified against `bin/sgt-recover` L12-15. J5 (strict ordering: replacement
launched and identity-validated before the original is ever terminated) is
correctly a governing constraint — a failed relaunch sequence must leave
the original intact, which is exactly the kind of destructive-action
ordering rule the ladder reserves for J5, not J2.

### BU-P7-094 -- verdict: CONFIRMED

Producer: PL-6, J5, STAND, `40-escalate-on-second-attempt` (helper).

Verified against `tests/sgt-recover-replacement-test.sh`. Consistent with
BU-P6-072 — same ordering invariant, reinforced by the abort-path
requirement (restore fleet state so the recorded identity still points at
the surviving original). No adjacent rung is plausible; this is not a
delegated choice.

### BU-P7-095 -- verdict: CONFIRMED

Producer: PL-6, J5, STAND, `40-escalate-on-second-attempt` (helper).

Verified against `tests/sgt-recover-test.sh`. "A single bounded operation,
not an open-ended retry loop" restates the package's own top-level PL-4
invariant (BU-RSW-01) at the mechanical level — correctly PL-6/J5, and
correctly not treated as a duplicate of BU-RSW-01 since it states the
concrete four-step mechanical sequence (terminate, relaunch, update
metadata, notify), which BU-RSW-01 does not.

### BU-P8-109 -- verdict: CONFIRMED

Producer: PL-5, J5+J2, STAND, `50-escalate-undocumented`.

Verified against `troubleshooting.md` L242-244 — quote is faithful,
including the specific reference to the `sergeant-help` skill (confirmed
to exist at `skills/sergeant-help/`, not a dangling reference). PL-5 is
correct: composing a well-formed `td` task from an undocumented failure
requires judgment (what reproduction, what acceptance criteria), not
mechanical dispatch. This stage is also the workflow's own catch-all J0
escape valve for any stall condition the documented signals can't
classify — correctly distinct in scope from the cross-cutting gap below,
which concerns a *documented*, correctly-classified stall whose recovery
attempt is blocked for an operational reason (drain, unprovable lease).

### BU-RSW-13 -- verdict: CONFIRMED

Producer: N/A (authoring-format compliance), J5 (ADR 0013 decision 4 +
`convention.md` §6.1), STAND with in-place amendment required.

Independently verified: all three stage `CONTEXT.md` files were read in
full and each carries only the generic "## Judgment required" paragraph
("This is an actor stage (ladder §6.4) ... Treat the statements above as
binding constraints"), not the canonical `## Bounded judgment` shape from
proposal §7.3 (`### J2 — delegated to this stage` / `### J1` / `### J0` /
`### Completion boundary` / `### Decision evidence`). Cross-checked against
`validate-and-ship`'s current, already-amended stage `CONTEXT.md` files
(e.g. `40-drive-gates/CONTEXT.md:98`), confirming the target shape exists
and is achievable — this is a rollout gap, not an open design question, as
the producer states. The disposition wording ("STAND ... in-place content
amendment required") is identical in form to `validate-and-ship`'s
BU-VAS-13 at ICM-R2, correctly following that precedent rather than
inventing new vocabulary.

### BU-RSW-14 -- verdict: CONFIRMED

Producer: N/A, J5 (`convention.md` §6.1: every workflow Layer-1
`CONTEXT.md` carries an `## Authority envelope` section), STAND with
in-place amendment required.

Independently verified the section is genuinely absent from `CONTEXT.md`
(read in full — it has Purpose/Trigger/Stages/Adjudication
note/Provenance, no `## Authority envelope`). The producer's rejection of
treating this as a J0/needs-input case (in "Alternatives considered"),
analogous to `validate-and-ship`'s BU-VAS-15 push/pr/ci gap, is the most
consequential judgment call in the draft and I independently re-derived it
rather than accepted it on citation:

- Confirmed directly: `sgt-recover` requires `<task-id> <repo>` as
  positional arguments (`bin/sgt-recover` L21: `#   sgt-recover <task-id>
  <repo>`) — it cannot be invoked without already knowing the specific
  target.
- Confirmed directly: `bin/sgt-watch` never calls `sgt-recover`
  (`grep -n sgt-recover reference/sergeant-upstream/bin/sgt-watch` returns
  only a comment at L325, not an invocation).
- Confirmed directly via `troubleshooting.md` L52-68's own prose (not cited
  by the producer, but corroborating): "Preserve the worktree, branch,
  task, response generation, and handoff first, then use `sgt-recover
  <task-id> <repo>` only for that exact stall classification" — the
  upstream doc itself frames this as a deliberate, targeted, human-issued
  action following a diagnosis, not an automatic reaction.
- Checked whether anything in the current catalog could auto-dispatch this
  workflow without a human naming the target: `grep -rl
  recover-stalled-worker` across all workflow/skill Markdown and TOML found
  no other package delegating to it (confirmed independently, matching the
  producer's own "Relationships to other workflows" section) — there is no
  live auto-dispatch path today.

Unlike `validate-and-ship`'s BU-VAS-15 (where the source mechanism itself —
`scripts/gate.sh --skip push,pr,ci` — proves push/PR/CI is a
separately-controllable action the *workflow content is silent on
entirely*, with no equivalent precondition anywhere), this package's source
mechanism structurally requires the authorization-bearing information
(which worker) at the moment of invocation, and the package already
inherits that structural gate through its admission boundary. The rejection
of a J0 treatment holds up under independent re-derivation. The remaining
gap is real but is a documentation-completeness gap (state the precondition
explicitly), not an unresolved authority question — CONFIRMED as
dispositioned.

### BU-RSW-15 -- verdict: CONFIRMED

Producer: N/A (dangling in-package reference), J5 (`record-shapes.md` §1a
rule 1), **FOLD**, `CONTEXT.md`/`index.md`.

Independently verified the dangling reference: both `CONTEXT.md`
("See `provenance.md` for the complete stage-to-behavior-unit mapping")
and `index.md` reference a co-located `provenance.md` that does not exist
under `.sergeant/workflows/recover-stalled-worker/` (confirmed: `find
.sergeant/workflows -maxdepth 2 -iname provenance.md` returns nothing
anywhere in the catalog). The actual file is at
`docs/gauntlet/promoted-provenance/recover-stalled-worker.md`, confirmed
present and confirmed to carry the complete stage-to-behavior-unit mapping
matching every citation used elsewhere in this package.

I initially expected **STAND** (matching the parallel wording used for
BU-RSW-13/14) rather than **FOLD** for what looks like a simple broken
citation, but checked the exact precedent the producer cites:
`validate-and-ship`'s BU-VAS-10 used identical framing for an identical
defect category (a dangling in-package reference, there to a
`route-review-findings` workflow that doesn't exist) and was dispositioned
`FOLD` at ICM-R2, and `validate-and-ship/CONTEXT.md` now carries the actual
corrected text with an explicit "Corrected 2026-08-16, ICM-R2 pilot review
(BU-VAS-10)" note confirming that fix already landed in exactly this shape.
The producer's FOLD choice is precedented, not a misuse of the modifier
vocabulary — CONFIRMED, and my initial expectation was wrong.

---

## Cross-cutting finding (not in the producer's table) -- NEEDS-REVISION

**Claim:** the package's stated Outcome only accounts for two terminal
paths — (1) a bounded recovery attempt is made, or (2) the stall
classification is undocumented/unrecognized and escalates via
`50-escalate-undocumented`. It does not account for a third, real path: a
**documented, correctly-classified** stall whose preflight nonetheless
blocks the attempt for an operational reason — an active drain
(BU-P7-092) or an unprovable lease owner (BU-P7-093, BU-P6-073). Per
BU-P6-075, a failed preflight explicitly does not consume the one-shot
stamp/budget, so this is not the "second attempt" escalation path either
(BU-P6-071) — it is a first attempt that was correctly blocked and simply
stops.

**Why this matters:** for the standalone `sgt-recover` CLI, this gap is
invisible because the operator is watching the terminal synchronously and
sees the refusal directly. Inside a Sergeant Work, a stage that completes
with only `evidence`-disposition output (per `40-escalate-on-second-
attempt/output/README.md`) and no `needs_input` gives an asynchronous
operator no equivalent signal — the Work shows `completed`, not
`needs_input`, and nothing in the package states whether the operator is
expected to notice the worker is still stalled some other way (e.g.,
relying on `sgt-watch`'s independent periodic reclassification to
re-surface it later) or whether this is an actual gap.

**Checked whether this is already covered:** `50-escalate-undocumented`'s
own behavior contract (BU-P8-109) is scoped to "documentation does not
cover an observed failure" — a different condition from "the failure is
well-documented and well-classified, but the fleet's current state (drain,
lease) blocks acting on it right now." The two are not the same trigger,
and the package's Outcome section does not describe a third path.

**Disposition:** not classified as a placement error (no PL rung change is
implied — this is squarely inside `40-escalate-on-second-attempt`'s
existing PL-5/PL-6 scope) and not necessarily a genuine engine gap (the
existing `evidence`-disposition stage output already gives a human who
looks a durable record of what happened). It is a **missing J-boundary
statement**: the package should either (a) state explicitly that a
drain/lease-blocked preflight failure completes the stage with
`evidence`-only output and relies on `sgt-watch`'s own periodic
reclassification for continued visibility — an intentional design choice
mirroring the upstream architecture's division of labor between watcher
and recoverer — or (b) if that reliance is not actually intentional, add a
named J0 trigger for "preflight blocked for an operational (non-stamp)
reason" alongside the existing "undocumented stall class" J0. Either
answer is legitimate; leaving it unstated is the defect. This belongs in
the same "Surviving package design" remediation list as BU-RSW-13/14/15
(most naturally as an addition to item 1's `## Bounded judgment` section
for `40-escalate-on-second-attempt`, since it is exactly the kind of named
J0 trigger that section is supposed to enumerate), not a new package-level
verdict.

---

## Overall verdict on Final disposition

**Confirmed: STAND.**

Independently re-checked against the Placement Ladder, the Bounded-
Judgment Ladder, and the live upstream source: every behavior-unit rung
(PL-5 for the three surviving actor stages, PL-6 for the three N1-folded
helpers) holds under my own re-application of §6.3's reimplementation test,
not merely the producer's citation of it. The package's isolation from
`dispatch` and `worker-mission` is independently confirmed correct — I
re-ran the same greps and read the same relationship sections the producer
cites, and found no shared source mechanism, no shared behavior-unit
citation, and no other package's content naming `recover-stalled-worker`
as a delegation target anywhere in the current catalog. No REHOME, SPLIT,
HARVEST, or ABSORBED case is supportable from the evidence. No unjustified
engine-gap claim was made, and none is warranted — the "exactly one
attempt, ever" stamp mechanism is durable *fleet* state (a file/metadata
concern), not a Sergeant Work-journal concern, so PL-6/folded-helper is the
correct rung rather than PL-7.

The producer's rejection of a J0/needs-input treatment for the
admission-authority question (BU-RSW-14, the analogue of `validate-and-
ship`'s BU-VAS-15) is upheld on independent re-derivation from the source
CLI's own positional-argument requirement and `sgt-watch`'s confirmed
non-invocation of `sgt-recover` — this is the single most important claim
in the draft and it survives adversarial challenge.

One gap survives that the producer's table does not cover: the missing
third outcome path for a documented-but-operationally-blocked preflight
failure (Cross-cutting finding, above). This does not change the package's
identity, placement, or Final disposition — it is exactly the same class
of in-place content amendment as BU-RSW-13/14/15, and should be folded into
the same remediation batch before this package can be called
authority-valid per `reference/proposal-icm-r-procedure-authority.md`
§9.1 claim 3. The producer's own "Validation evidence" section already
correctly declines to call the package authority-valid yet; this review
adds one more named item to what "authority-valid" requires before that
claim can be made, without disturbing STAND as the Final disposition.

This record is itself draft reviewer output — per ADR 0013 decision 6, it
does not self-promote; the reconcile-and-publish step
(`reference/proposal-icm-r-procedure-authority.md` §8.12) remains a
separate, later action by the owner or delegated promotion gate.
