# Upstream Sergeant core function map — 2026-08-11

Read-only pass over `reference/sergeant-upstream/bin/` (37 entries: 29
`sgt-*`/`wiki-*` executables + 7 `_sgt-*.sh` libraries + a stale
`__pycache__/sgt-callbackcpython-312.pyc` build artifact, not a source
entry). Verdicts judge each script's **logical function** against sergeant-rs
today, not script-for-script. Product shelf verified against source before
use: `src/cli.rs` `Command` enum (daemon/status/run/work/respond/retry/
cancel/analytics/web/doctor), `src/api.rs` `/v1/work*`, `/v1/graph/work/{id}`,
`/v1/analytics*`, `/v1/events*`, `/v1/system`, `/healthz` (routes read
directly, `src/api.rs:381-413`), plus `docs/icm/retriage-2026-08-11.md`'s R1
shelf (engine-internal mechanisms: `recovery.rs` restart reconciliation,
`permission_mode` profile config, git-worktree materialization, append-only
journal + blob store, single-canonical-intent-per-Work, needs_input/respond
hold, work-neighborhood graph, disposable analytics projection) — cross-checked
against `src/cli.rs`/`src/api.rs`, matches.

No fixed catalogue of "operator skills" exists yet in this repo — the
`u-series-scope-draft`'s own §2(g) says the skills layer "does not exist yet
as a place in sergeant-rs's own repo." So SKILL verdicts below name the
harness-assistance job the script performs rather than picking from a
preexisting list of five.

## `sgt-*` / `wiki-*` verdicts (29)

| Script | What it logically does | Verdict | Rationale |
|---|---|---|---|
| `sgt-ack-response` | Exactly-once file-lock+archive protocol clearing a consumed worker response, tied to a gate-generation counter | ABSORBED | `Command::Respond`/`POST /v1/work/{id}/input` + needs_input hold + journal-is-only-truth make a separate ack step unnecessary — the append *is* the consumption record |
| `sgt-callback` | Durable, idempotent outbound webhook delivery (needs_input/blocked/failed/done) to an external profile, with retry/backoff/ack-reject and a cleanup seal | VERB-CANDIDATE | No push-to-external-system primitive exists; SSE (`/v1/events/stream`) is pull-only for connected clients, not a durable retried delivery guarantee — engine-gap G3 (retriage), narrowed to an ack-gate |
| `sgt-cleanup` | Safe multi-phase teardown of a completed Work's surfaces: kill worker process tree, remove/return the worktree, retire response-handshake state, atomic rollback on any step failure | ABSORBED | `recovery.rs`'s terminal-surface-teardown crash-window sweep + the daemon's exclusive process-handle ownership (one owner) + git-worktree materialization remove the entire lsof/process-tree/cross-filesystem-backup problem class this script exists to solve |
| `sgt-context` | Emit a project's layered agent-instructions block (defaults→group→repo) for session start | SKILL | Owner pre-ruling confirmed by reading the script: pure config read, no worktree or Work state touched |
| `sgt-dag-dispatch-hook` | Stage-ready hook: calls `sgt-dispatch`, writes `dagr` run/stage IDs into fleet state | INTEGRATION | Wraps external `dagr`; no engine primitive for cross-Work dependency graphs exists (single-Work stage sequencing is native, but this is cross-repo/cross-Work) |
| `sgt-dag-run` | Reads a project's `dag:` YAML block, creates/verifies it in `dagr`, dispatches initially-ready stages | INTEGRATION | Same as above — external DAG tool bolted onto shell dispatch; not absorbed because the multi-Work graph object it manages has no engine analog |
| `sgt-dispatch` | Core dispatch: per-repo isolated worktree, mission-brief templating, tmux-spawned agent, branch/deps/td-task handling, drain check | ABSORBED | `Command::Run`/`POST /v1/work` + git-worktree surface materialization + staged workflow execution is the native replacement; retriage already ruled `20-prepare-intent` ABSORBED (single-canonical-intent-per-Work) |
| `sgt-drain` | Set/clear a global or per-project admission block; `--wait` blocks until live workers finish; lock-protected state files | VERB-CANDIDATE | No admission-block primitive exists (`sgt fleet drain`) — engine-gap G4 (retriage) |
| `sgt-drain-force` | Force-terminate workers left running under an active drain, with identity-verified kill and dry-run preview | VERB-CANDIDATE | Same family as `sgt-drain`; `sgt fleet force-stop` |
| `sgt-graphify` | Run external `graphify` per repo, merge into one atomically-published project code graph | INTEGRATION | Owner pre-ruling confirmed: wraps github.com/Graphify-Labs/graphify end to end |
| `sgt-interactive-worker` | Own one persistent interactive agent pane: harness capability gate, readiness probe, cooperative drain, notification delivery, progress/stall recording | OBSOLETE | tmux-pane supervision is exactly the mechanics the daemon's exclusive process ownership retired; the underlying capability (agent turn execution) is absorbed by `daemon.rs`/`backend/claude.rs`/`recovery.rs`, not carried forward as a script |
| `sgt-list` | List known projects from `~/.config/sergeant/*.yaml` | VERB-CANDIDATE | `sgt project list` per retriage's `load-project` SPLIT verdict; NET-NEW-SURFACE, blocked on U-R2's `repos/` estate model |
| `sgt-no-mistakes-finding` | Apply a disposition (gate/td/ignore/ask-user) to one no-mistakes finding, dedup by marker search, create/update a td card | VERB-CANDIDATE | Same "gate"/finding-routing gap as `sgt-review-findings` below — no such domain concept in the engine |
| `sgt-notify` | Classify a worker update (completion/escalation/update), write a durable wake marker, sync callback, log wiki activity | ABSORBED | Journal event append + `GET /v1/events`/`/v1/events/stream` + needs_input hold is the native replacement for "tell the primary session something changed"; the tmux-injection transport it still carries is OBSOLETE and the wiki tail is INTEGRATION |
| `sgt-recover` | One bounded stall-recovery attempt for a live-but-idle `in_progress` worker: kill/relaunch after verifying stall proof and lease convergence | VERB-CANDIDATE | Retriage: `recovery.rs` only covers restart-time reconciliation, not live-turn staleness while the daemon stays up — a genuine gap, not a skill, because it needs privileged process control the daemon alone can safely exercise |
| `sgt-respond` | Deliver a human response; live-pane notify or dead-pane relaunch; drain check; notification-lease convergence; legacy-state migration | ABSORBED | `Command::Respond`/`POST /v1/work/{id}/input` + needs_input hold + one-owner process ownership eliminate the pane-identity/lease machinery entirely |
| `sgt-review-findings` | Route structured independent-review findings (severity/axis/disposition) to td, content-digest dedup, blocking-gate publication | VERB-CANDIDATE | Retriage: no "gate" or finding-routing concept anywhere in the engine — `sgt review route-findings`/`sgt gate clear` |
| `sgt-status` | Git status/branch/ahead-behind across every repo in a project | VERB-CANDIDATE | `sgt project status` per `load-project` SPLIT; NET-NEW-SURFACE pending the estate/repos registry |
| `sgt-sync` | Clone missing / pull existing repos for a project | SKILL | Owner pre-ruling confirmed: git-reading harness assistance, no Work state |
| `sgt-td-create` | Create one td task per target repo for a cross-repo brief, all-or-nothing rollback | VERB-CANDIDATE (td question, see below) | Creates a *standing* backlog item independent of execution — no analog; `sgt run` starts a Work immediately, there is no create-now/dispatch-later concept |
| `sgt-td-list` | Query/merge td tasks across a project's repos, filtered by status/priority | VERB-CANDIDATE (td question, see below) | Partly overlaps `sgt work list`/analytics for "what ran"; the human-triaged not-yet-started backlog view has no analog |
| `sgt-td-memory` | Record redacted handoff/response-provenance breadcrumbs into td for a future session, scoped to the owned worktree | OBSOLETE | See td answer below — the journal's full, unredacted, replayable history is a strictly more complete substitute reached through the same owned API |
| `sgt-treehouse-init` | Initialize external `treehouse` worktree pools per repo/group | INTEGRATION | Wraps github.com/kunchenguid/treehouse; optional dependency, no engine role |
| `sgt-undrain` | Clear a global or per-project drain record | VERB-CANDIDATE | Same family as `sgt-drain` |
| `sgt-validate` | Coordinate an isolated `no-mistakes` run in a second tmux pane beside the implementation worker: ownership handshake, HEAD/snapshot verification, atomic commit gating | OBSOLETE | Dual-pane handshake protocol is retired tmux-era mechanics; the logical function (independently verified validation gating ship) is carried by the promoted `validate-and-ship` WORKFLOW plus native git-worktree isolation, not by a script |
| `sgt-validation-worker` | Child half of `sgt-validate`'s handshake; runs `no-mistakes axi run` under coordinator supervision | OBSOLETE | Paired mechanism to the above |
| `sgt-wake` | Evaluate a durable wake condition (not_before/github_check/fleet_dependency/td_dependency/deployment/human_response) with backoff+jitter, resume or escalate | VERB-CANDIDATE | Retriage: engine-gap G1, no periodic/processless re-evaluation scheduler exists; cannot be a skill because it needs unattended re-invocation only a daemon can host |
| `sgt-watch` | Reconcile fleet state from worktree markers, classify live-but-idle stall, recycle terminal panes, advance `dagr`, machine-readable `--snapshot` busy-check | OBSOLETE | Substance is tmux-pane liveness/reconciliation plumbing the one-owner daemon retires; `--snapshot`'s "is Sergeant doing work" question is already answerable via `GET /v1/system`+`GET /v1/work`+SSE; the stall-classification piece is the same gap as `sgt-recover` |
| `wiki-daily-digest` | Synthesize a daily digest from opencode/goose/claude session history + merged PRs + td tasks via the Anthropic API into a personal wiki | INTEGRATION | Hardcoded to the original author's paths/repos (`~/wiki`, `ascend-arch-smith*`); retriage already recommends parking it — questionable fit for `sgt` at all |

**Counts: ABSORBED 5, SKILL 2, VERB-CANDIDATE 12, INTEGRATION 5, OBSOLETE 5** (29 total)

## `_sgt-*.sh` libraries (7) — helpers, not verbs

| Library | Logical function it carries | What survives |
|---|---|---|
| `_sgt-bash-version.sh` | Minimum-Bash-version gate | OBSOLETE — a compiled binary has no shell-version concern |
| `_sgt-drain.sh` | Drain state/lock file format + admission-lock hard-link protocol + Claude background-session stop/liveness | The admission-lock *concept* (not code) informs the `sgt-drain` VERB-CANDIDATE; the Claude-session reconciliation half is OBSOLETE (daemon owns the process directly, no separate background-session identity to reconcile) |
| `_sgt-harness.sh` | One registry driving capability-gate + readiness-probe + launch-args per harness (opencode/goose/claude), to stop the three from drifting | OBSOLETE mechanism (tmux render-probe); the discipline it encodes — one measured definition per backend, never three — is already how `src/backend/claude.rs` works (Rust's `Backend` trait enforces it structurally, per LESSONS L1) |
| `_sgt-intent.sh` | The "Sergeant Intent" structured document (8 required sections), SHA-revision hashing, re-verification before privileged actions, safety-keyword gate forcing `--intent-file` for stateful/security work | **Genuine gap, not carried anywhere** — see DELTA below; `Work.intent` today is free text with no structure or revision-integrity check |
| `_sgt-response-lock.sh` | File-lock + response-archive format + notification action-lease finalization for exactly-once delivery across process boundaries | OBSOLETE — the "prove a separate process consumed this exactly once" problem does not exist when the daemon owns the process and the journal provides exactly-once event append |
| `_sgt-review-axes.sh` | Canonical review axis (standards/spec/readiness/accessibility) + severity vocabulary shared between dispatch's brief and the findings router | ABSORBED into workflow *content*, not code — `code-review`'s "Two isolated axes" is already how the promoted WORKFLOW carries this vocabulary |
| `_sgt-lib.sh` | Grab-bag: env resolution, per-harness model/variant launch contract, `dev_root`/global config, path resolution, wiki write, `_require_*` gates, tmux pane-identity, notification publish/wait, td version gate, systemd-based per-task background monitor, unpushed-commit check | Split: per-harness launch contract → ABSORBED in spirit by the measured-adapter discipline (`backend/claude.rs`); config/path resolution → SKILL layer; wiki write → INTEGRATION; tmux pane-identity/notification/systemd monitor → OBSOLETE (daemon is itself the always-on supervisor); unpushed-commit check → ABSORBED into worktree-safety concerns `recovery.rs` already owns |

## DELTA — what upstream's core did that sergeant-rs genuinely does not do yet

1. **Admission-block ("drain")**: pause new work for a scope, optionally wait
   for in-flight work to finish, force-stop what doesn't. VERB-CANDIDATE
   (`sgt fleet drain`/`force-stop`) — rung: engine-gap G4 (retriage).
2. **Live-turn stall detection + bounded one-shot recovery**, distinct from
   restart-time reconciliation. VERB-CANDIDATE, engine primitive — rung:
   `recovery.rs` covers only restart, not a daemon-resident staleness
   detector for a turn still nominally running (retriage).
3. **Durable outbound callback delivery** bound to Work state transitions,
   idempotent, retried, ack/reject. VERB-CANDIDATE, engine primitive — rung:
   engine-gap G3, narrowed to an ack-gate (retriage).
4. **Periodic/processless wake-condition scheduler** (time, external CI
   check, dependency on other work, deployment). VERB-CANDIDATE, engine
   primitive — rung: engine-gap G1 (retriage).
5. **A "gate"/finding-routing domain concept**: review and no-mistakes
   findings need to become durable, deduped, human-triaged debt independent
   of the run that found them. VERB-CANDIDATE (`sgt review route-findings`/
   `sgt gate clear`) — rung: NET-NEW-SURFACE, no such concept exists
   (retriage).
6. **Project/estate registry** (list/status/sync across a project's repos).
   VERB-CANDIDATE (`sgt project list/status/sync`) — rung: NET-NEW-SURFACE,
   blocked on U-R2's `repos/` estate model landing first.
7. **Structured, revision-verified intent contract**: upstream re-hashes and
   re-checks an 8-section intent document before every privileged action;
   `Work.intent` today is unstructured free text with no integrity check.
   VERB-CANDIDATE / engine primitive (extend `Work`'s intent field) — rung:
   none yet — new finding this session, not previously on the R1 shelf or in
   the retriage sweep; worth an owner ruling.
8. **Standing task backlog independent of execution** (td's queue role
   beyond in-flight dedup). VERB-CANDIDATE, **OPEN** — this is the owner's
   own td question; rung: needs the fleet/backlog domain-concept ruling
   already flagged as open question 2 in `u-series-scope-draft-2026-08-11.md`.
