# Package adjudication review: research

Independent adversarial review, ICM-R2 pilot (`reference/proposal-icm-r-
procedure-authority.md` §8.11 / `docs/adr/0013-icm-r0-owner-rulings.md`
decision 7). Reviewer is independent of the producer who wrote
`adjudication-draft.md` in this same directory; this record challenges
that draft against the actual package content
(`.sergeant/workflows/research/`), the upstream source
(`reference/sergeant-upstream/.agents/skills/research/SKILL.md`), the two
ladders (proposal §5, §6), and `docs/icm/record-shapes.md` §6 — re-derived
independently, not taken from the producer's own citations. This review
has no edit authority over the live package or the producer's draft.

## `BU-P3-040` — verdict: CONFIRMED

Re-derivation: `reference/sergeant-upstream/.agents/skills/research/
SKILL.md` frontmatter `description:` matches the quoted behavior exactly.
It is workflow-identity framing, not a delegable decision — PL-4,
`driver: n/a` is the correct call; there is no adjacent rung it could
plausibly sit at instead (it names no trigger/stage boundary of its own).
No source-fidelity, rung-order, or authority-grant defect found.

## `BU-P3-041` — verdict: CONFIRMED

Re-derivation: SKILL.md body line 6 ("Spin up a **background agent** to
do the research, so you keep working while it reads") matches exactly.
The producer's framing — that *whether* to background this is a Captain-
side, pre-admission decision (proposal §5.4's PL-2 discriminator: "decides
whether work should remain direct or become durable Work") and not
itself a workflow behavior unit `research` owns — is correct and
consistent with the package's own admission-boundary claim ("in-work").
Correctly kept as non-actionable context rather than promoted into a
stage contract.

## `BU-P3-042` — verdict: CONFIRMED

Re-derivation: SKILL.md item 1, line 10 ("Investigate the question
against **primary sources** ... Follow every claim back to the source
that owns it") matches exactly. PL-5 (`00-investigate`) is correct under
the reimplementation test (§5.7) — source selection and citation-tracing
is exactly the kind of judgment call whose failure or evidence quality
operators would care about independent of implementation. J2 is the right
rung: the stage contract explicitly names the delegated decision class
("choose which primary sources are authoritative; trace every claim"),
satisfying §6.5's requirement that a J2 grant name a bounded decision
class, not just say "use judgment."

## `BU-P3-043` — verdict: NEEDS-REVISION

Source citation (SKILL.md item 2, line 11) is exact — no fidelity issue
there. The rung rationale is where this fails independent re-derivation.

The producer's own table cites, as PL-6's rationale, "no `kind =
"execute"` stage exists in the current engine" — carried forward
unchanged from the pre-existing `00-investigate/CONTEXT.md` text and N1
adjudication A4. That claim is **false as of this repository's current
state**, not merely stale: `.sergeant/workflows/repo-to-icm/workflow.toml`
already declares a `[stage."65-self-check"]` block with `kind =
"execute"`, adjudicated under N4 (2026-08-12) and implemented by the
`docker` backend (`src/backend/docker.rs`, which explicitly documents
itself as the runtime `kind = "execute"` stages route through). Whatever
was true when N1 adjudication A4 folded `10-write-findings` into a
helper, the engine capability the fold's own justification says doesn't
exist now does exist, live, in this same branch.

This directly reopens exactly the "stage/helper boundary" and
"unjustified engine gaps" questions §8.11 requires this review to
challenge. It does not automatically mean write-and-place should become
its own actor stage — `repo-to-icm`'s precedent is a narrow, mechanical
*validator* riding after an actor stage (`60-draft` produces content,
`65-self-check` deterministically checks it), not a replacement for
judgment-bearing work. But "every claim carries a source citation" is
exactly the kind of mechanically-checkable property `65-self-check`'s
precedent shows the engine can now verify deterministically (a linter
over the findings file, structurally analogous to
`validate-structure.py`), rather than being trusted entirely to one
actor's own unaudited self-check inside a folded helper. The producer's
"Surviving package design" section asserts flatly that "no behavior
unit's evidence argues for a different rung" — that assertion did not
hold up under re-derivation against current engine state, and the
Alternatives Considered section never raises or dismisses this option
(it only discusses whole-package REHOME and an unrelated engine
containment fix for B9, not a `kind = "execute"` citation-check for
`00-investigate`'s output).

Separately: the J1 disposition bundles two different things — (a) the
mechanical choice of "single Markdown file" (genuinely J1: local,
reversible, doesn't change acceptance), and (b) "every claim carries a
citation," which is restated verbatim as a gating condition in the
producer's own proposed Amendment 2 Completion boundary ("may complete
only when ... every claim in it carries a source citation"). A condition
that gates stage completion is not a private/reversible J1-style choice
by §6.6's own test ("cannot change scope ... acceptance"); it is either
inherited from `BU-P3-042`'s J2 grant (tracing to source) or deserves its
own explicit citation as such, not a bare J1 label on the combined
statement.

## `BU-P3-044` — verdict: DISPUTED

Source citation (SKILL.md item 3, line 12) is exact. PL-5 (judgment-
bearing placement choice — "choose a sensible location ... say where" is
not mechanical) is the more defensible rung on independent re-reading;
PL-6 (5.8) explicitly requires the behavior's "invocation does not itself
require substantive judgment," which placement-with-no-convention fails
by the producer's own admission in the same cell. So far this matches the
producer.

Where the draft is internally inconsistent: having assigned this unit a
*different* rung than its sibling `BU-P3-043` (PL-5 vs PL-6), the
producer still keeps both under the same destination — "the same folded
helper invocation" — without resolving what it means for a PL-5-rung
behavior to live inside a PL-6 helper block. This is the "false pairing
assumption" §8.11 asks reviewers to check for: the two units were paired
by N1 adjudication A4 at the *stage* level (the whole former
`10-write-findings` stage folded as one unit), not re-evaluated at
individual behavior-unit granularity — and at that finer granularity they
now carry different rungs.

The practical symptom: `BU-P3-044`'s content (placement choice) now
appears in **two** places in the amended `00-investigate/CONTEXT.md` —
once, unchanged, in the "Helper invocation: write findings" section, and
again, restated, in Amendment 2's new `## Bounded judgment` → `J2 —
delegated to this stage` list ("Choosing where the findings file is
placed ..."). (The same duplication affects `BU-P3-042`, which is stated
in both the existing "Behavior contract" section and restated in
Amendment 2's J2 list.) This is exactly the "duplicated or drift-prone
content" risk named in §8.11: two prose statements of the same delegated
authority, in the same file, with nothing tying them together — a future
edit to one is not guaranteed to reach the other. The Amendment should
either have the Bounded judgment section's J2 list *reference* the
existing Behavior contract / Helper invocation prose (by unit ID, as it
already does) rather than restate the requirement text, or the two
existing sections should be folded so there is one prose statement per
unit, not two.

## `BU-R2-045` — verdict: CONFIRMED

Re-derivation: `GAUNTLET.md`'s B9 row and `docs/adr/
0013-icm-r0-owner-rulings.md` decision 8 were re-read directly and match
the quoted text. PL-5 (the clause belongs inside `00-investigate`'s own
contract, not a new stage) and J0 (no higher rung resolves an unexpected
surface state; by §6.7's own basis, an out-of-worktree write is
definitionally "irreversible state" / scope-changing) are both correctly
derived, and FOLD is the right disposition modifier (§5.10: "unit becomes
context or a helper inside an owning package").

The producer's decision *not* to generalize this clause into
`@@bounded-judgment` itself is reasonable restraint — minting universal
policy from one package's pilot pass would overreach this review's own
scope, and the producer correctly defers that question to cross-package
synthesis rather than deciding it unilaterally. One citation defect in
that same paragraph, however: the producer attributes the "shared only
when two or more consumers use the same contract" rule to "`docs/icm/
convention.md` §2 rule 2." Re-checked directly: `convention.md` §2 is
"The draft publication boundary" (rules about `drafts/` vs `workflows/`
promotion) and has no rule numbered 2 matching this claim at all. The
actual rule is `reference/proposal-icm-r-procedure-authority.md` §5.10
("Shared/local is another modifier, not a rung. A method becomes shared
only when two or more consumers use the same contract"), with an
analogous local rule at `convention.md` §5 rule 3 for helper scripts
specifically. This is a source-fidelity miscitation — the underlying
reasoning survives re-derivation against the correct source, but the
citation itself does not resolve as written and should be corrected
before promotion.

## Overall verdict on Final disposition

**STAND is confirmed** as the package's identity/surface verdict — no
behavior unit's re-derivation argues for REHOME, SPLIT, HARVEST,
ABSORBED, or RETIRE, and the producer's REHOME-to-Captain-skill
alternative is correctly rejected on PL-2's own discriminator (research's
product is a durable artifact usable with Captain absent).

STAND does not mean the draft is ready to promote as-is, though. Three
concrete corrections are owed before Captain's reconcile-and-publish pass
(§8.12), all content-only and all inside this same package:

1. **Re-open the stage/helper boundary for `BU-P3-043`.** The "no `kind =
   execute` stage exists" premise the current fold rests on is false
   given `repo-to-icm`'s `65-self-check` precedent. At minimum, the
   Alternatives Considered section needs to explicitly evaluate and
   accept-or-reject a citation-completeness `kind = "execute"` check for
   `00-investigate`'s output, rather than silently carrying forward a
   premise that no longer holds.
2. **Resolve the `BU-P3-043`/`BU-P3-044` rung mismatch and destination
   duplication.** Either stop pairing them under one "Helper invocation"
   destination now that they carry different PL rungs, or make the
   pairing's rationale explicit. Deduplicate the restated J2/Behavior-
   contract prose so there is one authoritative statement per unit, with
   the Bounded judgment section citing rather than repeating it.
3. **Fix the `convention.md` §2 rule 2 miscitation** in Alternatives
   Considered — the correct source is `reference/proposal-icm-r-
   procedure-authority.md` §5.10 (and `convention.md` §5 rule 3 by
   analogy for helper scripts).

None of the three findings above invalidate `BU-R2-045`'s J0 clause
itself, which is well-evidenced and should proceed to Amendment 2 as
drafted; they bear on `BU-P3-043`/`BU-P3-044`'s representation and on one
citation, not on the package's surviving identity.
