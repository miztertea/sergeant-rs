# Lessons

Memory file across gauntlet units. One lesson per entry, one-line summary first.
Corrections and confirmed approaches alike, with why they mattered. Update rather
than duplicate; delete what proves wrong. Entries marked **[world-delta]** are
candidates for promotion into the owner's knowledge corpus — promotion happens
only on the owner's explicit ask.

## L1 — Measure the Claude CLI, never trust its docs or its exit codes

From the fork's background-harness spike (see
`reference/sergeant-upstream/docs/research/claude-background-harness-spike.md`):
launch exit code 0 is compatible with three different model-pin failure shapes,
and a valid-but-unentitled alias silently substitutes another model, detectable
only in the transcript. Every adapter claim must be backed by a contract test
against the installed binary, re-run on version bumps. Spike measured 2.1.220;
this container has 2.1.226 — re-measure before trusting.
