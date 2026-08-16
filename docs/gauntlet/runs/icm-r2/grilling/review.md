# Independent adversarial review: grilling

ICM-R2 pilot, reviewer pass (`reference/proposal-icm-r-procedure-authority.md`
§8.11; `docs/adr/0013-icm-r0-owner-rulings.md` decision 7 — a separate
execution, explicit inputs, review-only contract, no edit authority).
Reviewing `docs/gauntlet/runs/icm-r2/grilling/adjudication-draft.md` against
the actual package content (`skills/grilling/SKILL.md`; confirmed no
`.sergeant/workflows/grilling/` exists, and no `draft/` mirror exists for
this package) — re-derived independently, not read off the producer's own
citations.

## Behavior-unit dispositions

### BU-R2-GRILL-01 — verdict: CONFIRMED

Re-derivation: `skills/grilling/SKILL.md` "## When to use" (lines 18-22,
verified against the live file) states the trigger and "never via `sgt
run`" directly. `NORTH-STAR.md` line 52 gives R-NS-6 verbatim: **"(execution
≠ dialogue)... Nothing conversational is ever engine work... Consequence:
the WORKFLOW-IF-E3 category is empty — grilling-class packages are operator
skills."** This is stronger and more direct support for PL-2/STAND than the
draft's paraphrase suggested — R-NS-6 names grilling-class packages
explicitly. PL-2 discriminator (§5.4: "if the procedure's job is to decide
what Work should exist, it cannot itself require an already-existing Work
merely to make that decision") holds. J5 is correctly cited: R-NS-6 is
repository doctrine forbidding dispatch as durable Work, and a lower rung
(e.g. a package preferring to run as a workflow) cannot override it.

### BU-R2-GRILL-02 — verdict: CONFIRMED

Re-derivation: "## How to grill" para 1, lines 26-29, verified verbatim
against the file. J2 basis (§6.5: "the active skill or stage explicitly
delegates this class of decision within named bounds") is met — the text
itself grants "offer your own recommended answer alongside each question"
as a named delegation, not a bare "use your best judgment."

### BU-R2-GRILL-03 — verdict: CONFIRMED

Re-derivation: bullet "One question at a time" at lines 31-32, verified.
J5 classification is defensible: §6.2's basis list includes "stage contract
requires or forbids the action," and this skill's own contract states the
prohibition in absolute terms ("Asking multiple questions at once is
bewildering") rather than delegating it as a judgment call — consistent
with treating a package's own explicit prohibition as governing, by the
same logic as the ladder's own example "review actors do not edit the
implementation under review."

### BU-R2-GRILL-04 — verdict: CONFIRMED

Re-derivation: bullet "Facts vs. decisions," lines 33-36, verified. J2 is
correct — the delegation is named and bounded (filesystem, `sgt doctor`,
tests, docs, `--help`, any available tool), matching §6.5's requirement
that delegation not be a bare "best judgment" grant.

### BU-R2-GRILL-05 — verdict: CONFIRMED (citation off by one line)

Re-derivation: the "Do not act on the plan" bullet actually begins at line
38, not line 37 as the draft's Source column states (line 37 is the tail
end of the preceding "Facts vs. decisions" bullet: "answer."). The
substance is unaffected — J5 as a governing constraint on when action may
be taken is well-supported by the bullet's own absolute language ("hard
gate before any implementation, `sgt run` submission, or file edit"). Minor
citation-precision defect, not a disposition problem; flagged per §8.11
"source fidelity."

### BU-R2-GRILL-06 — verdict: NEEDS-REVISION

Re-derivation: "## Failure behavior," lines 45-51, verified verbatim. The
*behavior* (degrade to best-guess and say so, rather than presenting an
unconfirmed guess as confirmed) is sound and correctly PL-2/J5. The
**evidentiary chain is broken**, and this is a genuine, not cosmetic,
source-fidelity problem (§8.11 first bullet):

- `docs/environments/cerberus.md` contains **no row or sentence** stating
  that "non-interactive turns cannot hold a mid-turn pause open" for the
  Claude CLI. Direct grep confirms no occurrence of "mid-turn," "pause," or
  "hold open" anywhere in that file.
- The Claude CLI row in `cerberus.md` that is the closest candidate concerns
  a **different mechanism**: `post_turn_summary` absence / `Capabilities::
  ask` withdrawal — Sergeant's own engine signal that lets a **dispatched
  workflow stage** (running under `sgt run`) tell the orchestrator it wants
  `needs_input`. The underlying measurement note
  (`docs/gauntlet/notes/cerberus-ask-grammar-remeasurement-2026-08-11.md`)
  is explicitly about `a5`'s `StageCompleted` vs `NeedsInput` derivation for
  a **stage's** completed turn, not about whether a live Captain
  conversation (this skill's actual execution context, per BU-R2-GRILL-01's
  own "never via `sgt run`" constraint) can hold a turn boundary open for
  the next human chat message.
- R-NS-6 itself (`NORTH-STAR.md` line 55) frames "whether a transport's
  actor can ask mid-run" as "a measured per-transport capability with
  runtime withdrawal" **in the context of a running execution** (`sgt`
  engine mechanics), which is precisely the `post_turn_summary`/
  `NeedsInput` machinery `cerberus.md` measures — not a statement about
  Captain-session chat turn-taking, which never goes through that
  machinery at all.
- The concern the "Failure behavior" section is actually gesturing at —
  an *unattended/headless* invocation of this skill with no live human on
  the other end of the conversation — is real and worth keeping (this
  review is itself being produced in exactly such a headless turn), but
  the citation offered does not establish it. The producer's draft
  reproduces this citation without checking it against the primary source,
  which is exactly the failure mode independent review exists to catch.

Recommendation: either (a) cite a source that actually measures Captain-
session turn-taking behavior in a headless/non-interactive invocation of
`claude`, or (b) rewrite the "Failure behavior" trigger condition to name
the real distinguishing fact directly ("this invocation has no live human
who will send the next message," which is self-evident from the harness's
own execution mode and does not need `cerberus.md` at all) rather than
borrowing an engine-mechanics measurement that answers a different
question. This is a defect in the live `SKILL.md` prose the draft
reproduces uncritically, not something the draft itself introduced — but
the draft's Source column presents it as verified evidence-backed fact
("citing `docs/environments/cerberus.md`... this is the concrete
evidence-backed rule") when it is not.

### BU-R2-GRILL-07 — verdict: NEEDS-REVISION (finding confirmed, record-shape defect)

Re-derivation: the gap is real — `skills/grilling/SKILL.md` has no `##
Bounded judgment` heading anywhere (verified by direct read of the full
file). The requirement is correctly sourced and is in fact **stronger**
than the draft's own citation suggests: `docs/adr/0013-icm-r0-owner-rulings.
md` decision 4 literally says "every actor **stage**," but
`docs/icm/convention.md` §6.1 (which the draft also cites) explicitly
extends this: "Every Captain skill's `SKILL.md` carries the same
conceptual section adapted to its driver" — and
`reference/proposal-icm-r-procedure-authority.md` §7.4 independently
confirms this a third way ("Every skill gains the same conceptual
section..."). Three converging sources, not one ambiguous one; the finding
stands as CONFIRMED on the merits.

What is flagged: `docs/icm/record-shapes.md` §6 rule 3 requires the
`Disposition` column to "use the modifier vocabulary (STAND, REHOME, SPLIT,
HARVEST, ABSORBED, FOLD, RETIRE)... not free text." The draft's table uses
`**gap — not present**` in that column for this row, which is free text,
not one of the seven modifiers. The correct disposition value for an
in-place content amendment to a package that otherwise stands is `STAND`
(the draft's own "Surviving package design" and "Final disposition"
sections already reach exactly this conclusion in prose) — the table cell
should say `STAND`, with the gap itself carried in a rationale/description
column or accompanying prose, not invented as a new disposition value
outside the canonical vocabulary. Mechanical fix, does not change the
underlying verdict.

## Additional §8.11 checks (no separate finding)

- **Rung order / PL and J:** independently walked PL-0 through PL-2 for
  the package as a whole — PL-0 does not apply (no absorbed/obsolete
  behavior; the prior workflow identity is confirmed retired, not merely
  deprecated), PL-1 does not apply (this is trigger/stage-specific, not a
  broadly-applicable stable rule), PL-2 holds on first check per §5.4's
  discriminator and R-NS-6's explicit naming. Order was correctly applied,
  stopping at the first rung that holds.
- **Captain/workflow boundary:** confirmed correctly drawn. No residual
  `CONTEXT.md`/stage/`workflow.toml` language found in `SKILL.md`; its own
  framing narrates the retirement as history, not as live mechanism.
- **Package identity/naming:** `grilling` name is unambiguous, no
  collision found (`find` for the name turns up only the live skill, its
  provenance file, this review's own directory, upstream reference copies,
  a dogfood journal, and one stale draft-workflow artifact under
  `docs/gauntlet/runs/n2-run4/.sergeant/drafts/workflows/grilling` — the
  last is historical gauntlet-run output, not a live competing package).
- **Duplicated/drift-prone content:** `grill-with-docs` delegates to
  `grilling` by reference ("Load the `grilling` skill and run its
  interview") rather than restating its rules — confirmed by direct read
  of `skills/grill-with-docs/SKILL.md`; no drift-prone duplication found.
- **False pairing assumptions:** none found — `grilling`/`grill-with-docs`
  is a real, evidenced pairing (`docs/icm/re-homing-record-2026-08-12.md`
  rows 32-33), not an assumed one.
- **Unjustified engine gaps:** none claimed by this package; nothing to
  challenge.
- **Authority grants and missing J0 cases:** no J0 case was identified as
  missing. The package's own hard gates (units 03, 05, 06) are correctly
  J5, not silently downgraded to J2/J1, and none of the six substantive
  units describes a decision that should have surfaced as
  `needs_input`-equivalent inside a live Captain session — Captain skills
  ask the user live per §6.7's own rule ("For a Captain skill, ask the
  question live and wait for the user's answer"), which is exactly what
  units 03/05 already require.

## Overall verdict on Final disposition

**STAND — CONFIRMED**, with the one required content amendment (the `##
Bounded judgment` section) also confirmed as required, and one correction
to how the table should record it (see BU-R2-GRILL-07 above: the
Disposition cell should read `STAND`, not free text).

Independent re-derivation from the package content and the two ladders
reaches the same placement as the producer: `skills/grilling/SKILL.md` is a
correctly-scoped PL-2 Captain skill, its six substantive behavior units are
each independently traceable to current source text with correct J-rung
citations (one line-number drift, immaterial), REHOME/SPLIT/HARVEST/FOLD/
RETIRE are each correctly rejected on the same grounds the draft gives, and
no destination other than the current file is warranted. The one place this
review disagrees with the draft's own characterization is BU-R2-GRILL-06:
the draft presents its `cerberus.md` citation as settled, verified,
"evidence-backed" fact, when independent inspection of the primary source
shows the citation does not establish the claim it is offered for. This
does not change the Final disposition — the underlying failure-behavior
*rule* is still sound and still belongs in the package — but the record
should not carry it forward as verified when it has not actually been
verified against the source it names.
