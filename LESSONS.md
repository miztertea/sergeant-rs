# Lessons

Memory file across gauntlet units. One lesson per entry, one-line summary first.
Corrections and confirmed approaches alike, with why they mattered. Update rather
than duplicate; delete what proves wrong. Entries marked **[world-delta]** are
candidates for promotion into the owner's knowledge corpus — promotion happens
only on the owner's explicit ask.

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
