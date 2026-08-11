# Standing Constraints — Direct Implementation

Layer 3 (`_config/`), stable across every run of this workflow — binds every stage, not one checkpoint, per `docs/icm/convention.md` §1a. Sourced from the behavior units below.

Direct-mode work always uses a feature branch and always opens a PR — never a direct push to the default branch. Direct mode requires an explicit user request and a single owning repository; it is not a way to avoid dispatch when the outcome genuinely spans repositories.

## Source units

- **Never use direct mode to edit several repositories in one checkout, or to bypass repository instructions, task ownership, review independence, or shipping gates.**
  (trigger: direct mode selected; outcome: direct mode cannot be used to circumvent multi-repo or gate discipline)
  — `BU-P1-016`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L39-41)
- **Use direct mode only when the user explicitly requests it and the work has one clear owning repository.**
  (trigger: explicit user request plus single-repository ownership; outcome: direct mode is selected as the executing procedure)
  — `BU-P1-007`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L22-23, direct-mode trigger)
- **Direct mode is used when the user explicitly requests implementation in the current session and one repository owns the outcome; it still requires a task, TDD, repository-native checks, independent review, shipping validation, and handoff.**
  (trigger: the trigger condition for direct mode holds; outcome: direct mode delivers through the same gates as dispatch mode, never a lighter path)
  — `BU-P1-107`, `reference/sergeant-upstream/docs/what-is-sergeant.md` (docs/what-is-sergeant.md L62-66, Direct mode definition)
- **Direct-mode implementation is an eight-step ordered procedure: load context, reconcile existing worktrees/workers, create-or-reuse a feature branch, implement TDD-first, run native validation and independent review, run the final shipping gate only at the approved boundary, open a PR and satisfy CI/review/merge authorization, then record handoff/PR/merge/deployment/cleanup state.**
  (trigger: direct mode is chosen for a piece of work; outcome: the eight named checkpoints are each crossed in order before the work is considered complete)
  — `BU-P8-055`, `reference/sergeant-upstream/docs/using-sergeant.md` (L21-28 (Direct mode))
