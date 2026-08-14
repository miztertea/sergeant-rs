# FOUNDATION-1 — refuter: invariants

Refuting `docs/gauntlet/runs/foundation-1/critics/invariants.md` against
`reference/proposal-foundation-rationalization.md`, per
`docs/gauntlet/contracts/FOUNDATION-1.md`. I did not write the proposal or
the critic findings. Independently re-verified every factual claim against
the repository; did not take critic evidence on trust.

---

## Finding 1: `inv-one-owner-relocated` — CONFIRMED, error severity holds

This is the unit's only error-severity finding, and the one I was asked to
attack hardest. Both legs of the claim were re-checked independently and
both hold.

**Leg (a): does any code path bind a `WorkSurface` to a branch it did not
mint?** Re-read `src/runtime/surface.rs` in full, not just the excerpts
quoted. `materialize`/`materialize_one` call `add_worktree`
(`src/runtime/surface.rs:636`), which always calls `add_worktree_from` with
`create_branch: true` (`:642`), running `git worktree add -b <branch> <path>
<start>` where `branch = work_branch(work_id) = "sergeant/{work_id}"`
(`:88-90`). `rematerialize` (`:438`) re-attaches to `binding.work_branch`,
which is the same `sergeant/<work-id>` string recorded at materialization —
never an externally supplied branch name. `teardown` (`:497`) retains "every
branch" but never rebinds one. There is no constructor, retry path, or
public function in this module that takes an arbitrary branch name and
attaches a surface to it. The critic's claim is correct: `WorkSurface`
mechanically only ever owns the branch it itself created from its own
`work-id`.

**Leg (b): does git actually refuse two worktrees on one branch?**
Reproduced directly rather than accepted on the critic's word:

```
$ git worktree add ../wt1 feature
Preparing worktree (checking out 'feature')
$ git worktree add ../wt2 feature
fatal: 'feature' is already used by worktree at '.../wt1'
```

Confirmed. This is not a corner case the critic invented; it is git's
ordinary worktree-exclusivity behavior, and it is exactly the constraint
that makes the proposal's silence on sequencing consequential rather than
academic.

**Does the proposal's own text supply an escape hatch the critic missed?**
Checked every use of "surface" and "branch" in the document (§4.2, §4.6,
§5.1, §5.2, §8.2, ADR 0005) for any statement that the gate Work operates
*on* the target Work's existing branch/surface rather than minting its own.
Found none. §5.1 says "The gate becomes a Work **with its own surface**"
and separately describes it "grading **another** Work's output" and
"reviewing a branch" — consistently two distinct Works, which by leg (a)
mechanically means two distinct `sergeant/<work-id>` branches. §8.2 asks
whether the review "stays independent," which presupposes the gate Work is
a separate reviewer of separately-produced output — the same two-Work
reading, not a same-surface one. Nothing in the document describes gating
as an additional stage folded into the *producing* Work's own pipeline
(which would trivially preserve one-owner by never needing a second
branch at all); if that were the intended design, §4.2 and §4.6 do not say
so, and §5.1's "grading another Work's output" language contradicts it.

**Does ADR 0005 itself contradict the finding?** Re-read in full. It states
plainly, in its own "Open questions" section: "The mechanics of how a gate
Work is specified — what workflow stage invokes
`scripts/gate.sh`/no-mistakes ... whether it is a new named ICM workflow or
a stage folded into an existing one — are not decided here; this ADR
records that gating becomes a dispatched Work, not the shape of that Work."
The critic's quote is verbatim and accurate. This directly contradicts
§4.2's "does not weaken this" and §4.6's "strengthens this rather than
bending it," both of which assert the mechanism as settled.

**Non-goals check.** This is not grading an unbuilt implementation (the
finding is about what §4.2/§4.6 *claim*, not about missing code), not
re-litigating ADR 0005's ruling (the finding accepts "gating becomes a
dispatched Work" as decided and only disputes an unsupported corollary
about *how* one-owner is preserved), and not designing the implementation
(the critic's proposed fix is to state the mechanism is open — matching
§8's own existing convention elsewhere in the same document — not to
specify a binding scheme).

**Severity.** The contract's bar for error is "a Work would be blocked or
produce a wrong artifact." Checked against both branches of the critic's
own (a)/(b) split: a Work implementing §5.1 as literally described — a gate
Work with its own freshly-minted surface reviewing an already-produced
branch — hits exactly the git refusal reproduced above the moment both
worktrees coexist, which blocks it; the alternative of cutting a copy
branch avoids the collision but silently breaks the `axi sync --recover`
custody-transfer step `docs/DEVELOPMENT.md`'s gate section documents
(re-verified: "`no-mistakes axi sync --recover` to take custody of pipeline
commits" — commits are recovered *onto the branch being shipped*, not onto
a disposable copy), which produces a wrong artifact (a gate Work that
"passes" its own copy while the actual shipping branch never received the
pipeline's fixes). Either path an implementer takes without addressing this
first produces a real failure, not a cosmetic one. Error severity stands;
not inflated.

**Verdict: CONFIRMED. Severity unchanged (error).**

---

## Finding 2: `inv-section4-cites-unsettled-mechanisms` — CONFIRMED, warning severity holds

Re-verified independently rather than accepted:

- ADR 0007's "Open questions" section, read in full: "The exact detection
  logic for (b) ... and what state a closing stage should report instead of
  plain `completed` ... is not specified here." Matches the critic's quote
  exactly. §4.3's "non-terminal state" is a specific detail the ADR does not
  commit to — a new terminal-but-flagged state (distinct from `completed`)
  would equally satisfy "must not land in plain `completed`" and would not
  be non-terminal. `src/runtime/surface.rs`'s own doc comments (`§12 makes
  retry the one door back out of failed, blocked and waiting`) confirm the
  codebase already distinguishes terminal (`completed`, presumably also a
  cancel-type terminal state) from non-terminal (`failed`, `blocked`,
  `waiting`) — so "non-terminal" is a real, checkable claim beyond what the
  ADR rules, not a distinction without a difference.
- §5.6's own text, re-read: "**Open, not decided:** whether the homepage is
  estate-aware ... or a static banner." §8.3, re-read: "The homepage's
  estate-awareness. §5.6, unruled." Both confirmed verbatim. §4.4's "reads a
  manifest" states one of these two options as the description of what the
  homepage does, three and seven paragraphs before the same document calls
  it undecided.

**Does the proposal already say elsewhere what this finding claims is
missing?** No — §8.2 addresses a different question (review independence),
not this one (which specific mechanism §4 is entitled to cite as the reason
an invariant holds). §5.6/§8.3 mark the homepage detail open but don't
retroactively soften §4.4's phrasing of it as accomplished fact; that
inconsistency is the finding.

**Non-goals / style check.** This is not a demand to pick one of the two
unruled options (not designing the implementation) and not a stylistic
preference — it is a specific, checkable claim that §4's argument names a
mechanism it doesn't have standing to name, contradicted by the same
document's own later sections. The critic explicitly considered a related
citation-mismatch issue (§4.3 citing the wrong `docs/DEVELOPMENT.md`
bullet for §5.5) and declined to file it once satisfied the underlying
behavior was still fail-closed under the rule it actually extends — that
restraint is itself evidence against a rubber-stamping failure mode, and I
found no basis to second-guess that call either.

**Severity.** Warning is right, not error: the critic's own analysis
that "under either resolution... the invariant actually holds" checks out
— I traced both options (non-terminal vs. terminal-but-flagged; manifest-read
vs. static banner) and in neither case does the underlying invariant
(single source of truth; usability not functionality) break. No basis to
upgrade to error; no basis to downgrade to none, since the argument defect
is real and independent of the safe conclusion.

**Verdict: CONFIRMED. Severity unchanged (warning).**

---

## Finding 3: `inv-ladder-incomplete-and-5.7-mislabeled` — CONFIRMED, warning severity holds

Re-verified independently:

- Grepped the full proposal for rung vocabulary (`rung`, `R1`–`R7`): only
  §5.1 ("reuses `validate-and-ship`"), §5.2 ("an `exec`, not a
  supervisor"), and §5.7 ("a deletion — the lowest rung available") are
  named in §4.7. §5.3, §5.4, §5.5, §5.6 have no rung citation anywhere in
  the document — the one other `R1` mention (line 353, "three
  hand-maintained renderings of one API for one human is R1 failure three
  ways") is part of §5.7's own reasoning about the dashboard, not a rung
  log for §5.3–§5.6. Confirmed as claimed.
- `reference/notes/ideaos-agent-contract.md`'s rung-logging convention, read
  directly: "every design decision in a ledger entry, every
  deviation-register row, and every new dependency, file, trait, or store
  records the rung it resolved at (`R1`–`R7`). An `R7` entry must name which
  lower rungs were checked and why they failed." `docs/DEVELOPMENT.md:87`
  confirms this is binding here: "Design decisions log their Ponytail
  rung." §5.6 (`sgt tui` as a new verb plus a new homepage renderer) is
  bespoke code with no rung citation and no R7 justification anywhere in
  the document — matches the claim exactly.
- ADR 0011, read directly: "Alternatives considered" compares full removal
  against "disabling the route and leaving the code in place," and its
  stated reason for the larger change is "a stub carrying two open issues
  indefinitely is a maintenance claim, and deletion commits to less than
  that stub would" — a maintenance-commitment argument, not a
  smaller-diff/more-reuse argument. Line counts independently verified:
  `src/web.rs` is 779 lines (`wc -l` confirms), matching ADR 0011's own
  "779 lines" and the proposal's "delete `src/web.rs` (779) + `web/`
  (224)." The ladder genuinely has no rung for "remove code" — R1–R7 in the
  source table order how much *new* machinery a change builds, per its own
  stated purpose ("blocks the jump from 'I understand the requirement' to
  'I should create a new abstraction'"). Calling a deletion "the lowest rung
  available" borrows vocabulary the ladder doesn't apply to net-negative
  changes.

**Does the proposal already address this elsewhere?** No section restates
rungs for §5.3–§5.6 anywhere else in the document; this is a genuine gap,
not a cross-reference the critic missed.

**Non-goals / style check.** Not a style preference — the rung-logging
convention is a binding, cited requirement (`docs/DEVELOPMENT.md:87`), and
the finding is that four of seven changes are silent against a rule the
document itself is subject to. Not designing an implementation — the fix is
additive labeling, not a redesign of §5.3–§5.6. Not re-litigating any of the
seven owner rulings — the finding does not argue any change was wrong, only
that the ladder bookkeeping the proposal itself invokes is incomplete and
that one entry uses the ladder's vocabulary for an argument the ladder
doesn't make.

**Severity.** Warning is correct, not error: missing rung citations are
additions the proposal can make without touching any decision's substance,
and §5.7's actual argument (stated in ADR 0011, maintenance-commitment) is
sound independent of the ladder-vocabulary slip — dropping "the lowest rung
available" costs nothing and fixes it. No basis to upgrade.

**Verdict: CONFIRMED. Severity unchanged (warning).**

---

## Summary

| Finding | Verdict | Severity |
|---|---|---|
| `inv-one-owner-relocated` | CONFIRMED | error (unchanged) |
| `inv-section4-cites-unsettled-mechanisms` | CONFIRMED | warning (unchanged) |
| `inv-ladder-incomplete-and-5.7-mislabeled` | CONFIRMED | warning (unchanged) |

All three findings survive adversarial refutation. I attempted to knock
each one down on its factual claims (re-ran the git worktree-collision
experiment myself in a scratch repo at `/var/tmp`, re-read
`src/runtime/surface.rs` end to end rather than trusting the critic's line
citations, re-read ADR 0005/0007/0011 and `docs/DEVELOPMENT.md` in full,
grepped the proposal directly for escape-hatch language), on scope (checked
each against all four non-goals), on style-vs-defect (checked whether any
finding was really "I would have written this differently" — none were),
on whether the document already says elsewhere what a finding claims is
missing (checked all eight sections for each finding; found none), and on
severity (traced the concrete failure mode Finding 1's error severity
implies, and separately confirmed Findings 2 and 3's warning severity is
not inflatable to error nor deflatable to nothing). None fell. The
unit's one error-severity finding — the specific target of this refuter's
attack — held up under direct reproduction of both legs of its claim.
