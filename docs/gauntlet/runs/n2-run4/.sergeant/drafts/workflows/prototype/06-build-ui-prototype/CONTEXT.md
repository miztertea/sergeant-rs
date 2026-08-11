# 06-build-ui-prototype

## Inputs

| File | Layer | Why |
|---|---|---|
| ../05-select-ui-subshape/output/outcome.md | L4 | upstream evidence produced by `select-ui-subshape` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** the number of variants for a UI prototype is being decided

**Outcome:** the variant count stays in the 3-5 range rather than growing unbounded

**Statement (the operative rule):** A UI prototype defaults to 3 variants and caps at 5, because beyond 5 variants stop being radically different and start being noise.

## What must become true here (durable outcome)

The variant count stays in the 3-5 range rather than growing unbounded — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1109`: Before generating variants, the plan (how many variants, switching mechanism, host route) is written down in one line, in the prototype's location or a top-of-file comment.
- `BU-1110`: UI prototype variants must be structurally different (layout, information hierarchy, primary affordance), not just different in color; if two drafted variants come out too similar, one is redone with explicit guidance against the pattern that made them converge.
- `BU-1111`: In sub-shape A, the switcher keeps all the route's existing data fetching above it unchanged — only the rendered subtree changes per variant.
- `BU-1112`: In sub-shape B, the throwaway route mounts the same switcher component used in sub-shape A.
- `BU-1113`: The floating switcher bar has three fixed pieces: a left arrow that cycles to the previous variant (wrapping around), a label showing the current variant's key and name, and a right arrow that cycles forward (wrapping around).
- `BU-1114`: Clicking a switcher arrow updates the URL's variant search param via the framework's router, so the currently-shown variant is shareable and stable across reloads.
- `BU-1115`: The left/right arrow keys also cycle the switcher's variant, except when an input, textarea, or contenteditable element is currently focused, in which case the arrow keys are not intercepted.
- `BU-1116`: The floating variant switcher is hidden in production builds, gated on an environment check like NODE_ENV !== 'production', so a stray prototype merge cannot ship the switcher bar to real users.
- `BU-1117`: The floating switcher is built as a single shared component reusable by both sub-shapes A and B, located wherever shared UI already lives in the project.
- `BU-1122`: UI prototype variants that differ only in color or copy are treated as a tweak rather than a genuine prototype, since real variants must disagree about structure.
- `BU-1123`: UI prototype variants avoid sharing too much code with each other — a shared header component is fine, but a shared layout defeats the point, since each variant should be free to discard the layout.
- `BU-1124`: UI prototype variants are read-only by default; if a variant needs to mutate data, it is pointed at a stub rather than wired to real mutations, since the prototype's question is what the UI should look like, not whether the backend works.

