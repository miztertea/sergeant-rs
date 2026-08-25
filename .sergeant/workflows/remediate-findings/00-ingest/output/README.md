# Output — `00-ingest`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This
directory is empty in the authored tree; a run of this stage writes its
artifact(s) here in the materialized work surface, Git-tracked on the Work
branch and reviewable in the diff like any other change.

**Expected artifact:** `ingest.md` — the accepted finding set (copied in
full) and the confirmed authorization, or the explicit refusal and its
reason.

**Required columns:** `id`, `axis`, `claim`, `evidence`, `severity`,
`status`, `refutation` — #260's structural gate: an accepted finding set
must carry every §2.7 column as a markdown table header, or the engine
refuses the same way it refuses a stage that never wrote `ingest.md` at
all. An explicit refusal (no accepted set at all) is not a table and
never reaches this check — it is asked for as `needs_input` by this
stage's own actor turn (CONTEXT.md's J0 list), before `StageCompleted`
is ever signalled.

**Disposition:** `evidence`
