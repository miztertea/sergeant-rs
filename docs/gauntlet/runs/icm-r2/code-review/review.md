# Independent adversarial review: code-review package adjudication

ICM-R2 pilot, independent reviewer position (`docs/adr/0013-icm-r0-owner-rulings.md`
decision 7; `reference/proposal-icm-r-procedure-authority.md` §8.11). Fresh
execution, no edit authority over the live package or the producer's draft.
Every disposition below is re-derived directly against
`.sergeant/workflows/code-review/` (live),
`docs/gauntlet/runs/icm-r2/code-review/draft/` (proposed), the upstream
source `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md`,
and the reference-corpus classification records
(`reference-corpus/behavior-units/P2.ndjson`, `helper-map.md`,
`shared-context-map.md`, `synthesis.md`, `classification-ledger.md`) — not
from the producer's own citations, per §8.11's challenge list (source
fidelity, rung order, Captain/workflow boundary, stage/helper boundary,
authority grants and missing J0 cases, package identity/naming, duplicated
or drift-prone content, false pairing assumptions, unjustified engine
gaps).

Producer draft under review:
`docs/gauntlet/runs/icm-r2/code-review/adjudication-draft.md`.

## Behavior-unit dispositions

### BU-P2-004 -- verdict: CONFIRMED

Statement and citation (SKILL.md Step 1, line 19) verified verbatim against
the upstream file. PL-5/J0 is the correct rung: this is a stage-level
checkpoint gated on an explicit "ask, don't guess" rule, matching J0's
canonical shape. The draft's `00-pin-fixed-point/CONTEXT.md` carries the
unit in both its Behavior contract and a real `## Bounded judgment` → `J0`
section ("stop and ask rather than guessing `HEAD~1`, `main`, or any other
default"). STAND confirmed.

### BU-P2-005 -- verdict: CONFIRMED

`reference-corpus/helper-map.md` line 332 independently confirms this unit
was classified `helper`/workflow-local and destined for `code-review`,
never materialized in the live package (`.sergeant/workflows/code-review/`
has no `scripts/` directory at all). `classification-ledger.md` has no
entry adjudicating it out — the "silently dropped, not adjudicated"
finding is real. The draft's `scripts/capture-diff.sh` matches the source's
three-dot diff form and commit-list capture (SKILL.md line 21) and is
correctly scoped workflow-local per `docs/icm/convention.md` §5 rule 3
(single consumer). PL-6/J1 is correct: this is deterministic git mechanics
folded into `00-pin-fixed-point`'s existing checkpoint rather than promoted
to its own `kind = "execute"` stage — consistent with the same demotion
pattern used elsewhere in this repo (`vet-external-skill`, `to-tickets`,
`triage`, `wayfinder`) where diff-capture-shaped machinery adds no
independent retry/failure/measurement boundary beyond the checkpoint it
already belongs to. HARVEST confirmed.

### BU-P2-006 -- verdict: CONFIRMED

Citation verified (SKILL.md line 23). Already present in the live package
and in the draft, now backed by a real `J2 — delegated to this stage` entry
("Confirm the fixed point resolves... before proceeding"). STAND confirmed.

### BU-P2-007 -- verdict: CONFIRMED

Citation verified against the four-step priority order at SKILL.md lines
27-32. Already present in the live package; draft adds a proper `Bounded
judgment` section splitting the walk itself (J2) from the terminal ask
(J0), plus a correct cross-reference explaining that the recorded "no
spec" answer is what lets the merged review stage skip the Spec sub-agent
at J4 rather than re-asking. STAND confirmed.

### BU-P2-001 -- verdict: CONFIRMED

Independently re-read: SKILL.md lines 8-9 sit in the body's opening
two-bullet axis definition, not literal YAML front matter (front matter is
lines 1-4) — the producer's "front matter" locator is loose but the line
numbers and quoted content are correct, so this does not change the
finding. Correctly identified as workflow-level scope, not
stage-20-specific: the live package's `20-parallel-review-standards/
CONTEXT.md` does carry it as a stage "must-do" bullet today. PL-4/HARVEST
to package `CONTEXT.md` Purpose is the right call — a workflow's own axis
definition is a Purpose-level fact, not a single stage's job. Verified the
draft package `CONTEXT.md` Purpose carries it (with citation) and it is
absent from `20-30-parallel-review/CONTEXT.md`'s Behavior contract.
HARVEST confirmed, cleanly executed.

### BU-P2-002 -- verdict: NEEDS-REVISION

Source fidelity and rung classification confirmed identical to BU-P2-001's
reasoning (same "front matter" locator looseness, same PL-4/HARVEST
rationale, correctly "misfiled by axis" in the live package where it sits
inside the *Standards* stage despite defining the *Spec* axis). The
disposition table's stated destination is singular: "package `CONTEXT.md`
Purpose." That part is done correctly — the draft package `CONTEXT.md`
Purpose does carry `BU-P2-002`.

But independent inspection of the full draft tree (not just the table)
found the unit is **also** still present, uncredited by the table, in
`docs/gauntlet/runs/icm-r2/code-review/draft/40-aggregate/CONTEXT.md`'s
Behavior contract, as its first bullet — content that was not in the live
package's `40-aggregate/CONTEXT.md` at all (checked directly: the live
file has only `BU-P2-016/017/018`) and is not the sibling unit `BU-P2-001`
(which correctly appears nowhere outside the package `CONTEXT.md`). This
is a real instance of exactly what §8.11 asks a reviewer to hunt for —
duplicated, drift-prone content — introduced during this same revision,
not inherited from the live package, and it survived the producer's own
§8.10 self-check ("every behavior unit is dispositioned... citations
resolve"), which the "Behavior-unit dispositions" table's single-location
destination for `BU-P2-002` does not actually match the tree it
describes. Contrast with `BU-P2-009`, whose table entry explicitly names
*two* destinations ("package `CONTEXT.md` Authority envelope... referenced
again from the merged review stage") — a deliberate, documented dual
citation. `BU-P2-002`'s table entry names one destination but the tree has
two, undocumented. Before promotion, either remove the stray bullet from
`40-aggregate/CONTEXT.md` or update the table to declare the dual location
and its rationale (mirroring `BU-P2-009`'s pattern).

### BU-P2-008 -- verdict: CONFIRMED

`reference-corpus/shared-context-map.md` line 343 independently confirms
`helper`/shared-context classification destined for `code-review`, absent
from the live package. Draft's `references/smell-baseline.md` carries it
verbatim with citation, correctly scoped workflow-local (single consumer,
`docs/icm/convention.md` §4 rule 1 — no second workflow shares this
contract). HARVEST confirmed.

### BU-P2-009 -- verdict: CONFIRMED

Citation verified (SKILL.md line 40). PL-5/J5 is correct — "the repo
overrides" is a governing constraint, not a delegated judgment call;
misclassifying it as J2 would let an actor treat the override as
optional. FOLD to package `CONTEXT.md` Authority envelope, restated inside
the merged review stage, is deliberately dual and both locations verified
present in the draft. Confirmed as designed.

### BU-P2-010 -- verdict: CONFIRMED

Citation verified (SKILL.md line 41). Present in the live package and
retained correctly in the draft's merged `20-30-parallel-review/
CONTEXT.md`, now with a `J2 — delegated to this stage` entry ("Judging
which baseline smells are worth reporting..."). STAND confirmed.

### BU-P2-011 -- verdict: NEEDS-REVISION (rung terminology only)

`reference-corpus/helper-map.md` line 332 confirms this unit (the twelve-
item smell list itself) was classified alongside BU-P2-005/012 as
`helper`/workflow-local, absent from the live package. The draft's
destination (`references/smell-baseline.md`, merged with BU-P2-008) is
correct and matches what the corpus already earmarked. But the producer's
own table entry labels the rung "PL-3 (workflow-local shared-helper
content)" — mixing PL-3 (actor skill / shared method, §5.5) with "helper"
terminology that `docs/icm/convention.md` §5 defines specifically as
*deterministic machinery*, which prose reference content is not. This is
carried over from the reference-corpus's own inconsistent map naming
(`helper-map.md` calls it a helper; it is not one by the PL ladder's own
rung definitions) and should read PL-3 shared-context, full stop, to avoid
implying a script-shaped artifact. Destination and disposition (HARVEST)
are unaffected — this is a citation-hygiene fix, not a structural one.

### BU-P2-003 -- verdict: CONFIRMED

Citation verified (SKILL.md line 11: "Both axes run as **parallel
sub-agents**... then this skill aggregates their findings"). Independently
confirmed the live package's split into `20-parallel-review-standards` →
`30-parallel-review-spec` cannot represent single-message concurrent
dispatch: `30-parallel-review-spec/CONTEXT.md`'s own Inputs table names
`../20-parallel-review-standards/output/README.md` as an L4 dependency —
this is a real sequential ordering edge in the live package, verified
directly, not merely alleged. Also independently confirmed (`grep parallel
.sergeant/workflows/*/workflow.toml`) that no `workflow.toml` in this
repository declares a concurrent-stage primitive — the engine has no way
to represent "two stages, one concurrent checkpoint" other than merging
them into one stage, so PL-5-as-single-merged-stage is the only rung that
can faithfully carry this behavior; no engine-gap claim is warranted
(§8.11's "unjustified engine gaps" check — none raised, correctly). The
draft's merged `20-30-parallel-review/CONTEXT.md` carries the unit with an
explicit two-call, single-message framing. HARVEST confirmed.

### BU-P2-012 -- verdict: CONFIRMED

Citation verified (SKILL.md line 60: "Send a single message with two
`Agent` tool calls"). `reference-corpus/behavior-units/P2.ndjson`
independently confirms the statement text; `helper-map.md` line 332
confirms prior classification. The producer's claim that the reference
corpus's own classification record already names this stage
`20-30-parallel-review` is corroborated structurally: `helper-map.md` and
`shared-context-map.md` both use the `W24 code-review` package label
consistent with one merged concurrent-dispatch checkpoint, not two. HARVEST
confirmed — genuinely a promotion-execution defect in the live package
(split into two stages), not a reason to change the package's PL-4 rung.

### BU-P2-013 -- verdict: CONFIRMED

Citation verified (SKILL.md lines 62-66, including the "the sub-agent has
no other access to it" phrase quoted accurately). Independently confirmed
the live package's defect: `20-parallel-review-standards/CONTEXT.md` (the
Standards actor) has no sub-agent-spawning instructions at all, while
`30-parallel-review-spec/CONTEXT.md` carries this Standards-prompt-assembly
unit under a stage whose own stated purpose is "an isolated review against
the identified spec source" — confirmed by direct read of both live files.
The draft's merged `20-30-parallel-review/CONTEXT.md` resolves this by
filing it alongside `BU-P2-014` in the one stage that now owns both
spawns, and correctly points to `../references/smell-baseline.md` for the
"full smell baseline" the source demands be pasted in. HARVEST confirmed.

### BU-P2-014 -- verdict: CONFIRMED

Citation verified (SKILL.md lines 68-72). Correctly filed under stage 30 in
the live package already; folds into the merged stage unchanged in the
draft. HARVEST confirmed.

### BU-P2-015 -- verdict: CONFIRMED

Citation verified (SKILL.md line 74). J4 boundary is correct — this
consumes stage 10's already-recorded "no spec" decision rather than
reopening it; the draft's merged stage Completion boundary cites this
correctly ("has either reported or been explicitly skipped per stage 10's
recorded 'no spec' answer... J4"). HARVEST confirmed.

### BU-P2-016 -- verdict: CONFIRMED

Citation verified (SKILL.md line 78). J5 rung correct: "never merge or
rerank" is a governing constraint, not a per-run judgment call — misfiling
it as J2 would let an actor treat separation as optional under pressure to
produce one verdict. Present in both the live and draft `40-aggregate/
CONTEXT.md`, and now also restated in the draft package `CONTEXT.md`
Authority envelope's "Workflow may not decide" list — a deliberate,
documented second location analogous to BU-P2-009's pattern, not an
undocumented duplicate. STAND confirmed.

### BU-P2-017 -- verdict: CONFIRMED

Citation verified (SKILL.md line 80). Present in the live package and
retained in the draft with a `J2` entry for the closing summary. STAND
confirmed.

### BU-P2-018 -- verdict: CONFIRMED

Citation verified (SKILL.md "Why two axes", lines 84-87). Correctly
n/a for J-rung (design rationale, not a decision point). Present in the
live and draft `40-aggregate/CONTEXT.md`, plus referenced in both the live
and draft package `CONTEXT.md` Notes. STAND confirmed.

## Structural findings (re-derived independently)

1. **No `## Authority envelope`.** Confirmed directly: live
   `.sergeant/workflows/code-review/CONTEXT.md` has no such heading.
   `docs/icm/convention.md` §6.1 requires it on every workflow's Layer-1
   `CONTEXT.md`. The draft adds it with `### Workflow may decide` / `may
   not decide` / `Human or Captain gates` / `Decision record` subsections,
   matching §6.1's required content exactly. Confirmed fixed.

2. **No `## Bounded judgment` on any stage; identical boilerplate
   instead.** Confirmed directly: all five live stage `CONTEXT.md` files
   carry the exact same "## Judgment required... This is an actor stage
   (ladder §6.4)..." paragraph, byte-for-byte identical across files
   (diffed independently). `docs/icm/convention.md` §6.1 requires a
   `## Bounded judgment` section with J2/J1/J0/completion-boundary/
   decision-recording content, always present. The draft's four stages
   each carry a real, stage-specific `## Bounded judgment` section citing
   `@@bounded-judgment` with distinct J2/J1/J0 content per stage — verified
   these are not copy-pasted identically across stages (each names
   different delegated classes and different J0 triggers). Confirmed
   fixed.

3. **Broken `provenance.md` self-reference.** Confirmed directly: live
   `CONTEXT.md` says "See `provenance.md`..."; no such file exists under
   `.sergeant/workflows/code-review/`; the real file is
   `docs/gauntlet/promoted-provenance/code-review.md`. The draft corrects
   the pointer to the real path in both `CONTEXT.md` and `index.md`.
   Confirmed fixed.

4. **Review independence note.** Confirmed no in-package violation: none
   of the five (four, post-merge) stages reviews or promotes this
   package's own output — the package's job is reviewing *external*
   diffs. The independence requirement correctly resolves at the ICM-R2
   pilot level (this review being that independent execution), per
   `docs/icm/convention.md` §6.3. No finding.

## New finding not identified by the producer

5. **Duplicated `BU-P2-002` citation** (see BU-P2-002 above). The draft's
   `40-aggregate/CONTEXT.md` retains a Behavior-contract bullet for
   `BU-P2-002` that the disposition table does not account for as a second
   destination, unlike the analogous and correctly-documented dual
   citation for `BU-P2-009`/`BU-P2-016`. This is exactly the class of
   defect §8.11 asks an independent reviewer to hunt for and the
   producer's own §8.10 self-check did not catch. Must be resolved
   (delete the stray bullet, or document the second location with
   rationale) before this record is promotable — the promotion policy's
   own acceptance criterion ("every behavior unit above is dispositioned
   with a resolving citation") is not currently met as written.

## Overall verdict on Final disposition

**STAND — CONFIRMED**, with one required fix before promotion.

Independently re-derived from the units above: package identity, the PL-4
workflow rung, and the trigger/outcome/driver classification all hold
under direct re-inspection of `.sergeant/workflows/code-review/` and the
upstream source — nothing here shows `code-review` belongs on a different
surface or should be split/rehomed. The `REHOME`/`SPLIT` alternatives in
the producer's "Alternatives considered" section are correctly rejected
for the same reason: every defect found (stage-merge fidelity, missing
Authority envelope, boilerplate Bounded-judgment sections, a broken
pointer, and now the BU-P2-002 duplication) is an internal-fidelity defect
against the package's own already-adjudicated source classification, not
evidence the package is misplaced.

This STAND verdict is not, however, evidence that the draft is ready to
promote as-is. Finding 5 above is a genuine, previously uncaught defect;
per this record's own "Review and promotion policy" acceptance criterion,
the disposition table's citations must actually resolve against the full
draft tree, and `BU-P2-002`'s currently do not. Recommend: producer
resolves finding 5 (and, optionally, the BU-P2-011 rung-terminology
cleanup, which is cosmetic and does not block promotion on its own), then
this record is promotable under Captain's reconcile-and-publish pass
(§8.12).
