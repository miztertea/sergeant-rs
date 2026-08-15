# Session retrospective — cross-platform sprint + foundation rationalization

Written 2026-08-15, after PR #89 merged to `main`. Covers the whole session:
Wave 1–2 bug sprint, the `grill-with-docs` interview, ADRs 0005–0011, the
FOUNDATION-1 gauntlet, and the five enactment change-sets.

`lessons.md` in this directory is the raw capture written *during* the sprint
(sections A–H). This document does not repeat it. It carries three things that
capture could not: the **post-merge residue sweep** with numbers, the
**tool-call patterns** visible only across the full session, and the **doc and
workflow changes** those two justify.

Owner's framing throughout, from the message that started the capture: friction
is evidence of leakage or ambiguity, not junk. Everything comes from somewhere.

---

## 1. Residue sweep — what the environment still held after the merge

Run against a fully-merged tree with all 28 Works terminal (26 `completed`,
2 `canceled`), no daemon work in flight, no orphan processes, no leaked
containers.

| Residue | Size | Cause | Disposition |
|---|---|---|---|
| `.sergeant/data/surfaces/` × 3 | **30 GB** | `retained_dirty` teardown, no reaping verb | **#109** — left in place, owner's call |
| `/tmp/sgt-watch-test-hold-never-released-*` × 198 | 0 B each | dead-man test never removes its `.ready` marker | **#108** |
| `/var/tmp/sgt-rs-tests/.tmp*` × 8 | 1.7 GB | disposable test rigs; `Drop` never ran on SIGKILLed runs | sweep |
| `/var/tmp/sgt-gate-tmp/` | 72 MB | orchestrator gate scratch | sweep |
| `/var/tmp/sgt-rescue/` | ~100 KB | orchestrator rescue parking | sweep, one file kept |
| local `critic/*`, `refuter/*`, `inv/*` branches × 9 | — | gauntlet Work branches, stale, nothing unique | sweep (§1.4) |
| local branches with `[gone]` upstream × 19 | — | merged PRs | sweep |
| `no-mistakes/*` remote branches × 17 | — | gate pipeline's own refs | external tool |

Two of these are product defects. The rest are hygiene, and each still traces
to a cause worth naming.

### 1.1 The 30 GB — a correct policy that reads as a leak (#109)

Three surfaces survived teardown, all recording
`teardown.disposition = retained_dirty`, `clean: false`. `.git/worktrees` is
empty, so teardown *ran*; the directories were retained deliberately.

That policy is right. Teardown preserves uncommitted work and never deletes a
branch, and `AGENTS.md` puts preserved state outside anything standing
authorization may destroy. **The gap is the other half:** no verb lists what is
retained, explains why, or disposes of it once the work is banked. So the only
route to 30 GB is `rm -rf` inside the engine's data dir — an ad hoc shell
reconstruction of an engine operation, aimed at preserved state, which is what
the guardrails forbid and what an operator will do anyway when a disk fills.

Essentially the whole 30 GB is `target/`: build artifacts, gitignored, not
"uncommitted work" under any reading. Two of the three Works landed and merged;
what is genuinely worth retaining is a few hundred KB of tracked diff. The
retention decision is made at *directory* granularity while the thing being
protected is defined at *git* granularity.

**The rate is the finding, not the total.** 30 GB came from **three** Works.
This estate ran 28 in a single session; had every one torn down dirty, that is
roughly 280 GB from one night's work on one repository, on a machine nobody
warned. Retention is unbounded in the dimension that matters — it scales with
Works dispatched, which is precisely the number sergeant exists to make large.
Nothing in the engine samples, caps, or reports the total.

Owner's ruling, and it reframes this issue: **what a run writes to disk is a
contract, not hygiene.** Sergeant currently has no stated disk-footprint
boundary — no declaration of what a Work may write, where, whose it is when the
Work ends, or who is permitted to reclaim it. `retained_dirty` is an
*implementation* of half of that contract with the other half unwritten, which
is why 30 GB accumulated without a single surface reporting anything wrong.
Recorded in #109; the missing contract is the larger item behind it.

Generalized as `LESSONS.md` **L22**: a never-delete guarantee ships incomplete
without its inspect-and-reap verb, because the absence gets routed around with
something more dangerous than the verb would have been.

### 1.2 The 198 files — the failure-path test is the one that leaks (#108)

`test_hold_wait` writes a `<path>.ready` rendezvous marker. The happy-path test
removes it. The dead-man test — whose premise is that the release path never
appears — removes nothing. One leak per suite run, into `std::env::temp_dir()`,
which on this host is the same 16 GB tmpfs whose exhaustion caused a host-wide
incident on 2026-08-13 (#70) and which `docs/DEVELOPMENT.md` already forbids for
test artifacts.

Generalized as **L21**: cleanup belongs in the code under test or an RAII guard,
never only in the body of whichever test succeeds. The abnormal path is the
least likely to carry cleanup and runs just as often.

### 1.3 The 1.7 GB of test rigs — a known cause, still uncaught

Eight `TempDir`-shaped rigs under `/var/tmp/sgt-rs-tests`. `DEVELOPMENT.md`
already rules that disposable build dirs get cleaned after use. They
accumulated anyway, and the cause is `lessons.md` E2's: when a run is SIGKILLed
mid-suite — the R-MVP1-7 ceiling (#90), or the harness killing a backgrounded
`cargo test` — `Drop` never runs.

This is the same root as the leaked Docker container that poisoned an unrelated
branch's gates (#91), and #91 was fixed with a `Drop` guard. Worth noting that a
`Drop` guard does not survive `SIGKILL`, so the fix is partial by construction:
**a reaper that runs at suite start is the only cleanup that survives the way
these processes actually die.** `tests/support/mod.rs`'s `DataDir` guard already
reaps daemons by `/proc` argv scan on `Drop` — extending that reaping to
start-of-run, for rigs as well as daemons, would close the class rather than
another instance.

### 1.4 Branch residue — all of it ordinary, once checked

The 19 local branches with `[gone]` upstreams are merged-PR debris: ordinary
hygiene, safe under `git branch -d`'s own merge check.

The 9 `critic/*`, `refuter/*`, `inv/*` branches from the FOUNDATION-1 gauntlet
were initially written up here as a *different* class — unreaped Work branches
holding the only copies of critic and refuter reasoning, and therefore part of
#109's missing-verb question rather than branch hygiene.

That was wrong, and the check that disproved it took one command:

```
git diff main..<branch> --diff-filter=A --name-only
```

Every one of the nine holds exactly the same five files main lacks:
`src/web.rs`, `web/dashboard.{css,js}`, and two dashboard screenshots — the
dashboard deleted in §5.5. They hold **no** unique reasoning; every critic and
refuter output is committed under `docs/gauntlet/runs/foundation-1/`. They are
stale branches cut before a deletion, identical in class to the other 19.

Recorded rather than quietly fixed, because it is §2.1's pattern one more time
and the last instance in this document: a plausible mechanism, asserted from
branch *names* and a diffstat, when the question "does this hold anything
unique?" has a direct command that answers it. The diffstat even showed it —
those branches are thousands of lines *behind* main, which is what a stale
branch looks like.

### 1.5 Cleanup not performed

The sweep column above is what *should* be swept, not what was. Deletion was
refused by the permission classifier in this session's mode — first as a
bundled four-operation script (correctly: it was un-reviewable), then
individually. The residue is documented and left in place.

The bundling attempt is its own small lesson: **several destructive operations
in one command is not efficiency, it is an un-reviewable diff.** A human
approving that command reads it the same way the classifier did.

---

## 2. Tool-call patterns worth tightening

These are visible only across the whole session; no single instance looked like
a pattern.

### 2.1 Every reduction of output lied at least once — promoted as L23

Six instances, four tool families, one shape: a view was narrowed and the
*narrowing* produced the answer.

| Reduction | False conclusion |
|---|---|
| `grep -v "^+.*///"` on a diff | "the worker left a dangling doc fragment" |
| `grep -E "^\+\s+fn "` | "#70 has no test" — it was a top-level `fn` |
| `grep -E "outcome:"` unanchored | "gate run terminal" — matched no-mistakes' help text |
| guessed JSON path `run.surface` | "teardown never recorded" — the key is top-level |
| `\| tail -1` on `gh pr edit` | "PR title updated" — the command had exited 1 |
| `cmd \| head -3 && echo OK` | "patch content already in tree" — `&&` saw `head`'s status |

The last two are the dangerous class: the reduction ran on a **mutating**
command and discarded the channel reporting failure. Same defect as E3's
`2>/dev/null` on a `git branch -D` that silently failed.

Two of these happened *during this retrospective*, after the pattern was
already written down — the JSON path and the `&&` — plus §1.4's wrong claim
about the gauntlet branches, which is the parent class rather than a reduction:
asserting a mechanism from names instead of running the command that answers it.

That is the honest calibration signal, and it is the most useful line in this
document: **naming a pattern does not stop it.** Three instances inside the
retrospective that names it. What stops it is a mechanical rule applied without
judgment about whether this case needs it:

- Anchor every monitor pattern (`^outcome:`).
- Never put a pipe or `&&` between a mutating command and its exit status;
  capture `rc=$?` on its own line.
- Before reporting an absence, re-check against the unfiltered artifact.
- Never guess a JSON path — print the shape first.
- Before writing a mechanism into a durable artifact, run the one command that
  would falsify it. (`lessons.md` B2 filed a wrong mechanism into issue #90 for
  exactly this reason.)

### 2.2 Monitoring discipline — the failure was structural, not forgetful

Twice a long wait ran with no armed watcher, and the second time the owner had
to say so. The framing "I forgot to set a monitor" is wrong and unfixable; the
accurate framing is that **the monitor was armed after the wait began**, so
anything landing in the gap was invisible and the only recovery was polling.

`AGENTS.md` step 6 already states the correct rule for the estate-wide case —
attach the watcher *before* reconciliation, because an estate-wide watch is
edge-triggered from the moment it attaches. The same reasoning covers a gate
run and it is not written there, because the gate is an external tool.

Concrete: arm the watcher as part of *starting* the long thing, in the same
response, never as a follow-up.

### 2.3 A one-shot foreground wait is a ten-minute bet

Foreground tool calls cap around ten minutes on this harness. Several gate runs
and suite runs exceeded that, and each overrun cost a re-check cycle. The rule
already in `AGENTS.md` — background `--follow` for long or multiple waits — is
right; what was missing is that **the estimate is the decision point**, and any
wait that *might* exceed ten minutes should be backgrounded from the start
rather than promoted after it stalls.

### 2.4 Dispatch briefs: what measurably worked

Recorded because the corpus should carry confirmed approaches, and because
these are cheap to keep doing.

- **Stating the failure mode gets the failure mode handled.** #83's brief said
  a truthful "not proven" was a *successful* outcome and named an invalid
  experiment not to repeat. The Work proved the race (3 failures in 40 runs),
  found the idiom already existed two files over, and wrote a test that
  manufactures the race deterministically.
- **Naming the trailer prevents the wrong closure.** G3 was told to use `Refs`,
  never `Fixes`, for portability it could not measure. It complied and said so.
- **Explaining *why* leaves room to be corrected.** Two Works overrode their
  briefs on technical grounds and were right both times (a `macos-latest`
  runner instead of a cross `cargo check`; `cfg(any(test, target_os))`). A
  brief that only issues instructions forecloses that.
- **Naming the prior art as settled prevents re-derivation.** The §5.1 brief
  said "the hard part is already solved — read it, do not redo it" and pointed
  at the §8.6 investigation. The Work read it and spent its turns on the part
  that was actually open.
- **Passing briefs as files, not inline strings.** After backticks in a `sgt
  run` intent caused shell command substitution to execute `sgt work list` and
  embed its output into a brief, every subsequent brief was written to a file
  and passed as `"$(cat file)"`. Worth making the default.

---

## 3. Instruction and workflow changes

### 3.1 Made in this PR

**`docs/DEVELOPMENT.md` — the bracket trick, stated as a class.** It was
written about `pgrep`, and got scoped to "checking commands" rather than to
"matching processes"; an unbracketed `pkill` then killed this session's own
shell twice. Now states it for any process-matching command, and records that
the `pkill` failure is *silent* (exit 144, no output), which reads as a hung
command rather than an error.

**`docs/DEVELOPMENT.md` — the shipping gate cites its workflow.** The section
restated `validate-and-ship`'s procedure in prose and never named it, so a
session that found the prose complete never reached the catalog and hand-rolled
seven stages, inventing a workaround around `--keep-local` rather than using
it. Now marked as a summary, pointing at
`.sergeant/workflows/validate-and-ship/`, with the two clarifications the prose
cost: pipeline-owned means *do not write to the worktree at all*, and undo on a
gated branch is `revert`, never `reset`.

Root cause is documentation coupling, not model behavior — the prose predates
the engine being able to run work at all.

Promoted as **L20**, reframed on the owner's ruling: a document says what we
knew when it was written; if it is wrong, supersede it and move on. The
sharpening the incident adds is about the *trigger*, not the action. **That
prose was never wrong** — every sentence in it was true, which is exactly why
nothing prompted supersession. Supersession fires on contradiction, and
staleness that keeps telling the truth produces no contradiction to fire on.

So the hook cannot be "notice when a doc is wrong." It has to be: **when a
capability ships, the prose that predates it is part of what ships** — the
ADR-refresh rule this repo already enforces (the gate caught ADR staleness
three separate times this sprint), generalized past ADRs to any document
describing a procedure the new capability now owns. And the layering rule
("the document that owns the topic wins") could not have caught this: it
resolves disagreements, and there was none.

**`docs/DEVELOPMENT.md` — test artifacts follow the build-dir placement rule.**
Nothing a suite creates may be left in `std::env::temp_dir()`, and cleanup goes
in the code under test or a guard rather than a test body.

**`LESSONS.md` L20–L23.**

### 3.2 The proposals named the wrong file — corrected

As first written, §3.2 proposed two changes to **`CLAUDE.md`**. There is no
such file. It is a git symlink (mode `120000`) to `AGENTS.md`, which is the
tracked artifact; the harness loads project instructions and reports the path
it opened, and that path was cited all session without ever being resolved.

The correction was on **line 8 of `docs/DEVELOPMENT.md`** — "This file used to
be `CLAUDE.md`; that path is now a git symlink to `AGENTS.md`" — in a document
this session edited twice tonight without reading its first twelve lines. It
is also not the first time: `GAUNTLET.md` CH-5 records a skill citing
"CLAUDE.md L1" after the symlink commit had moved that text away.

Repo-wide the drift is live: 145 files cite `AGENTS.md`, 48 cite `CLAUDE.md`.
Citations through the alias resolve correctly for a reader with a checkout, so
this is not urgent — but it is the reason a proposal was addressed to a
symlink, and it is worth a decision on whether `CLAUDE.md` is ever a valid
citation target or only an entry point.

Folded into **L23** as its last row, because it is that lesson's shape rather
than a new one: an indirection sat between the artifact and the reader, and
the indirection got cited instead of the artifact.

### 3.3 Proposed, not made — these are the owner's to rule on

Deliberately not enacted: they change governing text or product behavior.

1. **`AGENTS.md` step 6 — generalize the watcher-before-wait rule.** It is
   currently written for the estate-wide `sgt watch` case. The same
   edge-triggered reasoning covers any long external wait, including a gate run.
   (§2.2)
2. **`AGENTS.md` — brief transport.** Make file-passed briefs the documented
   default over inline strings, given the command-substitution hazard. (§2.4)
3. **A start-of-run reaper, not another `Drop` guard.** `Drop` does not survive
   `SIGKILL`, and SIGKILL is how these processes actually die. Extending
   `DataDir`'s `/proc`-scan reaping to run at suite start, covering rigs as well
   as daemons, closes the class. (§1.3)
4. **Retention scope (#109 item 3).** Whether teardown should retain the dirty
   *state* — patch, stash, bundle — rather than the *directory*, and whether
   `target/` is ever in scope. This is the decision that makes #109 either a
   hygiene feature or a design correction.
5. **A timestamped `sgt work transcript`.** Its output renders without
   prominent timestamps, so "recent-looking" and "recent" are easy to conflate;
   this produced a wrong liveness claim the owner caught from a raw event.
   (`lessons.md` B1)

---

## 4. The honest accounting

The five defect classes the shipping gate caught that this session's review
passed are recorded in `close-out.md` and `lessons.md` D. The one-line version:
**the session verified changes against their own claims; the gate verified them
against invariants the codebase states elsewhere.** Both were Sonnet. It is a
stance difference, not a capability one.

Three owner corrections drove more improvement than any finding:

- Authoring code instead of dispatching it — twice. Both rationalized as
  "small"; both size judgments were wrong within minutes. The re-dispatched
  version reached the same answer independently *with one more test*, through
  `tdd` and `30-review`, in 2 of 15 turns. Hand-rolling it cost a `reset`, a
  non-fast-forward rejection, a branch replacement, and a re-gate.
- Narrating work instead of doing it — "Writing the §5.2 brief now," followed
  by nothing.
- Not surfacing what needed a decision, while stopping repeatedly for things
  that did not.

The line now in force, from `lessons.md` H: ephemeral verification is the
orchestrator's; anything that survives as a commit is a Work, with no exception
for size; custody transfer of a Work's already-written output is not authoring.

Time cost worth recording against the roadmap: the gate is serial and
Captain-owned, so all five enactment change-sets queued behind runs supervised
by hand, twice re-running because the orchestrator had dirtied a pipeline-owned
worktree. That is the cost §5.1 exists to remove and has now only half removed —
items 1 and 2 shipped, items 3 and 4 are adjudicated recommendations. Until
those land, gating runs at Captain's pace, which was the slowest thing in the
loop.
