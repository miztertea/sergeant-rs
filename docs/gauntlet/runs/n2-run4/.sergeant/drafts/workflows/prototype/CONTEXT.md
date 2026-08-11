# prototype — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** the user wants a throwaway prototype to answer a design question
- **Outcome:** the user can independently explore variants, and cross-variant preferences are captured as signal rather than treated as noise
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `drive-ui-prototype`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-select-branch` — the question is classified as logic or UI and routed to the matching branch's process
2. `02-fold-and-preserve-prototype` — the validated decision reaches main, the prototype itself is durably preserved off main, and both are cross-referenced
3. `03-build-logic-prototype` — the question being answered is written down and checkable, rather than implicit in the author's head
4. `04-drive-logic-prototype` — user surprises are treated as the prototype's real output, and the prototype is extended on request rather than treated as frozen
5. `05-select-ui-subshape` — sub-shape A is chosen unless there's a specific reason it can't host the variants
6. `06-build-ui-prototype` — the variant count stays in the 3-5 range rather than growing unbounded
7. `07-drive-ui-prototype` — the user can independently explore variants, and cross-variant preferences are captured as signal rather than treated as noise

