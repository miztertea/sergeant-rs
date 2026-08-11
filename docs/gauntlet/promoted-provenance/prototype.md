# Provenance — Prototype

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W21** `prototype`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-010` | prototype is a workflow that builds throwaway code to answer a specific design question, either about logic/state or about UI appearance. | `reference/sergeant-upstream/.agents/skills/prototype/SKILL.md` (frontmatter: description) |
| `BU-P3-011` | The defining premise of the workflow: a prototype exists only to answer one question, and the nature of that question determines which branch and shape the prototype takes. | `reference/sergeant-upstream/.agents/skills/prototype/SKILL.md` (body line 8) |

## Stages

### `00-select-branch`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-012` | The first checkpoint of the prototype workflow is determining which of the two question-types (logic/state vs. UI) is being asked, using the prompt, surrounding code, or the user directly. | `reference/sergeant-upstream/.agents/skills/prototype/SKILL.md` (line 12) |
| `BU-P3-013` | When the question is about appearance, the workflow routes to the UI-prototype branch, which produces several structurally different UI variants switchable in-browser. | `reference/sergeant-upstream/.agents/skills/prototype/SKILL.md` (line 15) |
| `BU-P3-014` | When the branch choice is ambiguous and the user cannot be reached, the workflow falls back to a heuristic based on the surrounding code's shape and records the assumption explicitly in the prototype, rather than blocking. | `reference/sergeant-upstream/.agents/skills/prototype/SKILL.md` (line 17) |

### `10-record-question`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-021` | Before any code is written, the actor records the state model and the exact question the prototype answers, so the question can be checked against the eventual result even if the user returns to it later. | `reference/sergeant-upstream/.agents/skills/prototype/LOGIC.md` (step 1, line 18) |

### `20L-build-logic`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-020` | The logic-prototype branch builds a small interactive terminal app so the user can hand-drive a state model through the cases that are hard to evaluate on paper. | `reference/sergeant-upstream/.agents/skills/prototype/LOGIC.md` (header, line 3) |
| `BU-P3-022` | The logic under test must be isolated behind a small, pure interface that can later be lifted into the real codebase; only the terminal UI shell around it is truly throwaway. | `reference/sergeant-upstream/.agents/skills/prototype/LOGIC.md` (step 3 intro, line 28) |
| `BU-P3-023` | The logic module must stay pure — no I/O, no terminal code, no console output used for control flow — and the dependency direction is one-way: the TUI imports the logic module, never the reverse. | `reference/sergeant-upstream/.agents/skills/prototype/LOGIC.md` (step 3, line 37) |
| `BU-P3-024` | The terminal UI re-renders the full frame from scratch on every update rather than appending output, so the user always sees one stable current view. | `reference/sergeant-upstream/.agents/skills/prototype/LOGIC.md` (step 4, line 43) |
| `BU-P3-025` | After each user action, the shell replaces the displayed frame entirely rather than appending to it. | `reference/sergeant-upstream/.agents/skills/prototype/LOGIC.md` (step 4, line 54) |
| `BU-P3-026` | The logic prototype is wired into the host project's existing task runner so it can be started by name rather than by remembering a file path. | `reference/sergeant-upstream/.agents/skills/prototype/LOGIC.md` (step 5, line 61) |

### `20U-build-variants`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-029` | The UI-prototype branch produces several structurally distinct UI variants on one route, switchable live in the browser, from which the user picks or recombines before the rest is discarded. | `reference/sergeant-upstream/.agents/skills/prototype/UI.md` (header, line 3) |
| `BU-P3-030` | The UI branch prefers mounting variants inside an existing page (sub-shape A) over a standalone throwaway route (sub-shape B), because judging a variant against the app's real surrounding context is more informative than judging it in isolation. | `reference/sergeant-upstream/.agents/skills/prototype/UI.md` (sub-shape rationale, line 16) |
| `BU-P3-031` | When sub-shape B is used, the new throwaway route must follow the project's existing routing convention and be named so it is obviously a prototype. | `reference/sergeant-upstream/.agents/skills/prototype/UI.md` (sub-shape B naming, line 28) |
| `BU-P3-032` | The UI branch defaults to producing three variants and caps at five, beyond which additional variants are considered noise rather than useful signal. | `reference/sergeant-upstream/.agents/skills/prototype/UI.md` (process step1, line 38) |
| `BU-P3-033` | Each UI variant must differ structurally (layout, information hierarchy, primary affordance), not merely cosmetically; if two drafts converge, one must be redone with an explicit constraint forcing structural divergence. | `reference/sergeant-upstream/.agents/skills/prototype/UI.md` (process step2, line 54) |
| `BU-P3-034` | The variant switcher UI must be gated off in production builds so that an accidental merge of prototype code cannot expose it to real users. | `reference/sergeant-upstream/.agents/skills/prototype/UI.md` (process step4, line 90) |
| `BU-P3-038` | UI variants must not perform real mutations; any mutation a variant needs should hit a stub, keeping the prototype scoped to appearance rather than backend correctness. | `reference/sergeant-upstream/.agents/skills/prototype/UI.md` (Anti-patterns, line 111) |

### `30-hand-off`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-035` | After building the variants, the actor hands the user the URL and variant keys; the most useful feedback typically recombines pieces across variants rather than picking one outright. | `reference/sergeant-upstream/.agents/skills/prototype/UI.md` (process step5, line 96) |

### `40-capture`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-019` | Once a prototype has answered its question, the validated decision is folded into real code while the prototype itself is preserved as a primary source on a throwaway branch (not main), with the answer and question recorded on the implementation issue or a commit, and only the validated decision surviving on the main branch. | `reference/sergeant-upstream/.agents/skills/prototype/SKILL.md` (item 6, line 26) |
| `BU-P3-027` | For the logic branch specifically, the capture stage means the validated reducer/state-machine/function set is absorbed into the real module while the TUI shell is preserved only on the throwaway branch. | `reference/sergeant-upstream/.agents/skills/prototype/LOGIC.md` (step 7, line 71) |
| `BU-P3-028` | The throwaway TUI shell must never reach production; only the underlying logic module is meant to survive. | `reference/sergeant-upstream/.agents/skills/prototype/LOGIC.md` (Anti-patterns, line 79) |
| `BU-P3-036` | For sub-shape A, capture means folding the winning variant into the existing page and removing the other variants and the switcher from the main branch. | `reference/sergeant-upstream/.agents/skills/prototype/UI.md` (process step6, line 102) |
| `BU-P3-037` | For sub-shape B, capture means promoting the winning variant to a real, permanent route and removing the throwaway route and switcher from main. | `reference/sergeant-upstream/.agents/skills/prototype/UI.md` (process step6, line 103) |
| `BU-P3-039` | Winning variant code must be rewritten to production standards when folded in, not merged as-is, since it was written under prototype constraints such as no tests and minimal error handling. | `reference/sergeant-upstream/.agents/skills/prototype/UI.md` (Anti-patterns, line 112) |

## Notes

**Synthesis notes:** The A/U branch at `20L`/`20U` is the corpus's cleanest evidence for *conditional* procedure. It is representable today as one selection stage (`00-select-branch`) plus mutually-exclusive downstream stages — recorded as grammar pressure for a future conditional-stage schema extension, not an engine gap (the current linear `workflow.toml` requires both stage directories to exist; the non-selected one is a documented no-op for that run).

## Promotion note (docs/icm/promotion-spec-2026-08-11.md §1)

`40-capture`, this package's true closing stage, declares a `promote` output disposition with no finalize step — one of the 30 of 34 N1 packages in that shape, not one of the 3 (`drain-fleet`, `respond-to-worker`, `to-spec`) that name one. Recorded here per the spec's finalize-gap rule rather than silently promoted; disposition on whether this package needs a finalize step is left to human review at merge time, not applied mechanically by this curation act.

