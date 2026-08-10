# Provenance — Task Intake and Route

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W5** `task-intake-and-route`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-025` | Any task the user brings triggers a nine-step standard workflow: load context, check the task queue, choose an execution mode, reconcile existing state, confirm only unresolved decisions, execute, monitor real progress, handle decision gates, and reconcile and deliver. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L132-146, Standard workflow heading and steps) |

## Stages

### `01-load-context`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-026` | Load context: run sgt-context and identify the owning repository or repositories, inherited instructions, configured paths, and cross-repository dependencies before selecting an execution mode. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L136, step 1) |

### `02-check-queue`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-027` | Check the queue: run sgt-td-list and reuse a matching task in direct or dispatch mode; create a task only when no canonical task exists. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L137, step 2) |

### `03-choose-mode`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-028` | Choose execution mode: direct for explicit single-repository work in this session; dispatch for cross-repository, parallel, or explicitly delegated work. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L138, step 3) |
| `BU-P1-003` | Use dispatch mode when work spans repositories, contains two or more independent repository-owned tasks, needs an isolated independent review worker, or the user asks for workers. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L15-17, dispatch-mode trigger) |
| `BU-P1-108` | Dispatch mode is used for cross-repository work, independent parallel repository tasks, isolated review workers, or an explicit request for workers; Sergeant creates isolated worktrees, injects repository instructions, and records fleet state. | `reference/sergeant-upstream/docs/what-is-sergeant.md` (docs/what-is-sergeant.md L68-72, Dispatch mode definition) |
| `BU-P8-053` | Direct mode is chosen when the user explicitly requests work in the current session and one repository owns the complete outcome. | `reference/sergeant-upstream/docs/using-sergeant.md` (L18-19 (Direct mode)) |
| `BU-P8-054` | Dispatch mode is chosen for cross-repository work, independent repository-owned tasks, isolated review workers, or an explicit request for workers. | `reference/sergeant-upstream/docs/using-sergeant.md` (L30-33 (Dispatch mode)) |

### `04-reconcile-state`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-029` | Reconcile existing state: run sgt-watch --sync-all, then inspect active workers, branches, worktrees, retained gates, and handoffs before starting; resume or take over preserved work rather than creating duplicates. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L139, step 4) |

### `05-confirm-decisions`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-030` | Confirm only unresolved decisions that change scope or risk: ask only when repository ownership, user-visible behavior, security/privacy policy, data retention, destructive action, or an irreversible tradeoff is unknown; do not ask the user to reconfirm an execution mode, plan, or tradeoff already recorded in the conversation or td. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L140, step 5) |

### `06-execute`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-031` | Execute: in direct mode, start the td task and implement through tests, review, and delivery; in dispatch mode, run sgt-dispatch with a repository list or a td id. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L141-143, step 6) |

### `07-monitor`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-032` | Monitor real progress: require recent meaningful events or an active child operation plus exact pane/process identity — parent-process liveness alone is insufficient; in OpenCode use a managed background watch and verify it started, falling back to bounded one-shot status checks rather than a blocking watch call when unavailable. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L144, step 7) |

### `08-handle-decisions`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-033` | Handle decisions: for needs_input, blocked, or ask-user gates, read the exact finding, obtain only genuinely missing user decisions, record them in td, and continue approved remediation without asking again merely to dispatch. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L145, step 8) |
| `BU-P1-038` | Use sgt-respond, sgt-wake, or supported recovery only after reconciling status, response generation, pane identity, and handoff evidence. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L148, resume preconditions) |

### `09-reconcile-deliver`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-034` | Reconcile and deliver: surface PRs and merge order, complete approved merges/deployments, and run cleanup only after terminal state and preserved evidence are verified. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L146, step 9) |

