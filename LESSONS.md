# Lessons

Memory file across gauntlet units. One lesson per entry, one-line summary first.
Corrections and confirmed approaches alike, with why they mattered. Update rather
than duplicate; delete what proves wrong. Entries marked **[world-delta]** are
candidates for promotion into the owner's knowledge corpus — promotion happens
only on the owner's explicit ask.

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
