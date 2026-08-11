# 02-preserve-retry-evidence-on-failure

## Inputs

| File | Layer | Why |
|---|---|---|

## Purpose

**Trigger:** a review-finding route fails after the findings have been parsed

**Outcome:** the parsed, sanitized findings are durably retained with an exact retry command surfaced

**Statement (the operative rule):** When a review-finding route fails after parsing, the sanitized findings are retained under `<worktree>/.sergeant-review-artifacts/<axis>-<source>/` and the blocked message names the exact retry command.

## What must become true here (durable outcome)

The parsed, sanitized findings are durably retained with an exact retry command surfaced — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0104`: The retained review-artifact holds only post-redaction fields, never the reviewer's original output, and a retry re-validates every field and recomputes each content digest before anything reaches the task tracker; a route refuses to overwrite an artifact nobody has retried yet.
- `BU-0312`: Malformed reviewer output or a routing failure is treated as blocked; the worker never continues past it or leaves the findings only in worker.log.
- `BU-0716`: A --retry artifact path must resolve to a location strictly inside this worktree's own artifact root, matching a safe filename pattern, or the retry is refused.
- `BU-0717`: A retry artifact must contain both a findings file and a meta file, or the retry is refused as incomplete.
- `BU-0718`: A retained artifact's stored project and repo (from its meta file) must exactly match the project/repo the retry was invoked against, or the retry is refused.
- `BU-0719`: An unrecognized key in a retry artifact's meta file causes the retry to be refused rather than silently ignored.
- `BU-0735`: A retried (replayed) review artifact is re-validated against the exact same rules used for fresh input, because a retry replays a retained artifact without the reviewer's original JSON and without re-running the redaction step, so the same content-safety guarantees have to be re-established independently rather than assumed from the fact that the artifact was retained.
- `BU-0736`: A retained review artifact is never allowed to be empty; only a fresh (non-retry) parse of a review with genuinely zero findings is allowed to be empty.
- `BU-0739`: On retry replay, each finding's content digest is re-derived from its own field content and compared against the stored digest; a stored digest can never certify text that has since been altered.
- `BU-0740`: The review-findings router refuses to overwrite an existing pending retained artifact for the same axis/source — the existing one must be retried first, since by construction it is the only surviving copy of that review until it is.
- `BU-0741`: Retaining parsed findings stages them into a private temporary directory and publishes the whole artifact (findings + meta) with a single atomic rename, so a crash partway through can never leave findings on disk without its matching meta, and two concurrent invocations cannot collide on the same temp name.
- `BU-0742`: Removing a retained artifact after successful routing treats a removal failure as non-fatal (a warning, not an abort), while continuing to name the still-present artifact so a later routing failure's retention message stays truthful rather than falsely claiming nothing was kept.
- `BU-0743`: A fresh review that parses to zero findings is not retained — there is nothing to retry.

