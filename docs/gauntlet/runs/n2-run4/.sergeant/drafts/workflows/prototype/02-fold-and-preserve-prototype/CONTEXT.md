# 02-fold-and-preserve-prototype

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-select-branch/output/outcome.md | L4 | upstream evidence produced by `select-branch` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a prototype has answered its question

**Outcome:** the validated decision reaches main, the prototype itself is durably preserved off main, and both are cross-referenced

**Statement (the operative rule):** Once a prototype is done, any validated decision is folded into the real code, and the prototype itself is committed to a throwaway branch out of main as a primary source with a context pointer left on the implementation issue and the verdict/question captured in the issue or a commit — main keeps only the validated decision, not the prototype.

## What must become true here (durable outcome)

The validated decision reaches main, the prototype itself is durably preserved off main, and both are cross-referenced — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1097`: Once a logic prototype has answered its question, the answer is captured and the prototype itself is captured per the SKILL's rule, with the logic-specific mapping: the validated reducer/machine/function set lifts into the real module while the TUI shell rides along to the throwaway branch that preserves the prototype as a primary source.
- `BU-1102`: The TUI shell built for a logic prototype is never shipped into production — only the logic module behind it is worth keeping, since the shell is optimized for being driven by hand from a terminal.
- `BU-1119`: When sub-shape A's winning variant is chosen, it is folded into the existing page and the losing variants and the switcher are dropped from main.
- `BU-1120`: When sub-shape B's winning variant is chosen, it is promoted to a real route and the throwaway route and switcher are dropped from main.
- `BU-1121`: The full set of UI-prototype variants is preserved as a primary source on the throwaway branch rather than deleted, because leftover variant components and the switcher left in the main branch rot quickly and confuse the next reader.
- `BU-1125`: A UI prototype's variant code, written under prototype constraints (no tests, minimal error handling), is never promoted directly to production — it is rewritten properly when folded into the real codebase.

