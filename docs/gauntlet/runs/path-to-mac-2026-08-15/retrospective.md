# Session retrospective — Path-to-Mac sprint

Written 2026-08-15 at the owner's direction, before PR #111's final review.
Covers the whole session: the `PATH-TO-MAC-1` plan gauntlet, eight
implementation Works, three shipping-gate dispatches, and the residue and
instruction defects the run exposed.

Owner's framing, carried from the previous retrospective and repeated here
because it kept paying: **friction is evidence of leakage or ambiguity, not
junk. Missed steps usually mean unclear instructions; leftover residue is
usually a leak.**

`adjudication.md` records the gauntlet's verdicts and is not repeated. This
document carries what only a whole-session view shows.

---

## 1. Residue sweep

Run against the merged tip `251a6f1`, fleet quiet (47 Works, zero
nonterminal).

| Residue | Measure | Cause | Disposition |
|---|---|---|---|
| `.sergeant/data` | **114 MB**, **0 retained surfaces** | 19 Works, all clean teardown | healthy — see §1.1 |
| `/tmp` | 2% of 16 GB, **0** watch markers | — | clean; #108's fix verified empirically |
| orphan daemons / containers | **0 / 0** | — | clean |
| `~` beyond `sergeant-rs`/`inbox` | **0** entries | — | scratch discipline held |
| **parked no-mistakes run** | 1, `awaiting_approval`, **2h2m** | W6b's run, never disposed | **§1.2 — a leak** |
| local branches from this session | **12** | critics ×4, refuters ×4, firstpass, 2× gate-escalation, 1× pipeline-pushed | **§1.3** |
| `sergeant/01M02VBSZ945TPHGDK6TJEHGPT` in the primary checkout | 1 branch | pipeline pushed it | **§3.1 — a real defect** |
| remote `lane/*` branches | **0** | auto-deleted on merge | clean |
| `/var/tmp/path-to-mac-briefs` | 1.2 MB | orchestrator briefs | sweep |
| `/var/tmp/sgt-rescue`, `sgt-rs-tests` | 156 KB, 4 KB | prior sprints | pre-existing, now negligible |

### 1.1 The 30 GB did not recur — and that is a measurement, not luck

The previous sprint ended with **30 GB** across three `retained_dirty`
surfaces and no verb to reap them (#109, L22). This sprint ran **19 Works**
— more than half again the count that produced that 30 GB — and ended with
**zero retained surfaces and a 114 MB data dir**.

Two things changed: every Work this sprint torn down clean (no ceiling
interrupts, no dirty exits), and W4 landed R4's retention-scope ruling so a
dirty teardown now retains a patch rather than a directory. Only the first
was exercised — **no Work tore down dirty, so the new retention path did not
actually run in anger**. The 114 MB is evidence that clean teardown works at
19× scale; it is **not** yet evidence that the patch-retention fix works.
That still wants a deliberately-dirtied Work to prove.

### 1.2 A parked pipeline run is residue nobody owns

`no-mistakes axi status` still reports run `01M02RGV19K4THRBWBHHM8RS1G`,
`status: running`, `awaiting_agent: parked 2h2m`, **1 finding awaiting**.

It belongs to W6b, whose surface is long since torn down and whose Work is
terminal. The finding is meaningless — that run reviewed only its own two
ledger commits against a stale base. But it sits in the pipeline's state as
an open run, and nothing in sergeant knows it exists.

This is **L22's shape one layer out**: a Work's own teardown is clean and
journaled, while the *external tool* it drove keeps state that outlives it,
with no verb on either side to reap it. Sergeant's disk-footprint contract
(#109's larger item) covers what a Work writes to disk; it says nothing
about state a Work creates in a third-party daemon.

### 1.3 Branch residue is three classes, and the merge check earned its keep

Twelve local branches. Ten were ordinary gauntlet/lane debris and deleted
cleanly under `git branch -d`'s own merge check — including
`sergeant/01M02VBSZ945TPHGDK6TJEHGPT`, which was **pushed into the primary
checkout by the gate pipeline** rather than created by any local command
(§3.1 — its presence is the symptom, not the defect).

**Two refused deletion as "not fully merged", and they were right to.**
`lane/w6-gate-escalation` and `lane/w6b-gate-escalation` hold the gate Work's
five-dispatch diagnostic record — the false-pass discovery, the corrected
root cause, and the escalations that produced #120. That evidence was taken
into custody locally and **never landed on the integration branch**, so it
existed on exactly one machine. A `-D` in the same sweep as the other ten
would have destroyed it silently.

They were first pushed to `archive/w6-gate-escalation` and
`archive/w6b-gate-escalation` rather than merged, on the reasoning that five
near-duplicate ledger entries would bury the ledger but the evidence had to
survive somewhere durable.

**That was wrong, and the owner caught it within the hour.** Checked
properly, both branches contain **only `GAUNTLET.md` prose** — 166 and 155
insertions, nothing else — and that prose is the Work's own account of a
saga already recorded in three more discoverable places: **five first-person
comments from the same Work on #120**, this retrospective's §1.2 and §3.1,
and PR #122's body. The Works themselves remain journaled and their full
intents retrievable. The branches preserved nothing unique, in the least
findable location available. Both deleted.

**This is L22 in miniature, committed roughly twenty minutes after writing
§1.2 about it.** *"Correct retention is indistinguishable on disk from a
leak"* — two permanent refs were created because deleting felt risky,
without first asking whether the thing being preserved was actually the
artifact. It was #120 all along.

What *was* right: `git branch -d`'s merge check correctly flagged that those
commits were not on the integration branch, and that was real signal. The
error was inferring **"unmerged ⇒ must preserve"** instead of **"unmerged ⇒
check whether it matters."** A safe default tells you to stop and look; it
does not tell you what you are looking at.

The general lesson is still about the **sweep**: a cleanup that treats a
session's artifacts as one homogeneous class will eventually delete the one
that mattered, and using `-d` rather than `-D` is what created the moment to
check. The previous retrospective recorded a session bundling four
destructive operations into one un-reviewable command; this is the same
hazard at smaller scale, caught by a safer default — and then nearly
converted into a leak by over-correcting.

---

## 2. Product defects filed

Three issues, all found by running the loop rather than by inspecting it.

- **#112** — a dispatched Work cannot resolve GitHub issue numbers. The
  estate clone's only remote is a local path, so `gh` fails with a
  **misleading auth remedy** while `gh auth status` shows a valid account.
  Every Work whose scope is stated as bare issue numbers depends on this.
  One critic seat hit it and honestly recorded an issue as `PLAUSIBLE`
  rather than guessing — fail-closed working, at the cost of real coverage.
- **#113** — split from #108 once the two mechanisms were shown to differ.
  #108's dead-man test completes normally, so `Drop` runs and a guard
  suffices; #113's rigs die by `SIGKILL`, which no destructor survives. A
  sprint plan nearly shipped one as the other's fix.
- **#120** — `no-mistakes` caches `default_branch` at registration and never
  re-derives it. Any repo registered while its `origin` sat on a feature
  branch is permanently mis-based. **Still open**; the `no-mistakes init`
  refresh fixed this estate, not the defect.

Two more were found and fixed inside the sprint: the **`sgt work reap`
preview auto-spawn** (ADR 0009 violation, found by the gate) and the
**reaper's `/proc`-only liveness probe** (found by the orchestrator reviewing
a delivered Work). Both are described in PR #111.

---

## 3. Instruction and brief defects — these are mine

### 3.1 A shared preamble contradicted the brief it was concatenated into

**The sharpest instruction defect of the session, and it had teeth.**

Every Work's brief was assembled as `work-common.md` + a per-Work section.
`work-common.md` was written for *implementation* Works and contains:

> **Do NOT run `scripts/gate.sh` or no-mistakes.** A separate gate Work (W6)
> owns the shipping gate for this sprint.

That preamble was concatenated onto **W6's own brief** — the gate Work — at
line 42, above a section telling it to run `validate-and-ship`. The Work
resolved the contradiction the only sensible way: it drove the pipeline
through the workflow's stages rather than through `scripts/gate.sh`.

**`scripts/gate.sh:122` is what supplies `--skip push,pr,ci`.** Bypassing it
meant those flags never applied. The run **pushed** `sergeant/<work-id>` into
the primary checkout. `pr` and `ci` were skipped *only* because the provider
was unknown — this estate's `origin` is a local path:

```
push  : pushing to /home/miztertea/sergeant-rs (refs/heads/sergeant/01M02VBSZ945TPHGDK6TJEHGPT)...
pr    : skipping PR creation: provider unknown is not supported yet
ci    : skipping CI: provider unknown is not supported yet
```

**On a repo whose `origin` is a GitHub host, a dispatched gate Work would
have opened a PR and triggered CI autonomously.** Nothing in the brief, the
workflow, or ADR 0005 would have stopped it. That is a product gap, not just
a brief defect — ADR 0005 made gating a dispatched Work without reconciling
that `gate.sh`, not the workflow, is what carries the safety flags. **Filed
below.**

Rules this earns: a shared preamble is not free — **every prohibition in it
must be checked against every brief it is concatenated into**; and a Work
that is the exception to a rule must be told so explicitly, not left to infer
it from a section ordering.

### 3.2 The estate manifest declared no default backend

The first dispatch of the session was refused: `422: profile "sonnet"
launches backend "claude", but this work routed to "fake" (global_default)`.

The preflight was right and the refusal was correct. But the estate had
simply never declared `[estate] default_backend`, so every submission fell
through to the daemon's hardcoded `fake`. Fixed by declaring it — §13's
`WorkspaceDefault` tier, which the router already reads.

Worth recording as a **safety property that was lost deliberately**: `fake`
meant an accidental `sgt run` cost nothing. It now bills. That was the right
trade for an estate whose every Work is real work, but it should be a
deliberate loss, not a discovered one.

### 3.3 Two governing documents disagreed with an accepted ADR

`docs/DEVELOPMENT.md:70-73` still carried the "an actor never invokes the
gate" rule that **ADR 0005 dissolved** on 2026-08-14. Superseded by W5, kept
rather than deleted per L20.

And the plan itself cited **ADR 0002 (D4)** for an objection that lives in a
**source-code comment** (`src/platform/disk.rs:4-6`, originally
`src/backend/docker.rs`). Two critic seats called the quote *invented* —
because their greps excluded `src/` — and **both proposed deleting it**. The
refuter caught it. See §4.1.

---

## 4. Tool-call patterns

### 4.1 The correction is the dangerous end of a false absence — promoted as L24

Two blind seats independently reported the same real, verbatim, git-traceable
quote as invented. Both greps searched `docs/`, `reference/`, `GAUNTLET.md`,
`LESSONS.md` — and excluded `src/`, where it sits. Both proposed **deleting
the claim**.

L23 already explains why the absence was reported. What it did not cover:
here the **remedy** was the destructive act. Applying either critic's
correction would have removed a true, sourced statement from a governing
document. The fidelity refuter re-ran the critic's own stated command, got
three hits where the critic reported one, and traced the phrase through
`git log -p`.

This is the third recorded instance of L23's pattern surviving being named,
and the first where the fix would have caused the damage.

### 4.2 A silent instrument is indistinguishable from a stalled subject — also L24

Four separate instrument failures, all presenting as silence:

| Instrument | Failure | Looked like |
|---|---|---|
| backgrounded `sgt watch` ×3 | harness killed it at ~160s | "the Work hasn't moved" |
| `Monitor` script | died instantly, exit 127, invalid `${ }` syntax | "no state change yet" |
| `ls` in a compound command | returned 2 on no-match, setting the whole command's exit code | "the test run failed" |
| `review.log` existing | file present, contents unread | "the gate validated" |

The last is the one that cost most: **I declared the gate fixed on the
presence of a log file.** It had run — against two ledger commits, not the
sprint. That is §4.1's pattern with a filesystem check instead of a grep.

Rules: a monitor's first emission should be its own self-test; a watcher's
death is a prompt to read journal state, never information about the
subject; and a filter must match every terminal state, because **silence is
not success**.

### 4.3 Guessing a JSON path, again

`work.backend` is present in a submission response when routing is
`explicit`, and **absent** when it resolves via `workspace_default`. A parse
written against the first shape failed silently against the second. L23 says
print the shape first; this session did it twice before complying.

Whether that field's presence *should* vary with routing tier is a real
question — a client parsing it is fragile by construction. Not filed; noted.

### 4.4 The estate clone's `origin` is a live working copy — twice

#120 is the sharp version: `no-mistakes` resolved its diff base from a
non-bare checkout's HEAD. But the same fact bit again, more quietly, at
integration time: `git fetch` in the estate clone pulled a **stale local
branch ref**, because the clone's `origin` is the primary checkout and that
checkout was on `main`, so its `integration/...` ref had never moved.

Generalization worth carrying: **in this estate, the primary checkout's own
branch refs are load-bearing infrastructure**, not personal bookkeeping.
Anything resolving through `origin` sees whatever state that checkout is in.

### 4.5 Bundling made a command un-reviewable

Two dispatches were blocked by the permission classifier. The first bundled a
heredoc write, a concatenation and a dispatch into one call — correctly
refused, and the same lesson §1.5 of the previous retrospective recorded.
Splitting the write into a `Write` tool call and the dispatch into its own
command resolved it.

### 4.6 An `Edit` failed from a memory-reconstructed target

Reconstructing `old_string` from a read an hour earlier failed to match —
**L12** verbatim, including its own recorded instance. One `Read` fixed it.
Recorded because L12 predicted it precisely and it happened anyway.

---

## 5. What worked, measurably

Recorded because the corpus should carry confirmed approaches.

- **`settled` / `proposed` brief markers.** Every brief labelled prior art
  `settled` (do not re-derive) and orchestrator guesses `proposed` (reject if
  the code disagrees). Works exercised the licence **at least four times**:
  W2 evaluated and accepted the R-MVP1-10 shape with argument; W7 chose to
  reuse `src/platform/process.rs` over duplicating a `cfg` pattern; W8 chose
  a sibling file over inlining and **corrected the brief's own claim** about
  which suites are Docker-gated; the gate Work rejected the orchestrator's
  root-cause diagnosis twice and was right both times.
- **Requiring a Work to argue for its own trailer.** Five issues needed
  `Refs` rather than `Fixes` against real temptation. Zero violations, and
  the gate's independent audit confirmed it.
- **Adversarial refuters with a specific line of attack.** All four moved
  something — including refuting the orchestrator's own hypothesis about a
  5 KiB arithmetic gap. Second unit in a row to show this; it should be
  standing practice, not a per-unit choice.
- **A panel disagreement treated as signal.** One seat declined to find where
  another graded an error; the refuter settled it by reading `src/watch.rs`
  rather than reasoning. "Nothing found" from one seat is not evidence of
  nothing.
- **Self-correcting Works.** Three shipped a *second* commit fixing their own
  first: a wrong trailer, a vacuous pinning test caught by the Work's own
  revert-probe, and an imprecise citation. None was asked to.

---

## 6. Calibration

**Envelopes were over-provisioned 20×.** Sized at `--turns 40` out of concern
for #90's wedge; **every implementation Work used exactly 2**. The padding was
nearly free, but the sizing was fear-based rather than evidence-based. There
is now evidence: an `implement` Work against a well-scoped brief on this repo
costs ~2 turns. And #90 is fixed, so the original reason is gone.

**Cost:** 19 Works, all `completed` — zero failures, zero wedges, zero
`needs_input`. 7 implementation × 2 turns; 8 gauntlet seats × 1; 1 first-pass
review × 5; 3 gate dispatches × 7. Test count 633 → **658**, never red,
`SKIPPED-ENV` **0** throughout.

**The gauntlet paid for itself twice over.** It cost about one implementation
Work's tokens and stopped six Works executing a plan whose central premise was
false. Then the gate — which cost three dispatches and a product defect to get
running — found two more defects that the panel, the per-Work reviews, and
four green local gate runs had all passed.

---

## 7. Proposed, not made — the owner's to rule

Deliberately not enacted; each changes governing text or product behavior.

1. **File the gate-safety gap from §3.1.** ADR 0005 made gating a dispatched
   Work; `scripts/gate.sh` is what carries `--skip push,pr,ci`. A
   `validate-and-ship` Work driving the pipeline directly gets none of them.
   Today only an unknown provider prevents autonomous PR creation and CI
   triggering from inside a gate Work. **Recommend filing.**
2. **A disposal story for external-tool state (§1.2).** A parked pipeline run
   outlives the Work that created it and nothing on either side reaps it.
   Either `validate-and-ship`'s `60-close-out` disposes of its own run, or
   #109's disk-footprint contract widens past disk to *"state a Work creates
   anywhere."*
3. **Prove the patch-retention path (§1.1).** R4's fix shipped but never ran
   — no Work tore down dirty this sprint. A deliberately-dirtied Work would
   convert a shipped fix into a measured one.
4. **A brief-composition rule (§3.1).** Every prohibition in a shared
   preamble must be checked against each brief it is concatenated into, and
   a Work that is the exception must be told so explicitly.
5. **`work.backend`'s shape varying with routing tier (§4.3).** Decide
   whether that is intended; a client parsing it cannot rely on it today.
