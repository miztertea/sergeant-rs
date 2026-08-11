# 01-research-and-answer

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** sergeant-help is answering a question

**Outcome:** the answer follows this fixed research/answer sequence rather than free-form search

**Statement (the operative rule):** The query procedure classifies the question against the documentation map, reads the primary document first, escalates to a repository-wide grep search only for unresolved terms, consults the graph-generation tool for architectural questions when a graph exists, and answers with the exact command, required preconditions, expected evidence, and links to repository-relative documentation.

## What must become true here (durable outcome)

The answer follows this fixed research/answer sequence rather than free-form search — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0125`: When sources disagree, precedence is: command behavior/tests/supported `--help` output for released syntax; `AGENTS.md` for always-on execution/safety policy; the trigger-loaded skill for its procedure; `docs/schema.md` for project fields; user documentation for walkthroughs.
- `BU-0126`: The skill states when a behavior is undocumented or contradictory rather than inventing a command, flag, state transition, or safety guarantee.
- `BU-0127`: Destructive operations are kept out of examples unless the documentation requires confirmation and the user explicitly requested them.

