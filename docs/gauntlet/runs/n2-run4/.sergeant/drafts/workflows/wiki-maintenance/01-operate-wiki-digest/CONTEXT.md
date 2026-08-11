# 01-operate-wiki-digest

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** a coordinator runs or regenerates a wiki daily digest

**Outcome:** digest changes are always previewed and inspected before they take effect and are always followed by verification

**Statement (the operative rule):** The daily-digest operating procedure is fixed: read SCHEMA.md first, dry-run before any regeneration or logic change, inspect the dry-run preview for secrets/duplicates/incorrect outcomes/generation errors, only then run the real command, verify the session page and index link exist, and finally append or verify the ingest log entry.

## What must become true here (durable outcome)

Digest changes are always previewed and inspected before they take effect and are always followed by verification — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0822`: A daily digest page must synthesize outcomes, decisions, blockers, and next state rather than reproduce the underlying conversation as a transcript.
- `BU-0823`: Installing or expanding a scheduler for the daily digest is treated as a task separate from ordinary digest operation.
- `BU-0825`: If SCHEMA.md is missing or unreadable, digest generation stops without writing any curated page.
- `BU-0826`: If a dry-run preview is found to contain secrets, generation stops, only the affected source class (not the secret content itself) is recorded, and redaction is fixed before retrying.
- `BU-0827`: If a PR or the task tracker task's state cannot be resolved during digest generation, its outcome is marked unresolved rather than inferred as complete.
- `BU-0828`: If regenerating a page would replace it with less information than it already has, the existing page is preserved and the rejected update is reported instead.
- `BU-0829`: If the wiki index update fails after a session page is generated, the generated page is kept, its exact path is reported, and the digest is left marked incomplete rather than the page being discarded or the failure hidden.
- `BU-0834`: The wiki-digest job refuses to run if the opencode session database does not exist.
- `BU-0835`: The wiki-digest job refuses to run if the opencode auth.json file (source of the Anthropic API key) does not exist.
- `BU-0836`: The wiki-digest job refuses to run if ~/wiki/SCHEMA.md does not exist.
- `BU-0837`: The wiki-digest job refuses to run unless both sqlite3 and python3 are available on PATH.
- `BU-0838`: The wiki-digest job reads the Anthropic API key out of the opencode auth.json file and refuses to run if no key is found there.
- `BU-0848`: A failed synthesis API call (HTTP error) is surfaced with its status code and response body on stderr and causes the process to exit nonzero, rather than the digest silently proceeding with no or partial content.
- `BU-0850`: The temporary prompt file (which may contain raw session content pulled from multiple sources) is deleted immediately after the synthesis API call completes, regardless of the call's outcome path shown here.
- `BU-0851`: A date whose session page already exists is skipped for real (non-dry-run) generation; the existing page is never automatically regenerated or overwritten, only manual deletion allows a re-run for that date.
- `BU-0854`: When a session page for the date already exists (only reachable in --dry-run, since a real run already skipped this date at BU-0851), its existing content is loaded and included in the synthesis prompt so regeneration is update-in-place rather than starting blind.
- `BU-0855`: In --dry-run mode the assembled synthesis prompt is printed and the loop moves to the next date without ever calling the synthesis API or writing a page.
- `BU-0856`: An empty synthesis result for a date is treated as an error and that date is skipped, rather than an empty session page being written.
- `BU-0858`: Every successful (non-dry-run) session page write unconditionally appends a fixed-format ingest log line to ~/wiki/log.md.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0830`: The wiki-digest job rejects an unrecognized CLI argument with a usage message rather than ignoring it or proceeding with defaults.
- `BU-0831`: Invoking the wiki-digest job with --since builds and processes one date per day from the given date up to (but not including) today, i.e. a backfill run, instead of a single date.
- `BU-0832`: The literal value "yesterday" given to --date is resolved to the actual calendar date one day before the run, not treated as a literal filename fragment.
- `BU-0833`: With neither --since nor --date given, the wiki-digest job defaults to generating a digest for today only.
- `BU-0839`: Opencode sessions considered for a day's digest exclude sub-sessions (sessions with a parent_id) and sessions still carrying the default "New session..." title.
- `BU-0840`: Per-opencode-session content pulled into the digest is limited to assistant-authored text parts longer than 20 characters, concatenated in chronological order and truncated to MAX_CHARS_PER_SESSION.
- `BU-0841`: Goose is an optional session source: if the goose sessions database does not exist, goose session extraction succeeds with no sessions rather than failing the run.
- `BU-0842`: Claude Code project history is an optional session source: if the claude projects directory does not exist, claude session extraction succeeds with no sessions rather than failing the run.
- `BU-0843`: A claude session is attributed to the digest date of its last (most recent) message timestamp, not its first message timestamp, and is skipped entirely if it has no resolvable timestamp.
- `BU-0844`: PR enrichment degrades gracefully when the gh CLI is unavailable: the digest still generates, with a "(gh not available)" placeholder instead of PR data, rather than the whole run failing.
- `BU-0845`: Merged-PR enrichment is scoped to a fixed, hardcoded list of four known repos, and within each repo only PRs whose mergedAt date matches the target digest date are included.
- `BU-0846`: Task enrichment degrades gracefully when the task tracker CLI is unavailable: the digest still generates, with a "(the task tracker not available)" placeholder instead of task data, rather than the whole run failing.
- `BU-0847`: Task enrichment includes a task in a day's digest only if the task was updated that date and its status is closed or review.
- `BU-0849`: The full synthesis prompt — which embeds raw extracted session content — is written to a temporary file before being passed to the API call.
- `BU-0852`: A date is skipped with no page written only if all three sources (opencode, goose, claude) return zero sessions; any one source having sessions is enough to proceed.
- `BU-0853`: The full text of ~/wiki/SCHEMA.md is injected verbatim into the synthesis prompt as the authoritative instructions on every digest run, rather than being paraphrased or hardcoded into the script.
- `BU-0857`: The wiki index is only appended a link for a date's session page if that date is not already referenced in the index, so re-running a digest for an already-indexed date never adds a duplicate index entry.

