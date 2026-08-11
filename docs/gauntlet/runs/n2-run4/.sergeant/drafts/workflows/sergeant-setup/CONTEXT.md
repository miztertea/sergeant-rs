# sergeant-setup — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** a checklist step is reached
- **Outcome:** a successful Graphify run is verified by the presence of both named output files
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `phase9-graphify-init`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-run-checklist` — already-satisfied steps are skipped silently and every step's outcome is recorded visibly
2. `02-phase1-detect-prerequisites` — `td` is accepted only if it is the specific Marcus implementation with the named flag support, and at least one of the three named interactive agents must be present
3. `03-phase2-install-checkout` — the destination is confirmed with the user before any clone command runs
4. `04-phase3-global-config` — the config file is written only after an explicit confirmed preview
5. `05-phase4-new-project-interview` — each question is answered in sequence before the next is asked
6. `06-phase5-repair-project-yaml` — the run stops on the parse error rather than attempting further changes
7. `07-phase6-verify-installation` — verification proceeds strictly in order and halts at the first failure
8. `08-phase7-task-tracker-init` — each repository's task tracker state is either confirmed `[ok]` or consent-gated before initialization
9. `09-phase8-treehouse-init` — Treehouse is initialized only with consent, and its absence or decline never blocks overall setup completion
10. `10-phase9-graphify-init` — a successful Graphify run is verified by the presence of both named output files

