# Cross-platform sprint + foundation rationalization — close-out

Companion to `plan.md` (written before the work ran, amended in place) and
`lessons.md` (what the run taught about instructions and model behavior).
Gauntlet record for the proposal half: `docs/gauntlet/runs/foundation-1/`
and `GAUNTLET.md`'s FOUNDATION-1 entry.

## What this round became

It was scoped as four bug fixes. It ended as two rounds: a cross-platform
bug sprint, and a foundational rationalization of Captain and Sgt that the
sprint itself exposed the need for.

The pivot is the finding worth keeping: **running the process surfaced more
than looking for problems did.** Waves 1 and 2 closed six issues and
surfaced seven the board did not have, three of which were actively
degrading every wave that followed. The owner then re-prioritized on that
evidence — engine integrity before low-severity perf — and a grilling
interview turned the accumulated friction into seven rulings.

## Landed

**Bug sprint.** #83 (ETXTBSY race, proven at 3 failures in 40 runs and fixed
with a deterministic manufactured-window test), #70 (test rigs onto real
disk), #16 (TUI SSE auto-reconnect), #86 (bash 3.2 floor), #87 (tiered CI
with a real macOS runner), #91 (per-run container names, both triggers).

**Foundation rationalization**, from ADRs 0005–0011:

| § | Change | Issues |
|---|---|---|
| 5.3 | The actor runtime contract — execution model stated, `completed` can no longer mean "committed nothing" | Refs #94, #60 |
| 5.4 | `[estate] data_dir`; #64 re-ruled rather than implemented | Refs #80, Fixes #64 |
| 5.5–5.7 | Observation never materializes the daemon; bare `sgt` is a homepage and the TUI a verb; the dashboard deleted | Fixes #68, #21, #15 |
| 5.2 | Harness passthrough — `exec`, never supervise; doctor verifies the same list | Fixes #100, #60 |
| 5.1 | Gate-as-a-Work: §8.6 Mechanism A's branch-takeover primitives | Refs #90 |

## Deliberately not closed

**#18, #81, #82, #95** close on a real macOS host, not on Linux. Every macOS
arm shipped this round is marked UNVERIFIED in its own doc comment. ADR
0001's posture is that "measured" is the only claim this project makes, and
writing an arm is not measuring it.

**#90** (ceiling interrupt wedges a Work with no exit door) and **#94**'s
remaining half stay open. #94's first half — the execution-model statement —
shipped; it closes when someone confirms actors stop guessing.

**§5.1 is not end-to-end.** Items 1 and 2 of §8.6's sizing shipped (the
precondition check and the attach path). Items 3 and 4 — a submission shape
for "this Work reviews that Work", and the operational response to each
precondition failure — were deliberately left as recommendations, adjudicated
but unimplemented.

## What the process caught that review did not

The shipping gate found defects the orchestrating session's own review had
passed, repeatedly, and the pattern is consistent enough to name: **the
session verified changes against their own claims, and the gate verified them
against invariants the codebase states elsewhere.**

- A retry loop pasted into four call sites, on top of two pre-existing copies.
- `Live::Attached` set after a failed refresh — reproducing the exact defect
  the `Live` enum exists to prevent, in a change whose commit message said so.
- A regression *its own fix* introduced: `backoff.reset()` above the new
  early return, pinning every retry at 250 ms and never reaching the cap.
- ADR staleness three separate times, which is a brief defect rather than
  three accidents; later briefs carry refresh-on-ship explicitly.
- Two host-dependence defects (`#70`'s `/var/tmp` assertion, the `environment`
  check's ambient `$HOME`/`$PATH`) that would have failed the macOS lane for
  reasons unrelated to the change under test.

Workers also corrected the orchestrator twice on technical grounds — a
specified Linux→macOS cross-check replaced with a real runner because
`duckdb`'s bundled C++ needs an Apple toolchain, and the same constraint hit
independently via `ring`.

## The gauntlet on the proposal

First time this ledger graded a proposal rather than an implementation.
13 findings, 1 refuted, 12 confirmed, 3 severity downgrades, 1 surviving
error — against the proposal's own architectural justification, caught by a
refuter that empirically tested git's two-worktrees-per-branch refusal rather
than reasoning about it.

The refuted finding is the argument for keeping refuters: a critic invented a
contract citation to support it, and only the adversarial pass caught the
fabrication.

Method note: the confirm rate ran well above prior units. The honest reading
is not a lenient panel — refuters downgraded three severities and struck one
finding after independent re-verification — but an artifact written in a
single pass with no prior review, by the session that then convened the
panel. Every Work reviewed this round had a `tdd` stage, a `30-review` stage
and a gate before the orchestrator saw it. The proposal had none.

## Candidates for the next round

- **Nothing enforces the two-environment rule.** `docs/DEVELOPMENT.md`
  requires fixtures to work in both known environments or probe-gate loudly;
  every catch so far has been a reviewer noticing. With #87's matrix landing,
  the failure mode changes from "a test is wrong" to "a lane is red for
  unrelated reasons," which is how people learn to ignore red.
- **§5.1 items 3 and 4**, with the adjudicated recommendations as input.
- **The macOS session**: #18, #81, #82, #95, plus verifying the passthrough's
  environment list and every UNVERIFIED arm.
- **#90 and #94's remaining half** — the engine-side exit door.
- The queued perf work: #85, #12, #10, #8 → #4.
