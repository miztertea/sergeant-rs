# 40-capture: capture

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-hand-off/output/README.md | L4 | upstream artifact produced by `30-hand-off` |

## Purpose

A validated decision is folded into real code and rewritten to production standards; the throwaway is preserved on a throwaway branch.

Trigger (workflow-level): The user wants to sanity-check whether a state model or logic feels right, or explore what a UI should look like.

## What must become true here (durable outcome)

A validated decision is folded into real code and rewritten to production standards; the throwaway is preserved on a throwaway branch.

## Behavior contract

- **Once a prototype has answered its question — confirmed by the user, not inferred by the actor — the validated decision is folded into real code while the prototype itself is preserved as a primary source on a throwaway branch (not main), with the answer and question recorded on the implementation issue or a commit, and only the validated decision surviving on the main branch.**
  (trigger: the prototype has answered its question and the user has confirmed it; outcome: real code reflects the validated decision; the prototype and its verdict are durably recorded outside main)
  — `BU-P3-019`, `reference/sergeant-upstream/.agents/skills/prototype/SKILL.md` (item 6, line 26). **Corrected 2026-08-16, ICM-R3 (BU-PROTO-19):** the prior trigger, "once a prototype has answered its question," named no one who decides the question is answered — unlike the UI sub-shapes below, which already require the user to have picked a winning variant. The trigger now states explicitly that user confirmation is what closes this gate, matching the UI branch's own already-stronger language.
- **For the logic branch specifically, once the user has confirmed the logic prototype answered its question, the capture stage means the validated reducer/state-machine/function set is absorbed into the real module while the TUI shell is preserved only on the throwaway branch.**
  (trigger: the user has confirmed the logic-prototype question has been answered; outcome: the real codebase gains the validated logic module; the TUI shell is archived, not merged)
  — `BU-P3-027`, `reference/sergeant-upstream/.agents/skills/prototype/LOGIC.md` (step 7, line 71). **Corrected 2026-08-16, ICM-R3 (BU-PROTO-20):** same fix as `BU-P3-019` above, extended to the logic-branch-specific trigger.
- **The throwaway TUI shell must never reach production; only the underlying logic module is meant to survive.**
  (trigger: capturing or reviewing prototype output; outcome: production code never contains the prototype's TUI shell)
  — `BU-P3-028`, `reference/sergeant-upstream/.agents/skills/prototype/LOGIC.md` (Anti-patterns, line 79)
- **For sub-shape A, capture means folding the winning variant into the existing page and removing the other variants and the switcher from the main branch.**
  (trigger: the user has picked a winning variant (sub-shape A); outcome: main contains only the winning variant, folded into the existing page)
  — `BU-P3-036`, `reference/sergeant-upstream/.agents/skills/prototype/UI.md` (process step6, line 102)
- **For sub-shape B, capture means promoting the winning variant to a real, permanent route and removing the throwaway route and switcher from main.**
  (trigger: the user has picked a winning variant (sub-shape B); outcome: main contains a real route for the winning variant; the throwaway route and switcher are gone)
  — `BU-P3-037`, `reference/sergeant-upstream/.agents/skills/prototype/UI.md` (process step6, line 103)
- **Winning variant code must be rewritten to production standards when folded in, not merged as-is, since it was written under prototype constraints such as no tests and minimal error handling.**
  (trigger: folding the winning variant into real code (capture); outcome: the code that lands in production has been rewritten to meet normal production standards)
  — `BU-P3-039`, `reference/sergeant-upstream/.agents/skills/prototype/UI.md` (Anti-patterns, line 112)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Rewriting the winning code to production standards when folding it in (`BU-P3-039`).

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- **The user has not confirmed the prototype answered its question.** Do not fold a decision into real code, or capture a variant as winning, on the actor's own inference — `BU-P3-019`/`027` (logic) and `BU-P3-036`/`037` (UI) all require the user's own confirmation as the trigger.

### Completion boundary
This stage may complete only when the validated decision is folded into real code (rewritten to production standards), the throwaway is preserved on a throwaway branch out of main, and the question/answer are recorded on the implementation issue or a commit.

### Decision evidence
The capture commit and its recorded question/answer are this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
