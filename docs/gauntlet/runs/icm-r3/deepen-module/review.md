# Independent adversarial review: `deepen-module`

ICM-R3, `reference/proposal-icm-r-procedure-authority.md` §8.11 (this is
that step). Reviewer position only: no edit authority over the live
package (`.sergeant/workflows/deepen-module/`) or over the producer's
draft (`docs/gauntlet/runs/icm-r3/deepen-module/adjudication-draft.md`,
no `draft/` subdirectory exists for this package). Every claim below was
independently re-derived against the live package content, the upstream
source (`reference/sergeant-upstream/.agents/skills/codebase-design/
DEEPENING.md` and `DESIGN-IT-TWICE.md`), `reference-corpus/synthesis.md`
and `shared-context-map.md`, and the two ladders
(`reference/proposal-icm-r-procedure-authority.md` §5/§6,
`docs/icm/convention.md` §6) — not taken from the producer's own
citations.

Checklist applied per §8.11: source fidelity; rung order (PL and J);
Captain/workflow boundary; stage/helper boundary; authority grants and
missing J0 cases; package identity/naming; duplicated or drift-prone
content; false pairing assumptions; unjustified engine gaps.

## Per-unit verdicts

### BU-P4-013 -- verdict: CONFIRMED

Re-derivation: `CONTEXT.md` (L1) "Purpose"/"Trigger" and `index.md`
restate the same bounded-procedure claim; `DEEPENING.md` L1-3 supports it
verbatim in substance. PL-4 holds against §5.6's checklist (recognizable
trigger, bounded outcome, explicit inputs/outputs per stage, durable
Work-branch checkpoints, coherent envelope). J5 citing `workflow.toml`'s
stage-order fixation (`convention.md` §1 rule 4) is a defensible use of
"repository doctrine" even though the constraint is structural rather
than behavioral — J5's basis text explicitly includes "workflow
prohibition" and "stage contract requires," which covers a fixed stage
list.

### BU-P4-014 -- verdict: CONFIRMED

Verified verbatim against `DEEPENING.md` L11 ("Pure computation, in-memory
state, no I/O. Always deepenable — merge the modules and test through the
new interface directly. No adapter needed."). PL-5/J2 both hold. See
"Additional finding" below for a gap in this stage's classification
coverage that is not specific to this unit's own citation.

### BU-P4-015 -- verdict: CONFIRMED

Verified against `DEEPENING.md` L15. PL-5/J2 hold.

### BU-P4-016 -- verdict: CONFIRMED

Verified against `DEEPENING.md` L19, including the recommendation-shape
sentence the package's own contract paraphrases correctly ("logic lives
in one deep module; transport is swappable"). PL-5/J2 hold.

### BU-P4-017 -- verdict: CONFIRMED

Verified against `DEEPENING.md` L25. PL-5/J2 hold.

### BU-P4-018 -- verdict: CONFIRMED

Independently re-checked, not trusted from the draft: `DEEPENING.md` L29
("One adapter means a hypothetical seam. Two adapters means a real one...")
is absent from the live `00-classify-dependencies/CONTEXT.md` (confirmed
by direct read of that file — only the four dependency-category
statements are present, nothing resembling seam discipline). The
workflow-local classification is independently confirmed at
`reference-corpus/shared-context-map.md` line 347 and
`reference-corpus/synthesis.md` line 758 ("`deepen-module`: BU-P4-018
BU-P4-019 BU-P4-025" under "Workflow-local contexts (kept local per
§6.6)"), and `synthesis.md` line 489's 11-unit delivered list for W25
excludes all three of BU-P4-018/019/025 — an independent corroboration
that the corpus itself never actually delivered them, not just an
assertion repeated from the draft. PL-5/J2/FOLD all hold.

### BU-P4-019 -- verdict: CONFIRMED

Same verification as BU-P4-018, against `DEEPENING.md` L30. Confirmed
absent from the live `00-classify-dependencies/CONTEXT.md`.

### BU-P4-022 -- verdict: CONFIRMED

Verified against `DESIGN-IT-TWICE.md` L3. PL-5/J2 hold.

### BU-P4-023 -- verdict: NEEDS-REVISION

Source citation is accurate (`DESIGN-IT-TWICE.md` L17: "Show this to the
user, then immediately proceed to Step 2."). The **J1 rung is
inconsistent with the draft's own treatment of structurally identical
material** elsewhere in the same table. `DESIGN-IT-TWICE.md`'s directive
here is stated with the same unconditional, no-hedge imperative form as
`DEEPENING.md`'s testing-strategy directives, which the draft itself
classifies J5 for BU-P4-020/021 with the rationale "stated as an
unconditional discipline... not a case-by-case judgment call." "Show...
then immediately proceed... without waiting for a reply" is equally
unconditional in the upstream text — no hedge, no case-by-case carve-out.
J1's own definition requires the choice be "local... and cannot change
scope, authority... or another actor's contract," with examples limited
to things like variable names and equivalent-output formatter choices.
Whether the stage blocks on a live user reply before spawning sub-agents
is not cosmetic: the draft's own "Driver and admission boundary" section
leans on this exact unit as its load-bearing evidence that the stage is
PL-4/PL-5 rather than PL-2 ("the opposite of Captain-style live dialogue
gating on a user decision"). A unit doing that much classification work
cannot simultaneously be "local, reversible, non-contractual" — it is a
governing behavioral commitment the package's own PL argument depends on.
Re-derivation: this should be **J5** (parity with BU-P4-020/021) or, at
minimum, **J3** (a settled discriminator this producer's own admission-
boundary analysis already treats as authoritative), not J1. This does not
change the STAND disposition or the PL-5 placement, only the J-rung
citation, and should be corrected when `10-design-it-twice/CONTEXT.md`'s
`## Bounded judgment` section is drafted per the draft's own remediation
item 3.

### BU-P4-024 -- verdict: CONFIRMED

Verified against `DESIGN-IT-TWICE.md` L21 (Process step 2). PL-5/J2 hold.

### BU-P4-025 -- verdict: CONFIRMED

Verified against `DESIGN-IT-TWICE.md` L30 ("Include both [SKILL.md]
vocabulary and CONTEXT.md vocabulary in the brief so each sub-agent names
things consistently..."). Confirmed absent from the live
`10-design-it-twice/CONTEXT.md` (only file-paths/coupling/dependency-
category briefing content is present via BU-P4-023's neighboring text;
no vocabulary-consistency requirement). Corroborated via
`shared-context-map.md` line 347 and `synthesis.md` line 758/489 as with
BU-P4-018/019. PL-5/J2/FOLD hold.

### BU-P4-026 -- verdict: CONFIRMED

Verified against `DESIGN-IT-TWICE.md` L42 (Process step 3). PL-5/J2 hold.
The comparison axes (depth/locality/seam placement) and "opinionated
recommendation... hybrid if warranted... not a menu" language both match
verbatim in substance.

### BU-P4-020 -- verdict: CONFIRMED

Verified against `DEEPENING.md` L34 (Testing strategy: replace, don't
layer). J5 rung is well-supported — the source states this as an
unconditional discipline, matching J5's "stage contract requires or
forbids" basis exactly.

### BU-P4-021 -- verdict: CONFIRMED

Verified against `DEEPENING.md` L36-37. J5 rung holds for the same reason
as BU-P4-020.

### n/a (all three stages: `## Judgment required` boilerplate, no ADR 0013 `## Bounded judgment` shape) -- verdict: CONFIRMED

Independently verified by direct read of all three stage `CONTEXT.md`
files: each carries the identical generic "Judgment required" paragraph
and none names a J2 delegation class, a J1 local-choice boundary, or a
J0 escalation trigger. `convention.md` §6.1 and ADR 0013 decision 4 are
correctly cited as the governing requirement (J5, "omission is never
ambiguous"). STAND-with-amendment is the correct disposition — this is a
content gap in an otherwise correctly-placed package, not a placement
defect.

### n/a (`CONTEXT.md` L1, no `## Authority envelope` section) -- verdict: CONFIRMED

Independently verified: `.sergeant/workflows/deepen-module/CONTEXT.md`
has no such section. `convention.md` §6.1 correctly cited.

### n/a (dangling `provenance.md` reference, `CONTEXT.md` L28) -- verdict: CONFIRMED

Independently re-run, not trusted from the draft: `grep -l
provenance.md .sergeant/workflows/*/CONTEXT.md` was re-executed and
confirms the reference is systemic across the catalog, not specific to
this package (the actual archived provenance for every package lives at
`docs/gauntlet/promoted-provenance/<name>.md`, matching the draft's
claim). Correctly scoped out of this single-package pass.

## Additional finding — missing J0 case in `00-classify-dependencies` (new, not in the producer's table)

The producer's draft names exactly one new J0 gap (the contested-design-
recommendation case for `10-design-it-twice`) but the same reasoning
applies, unaddressed, to the earlier stage. `DEEPENING.md`'s four
dependency categories (In-process / Local-substitutable / Remote-but-
owned / True-external) are presented as a clean partition, and the
package's `Behavior contract` in `00-classify-dependencies/CONTEXT.md`
restates them the same way — but nothing in the upstream source or the
package addresses a candidate whose dependencies genuinely straddle two
categories (e.g., a module with both pure in-process logic and one
true-external call). The classification result is not cosmetic: it
determines whether a port is introduced at all, which is exactly the
kind of "downstream-binding effect... not a reversible local one" the
producer's own reasoning used to justify the `10-design-it-twice` J0
case (the design choice there binds what `20-test-at-new-interface`
commits tests against; the classification here binds whether
`10-design-it-twice` designs a port-bearing interface or a bare merge).
J0's own basis text names "evidence conflicts" as a direct trigger,
which a straddling/mixed-dependency candidate literally is.

Rungs checked: **J5** no constraint requires or forbids treating a
straddling candidate as any one category; **J4** no user/Work decision
addresses it; **J3** no settled record addresses it; **J2** the stage
delegates classification, but classifying among four mutually-exclusive
categories is not the same delegation as resolving a case where none
cleanly applies; **J1** does not apply, the choice is downstream-binding.
**Conclusion: J0.** When `00-classify-dependencies/CONTEXT.md`'s
`## Bounded judgment` section is drafted (the producer's own remediation
item 3), "the candidate's dependencies straddle more than one category
and no single classification is clearly dominant" should be named as a
`needs_input` trigger, the same way the producer already required for
`10-design-it-twice`'s contested-recommendation case.

## Checklist coverage not otherwise called out

- **Source fidelity:** all fourteen numbered behavior-unit citations
  (BU-P4-013/014/015/016/017/018/019/020/021/022/023/024/025/026)
  independently traced to `DEEPENING.md`/`DESIGN-IT-TWICE.md` and found
  verbatim-accurate in substance, including line numbers.
- **Captain/workflow boundary:** independently re-checked the rejected
  PL-2 alternative for `10-design-it-twice`. Agree with the producer's
  rejection — BU-P4-023's own text ("proceed immediately... without
  waiting for a reply") forecloses PL-2's live-dialogue-gating
  requirement, and the execution-surface test (`convention.md` §2a) holds
  for an already-identified deepening candidate. (Note the J-rung finding
  above does not disturb this — the unit is still correctly read as
  ruling out PL-2, only its own J-citation is off.)
- **Stage/helper boundary:** applied the reimplementation test (§5.7)
  independently to all three stages. Each produces a distinct artifact
  another stage depends on, and `20-test-at-new-interface`'s `promote`
  vs. the earlier two stages' `evidence` disposition is a materially
  different authority boundary. No merge/split case found.
- **Package identity/naming:** no issue. `deepen-module` is unambiguous
  and matches its own `index.md`/`workflow.toml`.
- **Duplicated or drift-prone content:** one additional observation, not
  requiring action by this pass — `CONTEXT.md` L2 still reads "Draft
  workflow package" while `index.md` states `status: published`. Checked
  for whether this is package-specific: `grep -l "^Draft workflow
  package" .sergeant/workflows/*/CONTEXT.md` returns 19 of the catalog's
  packages, all `status: published`, so this is systemic boilerplate
  drift, not a `deepen-module`-specific defect — same scoping logic the
  producer already applied to the `provenance.md` dangling reference.
  Recording it here so it isn't silently lost, not asking for a
  disposition change.
- **False pairing assumptions:** checked whether `10-design-it-twice`'s
  always-run seam/depth/locality comparison wrongly assumes a port
  decision is pending even when `00-classify-dependencies` concludes
  "in-process, no adapter." It does not — `DESIGN-IT-TWICE.md` is about
  interface design generally, and "seam placement" remains meaningful for
  an internal, portless seam. No false pairing found.
- **Unjustified engine gaps:** none claimed (no PL-7 rung anywhere in the
  draft) and independent review found no behavior in this package that
  the current engine cannot represent — sub-agent parallel spawn is an
  existing capability (Agent tool), not a missing runtime primitive.

## Overall verdict

**Final disposition: STAND — CONFIRMED**, with the producer's own
qualification upheld: the package is not yet authority-valid. This
review adds two items to the required remediation, both scoped to the
same in-place content amendment the producer already identified (drafting
each stage's `## Bounded judgment` section) rather than changing the
package's PL-4/PL-5 placement, disposition, or the STAND verdict itself:

1. `10-design-it-twice`'s future `## Bounded judgment` section should
   cite BU-P4-023 at **J5** (or J3), not J1 — see BU-P4-023 above.
2. `00-classify-dependencies`'s future `## Bounded judgment` section
   needs its own J0 clause for straddling/mixed-category dependency
   candidates, parallel to the one the producer already required for
   `10-design-it-twice`'s contested recommendation — see "Additional
   finding" above.

No behavior-unit's PL rung, disposition, or destination is disputed. No
citation was found to be fabricated, mismatched, or misquoted. The
package's identity, stage boundaries, and Captain/workflow classification
all independently re-derive to the same conclusions the producer reached.
