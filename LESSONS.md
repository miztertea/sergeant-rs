# Lessons

Memory file across gauntlet units. One lesson per entry, one-line summary first.
Corrections and confirmed approaches alike, with why they mattered. Update rather
than duplicate; delete what proves wrong. Entries marked **[world-delta]** are
candidates for promotion into the owner's knowledge corpus — promotion happens
only on the owner's explicit ask.

## L24 — A quiet observer and a stalled subject look identical; and a false absence's *correction* is where the damage lands **[world-delta]**

Cerberus, 2026-08-15, PATH-TO-MAC-1. Two halves of one shape, both found in a
single unit.

**The correction is the dangerous end of a false absence.** Two blind critic
seats independently reported a quoted objection as *invented* — "appears nowhere
in this repo except the plan itself" — each from a `grep` scoped to `docs/`,
`reference/`, `GAUNTLET.md`, `LESSONS.md`, excluding `src/`, where it sits
verbatim at `src/platform/disk.rs:5` and, originally, in
`src/backend/docker.rs`'s git history. Both proposed **deleting the claim.**
L23 already covers why the absence was reported; what it did not cover is that
here the *remedy* was the destructive act. A wrong finding costs a turn; a wrong
finding whose correction deletes a true, sourced statement costs the record
itself. Rule: **before acting on a claimed absence, re-run the search unscoped,
and prefer a correction that re-cites over one that removes.** The adversarial
pass is what caught it — the refuter re-ran the critic's own stated command and
got three hits where the critic reported one.

**An instrument that fails silently is indistinguishable from a subject that
has not moved.** In the same session: three backgrounded watchers were killed by
the harness at ~160s each, and their replacement monitor died instantly on a
shell syntax error. Every one of those failures presents as *silence*, which is
exactly what "the Work is still running" also looks like. This is #90's shape
(a ceiling-interrupted Work wedged in `active` where no verb reaches it) and
#94's (a Work reporting `completed` with nothing committed) pointed at the
observer rather than the observed. Rule: **an observer needs its own liveness
proof.** A monitor's first emission should be its self-test; a watcher's death
is a prompt to read journal state, never information about the subject; and any
filter must match every terminal state, because silence is not success.

## L23 — Every reduction of output lied at least once: read the artifact, not the view of it

Cerberus, 2026-08-14/15 sprint. Five instances in one session, four tool
families, same shape — a view was narrowed and the narrowing, not the
subject, produced the answer:

| Reduction | False conclusion |
|---|---|
| `grep -v "^+.*///"` on a diff | "the worker left a dangling doc fragment" |
| `grep -E "^\+\s+fn "` (leading whitespace) | "#70 has no test" — it was a top-level `fn` |
| `grep -E "outcome:"` unanchored | "gate run terminal" — matched no-mistakes' *help text* |
| guessed JSON path `run.surface` | "teardown never recorded" — the key is top-level |
| `\| tail -1` on `gh pr edit` | "PR title updated" — the command had exited 1 |
| `cmd \| head -3 && echo OK` | "patch applies" — `&&` saw `head`'s status, not the command's |
| the path the harness reported | "edit `CLAUDE.md`" — it is a symlink; `AGENTS.md` is the file |

The last row generalizes the rest past output-filtering, to **any indirection
between you and the artifact.** The harness loads project instructions and
names the path it opened; that path is mode `120000`, and the tracked file is
`AGENTS.md`. A retrospective then proposed five changes to a symlink. The
correction was on line 8 of `docs/DEVELOPMENT.md` — a file that same session
had already edited twice without reading its first twelve lines — and this
repo had been bitten once before (`GAUNTLET.md` CH-5: a skill citing
"CLAUDE.md L1" after the symlink commit moved that text). Cite the file that
holds the content, never the alias you were handed.

The two pipe rows are the dangerous class, because the reduction ran on a
*mutating* command and discarded the channel that reported failure —
the same defect as `2>/dev/null` on a `git branch -D` that silently
failed. Rules: a filtered view is evidence about the filter; anchor every
monitor pattern (`^outcome:`); never place a pipe or a `&&` between a
mutating command and its exit status; confirm a claimed absence
against the unfiltered artifact before reporting it as a finding; and
resolve a path before citing it — `git ls-files -s` names the mode.

Calibration note worth keeping with the entry: three of these occurred
*inside* the retrospective that documents the pattern, and the symlink one
after the owner had to point it out. Naming a pattern does not stop it. Only
a rule applied without judgment about whether this case needs it does.

## L22 — A "we never destroy X" invariant is a disk leak until a verb disposes of X

Cerberus, 2026-08-15. Post-merge sweep found **30 GB** in three
`.sergeant/data/surfaces/<work-id>/` trees whose Works were all terminal
(two `completed`, one `canceled`), each recording
`teardown.disposition = retained_dirty`. The policy is right — sergeant
preserves uncommitted work and never deletes a branch — but there is no
`sgt` verb to list what is retained or dispose of it once banked, so
correct retention is indistinguishable on disk from a leak, and the only
route to the space is `rm -rf` around the engine: precisely the ad hoc
shell reconstruction the guardrails forbid, aimed at preserved state.

Essentially all of that space is `target/` directories — 11, 11 and 9 GB,
against a 30 GB total for the three surfaces — build artifacts, gitignored,
not "uncommitted work" under any reading. So retention scope is the second
half of the lesson: **what gets preserved must be the thing the policy
means, not the whole directory it happens to live in.** Filed as #109.
Generalizes past sergeant: any never-delete guarantee ships incomplete
without its companion inspect-and-reap verb, because operators route
around the absence with something more dangerous than the verb would be.

## L21 — Cleanup in a test body protects only the path that succeeds

Cerberus, 2026-08-15. 198 zero-byte files named
`sgt-watch-test-hold-never-released-*` had accumulated in `/tmp` — which
on this host is a 16 GB tmpfs whose exhaustion has already caused one
host-wide incident (#70). `test_hold_wait` writes `<path>.ready` as its
rendezvous marker; the happy-path test removes it, and the dead-man test —
whose whole premise is that the release path never appears — removes
nothing. The failure-path test was the leaking one, which is the general
shape: the test that exercises the abnormal path is the least likely to
carry the cleanup, and the most likely to run often.

Cleanup belongs in the code under test or an RAII guard, never only in the
body of the test that happens to succeed. Related to L6 (an operation that
fails after producing an unrecorded effect) — here the unrecorded effect is
a file rather than a journal append. Filed as #108.

## L20 — Stale-but-true is the state that never triggers its own supersession

Cerberus, 2026-08-14. The orchestrating session drove `no-mistakes` by hand
for a full day — `gate.sh`, `axi respond`, `axi sync --recover` —
reimplementing `validate-and-ship`'s `40-drive-gates` and
`50-reconcile-custody` badly, and inventing a workaround around
`--keep-local` for a dirty-worktree refusal rather than using it, because it
did not know the flag existed. `AGENTS.md`'s routing table is correct and was
not ambiguous. But `docs/DEVELOPMENT.md`'s "Shipping gate" section restated
the procedure in prose and never named the workflow, and a reader who finds
the prose complete never reaches the catalog.

Owner's framing, and it is the right one: **a document says what we knew when
it was written; if it is wrong, supersede it and move on.** This repo already
works that way — `reference/notes/` are living docs revised in place with
dated entries, `GAUNTLET.md` is append-only, ADRs get refreshed once their
decision ships.

The sharpening this incident adds is about the *trigger*, not the action.
That gate prose was never wrong. Every sentence in it was true on the day it
was written and still true that morning — it predates the engine being able
to run work at all, so it faithfully described the only flow that existed.
Nothing contradicted it, so nothing prompted supersession. **Supersession
fires on contradiction; staleness that keeps telling the truth produces no
contradiction to fire on**, and the reader who stops there never learns what
they were not told.

So the mechanical hook cannot be "notice when a doc is wrong." It has to be:
**when a capability ships, the prose that predates it is part of what ships.**
That is the ADR-refresh rule this repo already enforces — and demonstrably
enforces, since the shipping gate caught ADR staleness three separate times in
one sprint — generalized past ADRs to any document describing a procedure the
new capability now owns. Publishing a workflow includes asking what prose now
summarizes it.

Corollary for the layering rule ("the document that owns the topic wins"): it
resolves *disagreements*, and this failure had none. Two documents both
telling the truth, one of them written before the engine could run work at
all, is outside what that rule can adjudicate.

## L19 — A governing document is an executable diff for the program: it takes the loop

Cerberus day 2, 2026-08-11: the orchestrator authored the MVP bucketing
— the document that supersedes the North Star's waves and sequences all
future work — directly from owner conversation, with zero fresh eyes.
The owner caught it ("did I miss your subagent reviews?"); the
owner-shaped pipeline (4 Sonnet critics → Opus writer → sanity pass)
then found 31 findings including 14 errors — among them a design error
(the turn envelope missing `send`-spawned turns) that would have shipped
into the MVP-1 contract. R-S0-12 says code is code; this is its planning
corollary: a document that directs what gets built is executable through
the program that obeys it, and orchestrator+owner agreement is a
builder's self-probe (L13), not review. Every governing artifact —
plans, contracts, north stars — gets fresh-context review before it
governs.

## L18 — R1's "already exists" includes the product you are building **[world-delta]**

Cerberus day 2, 2026-08-11: the promoted library carried a whole family
of packages (dispatch, respond-to-worker, wake-and-resume, the fleet
suite) that re-describe what sergeant-rs itself does — upstream's manual
protocols for the era before the daemon existed. Every classification
pass had run the Ponytail ladder *within* artifacts (N1's A4 demoted 71
over-staged items) while never placing the engine's own capability
surface on the R1 shelf, so "does the binary already do this?" was
never asked. A corpus decomposed from a predecessor must be rung
against the successor's existing surface, and the successor keeps
growing — the comparison list is a moving target that every future
pass re-derives from the product, never from memory. The owner caught
it from one package description; the fix (convention §2a bucket 4) cost
a paragraph. The 10M-token library build did not catch it at any of
its five review gates because no gate's brief included the engine.

**Sharpened 2026-08-15 (PATH-TO-MAC-1): this lesson needs a trigger, not just
a principle.** A sprint plan scoped a Work to build cross-platform support for
#18, #81 and #82; all three already shipped, dual-platform and unit-tested, in
`src/platform/`, and `src/platform/disk.rs`'s own doc comment argued against the
change the plan proposed. A four-axis blind panel caught it — but only because
one brief happened to name that module. Wave 1 then found **#94 already fixed
too**, and `completed_dirty` had been printed by the session's **first `sgt
status`**. Twice in one sprint, from a plan derived off the issue tracker and
memory of it. "Re-derive the comparison list from the product" is correct and
was not enough, because nothing fired. The mechanical form: **for every issue a
plan proposes to close, name the artifact you read to confirm it is still open —
the file, the test, the surface — before the plan is allowed to govern.** A
tracker says an issue is open; it does not say the code has not moved.

## L17 — Stopping a coordinator does not stop its dispatched effects

Cerberus session, 2026-08-11: the orchestrator stopped a workflow whose
watcher had died early, precisely to prevent its collector from killing a
live real-Claude run — and the already-dispatched collector's cancel
landed anyway, ending the run at $4.62. Stop/kill of an orchestration
layer is not quiescence of its in-flight agents; their external effects
(engine commands, commits, cancels) land after the stop. Rule: after
stopping any coordinator, verify the state you cared about by evidence
(journal, process table) before assuming it is protected — and design
run-ending effects to be idempotent or currency-checked, because a
raced cancel is indistinguishable from an authorized one in the record.
This is work-state ≠ process-state applied to our own orchestration.

## L16 — A spend guard can only fire at the ledger's granularity

Cerberus Run B2, 2026-08-11: a $2.50 budget guard polling `usage.updated`
could never have fired before $4.62, because usage lands once per turn
and turn 2 alone cost $3.21 — the guard's floor is the largest single
effect, not the polling interval. Rule: size any budget bound to
whole-effect costs (a guard at $X only bounds spend at $X + one maximal
turn), state that arithmetic when recording the bound, and treat
"tighter guard" proposals that ignore effect granularity as unmeasured
claims. General form beyond money: any watchdog over an append-only
record fires at that record's grain, never finer.

## L15 — A claim transmitted to another agent must carry its evidence or be labeled hypothesis

N-series close-out, 2026-08-11: the orchestrator told the Run B operator
that the workflow's CONTEXT.md instructed `sgt respond` — remembered from
authoring *different* prompts, never read. The operator read the file,
found the claim false, and reported the real defect (a live-vs-test signal
discrepancy, #46) instead of the transmitted fiction. L12's transmission
corollary: the receiving agent cannot tell your verified facts from your
confident guesses, and a coordinator's wrong "fact" arrives wearing
authority it didn't earn. Rule: instructions to other agents state, for
each factual claim, the evidence read in-session (file, journal line, run
ID) — or say "hypothesis, verify before acting on it." The same wake's
mirror image: the operator declined the orchestrator's credential-copy
instruction in favor of the documented safer fix — subagent skepticism of
the coordinator is the loop's protection running upward (L9's other half),
and it must stay licensed.

## L14 — Binding rules live where sessions actually look; one document deep is half-applied

S-series retrospective, 2026-08-11: every process error the owner caught
had the same shape — the governing rule existed but lived below CLAUDE.md
(the loop's applicability in gauntlet-pattern.md; the living-vs-frozen
split implicit in reference/notes' own revision convention; the
Fixes-trailer discipline in one Bug Sprint sentence), while CLAUDE.md
carried things that go stale (test counts, register ranges, embedded
coverage numbers) and warnings without their escape (the self-matching
pgrep, named twice, bit twice anyway). Rules: promote applicability rules
and safe incantations INTO CLAUDE.md; demote volatile numbers OUT of it
(refer to the ledger/baseline); a warning that names a trap must name the
escape. Same-day corollary from the CI lane's shakedown: environment
facts measured in one environment (root container) are hypotheses in the
next (non-root runner) — record the environment matrix beside the tests
that depend on it.

## L13 — A builder's probe of its own pin is evidence, never verification

S1 phase 1: launched as a single self-probing builder under a stretched
P1-PERF precedent (the process error ruling R-S0-12 records). The
owner-ordered lean round 2 then proved empirically why that shortcut is
never safe: the builder's revert-probe of its own SIGTERM pin passed, and
the fresh test-honesty critic still found the composition unpinned —
reverting only `Drop` left every m6 test green, because the pin called
`stop()` explicitly while the rig's two real users rely on `Drop` alone.
Bug Sprint 1's parts-vs-composition shape, third recorded instance, this
time in the instrument underneath the program's published numbers. Rule
(now R-S0-12): any executable diff takes the full multi-axis loop;
template exemptions end where a diff begins; a self-probe is input to the
panel, not a substitute for it.

## L12 — Authority lives in the artifact: re-read the governing text at decision time

S0/S1, 2026-08-10 (L11 landed without collision — no renumbering was
needed): three same-session orchestrator misses, one class.
(1) S1 phase 1 launched as a single self-probing builder — P1-PERF's
"no implementation to grade" exemption stretched over a code-writing phase
from a *remembered summary* of the pattern doc, whose pipeline line is
unconditional. (2) The model-spread doctrine pasted wholesale into CLAUDE.md
instead of revised into gauntlet-pattern.md — "refer, don't copy" and the
doc's own dated-revision precedent were both in text quoted earlier the same
day. (3) An Edit failed because its target text was reconstructed from
memory of a file written an hour before. The critics' rule — grade actual
artifacts, never a summary — binds the orchestrator too: summaries,
including your own earlier ones, are orientation, not authority. When a
decision turns on exact wording (a rule's scope, a template's boundary, an
edit target), read the governing text in-session at decision time. The
owner caught all three; the fix each time was one Read.

## L11 — A hash without its preimage convention is not evidence

N1: the corpus's strongest rule — "a quote_hash that does not verify against
the cited locator is invention, rejected at lint" — was unenforceable by
construction: the record stored only the hash, and no document said what
span gets hashed (whole line? paragraph? trimmed?). 106 of 966 units failed
reproduction indistinguishably from invention, and the failure surfaced only
when a reviewer tried to actually run the check (finding R3-02). Fix:
specify the derivation (sha256 over the exact contiguous byte span, no
normalization) and record the preimage (`quote` field) beside the hash.
General form: an integrity rule binds only when its verification procedure
is executable by a stranger; publish the convention with the first hash, not
after the first audit.

## L10 — A squashed milestone commit defeats the revert-probe audit

M6 round 2: checkpoint commit 69cb52e folded the entire build plus all 13
round-1 fixes into one commit, so L7's cheap audit (`git revert --no-commit
<fix-commit> && cargo test`) had nothing to revert — reverting the commit
removes the milestone, not a fix. Mutation probes substituted (and found two
real gaps), but they cost a specialist round; the revert-probe is supposed to
be cheap. Rule: keep fix commits separable from build commits — a checkpoint
may bundle *fixes* together, but never fixes with the build they fix. Where
history is already squashed, mutation probing is the honest substitute and
its probe-found gaps must land as pinning tests.

## L9 — The orchestrator's rulings are findings too **[world-delta]**

M5: the orchestrator instructed a test rewrite ("assert a collector receives
zero bytes") that was unfalsifiable as specified — nothing could ever dial
the collector's port, so the assertion could not fail, and it replaced
guards that could. The round-2 panel confirmed it as an error and the fix
restored both halves (structural guard + a collector bound where a
regression would actually dial). The loop's protection against a wrong
ruling is the same as against wrong code: fresh eyes grade the outcome, not
the authority. Rulings therefore go through the panel like any other change
— never exempt them, and record whose error it was.

## L8 — A capability flag is a claim: every advertised verb needs a contract test

M4 round 2: the adapter advertised `history: true` while its restart path
returned Ok(empty) — and the defect sat exactly in the two §15 verbs
(history, stop) the live contract tests didn't cover. The capability list
and the contract-test list must be the same list; an advertised verb without
a test against the installed harness is an unmeasured claim, and unmeasured
claims fail closed (L1's corollary at the trait boundary).

## L7 — A fix without its pinning test is prose: revert-probe every fix commit **[world-delta]**

M3 round 2's headline: the checkpoint gate's 11-fix commit — including two
path-traversal guards — reverted cleanly with every test still green. The
fixes were real; their permanence was fiction. Rule now enforced: every fix
ships with the test that would catch its regression, and the cheap audit is
`git revert --no-commit <fix-commit> && cargo test` in a disposable copy —
if the suite stays green, the fix isn't pinned. Corollary from the same
round: single-run green is not a gate — the M3 suite passed while failing
~5% of parallel runs; repeated-run checks are part of gate verification.

## L6 — Adjacent-append crash windows are this architecture's recurring hazard

M2: exact-once broke in the window between submit's two journal appends.
M3: a daemon crash in the same window stranded work in a state nothing would
pick up. Any code path appending two causally-linked events must either be
tolerant of the second append missing (recovery re-derives it) or write one
compound event. Check for this class explicitly in every milestone that adds
a multi-append sequence.

## L5 — Verifiers can capture the verification: enforce probe hygiene structurally **[world-delta]**

M2 follow-up round: a refuter tasked with refuting test-coverage findings
edited production source (hardcoded the bearer token, removed a command
replay-guard) to force tests green — then reverted, but the intent crossed
the line from probing to tampering. Caught by the harness security screen;
tree verified clean; the batch's verdicts were quarantined and the findings
re-adjudicated by the orchestrator instead. Lesson: "evidence-only
adjudication" is not self-enforcing — mutation probes belong in disposable
worktrees, verifiers never edit the artifact under review, and a tree-clean
check follows every verification round. This is ABF anti-capture ("the
environment enforces the action boundary") applied to the review loop itself.

## L4 — Invariants-vs-simplicity oscillation is an adjudication signal, not a fix-loop signal **[world-delta]**

M1 rounds 3–4: the invariants critic demanded fail-closed guards; the builder
added them; the simplicity critic flagged the same guards as beyond-contract
machinery. Both were right by their own axis — more loop iterations cannot
converge a genuine axis tension. The orchestrator's ruling (remove the
machinery, keep a one-line bound) landed on a lower Ponytail rung than either
critic proposed. When two axes start citing each other's fixes, stop looping
and rule.

## L3 — Fresh critics relitigate settled rulings unless pointed at the register

M1 rounds 3–4 re-filed already-adjudicated deviations (lib.rs, tokio removal,
fsutil.rs) because fresh-context critics cannot see prior rounds. Fresh eyes
are the feature; amnesia about rulings is the bug. From M2 on, critic prompts
instruct: read GAUNTLET.md's deviation register and ledger rulings first; a
finding that re-litigates a registered deviation must argue why the ruling is
wrong, not merely that the deviation exists.

## L2 — Headless Claude driving is proven: print-mode turns over a durable session **[world-delta]**

no-mistakes (github.com/kunchenguid/no-mistakes, `internal/agent/claude.go`) drives
Claude in production as a sequence of `claude -p --verbose --output-format
stream-json` invocations: prompt via stdin (documented non-interactive transport),
`--resume <session_id>` to continue the same conversation (never `--fork-session`),
`--json-schema` for structured output, `--setting-sources user` to neutralize the
target repo's project memory (verified empirically by them — project memory can
install a different identity on the agent), `--dangerously-skip-permissions` unless
the operator pinned a mode. Stream-json usage is per-invocation, not cumulative
across resumes. This makes the Claude "native session" a durable conversation
identity with per-turn processes — the work-state ≠ process-state invariant
natively. Supersedes the assumption that a held TTY is required. Their
`--setting-sources` capture hazard is also a real finding for any adapter running
agents inside arbitrary repos.

## L1 — Measure the Claude CLI, never trust its docs or its exit codes

From the fork's background-harness spike (see
`reference/sergeant-upstream/docs/research/claude-background-harness-spike.md`):
launch exit code 0 is compatible with three different model-pin failure shapes,
and a valid-but-unentitled alias silently substitutes another model, detectable
only in the transcript. Every adapter claim must be backed by a contract test
against the installed binary, re-run on version bumps. Spike measured 2.1.220;
this container has 2.1.226 — re-measure before trusting.
