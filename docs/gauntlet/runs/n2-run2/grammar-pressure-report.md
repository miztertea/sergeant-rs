# N2 — Grammar-pressure report

**What the ordered-actor grammar could and could not express, on the current
engine.** This is the N2 deliverable that licenses or defers Program B scope
(`docs/gauntlet/contracts/N2.md` Outcome §4; N2 Gate: "every engine-gap claim
carries failed lower rungs").

Compiled from the runs' own recorded evidence only —
`docs/gauntlet/runs/n2-run1/` and `docs/gauntlet/runs/n2-run2/`, their
journals, and the shipped engine source. Every claim is written to
`reference/proposal-next-iteration-icm-workflows.md` §6.7's template:

```text
behavior that cannot be represented
source evidence requiring it
lower-rung representations attempted
why each lower rung fails
minimum runtime capability required
observable acceptance test
```

§6.7's own disqualifier is applied without mercy: **"would be convenient" and
"could be more elegant" are not engine-gap evidence.** One of the two claims
the run itself filed does not survive that test and is rejected below.

## Verdict table, ranked by evidence strength

| # | Claim | Verdict | Evidence grade |
|---|---|---|---|
| GP-5b | `finalize.py` destroys uncommitted evidence; both self-reports claim it is recoverable | **Real defect — helper semantics, not a new durable fact** | **A** — git-verified, reproducible |
| GP-3 | Fresh context per stage | **CAPABILITY CONFIRMED — not pressure** | **A** — full-corpus scan, 0 violations |
| GP-4 | Backend execution identity does not survive daemon restart | **Engine failed closed correctly.** 2 residuals | **A** — journal seq 32–42 + engine source |
| GP-2 | No actor-initiated mid-run ask primitive | **CONFIRMED engine gap** (narrow) | **A** — engine source: no producer, no endpoint |
| GP-5a | Generated drafts fail `[S12]` (no finalize step) | **Authoring-guidance defect — no engine need** | **A** — template/exemplar diff |
| GP-1 | Harvest turn-budget / volume wall (16 of 136 files) | **Phenomenon real; engine-gap claim REJECTED** — a lower rung was never attempted | **A** for the wall, **F** for the claim |
| GP-6 | Zero-findings review stage | **Stage-context guidance gap** | **B** — checklist text; counterfactual reasoned, not measured |
| §21.8 | Workflow composition | **Untriggered by this run** | **A** — trigger conditions unmet on the run's own evidence |

---

## GP-1 — The harvest turn-budget / volume wall

**Filed by the run** as `90-reconcile/output/grammar-pressure.ndjson` record 1.
**Adjudicated verdict: the wall is real and important; the engine-gap claim is
rejected at §6.7's rung-discipline axis and downgraded to an authoring
problem.**

### The wall, as measured

| Fact | Value | Source |
|---|---|---|
| `decompose` files inventoried | 136 | `10-inventory/output/inventory.md` |
| Files reached by `20-harvest` | **16** | re-enumerated from `20-harvest/output/behavior-units.ndjson` (16 distinct `source.path`; the run's own "18" is wrong — scorecard D-1) |
| Behavior units produced | 108 | same |
| `20-harvest` attempt-1 wall clock | **6165.6 s (1 h 42 m 46 s)** | journal `stage.entered` 13:11:25.440 → `stage.input_received` 14:54:11.024 |
| Named partitions available over the `decompose` set | **5** (two of them further split into 5 and 7 sub-groups) | `measurement-package.md`, "Partition count" |

Sub-file truncation confirms it is a context wall, not a file-selection
choice: `README.md`'s orientation/genesis region (`BU-P1-062`–`BU-P1-067`) and
`AGENTS.md`'s six-rule skills-routing table (`BU-P1-132`–`BU-P1-137`)
produced nothing while later regions of the same two files were extracted.

### §6.7 template, as the run filed it

- **Behavior that cannot be represented** — extracting behavior units from a
  136-file `decompose` set within a bounded number of actor turns, so a
  harvest run covers every file rather than leaving most unreached.
- **Source evidence** — `20-harvest/output/coverage-note.md` (one citation;
  see GP-5b — the file no longer exists).
- **Lower rungs attempted** — `stage`, `helper`.
- **Why each fails** — *stage*: the stage list is fixed in `workflow.toml` at
  definition time and cannot be multiplied per-run from a subject
  repository's file count. *helper*: a helper can enumerate and hash but
  cannot perform the judgment of recognizing a behavior in prose.
- **Minimum runtime capability** — a durable fan-out: one stage spawning a
  bounded set of actor sub-turns over a partitioned input set, with the
  daemon tracking done/pending/failed per partition and aggregating before
  the stage completes.
- **Observable acceptance test** — one `behavior-units.ndjson` covering every
  decomposed file, with the journal showing more than one actor turn and the
  daemon tracking partition completion.

### Why the claim is rejected

**R1 — The `needs_input`/`respond`/`retry` loop rung was never attempted, and
this run's own journal proves it works.**

The rung list names `stage` and `helper` and stops. It never names the
re-enterable-stage loop that already ships: a stage that ends in
`needs_input` (or `waiting`/`blocked`), receives a response, and is
**re-entered as a fresh execution** via `sgt work retry`. Run 2 exercised
exactly that mechanism by accident:

```
seq 40  14:55:15.143  work.resumed   {"reason":"retry","stage_id":"20-harvest"}
seq 41  14:55:15.147  stage.entered  {"attempt":2,"index":2,"stage_id":"20-harvest"}
seq 42  14:55:15.148  execution.started {"attempt":2,...,"execution_id":"01KZP2QD6B..."}
```

`20-harvest` was re-entered as attempt 2 with a fresh execution and fresh
actor context, appending to the same worktree artifacts. Loop that N times —
one partition per attempt, `10-inventory`'s own 5 named partitions as the
work list — and the acceptance test's second clause ("the journal shows more
than one actor turn was used") is already satisfied today.

**This is not a novel objection. It is the ruling that killed the
reference's own G5.** `reference-corpus/engine-pressure.md` records G5
rejected at `adjudication-round1.md` A13 on the premise that its "never
attempted" lower rung — a re-enterable `needs_input` stage — *already ships*,
and that the first pass had only considered a coarser variant. GP-1 repeats
G5's mistake at a coarser grain: it never considered the loop at all.

**R2 — The fixed-stage rung is refuted with a requirement stronger than the
problem.** The claim rejects per-partition stages because a workflow cannot
declare "as many turns as this file count needs." But the run does not need
unbounded fan-out; it needs 5 (or 17, counting sub-groups). A workflow can
declare `20-harvest-p1 … 20-harvest-p5` against `10-inventory`'s *own named
partitions*, each with a no-op-if-empty rule. Substituting an unbounded
requirement for a bounded need is precisely the "would be more elegant"
move §6.7 disqualifies.

**R3 — The engine did not impose the bound.** The engine held `20-harvest`
open for 1 h 42 m 46 s with no timeout, no turn cap, and no pressure of any
kind; the fake backend's `needs_input` hold has no expiry. The bound was the
*actor session's* context window. §6.7 asks whether the **runtime** must own
the semantics. A durable partition ledger would be a coherent runtime-owned
fact — but a capability is only licensed when the cheaper rungs have been
tried and failed, and here they were not tried.

### Adjudicated disposition

**Downgrade to authoring guidance.** The next `repo-to-icm` version should:
(a) split `20-harvest` into per-partition stages bound to `10-inventory`'s
named partitions; (b) give the harvest stage an explicit re-entry protocol —
end in `needs_input` with the remaining-partition list as the prompt, resume
via retry; (c) require `20-harvest`'s `output/README.md` to declare the
partition ledger as a `promote` artifact so coverage state is durable
(GP-5b).

**Re-file as an engine gap only if** a run that actually executes (a)+(b)
still cannot cover the set — for example if per-partition retry loses the
aggregation invariant, or if partition bookkeeping held in actor-written
files proves unrecoverable across a daemon restart. That would be new
evidence. This run produced none.

---

## GP-2 — No actor-initiated mid-run ask primitive

**Verdict: CONFIRMED engine gap.** Narrow, well-evidenced, and the only claim
in this report that licenses new Program B scope.

- **Behavior that cannot be represented.** An actor, *inside its own turn*,
  discovering that it cannot proceed without a human decision, and pausing
  the stage with that specific question attached — so the human answers the
  actual question and the same stage resumes. Today the actor's only
  fail-closed move is to write a marker and stop, which ends the run.

- **Source evidence.**
  1. Run 1, `00-contract/output/contract.md` meta-note (relayed in
     `docs/gauntlet/runs/n2-run1/run-manifest.md` §4): "the engine gives no
     actor stage a way to pause its own turn and ask a human a disambiguating
     question mid-run — `needs_input`/`waiting` is runtime-driven from
     outside the actor's turn, never actor-requested. The only fail-closed
     action available was to write the marker and stop."
  2. Run 1's outcome is the cost, measured: the initiating task named no
     subject repository and no revision; `00-contract` ran six checks, found
     no pin, wrote `# AMBIGUOUS — NOT RESOLVED`, and **all ten stages
     produced nothing**. One answerable question ("is the subject
     `reference/sergeant-upstream`, and at what revision?") consumed an
     entire run. Run 2 answered it out-of-band by pinning `UPSTREAM.md` into
     the subject commit before starting.
  3. Engine source. `BackendSignal::NeedsInput { prompt }` exists
     (`src/backend/mod.rs:286`) and its doc comment reads *"What the actor is
     asking"* — but the only producers anywhere in the tree are in
     `src/backend/fake.rs` (lines 91, 550, 665), where it is **scripted by
     the test harness**. `src/backend/claude.rs` never constructs it. The
     API's only input-shaped route is `POST /work/{id}/input`
     (`src/api.rs`) — the *delivery* direction, from outside in. So the
     signal shape exists, is unreachable from any real harness, and has no
     inbound counterpart at all.

- **Lower rungs attempted.**
  - *Stage* — pre-declare a `needs_input` checkpoint before the judgment.
  - *Stage context* — instruct the actor to pause and ask.
  - *Helper* — a script that writes the question somewhere durable.
  - *Fail-closed marker* — what run 1 actually did.

- **Why each fails.**
  - *Stage*: the hold is authored before the run and cannot know the
    question. Run 1's ambiguity was discovered *during* `00-contract`'s sixth
    check; no pre-declared prompt could have carried it.
  - *Stage context*: guidance cannot create a transition. The actor can be
    told to ask; it has no verb with which to ask. `sgt work respond` is the
    human's verb, and it requires the work to already be in `needs_input`.
  - *Helper*: writing a question to a file produces no state change, no
    event, and no notification. Nothing wakes anyone; the turn still has to
    end. The journal — the only truth — records nothing distinguishable from
    an actor that simply stopped.
  - *Fail-closed marker*: measured in run 1 as the whole-run cost above. It
    is correct behavior and it is also total loss: `_config/run-discipline.md`
    §2 forecloses ordinary adjudication once the marker is set, so even the
    two genuine findings `80-adversarial-review` produced were left
    undisposed.

- **Minimum runtime capability required.** A backend-originated signal the
  engine already models — `BackendSignal::NeedsInput { prompt }` — reachable
  from a real harness turn, carrying the actor's own question text, and
  landing the stage in `needs_input` *without consuming the attempt*, so the
  delivered answer resumes the same stage rather than restarting it. This is
  a wiring and protocol question, not a new durable fact class: the event
  kinds (`stage.needs_input`, `work.needs_input`, `stage.input_received`),
  the state, and the delivery endpoint all already exist and are exercised in
  run 2's journal (seq 30, 35).

- **Observable acceptance test.** A stage whose actor emits the agreed
  in-turn ask marker produces `stage.needs_input` carrying that actor's
  literal question as the prompt; `sgt work show` displays it; delivering an
  answer via `POST /work/{id}/input` produces `stage.input_received` and
  resumes **the same stage at the same attempt**, with the answer in the
  journal. Reverting the wiring makes the test fail with the actor's question
  absent from the prompt.

**Scope discipline.** This licenses one narrow item. It does **not** license a
general bidirectional actor↔runtime RPC channel, an approval-flow capability,
or a question queue. The measured need is: one question, one prompt, one
resumption.

---

## GP-3 — Fresh context per stage: CAPABILITY CONFIRMED, not pressure

Recorded here because a confirmed capability is as much a measurement result
as a gap, and because U2 in the N2 contract asked precisely this.

**Claim: stage-scoped actors given only their stage's pinned context
reproduced the workflow without cross-stage leakage.** Verified three ways
over run 2's complete artifact set:

1. **Declared-inputs conformance (full-corpus scan, run for this report).**
   For each of the 10 stages, every `NN-stage/output` reference appearing
   anywhere in that stage's own output artifacts was checked against the
   stage's `CONTEXT.md` Inputs table. **Nine of ten stages: zero undeclared
   upstream reads.** The tenth is `80-adversarial-review`, which cites
   `10-inventory/output/inventory.md:10` and
   `20-harvest/output/coverage-note.md:76` — both outside its Inputs table
   and both *required* by its own L3 checklist ("Grep every artifact this run
   has produced so far", `references/challenge-checklist.md`). Authorized
   reading, under-declared table — see the residual below.
2. **Blindness held.** `reference-corpus/` was never opened. The review
   stage's literal grep found 3 hits across every artifact, all classified as
   exclusion-policy prose, none inside a `source.path`/`locator`/`quote`/
   provenance/finding-evidence field. Independently corroborated: the run
   worktree contained no `reference-corpus/` directory at all.
3. **Layer discipline held.** All 6 member stages across the materialized
   candidates carry `output/` directories containing only `README.md` — no
   Layer-4 artifact fabricated at draft time — and every Inputs row is
   correctly tagged L1/L3/L4.

**Positive consequence, and the reason this matters more than it looks:**
the two genuine blind convergences in the scorecard —
`standard-task-workflow` recovering `task-intake-and-route`'s numbered steps,
and `ship-with-no-mistakes` naming `start-run`/`drive-gates`/`finish-run`/
`route-findings` — were produced by actors with no access to the reference
*and* no access to each other's reasoning. Fresh-context-per-stage is what
makes those results evidence rather than coincidence.

**Residual (minor, authoring-level).** The Inputs table is not the complete
pinned set for `80-adversarial-review`: its L3 checklist widens its reach
beyond what the table declares. Fix in the workflow, not the engine — either
add the two rows to the table, or narrow the checklist to the declared set.
Left as-is, a future reviewer cannot tell an authorized wide grep from a
leak by reading the table.

---

## GP-4 — Backend execution identity does not survive daemon restart

**Verdict: the engine failed closed correctly. This is evidence the recovery
invariant works. Two residuals attach.**

### What happened, from the journal

```
seq 30  13:11:25.442  work.needs_input      {"prompt":"hold-stage-2","stage_id":"20-harvest"}
seq 32  14:43:06.160  daemon.started        {"pid":2184}          ← restart, new daemon
seq 33  14:43:08.959  backend.probed        {"backend":"claude"}
seq 34  14:43:08.960  backend.probed        {"backend":"fake"}
seq 35  14:54:11.024  stage.input_received  {"stage_id":"20-harvest"}
seq 36  14:54:11.025  work.resumed          {"reason":"input_received"}
seq 37  14:54:11.025  stage.blocked         {"detail":"backend \"fake\" does not recognise execution 01KZNWS9G19T83S9WXEENP9856"}
seq 38  14:54:11.026  work.blocked          {"reason":"cannot deliver input: backend \"fake\" does not recognise ..."}
seq 40  14:55:15.143  work.resumed          {"reason":"retry","stage_id":"20-harvest"}
seq 41  14:55:15.147  stage.entered         {"attempt":2,...}
```

The fake backend's execution table is in-memory. The restart at 14:43:06
evaporated it. When the actor responded 11 minutes later, `respond` could not
resolve the execution and the engine **blocked with the literal evidence**
(`BackendError::UnknownExecution`, `src/backend/mod.rs:338`) rather than
guessing the stage had completed. `sgt work retry` then re-entered
`20-harvest` as attempt 2 against artifacts already on disk. No content lost.

This is CLAUDE.md's stated invariant — *"ambiguity fails closed into `blocked`
with a reason, never a guess"* — holding under a real, unplanned restart. It
is the strongest single piece of runtime evidence either run produced, and it
is a **confirmation, not pressure.**

### Residual R-1 — the restart itself reconciled nothing

`src/runtime/recovery.rs` resumes works that were *in flight* and fails closed
on ambiguity; a work parked in `needs_input` is "exactly where its last
explicit signal left it" and is left alone. Correct by its own contract — and
the observed consequence is that a work holding a now-dead execution id sat
apparently healthy for 11 minutes, and would have sat there indefinitely. The
failure surfaced only when the actor tried to deliver **1 h 43 m of work**.

Not an engine gap (no new durable fact is needed — the run record already
names the backend and execution id). It is a recovery-completeness question:
should reconciliation validate the execution handle of a `needs_input` work
against its backend at startup, and mark it `blocked` immediately with the
same evidence? The cost of not doing so is measured here at 11 minutes of
false-healthy state on a run where a single actor turn was worth 1 h 42 m.

**Acceptance test if pursued:** start a work, drive it to `needs_input`,
restart the daemon, and assert `stage.blocked` with the
`does not recognise execution` evidence appears at reconciliation time — not
at the next `respond`.

### Residual R-2 — an L6 adjacent-append window on the same code path

`Engine::respond` (`src/runtime/engine.rs` ~L470–495) appends
`stage.input_received`, then `work.resumed`, and **then** calls
`backend.send(...)`:

```rust
self.commit(core, work_id, KIND_STAGE_INPUT_RECEIVED, json!({...}))?;
self.transition(core, work_id, KIND_WORK_RESUMED, json!({"reason":"input_received"}))?;
if let Err(e) = backend.send(&handle_of(&execution), input) { /* stage.blocked, work.blocked */ }
```

A crash after either append leaves the journal asserting the input was
received and the work resumed, while the backend never received it. This is
exactly the adjacent-append class LESSONS L6 names and CLAUDE.md requires
checking on any journal-touching change — and run 2's journal is a live
instance of the branch where the send fails (seq 35→37 landed 1 ms apart, on
the very first real exercise of this path).

Not an engine gap either: the remedy is at the lower rung — journal the
delivery attempt as one compound event, or append `stage.input_received`
only after `backend.send` returns. Recorded here as a defect for the runtime
line (N0/N3 remediation), with a failing-on-revert test as the acceptance
bar per LESSONS L7.

---

## GP-5 — Finalize and disposition: one authoring fix, one real defect

### GP-5a — `[S12]`: generated drafts name no finalize step

**Verdict: authoring-guidance defect. No engine need. Fixable in one file.**

Two of three packages fail `[S12]` (`dispatch-mode`/`10-dispatch-worker`,
`standard-task-workflow`/`50-reconcile-and-deliver`). The rule
(`scripts/validate-structure.py:441-467`) is mechanical: if any stage's
`output/README.md` declares an artifact, the closing stage's `CONTEXT.md`
must contain the literal token `finalize`.

The diagnosis is a three-line diff of the run's own tree:

| Artifact | S12 half it covers |
|---|---|
| `repo-to-icm`'s own tree (the exemplar) | **Passes S12** — `90-reconcile/CONTEXT.md` names `finalize.py` in its Inputs table and at step 4 |
| `60-draft/references/draft-package-template.md` (what the drafting actor is handed) | Names artifact declaration + disposition — the half that **triggers** S12. Contains the word "finalize" **zero** times |
| `60-draft/CONTEXT.md` Inputs table | Does not list `scripts/validate-structure.py` — the drafting stage never sees the rule it is graded on |

`70-lint` handled the consequence correctly: it classified `[S12]` substantive
rather than force-fixing it, quoting each closing stage's own admission that
it is "orientation-level … not a finished, ready-to-run stage contract." That
is the right call under `references/mechanical-vs-substantive.md`. But the
defect is upstream of the lint: the template mandates the trigger and omits
the satisfier.

**Fix:** add the closing-stage finalize step to
`draft-package-template.md`, and add `scripts/validate-structure.py` to
`60-draft`'s Inputs table so the drafting actor can self-check before `70-lint`
sees it. **No engine capability is implicated.**

### GP-5b — Finalize destroyed the run's own coverage evidence, and two artifacts claim otherwise

**Verdict: real defect in the finalize helper's semantics. Grade A evidence.
Not a new durable fact — a helper ordering bug plus two false claims.**

- **What happened.** `scripts/finalize.py` removed
  `20-harvest/output/coverage-note.md` because
  `20-harvest/output/README.md` declares only `behavior-units.ndjson` as
  `promote`. The dry-run plan is printed verbatim in
  `measurement-package.md` ("would remove
  20-harvest/output/coverage-note.md"; exit 0) and the apply was committed as
  `b1546e9`.
- **The claim.** `measurement-package.md`: content "recoverable from
  Work-branch history regardless, per D9 convention."
  `run-manifest.md`: "remains recoverable from Work-branch history per the
  D9 convention."
- **Both are false, verified.** On the retained work repo, branch
  `sergeant/01KZNW46C3Y2W890DE7S8M94NZ` holds exactly two commits. The
  finalize commit `b1546e9` is **`40 files changed, 2721 insertions(+)`, zero
  deletions**; `git log --all --diff-filter=A -- "*coverage-note.md"` returns
  nothing. The file was deleted from an **uncommitted** working tree, so it
  was never in any tree at any revision. It is gone.
- **Why it matters beyond one file.** The destroyed artifact was
  (i) the primary record of which 120 files were not reached — the run's
  entire coverage-honesty claim; and (ii) the **sole `source_evidence`
  citation** for grammar-pressure claim GP-1. All three comparison sections
  cite it; none could open it.

**Lower rungs, per §6.7.** *Helper*: `finalize.py` already is the helper —
the fix is ordering, `git add -A` before applying removals so every removal
is a recoverable deletion rather than an unrecorded one. *Stage context*:
instruct `90-reconcile` to commit before finalizing (`90-reconcile/CONTEXT.md`
already sequences finalize at step 4 for a related reason — "`finalize.py`'s
`git rm` fails on a file that…"). Both rungs work. **No runtime capability is
required; the claim stops here.**

**Acceptance test:** run `finalize.py` on a tree containing an undeclared,
uncommitted artifact; assert the artifact is retrievable from the work branch
after the closing commit.

### GP-5c — The drafts-outside-`output/` claim (run 2's grammar-pressure record 2)

**Verdict: the observation is correct; the engine-gap framing is premature.**
`60-draft` writes its principal deliverable to `.sergeant/drafts/workflows/`,
outside any stage's `output/`, where D9 dispositions and `finalize.py` cannot
reach it. The record's rung analysis is honest and specific (a stage's
`output/README.md` binds only filenames under that same stage's `output/`;
`parse_readme` never looks elsewhere).

But the minimum capability it asks for — "a per-run disposition registry …
for content written anywhere under the worktree" — is a *convention* scoping
question before it is a runtime one. The untried rung: let a stage declare
its `output/` artifact to *be* a manifest of externally-materialized paths,
and let `finalize.py` follow that manifest. That is deterministic machinery
reading a declared file, squarely inside the helper rung. **Defer.** Re-file
if a manifest-following finalize proves unable to express a disposition some
real run needs.

Note the ordering defect that produced this report's GP-5b is the *same
mechanism's* symptom at the other end: content inside `output/` gets a
disposition and loses recoverability; content outside gets neither. Fix them
together.

---

## GP-6 — The zero-findings review stage: what the next workflow version needs

**Verdict: stage-context guidance gap. Grade B** — the checklist text is hard
evidence; the counterfactual ("this check would have caught it") is reasoned,
not measured.

`80-adversarial-review` returned 0 findings across 3 axes × 3 severities, and
the pass was genuinely effortful: 29 citations re-hashed spanning all 16
source files, 16 provenance ids re-confirmed, §6.3 re-applied to all 6 `stage`
records, a literal blindness grep with all 3 hits individually classified.
Its restraint on reference-dependent dimensions is *correct discipline* —
a blind reviewer cannot check recall or boundary agreement against a corpus
it is forbidden to open, and `measurement-package.md` declares those five
dimensions out of scope accurately.

The zero is nonetheless not a quality signal, and the reasons are structural
and fixable in `references/challenge-checklist.md`:

**1. Make the ladder test bidirectional (highest value).** Axis 2 currently
reads *"For every `representation: stage` record, re-apply the reimplementation
test"* — promotions only. Nothing asks the demotion question of the 79
`shared-helper` records. That single asymmetry is why the run's largest
disagreement (`shared-helper` 9.3× overrepresented; scorecard
"Representation agreement") passed unflagged. **New wording:** *for every
record on either side of the §6.3/§6.5 boundary, state which question was
answered first; a `shared-helper` classification whose rationale answers §6.5
without recording a §6.3 answer is a finding.*

**2. Give Axis 3 a generative half.** Axis 3 reads *"For every
`representation: engine-gap` record…"* and with zero such records reported
itself, correctly, "applied vacuously." A refutation-only axis cannot detect
the failure mode that actually occurred — that **no** unit was routed to §6.7,
including 20 units from the four files (`bin/sgt-wake`,
`bin/sgt-dag-dispatch-hook`, `bin/sgt-dag-run`, `bin/sgt-drain`) that carry the
reference's own in-scope engine-gap units. **New wording:** *if the corpus
contains zero `engine-gap` records, that is itself the thing to challenge —
name at least three behaviors whose §6.5 rationale is closest to a
runtime-ownership question and record why each does not clear §6.7. A
vacuous axis is a finding of its own.*

**3. Add a cross-artifact consistency axis.** The review wrote "16 distinct
`source.path` values" while the upstream artifact open on its own desk said
18. Nothing required reconciling the two. **New wording:** *every count an
upstream artifact asserts about another upstream artifact must be recomputed;
a mismatch is a finding regardless of which side is right.*

**4. Route lint defects into the findings channel.** `[S12]`×2, `[S3]`,
`[S10]` and the `contract.md`/`inventory.md` discrepancy were correctly
generated, correctly classified substantive, correctly not re-litigated by
`80-adversarial-review` — and then fell outside `90-reconcile`'s
accept/reject/park mechanism, which by its own scope reads only
`findings.ndjson`. **Five known defects exit the pipeline with no disposition
anywhere.** Fix in the workflow shape: either `70-lint` writes substantive
defects into `findings.ndjson`, or `90-reconcile`'s adjudication scope
explicitly includes `lint-report.md`.

**5. Align the Inputs table with the checklist's reach** (GP-3's residual).

None of the five requires an engine change.

---

## Where genuine fan-out / composition pressure begins (§21.8)

§21.8's trigger requires procedures that are (i) invoked from more than one
parent workflow, (ii) need independent durable entry / retry / block /
measurement / recovery, (iii) cannot be represented by shared context or
helpers, and (iv) create harmful duplication when inlined.

**This run triggers none of them, and GP-1 is not composition at all.**
Harvest partitioning is homogeneous iteration inside one parent: no partition
needs its own identity, no partition is invoked from a second workflow, and
inlining creates no duplication. Conflating "run this stage N times" with
"invoke a child workflow" would license N7 on iteration evidence — precisely
the overfitting §9.10 warns against ("a grammar feature should not be promoted
because one historically complex Bash project made it tempting").

Where real composition pressure lives, on the *reference's* evidence and not
this run's:

- `reference-corpus/engine-pressure.md` **G6** — "conditional invocation of a
  named child procedure with its own checkpoints" — is the corpus's own
  composition claim, and its adjudicated verdict is **survives partially,
  downgraded to grammar pressure, ranked last (6th)**. Its four named losses
  under inlining (parent/child visibility, cancellation reaching into the
  child, parent-aware recovery, per-subworkflow telemetry) are the honest
  statement of what composition would buy.
- G6's evidence is `skills/dispatch/SKILL.md` L168–173,
  `templates/worker-brief.md`, and the `cross-repo-work`/`dispatch`
  duplicate-reconciliation observation (N1 adjudication A8/BH-10) — **all
  outside this run's 16-file scope.** Run 2 contributes zero independent
  evidence toward the trigger.
- The nearest thing run 2 *could* have contributed, it did not: the reference
  resolves the 14 fleet-lifecycle files this run read into seven independent
  packages (`drain-fleet`, `recover-stalled-worker`, `respond-to-worker`,
  `wake-and-resume`, `monitor-fleet`, `reconcile-and-cleanup-fleet`, plus
  `dispatch`), several of which are plausible multi-parent callees. Run 2
  produced one single-stage package instead — downstream of the
  `shared-helper` misclassification, not of any grammar limit.

**Conclusion: §21.8 stays untriggered.** Re-open only when a run whose
coverage reaches `skills/dispatch/SKILL.md` and `templates/worker-brief.md`
independently reproduces G6's seam, *and* names at least two distinct parent
workflows invoking the same child procedure.

---

## What this report licenses

| Scope | Disposition |
|---|---|
| **Actor-initiated mid-run ask** (GP-2) | **Licensed for Program B**, narrowly: wire `BackendSignal::NeedsInput` to a real harness turn carrying the actor's own question; resume the same stage at the same attempt. Nothing broader. |
| Durable harvest fan-out (GP-1) | **Deferred.** Lower rung (`needs_input`/retry loop, per-partition stages) never attempted; this run's own journal shows it working. Re-file only with evidence from a run that tried it. |
| Per-run disposition registry (GP-5c) | **Deferred.** Manifest-following finalize is an untried helper-rung solution. |
| Recovery completeness on restart (GP-4 R-1) | **Runtime-line defect**, not Program B. Reconcile `needs_input` works' execution handles at startup. |
| `respond()` adjacent-append window (GP-4 R-2) | **Runtime-line defect** (LESSONS L6 class). Compound event or reorder; failing-on-revert test required. |
| `finalize.py` destroys uncommitted evidence (GP-5b) | **Helper defect.** Commit before removing. |
| `[S12]` template gap (GP-5a) | **Workflow-authoring fix.** One template, one Inputs row. |
| Review-stage guidance (GP-6) | **Workflow-authoring fix.** Five checklist amendments; none needs the engine. |
| Workflow composition (§21.8) | **Untriggered.** No evidence from this run. |

Program B is licensed for **one** capability, at the smallest scope the
evidence supports. Every other "could not" in this run resolved to an
authoring fix, a helper ordering bug, a runtime-line defect on the existing
line, or a lower rung nobody tried.
