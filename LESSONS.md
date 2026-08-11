# Lessons

Memory file across gauntlet units. One lesson per entry, one-line summary first.
Corrections and confirmed approaches alike, with why they mattered. Update rather
than duplicate; delete what proves wrong. Entries marked **[world-delta]** are
candidates for promotion into the owner's knowledge corpus — promotion happens
only on the owner's explicit ask.

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

S0/S1, 2026-08-10 (numbered assuming the N-branch's L11 lands first; renumber
at merge if it collides): three same-session orchestrator misses, one class.
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
