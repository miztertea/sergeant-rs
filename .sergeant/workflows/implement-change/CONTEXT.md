# Implement Change

Layer 1 orientation only — never delivered as a stage's instructions;
each stage's own `CONTEXT.md` (Layer 2) is the actor's contract
(`.sergeant/common/contexts/icm-policy.md` §1a rule 5).

## Purpose

Take an intent for a change to code, produce the change with its tests,
attack it on four axes, refute the attack, fix only what survives,
re-attack the fixes, and close with evidence tied to acceptance.

## Trigger

A change to a registered repository is specified well enough to build,
and the Work should end with reviewed, evidence-backed commits.

## Stages

| Stage | Rung | Durable outcome |
|---|---|---|
| `00-orient` | actor-stage (§6.4, judgment) | The revision is pinned, the spec/acceptance source is located, and the change's boundary is stated in the actor's own words. |
| `05-baseline` | actor-stage | The pre-change state is recorded: which tests exist, which pass, which command runs them, and what the change is expected to move. |
| `10-implement` | actor-stage | The change exists as commits, built test-first per `@@test-first`. |
| `15-validate` | actor-stage | The targeted validation named at `05` has been run against the change, with its real output recorded — pass or fail. |
| `20-panel` | actor-stage | Four axis seats have reported; every finding is recorded in the typed set at `status: raised`. |
| `25-refute` | actor-stage | Every raised finding carries a refuter verdict; findings not overturned are `confirmed`, all others stay `refuted`. |
| `30-fix-confirmed` | actor-stage | Every `confirmed` finding is either fixed in a named commit or recorded as unfixed with a reason. Nothing else changed. |
| `35-re-verify` | actor-stage | The fixer's commits have been re-attacked and their tests audited; new findings are recorded and severity-ranked. |
| `40-close` | actor-stage | The evidence packet exists: what ran, against which revision, satisfying which acceptance criterion; every declared `promote` artifact is named and present. |

## Relationships to other workflows

This workflow recommends, and never dispatches, delegates to, or hands
off to, a downstream workflow. A stage that identifies further work
outside this change's scope records a recommended follow-up intent and
the workflow it recommends (most often `remediate-findings`, when a
finding is confirmed-but-deliberately-unfixed, or `review-change`, when a
colleague's later diff on the same area should be reviewed independently)
— Captain and the human authorize any of it. There is no child-workflow
dispatch and no worker-side submission (`.sergeant/common/contexts/icm-policy.md` §7.5;
design record §C.1–C.2).

## Authority envelope

This workflow receives an already-admitted Work intent (a change to
build).

### May decide
- The pinned revision's exact wording and the change's stated boundary
  (`00-orient`).
- Which existing tests and commands constitute the baseline (`05`).
- Implementation detail within the confirmed seams (`10`, per
  `@@test-first`).
- How to phrase each panel/refuter seat's brief within §2's fixed bounds
  (`20`, `25`).
- How to fix a confirmed finding, within `@@fix-confirmed`'s bounds
  (`30`).

### May not decide
- Whether to skip asking for a spec/acceptance source when none exists —
  J0 (`00`).
- Merging or re-ranking the four panel axes against each other — J5
  governing constraint, never merge.
- Fixing a `refuted` finding, or fixing anything beyond a `confirmed`
  finding's scope (`30`).
- Starting a second fix round after `35-re-verify` finds a new blocker —
  there is no loop primitive; a new blocker is a J0 escalation.

### Human or Captain gates
- No spec/acceptance source located at `00`, and the intent does not
  state acceptance either.
- A refuter's verdict turns on a scope, policy, or intentional-breaking-
  change question (`25`).
- A confirmed finding cannot be fixed within scope, or two fixes
  contradict (`30`).
- A new blocker at `35`.

### Decision record
Material decisions cite J-rungs inline in each stage's own output
artifact per `.sergeant/common/contexts/bounded-judgment.md` §Decision
evidence; the typed finding set (`output/findings.md`, carried and
updated across `20`/`25`/`35`) is this workflow's central decision record.

## Robustness

**(a) What each stage boundary buys under crash, stall, or usage-window
exhaustion.** Nine fresh executions, nine journal checkpoints: a stall or
usage-window exhaustion costs one stage, not the run, and the expensive
banked states are the baseline at `05` and the implementation at `10` —
neither has to be redone to resume at the next stage.

**(b) Which stage attacks the previous stages' output.** Four stages
attack earlier work: `20-panel` attacks the implementation, `25-refute`
attacks the panel, `35-re-verify` attacks the fix, and `15-validate`
attacks the claim that the change works at all.

**(c) What the workflow does when a stage cannot complete.** Failure
behavior is the design record's §2.8 degradation table, plus:
a `15-validate` failure is not routed around — the panel reviews a change
whose validation failed and the close packet records that rather than
silently waiting for a passing run.

## Notes for reviewers

`20-panel` through `35-re-verify` instantiate the shared panel machinery
(`@@fan-out-evidence`, `@@panel`, `@@refute`, `@@fix-confirmed`,
`@@re-verify`) rather than repeating its text; each stage's own
`## Behavior contract` narrows that machinery to this package's specific
inputs, per the design record §2's exact contract text for these four
stages. There is no stage-include mechanism in this engine (C.5): a
change to the shared contract must be hand-propagated here and re-read
against every consumer, which is drift by construction and is named
rather than hidden.

Panel budget (four seats plus four refuters, across two stages) is
unmeasured against the two-seat `code-review` precedent — this is a
ratify-at-review item on the head PR, not a settled claim this package
makes about its own reliability.
