# 20U-build-variants: build variants

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-record-question/output/README.md | L4 | upstream artifact produced by `10-record-question` |
| ../references/shared-rules.md | L3 | rules that apply to both branches (throwaway marking/location, one command to run, no persistence by default, surface the state) — added ICM-R3, closing a gap where these were extracted at N1 but never materialized |

## Purpose

UI variants are built to answer the recorded question.

Trigger (workflow-level): The user wants to sanity-check whether a state model or logic feels right, or explore what a UI should look like.

## What must become true here (durable outcome)

UI variants are built to answer the recorded question.

## Behavior contract

- **The UI-prototype branch produces several structurally distinct UI variants on one route, switchable live in the browser, from which the user picks or recombines before the rest is discarded.**
  (trigger: the branch question is about what something should look like; outcome: several in-browser-switchable UI variants exist for the user to compare)
- **The UI branch prefers mounting variants inside an existing page (sub-shape A) over a standalone throwaway route (sub-shape B), because judging a variant against the app's real surrounding context is more informative than judging it in isolation.**
  (trigger: the UI-prototype branch has been selected; outcome: sub-shape A is chosen unless no existing page can host the variants)
- **When sub-shape B is used, the new throwaway route must follow the project's existing routing convention and be named so it is obviously a prototype.**
  (trigger: sub-shape B has been chosen; outcome: the throwaway route is discoverable as a prototype and does not introduce a new routing convention)
- **The UI branch defaults to producing three variants and caps at five, beyond which additional variants are considered noise rather than useful signal.**
  (trigger: generating UI variants; outcome: the number of variants produced falls between the default of three and a hard cap of five)
- **Each UI variant must differ structurally (layout, information hierarchy, primary affordance), not merely cosmetically; if two drafts converge, one must be redone with an explicit constraint forcing structural divergence.**
  (trigger: drafting UI variants; outcome: every variant produced is structurally distinct from the others)
- **The variant switcher UI must be gated off in production builds so that an accidental merge of prototype code cannot expose it to real users.**
  (trigger: building the variant switcher; outcome: the switcher never renders in a production build)
- **UI variants must not perform real mutations; any mutation a variant needs should hit a stub, keeping the prototype scoped to appearance rather than backend correctness.**
  (trigger: building or wiring a UI variant; outcome: no UI variant performs a real write against production systems)

## Bounded judgment

Apply `@@bounded-judgment`. See `../references/shared-rules.md` for the rules this stage shares with `20L-build-logic`.

### J2 — delegated to this stage
- Choosing sub-shape A (mounted in an existing page) vs. sub-shape B (standalone route) — A is preferred unless no existing page can host the variants.
- How many variants to produce, between the default of three and the cap of five, and how to force structural divergence between them.

### J1 — local choices allowed
- Naming of a sub-shape B throwaway route, within the project's existing routing convention.

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers.

### Completion boundary
This stage may complete only when the variant switcher is gated off in production builds, no variant performs a real mutation, and every variant is structurally (not merely cosmetically) distinct — plus the shared rules in `../references/shared-rules.md`.

### Decision evidence
The built variants and the sub-shape choice are this stage's own durable output.

## Additional note

Conditional: entered only when `00-select-branch` selected the UI branch. Mutually exclusive with `20L-build-logic`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
