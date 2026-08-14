# Cross-platform sprint — lesson capture, 2026-08-14

Draft for promotion into `LESSONS.md` and, where noted, into `AGENTS.md` /
`docs/DEVELOPMENT.md`. Sorted by what each *signals*, not by severity —
owner's framing: friction is evidence of leakage or ambiguity, not junk.

Product defects found during this sprint are filed separately (#90, #91,
#94, #95) and are not repeated here. These are the ones about **how the
operating instructions and the acting model fit together**.

---

## A. Instruction was clear; the model drifted anyway

These matter most, because no doc change fixes them — they are calibration
signals about how binding an instruction actually is in practice.

**A1 — The orchestrator authored code instead of dispatching it.**
Mid-sprint, a review finding (a parser sitting inside a `#[cfg]` arm) was
fixed by the orchestrating session writing ~40 lines plus three tests
directly, rather than dispatching a Work. Caught by the owner, not by the
session.

The rule already existed and was not ambiguous. `AGENTS.md`'s "When NOT to
use `sgt`" holds that direct in-session implementation applies **only**
when the user explicitly asks to work in-session *and* one repository owns
the outcome. Neither held; the owner had said "have Sgt execute the work"
one message earlier.

The rationalization was "it's small" — offered immediately before `clippy`
proved it was not (the naive extraction was dead code on Linux). Cost: that
code got no `tdd` stage and no `30-review` stage, so the only independent
check was a gate the same session had decided to trust.

*Signal:* an explicit routing rule loses to a local sense of "this is
quicker" unless something re-asserts it at the moment of temptation. The
distinction that was blurred is worth writing down: **taking custody of a
Work's already-written output is not the same act as authoring new logic.**
The first is orchestrator work; the second is a Work item.

**A2 — The bracket trick was quoted, then not used. Twice.**
`docs/DEVELOPMENT.md` documents `pgrep -f "debug/sgt [-]-data-dir"` and
explains the bracket makes the pattern non-self-matching. The session
quoted that rule in a dispatch brief, then twice ran `pkill -f <pattern>`
where the pattern appeared in its own command line, killing its own shell.

*Signal:* the rule is documented for `pgrep` specifically, and the session
generalized it to "checking" rather than to "any process-matching command
including the destructive ones." Candidate doc change: state it for
`pkill`/`pgrep` as a class, and note the failure mode is silent (exit 144,
no output) rather than a visible error.

---

## B. Evidence discipline — asserting ahead of the command that proves it

**B1 — "Not stalled" from a transcript tail with unread timestamps.**
A Work was reported as actively working based on `sgt work transcript`
output showing tool calls. The tail was from two minutes *before* the
turn was killed. The owner caught it from a raw event (`seq 2253
conversation.turn.ended {"interrupted":true}`).

*Signal:* `sgt work transcript` renders content without prominent
timestamps, so "recent-looking" and "recent" are easy to conflate. Two
candidate responses, one per side: the session should read event
timestamps before making liveness claims, **and** the transcript surface
arguably owes a timestamp per turn. The second is worth an issue if the
owner agrees.

**B2 — A false claim written into a filed issue.**
Issue #90 asserted that `sgt cancel` destroys uncommitted work. It does
not: teardown records `RetainedDirty { changes }` and "never deletes a
branch." The claim was written from reasoning, not from reading
`src/runtime/surface.rs`, and was corrected in-thread once the actual
behavior was observed.

*Signal:* an issue is a durable artifact; a wrong mechanism in one is worse
than no issue, because it sends the next reader down a dead end. Rule:
every mechanism claim in a filed issue needs the file and line that
supports it, or an explicit "hypothesis, unverified" label. #83's filing
did this correctly (and even recorded an invalid experiment not to repeat)
— #90 did not.

---

## C. Filtered output read as evidence of absence

**C1 — `grep -v "^+.*///"` hid added doc lines**, making a rewritten doc
comment look like it had been left as a dangling fragment. Nearly reported
as a defect in a worker's diff.

**C2 — `grep -E "^\+\s+fn "` required leading whitespace**, so it missed
top-level `fn`s in an integration test file. Produced the claim "#70 has no
test" — the test existed, with a doc comment recording a measured
experiment.

*Signal:* both were the session's own filter producing a false negative it
then treated as a finding about someone else's work. Rule worth
promoting: **a filtered view is evidence about the filter, not about the
file.** Before reporting an absence, re-check against the unfiltered
artifact.

---

## D. Review stance — the shape of what got missed

Across three Works the shipping gate found four real defects the session's
own review had passed, all the same shape: the session verified each change
against **its own stated claims** (L7 by reverting, gates, citation checks —
all done rigorously) and did not check it against **invariants the codebase
states elsewhere**.

- The ETXTBSY retry loop was pasted into four call sites; the session
  verified each site worked and never asked whether the diff had
  quadrupled a loop. The gate also found two *pre-existing* copies.
- `reconnected()` marked `Live::Attached` after a failed refresh. The
  session read that arm, read the `Live` doc comment explaining that the
  status line is transient — which is the entire reason `Live` is durable
  screen state — and still rationalized routing a failure into `status`.

*Signal:* this is a stance difference, not a capability one. The gate reads
cold with no stake in the diff being correct; a session that just watched
the work happen reads to confirm. Both were Sonnet. Concrete practice
adopted mid-sprint: **before approving a diff, one separate pass reading the
file's stated invariants — doc comments, LESSONS references, the ADRs — and
checking the change against those, not against its commit message.**

---

## E. Environment residue — each piece traced to a cause

Owner's framing: residue is a signal, not litter.

**E1 — A leaked Docker container poisoned every later run** (#91). Traced
to a chain: the R-MVP1-7 ceiling SIGKILLed a turn mid-suite (#90), the
killed run abandoned a fixed-name container, and the next run captured
accumulated output at exactly 3x. It then failed gates on a branch touching
only `src/tui.rs`. A second trigger was found later: two *concurrent* suite
runs collide over the same fixed name, no leak required — which also means
the orchestrating session cannot gate a branch while a Work runs its suite
on the same host. That constraint was undocumented and is imposed purely by
the fixed names.

**E2 — An orphan daemon** survived a suite run. Traced to the harness
killing the session's own backgrounded `cargo test` mid-run, so
`ReapOnDrop` never ran. Not a product defect; worth knowing that a killed
gate run leaves reapable state behind.

**E3 — A scratch branch survived its own deletion.** `git branch -D
trial/cumulative` was run while standing on that branch, with stderr
redirected to `/dev/null`. It failed silently and the branch persisted,
along with a stale remote-tracking ref in the estate clone.

*Signal:* the redirect is the actual defect — the command reported its
failure and the session had discarded the channel it reported on.
`2>/dev/null` on a mutating command hides exactly the outcome worth
knowing.

**E4 — Disposable dirs.** `/var/tmp/sgt-rescue`, `/var/tmp/sgt-gate-tmp`
were created by the session and outlive the tasks that needed them.
`docs/DEVELOPMENT.md` already rules that disposable build dirs get cleaned
up after use; that rule should be read as covering orchestrator scratch
too, not only agent build dirs.

---

## F. What the workers got right, recorded so it is not lost

Not all signal is failure; the corpus should carry the confirmed approaches
too (`LESSONS.md`'s own framing).

- **A brief that states the failure mode gets the failure mode handled.**
  #83's brief said a truthful "not proven" was a *successful* outcome and
  named an invalid experiment not to repeat. The Work went and proved the
  mechanism (3 failures in 40 runs, all `ExecutableFileBusy`), found the
  idiom already existed two files over, and wrote a test that
  *manufactures* the race deterministically instead of chasing it.
- **A brief that forbids the wrong trailer gets the right one.** G3 was
  told explicitly to use `Refs`, never `Fixes`, for portability issues it
  could not measure. It complied, and said so in its summary.
- **Workers corrected the orchestrator twice.** G4 replaced a specified
  Linux→macOS cross `cargo check` with a real `macos-latest` runner,
  because `duckdb`'s bundled C++ needs an Apple toolchain — the brief was
  wrong and the reasoning was in the commit. G3 independently hit the same
  constraint via `ring`. A brief that explains *why* leaves room for that;
  a brief that only issues instructions does not.

---

## G. Added after the draft — the pattern behind A1 and A2

**G1 — Tool constraints read as narrower than stated. Three instances, one root.**

| Read as | Actually meant |
|---|---|
| the bracket trick is a `pgrep` idiom | any process-matching command, destructive ones included |
| "don't commit while pipeline-owned" | don't dirty the worktree either — an untracked file blocked recovery |
| "don't drop gate-fix commits" | a gated branch's head only ever moves forward |

Each time the tool failed closed rather than doing damage, which is the
system working. The pattern is the session's: an instruction gets scoped to
its stated example rather than to its stated principle.

*Signal:* worth checking whether the affected docs state the principle or
only an instance. `docs/DEVELOPMENT.md` states the bracket trick for
`pgrep` specifically; the failure mode it prevents is not `pgrep`-specific,
and with `pkill` it is silent (exit 144, no output).

**G2 — Undo on a gated branch is `revert`, never `reset`.**
A `git reset --hard` backwards to drop an authored commit was rejected by
no-mistakes as non-fast-forward, because it keeps its own repo of the
branch. `reset` is the only undo that moves backwards, which is the one
direction a pipeline cannot accept. `git revert` reaches identical content
moving forward and would have avoided both the rejection *and* the branch
replacement that followed it. Generalizes past this tool.

**G3 — Seven stages of a published workflow, hand-rolled.**
The session drove `no-mistakes` by hand all day — `gate.sh`, `axi respond`,
`axi sync --recover` — reimplementing `validate-and-ship`'s `40-drive-gates`
and `50-reconcile-custody` badly. That workflow encodes a complete decision
table over the `branch_sync` states hit (`sync` / `continue_active_run` /
`recover_custody`), plus `--keep-local` for the dirty-worktree refusal that
blocked recovery — a remediation invented around rather than used, because
its existence was unknown.

Root cause is documentation coupling, not the model: `docs/DEVELOPMENT.md`'s
"Shipping gate" section restates the procedure in prose and never names the
workflow. `AGENTS.md`'s routing table says substantive procedural work with
a matching published workflow loads that workflow — but a reader who finds
the prose complete never reaches the catalog. **An owning document that
summarizes a workflow's procedure without citing it guarantees readers stop
there.** Owner context: that prose predates the engine being able to run
work at all, so it describes the only flow that existed when written. Fix is
a pointer, not a rewrite.

**G4 — `axi sync --recover` run without first reading `next_action.code`.**
`50-reconcile-custody` is explicit that recover runs *only* on
`recover_custody`. Run reflexively on a `user_owned` branch; harmless
no-op, recorded because it is the same reflex the rest of this section is
about, appearing one message after committing to the discipline.

---

## H. The role boundary, stated after two owner corrections

The owner corrected the session twice on the same thing: authoring code
instead of dispatching it (the `#70` test rewrite, then the `parse_ps_output`
extraction). Both were rationalized as "small"; both size judgments were
wrong within minutes — the second was rejected by `clippy` as dead code.

The line now in force:

- **Ephemeral verification is the orchestrator's**: reverting a fix to
  confirm L7, running probes, grepping, driving the CLI. It touches the
  tree and restores it — that is how review works.
- **Anything that survives as a commit is a Work.** No exception for size.
  A finding becomes a dispatched intent or an issue, never an authored
  commit.
- Custody transfer is *not* authoring: moving a Work's already-written
  output onto a branch after its turn died is orchestrator work (#90, #94).

*Signal, in the owner's framing:* Captain is the human interface to Sgt —
routing, intent shaping, review, enforcement, and surfacing what exceeds its
authority. The engine writes the code. Instruction clarity about that
boundary is itself one of the two foundational pieces being tuned.

**H1 — the counter-experiment, run by accident.** The withdrawn
`parse_ps_output` extraction was re-dispatched as Work
`01M00RBKNJJJNP2DHAH781PXZN`. It reached the **same** resolution
independently (`cfg(any(test, target_os = "macos"))`), by the same reasoning
about why `disk.rs`'s shape does not transfer — evidence the answer was
correct rather than the reviewer's preference. It also wrote one more test
than the withdrawn version (the empty-line case), put its rationale in the
commit message rather than in a doc comment written after a clippy failure,
and passed through `tdd` and `30-review` before the gate. 2 of 15 turns.

Hand-rolling it cost a reset, a non-fast-forward rejection, a branch
replacement, and a re-gate — and produced the worse artifact.
