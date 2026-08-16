# Package adjudication review: vet-external-skill

ICM-R3 independent adversarial review of
`docs/gauntlet/runs/icm-r3/vet-external-skill/adjudication-draft.md`, per
`reference/proposal-icm-r-procedure-authority.md` §8.11 (challenge list)
and ADR 0013 decision 7 (independence lives in the execution boundary: a
fresh execution, explicit inputs, a review-only contract, no edit
authority). This review re-derives the producer's classification directly
against `.sergeant/workflows/vet-external-skill/` and the cited upstream
source (`reference/sergeant-upstream/docs/skills.md`), not from the
producer's own citations, then compares the two. No file under
`.sergeant/workflows/vet-external-skill/` or the producer's draft was
edited to produce this record.

## Method note

Verified directly, independent of the draft: the seven stage
`CONTEXT.md` files and workflow-level `CONTEXT.md` were read in full;
`workflow.toml`'s stage list; `index.md`; `reference/sergeant-upstream/
docs/skills.md` L95-153; `tests/` (repository root) contents and a
repo-wide search for `instruction-policy-test.sh`;
`docs/gauntlet/promoted-provenance/vet-external-skill.md`;
`docs/icm/convention.md` §6.1/§7.2/§7.3 (Authority envelope / Bounded
judgment section requirements); a grep of `src/` for stage-subset-entry
or partial-stage-walk engine support; and the file the producer cites as
carrying the `validate-and-ship` single-linear-stage-list precedent.

## Behavior-unit dispositions

### BU-VES-01 — verdict: CONFIRMED

Re-derived independently: `reference/sergeant-upstream/docs/skills.md`
L124-131 states the six-step "Before adopting an external skill" sequence
verbatim as the producer describes it, and `CONTEXT.md` (Purpose) states
the same governing sequence. PL-4 (package) is correct — the workflow
receives a bounded intent (vet skill X) and executes durably to a
terminal accept/reject result, independent of conversation continuing.
J5 is the right rung: this is the whole package's reason to exist, not a
delegated or local choice. No adjacent rung (J4/J3) fits better since no
single explicit user decision or settled record states the prohibition —
it is a standing governing constraint from the source doctrine itself.

### BU-VES-02 — verdict: CONFIRMED

`00-read-source/CONTEXT.md`'s Behavior contract matches `BU-P1-120` and
upstream L126 exactly. PL-5 holds under the reimplementation test — the
outcome ("the skill's full instructions and scripts were read") survives
any change to how reading happens. J2 for "what counts as a referenced
script" is a reasonable delegated judgment class; nothing in the stage
text overclaims a J5/J4 basis for it.

### BU-VES-03 — verdict: CONFIRMED

Independently re-read `10-confirm-provenance/CONTEXT.md` in full: the
Behavior contract is exactly "Confirm the external skill's source and
update mechanism," with no branch for unconfirmable provenance anywhere
in the stage's own text. The producer's rung-by-rung J0 derivation (J5
through J1 checked and correctly found not to resolve it) holds up
independently — this is a genuine authoring gap, not an artifact of the
producer's reading.

### BU-VES-04 — verdict: CONFIRMED

Independently re-read `20-check-actions/CONTEXT.md` in full: the five
categories are named, no consequence clause exists for what a bad finding
means. The producer's observation that `30-verify-no-conflict` might seem
to absorb this judgment is correctly rejected in the producer's own
record (that stage doesn't reference "findings from `20`," it re-derives
conflict independently against `AGENTS.md`/safety policy) — re-checked
directly against `30-verify-no-conflict/CONTEXT.md`'s text, which indeed
never mentions `20`'s findings. J0 conclusion holds.

### BU-VES-05 — verdict: NEEDS-REVISION

The producer's table disposes this unit as a clean STAND (J5 + J2, no gap
noted), but independent re-reading of `30-verify-no-conflict/CONTEXT.md`
in full shows it has **exactly the same missing-consequence shape the
producer itself identified for BU-VES-03/04/08**: the Behavior contract
is "Verify the external skill does not conflict with repository
`AGENTS.md` or safety policy" — a check, stated with no branch for what
happens when a conflict **is** found. Running the producer's own rung
sequence against this stage: J5 — no in-stage governing text says what to
do on a confirmed conflict (the *fact* that conflicting skills must not
be adopted is BU-VES-01's workflow-level constraint, restated at the
stage level as a check, not an escalation clause — the same distinction
the producer draws for BU-VES-04's J5 check). J4 — none. J3 — none. J2 —
"verify… does not conflict" names the check, not "decide what to do if it
does." J1 — does not apply; whether an adoption proceeds past a
discovered conflict is exactly the risk-changing, non-local choice J0
exists for. **Conclusion: J0**, same procedure as the three gaps the
producer already recorded — this is a fourth, uncaught instance of the
identical defect class, not a new kind of finding. It should be added to
"Surviving package design" item 1's stage count and to the gap-records
section, alongside BU-VES-03/04/08.

### BU-VES-06 — verdict: NEEDS-REVISION

Same defect class again. Independent re-reading of
`50-test-in-disposable-copy/CONTEXT.md` in full: the Behavior contract is
"Test the external skill in a disposable repository or worktree before
broad installation" — no clause for what a failing disposable-copy test
means (do not install broadly / re-vet / reject). Rung check: J5 — the
governing "no broad installation without a prior disposable-copy test"
constraint the producer already cites for this same unit says a test must
happen, not what a failed test requires. J4 — none. J3 — none. J2 — "is
the run representative" is named (the producer's own J2 clause); "what to
do if the representative run fails" is not. J1 — does not apply, same
reasoning as the other four. **Conclusion: J0.** The producer's own
"Additional note" and "Alternatives considered" sections repeatedly treat
this exact defect shape (named check, missing consequence) as
J0-worthy when found at BU-VES-03/04/08 — the standard was not applied
uniformly across all seven stages, only three of the five stages that
actually exhibit it.

### BU-VES-07 — verdict: CONFIRMED

Independently re-read `50-test-in-disposable-copy/CONTEXT.md`'s "Helper
invocation: pin source" section and the workflow.toml history implied by
N1 adjudication A4 (cross-checked against
`docs/gauntlet/promoted-provenance/vet-external-skill.md`'s own
`40-pin-source — DEMOTED` note, which independently corroborates the same
account). PL-6 fold is correctly executed: `BU-P1-124` is present, framed
as a helper step performed before testing, matching the reimplementation
test (pin/lock is mechanical once the decision to proceed is made). No
further placement change needed — CONFIRMED as-is.

### BU-VES-08 — verdict: CONFIRMED

Independently re-read `60-update-managed/CONTEXT.md` in full: "accept" is
covered by the Judgment-required prose; "reject/escalate on a suspicious
diff" is not named anywhere in the stage. Rung derivation matches the
producer's. J0 conclusion holds.

### BU-VES-09 — verdict: CONFIRMED

Independently verified the source-fidelity defect directly, not from the
producer's citation: `reference/sergeant-upstream/docs/skills.md`
L142-144 says `` `bash tests/instruction-policy-test.sh` `` verbatim; a
repo-wide search (`find . -iname 'instruction-policy-test.sh'`) finds
exactly one match, `reference/sergeant-upstream/tests/
instruction-policy-test.sh` — frozen upstream-source material, not this
repository's own `tests/` (`estate_routes.rs`, `m1_event_core.rs`, …,
`t2_workflow_catalog.rs`, no such script). The producer's citation of this
defect, and its refusal to invent a substitute path, is independently
sound.

### BU-VES-10 — verdict: CONFIRMED

Independently re-read all seven stage `CONTEXT.md` files' final sections:
every one carries the identical "## Judgment required" boilerplate
paragraph ("This is an actor stage (ladder §6.4)…"), none contains a
`## Bounded judgment` heading, and none names J2 decision classes, J1
local choices, or J0 escalation triggers in the ADR-0013/`convention.md`
§6.1 required shape. Cross-checked `convention.md` line 425 directly:
"every actor stage's `CONTEXT.md` carries a `## Bounded judgment`
section... omission is never ambiguous." Governing requirement,
unsatisfied, confirmed.

### BU-VES-11 — verdict: CONFIRMED

Independently re-read the workflow-level `CONTEXT.md` end to end: no
`## Authority envelope` heading exists anywhere in the file (sections
present are Purpose, Trigger, Stages, Notes for reviewers, Provenance).
Cross-checked `convention.md` line 422 directly: "Every workflow's
Layer-1 `CONTEXT.md` carries an `## Authority envelope` section." Gap
confirmed.

### BU-VES-12 — verdict: CONFIRMED

Independently re-read `CONTEXT.md`: line 34 ("See `provenance.md`'s
'Adjudication A4' section") and line 38 ("See `provenance.md` for the
complete stage-to-behavior-unit mapping and workflow-level citations")
both cite a co-located `provenance.md`. `find
.sergeant/workflows/vet-external-skill -iname 'provenance.md'` returns
nothing; the content actually lives at
`docs/gauntlet/promoted-provenance/vet-external-skill.md`, exactly as the
producer's suggested fix states. Line numbers, defect, and proposed fix
all independently verified.

### BU-VES-13 — verdict: DISPUTED (conclusion CONFIRMED, one citation in the producer's own evidence trail is wrong)

The producer's substantive conclusion — J0, the two-entry structural
tension is a genuine, currently-unverifiable engine-capability question,
not a defect in this package specifically — is independently re-derived
and holds. A direct grep of `src/` for stage-subset-entry or
partial-stage-walk support (`entry_stage`, `start_stage`, `stages\[0\]`,
`current_stage`, etc.) found nothing confirming the engine supports
starting a Work's walk at other than the first declared stage or
terminating before the last; this is consistent with, not contrary to,
the producer's refusal to assert either way from content alone.

However, the producer's own source-fidelity fails on this unit's
supporting citation, exactly the class of error §8.11 asks a reviewer to
catch. The producer writes: *"`validate-and-ship`'s own already-accepted
precedent, quoted directly in that package's
`20-select-intent-transport/CONTEXT.md` line 61."* Independently checked
directly: the quoted text ("...per convention.md's single-linear-stage-
list model (no engine-level branching exists at this milestone)") does
not appear in `20-select-intent-transport/CONTEXT.md` at all — it is at
`.sergeant/workflows/validate-and-ship/CONTEXT.md:61`, the **workflow-
level** `CONTEXT.md`, not the named stage's. The quoted words themselves
are accurate (confirmed by direct grep), but the file attribution is
wrong. This does not change BU-VES-13's disposition or the J0 conclusion
— the precedent exists, just not where cited — but it is a factual error
in this adjudication record's own evidence trail and should be corrected
before the record is treated as settled, per the same source-fidelity
discipline the producer correctly applied to the package's own citations
(BU-VES-09).

## Additional findings outside the producer's table

**Missing J0 gaps undercounted in "Surviving package design."** Item 1 of
the producer's remediation list states "Three of the seven
(`10-confirm-provenance`, `20-check-actions`, `60-update-managed`) need a
genuinely new J0 clause." Per BU-VES-05/BU-VES-06 above, this should read
**five of seven** — add `30-verify-no-conflict` and
`50-test-in-disposable-copy`. This is the same authoring gap, found by
applying the producer's own already-correct method to the two stages it
was not applied to.

**Captain/workflow boundary — re-checked, no change.** Independently
applied PL-2's discriminator ("does the procedure's job require deciding
what Work should exist") against this package: no, `vet-external-skill`
consumes an already-decided intent (which skill, from where) and
produces a durable verdict. The producer's rejection of the REHOME-to-
Captain alternative is independently sound; this dual-use, credential-
adjacent workflow does not become a Captain skill merely because its
stakes are high — PL-4's own execution-surface test governs, not risk
level.

**Stage/helper boundary — re-checked, no change.** The one PL-6 fold
(pin/lock into `50-test-in-disposable-copy`) is the only candidate;
independently re-applied the PL-5 reimplementation test to all seven
stages and found no stage that should instead fold into a neighbor, and
no additional deterministic-machinery candidate hiding inside any stage's
prose beyond the one already folded.

**Duplicated/drift-prone content — no new finding.** The seven stages'
identical "## Judgment required" boilerplate is already captured by
BU-VES-10; no further undisclosed duplication found.

**Package identity/naming — no issue.** `vet-external-skill` has no
existing collision (independently re-checked: no other `.sergeant/
workflows/` or `skills/` entry shares the name), and the name accurately
describes the package's scope (vetting, not general skill management).

**Unjustified engine gaps — none found.** No PL-7 claim is made anywhere
in the draft; the producer's explicit rejection of treating BU-VES-03/04/
08 (and, per this review, BU-VES-05/06) as engine gaps is independently
correct — each requires only stage-local content, not new runtime
capability.

## Overall verdict

**Final disposition: STAND — CONFIRMED**, with the producer's remediation
list (Surviving package design) requiring one correction before it is
treated as complete: items needing a genuinely new J0 clause are five
stages (`10-confirm-provenance`, `20-check-actions`,
`30-verify-no-conflict`, `50-test-in-disposable-copy`,
`60-update-managed`), not three, and BU-VES-13's supporting citation
should be corrected from `20-select-intent-transport/CONTEXT.md` to the
workflow-level `validate-and-ship/CONTEXT.md` before this record's
findings are treated as settled. Neither correction changes the package's
identity, its PL rungs, or its STAND disposition — both are in-place
amendments to the same five-item (now effectively six-item) remediation
list already scoped, consistent with ADR 0013 decision 6 (only the
promotable form of the eventual content change needs independent review;
this adjudication record itself is what that review step is for).
