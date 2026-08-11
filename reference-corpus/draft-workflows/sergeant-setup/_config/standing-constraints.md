# Standing Constraints — Sergeant Setup

Layer 3 (`_config/`), stable across every run of this workflow — binds every stage, not one checkpoint, per `docs/icm/convention.md` §1a. Sourced from the behavior units below.

Write only to Sergeant-owned paths. Never write to other tools' config surfaces. Never auto-initialize external tools without explicit consent. These apply across every stage of this workflow, not to one checkpoint — they belong in this workflow's `_config/` (Layer 3), not in any single stage's Inputs table.

## Source units

- **sergeant-setup is permitted to write only to Sergeant-owned paths: the global config at ~/.config/sergeant/config.yaml and per-project YAML files at ~/.config/sergeant/<project>.yaml.**
  (trigger: any step of sergeant-setup is about to write a file; outcome: writes stay confined to Sergeant's own config namespace)
  — `BU-P5-004`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 25-28)
- **sergeant-setup must never write to other tools' configuration surfaces (opencode config, ~/.claude or any repository's CLAUDE.md/.claude, Codex config, Goose config, any repository's AGENTS.md/.github) or to any path outside ~/.config/sergeant/ that the user did not explicitly name.**
  (trigger: any step of sergeant-setup is about to write a file; outcome: other tools' and repositories' configuration remains untouched)
  — `BU-P5-005`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 30-37)
- **The sergeant-setup skill must never instruct writes to any of the major global agent-config paths (~/.config/opencode/, ~/.claude/, ~/.codex/, ~/.goose/) or to AGENTS.md, and must explicitly say 'Never write to' those paths.**
  (trigger: the setup/repair skill runs against a developer's machine; outcome: Sergeant setup never silently reconfigures another tool's global configuration, matching the proposal's explicit call-out (§8.2) of this exact rule)
  — `BU-P7-036`, `reference/sergeant-upstream/tests/sergeant-setup-test.sh` (lines 54-67)
- **td, Graphify, and Treehouse are never auto-initialized by sergeant-setup; each requires an explicit confirmation prompt before any command runs, and if consent is declined the state is left unchanged and the skip is reported.**
  (trigger: a phase would initialize an optional external tool; outcome: no optional external tool is initialized without explicit per-tool consent)
  — `BU-P5-006`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 39-41)
