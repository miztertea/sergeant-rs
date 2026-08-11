# 01-route-finding

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** a dispatched worker produces a review finding artifact

**Outcome:** actionable findings become owning-repo task tracker tasks with durably published blocking guidance

**Statement (the operative rule):** Dispatched workers pass each axis's strict JSON finding artifact to the review-findings router, which creates or updates one owning-repository task tracker task per actionable finding, preserves active task state on reruns, and publishes blocking task IDs and remediation guidance through `.sergeant-message`, `.sergeant-status`, and the notify step.

## What must become true here (durable outcome)

Actionable findings become owning-repo task tracker tasks with durably published blocking guidance — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0097`: Cosmetic and false-positive review dispositions create no cards; the schema rejects free-form review bodies, and credential-shaped values in accepted fields are redacted before durable storage.
- `BU-0101`: If the stored card has changed since the router last wrote it, the whole stored revision is kept below a `--- Superseded revision (preserved) ---` separator and the card is labelled `needs-reconciliation`; that label is cleared only once a human has merged the two accounts.
- `BU-0311`: A reviewer's structured JSON finding artifact carries only a fixed field set (`findings`, and per finding `id`/`severity`/`disposition`/`summary`/`evidence`/`paths`/`acceptance_criteria`/`recommendation`); review bodies, prompts, secrets, or credentials are never passed into the artifact, the task tracker, or fleet metadata.
- `BU-0726`: Before any finding text is stored or displayed, the review-findings router redacts secret-shaped content out of it: password/token/secret/credential/api-key key:value pairs, Bearer and Basic auth headers, GitHub tokens, AWS access key IDs, credentials embedded in URLs, PEM private-key markers, and generic high-entropy strings are each replaced with a redaction marker.
- `BU-0729`: An unrecognized finding severity spelling is rejected with an error naming the finding and listing every accepted severity spelling, and the rejected value is echoed back only when it is itself a short safe token — otherwise it is reported as '(unprintable)'.
- `BU-0737`: No finding field (summary, evidence, paths, acceptance criteria, recommendation) may contain an embedded newline or carriage return, on both the fresh-parse and the retry-replay validation paths.
- `BU-0738`: No finding field may begin with one of the two body lines the router itself gives structural meaning to ('Deduplication key: ' or 'Finding content digest: ') — the router gives no other line structural meaning, and this constraint is enforced on both the fresh-parse and retry-replay validation paths.
- `BU-0745`: A finding whose canonical severity has no known td-priority mapping causes the whole routing attempt to fail, rather than the finding being silently dropped or given a default priority.
- `BU-0747`: The dedup matcher for an existing task tracker card only ever identifies the card and classifies it as 'same' or 'diverged' by comparing the router's own recomputed content digest against the digest line stored in the card's description — it never reads the rest of the stored body back in order to merge, reconcile, or rewrite it.
- `BU-0748`: If more than one existing task tracker task matches a finding's deduplication key, or a match is reported with no task id or an unrecognized digest state, routing that finding fails rather than guessing which match to use.
- `BU-0749`: A finding whose stored card's content digest diverges from the freshly computed one is refused: the stored card is left completely untouched (no description, title, label, or status change), the refusal is reported, and routing continues with the rest of the batch rather than aborting.
- `BU-0750`: If every actionable finding in a batch was refused because its stored card no longer corresponds, the entire routing attempt is treated as a failure — an axis that ends up recording zero actual review evidence is not reported as a success.
- `BU-0751`: A same-digest matching task tracker card only has its priority and labels refreshed on a re-route — the description and title are never rewritten, so a human annotation or a hand-edited title on the card survives by construction.
- `BU-0753`: A same-digest matching task tracker card's defer_until is preserved across reruns; only an explicit human action (the task tracker defer --clear) may clear a deliberate deferral — an automated re-route does not clear it.
- `BU-0755`: A recurring finding's current occurrence is recorded as a new comment on the existing task tracker card rather than by mutating the stored description, so provenance of each occurrence is preserved without ever overwriting history.
- `BU-0760`: The review-findings router's final report distinguishes three outcomes to the caller: any refused findings are reported by name even on an otherwise successful run, a run with zero actionable findings is reported distinctly from one with nonblocking findings routed to the task tracker, and a run with blocking findings exits 2 with a published blocked state instead of reaching this final report at all.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0098`: `severity` is normalized to a canonical `error`/`warning`/`info` from the aliases reviewers actually emit — `blocker`/`critical`/`high` → `error` (P1); `major`/`medium` → `warning` (P2); `minor`/`low`/`informational` → `info` (P3) — and only the `error` family publishes a blocking gate.
- `BU-0099`: Deduplication is scoped to owning repo, axis, source, finding id, parent mission, and branch, so two sessions cannot collide on a generic finding id such as `spec-1`.
- `BU-0100`: An update never silently replaces a stored finding card: each revision the router writes ends with a `Revision block digest:` line over its own bytes, so a later route can tell whether the stored revision is still exactly what the router wrote.
- `BU-0102`: A closed card matching a finding is always reopened rather than abandoned, and the reopen is always reported.
- `BU-0302`: Only the canonical `error` severity family publishes a blocking review gate; `high` is deliberately treated as must-fix (mapped to `error`) so a genuinely blocking finding can never ship as mere debt.
- `BU-0711`: Replaying a retained review artifact via --retry is mutually exclusive with supplying fresh review context (--input/--axis/--source/etc.) in the same invocation.
- `BU-0720`: Before any repository or the task tracker action is taken, the review-findings router requires axis, source, branch, head SHA, and parent-task to be present, requires either --retry or --input to have been given, requires a real (non-'unknown') task id, and requires the axis and source to each match their accepted formats.
- `BU-0722`: The branch name, head SHA, parent task id, and fleet task id supplied to the review-findings router must each match a fixed safe-character format, or the routing attempt is refused.
- `BU-0723`: The review-findings router requires the named project's config to exist and the yq, git, and td (marcus/td) tools to be available, and requires either a retry or an existing input file, before doing any repository lookup.
- `BU-0724`: The owning repository's filesystem path is resolved by looking it up by name in the project's configuration, and the resolved path must exist and be an actual git repository, or routing fails.
- `BU-0725`: A review input file is rejected unless its top-level JSON is an object with exactly one key, 'findings', whose value is a list.
- `BU-0727`: Each finding object in a review input must have a field set exactly equal to the allowed finding schema (id, severity, disposition, summary, evidence, paths, acceptance_criteria, recommendation) — neither missing a required field nor carrying an unrecognized one.
- `BU-0728`: A finding's id must match a fixed safe-token character pattern after redaction/cleaning, or the input is rejected.
- `BU-0730`: A finding's reported severity spelling is normalized to one canonical severity; the original reported spelling is retained separately for provenance only when it differs from the canonical form.
- `BU-0731`: A finding's disposition must be one of actionable, cosmetic, or false-positive, or the input is rejected.
- `BU-0732`: A finding's paths field must be a list of strings, or the input is rejected.
- `BU-0733`: A finding's summary, evidence, acceptance_criteria, and recommendation fields must each be non-empty after cleaning, or the finding is rejected — an empty field would leave the task tracker card body ending in trailing whitespace, making the content digest depend on invisible characters.
- `BU-0734`: A finding's content digest is computed only over its sanitized, externally observable fields (severity, summary, evidence, paths, acceptance criteria, recommendation) — the branch, head SHA, parent mission, and fleet task are deliberately excluded, so a rerun of the same finding under different run metadata does not create what looks like a new revision.
- `BU-0744`: A finding whose disposition is not 'actionable' (cosmetic or false-positive) is skipped entirely — no task tracker lookup, no task tracker card, no gate effect — and reported as ignored.
- `BU-0746`: A finding's deduplication marker is scoped to the repo, review axis, review source, finding id, parent mission, and branch together — not to axis/source/finding-id alone — because reviewers commonly emit generic finding ids that would otherwise collide across unrelated review sessions and let one session's update overwrite another's evidence.
- `BU-0752`: A same-digest matching task tracker card that had been closed is reopened when the same finding is reported again, with a notice printed that the finding is still being reported.
- `BU-0754`: A same-digest matching task tracker card's labels are merged with the standard labels (independent-review, finding, axis) rather than replaced, deduplicating, so manually added labels are never stripped by a rerun.
- `BU-0756`: If no existing task tracker task matches a finding's deduplication key, the review-findings router creates a new task tracker task carrying the finding's computed priority, standard labels, and full composed body.

