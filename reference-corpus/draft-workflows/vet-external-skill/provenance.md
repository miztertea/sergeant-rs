# Provenance — Vet External Skill

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W34** `vet-external-skill`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-119` | Before adopting an external skill, it must be vetted through a fixed sequence: read its complete SKILL.md and referenced scripts, confirm its source and update mechanism, check its filesystem/shell/network/git/credential actions, verify it does not conflict with repository AGENTS.md or safety policy, pin or lock its source where supported, and test it in a disposable repository or worktree before broad installation. | `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L124-131, vet-external-skill workflow trigger) |

## Stages

### `00-read-source`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-120` | Read the external skill's complete SKILL.md and referenced scripts before adopting it. | `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L126, vet step 1) |

### `10-confirm-provenance`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-121` | Confirm the external skill's source and update mechanism. | `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L127, vet step 2) |

### `20-check-actions`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-122` | Check the external skill's filesystem, shell, network, Git, and credential actions. | `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L128, vet step 3) |

### `30-verify-no-conflict`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-123` | Verify the external skill does not conflict with repository AGENTS.md or safety policy. | `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L129, vet step 4) |

### `40-pin-source`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-124` | Pin or lock the external skill's source where the installer supports it. | `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L130, vet step 5) |

### `50-test-in-disposable-copy`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-125` | Test the external skill in a disposable repository or worktree before broad installation. | `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L131, vet step 6) |

### `60-update-managed`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-126` | For skills.sh-managed skills, rerun the official installer and inspect the diff and updated lock file before accepting changes. | `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L138-139, skills.sh update path) |

### `60-update-owned`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-127` | For Sergeant-owned skills, update this repository through a reviewed PR and run tests/instruction-policy-test.sh plus the full Sergeant test suite. | `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L142-144, Sergeant-owned update path) |

## Notes

**Synthesis notes:** Six ordered checkpoints (`00`-`50`) plus two mutually exclusive update variants (`60-update-managed`/`60-update-owned`) reached only when refreshing an already-adopted skill. Each step's outcome ("the source was read", "the actions were checked", "it was tested in a disposable copy") survives any reimplementation of *how* the checking is done — a strong candidate for the smallest complete reference workflow in the corpus.

