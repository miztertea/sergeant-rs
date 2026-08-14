# FOUNDATION-1 — adjudication

Orchestrator adjudication of the panel and refuter output, 2026-08-14.
Contract: `docs/gauntlet/contracts/FOUNDATION-1.md`. Artifact:
`reference/proposal-foundation-rationalization.md`.

## Outcome: **validated with findings**

Every finding's own verdict, and every refuter's, holds that the affected
section survives a local correction. **No decision was invalidated and no
section was sent back.** The corrections have been applied to the proposal;
what was proved undecided is now carried as an Unknown rather than asserted.

## Verdicts

| Axis | Findings | Refuted | Confirmed | Severity moves |
|---|---|---|---|---|
| invariants | 3 | 0 | 3 | none — the unit's only surviving **error** |
| enactability | 4 | 0 | 4 | 2 × error → warning |
| assumptions | 3 | 0 | 3 | none |
| fidelity | 3 | **1** | 2 | 1 × warning → info |
| **total** | **13** | **1** | **12** | **3 downgrades** |

## The surviving error

`inv-one-owner-relocated` — §4.2 claimed §5.1 "replaces one ownership
mechanism (no-mistakes' repo-wide branch lock) with one sergeant already
enforces (a Work owns its surface)."

Those are different resources. no-mistakes locks *the branch under review*;
a sergeant surface guarantees *a freshly-minted `sergeant/<work-id>` branch
the Work itself creates*, and `src/runtime/surface.rs` has no path binding a
surface to a branch it did not mint. The refuter attacked both legs and both
held — it verified the surface code independently and **empirically tested
that git refuses two worktrees on one branch** rather than reasoning about
it.

Severity was challenged and defended. The failure is not cosmetic: reviewing
a copy breaks no-mistakes' recovery of auto-fix commits onto the shipped
branch, producing "a gate Work that *passes* its own copy while the actual
shipping branch never received the pipeline's fixes."

**Applied:** §4.2 rewritten to claim only what is true regardless of
mechanism, with the false equivalence recorded rather than deleted. New
**§8.6** carries the binding question, and §5.1 must not be dispatched until
it is answered.

## The refuted finding

`fidelity F2` — that §5.1 dropped ADR 0005's negative-consequence framing.
Refuted as *"style preference dressed as a fidelity defect, plus a fabricated
contract citation."* The critic invented a contract quote to support it.

Recorded because it is the panel working as designed: a critic reaching for
a finding, and the adversarial pass catching the fabrication. It is also the
single strongest argument for keeping refuters — without one, that finding
would have been adjudicated on a citation that does not exist.

Note the substance was nonetheless worth acting on: the proposal now carries
the cost of keeping no-mistakes embedded, and flags that "rebuild only where
matched" has no operational trigger. That came from enactability's F4, not
from the refuted finding.

## Corrections applied to the proposal

| Source | Section | Change |
|---|---|---|
| invariants F1 | §4.2, §4.6 | Claim narrowed to what holds regardless of mechanism; false equivalence recorded, not silently deleted |
| invariants F1 | §8.6 (new) | Gate-branch binding carried as an open question, gating §5.1's dispatch |
| enactability F1 | §6 | *(see below)* |
| enactability F2 | §5.2 | `exec` shape is decided; the *contents* of "the environment" are not, and a Work cannot build until §8.1 resolves |
| enactability F3 | §5.4, §8.7 (new) | Manifest-vs-env precedence named as undecided; a Work must not guess |
| enactability F5 | §5.7 | Two removal categories named with a checkable done-condition |
| fidelity F1 | §5.1 | `needs_input`/`sgt respond` marked one plausible mechanism, not a ruling |
| fidelity F3 | §5.1 | "Rebuild only where matched" flagged as posture, not trigger |
| assumptions F1 | §2.2 | "closed six issues" → fixed, landed on an unmerged branch, still open on the tracker |
| assumptions F3 | §5.5 | `m2 t7` removed — it does not support the blast-radius claim |

**§6 remains open, deliberately.** enactability F1 showed the sequencing
justification argues the opposite of what it orders: §5.3(a)'s environment
guarantee is *attributed to* §5.2, not manufactured by §5.3. The fix the
critic proposes — state which half of §5.3(a) can land before §5.2, or swap
the order — is a **sequencing decision**, and sequencing that touches what a
Work is allowed to assert is closer to a ruling than a correction. §5.3's
two halves are separable (the execution model can be stated today; the
environment guarantee cannot be stated honestly until §5.2 exists), but
choosing to split a decision's implementation across two Works is the
owner's call, not the orchestrator's. Carried to the owner rather than
resolved here.

## Method notes for the ledger

**The axes produced real findings on prose.** Contract Unknown 3 asked
whether a gauntlet built for code degrades into style commentary on a
document. It did not: 12 of 13 findings survived adversarial refutation, and
the strongest ones were mechanical facts about the repository — a git
constraint, a branch-minting code path, six issues' actual tracker state.

**The confirm rate was unusually high** (12/13 vs. M0's 2/3). The honest
explanation is not that the panel was lenient — two refuters downgraded
severities and one refuted a finding outright, each after independent
re-verification. It is that the artifact was written in a single pass with
no prior review, by the same session that then convened the panel. Every
Work reviewed on 2026-08-14 had a `tdd` stage, a `30-review` stage and a
shipping gate before the orchestrator saw it; this proposal had none.

**A specific line of attack changes refuter output.** The three axes where
the refuter was given a concrete thing to try — test the git constraint,
check whether §8.1 "hides" anything, distinguish dropping a cost from
summarizing — produced the unit's only refutation and all three downgrades.
Worth carrying into future refuter briefs.
