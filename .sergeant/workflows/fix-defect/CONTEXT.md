# Fix Defect

Layer 1 orientation only — never delivered as a stage's instructions;
each stage's own `CONTEXT.md` (Layer 2) is the actor's contract
(`docs/icm/convention.md` §1a rule 5).

## Purpose

Reproduce a defect before touching it, prove the cause, fix it with a
regression test, and put the fix through the same review chain a feature
change gets (R-S0-12: a fix is code).

## Trigger

"Diagnose"/"debug this", or something reported broken, throwing, failing,
slow.

## Stages

| Stage | Rung | Durable outcome |
|---|---|---|
| `00-build-feedback-loop` | actor-stage | The failure is observable on demand: a command, a fixture, a log path. |
| `10-reproduce-and-minimize` | actor-stage, hard gate | The defect reproduces, minimized to the smallest input that still shows it. No edit to the subject happens before this. |
| `20-hypothesize` | actor-stage | A causal hypothesis is written down before instrumenting, with what would disprove it. |
| `30-instrument` | actor-stage | Evidence gathered against the hypothesis; the hypothesis is confirmed, corrected, or replaced. |
| `40-fix-with-regression-test` | actor-stage | The regression test exists and fails against the unfixed code first; then the fix; then it passes. |
| `50-panel` | actor-stage | Four axis seats have reported on the fix; every finding is recorded at `status: raised`. |
| `55-refute` | actor-stage | Every raised finding carries a refuter verdict. |
| `60-re-verify-and-postmortem` | actor-stage | The fix commits are re-attacked and their tests audited; the root-cause record exists: what was actually wrong, how it got in, what would have caught it. |

## Relationships to other workflows

This workflow recommends, and never dispatches, an architectural follow-up
(most often as an `implement-change` or `investigate` recommendation) when
`60-re-verify-and-postmortem` finds the fix implicates a structural gap —
no good test seam, tangled callers, hidden coupling. There is no
child-workflow dispatch and no worker-side submission
(`docs/icm/convention.md` §7.5).

## Authority envelope

This workflow receives an already-admitted Work whose intent names a
reported defect.

### May decide
- Which feedback-loop construction strategy to attempt first, and how to
  tighten it (`00`).
- What to cut while minimizing, and in what order (`10`).
- Generating and ranking falsifiable hypotheses (`20`).
- Which instrumentation tool to reach for first (`30`).
- Judging whether a candidate regression-test seam is load-bearing or too
  shallow (`40`).
- How to phrase each panel/refuter seat's brief within §2's fixed bounds
  (`50`, `55`).

### May not decide
- Proceed past `10-reproduce-and-minimize` without both reproducing and
  minimizing — a governing completion gate, not a discretionary choice.
- Proceed past `00` without a red-capable loop, or hypothesize before one
  exists.
- Merge or re-rank the four panel axes against each other — J5, never
  merge.
- Start a second fix round after `60-re-verify-and-postmortem` finds a new
  blocker — there is no loop primitive; a new blocker is a J0 escalation.

### Human or Captain gates
- No feedback loop can genuinely be built (`00`).
- Two hypotheses are equally supported and distinguishing them needs a
  decision only the human can make (`20`).
- The fix requires a scope or policy change (`40`).
- A refuter's verdict turns on a scope, policy, or intentional-breaking-
  change question (`55`).
- A new blocker at `60`.

### Decision record
Material decisions cite J-rungs inline in each stage's own output
artifact per `.sergeant/common/contexts/bounded-judgment.md` §Decision
evidence; the typed finding set (carried through `50`/`55`) and the
root-cause postmortem (`60`) are this workflow's central decision records.

## Robustness

**(a)** Eight checkpoints; the expensive part — reproduction — is banked
at `10` so a stall afterwards does not re-pay it.

**(b)** `30-instrument` attacks `20-hypothesize`'s output; `50`/`55`/`60`
attack the fix.

**(c)** The hard gate at `10` turns "could not reproduce" into a terminal,
evidenced, journaled `work.blocked` rather than a speculative patch.
Failure behavior otherwise is the design record's §2.8 degradation table.

## Notes for reviewers

`50-panel` through `60-re-verify-and-postmortem` are instantiations of the
shared panel machinery (`@@panel`, `@@refute`, `@@re-verify`), not
independently authored content — the three new stages this package adds
beyond `diagnose-bug`'s six. `00` through `40` carry `diagnose-bug`'s
content over with renumbering (`10-build-feedback-loop`→`00-`,
`20-reproduce-and-minimize`→`10-`, `30-hypothesize`→`20-`,
`40-instrument`→`30-`, `50-fix-with-regression-test`→`40-`); the old
`60-cleanup-and-postmortem`'s closing checklist and architectural-finding
behavior fold into `60-re-verify-and-postmortem`'s postmortem half.
