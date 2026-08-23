# Review Change

Layer 1 orientation only — never delivered as a stage's instructions;
each stage's own `CONTEXT.md` (Layer 2) is the actor's contract
(`docs/icm/convention.md` §1a rule 5).

## Purpose

Review a diff that arrives from outside a Work — a colleague's PR, a
merge candidate, a change someone else built — on four axes, and emit
verified findings, not fixes.

## Trigger

A diff needs review before merge.

## Stages

| Stage | Rung | Durable outcome |
|---|---|---|
| `00-pin-fixed-point` | actor-stage (§6.4, judgment) | The fixed point resolves and the diff is non-empty, or the run fails here rather than inside a seat. |
| `10-identify-spec-source` | actor-stage | The spec/acceptance the diff is judged against is located by a fixed priority order, or its absence is recorded — never invented. |
| `20-panel` | actor-stage | Four axis seats have reported; every finding is recorded at `status: raised`. |
| `25-refute` | actor-stage | Every raised finding carries a refuter verdict. |
| `30-verify-and-severity` | actor-stage | Each surviving finding is independently verified against current state (a finding may already be stale) and assigned `blocker`/`major`/`minor`. |
| `40-report` | actor-stage | The typed finding set is complete, with the panel's coverage stated. |

## Relationships to other workflows

This workflow recommends, and never dispatches, `remediate-findings` when
its report is authorized for action — the consumer that makes this
package's read-only separation pay. There is no child-workflow dispatch
and no worker-side submission (`docs/icm/convention.md` §7.5).

## Authority envelope

This workflow receives an already-admitted Work whose intent names a diff
to review.

### May decide
- Which fixed comparison point to confirm as valid, given the user's
  input (`00-pin-fixed-point`).
- Where to look for and how to select the spec source, within the fixed
  priority order (`10-identify-spec-source`).
- How to phrase each panel/refuter seat's brief within §2's fixed bounds
  (`20`, `25`).
- Whether a surviving finding is stale against current state, and its
  severity (`30-verify-and-severity`).

### May not decide
- Skip asking the user for the fixed point when none is given — J0,
  `00-pin-fixed-point`.
- Merge or re-rank the four axes against each other — J5, never merge.
- Edit the code under review at any point. **The reviewing actor never
  edits the code** (`@@independent-review`).

### Human or Captain gates
- Naming the fixed comparison point when the user did not supply one.
- Naming the spec source when none is discoverable by the fixed priority
  order.
- A refuter's verdict turning on a scope or policy question (`25-refute`).

### Decision record
Material decisions cite J-rungs inline in each stage's own output
artifact per `.sergeant/common/contexts/bounded-judgment.md` §Decision
evidence; the typed finding set (`output/findings.md`, carried through
`20`/`25`/`30`/`40`) is this workflow's central decision record.

## Robustness

**(a)** Six fresh executions, six journal checkpoints: a stall or
usage-window exhaustion costs one stage, not the run; the fixed point and
spec source banked at `00`/`10` are not re-derived on resume.

**(b)** `25-refute` attacks `20-panel`'s raised findings; `30-verify-and-
severity` attacks the refuted set again, against *current* state — a
finding may have already gone stale between panel and report, which
neither `20` nor `25` alone could catch.

**(c)** Failure behavior is the design record's §2.8 degradation table: a
panel seat that cannot complete degrades to fewer axes, named in the
report; a refuter seat that cannot complete leaves its axis's findings
unconfirmed. This workflow has no fixer stage, so a confirmed-but-severe
finding is never silently absorbed into a fix — it is reported, and
`remediate-findings` is recommended, never dispatched.

## Notes for reviewers

**Read-only by contract.** No fixer stage exists here; this is the
consumer-facing half of the same panel machinery `implement-change`
carries a fixer for. `@@independent-review` states the never-edit rule
this package's Authority envelope restates above.

**Descent.** Reshaped from `code-review` (v2, 4 stages, 2 axes).
`00-pin-fixed-point` and `10-identify-spec-source` carry over
substantially intact. `references/smell-baseline.md` moves under
`20-panel/references/smell-baseline.md` as the simplicity axis's Layer-3
reference — the only Layer-3 file this wave carries forward, because it
is genuinely stable and stage-local. The old two-axis names (Standards,
Spec) do not survive: `spec-fidelity` absorbs Spec, `simplicity` absorbs
Standards' smell half, and the repo-standard-overrides-baseline rule is
preserved verbatim in the simplicity axis's brief.
