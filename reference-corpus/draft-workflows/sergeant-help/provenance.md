# Provenance — Sergeant Help

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W4** `sergeant-help`.

## Stages

### `00-classify-and-locate`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-117` | Answering a question first classifies it against the documentation map, then reads the primary document before searching broadly. | `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (lines 30-31) |
| `BU-P5-126` | If the primary document for a question is missing, sergeant-help reports its expected path and stops before guessing. | `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (line 71) |

### `10-resolve-source-conflicts`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-122` | When sources disagree, precedence is: command behavior/tests/supported --help output for released syntax, then AGENTS.md for always-on execution/safety policy, then the trigger-loaded skill for its own procedure, then docs/schema.md for project fields, then user documentation for walkthroughs. | `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (lines 45-50) |
| `BU-P5-127` | If observed command behavior differs from documentation, sergeant-help reports the mismatch, trusts tested/released behavior or supported --help output over the stale doc, and creates or suggests a documentation task. | `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (line 72) |
| `BU-P5-120` | For flag or argument questions, sergeant-help runs --help only when the command supports it; otherwise it inspects the command's actual emitted usage/error contract and its tests. | `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (lines 41-42) |
| `BU-P5-119` | For architectural questions where a configured Sergeant graph exists, sergeant-help runs graphify query and uses cited source locations in the answer. | `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (lines 39-40) |

### `20-answer-or-hand-off`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-121` | Answers state the exact command, required preconditions, expected evidence, and links to repository-relative documentation paths. | `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (lines 43-44) |
| `BU-P5-125` | Destructive operations are kept out of examples unless the documentation itself requires confirmation for them and the user explicitly requested them. | `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (lines 64-65) |
| `BU-P5-128` | If a question actually requires project ownership context, sergeant-help loads load-project and runs sgt-context rather than answering from memory. | `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (line 73) |
| `BU-P5-129` | If a question actually requires implementation or fleet mutation, sergeant-help hands off to the owning procedural skill; help itself remains strictly read-only. | `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (line 74) |
| `BU-P5-113` | sergeant-help answers Sergeant installation, setup, usage, skills, and troubleshooting questions strictly from repository-owned documentation. | `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (lines 3-4) |
| `BU-P5-114` | sergeant-help is loaded when the user asks what Sergeant is, how to install/configure/use it, where skills come from, how to run a command/workflow, or how to diagnose a Sergeant error. | `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (lines 7-8) |
| `BU-P5-115` | sergeant-help is never used as a substitute for load-project, cross-repo-work, dispatch, or wiki once the user has actually requested execution of those procedures. | `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (lines 12-13) |

## Adjudication A4 (N1-BH-02 sweep)

No changes. All three stages (`00-classify-and-locate`, `10-resolve-source-conflicts`, `20-answer-or-hand-off`) were classified "Judgment required" (§6.4) at extraction; none carries a "Deterministic-machinery candidate" (§6.5) heading anywhere in this package, so no stage is in scope for A4's default-demote or case-by-case reimplementation test. Stage count unchanged at 3.

