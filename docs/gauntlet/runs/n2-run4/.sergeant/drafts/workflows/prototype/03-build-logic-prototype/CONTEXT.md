# 03-build-logic-prototype

## Inputs

| File | Layer | Why |
|---|---|---|
| ../02-fold-and-preserve-prototype/output/outcome.md | L4 | upstream evidence produced by `fold-and-preserve-prototype` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a logic prototype is about to be built

**Outcome:** the question being answered is written down and checkable, rather than implicit in the author's head

**Statement (the operative rule):** Before writing any code for a logic prototype, the state model and the question being prototyped are written down in one paragraph (in the README or a top-of-file comment), so the question is explicit and checkable later whether the user is watching now or returns to it later.

## What must become true here (durable outcome)

The question being answered is written down and checkable, rather than implicit in the author's head — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1087`: A logic prototype uses whatever language/runtime the host project already uses; if the project has no obvious runtime, the user is asked rather than one being picked unilaterally.
- `BU-1088`: A logic prototype matches the project's existing tooling conventions rather than adding a new package manager or runtime just for the prototype.
- `BU-1089`: The actual logic being tested is isolated behind a small, pure interface that could be lifted out into the real codebase later — the throwaway TUI wraps it, but the logic module itself is not meant to be thrown away.
- `BU-1090`: The shape of the isolated logic module is chosen to fit the question: a pure reducer for discrete actions over a single state value, a state machine when legal-action-ness is itself part of the question, a small set of pure functions when there's no implicit current state, or a class/module with a clear method surface when the logic genuinely owns ongoing internal state.
- `BU-1091`: The isolated logic module stays pure — no I/O, no terminal code, no console.log for control flow — with the TUI only ever calling into it, never the reverse.
- `BU-1092`: The logic prototype's TUI is built as a lightweight terminal app that clears the screen and re-renders the whole frame on every tick, so the user always sees one stable view rather than an ever-growing scrollback.
- `BU-1093`: Each rendered TUI frame has two parts in a fixed order: the current state (pretty-printed, diff-friendly, styled with bold/dim for emphasis) first, then the available keyboard shortcuts listed at the bottom.
- `BU-1094`: The logic prototype's TUI runs a fixed loop: initialize a single in-memory state object and render the first frame on start, read one keystroke or line at a time and dispatch it to a handler that mutates state, re-render the full frame after every action by replacing rather than appending, and loop until the user quits.
- `BU-1095`: The logic prototype is made runnable in one command by adding a script to the project's existing task runner; if the host project has no task runner, the command is instead placed at the top of the prototype's README.
- `BU-1098`: A logic prototype does not get tests added to it — a prototype that needs tests is no longer a prototype.
- `BU-1099`: A logic prototype does not wire to the real database — an in-memory store is used instead, unless the question being prototyped is specifically about persistence.
- `BU-1100`: A logic prototype does not generalize beyond the one question it exists to answer — speculative "what if we wanted to support X later" scope is excluded.
- `BU-1101`: The logic and the TUI are kept from blurring together: if the reducer or state machine references console.log, prompts, or terminal escape codes, it is no longer portable, so the TUI is kept as a thin shell over a pure module.

