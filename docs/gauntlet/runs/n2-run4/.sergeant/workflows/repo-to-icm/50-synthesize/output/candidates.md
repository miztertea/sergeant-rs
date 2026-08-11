# Synthesis candidates — repo-to-icm N2 run (21-partition corpus)

Source: every classification record in `../40-classify/output/classifications.ndjson`
(**1333** records) clustered per `references/synthesis-method.md`'s seven
buckets. Behavior text (`statement`/`trigger`/`outcome`) is pulled from
`../30-normalize/output/behavior-units.normalized.ndjson` by matching `id`.

`../40-classify/output/classifications.ndjson` does not open with
`# AMBIGUOUS — NOT RESOLVED`, so this stage proceeded with its ordinary work
(`../_config/run-discipline.md` §2 checked, not triggered).

## Representation counts (input accounting)

| representation | count |
|---|---|
| `workflow` | 8 |
| `stage` | 174 |
| `stage-context` | 795 |
| `agents-invariant` | 126 |
| `shared-helper` | 23 |
| `shared-context` | 0 |
| `helper` | 207 |
| `obsolete-mechanism` | 0 |
| `engine-gap` | 0 |
| **total** | **1333** |

`shared-context` and `obsolete-mechanism` and `engine-gap` are 0 in this
run's corpus — buckets 5 (context half), 6, and 7 are reported empty below,
not omitted, per "what must not happen": every representation is accounted
for whether or not it has members.

## Note on stage ordering method (applies across every workflow candidate below)

Default order: ascending `behavior_id` within a workflow's stage set, used as
a proxy for source/document appearance order. This is a defensible default,
not a proven trigger→outcome chain, for two reasons: (1) behavior units were
extracted top-to-bottom from their source files during `20-harvest`, so
ascending id already tracks document order for any workflow whose stages
come from one procedural document (most of the smaller candidates below);
(2) for the handful of large, graph-shaped candidates whose stages are
independent entry points on a state machine rather than a single pipeline
(callback-protocol, dispatch-mode, review-findings-routing, triage, validation-pipeline-gate, worker-lifecycle), no single linear trigger→outcome chain
actually exists to recover — imposing one would be manufacturing a tidy
boundary the classification records do not support (synthesis-method.md's
own "what must not happen"). Those candidates are flagged individually
below with this same one-line reason rather than a fabricated per-stage
chain justification. Where a smaller candidate's trigger/outcome text made
the source order look wrong on inspection, it is called out inline instead.

## Bucket 1–3: Workflow candidates, their ordered stages, and stage-context attachments

**44** distinct workflow candidates: **41** named by a
`workflow` field value shared across `stage`/`stage-context`/`helper` records,
plus **3** standalone candidates established directly by a
`representation: workflow` record with no downstream `stage`/`stage-context`/`helper`
record naming it (single-behavior workflows — reported as such per "what must
not happen," not padded out). Of the 8 `workflow`-rung records, 5 match an
existing field-value cluster by shared source file/topic (matched below); the
other 3 are the standalone candidates.

### `callback-protocol`

- **Trigger:** a callback profile is installed or invoked
- **Outcome:** delivery uses a claim-with-timeout lease pattern and bounded backoff/batch size rather than unbounded retry storms or unclaimed concurrent delivery
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `retry-delivery`'s.
- **Member stage count:** 7
- **Ordering note:** `callback-protocol` is graph-shaped (independent event-triggered
  entry points, not a single pipeline) — source/behavior_id order is used as
  the defensible default per the run-wide note above, not a proven chain.

**Workflow-level helpers** (`workflow=callback-protocol`, `stage=null` — deterministic
machinery referenced by the workflow as a whole, not one specific stage):

- `BU-0761` — Task IDs, callback profile names, and correlation IDs must each match a strict identifier pattern before being used to build any filesystem path or event record; correlation IDs are additionally rejected if they contain a long (17-20 digit) numeric platform identifier.
- `BU-0763` — A directory the callback-delivery step creates or reuses for its own callback state is forced to mode 0700 — created private from the start if new, or re-tightened to private if it already existed with looser permissions.
- `BU-0765` — Every durable file the callback-delivery step writes (origin, sequence, event, state, seal) is published by writing to a temp file in the same directory, fsyncing the file, atomically renaming it into place, and then fsyncing the containing directory — so a crash can never leave a half-written file visible at its final path.
- `BU-0768` — All mutating callback operations on one task (enqueue, drain, retry, seal, unseal) are serialized against each other by opening the task's lock file without following a symlink, verifying it is a regular user-owned file, forcing its mode to 0600, and then holding an exclusive flock for the whole critical section.
- `BU-0815` — Every domain-specific failure in the callback-delivery step (any CallbackError raised anywhere — validation, locking, or I/O) is caught once at the top level, reported to stderr with a stable 'sgt-callback: ' prefix, and causes the process to exit 1.

**Ordered member stages:**

1. **`resolve-callback-executable`** — `BU-0214` (`docs/callbacks.md (docs/callbacks.md L10-15)`)
   - Trigger: a callback profile is installed or invoked
   - Outcome: only a locally-installed, ownership-and-permission-verified executable can ever run as a callback, never a path supplied through request/fleet data
   - Statement: A callback profile executable must be real (not a symlink), owned by the Sergeant user, not group/world writable, and have its owner execute bit set; Sergeant never accepts an executable path from fleet state.
   - Stage-context attachments (4):
     - `BU-0215`: `SERGEANT_CALLBACKS` may select a different fixed callbacks directory for tests or an isolated service account, but that environment setting is itself treated as trusted local configuration, never as request input.
     - `BU-0762`: Every directory the callback-delivery step trusts (fleet root, task directory, callbacks root, callback event directories) must be a real directory, not a symlink, owned by the invoking user, and — unless explicitly marked writable-ok — not group- or world-writable, or the operation fails.
     - `BU-0764`: Before a callback profile can be registered against or invoked for any task, its executable file must be verified as a real, non-symlink, user-owned, non-group/world-writable file that is executable by its owner — any violation is reported as the profile not being installed or not executable, not silently accepted.
     - `BU-0766`: Reading a trusted file re-checks, on the opened file descriptor, that it is still a regular file owned by the current user (and, where a mode is required, has exactly that mode) after the open — not only via the pre-open lstat — and rejects a file exceeding its declared maximum size rather than silently truncating it.

2. **`register-origin`** — `BU-0216` (`docs/callbacks.md (docs/callbacks.md L31-33)`)
   - Trigger: a correlation ID is supplied at origin registration
   - Outcome: the ID is validated to be opaque and rejects anything shaped like a real platform identifier
   - Statement: A correlation ID must match `^[a-z][a-z0-9._-]{7,127}$` and must not contain a 17-20 digit platform ID; a new opaque request ID is used, never a Discord guild/channel/user/message ID.
   - Stage-context attachments (5):
     - `BU-0217`: Origin registration writes only `.callbacks/origin.json` with correlation_id, profile, and version; it never stores request text, Discord IDs, destination IDs, tokens, secrets, message content, callback commands, or logs.
     - `BU-0218`: Repeating the same origin registration is idempotent; changing an existing registration is rejected.
     - `BU-0767`: Reading back a task's registered callback origin re-validates its schema version and re-checks the profile name and correlation ID against the same format rules enforced at registration time — a stored origin is never trusted merely because it exists on disk.
     - `BU-0781`: Registering a callback origin for a task first validates that the named profile is an installed, safely-permissioned executable and that the correlation ID is well-formed, before any origin record is written.
     - `BU-0782`: Registering a callback origin for a task that already has one registered is a silent no-op if the new registration is identical to the existing one, and a hard failure if it differs — a task's callback origin is pinned once and cannot be silently changed thereafter.

3. **`sync-and-produce-events`** — `BU-0219` (`docs/callbacks.md (docs/callbacks.md L67-70)`)
   - Trigger: the callback-delivery step sync runs repeatedly against the same underlying state
   - Outcome: re-running sync is idempotent — it never fabricates a new event generation for state it has already classified
   - Statement: Waiting-event identity includes the repository, class, and `.sergeant-gate-generation`; terminal identity includes the repository and terminal class; repeated synchronization of the same source creates no new generation.
   - Stage-context attachments (10):
     - `BU-0229`: Automatic callback producers make one bounded delivery attempt and return without waiting indefinitely; events survive callback/process restarts.
     - `BU-0604`: The interactive fleet-watch loop triggers the callback-delivery step sync for a task only when that task's fleet state carries a .callbacks/origin.json marker (as a file or symlink); tasks with no such marker never invoke the callback path during reconciliation.
     - `BU-0681`: When a task has a registered durable callback origin, the notify step triggers a sync of that task's durable callback events as part of handling the update.
     - `BU-0682`: A failed durable-callback sync during a notify call does not abort the notify call itself, but is recorded so the command's own exit status still reflects the failure.
     - `BU-0783`: When determining a repo's authoritative status for callback purposes, a recorded worktree pointer's own status file is consulted in preference to the fleet-level status file, and the worktree pointer itself must be an absolute path to a real, user-owned directory or the lookup fails.
     - `BU-0785`: A repo in needs_input or blocked status produces a callback event of the matching type whose deduplication source id is derived from a hash of the repo name, the status, and the gate generation together — so repeated syncs of the same unresolved gate do not enqueue duplicate events, but the gate generation advancing produces a fresh one.
     - `BU-0786`: A repo whose authoritative status is failed:<reason> produces a 'failed' callback event whose payload is the reason text following the prefix, with a source id scoped as a one-time terminal marker for that repo.
     - `BU-0787`: A repo whose authoritative status is 'done' produces a 'done' callback event carrying the task's result content, with a source id scoped as a one-time terminal marker for that repo.
     - `BU-0788`: A failure syncing one repo's callback status within a multi-repo task does not prevent the other repos in that task from being synced — per-repo failures are collected across all repos and reported together once every repo has been attempted.
     - `BU-0789`: Syncing a task's callback events always attempts a bounded (one-event) drain at the end of the call, whether or not new events were enqueued during that same sync.
   - Helper attachments (1):
     - `BU-0784`: Syncing a task's callback events is a no-op if the task has no registered callback origin.

4. **`enqueue-event`** — `BU-0220` (`docs/callbacks.md (docs/callbacks.md L80-82)`)
   - Trigger: the callback-delivery step enqueue is called
   - Outcome: the source identity is validated, never stored in plaintext, and re-use is idempotent rather than creating a duplicate event
   - Statement: The source ID for an enqueued event must match `^[a-z][a-z0-9._:-]{0,127}$` and is hashed before persistence; reusing the same event class/source ID returns the original event and generation.
   - Stage-context attachments (11):
     - `BU-0221`: Callback payloads must be nonempty UTF-8, at most 4096 bytes and 16 lines, contain no NUL/control data other than tab/newline, and Sergeant rejects shell command metacharacters, command-like lines, secret-shaped assignments, and 17-20 digit platform IDs.
     - `BU-0769`: Any callback event payload is capped at 4096 bytes, must decode as UTF-8, and — after a trailing newline is stripped — must be non-empty and no more than 16 lines, or it is rejected before being stored or delivered.
     - `BU-0770`: A callback event payload is rejected if it contains raw control characters (other than newline/tab), shell/command metacharacters, a line that looks like an executable command invocation (shebang, bash/sh/curl/wget/ssh/sudo/rm/mv/cp/chmod/chown/python/node at line start), a secret-shaped key:value pattern (password/token/secret/api-key/etc.), or a long numeric platform identifier.
     - `BU-0771`: Enqueuing a callback event for a source that has already produced an event of the same type is idempotent — the existing event is returned unchanged rather than a duplicate being created.
     - `BU-0772`: A stored callback event's field set and schema version must match exactly, its generation must be a bounded positive integer, and the directory it is stored under must be named as the zero-padded form of that same generation, or it is rejected as an unsupported or inconsistent event.
     - `BU-0773`: A stored callback event's correlation ID and profile must exactly match the task's registered origin; an event does not carry independent, potentially-diverged identity from the origin it belongs to.
     - `BU-0774`: A stored callback event's idempotency key is not trusted from storage — it is recomputed from the event's correlation ID and generation and compared against the stored value, and a mismatch is rejected.
     - `BU-0775`: Reading back a stored callback event re-validates its type against the known set, re-runs the same content-safety checks on its payload that were applied when it was first enqueued, and checks that its source hash and creation timestamp are well-formed.
     - `BU-0778`: A new callback event's generation number is computed as one more than the higher of a durably persisted sequence counter and the highest generation number already present among that task's event directories, and the advanced counter is itself durably written before the generation is handed out.
     - `BU-0780`: A newly enqueued callback event's event.json and state.json are written into a freshly created, privately-permissioned temporary directory and only made visible under their generation-numbered name via a single atomic directory rename, with the temporary directory's contents cleaned up on any failure.
     - `BU-0813`: The enqueue CLI command refuses to enqueue a callback event for a task that has no registered callback origin.
   - Helper attachments (1):
     - `BU-0814`: The enqueue CLI command reads its payload from stdin bounded to one byte more than the maximum allowed payload size, so the size check itself can detect an over-limit read rather than silently truncating it.

5. **`invoke-consumer`** — `BU-0222` (`docs/callbacks.md (docs/callbacks.md L96-98)`)
   - Trigger: a callback event is delivered to its consumer executable
   - Outcome: the consumer receives a minimized, argument-free, environment-scrubbed invocation surface
   - Statement: Sergeant invokes the fixed callback profile with no arguments, a minimal environment (HOME, PATH, locale, and temp-directory variables only), stderr discarded, and exactly one compact UTF-8 JSON object on stdin.
   - Stage-context attachments (4):
     - `BU-0223`: The consumer must durably deduplicate by `idempotency_key` before creating a user-visible message, because a crash after external delivery but before acknowledgement causes an intentional retry.
     - `BU-0224`: The callback executable has 15 seconds by default (`SGT_CALLBACK_TIMEOUT_SECONDS`, range 1-120) and may write at most 1024 bytes to stdout.
     - `BU-0234`: The ws-lab hermes-discord consumer must forward the unchanged event through its source-bound forced transport, deduplicate durably by idempotency_key, map only the four event classes to bounded Discord text, return ack only after the fixed approved destination confirms delivery, and must never accept destination IDs or commands from any event field; Discord and Doppler credentials remain exclusively on the Hermes host and must not appear in the callback executable environment, stdout, stderr, or Sergeant fleet state.
     - `BU-0790`: Invoking a callback profile passes it the event JSON on stdin and only a fixed allowlist of environment variables (HOME, PATH, LANG, LC_ALL, TMPDIR) — the invoking process's full environment, which may hold secrets, is never exposed to the callback subprocess.

6. **`process-acknowledgement`** — `BU-0225` (`docs/callbacks.md (docs/callbacks.md L134-138)`)
   - Trigger: a consumer returns from a callback invocation
   - Outcome: the event's next state is determined by this closed set of outcomes, with every malformed/unexpected response defaulting to pending (never silently ack'd)
   - Statement: `ack` durably suppresses all later callback attempts for that generation; `retry` keeps the event pending with an optional bounded `retry_after_seconds` (0-3600); `reject` records a permanent policy failure without deleting or auto-retrying the event; any timeout, nonzero exit, malformed JSON, wrong version/key, unknown field/status, or oversized output leaves the event pending.
   - Stage-context attachments (10):
     - `BU-0226`: Consumer stderr and output details are never persisted.
     - `BU-0776`: A stored callback delivery state's field set and schema version must match exactly, and its status must be one of pending, delivering, acknowledged, or rejected, or it is rejected as unsupported or invalid.
     - `BU-0777`: A stored callback delivery state's attempt count and next-attempt time must be non-negative integers, its claim and acknowledgement timestamps must be the correct type or absent, and its last delivery result must be one of a fixed set of outcomes.
     - `BU-0791`: A callback invocation is bounded by a configurable timeout (1-120 seconds); exceeding it is treated as its own distinct delivery outcome ('timeout') rather than letting the call hang indefinitely or crash the drain loop.
     - `BU-0792`: A non-zero exit from the callback executable is always treated as a delivery failure ('callback_error'), never as an implicit success.
     - `BU-0793`: The callback's stdout is size-capped, and any oversized, non-UTF8, non-JSON, or non-dict response is treated as 'invalid_ack' rather than assumed to mean success.
     - `BU-0794`: The callback acknowledgement's field set is an exact allowlist (version, idempotency_key, status, retry_after_seconds); any unexpected field invalidates the whole response.
     - `BU-0795`: A callback's acknowledgement is only accepted if its version and idempotency key exactly match the event that was sent to it — an ack cannot be misapplied to acknowledge a different event.
     - `BU-0796`: A callback acknowledgement's status must be ack, retry, or reject; retry_after_seconds is only meaningful — and only accepted — when status is retry, and must be a bounded non-negative integer when present.
     - `BU-0803`: An acknowledged outcome moves a callback event to a terminal 'acknowledged' state; a reject outcome moves it to a terminal 'rejected' state; any other outcome (timeout, callback_error, invalid_ack, retry) returns it to 'pending' with a computed backoff — an event is never left stuck in 'delivering'.

7. **`retry-delivery`** — `BU-0227` (`docs/callbacks.md (docs/callbacks.md L146-149)`)
   - Trigger: a callback event delivery is retried
   - Outcome: delivery uses a claim-with-timeout lease pattern and bounded backoff/batch size rather than unbounded retry storms or unclaimed concurrent delivery
   - Statement: Each callback event is claimed before invocation, a stale `delivering` claim becomes eligible again after 60 seconds by default, failed attempts use exponential backoff (5-300 seconds by default), and each drain processes a bounded number of distinct events.
   - Stage-context attachments (11):
     - `BU-0228`: After repairing a permanent consumer policy/configuration failure, an operator can requeue a retained event with the callback-delivery step without changing its idempotency key.
     - `BU-0797`: A callback-requested retry delay (retry_after_seconds) is honored as given; absent that, the retry delay is computed as an exponential backoff from a configurable base, capped at a configurable maximum.
     - `BU-0798`: Before a callback event is delivered, the callback-delivery step durably records it as 'delivering' (with an incremented attempt count and claim timestamp) while holding the task lock, then releases the lock before actually invoking the callback subprocess.
     - `BU-0799`: A callback event already claimed as 'delivering' is only eligible to be reclaimed for a fresh attempt once its claim has been held longer than a configurable claim timeout — a still-fresh in-flight claim is left alone.
     - `BU-0800`: Callback events in a terminal state (acknowledged or rejected) are never revisited by drain.
     - `BU-0801`: Callback events not yet due (next_attempt_at in the future) are skipped for the current drain pass.
     - `BU-0802`: After a callback invocation returns, the callback-delivery step only writes the delivery outcome back if the event's stored state still exactly matches the attempt count and claim timestamp this same attempt itself set — if the state has since changed, the outcome is discarded rather than overwriting whatever changed it.
     - `BU-0804`: Draining all tasks does not abort the whole sweep on one task's failure — every fleet task with a registered callback origin is attempted, and failures are collected and reported together at the end.
     - `BU-0807`: A callback event that has already reached the acknowledged terminal state cannot be retried.
     - `BU-0808`: Retrying a matched callback event resets its state to immediately eligible (pending, next_attempt_at=0, claimed_at=None), so the next drain attempts it right away rather than waiting for its previously scheduled backoff.
     - `BU-0809`: retry_event fails if the given idempotency key does not match any known event for the task.

### `check-repo-status`

- **Trigger:** (no stage candidates for this workflow value)
- **Outcome:** (no stage candidates for this workflow value)
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `(none)`'s.
- **Member stage count:** 0

**Workflow-level helpers** (`workflow=check-repo-status`, `stage=null` — deterministic
machinery referenced by the workflow as a whole, not one specific stage):

- `BU-0239` — For each repo, the status step reports NOT CLONED or NOT A GIT REPO and skips further git inspection for that repo, rather than attempting git commands against a missing or non-repo path.
- `BU-0240` — When a repo has an upstream branch, the status step reports the ahead/behind commit counts relative to that upstream whenever either is nonzero.

_No `stage`-rung records carry this `workflow` value — candidate has no_
_member stages (a `stage`/`stage-context`/`helper`-only cluster whose_
_checkpoint boundary, if any, was never classified as `stage` rung)._

### `ci-verification`

- **Trigger:** the toolchain/task runner run test:docker:drain runs
- **Outcome:** compatibility is proven under both the host's ambient Bash and the minimum supported Bash 3.2, not just one
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `verify-bash-compat-both-passes`'s.
- **Member stage count:** 1

**Ordered member stages:**

1. **`verify-bash-compat-both-passes`** — `BU-0212` (`mise.toml (mise.toml L225-282)`)
   - Trigger: the toolchain/task runner run test:docker:drain runs
   - Outcome: compatibility is proven under both the host's ambient Bash and the minimum supported Bash 3.2, not just one
   - Statement: The Docker drain test suite runs the same test file in two passes — system Bash under Debian bookworm-slim, and Bash 3.2 under an official Alpine bash:3.2 image — and reports overall pass/fail only once both passes are accounted for.

### `code-review`

Established by `BU-0927` (`.agents/skills/code-review/SKILL.md (.agents/skills/code-review/SKILL.md L6-11)`): A code review since a fixed point is evaluated along two independent axes: whether the code conforms to the repo's documented coding standards (Standards), and whether it faithfully implements what the originating issue/spec asked for (Spec).

- **Trigger:** both axes are ready to be evaluated
- **Outcome:** the two axes stay visibly separate in the final report
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `aggregate-review-report`'s.
- **Member stage count:** 3

**Ordered member stages:**

1. **`run-parallel-axis-reviews`** — `BU-0928` (`.agents/skills/code-review/SKILL.md (.agents/skills/code-review/SKILL.md L6-11)`)
   - Trigger: both axes are ready to be evaluated
   - Outcome: each axis's findings are reasoned about in an isolated context before being combined
   - Statement: The Standards and Spec reviews each run as a separate parallel sub-agent so that neither review's context-gathering pollutes the other's, and this skill aggregates their findings afterward.
   - Stage-context attachments (6):
     - `BU-0933`: The Standards axis always carries a fixed baseline of Fowler code smells in addition to whatever the repo's own documented standards say, applying even when the repo documents nothing.
     - `BU-0934`: A documented repo standard always overrides the smell baseline: where the repo endorses something the baseline would flag, the smell is suppressed.
     - `BU-0935`: Every baseline smell is reported as a judgement-call heuristic, never a hard violation, and is skipped if tooling already enforces it.
     - `BU-0936`: The Standards and Spec sub-agents are spawned together in a single message (two `Agent` tool calls), both using the `general-purpose` subagent type.
     - `BU-0937`: The Standards sub-agent's brief requires it to report documented-standard violations (citing the file and rule) separately from baseline smells (named and quoted), explicitly distinguish hard violations from judgement calls, skip anything tooling enforces, and stay under 400 words.
     - `BU-0938`: The Spec sub-agent's brief requires it to report requirements missing or partial, behaviour not asked for (scope creep), and requirements that look implemented but wrong, quoting the spec line for each finding, and stay under 400 words.

2. **`prepare-review-inputs`** — `BU-0931` (`.agents/skills/code-review/SKILL.md (.agents/skills/code-review/SKILL.md L23-23)`)
   - Trigger: always, prior to spawning the sub-agents
   - Outcome: a bad ref or an empty diff fails the review at this checkpoint instead of surfacing confusingly inside two parallel sub-agents
   - Statement: Before spawning the two review sub-agents, the skill confirms the fixed point resolves (`git rev-parse <fixed-point>`) and that the diff is non-empty.
   - Stage-context attachments (4):
     - `BU-0929`: If the repo's issue-tracker doc is missing, /setup-matt-pocock-skills is run to establish it before the review proceeds.
     - `BU-0930`: If the user doesn't specify a fixed point for the review, the skill asks for it rather than guessing.
     - `BU-0932`: The spec source for the Spec axis is looked up in a fixed priority order: issue references in commit messages, a path the user passed as an argument, a matching PRD/spec file under docs/specs/.scratch, and finally asking the user directly if nothing is found.
     - `BU-0939`: If no spec was found, the Spec sub-agent is skipped entirely (never run without a spec) and its absence is noted in the final report.

3. **`aggregate-review-report`** — `BU-0940` (`.agents/skills/code-review/SKILL.md (.agents/skills/code-review/SKILL.md L78-78)`)
   - Trigger: both sub-agent reports have returned
   - Outcome: the two axes stay visibly separate in the final report
   - Statement: The two sub-agent reports are presented verbatim (or lightly cleaned) under separate `## Standards` and `## Spec` headings, never merged or reranked against each other.
   - Stage-context attachments (1):
     - `BU-0941`: The report ends with a one-line summary of total findings per axis and the worst issue within each axis, without ever picking one overall winner across the two axes.

### `cross-repo-work`

- **Trigger:** a requested outcome is being decomposed across repositories
- **Outcome:** completion claims require every owning repo to individually be terminal or explicitly blocked, not merely a subset
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `reconcile-cross-repo-outcome`'s.
- **Member stage count:** 4

**Ordered member stages:**

1. **`assign-ownership`** — `BU-0267` (`skills/cross-repo-work/SKILL.md (skills/cross-repo-work/SKILL.md L23-25)`)
   - Trigger: a requested outcome is being decomposed across repositories
   - Outcome: ownership assignment is unambiguous (exactly one owner per behavior) and scoped to repos that actually must change
   - Statement: For each required behavior, exactly one repository is named as owning its implementation, and a repository is included only when it must change or produce delivery evidence.
   - Stage-context attachments (1):
     - `BU-0268`: The user is asked about repository ownership only when two repositories could legitimately own a user-visible or durable contract; otherwise ambiguity is resolved from the project graph and existing contracts.

2. **`order-dependencies`** — `BU-0269` (`skills/cross-repo-work/SKILL.md (skills/cross-repo-work/SKILL.md L53-54)`)
   - Trigger: a dependency graph among repositories contains a cycle
   - Outcome: dispatch does not proceed with a cyclic dependency graph; the cycle is broken by design instead
   - Statement: Dependency cycles are rejected before dispatch; if a cycle reflects a genuinely coupled contract, a contract artifact or compatibility phase is defined to break the cycle instead of dispatching a cyclic dependency graph.
   - Stage-context attachments (1):
     - `BU-0270`: Repository state is never stashed, reset, switched, or cleaned during cross-repo planning; an existing canonical branch/worktree is routed to the worker brief instead, or the procedure stops for a decision when state conflicts with the requested outcome.

3. **`handoff-plan`** — `BU-0271` (`skills/cross-repo-work/SKILL.md (skills/cross-repo-work/SKILL.md L81-85)`)
   - Trigger: cross-repo decomposition is complete
   - Outcome: the outcome (plan-only vs implement) matches exactly what was requested, and multi-repo direct editing by the primary session never happens
   - Statement: If the user requested planning only, the procedure stops after returning repository briefs, acceptance evidence, and the dependency graph, without dispatching or editing any repository; when implementation was requested, `dispatch` is loaded and the primary session never edits several repositories directly.

4. **`reconcile-cross-repo-outcome`** — `BU-0272` (`skills/cross-repo-work/SKILL.md (skills/cross-repo-work/SKILL.md L95-96)`)
   - Trigger: a cross-repo outcome is being reported
   - Outcome: completion claims require every owning repo to individually be terminal or explicitly blocked, not merely a subset
   - Statement: A cross-repo outcome is never reported complete until every owning repository has reached a terminal result or has an explicit preserved blocker.

### `dag-run`

- **Trigger:** a DAG stage is defined
- **Outcome:** the hook fails loudly rather than dispatching work it cannot later attribute to the DAG runner run
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `run-dispatch-hook`'s.
- **Member stage count:** 4

**Workflow-level helpers** (`workflow=dag-run`, `stage=null` — deterministic
machinery referenced by the workflow as a whole, not one specific stage):

- `BU-0201` — A project's `dag.name` must be unique across all projects known to the DAG runner, since the DAG-run step runs it as a DAG identified by that name.
- `BU-0579` — The interactive fleet-watch loop only advances a linked DAG runner DAG run when both dagr_run_id and dagr_stage_id are recorded for a repo and the DAG runner binary is available on PATH; otherwise it silently does nothing.
- `BU-0580` — When advancing a linked DAG runner DAG run, the interactive fleet-watch loop reports the literal result 'done' only for a done worktree status; every other terminal status is passed through verbatim as the step result string.
- `BU-0865` — Creating the DAG runner DAG is idempotent: an error from the DAG runner because the DAG already exists is deliberately ignored rather than failing the run.
- `BU-0866` — Adding each DAG runner stage is idempotent: an error from the DAG runner because the stage already exists is deliberately ignored rather than failing the run.

**Ordered member stages:**

1. **`resolve-stage-brief`** — `BU-0202` (`schema/project.yaml.example (schema/project.yaml.example L112-115)`)
   - Trigger: a DAG stage is defined
   - Outcome: the stage's brief source is one of the two named alternatives, resolved by whether the task tracker is set
   - Statement: Each DAG stage names the repos it dispatches to and pulls its brief from the task tracker task via `td:`, or from an explicit inline `brief:` when `td:` is not set.
   - Stage-context attachments (2):
     - `BU-0867`: A stage's explicit `brief` is only passed through to dispatch when the stage has no task tracker task reference; when both the task tracker and `brief` are given for a stage, the task tracker takes precedence and the explicit brief is dropped.
     - `BU-0868`: Every DAG stage is registered with the DAG dispatch hook (not the dispatch step directly) as its DAG runner hook, so stage readiness always routes through the hook that also writes the DAG runner tracking files.

2. **`advance-on-dependency-completion`** — `BU-0203` (`schema/project.yaml.example (schema/project.yaml.example L117-120)`)
   - Trigger: a DAG stage declares an `after:` dependency
   - Outcome: the stage only becomes ready to dispatch once its named predecessor stages have completed, advanced automatically by the interactive fleet-watch loop
   - Statement: A DAG stage's `after:` list names the stages that must complete before it runs, and the interactive fleet-watch loop auto-advances the DAG when fleet tasks complete.
   - Stage-context attachments (1):
     - `BU-0869`: A dag.stages entry's `after` list is translated into the DAG runner's own --depends-on flag, so stage ordering declared in the project YAML is enforced by the DAG runner itself, not re-implemented by the DAG-run step.

3. **`verify-dag-prerequisites`** — `BU-0859` (`bin/sgt-dag-run (bin/sgt-dag-run L39-42)`)
   - Trigger: the DAG runner is not on PATH when the DAG-run step is invoked
   - Outcome: the run fails closed with actionable install guidance rather than failing deep inside a later DAG-runner call
   - Statement: The DAG-run step refuses to run if the DAG runner binary is not installed, reporting it as an optional dependency with install instructions, and states that all other Sergeant commands work without the DAG runner.
   - Stage-context attachments (6):
     - `BU-0860`: The DAG-run step refuses to run if yq is not installed.
     - `BU-0861`: The DAG-run step refuses to run if the named project's YAML config file does not exist.
     - `BU-0862`: The DAG-run step refuses to run if the project's YAML config has no dag.name field.
     - `BU-0863`: The DAG-run step refuses to run if the project's dag block defines zero stages.
     - `BU-0864`: In --dry-run mode, the DAG-run step only prints what it would create or update — no DAG runner dag/stage/run mutation call is ever made, per-stage dry-run prints included, and the script exits before starting a run.
     - `BU-0870`: After the DAG runner has already started the run, the DAG-run step best-effort parses the run ID out of the DAG runner's own output (trying a UUID pattern, then a 'started run' token pattern), and falls back to printing a literal placeholder in the monitor-command hint if neither parse succeeds — a failure to parse the ID never undoes or reports the run as failed.

4. **`run-dispatch-hook`** — `BU-0871` (`bin/sgt-dag-dispatch-hook (bin/sgt-dag-dispatch-hook L21)`)
   - Trigger: the DAG runner calls the hook without DAGR_RUN_ID set
   - Outcome: the hook fails loudly rather than dispatching work it cannot later attribute to the DAG runner run
   - Statement: The DAG dispatch hook refuses to proceed (exits 1) if DAGR_RUN_ID is not set in its environment.
   - Stage-context attachments (5):
     - `BU-0872`: The DAG dispatch hook refuses to proceed (exits 1) if DAGR_STAGE_ID is not set in its environment.
     - `BU-0873`: If the fleet task ID cannot be parsed out of the dispatch step's output, the hook logs a warning and exits 0 (success) rather than failing the DAG runner stage — dispatch has already happened by this point and is not undone.
     - `BU-0874`: Once a fleet task ID is known, the hook writes the DAG runner run ID and stage ID into every one of that task's dispatched-repo directories, so the interactive fleet-watch loop can later read them back to auto-advance the DAG when the task completes.
     - `BU-0875`: Dagr tracking files are only written if the fleet task's state directory actually exists on disk; if it does not, the hook silently skips writing tracking files (and still exits successfully).
     - `BU-0876`: The hook's final stdout output is exactly the fleet task ID, which the DAG runner records as the hook's dispatch_id for the stage.

### `design-it-twice`

- **Trigger:** the user wants to explore alternative interfaces for a chosen deepening candidate
- **Outcome:** the user receives a structured, sequential presentation and a comparison along three named axes
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `compare-and-recommend`'s.
- **Member stage count:** 3

**Ordered member stages:**

1. **`frame-problem-space`** — `BU-1049` (`.agents/skills/codebase-design/DESIGN-IT-TWICE.md (.agents/skills/codebase-design/DESIGN-IT-TWICE.md L11-17)`)
   - Trigger: the user wants to explore alternative interfaces for a chosen deepening candidate
   - Outcome: the user has a problem-space framing to read concurrently with sub-agents already working, instead of waiting idle
   - Statement: Before spawning parallel design sub-agents, write and show the user a framing explanation of the problem space for the chosen deepening candidate — the constraints any new interface must satisfy, the dependencies and their category, and a rough illustrative (non-proposal) code sketch — then proceed immediately to spawning sub-agents while the user reads.

2. **`spawn-design-subagents`** — `BU-1050` (`.agents/skills/codebase-design/DESIGN-IT-TWICE.md (.agents/skills/codebase-design/DESIGN-IT-TWICE.md L21)`)
   - Trigger: alternative interface designs are being explored for a deepening candidate
   - Outcome: at least three meaningfully distinct interface designs are produced instead of one
   - Statement: At least 3 sub-agents are spawned in parallel to design the deepened module's interface, each required to produce a radically different interface.
   - Stage-context attachments (4):
     - `BU-1051`: Each design sub-agent is prompted with its own separate technical brief (file paths, coupling details, dependency category, what sits behind the seam), kept independent of the user-facing problem-space explanation shown in step 1.
     - `BU-1052`: Each spawned design sub-agent is assigned a different design constraint from a fixed contrasting set (minimize the interface, maximize flexibility, optimize for the most common caller, or design around ports & adapters) so their outputs diverge meaningfully.
     - `BU-1053`: Each design sub-agent's brief includes both the codebase-design skill's own vocabulary and the project's CONTEXT.md vocabulary, so sub-agents name things consistently with both the architecture language and the project's domain language.
     - `BU-1054`: Each design sub-agent's output must cover five specific elements: the interface itself (types, methods, params, invariants, ordering, error modes), a usage example, what the implementation hides behind the seam, the dependency/adapter strategy, and the trade-offs.

3. **`compare-and-recommend`** — `BU-1055` (`.agents/skills/codebase-design/DESIGN-IT-TWICE.md (.agents/skills/codebase-design/DESIGN-IT-TWICE.md L42)`)
   - Trigger: multiple alternative interface designs have been produced and are ready for review
   - Outcome: the user receives a structured, sequential presentation and a comparison along three named axes
   - Statement: The resulting alternative designs are presented to the user sequentially and compared in prose by depth (leverage at the interface), locality (where change concentrates), and seam placement.
   - Stage-context attachments (1):
     - `BU-1056`: After comparing the alternative designs, an explicit, opinionated recommendation is given for which design is strongest and why, with a hybrid proposed if elements from different designs would combine well — never left as an unopinionated menu of options.

### `diagnose-bug`

- **Trigger:** diagnosing any hard bug
- **Outcome:** the bug is not declared done until all five completion conditions hold
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `declare-bug-fixed`'s.
- **Member stage count:** 5

**Ordered member stages:**

1. **`build-feedback-loop`** — `BU-0944` (`.agents/skills/diagnosing-bugs/SKILL.md (.agents/skills/diagnosing-bugs/SKILL.md L14-14)`)
   - Trigger: diagnosing any hard bug
   - Outcome: effort concentrates on constructing the loop before anything else is attempted
   - Statement: A tight feedback loop that goes red specifically on this bug is the prerequisite for finding its cause; bisection, hypothesis-testing, and instrumentation only consume that signal, they do not replace it, and no amount of reading code substitutes for having one.
   - Stage-context attachments (7):
     - `BU-0943`: CONTEXT.md (if present) is read to build a mental model of the relevant modules, and ADRs in the touched area are checked, before further codebase exploration.
     - `BU-0945`: As a last resort when no other feedback loop can be constructed, a human-in-the-loop bash script is used to drive the human through a structured repro loop instead of an ad hoc manual one, with the captured output fed back to the actor.
     - `BU-0946`: Once a feedback loop exists, it is deliberately tightened along three dimensions: making it faster, making the signal sharper (assert on the specific symptom, not just 'didn't crash'), and making it more deterministic (pin time, seed RNG, isolate filesystem, freeze network).
     - `BU-0947`: For a non-deterministic bug, the goal shifts from a single clean repro to a higher reproduction rate — looping the trigger repeatedly, parallelising, adding stress, narrowing timing windows, injecting sleeps — until the failure rate is high enough to debug against.
     - `BU-0948`: If the actor genuinely cannot construct a feedback loop after trying, it stops and says so explicitly, lists what was tried, and asks the user for environment access, a captured artifact, or permission to add temporary production instrumentation — it does not proceed to hypothesise without a loop.
     - `BU-0949`: Phase 1 is complete only when the actor can name one already-run command, with its pasted invocation and output, that is simultaneously red-capable (catches the user's exact symptom, not merely 'runs without erroring'), deterministic, fast, and agent-runnable (any human-in-the-loop step going only through the HITL script).
     - `BU-0950`: If the actor notices itself reading code to build a theory before a red-capable command exists, it stops immediately — jumping to a hypothesis without a red-capable command is the exact failure this skill exists to prevent, and Phase 2 is not entered without one.

2. **`reproduce-and-minimize`** — `BU-0951` (`.agents/skills/diagnosing-bugs/SKILL.md (.agents/skills/diagnosing-bugs/SKILL.md L66-70)`)
   - Trigger: the feedback loop from Phase 1 has been run and gone red
   - Outcome: minimisation only begins once the loop is confirmed to be catching the right bug, reliably, with the symptom on record
   - Statement: Before minimising, Phase 2 confirms three things about the red loop: it reproduces the user's exact described failure (not a different nearby one, since the wrong bug means the wrong fix), it reproduces across multiple runs (or at a high enough rate for non-deterministic bugs), and the exact symptom is captured so later phases can verify the fix addresses it.
   - Stage-context attachments (3):
     - `BU-0952`: Minimising a reproduced bug cuts inputs, callers, config, data, and steps one at a time, re-running the loop after every cut, keeping only what is load-bearing for the failure.
     - `BU-0953`: Minimisation is done once every remaining element is load-bearing — removing any single one of them would make the loop go green.
     - `BU-0954`: Hypothesising is not started until both reproduction and minimisation are complete.

3. **`hypothesize-and-test`** — `BU-0955` (`.agents/skills/diagnosing-bugs/SKILL.md (.agents/skills/diagnosing-bugs/SKILL.md L84-84)`)
   - Trigger: beginning to hypothesise about a minimised, reproduced bug
   - Outcome: testing starts from a ranked set of candidates rather than the first idea
   - Statement: 3-5 ranked hypotheses are generated before any of them is tested, specifically to avoid anchoring on the first plausible idea.
   - Stage-context attachments (7):
     - `BU-0956`: Each hypothesis must be falsifiable — it must state the concrete prediction it makes ('if X is the cause, then changing Y will make the bug disappear / changing Z will make it worse').
     - `BU-0957`: A hypothesis that cannot be phrased as a falsifiable prediction is discarded or sharpened rather than tested as-is, since an unfalsifiable hypothesis is treated as a vibe, not a real hypothesis.
     - `BU-0958`: The ranked hypothesis list is shown to the user before testing begins, since domain knowledge can re-rank it or rule hypotheses out as already-known-false, but this checkpoint does not block: the actor proceeds with its own ranking if the user is AFK.
     - `BU-0959`: Each Phase 4 instrumentation probe is tied to a specific prediction from Phase 3, and only one variable is changed at a time.
     - `BU-0960`: The preferred instrumentation technique, in order, is a debugger/REPL breakpoint where the environment supports it, then targeted logs placed at the boundary that distinguishes hypotheses; 'log everything and grep' is never used.
     - `BU-0961`: Every debug log added during Phase 4 is tagged with a unique prefix (e.g. `[DEBUG-a4f2]`) so cleanup at the end is a single grep — an untagged log would otherwise survive cleanup by accident.
     - `BU-0962`: For a performance regression, logs are treated as usually the wrong tool; instead a baseline measurement (timing harness, profiler, or query plan) is established first and bisection is done second — measure first, fix second.

4. **`apply-fix`** — `BU-0963` (`.agents/skills/diagnosing-bugs/SKILL.md (.agents/skills/diagnosing-bugs/SKILL.md L110-110)`)
   - Trigger: the root cause is understood and a fix is about to be written
   - Outcome: a test-first fix is preferred, conditioned on a correct seam being available
   - Statement: The regression test is written before the fix, but only if a correct seam for it actually exists.
   - Stage-context attachments (3):
     - `BU-0964`: A correct seam is one that exercises the real bug pattern as it occurs at the call site; a seam that is too shallow (e.g. a single-caller test for a bug that needs multiple callers) gives false confidence rather than real coverage.
     - `BU-0965`: If no candidate seam for a regression test is correct, that absence is itself recorded as a finding — the codebase's architecture is preventing the bug from being locked down — and it is flagged for the post-mortem phase.
     - `BU-0966`: When a correct seam exists, the fix is applied in order: turn the minimised repro into a failing test at that seam, watch it fail, apply the fix, watch it pass, then re-run the original (un-minimised) Phase 1 feedback loop against the full scenario.

5. **`declare-bug-fixed`** — `BU-0967` (`.agents/skills/diagnosing-bugs/SKILL.md (.agents/skills/diagnosing-bugs/SKILL.md L126-132)`)
   - Trigger: a fix has been applied and passed its regression test
   - Outcome: the bug is not declared done until all five completion conditions hold
   - Statement: Phase 6 requires five things before declaring a bug fixed: the original repro no longer reproduces, the regression test passes (or seam absence is documented), all tagged debug instrumentation is removed, any throwaway prototypes are deleted or clearly relocated, and the hypothesis that turned out correct is recorded in the commit/PR message.
   - Stage-context attachments (1):
     - `BU-0968`: After a bug is fixed, the actor asks what would have prevented it; if the answer is architectural (no good test seam, tangled callers, hidden coupling) it hands off to the /improve-codebase-architecture skill with the specifics, and this recommendation is made only after the fix is in, not before.

### `direct-mode`

- **Trigger:** direct mode is active and an edit is about to be made
- **Outcome:** delivery is only declared complete once PR, CI, review, and merge authorization are all satisfied
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `deliver`'s.
- **Member stage count:** 4

**Ordered member stages:**

1. **`pre-edit-context-load`** — `BU-0010` (`AGENTS.md (AGENTS.md L22-36)`)
   - Trigger: direct mode is active and an edit is about to be made
   - Outcome: context and the task tracker task state are loaded before any edit
   - Statement: In direct mode, before editing, run the project context-resolution step and the task tracker for the owning task.
   - Stage-context attachments (1):
     - `BU-0011`: In direct mode, before editing, reconcile existing workers and preserved worktrees; never duplicate or race work already in progress.

2. **`implement`** — `BU-0012` (`AGENTS.md (AGENTS.md L22-36)`)
   - Trigger: direct mode is active
   - Outcome: the owning task tracker task is claimed/created and implementation proceeds test-first
   - Statement: In direct mode, claim or create the owning task tracker task, then implement TDD-first in the requested checkout or an isolated worktree.
   - Stage-context attachments (1):
     - `BU-0013`: In direct mode, the default branch is never edited; a feature branch is created or reused before the first implementation change.

3. **`validate-and-review`** — `BU-0014` (`AGENTS.md (AGENTS.md L22-36)`)
   - Trigger: direct mode implementation has reached validation/review/shipping
   - Outcome: direct-mode work passes through the same validation/review/gate steps as dispatched work
   - Statement: In direct mode, repository-native validation, independent reviews, and the final shipping gate are run exactly as a dispatched worker would run them.

4. **`deliver`** — `BU-0015` (`AGENTS.md (AGENTS.md L22-36)`)
   - Trigger: a direct-mode implementation is ready for delivery
   - Outcome: delivery is only declared complete once PR, CI, review, and merge authorization are all satisfied
   - Statement: In direct mode, a PR is opened for every implementation, and required CI, review threads, and merge authorization must be satisfied before delivery is called complete.
   - Stage-context attachments (1):
     - `BU-0016`: In direct mode, handoff, PR, merge, deployment, and cleanup outcomes are recorded.

### `dispatch-mode`

- **Trigger:** dispatch mode has been selected
- **Outcome:** a silent model substitution the account was never entitled to is durably surfaced even though the mission itself completed successfully
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `detect-model-substitution`'s.
- **Member stage count:** 19
- **Ordering note:** `dispatch-mode` is graph-shaped (independent event-triggered
  entry points, not a single pipeline) — source/behavior_id order is used as
  the defensible default per the run-wide note above, not a proven chain.

**Workflow-level helpers** (`workflow=dispatch-mode`, `stage=null` — deterministic
machinery referenced by the workflow as a whole, not one specific stage):

- `BU-0278` — When `treehouse.toml` exists in a repo, dispatch leases a pre-warmed worktree with the treehouse session manager and checks out the branch in it; when the treehouse session manager is not initialized in a repo, dispatch falls back to plain `git worktree add` at a sibling path.

**Ordered member stages:**

1. **`plan-and-decompose`** — `BU-0006` (`AGENTS.md (AGENTS.md L15-20)`)
   - Trigger: dispatch mode has been selected
   - Outcome: work is decomposed per-repository prior to dispatch
   - Statement: In dispatch mode, the coordinator loads context, plans, and decomposes the work by repository before dispatching.

2. **`dispatch-one-worker-per-repo`** — `BU-0007` (`AGENTS.md (AGENTS.md L15-20)`)
   - Trigger: dispatch mode has been selected and work has been decomposed by repository
   - Outcome: each owning repository receives one dispatched worker via the dispatch step
   - Statement: In dispatch mode, the coordinator dispatches exactly one worker per owning repository, using the dispatch step.

3. **`monitor-and-reconcile`** — `BU-0008` (`AGENTS.md (AGENTS.md L15-20)`)
   - Trigger: workers have been dispatched
   - Outcome: merge order, PR state, and cross-repo implications are reconciled by the coordinator
   - Statement: In dispatch mode, the coordinator monitors worker progress and reconciles merge order, PRs, and cross-repo implications.

4. **`validate-harness-selection`** — `BU-0057` (`AGENTS.md (AGENTS.md L186)`)
   - Trigger: a dispatch is about to create worker state
   - Outcome: only a recognized persistent-interactive agent selection is accepted; anything else is rejected before worker state is created
   - Statement: `SERGEANT_AGENT` or the dispatch step may select `opencode`, `oc`, `goose`, `claude`, or an equivalent path whose basename is one of those names; dispatch uses only persistent interactive sessions and rejects every other agent and all non-interactive launch modes before creating worker state.
   - Stage-context attachments (11):
     - `BU-0136`: Workers always run as persistent interactive TTY sessions; Sergeant never starts one-shot run, prompt, print, or automatic modes.
     - `BU-0138`: OpenCode workers are launched with `--dangerously-skip-permissions` because workers run in an automated dispatch context (no operator at the keyboard) and operator trust is scoped at dispatch time by the reviewed intent file and worker brief, which bound what the agent may do.
     - `BU-0139`: `--dangerously-skip-permissions` is not a capability grant; the actual trust boundary is the intent file content approved at dispatch time, the worker brief injected into the session, and the repository permissions of the worktree the worker checks out into.
     - `BU-0294`: When switching gh auth identity for a repo fails, dispatch records both a `failed:` status and a diagnostic to the repo's fleet directory before dying, rather than exiting with only a terminal error message.
     - `BU-0304`: Non-interactive agent launch modes are prohibited for a worker: it never launches `opencode run`, `goose run`, Claude print mode, `--auto`, `--prompt`, or an equivalent one-shot mode, since Sergeant delivers only a fixed ID-bearing notification through interactive terminal input.
     - `BU-0321`: The opencode and oc harnesses are launched with --dangerously-skip-permissions by deliberate security-posture decision: the operator's consent is scoped to the intent file and worker brief reviewed and approved at dispatch time, not to per-action interactive confirmation, and the worker cannot escalate beyond that dispatch-time-approved scope.
     - `BU-0343`: The harness capability/registry check runs before any model/variant resolution work or durable state creation for the worker, so an unregistered harness fails immediately instead of after state has already been created.
     - `BU-0360`: The Claude harness pre-flight capability gate verifies the resolved $AGENT binary itself is present, resolved through the harness path this worker was invoked with (not a hardcoded literal claude) so a fake-CLI test can substitute its own binary the same way every other harness-launch test does.
     - `BU-0879`: The interactive agent harness Sergeant launches defaults to auto-detection from the ambient environment (OpenCode markers, then Claude Code session markers, then a hardcoded opencode fallback) unless the operator explicitly overrides it with SERGEANT_AGENT.
     - `BU-0894`: Dispatch to an interactive agent dies before proceeding if the configured agent name is not one of the supported harnesses, or if the resolved agent binary is not found on PATH.
     - `BU-0895`: When the configured interactive agent is Goose, its support for interactive sessions is verified by live-probing 'goose session --help' rather than assumed; the check dies if the probe does not succeed.
   - Helper attachments (5):
     - `BU-0313`: One shared harness registry defines the accepted-harness capability gate, the readiness probe, and the launch invocation together, so the three can never drift out of sync with each other.
     - `BU-0314`: Harness registry validation rejects a row that is malformed or does not declare both a harness name and a readiness probe, naming the offending row and the expected format in the error.
     - `BU-0315`: Harness registry validation rejects a row that names a readiness probe not implemented by _sgt_harness_supported_probes, listing the supported probes in the error.
     - `BU-0316`: Harness registry validation rejects a harness name declared more than once in the registry.
     - `BU-0317`: The harness capability gate validates the entire registry before checking whether the specific requested harness is accepted, and when rejecting an unsupported harness it lists every currently accepted harness name.

5. **`resolve-and-record-model-pin`** — `BU-0058` (`AGENTS.md (AGENTS.md L187)`)
   - Trigger: a dispatch is being created
   - Outcome: the model is resolved deterministically by explicit precedence, and unpinned dispatches are explicitly recorded as such
   - Statement: The dispatch step or `SERGEANT_MODEL` pins the harness model as `provider/model[:variant]`, resolved with precedence `--model` > `SERGEANT_MODEL` > the harness's ambient default; an unpinned dispatch is recorded as `unpinned` rather than left blank, with no project-level model default.
   - Stage-context attachments (16):
     - `BU-0059`: A model tuple the selected harness cannot honor fails before any intent file, the task tracker task, worktree, or fleet state is created; a worker already handed an unhonorable tuple fails terminally instead of inheriting the ambient default.
     - `BU-0067`: Every model transport (opencode/oc argv, goose env vars, claude unmeasured) is measured on a host with that harness installed, never inferred from documentation, and Sergeant validates against it before creating any state.
     - `BU-0068`: For opencode, Sergeant writes a generated agent definition pinning both model and variant into fleet state, never the worktree, so a pinned variant cannot leave untracked files in the repository under review, and points the harness at it with `OPENCODE_CONFIG`.
     - `BU-0069`: A model/variant pin fails closed in exactly two distinct situations — 'no known transport' (harness measured, no way to pin that axis) and 'unmeasured' (harness not installed here) — and the diagnostic states which one applies.
     - `BU-0071`: `launch_state` is `intended` before the harness executes and becomes `confirmed` only once the harness reports itself ready, so a harness that rejects a pin and exits never leaves evidence claiming the model ran.
     - `BU-0072`: `variant_verified` is always `false` today because no supported harness reports back which variant it resolved, so Sergeant records the pin without claiming it was honored.
     - `BU-0074`: The model tuple is resolved from the flag, the environment, or the unpinned default, explicitly only — there is deliberately no project-level model default.
     - `BU-0345`: When the resolved harness cannot honor the pinned model tuple, the worker writes the rejection reason to a diagnostic file and to both the fleet-state and worktree status sentinels as an explicit "failed: cannot honor pinned model tuple" before exiting, rather than launching with a substituted model.
     - `BU-0346`: The launch record's launch_state field distinguishes intent from proof: it is written "intended" before the harness process is invoked and promoted to "confirmed" only once the harness is observably ready, so a harness that rejects the pin and exits never leaves evidence claiming the model actually ran; variant_verified is always recorded false because no supported harness reports back which variant it resolved.
     - `BU-0361`: Before spending a real API call, the Claude harness pre-flight rejects any model value that does not match a bare known alias (sonnet/opus/haiku/fable) or a full claude-<family>-<version> ID, as defence-in-depth against a bypass of the normal tuple-parsing path.
     - `BU-0880`: A pinned model is honored only on a launch transport Sergeant has actually measured for that harness; a harness whose model-launch transport is recorded as unmeasured fails closed rather than silently passing the pin through.
     - `BU-0881`: A pinned model tuple whose provider does not match a harness's fixed provider scope is rejected before any launch is attempted.
     - `BU-0882`: A pinned variant is honored only on a variant transport that is known and measured for the target harness; a harness whose variant transport is unknown or unmeasured fails closed on a variant pin.
     - `BU-0883`: When a harness's model transport does not carry the provider segment in its own argv (argv-bare), the launch records the provider as unverified rather than treating it as confirmed, because the invocation itself does not prove which provider was used.
     - `BU-0884`: A pinned variant carried via a generated harness agent-definition is written into Sergeant's own fleet state, never into the worktree under review, so pinning a variant cannot leave untracked files in the repository being worked on.
     - `BU-0887`: A worker dispatch that cannot honor its requested harness/model pin dies with an actionable diagnostic before any intent file, the task tracker task, worktree, or fleet state is created for it.
   - Helper attachments (5):
     - `BU-0066`: The model tuple is written `provider/model` with an optional `:variant`, using a restricted charset (`[a-z0-9-]` for providers, `[A-Za-z0-9._-]` for models); a tuple outside that charset is rejected.
     - `BU-0070`: Dispatch records the resolved model tuple in `agent_model` and its origin in `agent_model_source`; the worker records what it actually launched in `launch_record` (harness, model, provider, both transports, the generated definition, and exact argv/environment used).
     - `BU-0197`: GitHub CLI identity for dispatch resolves in strict precedence: `repo.identity` → `project.identity` → `config.default_identity` → no-op.
     - `BU-0885`: The generated harness agent-definition config file is written to a temporary file first and only then renamed into place, so a reader never observes a partially-written config; on any write failure the temporary file is removed.
     - `BU-0886`: A pinned agent tuple (provider/model[:variant]) must match a deliberately narrow character set; a tuple containing a shell metacharacter, whitespace, or anything outside that charset is rejected rather than passed through into durable launch evidence.

6. **`bind-and-verify-coordinator-pane`** — `BU-0060` (`AGENTS.md (AGENTS.md L188)`)
   - Trigger: a coordinator not inside tmux needs to bind a pane
   - Outcome: pane binding happens through exactly one of the two mutually exclusive paths, always verified live, without relaxing the persistent-interactive-worker requirement
   - Statement: The dispatch step or `--coordinator-pane <pane-id>` lets a non-tmux coordinator bind a coordinator pane; the two flags cannot be combined, the managed path never starts a tmux server, and every path verifies the pane against the live server before use, without weakening the persistent interactive worker requirement.
   - Stage-context attachments (7):
     - `BU-0075`: An API-driven coordinator not inside a tmux pane has exactly two options for binding a coordinator pane, and both still require a persistent interactive worker and still fail before any intent file, the task tracker task, worktree, or fleet state is created.
     - `BU-0076`: The `--managed-coordinator-pane` and `--coordinator-pane` options cannot be combined, and every path verifies through the live tmux server that the pane exists, is not dead, and reports back the same pane id, so an absent, stale, or forged identity is refused rather than adopted.
     - `BU-0077`: `--managed-coordinator-pane` deliberately does not start a tmux server; a coordinator must already be able to reach a live one, so a headless environment fails loudly instead of acquiring a pane nobody can observe.
     - `BU-0287`: The `--coordinator-pane` value is validated for tmux-pane-id shape before anything talks to tmux, so a malformed or argument-injecting value never reaches a tmux call.
     - `BU-0897`: Sergeant looks up the managed coordinator pane by an exact window-name match and refuses (rather than guessing which pane is the coordinator) when more than one pane exists in that window, since a substring match or a shared name must never silently become 'create another one'.
     - `BU-0899`: A pane found under the coordinator's window name is adopted as the coordinator only if it also carries Sergeant's own ownership marker (a tmux pane option stamped at creation), not merely because its window name matches.
     - `BU-0900`: A newly created coordinator pane is stamped with its ownership marker before being returned as adopted; if the marker cannot be read back confirming the stamp, the pane is killed and creation fails, rather than leaking an unmarked, unadoptable pane.

7. **`publish-canonical-intent`** — `BU-0135` (`docs/using-sergeant.md (docs/using-sergeant.md L54-58)`)
   - Trigger: a dispatch is created
   - Outcome: a single canonical intent revision is durably recorded and shared identically across fleet state and every worktree
   - Statement: Sergeant writes the same `.sergeant-intent.md` revision to fleet state and every selected worktree at dispatch time; this artifact is canonical for implementation decisions, reviews, PR text, successor/recovery work, and final validation.
   - Stage-context attachments (1):
     - `BU-0303`: `.sergeant-intent.md`'s revision is never silently regenerated by the worker; only an audited human decision creates a new intent revision, and successor/recovery work must inherit the exact existing revision.
   - Helper attachments (1):
     - `BU-0323`: The intent revision is computed as a SHA-256 digest of the intent file's contents, using shasum or sha256sum, and fails with an actionable error if neither tool is available.

8. **`validate-intent-file`** — `BU-0140` (`docs/using-sergeant.md (docs/using-sergeant.md L112-117)`)
   - Trigger: a dispatch objective touches any of the named sensitive categories
   - Outcome: dispatch requires a validated intent file before any mutating dispatch action, and validation failures block before mutation
   - Statement: `--intent-file` is required whenever the objective names auth/OAuth, security, secrets or credentials, payments, databases or migrations, stateful/production work, destructive work, persistent state, or state transitions; the file must contain the eight required sections, and malformed, missing, traversing, symlinked, or oversized input fails before dispatch mutation.
   - Stage-context attachments (6):
     - `BU-0327`: An intent file path is rejected as unsafe if any path component along its resolved chain — not just the final component — is itself a symlink.
     - `BU-0328`: An intent document is only valid if it contains exactly the eight required sections (Objective, Required Invariants, Approved Tradeoffs, Out Of Scope, State Transitions, Failure Windows, Negative Test Matrix, Validation Evidence), each appearing exactly once, in that exact order, and none left empty.
     - `BU-0329`: An operator-supplied intent file is rejected if its path contains a newline or carriage return, or if it attempts path traversal (".." as a whole component or embedded), or if it does not exist.
     - `BU-0330`: An operator-supplied intent file is rejected if it exceeds 65536 bytes, or if it contains control-character bytes other than tab and newline.
     - `BU-0331`: A dispatch objective is rejected — requiring an explicit --intent-file instead — when it matches safety-sensitive or stateful keywords (auth, oauth, security, secrets, credentials, payments, databases, migrations, stateful, production, destructive, or persistent/state-transition phrasing).
     - `BU-0332`: The synthesized standard-isolated intent (used when no --intent-file is supplied and the objective is not safety-sensitive) explicitly authorizes no persistent or externally published state transition, and requires stopping on any native validation, review, or dispatch failure without publishing partial work.
   - Helper attachments (1):
     - `BU-0333`: Installing a prepared intent file into its target path is done by copying to a temp file in the target directory and then renaming it into place, never by writing the target path directly.

9. **`prepare-worker-brief`** — `BU-0273` (`skills/dispatch/SKILL.md (skills/dispatch/SKILL.md L36)`)
   - Trigger: the dispatch step is invoked with --td
   - Outcome: the worker's brief carries the full task tracker lifecycle instructions rather than a freeform mission with no task tracking
   - Statement: When dispatch is done from an existing task tracker task, the brief, branch name, and full task context are pulled from the task tracker automatically, and the worker's brief includes the task tracker's start, log, handoff, and review instructions so the task lifecycle is tracked end-to-end.
   - Stage-context attachments (1):
     - `BU-0279`: A `--deps` ordering constraint causes the brief written into each dependent repo to include an instruction to wait for the prerequisite's `.sergeant-status` to read `done` before opening a PR; the workers themselves are responsible for honoring this, and the brief makes it explicit.

10. **`reconcile-dispatch-results`** — `BU-0276` (`skills/dispatch/SKILL.md (skills/dispatch/SKILL.md L112)`)
   - Trigger: a worker has opened a PR and other completion evidence looks satisfied
   - Outcome: dependency-gate satisfaction is a separate, required condition for done, not implied by other evidence
   - Statement: A worker is not considered done until its dependency gate is satisfied, even if merge order among dependent repos would otherwise suggest completion.
   - Stage-context attachments (1):
     - `BU-0277`: A fleet is never reconciled or cleaned up merely because every worker has opened a PR; all completion gates must be met.

11. **`create-tasks-before-spawn`** — `BU-0284` (`skills/dispatch/SKILL.md (skills/dispatch/SKILL.md L198)`)
   - Trigger: dispatch is invoked from a freeform brief across multiple repos
   - Outcome: task creation is all-or-nothing across the selected repos, with rollback on partial failure, before any worker is spawned
   - Statement: When dispatching from a freeform brief, the dispatch step creates exactly one task tracker task in each target repo before spawning any worker; if the task tracker is unavailable, task creation fails, generated metadata cannot be injected, or any selected repo does not get a generated task, dispatch aborts before spawning and rolls back the generated cards.
   - Stage-context attachments (4):
     - `BU-0285`: `--td` dispatch keeps using the existing task instead of generating replacement task tracker tasks.
     - `BU-0290`: Generated task tracker task results are strictly validated before use — every selected repo must have exactly one task, no repo or task id may repeat, and any malformed or unexpected result triggers a full rollback (deleting every task tracker task already created) with a failure message naming the exact validation error.
     - `BU-0297`: Every target repo is validated (cloned, the task tracker initialized) before any task is created, specifically so that a prerequisite failure never requires rolling back an already-created task.
     - `BU-0298`: If task creation fails, or the created task's JSON result cannot be parsed into a valid task id, for any target repo, every task tracker task already created in this run is rolled back (deleted) before the command exits with an error.

12. **`rollback-coordinator-pane-on-abort`** — `BU-0288` (`bin/sgt-dispatch (bin/sgt-dispatch L324-335)`)
   - Trigger: dispatch creates a new managed coordinator window and a later step aborts
   - Outcome: cleanup is scoped precisely to what this invocation created, and covers every later abort point from the moment the pane is bound
   - Statement: The managed-coordinator-pane rollback trap removes only the exact window this invocation created, never a window Sergeant merely selected/adopted, and is installed as an EXIT trap immediately after pane binding since several later preflight steps can still abort.
   - Stage-context attachments (1):
     - `BU-0296`: Once every target repo has been successfully dispatched, the managed-coordinator-pane rollback is disarmed, since the pane is now owned by live fleet state rather than by this invocation's own error-cleanup scope.

13. **`check-drain-admission`** — `BU-0289` (`bin/sgt-dispatch (bin/sgt-dispatch L474-486)`)
   - Trigger: dispatch is about to create its first durable side effect
   - Outcome: a race between a concurrent drain and this dispatch's admission is closed by holding the lock across the critical window, and any ambiguous drain record blocks rather than admits
   - Statement: The drain admission lock is acquired and held through the task tracker task creation (the first side effect), so a concurrent drain either waits until admission is committed or wins the lock first and blocks this dispatch; malformed, empty, or expired drain records fail closed and block dispatch.

14. **`acquire-worktree`** — `BU-0291` (`bin/sgt-dispatch (bin/sgt-dispatch L775-793)`)
   - Trigger: the target branch already exists and may carry prior uncommitted-upstream work
   - Outcome: prior committed work is never silently overwritten or orphaned by a fresh dispatch; resuming it requires an explicit --adopt-branch
   - Statement: Re-dispatching onto an existing branch is blocked when that branch carries committed work not reachable from any remote, unless `--adopt-branch` is passed; the reachability test checks every remote, not just origin, so re-dispatch to a repository the operator cannot push to is not incorrectly blocked.
   - Stage-context attachments (1):
     - `BU-0292`: After acquiring a worktree, dispatch requires the worktree's `.git` to be a real (non-symlinked) file containing a well-formed `gitdir:` pointer, and resolves and records both the pointer and the canonicalized git directory, failing closed if either check does not hold.
   - Helper attachments (3):
     - `BU-0293`: Dispatch records the worktree's HEAD SHA at dispatch time (`initial_sha`) so workers and the reconcile path can later detect committed work added above that base.
     - `BU-0925`: A branch's unpushed commits are determined by reachability from any configured remote-tracking ref, not by matching one specific remote branch name, so a commit that is published under a differently-named remote branch is correctly treated as already published.
     - `BU-0926`: Unpushed-commit detection first confirms the local branch itself exists; for an absent branch it reports no unpushed commits rather than letting git raise a fatal error for a nonexistent ref.

15. **`handle-spawn-failure`** — `BU-0295` (`bin/sgt-dispatch (bin/sgt-dispatch L916-969)`)
   - Trigger: any of the four named worker-spawn failure modes occurs
   - Outcome: every spawn failure path converges on the same explicit orphaning + evidence-recording sequence, never a silent or ambiguous half-started worker
   - Statement: If any step of spawning a worker fails — no pane returned, a notification-target creation race, failure to capture exact pane identity, or the worker not acknowledging its durable notification in time — dispatch stops any Claude background session the worker may have started, kills the pane, marks the repo `orphaned` with a named diagnostic, and hands off to the task-tracker memory step before dying, rather than leaving an ambiguous or silently-failed worker.

16. **`probe-harness-readiness`** — `BU-0318` (`bin/_sgt-harness.sh (bin/_sgt-harness.sh L204-211)`)
   - Trigger: a worker polls whether its pane can receive a nudge
   - Outcome: a dead or still-blank pane is never reported ready
   - Statement: The tui readiness probe requires the target tmux pane to be alive and to have rendered at least one non-whitespace glyph before the pane can be considered ready to receive input.
   - Stage-context attachments (5):
     - `BU-0319`: The tui readiness probe never reports a pane ready on the very first observation of drawn output; a later, second observation is required, because a TUI's first painted frame may still be installing its input handlers.
     - `BU-0320`: The tui readiness probe honors a configurable wall-clock settle time (SGT_HARNESS_SETTLE_SECONDS) as an additional minimum on top of the two-consecutive-observation rule, defaulting to none.
     - `BU-0322`: Dispatching a readiness check to a probe identifier that is declared but not implemented fails loudly, naming the harness and the unimplemented probe, rather than silently treating the harness as never ready.
     - `BU-0352`: When the bounded readiness gate's timeout elapses, the worker publishes a durable readiness_failed record (once) naming the notification, nonce, harness, and seconds waited.
     - `BU-0353`: The readiness-timeout failure message explicitly states that no acknowledgement, acceptance, delivery, or action lease was fabricated and that the notification is still pending, directing the operator to confirm the harness renders a prompt and resume via the worker response-delivery step.

17. **`capture-background-session-identity`** — `BU-0362` (`bin/sgt-interactive-worker (bin/sgt-interactive-worker L865-878)`)
   - Trigger: claude --bg returns a background ID
   - Outcome: a malformed background ID can never be persisted into fleet state where every termination backstop assumes the well-formed shape
   - Statement: The Claude background session ID returned by claude --bg is validated at capture time against the exact same charset regex every one of the nine termination-path backstops uses to resolve it; a malformed ID fails the launch immediately rather than being persisted and silently defeating every one of those backstops later.
   - Stage-context attachments (2):
     - `BU-0363`: The Claude background ID is persisted to fleet state immediately after capture, before the (potentially multi-second) post-launch liveness check loop runs, so a worker process death during that window cannot leave a genuinely live background session with no recorded identity anywhere.
     - `BU-0364`: During the bounded post-launch liveness poll, if the Claude background session's reported state reaches "failed", the launch is treated as a terminal failure (status/sentinel written, process exits) rather than continuing to poll or attaching anyway.

18. **`reattach-after-attach-exit`** — `BU-0365` (`bin/sgt-interactive-worker (bin/sgt-interactive-worker L1056-1067)`)
   - Trigger: the blocking claude attach call exits
   - Outcome: a legitimate cooperative gate (needs_input/blocked/waiting) is never mistaken for an unexpected death and spuriously re-attached
   - Statement: Respawn/reattach after an attach exit is only ever considered while .sergeant-status is still the unchanged in_progress it started at; any other value (done, failed:*, drained, orphaned, or a cooperative needs_input/blocked/waiting gate the agent itself published) is treated as the agent or an existing mechanism having already decided the outcome, so attach exiting afterward is expected, not an unexpected death.
   - Stage-context attachments (1):
     - `BU-0366`: When attach exits while status is still in_progress, the worker distinguishes a genuinely dead background session (state=stopped, requiring respawn before re-attach to restore the same session/conversation) from a session that never died (state=working/blocked, requiring only a direct re-attach with no respawn).

19. **`detect-model-substitution`** — `BU-0367` (`bin/sgt-interactive-worker (bin/sgt-interactive-worker L1109-1126)`)
   - Trigger: the claude attach invocation has exited and a model was pinned for this session
   - Outcome: a silent model substitution the account was never entitled to is durably surfaced even though the mission itself completed successfully
   - Statement: After the main Claude invocation exits, if a model was pinned, the worker scans the session transcript for the substitution warning line ("is restricted by your organization's settings... Using <Y> instead") and, if found, records a durable model_substitution_warning in fleet state and prints a WARNING — since neither the pre-flight regex nor the liveness check can catch a valid-but-unentitled model alias that Claude silently substitutes without any failed state or nonzero exit code.
   - Stage-context attachments (1):
     - `BU-0368`: The model-substitution warning is written to its own dedicated file rather than $REPO_STATE/diagnostic, because _finish unconditionally clears the shared diagnostic file on every non-orphaned exit (including this successful-mission path), which would otherwise erase the warning before anything could read it.

### `domain-modeling`

- **Trigger:** the user uses a term that conflicts with CONTEXT.md's existing definition
- **Outcome:** an ADR is offered only when the three-part test passes; otherwise it is not created
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `offer-adr`'s.
- **Member stage count:** 2

**Workflow-level helpers** (`workflow=domain-modeling`, `stage=null` — deterministic
machinery referenced by the workflow as a whole, not one specific stage):

- `BU-1057` — If no CONTEXT.md exists yet, create one lazily, at the point the first domain term is resolved — not in advance.
- `BU-1058` — If no docs/adr/ directory exists yet, create it lazily, at the point the first ADR is needed — not in advance.
- `BU-1076` — Which context-file structure applies is inferred rather than assumed: if CONTEXT-MAP.md exists it is read to find the contexts; if only a root CONTEXT.md exists the repo is treated as single-context; if neither exists, a root CONTEXT.md is created lazily when the first term is resolved.

**Ordered member stages:**

1. **`maintain-glossary-discipline`** — `BU-1059` (`.agents/skills/domain-modeling/SKILL.md (.agents/skills/domain-modeling/SKILL.md L44-46)`)
   - Trigger: the user uses a term that conflicts with CONTEXT.md's existing definition
   - Outcome: the conflict is surfaced to the user immediately instead of silently accepted
   - Statement: When the user uses a term that conflicts with the existing language already recorded in CONTEXT.md, that conflict is called out immediately rather than let pass.
   - Stage-context attachments (9):
     - `BU-1060`: When the user uses a vague or overloaded term, a precise canonical term is proposed to replace it.
     - `BU-1061`: When domain relationships are being discussed, they are stress-tested with invented edge-case scenarios that force precision about the boundaries between concepts.
     - `BU-1062`: When the user states how something works, the code is checked against that statement, and any contradiction found is surfaced to the user rather than left unresolved.
     - `BU-1063`: When a domain term is resolved, CONTEXT.md is updated inline at that moment rather than the update being batched up for later.
     - `BU-1072`: When multiple words exist for the same domain concept, one is opinionatedly picked as canonical in CONTEXT.md and the others are listed under that term's _Avoid_ line.
     - `BU-1073`: CONTEXT.md term definitions are kept to one or two sentences at most, and define what the term IS rather than what it does.
     - `BU-1074`: Only terms specific to the project's own context are included in CONTEXT.md; general programming concepts (timeouts, error types, utility patterns) are excluded even if heavily used, judged by whether the concept is unique to this context or general-purpose.
     - `BU-1075`: CONTEXT.md terms are grouped under subheadings when natural clusters emerge, and left as a flat list when all terms belong to a single cohesive area.
     - `BU-1077`: When multiple contexts exist, which context the current topic relates to is inferred; if that is unclear, the user is asked rather than guessed.

2. **`offer-adr`** — `BU-1065` (`.agents/skills/domain-modeling/SKILL.md (.agents/skills/domain-modeling/SKILL.md L68-74)`)
   - Trigger: a decision has just been made during the session
   - Outcome: an ADR is offered only when the three-part test passes; otherwise it is not created
   - Statement: An ADR is only offered when all three of hard-to-reverse, surprising-without-context, and result-of-a-real-trade-off are true; if any one of the three is missing, the ADR is skipped.
   - Stage-context attachments (3):
     - `BU-1068`: An ADR's required content is minimal: a short title plus 1-3 sentences covering the context, the decision, and the reason — the value is in recording that a decision was made and why, not in filling out sections.
     - `BU-1069`: ADR optional sections (Status frontmatter, Considered Options, Consequences) are included only when they add genuine value — most ADRs need none of them.
     - `BU-1071`: ADR-worthy decisions fall into specific named categories: architectural shape, integration patterns between contexts, lock-in technology choices, boundary/scope decisions (including explicit "no" boundaries), deliberate deviations from the obvious path, constraints not visible in the code, and non-obvious rejected alternatives.
   - Helper attachments (3):
     - `BU-1066`: ADRs live in docs/adr/ and are named with sequential zero-padded numbering (0001-slug.md, 0002-slug.md, and so on).
     - `BU-1067`: The docs/adr/ directory is created lazily, only when the first ADR is needed.
     - `BU-1070`: A new ADR's number is chosen by scanning docs/adr/ for the highest existing number and incrementing by one.

### `fleet-status-listing`

- **Trigger:** (no stage candidates for this workflow value)
- **Outcome:** (no stage candidates for this workflow value)
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `(none)`'s.
- **Member stage count:** 0

**Workflow-level helpers** (`workflow=fleet-status-listing`, `stage=null` — deterministic
machinery referenced by the workflow as a whole, not one specific stage):

- `BU-0576` — The interactive fleet-watch loop --list explicitly avoids claiming its retained task records (including terminal ones) are currently active, directing a caller who needs an activity determination to --snapshot instead.
- `BU-0606` — The interactive fleet-watch loop --list reports, per task, a full breakdown of repo counts across every recognized status (done, in-progress, needs-input, blocked, waiting, drained, orphaned, failed), not merely a single aggregate count.

_No `stage`-rung records carry this `workflow` value — candidate has no_
_member stages (a `stage`/`stage-context`/`helper`-only cluster whose_
_checkpoint boundary, if any, was never classified as `stage` rung)._

### `graphify`

- **Trigger:** the graph-generation step is run for a project
- **Outcome:** a failure leaves the previous graph output intact and cleans up its own temporary artifacts rather than leaving a half-swapped or missing output
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `recover-from-failed-publish`'s.
- **Member stage count:** 2

**Ordered member stages:**

1. **`run-graph-generation`** — `BU-0133` (`docs/getting-started.md (docs/getting-started.md L161)`)
   - Trigger: the graph-generation step is run for a project
   - Outcome: the run is only considered successful once both named output artifacts exist
   - Statement: A project graph run requires both `graph.json` and `GRAPH_REPORT.md` to exist at the configured project output.
   - Stage-context attachments (12):
     - `BU-0184`: Graphify output for a project is kept to one output path outside source repositories, and an existing graph is never regenerated or moved without confirming the desired global-per-project path.
     - `BU-0198`: If `graphify.output` is a directory symlink, the graph-generation step preserves the symlink and publishes into its target; Sergeant only replaces the published graph after a complete run and preserves existing `wiki/` and `memory/` directories.
     - `BU-0199`: If `graphify.output` lives inside a source repo, Sergeant stages extraction outside that repo and excludes the configured output path, so published graph artifacts are never re-ingested as source.
     - `BU-0245`: Publication of the merged project graph is atomic: concurrent readers observe either the complete old output or the complete new output throughout the run, never a partial or missing state, and the publish directory is placed so the final rename never crosses filesystem boundaries.
     - `BU-0248`: Whenever exclusion patterns apply to a repo (because the output path or an excluded pattern falls inside it), the graph-generation step stages a filtered copy outside the repo before running the graph-generation tool extraction, so the excluded/output paths are never re-ingested as source.
     - `BU-0250`: If zero repositories matched `include_groups`, or any included repository's extraction failed, graph-generation step exits with an error listing the failed repositories rather than publishing a graph built from a partial repo set.
     - `BU-0251`: Before publication, the graph-generation step verifies that `graph.json`, `manifest.json`, and `GRAPH_REPORT.md` all exist and are nonempty in the staged output; if any is missing or empty, the run errors out and does not publish.
     - `BU-0252`: Publication preserves existing `wiki/` and `memory/` directories under the graph-generation tool output by copying them into the newly staged output before the atomic swap, so the graph-generation tool run never destroys user-facing extensions it did not regenerate.
     - `BU-0254`: When the configured output is a symlink outside all source repos, publication atomically replaces the symlink itself via a single rename (`mv -T`) pointing at a newly staged backing directory, rather than writing through the symlink into its old target in place.
     - `BU-0262`: If no `graphify.output` is configured for a project, the procedure stops and requests or adds the project-level path before running Graphify, rather than inventing a default output location.
     - `BU-0263`: After a Graphify run, `<graphify.output>/graph.json` and `GRAPH_REPORT.md` are required to exist before the run is treated as successful.
     - `BU-0264`: Generated graph output is never published inside an owning source repository.
   - Helper attachments (4):
     - `BU-0196`: A repo's `name` must match `[A-Za-z0-9._-]+`, cannot contain spaces, and cannot be `.` or `..`, so Sergeant can safely prefix merged source paths with it for the graph-generation step.
     - `BU-0246`: A repo name that does not match `[A-Za-z0-9._-]+`, or is `.`/`..`, is rejected for the graph-generation tool with a named error rather than being used to build a merged output path.
     - `BU-0247`: graphify.output is never allowed to be identical to a source repository path; extraction refuses with a named error rather than extracting a repo into itself.
     - `BU-0249`: When no supported LLM API key is set in the environment, extraction passes `--code-only` so the graph-generation tool indexes code via local AST without attempting semantic extraction, avoiding an abort when it encounters doc/paper/image files with no key configured.

2. **`recover-from-failed-publish`** — `BU-0253` (`bin/sgt-graphify (bin/sgt-graphify L207-238)`)
   - Trigger: the graph-generation step exits (successfully or on failure) at any point after it may have started moving the old output aside
   - Outcome: a failure leaves the previous graph output intact and cleans up its own temporary artifacts rather than leaving a half-swapped or missing output
   - Statement: On any failure before publication completes, the exit trap restores the previously moved-aside old output, removes leftover temp symlinks and staging/backing directories, and only removes the old backup once the new output has actually been published.

### `grilling`

Established by `BU-0970` (`.agents/skills/grilling/SKILL.md (.agents/skills/grilling/SKILL.md L6-6)`): A grilling interview questions the user relentlessly about every aspect of the plan/decision/idea, walking each branch of the decision tree one by one and resolving dependencies between decisions, with a recommended answer offered for each question.

- **Trigger:** a grilling interview is in progress
- **Outcome:** the user is never presented with more than one open question at once
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `conduct-interview`'s.
- **Member stage count:** 1

**Ordered member stages:**

1. **`conduct-interview`** — `BU-0971` (`.agents/skills/grilling/SKILL.md (.agents/skills/grilling/SKILL.md L8-8)`)
   - Trigger: a grilling interview is in progress
   - Outcome: the user is never presented with more than one open question at once
   - Statement: Questions during a grilling interview are asked one at a time, waiting for the user's feedback on each before continuing, because asking several at once is bewildering.
   - Stage-context attachments (2):
     - `BU-0972`: A question resolvable by exploring the environment (filesystem, tools, etc.) is looked up directly instead of being asked; only genuine decisions are put to the user, and the actor waits for the user's answer on those.
     - `BU-0973`: The actor does not act on a grilling interview's conclusions until the user confirms a shared understanding has been reached.

### `implement`

- **Trigger:** implementation work has pre-agreed seams
- **Outcome:** the work is durably recorded in version control
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `commit-implementation`'s.
- **Member stage count:** 3

**Ordered member stages:**

1. **`implement-at-seams`** — `BU-0974` (`.agents/skills/implement/SKILL.md (.agents/skills/implement/SKILL.md L9-9)`)
   - Trigger: implementation work has pre-agreed seams
   - Outcome: implementation at those seams follows the TDD discipline rather than an ad hoc approach
   - Statement: /tdd is used where possible, at pre-agreed seams, when implementing work from a spec or tickets.
   - Stage-context attachments (1):
     - `BU-0975`: Typechecking and single test files are run regularly during implementation, and the full test suite is run once at the end.

2. **`review-implementation`** — `BU-0976` (`.agents/skills/implement/SKILL.md (.agents/skills/implement/SKILL.md L13-13)`)
   - Trigger: implementation work is complete
   - Outcome: the finished work receives a code review before being considered done
   - Statement: /code-review is used to review the completed implementation work.

3. **`commit-implementation`** — `BU-0977` (`.agents/skills/implement/SKILL.md (.agents/skills/implement/SKILL.md L15-15)`)
   - Trigger: the work has been reviewed
   - Outcome: the work is durably recorded in version control
   - Statement: Completed, reviewed implementation work is committed to the current branch.

### `install-sergeant`

- **Trigger:** the dependency check is being run during installation
- **Outcome:** the pull only succeeds when it is a clean fast-forward, never creating a merge commit or silently resolving divergence
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `update-checkout`'s.
- **Member stage count:** 4

**Ordered member stages:**

1. **`verify-prerequisites`** — `BU-0129` (`docs/getting-started.md (docs/getting-started.md L51-53)`)
   - Trigger: the dependency check is being run during installation
   - Outcome: installation does not proceed until both the td-implementation check and the agent-availability check pass
   - Statement: Setup continues only once `td create --help` shows Marcus `td` support for `--description`, `--json`, and `--work-dir`, and at least one supported agent resolves on PATH.
   - Stage-context attachments (7):
     - `BU-0132`: Sergeant requires the Marcus `td` implementation with JSON, task creation, and `--work-dir` support; a different executable named `td` is rejected.
     - `BU-0173`: If another executable named `td` is first on PATH, PATH is corrected rather than wrapping unsupported output indefinitely; `td create --help` must show `--description`, `--json`, and `--work-dir`.
     - `BU-0209`: The dependency check accepts `td` as present only when its version output is a supported version AND `td create --help` shows all three of `--description`, `--json`, and `--work-dir`; missing or unsupported td fails the check.
     - `BU-0210`: The dependency check accepts the first of `opencode`, `goose`, or `claude` found on PATH as the agent harness and reports success; if none is found, it reports the check as failed.
     - `BU-0211`: The dependency check exits nonzero and instructs the user to install missing required dependencies when any required check failed; it does not proceed with Sergeant-using tasks on that state.
     - `BU-0915`: The output of 'td --version' is accepted only if it is exactly Marcus td's plain single-line version string or its exact three-line update-available notice with internally consistent version numbers; any other or mixed output is rejected before dispatch creates any side effects.
     - `BU-0916`: Before any td-backed dispatch, Sergeant verifies both that the installed td's version output is recognized and that its 'td create --help' output advertises the required --description, --json, and --work-dir flags; failing either check dies naming the unsupported binary's path and version.

2. **`install-symlinks`** — `BU-0204` (`mise.toml (mise.toml L20-23)`)
   - Trigger: the install task runs
   - Outcome: every current and future matching script is linked, without needing the installer to be updated per new script
   - Statement: Installation links every `sgt-*`, `_sgt-*.sh`, and the wiki-digest job script by globbing the bin directory rather than enumerating filenames by name, because enumerating by name silently broke the install whenever a new `bin/_sgt-*.sh` helper was added.
   - Stage-context attachments (2):
     - `BU-0205`: Install removes legacy `oc-inject` links (a deleted feature) from `~/.local/bin` and `~/.config/opencode/plugins/`, but only when the target is a symlink (`-L` check), never an ordinary file.
     - `BU-0206`: Install links git hooks from `scripts/hooks/` into `.git/hooks/`, but only when both the hooks source and destination directories exist.

3. **`uninstall-symlinks`** — `BU-0207` (`mise.toml (mise.toml L93-104)`)
   - Trigger: uninstall:hooks runs
   - Outcome: only hooks this repository actually installed are removed; foreign or already-diverged hooks are preserved
   - Statement: Uninstalling git hooks removes a hook symlink only when it is a symlink whose target still matches this repository's own `scripts/hooks/<name>` file, leaving any other hook (or a hook that no longer points here) untouched.
   - Stage-context attachments (1):
     - `BU-0208`: Uninstalling command symlinks removes a `~/.local/bin` entry only when it is a symlink and its resolved target path contains `/sergeant/bin/`, so a same-named file that is not actually a link back into this repository is never removed.

4. **`update-checkout`** — `BU-0213` (`mise.toml (mise.toml L292)`)
   - Trigger: the toolchain/task runner run update is invoked
   - Outcome: the pull only succeeds when it is a clean fast-forward, never creating a merge commit or silently resolving divergence
   - Statement: Updating pulls the latest changes with `git pull --ff-only` rather than an ordinary pull, then reinstalls symlinks.

### `list-projects`

- **Trigger:** (no stage candidates for this workflow value)
- **Outcome:** (no stage candidates for this workflow value)
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `(none)`'s.
- **Member stage count:** 0

**Workflow-level helpers** (`workflow=list-projects`, `stage=null` — deterministic
machinery referenced by the workflow as a whole, not one specific stage):

- `BU-0235` — Listing projects enumerates YAML files directly under the Sergeant config directory and skips `config.yaml` specifically, since that file is global config, not a project.
- `BU-0236` — When no project YAMLs are found, the fleet-listing step reports the empty state and exits nonzero with guidance to create a project YAML.

_No `stage`-rung records carry this `workflow` value — candidate has no_
_member stages (a `stage`/`stage-context`/`helper`-only cluster whose_
_checkpoint boundary, if any, was never classified as `stage` rung)._

### `list-tasks`

- **Trigger:** (no stage candidates for this workflow value)
- **Outcome:** (no stage candidates for this workflow value)
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `(none)`'s.
- **Member stage count:** 0

**Workflow-level helpers** (`workflow=list-tasks`, `stage=null` — deterministic
machinery referenced by the workflow as a whole, not one specific stage):

- `BU-0243` — By default the task-tracker listing step filters to `status=open`; `--all` removes the status filter, and `--priority` ANDs an additional priority filter onto whatever status filter is active.
- `BU-0244` — The task-tracker listing step silently skips a target repo whose resolved path is not an initialized git repository, rather than erroring the whole listing.

_No `stage`-rung records carry this `workflow` value — candidate has no_
_member stages (a `stage`/`stage-context`/`helper`-only cluster whose_
_checkpoint boundary, if any, was never classified as `stage` rung)._

### `load-project`

- **Trigger:** the project name for a task is not already known exactly
- **Outcome:** the edit is validated against resolved context output, not just YAML syntax validity
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `edit-and-validate-project`'s.
- **Member stage count:** 3

**Ordered member stages:**

1. **`resolve-project-name`** — `BU-0255` (`skills/load-project/SKILL.md (skills/load-project/SKILL.md L17-19)`)
   - Trigger: the project name for a task is not already known exactly
   - Outcome: an exact registered name is confirmed before context loading proceeds
   - Statement: If a project name is unknown, the fleet-listing step is run and an exact registered name is required before proceeding, rather than guessing or fuzzy-matching a project.

2. **`load-repo-context`** — `BU-0257` (`skills/load-project/SKILL.md (skills/load-project/SKILL.md L28-29)`)
   - Trigger: a required repository is not yet cloned
   - Outcome: sync is deferred until actually needed, and a sync failure halts rather than proceeding with a missing repo
   - Statement: A missing required repository is synced only after the requested work is confirmed to require that repository, and the procedure stops if cloning or pulling fails.
   - Stage-context attachments (2):
     - `BU-0256`: A raw project YAML is read directly only when a required field is absent from the project context-resolution step output, not as a routine alternative to it.
     - `BU-0258`: Completion evidence for loading project context is the project context-resolution step block showing every owning repository as cloned, plus the instructions and paths that will govern execution.

3. **`edit-and-validate-project`** — `BU-0260` (`skills/load-project/SKILL.md (skills/load-project/SKILL.md L45-46)`)
   - Trigger: a project YAML has been edited
   - Outcome: the edit is validated against resolved context output, not just YAML syntax validity
   - Statement: After editing a project, context-resolution step is run and every edited field needed by agents is required to appear in the resolved output before the edit is considered validated.
   - Stage-context attachments (1):
     - `BU-0261`: If project registration/edit validation fails, the prior YAML is restored or the new file is left uncommitted, and the exact command error is reported.

### `no-mistakes-finding-routing`

- **Trigger:** the validation pipeline surfaces an actionable finding
- **Outcome:** remediation converges to one worker per root cause, is rechecked before merge, and escalates to a human after two unsuccessful cycles rather than looping indefinitely
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `remediate-grouped-findings`'s.
- **Member stage count:** 2

**Ordered member stages:**

1. **`route-finding`** — `BU-0089` (`README.md (README.md L302-304)`)
   - Trigger: the validation pipeline surfaces an actionable finding
   - Outcome: the finding becomes owning-repo task tracker work rather than being fixed inline by the run
   - Statement: The validation pipeline run is validation-only and must not fix findings; actionable findings are routed into separate, deduplicated owning-repo task tracker tasks with the no-mistakes-finding disposition step.
   - Stage-context attachments (3):
     - `BU-0704`: If more than one existing task tracker task matches a finding's deduplication key, the no-mistakes-finding disposition step fails rather than picking one to update.
     - `BU-0709`: A finding disposed gate causes the no-mistakes-finding disposition step to exit with status 2 after recording it in the task tracker, signaling to the caller that this finding remains blocking regardless of the task tracker side effect having succeeded.
     - `BU-0710`: A finding disposed ask-user causes the no-mistakes-finding disposition step to exit with status 2 after recording it in the task tracker, signaling that the finding requires human escalation.
   - Helper attachments (20):
     - `BU-0090`: The required `--disposition` per finding is explicit: `gate` creates/updates P1 work and retains the gate, `ask-user` creates/updates P1 work and preserves human escalation, the task tracker creates/updates nonblocking actionable debt, and `ignore` records that no card is needed.
     - `BU-0091`: Warning debt becomes P2, informational debt becomes P3, and repeated finding IDs update the same card while retaining the latest run ID, head SHA, location, description, and originating intent.
     - `BU-0092`: Reruns preserve any existing repo-specific or manually added task tracker labels while ensuring the required validation pipeline and `finding` labels remain present without duplication.
     - `BU-0093`: On rerun, visible active cards stay in their current state, while explicitly hidden states are resurfaced: closed cards are reopened and deferred cards are undeferred before the finding body is refreshed.
     - `BU-0094`: Correctness, security, data-integrity, and test findings cannot be deferred or ignored; cosmetic and evidence-only findings never create cards.
     - `BU-0693`: The no-mistakes-finding disposition step requires every finding field (run id, head SHA, finding id, severity, kind, description, intent, disposition) to be present, and fails naming the specific missing field.
     - `BU-0694`: A finding's disposition must be exactly one of gate, the task tracker, ignore, or ask-user; any other value is rejected.
     - `BU-0695`: A finding whose kind is correctness, security, data-integrity, or test may only be disposed as gate or ask-user; disposing such a finding to the task tracker or ignore is rejected outright.
     - `BU-0696`: A finding whose kind is cosmetic or evidence never creates the task tracker card, regardless of the requested disposition, and the command succeeds reporting that no card was created.
     - `BU-0697`: Only cosmetic or evidence findings may ever be disposed as ignore; for any other kind, requesting ignore is rejected — actionable findings must gate, ask the user, or route to the task tracker.
     - `BU-0698`: A finding's task tracker priority is derived from its disposition and severity: gate or ask-user is always priority P1; a td-routed finding is P2 for warning severity or P3 for info/informational severity; any other severity routed to the task tracker is rejected.
     - `BU-0699`: The no-mistakes-finding disposition step requires the named project's config file to exist and the yq, git, and the task tracker tools to be available before doing anything else.
     - `BU-0700`: The target repository's filesystem path is resolved by looking up its name in the named project's configuration; an unrecognized repo name is rejected.
     - `BU-0701`: The no-mistakes-finding disposition step refuses to act on a repo path that is not actually a cloned git repository.
     - `BU-0702`: Each finding is given a stable deduplication key derived from the repo and finding id, embedded as a 'Deduplication key:' line in the task tracker card body, so the same finding can be recognized again on a later run.
     - `BU-0703`: A finding lookup treats the task tracker list JSON result of null as 'no existing card' rather than an error, so the router proceeds to create a new task instead of failing.
     - `BU-0705`: If a matching finding's task tracker card was closed, the no-mistakes-finding disposition step reopens it before updating it.
     - `BU-0706`: If a matching finding's task tracker card had a deferral set, the no-mistakes-finding disposition step clears the deferral when the finding recurs, rather than leaving it silently deferred.
     - `BU-0707`: An existing matching task tracker card has its priority, description (the full current finding body), and labels rewritten to reflect the current run's finding data.
     - `BU-0708`: If no existing matching task tracker card is found, the no-mistakes-finding disposition step creates a new task tracker task carrying the full finding body, computed priority, and standard labels.

2. **`remediate-grouped-findings`** — `BU-0282` (`skills/dispatch/SKILL.md (skills/dispatch/SKILL.md L184)`)
   - Trigger: multiple findings share the same root cause
   - Outcome: remediation converges to one worker per root cause, is rechecked before merge, and escalates to a human after two unsuccessful cycles rather than looping indefinitely
   - Statement: Findings sharing the same originating run, head, owning module, and root cause share one serialized remediation worker/branch rather than one worker per finding; before merging the group, native tests and independent rereviews (verifying mutation before validation, partial publication or rollback, and identity/provenance) are rerun; after two remediation cycles, fix dispatch stops and an architectural/root-cause review plus a human decision is required.

### `notify-primary-session`

- **Trigger:** the notify step is invoked
- **Outcome:** a durable, searchable activity record exists for every update regardless of the transport outcome
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `capture-wiki-activity`'s.
- **Member stage count:** 2

**Workflow-level helpers** (`workflow=notify-primary-session`, `stage=null` — deterministic
machinery referenced by the workflow as a whole, not one specific stage):

- `BU-0680` — A worker update message is classified into one of three durable event kinds by a fixed prefix match on the message text: done/failed become completion, needs_input/blocked become escalation, anything else becomes update.

**Ordered member stages:**

1. **`publish-notification`** — `BU-0683` (`bin/sgt-notify (bin/sgt-notify L51-60)`)
   - Trigger: the notify step is invoked
   - Outcome: the fleet watcher can discover the update from a durably persisted marker even if the requested transport later fails
   - Statement: Every notify call durably records a metadata-only wake marker (event class and update timestamp) for the task by writing it to a private temp file and atomically renaming it into place, regardless of which notification transport is used.
   - Stage-context attachments (6):
     - `BU-0684`: Unless overridden, the notify step's only externally observable notification side effect is the durable wake marker — no direct injection into any session occurs by default.
     - `BU-0685`: In tmux transport mode, if no primary_pane has ever been recorded for the task, the notify step treats the update as satisfied via the durable callback queue when one is registered, and only fails hard if no durable callback origin exists either.
     - `BU-0686`: In tmux transport mode, a primary_pane file that exists but is empty is a hard failure to notify — unlike the missing-file case, this is not softened by an available durable callback origin.
     - `BU-0687`: In tmux transport mode, if the recorded primary pane's tmux session is no longer running, the notify step treats the update as satisfied via the durable callback queue when one is registered, and only fails hard if no durable callback origin exists either.
     - `BU-0688`: In tmux transport mode, when a live primary pane is available, the notify step injects the update message directly into that pane via tmux send-keys.
     - `BU-0692`: The notify step's own exit status reflects an earlier durable-callback sync failure even when the requested notification transport itself succeeded.
   - Helper attachments (1):
     - `BU-0689`: An unrecognized SERGEANT_NOTIFY_TRANSPORT value is a hard configuration error.

2. **`capture-wiki-activity`** — `BU-0691` (`bin/sgt-notify (bin/sgt-notify L119-124)`)
   - Trigger: the notify step is invoked
   - Outcome: a durable, searchable activity record exists for every update regardless of the transport outcome
   - Statement: Every notify call writes a wiki activity entry recording the task id, event class, full message text, and any extracted PR link — independent of which transport delivered, or failed to deliver, the update.
   - Helper attachments (1):
     - `BU-0690`: The wiki activity entry for an update extracts and links the first GitHub PR URL found in the message text, if any.

### `prototype`

- **Trigger:** the user wants a throwaway prototype to answer a design question
- **Outcome:** the user can independently explore variants, and cross-variant preferences are captured as signal rather than treated as noise
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `drive-ui-prototype`'s.
- **Member stage count:** 7

**Ordered member stages:**

1. **`select-branch`** — `BU-1078` (`.agents/skills/prototype/SKILL.md (.agents/skills/prototype/SKILL.md L12-15)`)
   - Trigger: the user wants a throwaway prototype to answer a design question
   - Outcome: the question is classified as logic or UI and routed to the matching branch's process
   - Statement: Which prototype branch to use is identified from the user's prompt, the surrounding code, or by asking the user if they are around: a logic/state-model question routes to the LOGIC.md branch, a what-should-this-look-like question routes to the UI.md branch.
   - Stage-context attachments (1):
     - `BU-1079`: If the branch (logic vs UI) is genuinely ambiguous and the user is not reachable, the branch is chosen by default to match the surrounding code (a backend module implies logic, a page or component implies UI), and the assumption is stated at the top of the prototype.

2. **`fold-and-preserve-prototype`** — `BU-1085` (`.agents/skills/prototype/SKILL.md (.agents/skills/prototype/SKILL.md L26)`)
   - Trigger: a prototype has answered its question
   - Outcome: the validated decision reaches main, the prototype itself is durably preserved off main, and both are cross-referenced
   - Statement: Once a prototype is done, any validated decision is folded into the real code, and the prototype itself is committed to a throwaway branch out of main as a primary source with a context pointer left on the implementation issue and the verdict/question captured in the issue or a commit — main keeps only the validated decision, not the prototype.
   - Stage-context attachments (6):
     - `BU-1097`: Once a logic prototype has answered its question, the answer is captured and the prototype itself is captured per the SKILL's rule, with the logic-specific mapping: the validated reducer/machine/function set lifts into the real module while the TUI shell rides along to the throwaway branch that preserves the prototype as a primary source.
     - `BU-1102`: The TUI shell built for a logic prototype is never shipped into production — only the logic module behind it is worth keeping, since the shell is optimized for being driven by hand from a terminal.
     - `BU-1119`: When sub-shape A's winning variant is chosen, it is folded into the existing page and the losing variants and the switcher are dropped from main.
     - `BU-1120`: When sub-shape B's winning variant is chosen, it is promoted to a real route and the throwaway route and switcher are dropped from main.
     - `BU-1121`: The full set of UI-prototype variants is preserved as a primary source on the throwaway branch rather than deleted, because leftover variant components and the switcher left in the main branch rot quickly and confuse the next reader.
     - `BU-1125`: A UI prototype's variant code, written under prototype constraints (no tests, minimal error handling), is never promoted directly to production — it is rewritten properly when folded into the real codebase.

3. **`build-logic-prototype`** — `BU-1086` (`.agents/skills/prototype/LOGIC.md (.agents/skills/prototype/LOGIC.md L18)`)
   - Trigger: a logic prototype is about to be built
   - Outcome: the question being answered is written down and checkable, rather than implicit in the author's head
   - Statement: Before writing any code for a logic prototype, the state model and the question being prototyped are written down in one paragraph (in the README or a top-of-file comment), so the question is explicit and checkable later whether the user is watching now or returns to it later.
   - Stage-context attachments (13):
     - `BU-1087`: A logic prototype uses whatever language/runtime the host project already uses; if the project has no obvious runtime, the user is asked rather than one being picked unilaterally.
     - `BU-1088`: A logic prototype matches the project's existing tooling conventions rather than adding a new package manager or runtime just for the prototype.
     - `BU-1089`: The actual logic being tested is isolated behind a small, pure interface that could be lifted out into the real codebase later — the throwaway TUI wraps it, but the logic module itself is not meant to be thrown away.
     - `BU-1090`: The shape of the isolated logic module is chosen to fit the question: a pure reducer for discrete actions over a single state value, a state machine when legal-action-ness is itself part of the question, a small set of pure functions when there's no implicit current state, or a class/module with a clear method surface when the logic genuinely owns ongoing internal state.
     - `BU-1091`: The isolated logic module stays pure — no I/O, no terminal code, no console.log for control flow — with the TUI only ever calling into it, never the reverse.
     - `BU-1092`: The logic prototype's TUI is built as a lightweight terminal app that clears the screen and re-renders the whole frame on every tick, so the user always sees one stable view rather than an ever-growing scrollback.
     - `BU-1093`: Each rendered TUI frame has two parts in a fixed order: the current state (pretty-printed, diff-friendly, styled with bold/dim for emphasis) first, then the available keyboard shortcuts listed at the bottom.
     - `BU-1094`: The logic prototype's TUI runs a fixed loop: initialize a single in-memory state object and render the first frame on start, read one keystroke or line at a time and dispatch it to a handler that mutates state, re-render the full frame after every action by replacing rather than appending, and loop until the user quits.
     - `BU-1095`: The logic prototype is made runnable in one command by adding a script to the project's existing task runner; if the host project has no task runner, the command is instead placed at the top of the prototype's README.
     - `BU-1098`: A logic prototype does not get tests added to it — a prototype that needs tests is no longer a prototype.
     - `BU-1099`: A logic prototype does not wire to the real database — an in-memory store is used instead, unless the question being prototyped is specifically about persistence.
     - `BU-1100`: A logic prototype does not generalize beyond the one question it exists to answer — speculative "what if we wanted to support X later" scope is excluded.
     - `BU-1101`: The logic and the TUI are kept from blurring together: if the reducer or state machine references console.log, prompts, or terminal escape codes, it is no longer portable, so the TUI is kept as a thin shell over a pure module.

4. **`drive-logic-prototype`** — `BU-1096` (`.agents/skills/prototype/LOGIC.md (.agents/skills/prototype/LOGIC.md L65-67)`)
   - Trigger: the logic prototype has been handed over to the user
   - Outcome: user surprises are treated as the prototype's real output, and the prototype is extended on request rather than treated as frozen
   - Statement: Once handed over with its run command, the logic prototype is driven by the user; moments where the user says something shouldn't be possible or assumed something would be different are treated as the real bugs being sought, and requested new actions are added since prototypes evolve.

5. **`select-ui-subshape`** — `BU-1103` (`.agents/skills/prototype/UI.md (.agents/skills/prototype/UI.md L16)`)
   - Trigger: a UI prototype is being started and a sub-shape must be chosen
   - Outcome: sub-shape A is chosen unless there's a specific reason it can't host the variants
   - Statement: Between the two UI-prototype sub-shapes, sub-shape A (adjustment to an existing page) is the default whenever a plausible existing page can host the variants; sub-shape B (a new page) is reached for only when the prototype genuinely has no nearby home.
   - Stage-context attachments (4):
     - `BU-1104`: In sub-shape A, variants are rendered on the existing route itself, gated by a ?variant= URL search param, while the route's existing data fetching, params, and auth are left unchanged — only the rendering swaps.
     - `BU-1105`: A feature that doesn't yet have its own page but would naturally live inside an existing one (a new dashboard section, a new settings card, a new flow step) is still treated as sub-shape A, with the variants mounted inside that host page.
     - `BU-1106`: Sub-shape B (a throwaway new route) is used only when the prototyped thing genuinely has no existing page to live inside; the route follows the project's existing routing convention, is named to be obviously a prototype, and uses the same ?variant= pattern as sub-shape A.
     - `BU-1107`: Before committing to sub-shape B, a sanity check is made for whether the prototype could really not be embedded in an existing page, because an empty route hides design problems that a populated one would expose.

6. **`build-ui-prototype`** — `BU-1108` (`.agents/skills/prototype/UI.md (.agents/skills/prototype/UI.md L38)`)
   - Trigger: the number of variants for a UI prototype is being decided
   - Outcome: the variant count stays in the 3-5 range rather than growing unbounded
   - Statement: A UI prototype defaults to 3 variants and caps at 5, because beyond 5 variants stop being radically different and start being noise.
   - Stage-context attachments (12):
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

7. **`drive-ui-prototype`** — `BU-1118` (`.agents/skills/prototype/UI.md (.agents/skills/prototype/UI.md L94-96)`)
   - Trigger: a UI prototype is handed over to the user
   - Outcome: the user can independently explore variants, and cross-variant preferences are captured as signal rather than treated as noise
   - Statement: Once a UI prototype is handed over, its URL (and variant keys) are surfaced to the user, who flips between variants at their own pace; feedback that mixes elements across variants (e.g. wanting one part from one variant and another part from a different one) is treated as the actual design signal.

### `record-recovery-pointer`

- **Trigger:** the task-tracker memory step is invoked with a worktree path
- **Outcome:** git identity is never captured against a wrong or unrelated checkout that merely happens to satisfy a looser "is a git worktree" test
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `bind-worktree-identity`'s.
- **Member stage count:** 1

**Workflow-level helpers** (`workflow=record-recovery-pointer`, `stage=null` — deterministic
machinery referenced by the workflow as a whole, not one specific stage):

- `BU-0334` — Recording a worker recovery pointer in the task tracker is skipped entirely (exit 0, no-op) when the repo state records no td_task for this repo.
- `BU-0335` — If the task tracker CLI is unavailable, the task-tracker memory step records a diagnostic naming the action and task and exits nonzero, rather than silently skipping the recovery-pointer write.

**Ordered member stages:**

1. **`bind-worktree-identity`** — `BU-0336` (`bin/sgt-td-memory (bin/sgt-td-memory L46-56)`)
   - Trigger: the task-tracker memory step is invoked with a worktree path
   - Outcome: git identity is never captured against a wrong or unrelated checkout that merely happens to satisfy a looser "is a git worktree" test
   - Statement: The task-tracker memory step binds every task tracker call to the worktree its own fleet state record actually owns: a missing owned-worktree record, or a WORKTREE argument that resolves (via realpath) to a different path than the recorded one, fails closed with a diagnostic rather than trusting the caller's argument.
   - Stage-context attachments (5):
     - `BU-0337`: For a handoff action, the worktree must resolve as the ROOT of a real git worktree (via git rev-parse --show-toplevel, realpath-compared), not merely a subdirectory of one or any unrelated repository; a response action has no such requirement since it records no git identity.
     - `BU-0338`: A handoff whose git HEAD cannot be resolved fails closed (diagnostic + exit 1) rather than recording a handoff with missing or fabricated git identity.
     - `BU-0340`: The task tracker handoff records a checkpoint summary (status/branch/head), a pointer to fleet state (message/diagnostic/worker.log) to reconcile before resuming, and an explicit decision note that raw escalation and response text stay out of the task tracker, delivered instead through the atomic .sergeant-response transport.
     - `BU-0341`: A response action's response ID is validated as exactly a 32-character lowercase hex string before any task tracker write; an invalid ID is diagnosed and the action fails.
     - `BU-0342`: The task tracker decision log for a delivered response intentionally excludes the response's exact text, recording only that a human response was received via Sergeant (by response-id) and directing the reader to the atomic .sergeant-response transport and updated fleet/git state.
   - Helper attachments (1):
     - `BU-0339`: A detached HEAD is recorded as the literal branch value "detached" in the handoff summary rather than left blank, since git reports an empty current-branch value for a real, valid detached state — not a capture failure.

### `research`

- **Trigger:** a topic needs primary-source research
- **Outcome:** research proceeds in parallel with the invoking actor's other work instead of blocking it
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `conduct-research`'s.
- **Member stage count:** 1

**Ordered member stages:**

1. **`conduct-research`** — `BU-0978` (`.agents/skills/research/SKILL.md (.agents/skills/research/SKILL.md L6-6)`)
   - Trigger: a topic needs primary-source research
   - Outcome: research proceeds in parallel with the invoking actor's other work instead of blocking it
   - Statement: A background agent is spun up to do primary-source research so the invoking actor keeps working while it reads.
   - Stage-context attachments (3):
     - `BU-0979`: The background research agent investigates against primary sources — official docs, source code, specs, first-party APIs — not secondary write-ups, following every claim back to the source that owns it.
     - `BU-0980`: The research findings are written to a single Markdown file, citing each claim's source.
     - `BU-0981`: The findings file is saved where the repo already keeps such notes, matching the existing convention; if there is no existing convention, it is placed somewhere sensible and the location is stated.

### `resolve-merge-conflict`

- **Trigger:** the resolving-merge-conflicts skill is invoked
- **Outcome:** the merge/rebase always reaches a resolved state rather than being abandoned mid-way
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `complete-merge`'s.
- **Member stage count:** 3

**Ordered member stages:**

1. **`establish-conflict-state`** — `BU-0982` (`.agents/skills/resolving-merge-conflicts/SKILL.md (.agents/skills/resolving-merge-conflicts/SKILL.md L6-6)`)
   - Trigger: the resolving-merge-conflicts skill is invoked
   - Outcome: resolution proceeds from an understood starting state rather than a blind one
   - Statement: Before resolving an in-progress merge/rebase conflict, the current state of the merge/rebase is established by checking git history and the conflicting files.

2. **`resolve-hunk`** — `BU-0984` (`.agents/skills/resolving-merge-conflicts/SKILL.md (.agents/skills/resolving-merge-conflicts/SKILL.md L10-10)`)
   - Trigger: a hunk's two sides' intents are understood
   - Outcome: the resolved hunk reflects one of the two original intents (or both), never a fabricated third behaviour
   - Statement: Each conflicting hunk is resolved by preserving both intents where possible; where they are incompatible, the side matching the merge's stated goal is picked and the trade-off is noted — never by inventing new behaviour.
   - Stage-context attachments (1):
     - `BU-0983`: For each conflicting change, the primary sources (commit messages, PRs, original issues/tickets) are found to understand deeply why each side's change was made and what its original intent was.

3. **`complete-merge`** — `BU-0985` (`.agents/skills/resolving-merge-conflicts/SKILL.md (.agents/skills/resolving-merge-conflicts/SKILL.md L10-10)`)
   - Trigger: a merge/rebase has conflicts
   - Outcome: the merge/rebase always reaches a resolved state rather than being abandoned mid-way
   - Statement: A merge or rebase conflict is always resolved rather than abandoned — `--abort` is never used.
   - Stage-context attachments (2):
     - `BU-0986`: After resolving conflicting hunks, the project's automated checks are discovered and run (typically typecheck, then tests, then format), and anything the merge broke is fixed.
     - `BU-0987`: Once checks pass, everything is staged and committed to finish the merge; if rebasing, the rebase is continued until all commits are rebased.

### `review-findings-routing`

- **Trigger:** a dispatched worker produces a review finding artifact
- **Outcome:** nothing observing the status file can ever see status=blocked while the message/generation describing why are still missing or stale
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `publish-blocked-gate`'s.
- **Member stage count:** 3
- **Ordering note:** `review-findings-routing` is graph-shaped (independent event-triggered
  entry points, not a single pipeline) — source/behavior_id order is used as
  the defensible default per the run-wide note above, not a proven chain.

**Ordered member stages:**

1. **`route-finding`** — `BU-0096` (`README.md (README.md L318)`)
   - Trigger: a dispatched worker produces a review finding artifact
   - Outcome: actionable findings become owning-repo task tracker tasks with durably published blocking guidance
   - Statement: Dispatched workers pass each axis's strict JSON finding artifact to the review-findings router, which creates or updates one owning-repository task tracker task per actionable finding, preserves active task state on reruns, and publishes blocking task IDs and remediation guidance through `.sergeant-message`, `.sergeant-status`, and the notify step.
   - Stage-context attachments (16):
     - `BU-0097`: Cosmetic and false-positive review dispositions create no cards; the schema rejects free-form review bodies, and credential-shaped values in accepted fields are redacted before durable storage.
     - `BU-0101`: If the stored card has changed since the router last wrote it, the whole stored revision is kept below a `--- Superseded revision (preserved) ---` separator and the card is labelled `needs-reconciliation`; that label is cleared only once a human has merged the two accounts.
     - `BU-0311`: A reviewer's structured JSON finding artifact carries only a fixed field set (`findings`, and per finding `id`/`severity`/`disposition`/`summary`/`evidence`/`paths`/`acceptance_criteria`/`recommendation`); review bodies, prompts, secrets, or credentials are never passed into the artifact, the task tracker, or fleet metadata.
     - `BU-0726`: Before any finding text is stored or displayed, the review-findings router redacts secret-shaped content out of it: password/token/secret/credential/api-key key:value pairs, Bearer and Basic auth headers, GitHub tokens, AWS access key IDs, credentials embedded in URLs, PEM private-key markers, and generic high-entropy strings are each replaced with a redaction marker.
     - `BU-0729`: An unrecognized finding severity spelling is rejected with an error naming the finding and listing every accepted severity spelling, and the rejected value is echoed back only when it is itself a short safe token — otherwise it is reported as '(unprintable)'.
     - `BU-0737`: No finding field (summary, evidence, paths, acceptance criteria, recommendation) may contain an embedded newline or carriage return, on both the fresh-parse and the retry-replay validation paths.
     - `BU-0738`: No finding field may begin with one of the two body lines the router itself gives structural meaning to ('Deduplication key: ' or 'Finding content digest: ') — the router gives no other line structural meaning, and this constraint is enforced on both the fresh-parse and retry-replay validation paths.
     - `BU-0745`: A finding whose canonical severity has no known td-priority mapping causes the whole routing attempt to fail, rather than the finding being silently dropped or given a default priority.
     - `BU-0747`: The dedup matcher for an existing task tracker card only ever identifies the card and classifies it as 'same' or 'diverged' by comparing the router's own recomputed content digest against the digest line stored in the card's description — it never reads the rest of the stored body back in order to merge, reconcile, or rewrite it.
     - `BU-0748`: If more than one existing task tracker task matches a finding's deduplication key, or a match is reported with no task id or an unrecognized digest state, routing that finding fails rather than guessing which match to use.
     - `BU-0749`: A finding whose stored card's content digest diverges from the freshly computed one is refused: the stored card is left completely untouched (no description, title, label, or status change), the refusal is reported, and routing continues with the rest of the batch rather than aborting.
     - `BU-0750`: If every actionable finding in a batch was refused because its stored card no longer corresponds, the entire routing attempt is treated as a failure — an axis that ends up recording zero actual review evidence is not reported as a success.
     - `BU-0751`: A same-digest matching task tracker card only has its priority and labels refreshed on a re-route — the description and title are never rewritten, so a human annotation or a hand-edited title on the card survives by construction.
     - `BU-0753`: A same-digest matching task tracker card's defer_until is preserved across reruns; only an explicit human action (the task tracker defer --clear) may clear a deliberate deferral — an automated re-route does not clear it.
     - `BU-0755`: A recurring finding's current occurrence is recorded as a new comment on the existing task tracker card rather than by mutating the stored description, so provenance of each occurrence is preserved without ever overwriting history.
     - `BU-0760`: The review-findings router's final report distinguishes three outcomes to the caller: any refused findings are reported by name even on an otherwise successful run, a run with zero actionable findings is reported distinctly from one with nonblocking findings routed to the task tracker, and a run with blocking findings exits 2 with a published blocked state instead of reaching this final report at all.
   - Helper attachments (23):
     - `BU-0098`: `severity` is normalized to a canonical `error`/`warning`/`info` from the aliases reviewers actually emit — `blocker`/`critical`/`high` → `error` (P1); `major`/`medium` → `warning` (P2); `minor`/`low`/`informational` → `info` (P3) — and only the `error` family publishes a blocking gate.
     - `BU-0099`: Deduplication is scoped to owning repo, axis, source, finding id, parent mission, and branch, so two sessions cannot collide on a generic finding id such as `spec-1`.
     - `BU-0100`: An update never silently replaces a stored finding card: each revision the router writes ends with a `Revision block digest:` line over its own bytes, so a later route can tell whether the stored revision is still exactly what the router wrote.
     - `BU-0102`: A closed card matching a finding is always reopened rather than abandoned, and the reopen is always reported.
     - `BU-0302`: Only the canonical `error` severity family publishes a blocking review gate; `high` is deliberately treated as must-fix (mapped to `error`) so a genuinely blocking finding can never ship as mere debt.
     - `BU-0711`: Replaying a retained review artifact via --retry is mutually exclusive with supplying fresh review context (--input/--axis/--source/etc.) in the same invocation.
     - `BU-0720`: Before any repository or the task tracker action is taken, the review-findings router requires axis, source, branch, head SHA, and parent-task to be present, requires either --retry or --input to have been given, requires a real (non-'unknown') task id, and requires the axis and source to each match their accepted formats.
     - `BU-0722`: The branch name, head SHA, parent task id, and fleet task id supplied to the review-findings router must each match a fixed safe-character format, or the routing attempt is refused.
     - `BU-0723`: The review-findings router requires the named project's config to exist and the yq, git, and td (marcus/td) tools to be available, and requires either a retry or an existing input file, before doing any repository lookup.
     - `BU-0724`: The owning repository's filesystem path is resolved by looking it up by name in the project's configuration, and the resolved path must exist and be an actual git repository, or routing fails.
     - `BU-0725`: A review input file is rejected unless its top-level JSON is an object with exactly one key, 'findings', whose value is a list.
     - `BU-0727`: Each finding object in a review input must have a field set exactly equal to the allowed finding schema (id, severity, disposition, summary, evidence, paths, acceptance_criteria, recommendation) — neither missing a required field nor carrying an unrecognized one.
     - `BU-0728`: A finding's id must match a fixed safe-token character pattern after redaction/cleaning, or the input is rejected.
     - `BU-0730`: A finding's reported severity spelling is normalized to one canonical severity; the original reported spelling is retained separately for provenance only when it differs from the canonical form.
     - `BU-0731`: A finding's disposition must be one of actionable, cosmetic, or false-positive, or the input is rejected.
     - `BU-0732`: A finding's paths field must be a list of strings, or the input is rejected.
     - `BU-0733`: A finding's summary, evidence, acceptance_criteria, and recommendation fields must each be non-empty after cleaning, or the finding is rejected — an empty field would leave the task tracker card body ending in trailing whitespace, making the content digest depend on invisible characters.
     - `BU-0734`: A finding's content digest is computed only over its sanitized, externally observable fields (severity, summary, evidence, paths, acceptance criteria, recommendation) — the branch, head SHA, parent mission, and fleet task are deliberately excluded, so a rerun of the same finding under different run metadata does not create what looks like a new revision.
     - `BU-0744`: A finding whose disposition is not 'actionable' (cosmetic or false-positive) is skipped entirely — no task tracker lookup, no task tracker card, no gate effect — and reported as ignored.
     - `BU-0746`: A finding's deduplication marker is scoped to the repo, review axis, review source, finding id, parent mission, and branch together — not to axis/source/finding-id alone — because reviewers commonly emit generic finding ids that would otherwise collide across unrelated review sessions and let one session's update overwrite another's evidence.
     - `BU-0752`: A same-digest matching task tracker card that had been closed is reopened when the same finding is reported again, with a notice printed that the finding is still being reported.
     - `BU-0754`: A same-digest matching task tracker card's labels are merged with the standard labels (independent-review, finding, axis) rather than replaced, deduplicating, so manually added labels are never stripped by a rerun.
     - `BU-0756`: If no existing task tracker task matches a finding's deduplication key, the review-findings router creates a new task tracker task carrying the finding's computed priority, standard labels, and full composed body.

2. **`preserve-retry-evidence-on-failure`** — `BU-0103` (`README.md (README.md L324-328)`)
   - Trigger: a review-finding route fails after the findings have been parsed
   - Outcome: the parsed, sanitized findings are durably retained with an exact retry command surfaced
   - Statement: When a review-finding route fails after parsing, the sanitized findings are retained under `<worktree>/.sergeant-review-artifacts/<axis>-<source>/` and the blocked message names the exact retry command.
   - Stage-context attachments (13):
     - `BU-0104`: The retained review-artifact holds only post-redaction fields, never the reviewer's original output, and a retry re-validates every field and recomputes each content digest before anything reaches the task tracker; a route refuses to overwrite an artifact nobody has retried yet.
     - `BU-0312`: Malformed reviewer output or a routing failure is treated as blocked; the worker never continues past it or leaves the findings only in worker.log.
     - `BU-0716`: A --retry artifact path must resolve to a location strictly inside this worktree's own artifact root, matching a safe filename pattern, or the retry is refused.
     - `BU-0717`: A retry artifact must contain both a findings file and a meta file, or the retry is refused as incomplete.
     - `BU-0718`: A retained artifact's stored project and repo (from its meta file) must exactly match the project/repo the retry was invoked against, or the retry is refused.
     - `BU-0719`: An unrecognized key in a retry artifact's meta file causes the retry to be refused rather than silently ignored.
     - `BU-0735`: A retried (replayed) review artifact is re-validated against the exact same rules used for fresh input, because a retry replays a retained artifact without the reviewer's original JSON and without re-running the redaction step, so the same content-safety guarantees have to be re-established independently rather than assumed from the fact that the artifact was retained.
     - `BU-0736`: A retained review artifact is never allowed to be empty; only a fresh (non-retry) parse of a review with genuinely zero findings is allowed to be empty.
     - `BU-0739`: On retry replay, each finding's content digest is re-derived from its own field content and compared against the stored digest; a stored digest can never certify text that has since been altered.
     - `BU-0740`: The review-findings router refuses to overwrite an existing pending retained artifact for the same axis/source — the existing one must be retried first, since by construction it is the only surviving copy of that review until it is.
     - `BU-0741`: Retaining parsed findings stages them into a private temporary directory and publishes the whole artifact (findings + meta) with a single atomic rename, so a crash partway through can never leave findings on disk without its matching meta, and two concurrent invocations cannot collide on the same temp name.
     - `BU-0742`: Removing a retained artifact after successful routing treats a removal failure as non-fatal (a warning, not an abort), while continuing to name the still-present artifact so a later routing failure's retention message stays truthful rather than falsely claiming nothing was kept.
     - `BU-0743`: A fresh review that parses to zero findings is not retained — there is nothing to retry.

3. **`publish-blocked-gate`** — `BU-0714` (`bin/sgt-review-findings (bin/sgt-review-findings L97-106)`)
   - Trigger: _publish_blocked runs
   - Outcome: nothing observing the status file can ever see status=blocked while the message/generation describing why are still missing or stale
   - Statement: When publishing a blocked review-gate state, the message and generation files are durably written and published before the status file itself is flipped to 'blocked' — the status transition to blocked is the last write, and only happens after the notify attempt.
   - Stage-context attachments (5):
     - `BU-0715`: A failure notifying the coordinator (the notify step) while publishing a blocked review state is reported as an error and its exit status is propagated from _publish_blocked, rather than being silently absorbed.
     - `BU-0721`: The review-findings router takes a snapshot, under lock, of the current global gate generation and (if one exists) this axis's own gate-file generation at the very start of a routing attempt, before doing any of the attempt's real work.
     - `BU-0757`: Any actionable finding whose severity is in the blocking class causes the run to publish a blocked review-remediation state naming every routed task-tracker task for the axis (whether newly created or deduplicated) and to exit 2 — this happens regardless of whether each finding's task tracker side effect was a create, an update, or came from a prior run.
     - `BU-0758`: Clearing this axis's own published gate file, and re-deriving the aggregate blocked message from what remains, only happens if the axis's gate-file generation is still exactly what was observed when this routing attempt started — a concurrent invocation that already advanced or cleared that same gate is not clobbered by this attempt's own clear.
     - `BU-0759`: When there is no per-axis gate file involved (a gate-less recovery from a prior routing failure), the review-findings router only clears the overall blocked status if no OTHER review axis currently has an open gate file — a clean retry for one axis must never unblock a worker that a different axis is still legitimately blocking.
   - Helper attachments (2):
     - `BU-0712`: Publishing a blocked review-gate state advances a single worktree-wide gate generation counter every time it is called.
     - `BU-0713`: Each review axis's blocked-state message is stored in its own gate file, and the worktree's aggregate blocked message is the concatenation of every currently active axis's gate message — one axis publishing a block does not erase another axis's still-open block message.

### `sergeant-help`

Established by `BU-0123` (`skills/sergeant-help/SKILL.md (skills/sergeant-help/SKILL.md L8-13)`): The sergeant-help skill is loaded for questions about what Sergeant is, install/configure/use, skill sources, running a command/workflow, or diagnosing an error, but is never loaded as a substitute for `load-project`, `cross-repo-work`, `dispatch`, or `wiki` once the user has requested execution of those procedures.

- **Trigger:** sergeant-help is answering a question
- **Outcome:** each condition triggers its own fixed required action rather than an ad hoc response
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `handle-failure-or-handoff`'s.
- **Member stage count:** 2

**Ordered member stages:**

1. **`research-and-answer`** — `BU-0124` (`skills/sergeant-help/SKILL.md (skills/sergeant-help/SKILL.md L30-44)`)
   - Trigger: sergeant-help is answering a question
   - Outcome: the answer follows this fixed research/answer sequence rather than free-form search
   - Statement: The query procedure classifies the question against the documentation map, reads the primary document first, escalates to a repository-wide grep search only for unresolved terms, consults the graph-generation tool for architectural questions when a graph exists, and answers with the exact command, required preconditions, expected evidence, and links to repository-relative documentation.
   - Stage-context attachments (3):
     - `BU-0125`: When sources disagree, precedence is: command behavior/tests/supported `--help` output for released syntax; `AGENTS.md` for always-on execution/safety policy; the trigger-loaded skill for its procedure; `docs/schema.md` for project fields; user documentation for walkthroughs.
     - `BU-0126`: The skill states when a behavior is undocumented or contradictory rather than inventing a command, flag, state transition, or safety guarantee.
     - `BU-0127`: Destructive operations are kept out of examples unless the documentation requires confirmation and the user explicitly requested them.

2. **`handle-failure-or-handoff`** — `BU-0128` (`skills/sergeant-help/SKILL.md (skills/sergeant-help/SKILL.md L69-74)`)
   - Trigger: one of the four named conditions occurs while answering a help question
   - Outcome: each condition triggers its own fixed required action rather than an ad hoc response
   - Statement: On a missing primary document, the skill reports the expected path and stops before guessing; on a command/docs mismatch it reports the mismatch and trusts tested/released behavior; on a question requiring project ownership it loads `load-project` and runs the project context-resolution step; on a question requiring implementation or fleet mutation it hands off to the owning procedural skill, since help remains read-only.

### `sergeant-setup`

- **Trigger:** a checklist step is reached
- **Outcome:** a successful Graphify run is verified by the presence of both named output files
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `phase9-graphify-init`'s.
- **Member stage count:** 10

**Ordered member stages:**

1. **`run-checklist`** — `BU-1266` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L45-48)`)
   - Trigger: a checklist step is reached
   - Outcome: already-satisfied steps are skipped silently and every step's outcome is recorded visibly
   - Statement: The skill maintains a visible, numbered checklist in terminal output; before each step it verifies whether the step is already complete and skips it without prompting if so, and after each step it writes an `[ok]` or `[skipped]` status line.
   - Stage-context attachments (2):
     - `BU-1267`: When a phase fails, the skill stops the current run with actionable output identifying the last completed phase.
     - `BU-1268`: On the next invocation the checklist starts over from Phase 1 but skips every phase that already passes verification; resumability works by re-checking each phase before acting on it, not by persisting state between runs.

2. **`phase1-detect-prerequisites`** — `BU-1269` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L59-68)`)
   - Trigger: the skill checks the required prerequisite list
   - Outcome: `td` is accepted only if it is the specific Marcus implementation with the named flag support, and at least one of the three named interactive agents must be present
   - Statement: Phase 1 classifies each prerequisite as `present`, `installable`, or `unsupported`; the required set includes `td` specifically as the Marcus implementation, verified with `td version` and `td create --help` and required to support `--description`, `--json`, and `--work-dir`, plus at least one interactive agent among `opencode`, `goose`, or `claude`.
   - Stage-context attachments (5):
     - `BU-1270`: Optional prerequisites (the toolchain/task runner, the treehouse session manager, the graph-generation tool, the validation pipeline, `node`/`npm`) are skipped, not failed, if absent.
     - `BU-1271`: For each unsupported prerequisite, the skill shows a draft task tracker issue (title, description, acceptance criteria) and asks for explicit approval; the issue is created only after the user types `y` or `yes`, and if declined the gap is reported in the summary without creating tracking work.
     - `BU-1272`: The skill does not continue past Phase 1 until all required prerequisites are either present or the user has explicitly accepted the risk of proceeding without them.
     - `BU-1273`: For each installable prerequisite, the skill shows the installation command and asks for explicit consent; the command runs only after the user types `y` or `yes`, and is not run on any other response.
     - `BU-1296`: When a prerequisite install is declined, the skill reports what was skipped and asks whether to continue, rather than silently treating the decline as either a hard stop or an implicit continue.

3. **`phase2-install-checkout`** — `BU-1274` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L107-114)`)
   - Trigger: no local Sergeant clone exists
   - Outcome: the destination is confirmed with the user before any clone command runs
   - Statement: If the Sergeant repository is not already cloned, the skill first asks the user where to place the clone and waits for an answer before proceeding to the next step.
   - Stage-context attachments (3):
     - `BU-1275`: The skill shows the exact `git clone` command and destination and asks for consent; the command runs only after the user types `y` or `yes`, leaving the filesystem unchanged on any other response.
     - `BU-1276`: If the toolchain/task runner is available, the skill determines the actual install directory, shows the resolved target, and asks for consent before running the toolchain/task runner; if the toolchain/task runner is unavailable or consent is declined, it instructs the user to symlink commands from `bin/` manually and verify the result before continuing.
     - `BU-1277`: The skill verifies that at least the fleet-listing step, the project context-resolution step, the dispatch step, and the interactive fleet-watch loop resolve on `PATH` before proceeding, reports any missing commands and their expected source path, and stops the current run if verification fails after install instructions were followed — the next run re-checks Phase 2.

4. **`phase3-global-config`** — `BU-1278` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L147-158)`)
   - Trigger: `~/.config/sergeant/config.yaml` does not exist
   - Outcome: the config file is written only after an explicit confirmed preview
   - Statement: In Phase 3, if `~/.config/sergeant/config.yaml` is missing, the skill asks the user for a `dev_root` path, shows a preview, and asks for confirmation; the file is written only after the user confirms, and the filesystem is left unchanged on any other response.
   - Stage-context attachments (2):
     - `BU-1279`: If the global config is present and `dev_root` is set, the skill reports `[ok]` without further action.
     - `BU-1280`: If the global config is present but invalid YAML, the skill validates it with `yq e '.' ~/.config/sergeant/config.yaml`, reports the parse error, and stops; it must not overwrite the file without a timestamped backup, a diff preview, and explicit confirmation.

5. **`phase4-new-project-interview`** — `BU-1282` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L169-171)`)
   - Trigger: a new project YAML is being created
   - Outcome: each question is answered in sequence before the next is asked
   - Statement: Phase 4's interview is for new projects only, and its questions are asked in order, stopping to wait for each answer before proceeding to the next.
   - Stage-context attachments (4):
     - `BU-1281`: If the project YAML already exists and the user wants to modify it, Phase 4 is skipped and Phase 5 (repair existing YAML) is used instead.
     - `BU-1283`: The project name, which becomes the YAML filename stem, must match `[a-z0-9_-]+`.
     - `BU-1284`: After all interview answers are collected, the skill shows a preview of the complete YAML before writing anything and asks for confirmation.
     - `BU-1285`: The file is written only after the user confirms; if the file already exists, a backup is created at `~/.config/sergeant/<name>.yaml.bak.<timestamp>` before writing.

6. **`phase5-repair-project-yaml`** — `BU-1286` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L204-205)`)
   - Trigger: the existing project YAML fails validation
   - Outcome: the run stops on the parse error rather than attempting further changes
   - Statement: In Phase 5, the skill validates the existing project YAML with `yq e '.' ~/.config/sergeant/<name>.yaml`; if it fails, it reports the parse error and stops without proceeding.
   - Stage-context attachments (3):
     - `BU-1287`: The skill computes and displays a minimal diff between the current YAML content and the proposed changes.
     - `BU-1288`: The skill asks for confirmation before any write or backup in Phase 5, and only after the user confirms does it create a timestamped backup at `~/.config/sergeant/<name>.yaml.bak.<timestamp>` and then write the new content.
     - `BU-1289`: The skill does not create the Phase 5 backup before confirmation, does not apply changes if the user declines, and the backup is mandatory when writing — it is never skipped even if the user asks to skip it.

7. **`phase6-verify-installation`** — `BU-1290` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L220-231)`)
   - Trigger: the project YAML has just been written
   - Outcome: verification proceeds strictly in order and halts at the first failure
   - Statement: In Phase 6, after the YAML is written the skill runs the fleet-listing step, the project context-resolution step, the status step, and the sync step in order, reporting the result of each, stopping and reporting the first failure with its full output and not continuing to the next command until the previous one succeeds.

8. **`phase7-task-tracker-init`** — `BU-1291` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L235-248)`)
   - Trigger: a project repository's task tracker status is unknown
   - Outcome: each repository's task tracker state is either confirmed `[ok]` or consent-gated before initialization
   - Statement: In Phase 7, for each project repository the skill checks the task tracker; if initialized it reports `[ok]`, and if not it shows the task tracker command and asks for consent, running it only after the user confirms and reporting a decline as `[skipped]` in the Phase 9 summary while continuing.
   - Stage-context attachments (1):
     - `BU-1292`: The skill does not initialize the task tracker in any repository that was not registered in the current project YAML.

9. **`phase8-treehouse-init`** — `BU-1293` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L255-263)`)
   - Trigger: the treehouse session manager may or may not be installed
   - Outcome: Treehouse is initialized only with consent, and its absence or decline never blocks overall setup completion
   - Statement: In Phase 8, if the treehouse session manager is present on `PATH` the skill offers to initialize Treehouse worktree pools, running the treehouse-init step only on confirmation, skipping silently on decline or absence, and never marking setup incomplete because Treehouse was skipped.

10. **`phase9-graphify-init`** — `BU-1294` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L267-275)`)
   - Trigger: the graph-generation tool is available and the project YAML requests output
   - Outcome: a successful Graphify run is verified by the presence of both named output files
   - Statement: In Phase 9, if the graph-generation tool is present on `PATH` and the project YAML has a `graphify.output` field, the skill offers to run the graph-generation step, running it only on confirmation and skipping silently on decline, and requires both `graph.json` and `GRAPH_REPORT.md` to exist at the configured output path after a successful run.

### `standard-workflow`

- **Trigger:** a task is brought to the session
- **Outcome:** cleanup runs only after terminal state and evidence preservation are verified
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `deliver-and-cleanup`'s.
- **Member stage count:** 8

**Ordered member stages:**

1. **`load-context`** — `BU-0025` (`AGENTS.md (AGENTS.md L136)`)
   - Trigger: a task is brought to the session
   - Outcome: context is fully loaded before an execution mode is chosen
   - Statement: Step 1 of the standard workflow: run the project context-resolution step and identify the owning repository/repositories, inherited instructions, configured paths, and cross-repository dependencies before selecting an execution mode.
   - Stage-context attachments (1):
     - `BU-0134`: The coordinator is started from the Sergeant checkout inside tmux so `AGENTS.md` is loaded and dispatch can bind the exact coordinator identity.
   - Helper attachments (3):
     - `BU-0237`: For each repository, the project context-resolution step reports one of three clone-status states: cloned with its current branch, directory exists but is not a git repo, or NOT CLONED.
     - `BU-0238`: When a project configures the graph-generation tool output, the project context-resolution step reports whether a built graph (`GRAPH_REPORT.md`) is available to read, or names the exact command to build one if not.
     - `BU-0888`: The default development root directory and default identity are read from the user's config.yaml if the file exists and yq is installed; otherwise Sergeant falls back to its built-in defaults.

2. **`load-or-create-task`** — `BU-0026` (`AGENTS.md (AGENTS.md L137)`)
   - Trigger: context has been loaded
   - Outcome: an existing canonical task tracker task is reused when one exists; a new one is created only otherwise
   - Statement: Step 2 of the standard workflow: run the task-tracker listing step and reuse a matching task in direct or dispatch mode; create a task only when no canonical task exists.

3. **`select-execution-mode`** — `BU-0027` (`AGENTS.md (AGENTS.md L138)`)
   - Trigger: the task queue has been checked
   - Outcome: an execution mode is chosen according to this rule
   - Statement: Step 3 of the standard workflow: choose direct mode for explicit single-repo work in this session, dispatch mode for cross-repo, parallel, or explicitly delegated work.

4. **`reconcile-existing-state`** — `BU-0028` (`AGENTS.md (AGENTS.md L139)`)
   - Trigger: an execution mode has been chosen, before starting work
   - Outcome: existing state is reconciled and reused rather than duplicated
   - Statement: Step 4 of the standard workflow: run the interactive fleet-watch loop, then inspect active workers, branches, worktrees, retained gates, and handoffs before starting; resume or take over preserved work rather than creating duplicates.

5. **`confirm-with-user`** — `BU-0029` (`AGENTS.md (AGENTS.md L140)`)
   - Trigger: state has been reconciled
   - Outcome: the user is asked only for genuinely unresolved, scope/risk-changing decisions
   - Statement: Step 5 of the standard workflow: ask the user only to confirm unresolved decisions that change scope or risk — repository ownership, user-visible behavior, security/privacy policy, data retention, destructive action, or an irreversible tradeoff that is unknown.
   - Stage-context attachments (2):
     - `BU-0030`: The user is not asked to reconfirm an execution mode, plan, or tradeoff already recorded in the conversation or in the task tracker.
     - `BU-0281`: If a consequential behavioral seam is undecided, the worker escalates `needs_input` rather than guessing.

6. **`execute`** — `BU-0031` (`AGENTS.md (AGENTS.md L141-143)`)
   - Trigger: decisions have been confirmed
   - Outcome: execution proceeds via the mode-appropriate path
   - Statement: Step 6 of the standard workflow (execute): in direct mode, start the task tracker task and implement through tests, review, and delivery; in dispatch mode, use the dispatch step.

7. **`resolve-blocking-gate`** — `BU-0034` (`AGENTS.md (AGENTS.md L145)`)
   - Trigger: a worker reaches needs_input, blocked, or an ask-user gate
   - Outcome: only genuinely missing decisions are solicited, recorded in the task tracker, and remediation continues without redundant re-asks
   - Statement: Step 8 of the standard workflow: for `needs_input`, `blocked`, or ask-user gates, read the exact finding, obtain only genuinely missing user decisions, record them in the task tracker, and continue approved remediation without asking again merely to dispatch.

8. **`deliver-and-cleanup`** — `BU-0035` (`AGENTS.md (AGENTS.md L146)`)
   - Trigger: work has reached a terminal or deliverable state
   - Outcome: cleanup runs only after terminal state and evidence preservation are verified
   - Statement: Step 9 of the standard workflow: surface PRs and merge order, complete approved merges/deployments, and run the fleet cleanup step only after terminal state and preserved evidence are verified.

### `sync-project-repos`

- **Trigger:** the sync step runs against an already-cloned repo
- **Outcome:** cloning happens only under the exact defined precondition, and ambiguous cases (occupied non-git path, no url) are skipped rather than acted on
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `clone-missing-repo`'s.
- **Member stage count:** 2

**Ordered member stages:**

1. **`sync-existing-repo`** — `BU-0241` (`bin/sgt-sync (bin/sgt-sync L30-39)`)
   - Trigger: the sync step runs against an already-cloned repo
   - Outcome: a diverged or detached-HEAD repo is left untouched with a warning instead of being force-merged
   - Statement: For an already-cloned repo, the sync step pulls with `--ff-only`, and skips the pull with a warning on detached HEAD or a failed (diverged/no-upstream) fast-forward, rather than forcing a merge or losing local state.

2. **`clone-missing-repo`** — `BU-0242` (`bin/sgt-sync (bin/sgt-sync L40-48)`)
   - Trigger: a configured repo path is missing
   - Outcome: cloning happens only under the exact defined precondition, and ambiguous cases (occupied non-git path, no url) are skipped rather than acted on
   - Statement: The sync step clones a repo only when its path does not exist and a `url` is configured; a path that exists but is not a git repo, or has no configured url, is skipped with a warning rather than being overwritten or guessed at.

### `tdd`

- **Trigger:** a TDD cycle is about to begin and seams have not yet been agreed
- **Outcome:** work proceeds one test-then-implementation slice at a time rather than as separate bulk test and implementation phases
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `run-red-green-loop`'s.
- **Member stage count:** 2

**Ordered member stages:**

1. **`agree-seams`** — `BU-1130` (`.agents/skills/tdd/SKILL.md (.agents/skills/tdd/SKILL.md L22)`)
   - Trigger: a TDD cycle is about to begin and seams have not yet been agreed
   - Outcome: testing effort is deliberately scoped to seams the user has confirmed, not left to improvisation
   - Statement: Before writing any test, the seams under test are written down and confirmed with the user; no test is written at a seam that hasn't been agreed, so testing effort lands on critical paths and complex logic rather than every edge case.
   - Stage-context attachments (1):
     - `BU-1127`: When exploring the codebase for TDD work, CONTEXT.md (if it exists) is read so test names and interface vocabulary match the project's domain language, and ADRs in the touched area are respected.

2. **`run-red-green-loop`** — `BU-1133` (`.agents/skills/tdd/SKILL.md (.agents/skills/tdd/SKILL.md L30)`)
   - Trigger: TDD work is being sequenced across multiple tests and their implementations
   - Outcome: work proceeds one test-then-implementation slice at a time rather than as separate bulk test and implementation phases
   - Statement: Horizontal slicing — writing all tests first, then all implementation — is an anti-pattern because bulk tests verify imagined shape rather than user-facing behavior and commit to test structure before the implementation is understood; vertical slices (one test, then one minimal implementation, repeated as a tracer bullet) are used instead.
   - Stage-context attachments (3):
     - `BU-1134`: In the red-green loop, the failing test is written first and only enough code is written to pass it, without anticipating future tests or adding speculative features.
     - `BU-1135`: Each TDD cycle covers exactly one seam, one test, and one minimal implementation.
     - `BU-1136`: Refactoring is not part of the red-green loop; it belongs to the separate review stage.

### `to-spec`

- **Trigger:** the current state of the codebase has not already been explored
- **Outcome:** the spec is immediately actionable in the tracker without a further triage pass
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `publish-spec`'s.
- **Member stage count:** 3

**Ordered member stages:**

1. **`prepare-spec-inputs`** — `BU-0990` (`.agents/skills/to-spec/SKILL.md (.agents/skills/to-spec/SKILL.md L13-13)`)
   - Trigger: the current state of the codebase has not already been explored
   - Outcome: the spec is grounded in the actual codebase and its existing vocabulary/decisions
   - Statement: If the current state of the codebase has not already been explored, it is explored before writing the spec; the project's domain glossary vocabulary is used throughout, and ADRs in the touched area are respected.
   - Stage-context attachments (2):
     - `BU-0988`: A spec produced by to-spec synthesizes what has already been discussed and understood about the codebase; the user is not interviewed.
     - `BU-0989`: If the issue tracker and triage label vocabulary have not been provided, /setup-matt-pocock-skills is run to establish them.

2. **`sketch-test-seams`** — `BU-0991` (`.agents/skills/to-spec/SKILL.md (.agents/skills/to-spec/SKILL.md L15-15)`)
   - Trigger: test seams for the feature are being sketched
   - Outcome: the spec settles on the minimum number of new, high-leverage test seams
   - Statement: Test seams for the spec's feature are sketched preferring existing seams over new ones, at the highest point possible; new seams are proposed only if needed, at the highest point possible, aiming for as few seams as possible (ideally exactly one).
   - Stage-context attachments (1):
     - `BU-0992`: Sketched test seams are checked with the user against their expectations before the spec is written.

3. **`publish-spec`** — `BU-0993` (`.agents/skills/to-spec/SKILL.md (.agents/skills/to-spec/SKILL.md L19-19)`)
   - Trigger: the spec has been written using the template
   - Outcome: the spec is immediately actionable in the tracker without a further triage pass
   - Statement: The finished spec is published to the project issue tracker with the `ready-for-agent` triage label applied, and no additional triage step is needed.
   - Stage-context attachments (5):
     - `BU-0994`: A published spec follows a fixed template, containing, in order: Problem Statement, Solution, User Stories, Implementation Decisions, Testing Decisions, Out of Scope, and Further Notes.
     - `BU-0995`: The spec's User Stories section is an extremely extensive, LONG numbered list covering all aspects of the feature, each in the form 'As an <actor>, I want a <feature>, so that <benefit>'.
     - `BU-0996`: The Implementation Decisions section does not include specific file paths or code snippets, since they may go outdated quickly.
     - `BU-0997`: If a prototype produced a snippet (state machine, reducer, schema, type shape) that encodes a decision more precisely than prose can, the snippet is inlined within the relevant decision, noted as having come from a prototype, and trimmed to only its decision-rich parts rather than kept as a working demo.
     - `BU-0998`: The spec's Testing Decisions section includes a description of what makes a good test (testing only external behaviour, not implementation details), which modules will be tested, and prior art (similar tests already in the codebase).

### `to-tickets`

- **Trigger:** the project name is not yet known
- **Outcome:** only tickets with no remaining blockers are reported as immediately dispatchable
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `report-dispatch-frontier`'s.
- **Member stage count:** 6

**Ordered member stages:**

1. **`load-ticket-context`** — `BU-1306` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L32)`)
   - Trigger: the project name is not yet known
   - Outcome: the project name is established before further context loading
   - Statement: Run the fleet-listing step if the project name is not already established, as the first step of loading project context.
   - Stage-context attachments (4):
     - `BU-1307`: Run the task-tracker listing step to deduplicate against every status before drafting tickets.
     - `BU-1308`: For architecture or codebase questions, use the existing graph-generation tool graph before reading files individually.
     - `BU-1309`: Read any referenced issue, PR, specification, ADR, or findings register in full before drafting tickets.
     - `BU-1310`: If an owning repository has no task tracker database, it is initialized with the task tracker only after confirming it is a real project repository.

2. **`draft-tickets`** — `BU-1316` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L80-84)`)
   - Trigger: a behavior spans multiple layers (storage, API, UI/CLI, tests)
   - Outcome: one ticket covers the full vertical slice rather than being split by layer, and is independently verifiable when done
   - Statement: Vertical slice rules require including every necessary layer for one behavior and forbid creating separate horizontal tickets (e.g. "write backend", "write frontend", "add tests") for that one behavior; a completed ticket must be demoable, testable, or operationally verifiable alone.
   - Stage-context attachments (8):
     - `BU-1312`: Decisions already approved are not reopened as questions when extracting decisions and unknowns.
     - `BU-1313`: A short investigation ticket is created only when an unknown cannot be answered from existing evidence, and it must name the decision or artifact it produces.
     - `BU-1314`: A ticket's `Blocked by` field lists only tickets that truly prevent starting or merging this work.
     - `BU-1315`: A ticket's `Preserved state` field records the branch, commit, PR, or worktree needed to resume the work.
     - `BU-1317`: Prefactoring is put first only when it materially reduces risk for the slices that follow it.
     - `BU-1318`: Wide refactors follow expand (add the new form beside the old), migrate (move callers in bounded, green batches), then contract (remove the old form after every migration ticket completes).
     - `BU-1319`: Migrate tickets are declared blocked by expand, and the contract ticket is declared blocked by every migration ticket.
     - `BU-1333`: A ticket must not include a brittle implementation file list unless a preserved prototype requires it.

3. **`review-breakdown`** — `BU-1320` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L100-101)`)
   - Trigger: publication has not been explicitly requested immediately
   - Outcome: the user reviews the breakdown before any ticket is actually published
   - Statement: Unless the user explicitly said to create or publish tickets immediately, the proposed breakdown is presented first, before publishing.
   - Stage-context attachments (1):
     - `BU-1321`: When confirming the breakdown, the skill asks only whether granularity, ownership, and blocking edges are correct, and does not ask the user to reconfirm decisions already made.

4. **`publish-tickets`** — `BU-1322` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L114-115)`)
   - Trigger: tickets and their parent epics are being published
   - Outcome: epics exist with real IDs before any child ticket that references them is created
   - Statement: Local epics are created first via the task tracker, so that child tickets can reference real epic IDs.
   - Stage-context attachments (5):
     - `BU-1323`: Tickets are created in dependency order, blockers first.
     - `BU-1324`: For an existing task, the skill updates it rather than creating a duplicate.
     - `BU-1325`: The task-tracker creation step is used when one approved logical outcome needs matching task records in several registered repositories, with repository-specific details then added via the task tracker.
     - `BU-1326`: Because the task tracker dependencies are repository-local, cross-repository blockers are represented by recording the counterpart repo and the task tracker ID in both descriptions or logs and stating the exact merge order, never by inventing a native dependency edge the task tracker cannot enforce across databases.
     - `BU-1327`: Newly published tasks are not marked `in_progress` by this skill; they remain `open` until dispatch or a worker starts them.

5. **`validate-published-graph`** — `BU-1328` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L166)`)
   - Trigger: publication has completed
   - Outcome: the dependency graph is checked to be free of cycles and fabricated cross-repo edges before being considered valid
   - Statement: After publishing, the skill confirms no circular or cross-repo pseudo-dependencies exist in the ticket graph.
   - Stage-context attachments (1):
     - `BU-1329`: Stale duplicate tickets are closed only with an explicit superseding task, via the task tracker.

6. **`report-dispatch-frontier`** — `BU-1330` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L179-180)`)
   - Trigger: publishing has completed and the frontier is being computed
   - Outcome: only tickets with no remaining blockers are reported as immediately dispatchable
   - Statement: The dispatch frontier reported after publishing consists of the tickets that have no unfinished blockers.
   - Stage-context attachments (2):
     - `BU-1331`: Recommended concurrency defaults to one worker per owning repository, unless the project explicitly supports more.
     - `BU-1332`: Dispatch does not happen unless the user asked to begin implementation.

### `treehouse-init`

- **Trigger:** (no stage candidates for this workflow value)
- **Outcome:** (no stage candidates for this workflow value)
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `(none)`'s.
- **Member stage count:** 0

**Workflow-level helpers** (`workflow=treehouse-init`, `stage=null` — deterministic
machinery referenced by the workflow as a whole, not one specific stage):

- `BU-0299` — Treehouse initialization is idempotent per repo: a repo that already has `treehouse.toml` is reported as already initialized rather than being re-initialized.

_No `stage`-rung records carry this `workflow` value — candidate has no_
_member stages (a `stage`/`stage-context`/`helper`-only cluster whose_
_checkpoint boundary, if any, was never classified as `stage` rung)._

### `triage`

- **Trigger:** an unlabeled issue enters triage
- **Outcome:** the named state is applied directly, bypassing the ordinary multi-step triage procedure
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `quick-override`'s.
- **Member stage count:** 8
- **Ordering note:** `triage` is graph-shaped (independent event-triggered
  entry points, not a single pipeline) — source/behavior_id order is used as
  the defensible default per the run-wide note above, not a proven chain.

**Workflow-level helpers** (`workflow=triage`, `stage=null` — deterministic
machinery referenced by the workflow as a whole, not one specific stage):

- `BU-1144` — A bare reference such as `#42` in a triage request is resolved to a specific issue or pull request according to the tracker config, not assumed to be one or the other.
- `BU-1182` — The `.out-of-scope/` knowledge base stores one file per rejected concept, not per issue — multiple issues requesting the same thing are grouped under one file.
- `BU-1184` — An `.out-of-scope/` file is named with a short, descriptive kebab-case concept name, recognizable enough that someone browsing the directory understands what was rejected without opening the file.
- `BU-1193` — When recording a rejection in `.out-of-scope/`, the triage skill appends the new issue to an existing matching file's prior-requests list if one already exists; otherwise it creates a new file with the concept name, decision, reason, and first prior request.

**Ordered member stages:**

1. **`operate-state-machine`** — `BU-1148` (`.agents/skills/triage/SKILL.md (SKILL.md L45)`)
   - Trigger: an unlabeled issue enters triage
   - Outcome: the issue is placed in the `needs-triage` state as its starting point
   - Statement: An unlabeled issue's default first state transition is to `needs-triage`.
   - Stage-context attachments (3):
     - `BU-1147`: If an issue's or PR's state roles conflict, the conflict is flagged and the maintainer is asked before any further action is taken.
     - `BU-1149`: An issue in `needs-info` automatically returns to `needs-triage` once the reporter replies.
     - `BU-1150`: The maintainer can override any state transition at any time, but a transition that looks unusual is flagged and confirmed with the maintainer before it proceeds.

2. **`surface-attention-queue`** — `BU-1151` (`.agents/skills/triage/SKILL.md (SKILL.md L58-62)`)
   - Trigger: the maintainer asks what needs triage attention
   - Outcome: three ordered buckets of items are presented, oldest first within each
   - Statement: When showing what needs attention, the triage skill presents three buckets — unlabeled items, items in `needs-triage`, and `needs-info` items with reporter activity since the last triage notes — ordered oldest first.
   - Stage-context attachments (2):
     - `BU-1152`: When pull requests are in scope for triage, the 'what needs attention' discovery buckets surface only external PRs — a collaborator's own in-flight PR is not included as triage work.
     - `BU-1153`: The external-PR-only discovery filter only limits what is surfaced automatically — a PR explicitly named by the maintainer is always triaged regardless of who authored it.

3. **`gather-context`** — `BU-1154` (`.agents/skills/triage/SKILL.md (SKILL.md L70)`)
   - Trigger: an issue or PR is being triaged
   - Outcome: a redundancy check against the existing codebase is performed and its search scope is reported
   - Statement: While gathering context on an issue or PR, the triage skill searches the codebase for an existing implementation of the requested behavior by domain concept (not just the request's wording), and records where it looked.
   - Stage-context attachments (9):
     - `BU-1155`: If the redundancy check finds that the requested behavior is already implemented, the disposition becomes `wontfix` (already-implemented) rather than proceeding through the rest of ordinary triage.
     - `BU-1156`: While gathering context, the triage skill also checks for prior rejection: it reads the `.out-of-scope/` knowledge base and surfaces any entry that resembles the current request.
     - `BU-1174`: When resuming triage on an issue or PR that already has prior triage notes, the triage skill reads them, checks whether the reporter has answered any outstanding questions, and presents an updated picture before continuing rather than re-asking questions that are already resolved.
     - `BU-1187`: During triage's gather-context step, the triage skill reads all files in `.out-of-scope/` when evaluating a new issue.
     - `BU-1188`: A new issue is matched against `.out-of-scope/` entries by concept similarity, not by keyword matching.
     - `BU-1189`: When a new issue matches an existing `.out-of-scope/` entry, the triage skill surfaces the prior rejection and its reason to the maintainer and asks whether they still feel the same way, rather than silently re-applying the old rejection.
     - `BU-1190`: The maintainer's response to a surfaced `.out-of-scope/` match branches three ways: confirm (the new issue is appended to the file's prior-requests list and closed), reconsider (the file is deleted or updated and the issue proceeds through normal triage), or disagree (treated as related but distinct, proceeding through normal triage).
     - `BU-1194`: If the maintainer reconsiders a previously rejected concept, the triage skill deletes the corresponding `.out-of-scope/` file.
     - `BU-1195`: The triage skill does not reopen old issues that were closed under a since-reconsidered rejection — they remain historical records; only the new issue that triggered the reconsideration proceeds through normal triage.

4. **`recommend-and-wait`** — `BU-1157` (`.agents/skills/triage/SKILL.md (SKILL.md L72)`)
   - Trigger: the triage skill has presented its recommendation to the maintainer
   - Outcome: no further triage action is taken until the maintainer responds
   - Statement: After presenting the category/state recommendation and a codebase summary (including whether the request is already implemented), the triage skill waits for the maintainer's direction before proceeding further.

5. **`verify-claim`** — `BU-1158` (`.agents/skills/triage/SKILL.md (SKILL.md L74)`)
   - Trigger: an issue or PR reaches the verify step
   - Outcome: the underlying claim (bug report or PR's stated effect) is actually exercised, not just read
   - Statement: Before any grilling, the triage skill verifies the claim holds up: for a bug it reproduces the issue from the reporter's steps, and for a PR it checks out the diff and confirms it does what it claims by running the relevant tests or commands.
   - Stage-context attachments (1):
     - `BU-1159`: Verification is reported as one of three outcomes — confirmed (with the code path), failed, or insufficient detail — and insufficient detail is treated as a strong signal to move the item to `needs-info` rather than accepted as a clean result.

6. **`grill-if-needed`** — `BU-1160` (`.agents/skills/triage/SKILL.md (SKILL.md L76)`)
   - Trigger: an issue or PR's requirements are underspecified enough to need grilling
   - Outcome: the request is progressively sharpened and decisions are recorded inline rather than left implicit
   - Statement: When a request needs fleshing out, the triage skill runs a structured grilling session — question by question — to sharpen the request's domain terms, recording decisions inline as they land.
   - Stage-context attachments (1):
     - `BU-1172`: Everything resolved during a grilling session is captured under the needs-info template's "established so far" section so that work is not lost.

7. **`apply-outcome`** — `BU-1161` (`.agents/skills/triage/SKILL.md (SKILL.md L79)`)
   - Trigger: an item's disposition is decided to be `ready-for-agent`
   - Outcome: an agent brief comment is posted
   - Statement: When an item's outcome is `ready-for-agent`, the triage skill posts an agent brief as a comment on the issue or PR.
   - Stage-context attachments (19):
     - `BU-1162`: When an item's outcome is `ready-for-human`, the triage skill posts a brief with the same structure as an agent brief, but additionally states why the work can't be delegated to an agent (judgment calls, external access, design decisions, manual testing).
     - `BU-1163`: When an item's outcome is `needs-info`, the triage skill posts triage notes using the needs-info template.
     - `BU-1164`: When an item is closed `wontfix` because the requested change already exists in the codebase, the triage skill points to where it already lives and does not write to `.out-of-scope/` — that knowledge base is only for rejected requests, not built ones.
     - `BU-1165`: When a bug report is closed `wontfix` as rejected, the triage skill posts a polite explanation and then closes it.
     - `BU-1166`: When an enhancement request is closed `wontfix` as rejected, the triage skill writes an entry to `.out-of-scope/`, links to it from a closing comment, and then closes the item.
     - `BU-1167`: When an item's outcome is `needs-triage`, the triage skill applies that role, with an optional comment if there is partial progress to record.
     - `BU-1173`: Questions posted in triage notes must be specific and actionable, not a generic ask like "please provide more info".
     - `BU-1175`: An agent brief is the authoritative specification an AFK agent works from when an item moves to `ready-for-agent`; the original issue/PR body and discussion are context only, not the operative contract.
     - `BU-1176`: An agent brief's scope differs by surface under the same principles: for an issue it covers building the change from nothing, and for a PR it covers what's left to do to the existing diff — finishing it, closing gaps, addressing review points.
     - `BU-1177`: An agent brief must describe interfaces, types, and behavioral contracts (naming specific types, function signatures, or config shapes), and must not reference file paths, line numbers, or assume the current implementation structure will persist — because the codebase may change before the brief is picked up.
     - `BU-1178`: An agent brief describes what the system should do, not how to implement it — the agent explores the codebase fresh and makes its own implementation decisions.
     - `BU-1179`: Every agent brief must have concrete, testable acceptance criteria, with each criterion independently verifiable.
     - `BU-1180`: An agent brief must state what is out of scope, to prevent the agent from gold-plating or making assumptions about adjacent features.
     - `BU-1181`: For a PR-targeted agent brief, "current behavior" describes the state of the existing diff, and the brief asks the agent to finish or fix that diff rather than build the change from scratch.
     - `BU-1183`: An `.out-of-scope/` file is written in a relaxed, readable style — more like a short design document, using paragraphs, code samples, and examples — rather than a terse database-entry format.
     - `BU-1185`: The reason recorded in an `.out-of-scope/` file must be substantive — referencing project scope/philosophy, technical constraints, or a strategic decision — not a bare "we don't want this".
     - `BU-1186`: The reason recorded in an `.out-of-scope/` file must be durable — a temporary-circumstance excuse ("we're too busy right now") is a deferral, not a real rejection, and should not be recorded as one.
     - `BU-1191`: The triage skill writes to `.out-of-scope/` only when an enhancement (not a bug) is rejected as `wontfix`; this applies equally to a rejected enhancement PR, which is recorded so the same request doesn't return as fresh code.
     - `BU-1192`: The triage skill never writes to `.out-of-scope/` when an item is closed `wontfix` because it is already implemented — that would poison the deduplication checks with a false rejection; instead the closing comment points to where the feature already lives.

8. **`quick-override`** — `BU-1168` (`.agents/skills/triage/SKILL.md (SKILL.md L90)`)
   - Trigger: the maintainer gives a direct state-change instruction
   - Outcome: the named state is applied directly, bypassing the ordinary multi-step triage procedure
   - Statement: When the maintainer directly names a target state for an item (e.g. asks to move it to `ready-for-agent`), the triage skill trusts that instruction and applies the state directly rather than re-running the full recommend/verify/grill procedure.
   - Stage-context attachments (3):
     - `BU-1169`: Even in the quick-override path, the triage skill confirms the specific action it is about to take (role change, comment, close) with the maintainer before acting.
     - `BU-1170`: The quick state override path skips grilling entirely.
     - `BU-1171`: If the quick override moves an item to `ready-for-agent` without a grilling session having been run, the triage skill asks the maintainer whether they want an agent brief written.

### `troubleshoot-failure`

- **Trigger:** a failure is not covered by existing documentation
- **Outcome:** the gap is escalated as a well-formed task tracker task rather than left unresolved or guessed at
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `escalate-undocumented-gap`'s.
- **Member stage count:** 1

**Ordered member stages:**

1. **`escalate-undocumented-gap`** — `BU-0192` (`docs/troubleshooting.md (docs/troubleshooting.md L242-244)`)
   - Trigger: a failure is not covered by existing documentation
   - Outcome: the gap is escalated as a well-formed task tracker task rather than left unresolved or guessed at
   - Statement: When documentation does not cover an observed failure, the `sergeant-help` skill is used to search the docs, then the task tracker task is created containing the exact reproduction, expected behavior, preserved state, and acceptance criteria.

### `validation-pipeline-gate`

- **Trigger:** a dispatched worker reaches readiness
- **Outcome:** the run only advances through an explicit pipeline-automation tool, never spontaneously
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `monitor-active-run`'s.
- **Member stage count:** 14
- **Ordering note:** `validation-pipeline-gate` is graph-shaped (independent event-triggered
  entry points, not a single pipeline) — source/behavior_id order is used as
  the defensible default per the run-wide note above, not a proven chain.

**Workflow-level helpers** (`workflow=validation-pipeline-gate`, `stage=null` — deterministic
machinery referenced by the workflow as a whole, not one specific stage):

- `BU-0396` — The validation worker records its validation pipeline exit code verbatim as exited:<status> in the durable validation_status file and propagates the same exit code as its own process exit status.
- `BU-1259` — Exit codes for the pipeline-automation tool commands are `0` for success, no-op, or normal decision gates, `1` for `failed` or `cancelled` final outcomes, and `2` for bad usage.

**Ordered member stages:**

1. **`launch-validation-run`** — `BU-0042` (`AGENTS.md (AGENTS.md L150-157)`)
   - Trigger: a dispatched worker reaches readiness
   - Outcome: exactly one validation-only boundary runs, in a split pane, with redundant stages skipped by default
   - Statement: After readiness, the coordinator uses the project-validation step to launch the single validation-only boundary in a split pane of the worker's tmux window; its default medium profile skips the redundant validation pipeline `review` and `document` stages.
   - Stage-context attachments (35):
     - `BU-0043`: Remediation that changes HEAD still requires independent rereview before updating the readiness marker, but must not trigger repeated validation pipeline review cycles.
     - `BU-0079`: The validation pipeline is used as a final shipping gate, not an implementation loop: implementation, focused repository-native tests, lint, and independent review must be complete before starting it.
     - `BU-0080`: Before starting the validation pipeline run: finish and commit on a feature branch, ensure the validation pipeline is healthy, and check the validation pipeline for an already-active matching run to reattach to rather than creating a duplicate.
     - `BU-0081`: `--yes` is never used when starting the validation pipeline; `--skip=<steps>` is used only for stages already proven irrelevant, since skipping is not a substitute for checks that were never performed.
     - `BU-0082`: Routine dispatched workers do not invoke the validation pipeline for ordinary completion, prototypes, investigations, documentation drafts, intermediate commits, or remediation loops; the coordinator starts a single run only after the implementation branch is committed and native validation is complete.
     - `BU-0161`: The project-validation step splits the worker's existing tmux window, renames it to `validation-<repo>-<task>`, and runs the validation pipeline interactively in the new coordinator-owned pane with the canonical intent; it never uses `--yes`, and its default medium profile skips `review`/`document` (already covered by required independent reviews and readiness evidence) unless an explicit `--skip` replaces the default list.
     - `BU-0170`: Validation is treated as validation-only: each actionable finding is routed into separate, deduplicated owning-repository task tracker work, source is never modified inside the retained validation run, and high-risk findings are escalated rather than approved.
     - `BU-0181`: If shared daemon credentials cannot access one repository, the global GitHub account is never switched while unrelated runs are active; an approved repo-scoped method, waiting, or an explicit manual-shipping override is used instead.
     - `BU-0182`: A one-shot `GH_TOKEN` for `gh` and a one-shot credential helper for Git are preferred, and the global GitHub account is never switched while other workers may invoke GitHub operations.
     - `BU-0376`: For a worker whose recorded status is not done or needs_input (i.e. it is expected to still be running), validation launch requires the recorded worker pane identity to still match a live pane; a done or needs_input worker is exempt from this check since its pane is expected to already be dead.
     - `BU-0382`: The --skip value is interpolated unquoted into the constructed validation-worker launch command deliberately, because printf %q would escape a comma (a brace-expansion special character) in a way that would not match the literally-recorded pane_start_command later compared against it; SKIP is independently constrained to a safe [a-z]+(,[a-z]+)* pattern beforehand.
     - `BU-0383`: If splitting a new validation pane beside the worker's own pane fails and the worker's own status is done (its pane expectedly already dead), the project-validation step falls back to opening a new tmux window instead, rather than failing the validation launch outright.
     - `BU-1197`: The validation pipeline gate validates code changes through a pipeline (intent, rebase, review, test, document, lint, push, PR, CI) before they reach the configured push target, driven through the validation pipeline command family which prints machine-readable TOON to stdout and progress to stderr.
     - `BU-1198`: When the user invokes `/no-mistakes`, the agent must report the pipeline's outcome at the end.
     - `BU-1199`: If the user asks for something specific (e.g. "skip the lint step"), the agent must translate that request into the matching pipeline-automation tool flags itself, such as `--skip=lint`.
     - `BU-1200`: In validate-only mode (bare `/no-mistakes`), the user's code changes are already committed; the agent validates them as-is and reports the outcome, without doing any task work first.
     - `BU-1201`: In task-first mode, before changing or committing anything the agent must inspect `git status`, preserve unrelated pre-existing uncommitted changes, and when committing include only the changes belonging to the user's task.
     - `BU-1202`: In task-first mode the agent must commit its work on a feature branch; if the user is currently on the repository's default branch, the agent must create a feature branch first, because the gate validates committed history on a non-default branch.
     - `BU-1203`: In task-first mode, once the work is committed the agent validates it by passing the user's task text as `--intent`, enriched with the decisions and tradeoffs made while doing the work.
     - `BU-1204`: The gate validates committed history, not the uncommitted working tree, so the work being validated must already be committed on a branch.
     - `BU-1205`: The user must be on a feature branch, not the repository's default branch, for the validation pipeline run to proceed.
     - `BU-1206`: The repository must already be initialized with the validation pipeline before a run can proceed.
     - `BU-1207`: The daemon must have a runnable configured pipeline agent (a supported native agent binary, the `agent: cursor` ACP alias, or an explicit `acp:<target>` through `acpx`); the invoking agent is the AXI driver, not an implicit pipeline-agent backend, and if none is available the run fails before its first step, with the validation pipeline reporting the configuration problem.
     - `BU-1208`: If any precondition is not met, the pipeline-automation tool returns an `error:` with the exact command needed to fix it, which the agent must read and act on (commit the work, or create a branch).
     - `BU-1209`: If the repository is not initialized, the agent must run the validation pipeline first.
     - `BU-1210`: If the validation pipeline command itself is missing or misbehaving, the validation pipeline reports what is wrong.
     - `BU-1211`: Before starting a new run, the agent must run the validation pipeline (the home view).
     - `BU-1212`: If the home view shows an active run on the current branch, the agent inspects it with the validation pipeline.
     - `BU-1213`: If the active run is parked at a gate, the agent drives it with the validation pipeline.
     - `BU-1214`: The agent may reattach an in-flight run by re-running the validation pipeline when it still matches the current `HEAD`, either as the submitted head or as the current pipeline head.
     - `BU-1215`: The validation pipeline is only for discarding a run before starting over; it is a between-runs action and must never be used to take over or bypass a gate while a run is still going.
     - `BU-1216`: If the home view shows an active run on another branch, the agent leaves that run alone and starts validation for its own current branch with the validation pipeline.
     - `BU-1217`: Starting a run requires `--intent`, describing what the user set out to accomplish (not a description of the diff), passed verbatim from what the agent knows from the conversation rather than left for the validation pipeline to infer from local agent transcripts.
     - `BU-1218`: The agent must err on the side of completeness rather than brevity in `--intent`, capturing the user's goal, the specific decisions and tradeoffs made, constraints ruled in or out, and anything explicitly requested that might otherwise look surprising in the diff, because the review step uses `--intent` to tell a deliberate decision apart from a mistake.
     - `BU-1219`: Starting the run with the validation pipeline blocks until the first decision point or the run's end.

2. **`drive-gate-findings`** — `BU-0084` (`README.md (README.md L283-287)`)
   - Trigger: the validation pipeline gate presents findings of a given disposition
   - Outcome: each disposition is handled by its own fixed rule, with ask-user always requiring a human decision
   - Statement: At each validation pipeline gate: `auto-fix` findings are authorized selectively via the pipeline-automation tool after review; `ask-user` findings are relayed to the user and never approved/fixed/skipped autonomously; `no-op` findings are informational and the gate is approved.
   - Stage-context attachments (27):
     - `BU-0083`: Running or responding to the pipeline-automation tool blocks while work is active — a quiet step is not a stall — and progress is checked against the validation pipeline's status without issuing duplicate run commands.
     - `BU-0085`: While the validation pipeline run is active: the pipeline-owned worktree is never edited, the run is never aborted or rerun to escape a gate, and all pipeline-created commits are preserved; abort is used only when intentionally discarding the entire run.
     - `BU-0086`: Driving stops at `checks-passed`; the PR is ready and the validation pipeline monitors it in the background, so the coordinator does not poll or wait for merge.
     - `BU-0180`: An `auto-fix` finding in Sergeant's validation-only workflow is never authorized as an in-run fix; it is instead routed to separate owning-repository task tracker remediation, and a retained gate is never edited, aborted, or restarted to bypass the finding.
     - `BU-0310`: The worker never approves a validation gate, routes a finding, or remediates validation output from its own pane; the Sergeant coordinator owns every validation pipeline gate and finding, and the worker resumes only through an explicit Sergeant response tied to that owning work.
     - `BU-1226`: When the run output contains a `gate:` object, the pipeline is waiting on the agent; the agent reads its `findings` table, where each finding has an `id`, `severity`, `file`, `description`, and an `action` classifying it.
     - `BU-1227`: A finding classified `auto-fix` is mechanical and low-risk, so the agent may authorize the fix on its own judgment.
     - `BU-1228`: A finding classified `no-op` is informational only; there is nothing to do about it.
     - `BU-1229`: A finding classified `ask-user` challenges the user's deliberate intent or touches product behavior, so it is a decision only the user can make.
     - `BU-1230`: Review auto-fix is disabled by default (`auto_fix.review: 0`; a repo or global `auto_fix.review > 0` override re-enables it), so blocking and ask-user review findings park for the agent's/user's decision rather than being silently self-fixed, while other steps such as test and lint may still auto-fix within the pipeline and re-run before ever gating.
     - `BU-1231`: The agent may respond the validation pipeline to accept a gated step as-is and continue.
     - `BU-1232`: The agent may respond the validation pipeline to have the pipeline fix specific findings and continue.
     - `BU-1233`: The agent may respond the validation pipeline to skip the current step.
     - `BU-1234`: While a run is active, the agent must never fix findings by editing the code itself, because the pipeline owns both the findings and the fixes; the agent's job at a gate is to decide and respond, and `--action fix` has the pipeline apply the fix and re-review the result.
     - `BU-1235`: For the same reason, while a run is active the agent must not `abort` or `rerun` to go fix a finding itself, even a real bug in its own code, because that discards the pipeline's in-flight work and forces a full re-validation; `abort` and `rerun` are for between runs only, never to circumvent a gate.
     - `BU-1236`: Each `respond` blocks until the next `gate:`, `checks-passed` decision point, or final outcome.
     - `BU-1237`: `--add-finding '<json>'` (with `--action fix`) folds a finding the agent spotted itself, that the pipeline did not surface, into the fix round.
     - `BU-1238`: `--step <name>` responds to a specific step instead of the one currently awaiting approval; it is rarely needed and omitted to answer the active gate.
     - `BU-1239`: `checks-passed` means the change is validated and CI is green but the PR is not merged yet; the agent is done driving the pipeline and must not wait for the merge, instead telling the user the PR is ready for review and merge (the PR link is in the `help` line), while the validation pipeline keeps monitoring the PR in the background until it is merged, closed, or its configured idle timeout elapses.
     - `BU-1240`: `passed` means the changes cleared the gate and the PR was merged or closed.
     - `BU-1250`: The CI step deliberately keeps watching the PR after checks pass, so the pipeline-automation tool returns `checks-passed` the moment checks are green rather than blocking on the human merge, and the agent must never poll or re-run waiting for the merge.
     - `BU-1253`: On a successful outcome, the agent closes the loop with the user by summarizing what the pipeline validated and found in a concise, easily readable format, and if the output includes a `fixes` table, explicitly acknowledges and lists each fix the pipeline made that the original change missed.
     - `BU-1254`: A gate whose findings are all `auto-fix` or `no-op` is safe for the agent to drive on its own judgment, but a finding marked `ask-user` must not be approved, fixed, or skipped by the agent on its own; instead the agent stops and brings it to the user before responding.
     - `BU-1255`: The agent relays each `ask-user` finding to the user exactly as the pipeline wrote it — its `id`, `file`, and full `description` verbatim — without paraphrasing, summarizing away detail, or pre-judging the answer.
     - `BU-1256`: The agent asks the user how they want to proceed, then translates their decision into the matching `respond` call: `--action fix` (with their guidance passed through `--instructions`), `--action approve`, or `--action skip`.
     - `BU-1257`: The one exception to escalating `ask-user` findings is `--yes`: the user's standing consent to drive every gate unattended, under which the agent resolves `ask-user` findings automatically instead of stopping to ask.
     - `BU-1258`: With clear consent to drive automatically, the agent passes `--yes` to the pipeline-automation tool, which treats every actionable finding (`auto-fix` and `ask-user` alike) as consent to fix, selects every current finding for one fix round, accepts the resulting fix review, and approves gates with only `no-op` findings; it is used only when the user has asked to drive the whole run without checking back.

3. **`recover-from-interrupted-run`** — `BU-0087` (`README.md (README.md L295-298)`)
   - Trigger: the validation pipeline run outcome is failed or cancelled
   - Outcome: recovery follows exactly one of the three named branch_sync-driven paths
   - Statement: If the validation pipeline outcome is `failed` or `cancelled`, `branch_sync` state is inspected first and handled by exactly one of three named responses: `sync` runs the validation pipeline, `continue_active_run` keeps driving the reported run, `recover_custody` uses the validation pipeline.
   - Stage-context attachments (12):
     - `BU-0088`: A reset, stash, force-push, or branch replacement is never improvised around a blocked sync state.
     - `BU-1241`: On `failed` or `cancelled`, the agent reads the output, fixes whatever it points at (a failing test, a lint error, a skipped finding), commits the fix on the same feature branch, and drives the pipeline again with a fresh pipeline-automation tool or the validation pipeline; this is correct only after a terminal outcome, never mid-run to circumvent a gate, and the agent must not leave the user at a `failed` outcome without either retrying or explaining what blocks it.
     - `BU-1242`: Before any post-pipeline local commit or fresh run, the agent must read the structured `branch_sync` object returned by AXI home, status, or a drive result.
     - `BU-1243`: Only when `branch_sync.next_action.code` is `sync` does the agent run the validation pipeline first.
     - `BU-1244`: The guarded sync may be a strict fast-forward or a content-equivalent diverged advance that anchors the pre-sync head before moving the branch with reset semantics, but genuine divergence stays blocked.
     - `BU-1245`: If `next_action.code` is `continue_active_run`, the pipeline still owns the branch: the agent runs the reported command, keeps driving the active run, and does not make local follow-up commits.
     - `BU-1246`: When `next_action.code` is `recover_custody`, a terminal run left unpublished pipeline commits preserved in the local gate: the agent runs the validation pipeline to return custody and fast-forward to the preserved head, or the validation pipeline to resume validating it instead.
     - `BU-1247`: A dirty or diverged worktree makes recovery refuse with explicit choices; `--keep-local` keeps the current head while the preserved commits stay anchored under `refs/no-mistakes/recover/<run>`.
     - `BU-1248`: If synchronization is blocked, the agent processes that structured state instead of improvising reset, stash, merge, rebase, force, or branch replacement.
     - `BU-1249`: After synchronization, the agent commits the follow-up on top and re-runs the validation pipeline with the original user intent, which preserves every prior gate-fix commit regardless of its configured subject.
     - `BU-1251`: A PR that falls behind the default branch or hits a merge conflict after checks pass needs no command from the agent and must never be hand-rebased: when the CI monitor sees an actual conflict it rebases onto the base, resolves it, and re-pushes the branch itself; a PR that is merely behind but still clean needs nothing either, since the platform merges it.
     - `BU-1252`: The one exception is when the CI monitor is no longer running (the PR was closed, the run was aborted or superseded, it idle-timed-out, or its auto-fix attempts were exhausted): the agent recovers with the validation pipeline, which cancels the stale monitor and re-runs the full pipeline including a deterministic rebase step; the agent must not reach for the validation pipeline to refresh a still-active PR, since after `checks-passed` it reattaches to the running monitor (HEAD unchanged) without rebasing.

4. **`declare-readiness`** — `BU-0160` (`docs/using-sergeant.md (docs/using-sergeant.md L312-316)`)
   - Trigger: native validation and independent reviews all pass
   - Outcome: readiness is durably recorded with intent/head/review evidence before the coordinator is notified, and the worker itself never invokes the validation pipeline
   - Statement: After native validation and independent reviews report zero blockers, the worker writes `.sergeant-validation-ready` with the recorded `intent_revision`, current `head_sha`, and passed values for `standards_review`, `spec_review`, and `readiness_review`, then notifies the coordinator; the worker must not run the validation pipeline.
   - Stage-context attachments (4):
     - `BU-0309`: Validation requires a clean worktree at the committed HEAD; readiness evidence is never created from an uncommitted diff, and the branch is committed before readiness is published.
     - `BU-0373`: Validation only launches if all three review axes recorded on the validation-ready marker (standards, spec, readiness) are exactly "passed"; any other value fails with a message naming the specific axis and its actual recorded value.
     - `BU-0374`: The worker's code tree must be clean (per _sgt_worktree_is_validation_clean) before a validation snapshot is taken; a dirty tree fails validation launch outright.
     - `BU-0914`: A worktree is considered 'validation clean' only if it has no staged or unstaged diffs against HEAD and no untracked files other than Sergeant's own .sergeant-* control files.

5. **`acquire-launch-reservation`** — `BU-0162` (`docs/using-sergeant.md (docs/using-sergeant.md L328-331)`)
   - Trigger: the project-validation step is about to clone a checkout or publish launch state
   - Outcome: exactly one validation launch proceeds per task/repository pair at a time, with concurrent attempts failing closed
   - Statement: Before cloning the validation checkout or publishing launch state, the coordinator acquires an identity-checked validation-launch reservation for that task/repository pair; concurrent launches fail closed until the recorded owner exits or stale-ownership recovery proves the reservation is abandoned.
   - Stage-context attachments (2):
     - `BU-0377`: The validation launch lock can be recovered from a stale prior owner only if that owner's PID is a genuine number, its recorded coordinator and purpose exactly match the current claimant's, and — critically — if that PID is still alive, its process start time must differ from the one recorded at lock time (proving PID reuse, not the same still-running holder) before the lock is treated as abandoned.
     - `BU-0387`: Acquiring the validation launch lock uses an atomic hard-link (ln) creation, which can only succeed for one caller; on failure, exactly one stale-lock recovery attempt is made before giving up, so two competing launches for the same task/repo can never both believe they hold the lock.

6. **`choose-intent-transport`** — `BU-0163` (`docs/using-sergeant.md (docs/using-sergeant.md L335-338)`)
   - Trigger: the project-validation step is about to create a validation run
   - Outcome: the default transport path never exposes intent content via process argv
   - Statement: Canonical intent must not appear in process arguments, where any local process can read it from `ps` or `/proc/<pid>/cmdline`; before creating a validation run, the project-validation step probes the validation pipeline and requires `--intent-file`, which delivers the intent through a path instead of argv.
   - Stage-context attachments (9):
     - `BU-0164`: When the installed validation pipeline does not offer `--intent-file`, the launch fails closed and names the required capability, the observed version, the observed flag surface, and the operator's options; no run, marker, or state change is created.
     - `BU-0165`: `--allow-argv-intent` consents, for that invocation only, to delivering the intent through `--intent`, accepting the exposure; consent is a flag rather than an environment variable so it cannot be exported once and silently reapplied to later runs.
     - `BU-0166`: The transport actually launched is recorded twice — `validation_intent_transport` for the current run (cleared on retry-reset) and an append-only owner-only `validation_transport.log` of every committed launch — and the validation worker re-checks the recorded transport against the build that will actually run, so the validation pipeline binary replaced between launch and run can neither downgrade the private transport into argv nor invoke a flag that build rejects.
     - `BU-0324`: The set of flags the validation pipeline's own `run` subcommand accepts is discovered by parsing its own --help output, not by inferring capability from a version number.
     - `BU-0325`: The intent transport is resolved by preferring the private --intent-file flag when the installed validation pipeline build supports it; the argv --intent flag is only selected when --intent-file is unavailable AND the operator has explicitly consented (allow_argv=true); otherwise resolution fails.
     - `BU-0375`: The intent transport (private intent-file vs. consented argv) is resolved and validated before any validation run, marker, or state change exists, specifically so that an incompatible validation pipeline build cannot record a failed run.
     - `BU-0384`: The intent transport actually used for a validation run (intent-file or consented argv) is recorded to an append-only audit log as the last publication step of a committed launch; the comment notes over-recording is the conservative direction for this privacy-relevant decision even though a failure at this step still rolls the whole launch back.
     - `BU-0389`: The validation worker re-checks the coordinator's recorded intent transport decision against the actually-installed validation pipeline build's real capability, so a binary swapped between launch time and run time can neither downgrade the private intent-file transport into argv nor invoke a flag the installed build rejects; the recorded decision is honored exactly and never re-optimized here.
     - `BU-0394`: With the intent-file transport only the intent's file PATH reaches the validation pipeline's argv, so intent content never appears in ps or /proc/<pid>/cmdline; the argv transport is only reachable at all because the coordinator passed --allow-argv-intent, and it does expose the full intent content through those same surfaces.

7. **`transfer-ownership`** — `BU-0167` (`docs/using-sergeant.md (docs/using-sergeant.md L359-374)`)
   - Trigger: a coordinator other than the original dispatcher needs to claim validation ownership
   - Outcome: ownership transfer requires cryptographic-strength process-ancestry proof of pane identity, and never displaces a live legitimate owner
   - Statement: Validation ownership belongs to the dispatching tmux pane; a coordinator in any other pane must claim ownership explicitly, and a claim is accepted only when the claiming pane proves it really runs inside the pane it names by walking its own process ancestry — a caller that merely exports `TMUX_PANE` cannot satisfy this — and the prior owner must be takeover-eligible (dead/absent pane, mismatched recorded identity, or explicit release) with a live unreleased owner never displaced.
   - Stage-context attachments (4):
     - `BU-0168`: Every ownership transfer appends the timestamp, reason, repository, prior and new pane, and both identity tuples to an owner-only `coordinator_handover.log`; a release is consumed by the claim that uses it, so it cannot be replayed later by a third pane.
     - `BU-0369`: Accepting a coordinator ownership handover from a previously unseen pane requires proving that this process genuinely lives inside that pane's process tree (walking up to 64 ppid hops); merely exporting the TMUX_PANE variable cannot satisfy this proof.
     - `BU-0370`: Claiming coordinator ownership determines its handover reason by checking, in order: whether the prior owner explicitly released ownership (released-by-owner); otherwise refusing outright if the prior owner's pane is still live and has not released; otherwise recording whether the prior pane still exists but was recycled to a different identity, or is simply gone.
     - `BU-0371`: --release-ownership is an ownership-only operation: it never inspects worker readiness and never launches a validation run, and it can only be performed by the pane currently recorded as the owner (both its tmux pane ID and its recorded identity must match exactly).

8. **`rollback-on-launch-failure`** — `BU-0169` (`docs/using-sergeant.md (docs/using-sergeant.md L384-390)`)
   - Trigger: a validation launch fails before commit
   - Outcome: rollback is scoped strictly to provably-owned artifacts of this invocation, never touching state it cannot prove it created
   - Statement: If launch fails before the validation child commits the release, Sergeant rolls back only the checkout, pane, temp files, and fleet-state markers that the current invocation both created and can still prove it owns, preserving preexisting state, reused panes, dangling paths, and concurrent replacements; after the recorded pane and process group have fully exited, rerunning the project-validation step safely resets only identity-matched finished state and retries.
   - Stage-context attachments (2):
     - `BU-0385`: Rollback of a failed or aborted validation launch removes an owned path (or restores a backed-up prior window/stage state) only if that path's captured identity (device+inode+birthtime, plus content checksum for files) still matches what was recorded when this launch owned it, so a path that was replaced by something else in the meantime is left untouched rather than deleted or overwritten.
     - `BU-0386`: Every durable state write during validation launch (_validation_write_owned) follows the same pattern: create a private temp candidate file, verify its identity is unchanged immediately after writing content, hard-link it into its final path, and record ownership of each intermediate path at each step, so the launch can distinguish and reliably roll back exactly what it created.

9. **`verify-intent-consistency`** — `BU-0326` (`bin/_sgt-intent.sh (bin/_sgt-intent.sh L112-127)`)
   - Trigger: the project-validation step checks whether the canonical intent revision matches before launching
   - Outcome: any divergence between the three intent copies, or between a recorded revision and the file's real hash, blocks validation rather than validating a possibly-stale or inconsistent intent
   - Statement: A coordinator-owned validation run only proceeds if the fleet-level, repo-state-level, and worktree-level copies of .sergeant-intent.md are byte-identical to each other AND their recorded revision hashes agree AND that revision re-verifies against the fleet copy's actual current content.
   - Stage-context attachments (3):
     - `BU-0372`: Before proceeding with any validation launch, the canonical intent revision recorded at the fleet, repo-state, and worktree levels must all match (via _sgt_intent_revision_matches); a mismatch is fatal, requiring an audited human decision or a new revision rather than proceeding on stale or divergent intent.
     - `BU-0388`: The validation worker refuses to proceed unless the canonical validation intent's own current revision hash exactly matches the revision it was invoked with, before doing anything else.
     - `BU-0393`: Immediately before invoking the validation pipeline, the validation worker re-computes the canonical intent's revision hash and fails (recording exited:2) if it no longer matches the expected revision, catching a content change made after the initial startup check rather than validating against it.

10. **`reset-retryable-state`** — `BU-0378` (`bin/sgt-validate (bin/sgt-validate L630-641)`)
   - Trigger: the project-validation step is retried after a prior validation exit
   - Outcome: a retry can never reset state while genuinely live validation processes, primary or unverified detached descendants, remain running
   - Statement: Resetting retryable validation state after a prior exit refuses to proceed if the recorded owner PID is still alive with the same recorded start time and process group (validation processes are still genuinely running), or if pgrep finds any live descendant process still in the recorded process group even after the primary PID is confirmed dead.
   - Stage-context attachments (1):
     - `BU-0379`: Before removing the isolated validation code snapshot during a retry reset, the project-validation step requires lsof to be installed and uses it to verify no process still has any file inside the snapshot open; any such process aborts the reset with the offending PIDs named.

11. **`create-isolated-snapshot`** — `BU-0380` (`bin/sgt-validate (bin/sgt-validate L833-836)`)
   - Trigger: the project-validation step creates the isolated validation snapshot
   - Outcome: the code actually validated is provably the exact reviewed commit, never a snapshot that silently drifted during creation
   - Statement: The isolated validation code snapshot is created as a --shared --no-checkout clone of the source worktree's root, then hard-checked-out to exactly the reviewed HEAD; the launch fails outright if the resulting HEAD or tree cleanliness does not match what was reviewed.
   - Stage-context attachments (2):
     - `BU-0381`: A validation checkout owner token (combining the lock purpose, lock identity, and the reviewed HEAD) is written into the isolated snapshot's own .git directory immediately after creation, binding that specific checkout to this specific launch.
     - `BU-0392`: Immediately before running the validation pipeline, the validation worker verifies the isolated snapshot's current HEAD still matches the expected reviewed HEAD and that the snapshot is still validation-clean; either mismatch is a fatal error.

12. **`check-coordinator-liveness`** — `BU-0390` (`bin/sgt-validation-worker (bin/sgt-validation-worker L73-82)`)
   - Trigger: the validation worker polls whether its coordinator is still alive
   - Outcome: a reused PID is never mistaken for the still-live original coordinator
   - Statement: The validation worker treats the coordinator as alive only if the validation-launch lock file exists (and is not a symlink), its recorded pid responds to kill -0, and that pid's current process start time still matches what was recorded at lock-acquisition time.
   - Stage-context attachments (1):
     - `BU-0395`: The validation worker waits for the coordinator's validation-launch lock to be released before running the validation pipeline, but if the coordinator process is found no longer alive while the lock file still exists, it fails rather than proceeding, unless the lock has genuinely already been removed by the time it checks.

13. **`publish-worker-readiness-handshake`** — `BU-0391` (`bin/sgt-validation-worker (bin/sgt-validation-worker L91-103)`)
   - Trigger: the validation worker is ready to publish its readiness handshake
   - Outcome: the readiness handshake cannot be published, or later replayed by an unrelated process, without matching this exact revision+pane+pid+start-time tuple
   - Statement: The validation worker requires a live TMUX_PANE and a resolvable process start time for itself before publishing its own readiness handshake, and requires the coordinator to still be alive at that moment; the handshake value binds together the expected intent revision, the pane, the child PID, and the child's process start time.

14. **`monitor-active-run`** — `BU-1222` (`.agents/skills/no-mistakes/SKILL.md (SKILL.md L107-110)`)
   - Trigger: the run reaches a gate or a long-running step
   - Outcome: the run only advances through an explicit pipeline-automation tool, never spontaneously
   - Statement: A long-running call is working, not stalled, and may be backgrounded if the harness needs to, but the run never advances past a gate on its own; the agent must read every return, respond on a `gate:`, and loop until an `outcome:` is reached, never idle-waiting for the run to move forward by itself.
   - Stage-context attachments (4):
     - `BU-1220`: The pipeline-automation tool and every pipeline-automation tool block synchronously, and the review, test, and CI steps can each take several minutes, so a single call may not return for a while; this is normal, and the agent must allow a long timeout and not cancel or re-issue the command because it seems slow.
     - `BU-1221`: To check progress without disturbing the run, the agent uses the validation pipeline from a separate call rather than cancelling or re-issuing the blocking call.
     - `BU-1223`: The `awaiting_agent: parked <duration>` field appearing under a run in status output is observability only: it does not change gate resolution, does not auto-resume the run, and does not make `--yes` the default.
     - `BU-1225`: If `last_activity` is prefixed `quiet`, no step log or native-agent lifecycle activity has arrived for longer than `step_quiet_warning`; this is a liveness clue only, not permission to cancel, rerun, or edit the worktree.
   - Helper attachments (1):
     - `BU-1224`: While a step is actively `running` or `fixing`, the pipeline-automation tool may include an `active_steps` table with `active_for`, `last_activity`, a native `agent_pid` when a subprocess agent is running, and the current round (e.g. `round 1`, `auto-fix 1/3`, `fix 2`).

### `wayfinder`

Established by `BU-0999` (`.agents/skills/wayfinder/SKILL.md (.agents/skills/wayfinder/SKILL.md L13-13)`): By default, wayfinder produces decisions rather than deliverables: each ticket resolves a decision, and the map is considered done once nothing is left to decide before someone goes and does the thing.

- **Trigger:** an effort's map Notes section states an override
- **Outcome:** ticket selection is cheap and deterministic, and claimed before any work starts
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `work-through-map-session`'s.
- **Member stage count:** 4

**Workflow-level helpers** (`workflow=wayfinder`, `stage=null` — deterministic
machinery referenced by the workflow as a whole, not one specific stage):

- `BU-1004` — If no issue tracker has been provided to the effort, wayfinder defaults to the local-markdown tracker rather than failing or guessing at a different one.

**Ordered member stages:**

1. **`select-wayfinder-mode`** — `BU-1000` (`.agents/skills/wayfinder/SKILL.md (.agents/skills/wayfinder/SKILL.md L13-13)`)
   - Trigger: an effort's map Notes section states an override
   - Outcome: the map can drive execution rather than only decisions, when explicitly opted in
   - Statement: An effort can override wayfinder's plan-don't-do default in its map's Notes section, carrying execution into the map itself.

2. **`claim-ticket`** — `BU-1007` (`.agents/skills/wayfinder/SKILL.md (.agents/skills/wayfinder/SKILL.md L67-67)`)
   - Trigger: a session is about to start work on a ticket
   - Outcome: two concurrent sessions do not duplicate work on the same ticket
   - Statement: A session claims a ticket by assigning it to the dev driving the map before doing any work on it, so concurrent sessions skip an already-claimed ticket; the assignment itself is the claim, so an open, unassigned ticket is unclaimed.

3. **`chart-the-map`** — `BU-1022` (`.agents/skills/wayfinder/SKILL.md (.agents/skills/wayfinder/SKILL.md L111-111)`)
   - Trigger: a loose idea too big for one session is being charted
   - Outcome: every subsequent charting step is shaped by an already-fixed destination
   - Statement: Charting a map for a loose, oversized idea starts by naming the destination first (via /grilling and /domain-modeling), since naming it shapes every ticket and fixes the scope before anything else is planned.
   - Stage-context attachments (5):
     - `BU-1023`: The frontier is mapped breadth-first across the whole space rather than deep on one thread; if this surfaces no fog at all — the way to the destination is already clear — no map is created, and the actor stops to ask the user how they'd like to proceed instead.
     - `BU-1024`: Once fog is surfaced, the map issue is created labelled `wayfinder:map` with Destination and Notes filled in, Decisions-so-far left empty, and the surfaced fog sketched into Not yet specified.
     - `BU-1025`: Tickets that can be specified now are created as children of the map, then blocking edges between them are wired in a second pass (since issues need ids before they can reference each other); wiring sorts tickets into the frontier versus blocked, and anything still unspecifiable stays in the fog.
     - `BU-1026`: A /research subagent is fired for each research ticket just created, in parallel, capturing its findings on a throwaway `research/<name>` branch with a context pointer from the ticket.
     - `BU-1027`: Charting stops once complete — it is only one session's work, and it hand-resolves nothing.

4. **`work-through-map-session`** — `BU-1028` (`.agents/skills/wayfinder/SKILL.md (.agents/skills/wayfinder/SKILL.md L122-123)`)
   - Trigger: a work-through-the-map session begins with a map
   - Outcome: ticket selection is cheap and deterministic, and claimed before any work starts
   - Statement: A work-through-the-map session loads the low-resolution map view (not every ticket body); the ticket to work is the one the user named, or otherwise the first frontier ticket in order, and it is claimed by self-assignment before any work begins.
   - Stage-context attachments (15):
     - `BU-1009`: A ticket's answer is not part of its body — it is recorded on resolution — and assets created while resolving are linked from the issue, not pasted into it.
     - `BU-1011`: A research ticket is resolved by a /research subagent, used when knowledge from outside the current working directory is required.
     - `BU-1012`: A prototype ticket is resolved by making a cheap, rough, concrete artifact via the /prototype skill to raise the discussion's fidelity, linked as an asset — used when 'how should it look/behave' is the key question.
     - `BU-1013`: A grilling ticket is resolved via the /grilling and /domain-modeling skills, one question at a time — the default ticket type.
     - `BU-1014`: A task ticket (manual work blocking a decision but not itself a decision) has the agent drive the work alone where it can (AFK) or hand the human a precise checklist (HITL); it is resolved once the work is done, and its answer records what was done plus any resulting facts later tickets depend on.
     - `BU-1016`: A not-yet-specified item becomes a ticket once the question can be stated precisely (even if it's blocked and can't be acted on yet); it stays fog if it can't yet be phrased that sharply — fog is not pre-sliced into ticket-sized pieces.
     - `BU-1017`: The Not yet specified section excludes anything already decided (Decisions so far), anything already a live ticket, and anything out of scope.
     - `BU-1018`: Work identified as beyond the map's destination is recorded as out of scope, not as fog, and does not belong in Not yet specified, because the destination fixes the scope.
     - `BU-1019`: Out-of-scope work never graduates; it returns only if the destination itself is redrawn, and then as a fresh effort, not a resumption of the old one.
     - `BU-1020`: If a ticket that already exists turns out to sit past the destination, it is closed (unambiguously off the frontier) and one line is added to the Out of scope section (the gist, why it's out of scope, and a link to the closed ticket); it stays out of Decisions so far, since that section records only the route actually walked.
     - `BU-1021`: Never more than one ticket is resolved per work-through-the-map session, except research tickets.
     - `BU-1029`: A claimed ticket is resolved by zooming in as needed — fetching the full body of any related or closed ticket on demand, invoking the skills named in the map's Notes block, and defaulting to /grilling and /domain-modeling when in doubt.
     - `BU-1030`: Once a ticket is resolved, the answer is posted as a resolution comment, the issue is closed, and a context pointer is appended to the map's Decisions so far.
     - `BU-1031`: After a ticket's resolution is recorded, newly-surfaced tickets are added (create-then-wire); any now-specifiable fog is graduated and cleared from Not yet specified; a ticket found to sit beyond the destination is ruled out of scope rather than resolved on the route; and any other map parts the decision invalidates are updated or deleted.
     - `BU-1032`: When the user runs unblocked tickets in parallel, other sessions may be editing the tracker concurrently, and this is an expected condition rather than an error.

### `wiki-maintenance`

Established by `BU-0816` (`skills/wiki/SKILL.md (skills/wiki/SKILL.md L7-9)`): The wiki skill is loaded only for explicit wiki-maintenance requests (ingest, backfill, regenerate, inspect, or change wiki output), never for routine dispatch, notification, or cleanup, which write automatic captures without coordinator action.

- **Trigger:** a coordinator runs or regenerates a wiki daily digest
- **Outcome:** a scheduling task cannot be marked done on the basis of installation alone, only on verified successful execution
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `schedule-wiki-digest`'s.
- **Member stage count:** 2

**Ordered member stages:**

1. **`operate-wiki-digest`** — `BU-0821` (`skills/wiki/SKILL.md (skills/wiki/SKILL.md L46-55)`)
   - Trigger: a coordinator runs or regenerates a wiki daily digest
   - Outcome: digest changes are always previewed and inspected before they take effect and are always followed by verification
   - Statement: The daily-digest operating procedure is fixed: read SCHEMA.md first, dry-run before any regeneration or logic change, inspect the dry-run preview for secrets/duplicates/incorrect outcomes/generation errors, only then run the real command, verify the session page and index link exist, and finally append or verify the ingest log entry.
   - Stage-context attachments (19):
     - `BU-0822`: A daily digest page must synthesize outcomes, decisions, blockers, and next state rather than reproduce the underlying conversation as a transcript.
     - `BU-0823`: Installing or expanding a scheduler for the daily digest is treated as a task separate from ordinary digest operation.
     - `BU-0825`: If SCHEMA.md is missing or unreadable, digest generation stops without writing any curated page.
     - `BU-0826`: If a dry-run preview is found to contain secrets, generation stops, only the affected source class (not the secret content itself) is recorded, and redaction is fixed before retrying.
     - `BU-0827`: If a PR or the task tracker task's state cannot be resolved during digest generation, its outcome is marked unresolved rather than inferred as complete.
     - `BU-0828`: If regenerating a page would replace it with less information than it already has, the existing page is preserved and the rejected update is reported instead.
     - `BU-0829`: If the wiki index update fails after a session page is generated, the generated page is kept, its exact path is reported, and the digest is left marked incomplete rather than the page being discarded or the failure hidden.
     - `BU-0834`: The wiki-digest job refuses to run if the opencode session database does not exist.
     - `BU-0835`: The wiki-digest job refuses to run if the opencode auth.json file (source of the Anthropic API key) does not exist.
     - `BU-0836`: The wiki-digest job refuses to run if ~/wiki/SCHEMA.md does not exist.
     - `BU-0837`: The wiki-digest job refuses to run unless both sqlite3 and python3 are available on PATH.
     - `BU-0838`: The wiki-digest job reads the Anthropic API key out of the opencode auth.json file and refuses to run if no key is found there.
     - `BU-0848`: A failed synthesis API call (HTTP error) is surfaced with its status code and response body on stderr and causes the process to exit nonzero, rather than the digest silently proceeding with no or partial content.
     - `BU-0850`: The temporary prompt file (which may contain raw session content pulled from multiple sources) is deleted immediately after the synthesis API call completes, regardless of the call's outcome path shown here.
     - `BU-0851`: A date whose session page already exists is skipped for real (non-dry-run) generation; the existing page is never automatically regenerated or overwritten, only manual deletion allows a re-run for that date.
     - `BU-0854`: When a session page for the date already exists (only reachable in --dry-run, since a real run already skipped this date at BU-0851), its existing content is loaded and included in the synthesis prompt so regeneration is update-in-place rather than starting blind.
     - `BU-0855`: In --dry-run mode the assembled synthesis prompt is printed and the loop moves to the next date without ever calling the synthesis API or writing a page.
     - `BU-0856`: An empty synthesis result for a date is treated as an error and that date is skipped, rather than an empty session page being written.
     - `BU-0858`: Every successful (non-dry-run) session page write unconditionally appends a fixed-format ingest log line to ~/wiki/log.md.
   - Helper attachments (17):
     - `BU-0830`: The wiki-digest job rejects an unrecognized CLI argument with a usage message rather than ignoring it or proceeding with defaults.
     - `BU-0831`: Invoking the wiki-digest job with --since builds and processes one date per day from the given date up to (but not including) today, i.e. a backfill run, instead of a single date.
     - `BU-0832`: The literal value "yesterday" given to --date is resolved to the actual calendar date one day before the run, not treated as a literal filename fragment.
     - `BU-0833`: With neither --since nor --date given, the wiki-digest job defaults to generating a digest for today only.
     - `BU-0839`: Opencode sessions considered for a day's digest exclude sub-sessions (sessions with a parent_id) and sessions still carrying the default "New session..." title.
     - `BU-0840`: Per-opencode-session content pulled into the digest is limited to assistant-authored text parts longer than 20 characters, concatenated in chronological order and truncated to MAX_CHARS_PER_SESSION.
     - `BU-0841`: Goose is an optional session source: if the goose sessions database does not exist, goose session extraction succeeds with no sessions rather than failing the run.
     - `BU-0842`: Claude Code project history is an optional session source: if the claude projects directory does not exist, claude session extraction succeeds with no sessions rather than failing the run.
     - `BU-0843`: A claude session is attributed to the digest date of its last (most recent) message timestamp, not its first message timestamp, and is skipped entirely if it has no resolvable timestamp.
     - `BU-0844`: PR enrichment degrades gracefully when the gh CLI is unavailable: the digest still generates, with a "(gh not available)" placeholder instead of PR data, rather than the whole run failing.
     - `BU-0845`: Merged-PR enrichment is scoped to a fixed, hardcoded list of four known repos, and within each repo only PRs whose mergedAt date matches the target digest date are included.
     - `BU-0846`: Task enrichment degrades gracefully when the task tracker CLI is unavailable: the digest still generates, with a "(the task tracker not available)" placeholder instead of task data, rather than the whole run failing.
     - `BU-0847`: Task enrichment includes a task in a day's digest only if the task was updated that date and its status is closed or review.
     - `BU-0849`: The full synthesis prompt — which embeds raw extracted session content — is written to a temporary file before being passed to the API call.
     - `BU-0852`: A date is skipped with no page written only if all three sources (opencode, goose, claude) return zero sessions; any one source having sessions is enough to proceed.
     - `BU-0853`: The full text of ~/wiki/SCHEMA.md is injected verbatim into the synthesis prompt as the authoritative instructions on every digest run, rather than being paraphrased or hardcoded into the script.
     - `BU-0857`: The wiki index is only appended a link for a date's session page if that date is not already referenced in the index, so re-running a digest for an already-indexed date never adds a duplicate index entry.

2. **`schedule-wiki-digest`** — `BU-0824` (`skills/wiki/SKILL.md (skills/wiki/SKILL.md L62-65)`)
   - Trigger: a coordinator has installed a scheduled job for the wiki-digest job
   - Outcome: a scheduling task cannot be marked done on the basis of installation alone, only on verified successful execution
   - Statement: Scheduling the daily digest is not reported complete until the job definition, executable path, environment, last exit status, and generated page have all been verified.

### `worker-contract`

- **Trigger:** a worker begins a phase of implementation work
- **Outcome:** done is only ever reported once every gate has genuinely passed, and failed carries an exact, specific reason rather than a generic one
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `report-terminal-status`'s.
- **Member stage count:** 2

**Ordered member stages:**

1. **`route-to-phase-skill`** — `BU-0280` (`skills/dispatch/SKILL.md (skills/dispatch/SKILL.md L168-173)`)
   - Trigger: a worker begins a phase of implementation work
   - Outcome: the worker's approach is selected by a fixed routing table matching work shape to skill, rather than improvised per worker
   - Statement: Before implementation, a worker routes to the canonical engineering skill matching the phase of work — huge/foggy work surfaces wayfinder/to-spec/to-tickets as escalation rather than being silently executed as implementation, hard bugs load diagnosing-bugs, uncertain logic/UI loads prototype (never promoting prototype code directly), approved implementation loads tdd, and merge/rebase conflicts load resolving-merge-conflicts (never aborting automatically).

2. **`report-terminal-status`** — `BU-0283` (`skills/dispatch/SKILL.md (skills/dispatch/SKILL.md L188)`)
   - Trigger: a worker reaches a terminal outcome
   - Outcome: done is only ever reported once every gate has genuinely passed, and failed carries an exact, specific reason rather than a generic one
   - Statement: A worker writes `.sergeant-result` and sets `.sergeant-status=done` only after every gate passes; `failed: <exact reason>` is reserved for an unrecoverable terminal failure.
   - Stage-context attachments (1):
     - `BU-0286`: To recover a waiting or orphaned worker, response-delivery step is used and the worker is never marked done manually; to retry a failed repo, the underlying issue is fixed first, and `.sergeant-result` plus `.sergeant-status=done` are written only after every completion gate passes.

### `worker-lifecycle`

- **Trigger:** a worker is resumed or recovered
- **Outcome:** the call refuses to stop anything and reports the inconsistency instead of guessing
- **Completion condition:** every member stage below has reached its own
  outcome, ending in stage `stop-background-monitor`'s.
- **Member stage count:** 26
- **Ordering note:** `worker-lifecycle` is graph-shaped (independent event-triggered
  entry points, not a single pipeline) — source/behavior_id order is used as
  the defensible default per the run-wide note above, not a proven chain.

**Workflow-level helpers** (`workflow=worker-lifecycle`, `stage=null` — deterministic
machinery referenced by the workflow as a whole, not one specific stage):

- `BU-0503` — The response lock is acquired via a uniquely-named candidate file that is hardlinked to the lock path; ownership of the lock is only confirmed once the hardlink succeeds AND the lock path and candidate are verified to share the same inode.
- `BU-0504` — A stale lock directory whose recorded owner pid is no longer alive is reclaimed by removing it and retrying acquisition, rather than blocking forever.
- `BU-0505` — A lock directory found with no owner pid file at all, but with other contents present, is treated as an invalid, unrecoverable lock state and acquisition fails outright rather than attempting to reclaim it.
- `BU-0506` — Releasing the response lock only removes the lock file when its recorded owner is exactly this process; if another process now owns it, the release leaves the on-disk lock untouched and only clears this process's own tracking of it.
- `BU-0507` — A dedicated reclaim path drops a lock recorded under this process's own PID specifically when a background loop of the same process was killed while holding it; the comment explicitly restricts calling it to only after every other lock-holding context in this same process has already terminated, since it would otherwise break mutual exclusion.
- `BU-0508` — A helper distinguishes the specific state where the lock is held by this process but not by the current calling context; a caller in that state must not wait on the lock, since the ordinary liveness check would see its own live PID forever and spin without end.
- `BU-0917` — A background-monitor task id must match a strict alphanumeric/dot/hyphen/underscore charset (and must not be empty, '.', '..', or contain a slash) before it is used to construct a systemd unit name, to prevent argument injection and unit-name collisions.
- `BU-0918` — A systemd unit's zero-GUID InvocationID (00000000000000000000000000000000) is treated as 'no active invocation', never as a real invocation identity.

**Ordered member stages:**

1. **`resume-model-pin-reverification`** — `BU-0073` (`README.md (README.md L227-229)`)
   - Trigger: a worker is resumed or recovered
   - Outcome: the original model pin is honored exactly on resume, and an unhonorable tuple fails terminally instead of silently substituting a default
   - Statement: A resumed or recovered worker reads the same fleet record and inherits the same model pin; a worker handed a tuple its harness cannot honor fails terminally rather than falling back to the ambient default.
   - Stage-context attachments (1):
     - `BU-0344`: The pinned model tuple lives in durable fleet state (not the ambient environment) so that a worker resumed by the worker response-delivery step, the stalled-worker recovery step, or the wake-condition step runs the exact same pinned model as the original dispatch; a tuple the harness cannot honor is a terminal launch failure rather than silently falling back to an ambient default model.

2. **`drain-admission-lock`** — `BU-0105` (`README.md (README.md L353-358)`)
   - Trigger: dispatch or respond needs to take the drain admission lock
   - Outcome: locking either succeeds via hard link or the operation fails closed — it never proceeds without the lock
   - Statement: Drain admission locking uses an atomic hard link rather than requiring `flock`; the drain state directory must be writable by the invoking user and on a filesystem that supports hard links, otherwise dispatch and respond fail closed rather than proceeding unlocked.
   - Stage-context attachments (2):
     - `BU-0521`: A project name that would collide with the admission-lock's own artifact filename is refused outright, so no drain or undrain can ever target the live lock.
     - `BU-0528`: The drain step --status, when scanning all project drains, explicitly excludes lock records and their staging/quarantine/temp-file artifacts from being reported as active drains — reporting them would invent drains nobody set, and the obvious operator response (the drain step) would delete a lock a live dispatch is holding.
   - Helper attachments (19):
     - `BU-0543`: The drain admission lock uses a hard link, not flock(1): flock is absent from macOS system installs and limited on BusyBox, and guarding it with `command -v flock` previously meant the whole lock silently degraded to a no-op; link(2) fails atomically when the target already exists and is available everywhere Sergeant runs.
     - `BU-0544`: The lock's owner record is written and fully staged before it becomes the lock (via link), so the lock can never exist in an unattributable state that no later contender is able to reclaim.
     - `BU-0545`: Every lock instance carries a nonce, and both reclamation and release are bound to that exact nonce, so a contender can neither destroy a lock that was reacquired while it was deciding to reclaim it, nor release a lock it no longer owns.
     - `BU-0546`: No EXIT trap is installed by this locking library on purpose, because release is explicit rather than kernel-backed and the library is sourced by scripts that own their own traps; a holder killed mid-flight simply leaves its record behind, which is safe because the record always names a verifiable owner the next contender can reclaim from.
     - `BU-0547`: Process liveness is checked via /proc when available (authoritative on Linux and BusyBox), because `kill -0` alone is unreliable — it also fails with EPERM for a live process owned by another user; without /proc and without a usable `ps`, liveness is reported as undeterminable rather than assumed dead.
     - `BU-0548`: Callers of the process-liveness check must treat an undeterminable (return code 2) result as 'still alive', so an unverifiable lock/worker owner is never displaced by a false claim of death.
     - `BU-0549`: PID-reuse detection prefers reading a process's start time from /proc/<pid>/stat field 22 (Linux/BusyBox) and falls back to `ps -o lstart=` (macOS) when /proc is unavailable.
     - `BU-0550`: Every value written into a lock record is sanitized — newlines and carriage returns stripped, truncated to 256 characters — before being written, because a newline embedded in USER, in `uname -n`, or in a process-start token could otherwise inject additional record fields; specifically, an injected owner_nonce read first by `grep -m1` would prevent the true owner from ever releasing its own lock.
     - `BU-0551`: A lock record is read in one snapshot rather than field-at-a-time, because a competing reclaimer renaming the record mid-check could otherwise hand back fields from different record generations — including an empty nonce, which previously made reclamation unbounded.
     - `BU-0552`: An unidentified host is never treated as matching another unidentified host when deciding whether a lock's recorded owner is running on this machine.
     - `BU-0553`: A lock record with no nonce is never eligible for reclamation, because treating it as reclaimable would let a contender delete a lock that was legitimately acquired during the very race being adjudicated.
     - `BU-0554`: Reclaiming a lock proven stale renames the record first (an atomic move, so at most one contender can move it), then re-checks the observed nonce against the one that justified the reclaim; if a different lock instance is found — meaning the lock was legitimately reacquired between the staleness decision and the rename — the quarantine copy is restored via `ln` (which refuses to clobber) instead of being discarded.
     - `BU-0555`: When restoring a wrongly-reclaimed lock record fails and no other copy of it has appeared, the quarantine copy is deliberately preserved (not deleted) and an error is printed, because it is the only remaining copy of a live lock record and must survive for the next contender to find.
     - `BU-0556`: Leftover quarantine and staging lock artifacts are swept only when the process id embedded in their own filename is provably gone, so a concurrent reclamation or acquisition already in progress is never disturbed by the sweep.
     - `BU-0557`: A drain-admission-lock acquisition timeout reports the current holder's pid, user, host, purpose, and age, plus explicit recovery guidance (retry; a dead owner's lock is reclaimed automatically), rather than an undiagnosable generic failure.
     - `BU-0558`: `ln` failing while the lock's target path does not exist is treated as a filesystem incapable of hard links (e.g. FAT/exFAT, some CIFS/FUSE mounts) rather than as contention, and fails immediately rather than spinning to the timeout deadline and reporting a nonexistent holder.
     - `BU-0559`: Releasing a drain-admission lock re-verifies the on-disk owner_nonce first, so release can never remove a lock that now belongs to a different process — for example after an operator manually removed the record and someone else has since acquired it; when the nonce no longer matches, an error is printed and the record is deliberately left in place.
     - `BU-0560`: When the drain admission lock cannot be acquired, the wrapped command is never invoked at all — the lock-acquisition outcome (timeout or unavailable) is returned in its place, and an empty invocation (no command given) is itself refused rather than silently reported as success.
     - `BU-0561`: Because a wrapped command may itself exit with status 2 or 3 (the same numeric sentinels the lock wrapper uses for timeout/unavailable), SGT_DRAIN_LOCK_STATE — not the numeric return code alone — is the authoritative signal a caller must use to distinguish a lock-acquisition failure from the wrapped command's own exit status.

3. **`deliver-mission`** — `BU-0137` (`docs/using-sergeant.md (docs/using-sergeant.md L83-86)`)
   - Trigger: a worker's mission/brief is being delivered at launch
   - Outcome: delivery is exactly-once-safe across TUI startup delay or coordinator crash, and never exposes the mission body via process args
   - Statement: A worker-owned loop retries only a fixed ID-bearing terminal nudge until the agent acknowledges that ID before acting, so delayed TUI startup and coordinator crashes do not lose or duplicate the mission, and no body appears in process arguments.
   - Stage-context attachments (4):
     - `BU-0305`: A worker follows a notification instruction exactly once per token; repeated nudges carrying the same token are treated as retries, not new work.
     - `BU-0911`: Notification delivery-confirmed (the nudge reached the exact target pane and was accepted) is a distinct, separately durable state from action-completion (the agent actually acted), recorded under separate artifacts (targets/<nonce>/handshake_complete vs. targets/<nonce>/completed), because previously conflating the two let a caller believe a turn had settled when nothing had actually published completion.
     - `BU-0912`: While waiting for notification delivery, the target pane's live identity is re-verified against the expected identity on every polling iteration, not only once at the start, before any delivered/accepted marker for that pane is trusted.
     - `BU-0913`: Waiting for worker-notification delivery is bounded by a configurable timeout (SGT_NOTIFICATION_ACK_TIMEOUT, default 60 seconds); once the bound is exceeded the wait returns failure rather than fabricating a success.
   - Helper attachments (2):
     - `BU-0907`: A notification id must match a strict alphanumeric/dot/hyphen/underscore charset before any notification-publish state is touched; anything else is rejected immediately.
     - `BU-0908`: A notification's durable record is only overwritten when its content actually differs from what is already on disk (compared byte-for-byte before publishing); when it does need to change, the new content is written to a temp file and atomically renamed into place.

4. **`bulk-reconcile-fleet-state`** — `BU-0141` (`docs/using-sergeant.md (docs/using-sergeant.md L149-155)`)
   - Trigger: the interactive fleet-watch loop --sync-all runs
   - Outcome: panes are stopped only after identity verification, and ambiguous interrupted records wait out a grace period before being marked failed
   - Statement: Bulk reconciliation (the interactive fleet-watch loop) syncs worktree status into fleet state, stops only identity-verified `done` or `failed` worker panes, and marks interrupted `dispatched` records failed only when they have neither a worktree nor an owned live pane after a default 300-second grace period.
   - Stage-context attachments (2):
     - `BU-0142`: Bulk reconciliation preserves `needs_input`, `blocked`, and `orphaned` worktrees, and dispatch runs this reconciliation automatically before creating new tasks.
     - `BU-0603`: The interactive fleet-watch loop --sync-all reconciles every task directory under the fleet root and reports how many it processed; unlike the interactive watch loop, it does not fail the invocation based on any individual repo's terminal status.
   - Helper attachments (1):
     - `BU-0607`: The interactive fleet-watch loop --sync <task-id> requires the named task directory to exist, dying with a usage-style error naming the fleet directory it looked in if the task cannot be found, rather than silently doing nothing.

5. **`stalled-worker-recovery`** — `BU-0146` (`docs/using-sergeant.md (docs/using-sergeant.md L169-172)`)
   - Trigger: recovery is being considered for a stalled worker
   - Outcome: recovery is gated on having already reconciled identity/worktree/handoff/notification evidence, and only applies to the one named diagnostic
   - Statement: The stalled-worker recovery step is used only after reconciling the exact pane identity, worktree, the task tracker handoff, and response/notification state, and only for the exact `live worker stalled` case.
   - Stage-context attachments (11):
     - `BU-0159`: The stalled-worker recovery step is one-shot per repo attempt: Sergeant records `stall_recovery_attempted`, relaunches only after replacement metadata is validated, and escalates to `needs_input` instead of retrying when the prior notification delivery still holds an unfinished action lease, the recorded pane identity no longer matches, or any later relaunch step fails.
     - `BU-0175`: If Sergeant refuses recovery because pane identity or unfinished notification delivery evidence no longer matches, the preserved state is kept and the resulting `needs_input` handoff is followed instead of forcing another retry.
     - `BU-0483`: Recovery is only available to a worker whose current status is exactly in_progress; any other status is refused.
     - `BU-0484`: Recovery requires the fleet diagnostic to begin with the literal prefix 'live worker stalled:', written specifically by the interactive fleet-watch loop's stall classification; a worker lacking that exact proof is refused recovery.
     - `BU-0485`: The response lock is acquired before any recovery mutation, and both the status and stall-diagnostic checks are re-verified again once the lock is held, not trusted from the pre-lock read.
     - `BU-0486`: Recovery is strictly one-shot: if a stall_recovery_attempted marker already exists for this worker, a second invocation is refused and the worker is instead escalated to needs_input.
     - `BU-0487`: A notification-lease owner is only ever treated as provably dead when its recorded tmux pane no longer resolves at all AND its recorded process is not running; a resolvable pane (matching or reused identity), a still-live recorded pid, or a missing/malformed identity record all fail closed instead.
     - `BU-0488`: An in-flight notification action lease from a prior supervisor blocks recovery unless the shared finalizer proves completion from the agent's own durable proof, or the lease owner is adjudicated provably dead; otherwise recovery is refused and the worker is escalated to needs_input rather than proceeding over unresolved delivery evidence.
     - `BU-0490`: Before any mutation, relaunch metadata (tmux availability, session, window name, agent) must be fully present, or recovery is refused and the worker is escalated.
     - `BU-0492`: The one-shot recovery marker is stamped only after every pre-flight check has passed, guaranteeing at most one recovery attempt is ever made even if a later step in this same invocation fails.
     - `BU-0499`: Stall evidence (the diagnostic and message files) is cleared only once recovery has fully succeeded; every failure path preserves it so the eventual escalation reports an accurate reason instead of an empty one.
   - Helper attachments (3):
     - `BU-0480`: The stalled-worker recovery step requires exactly two positional arguments, a task ID and a repo; any other count is rejected.
     - `BU-0481`: Each of the task ID and repo arguments to the stalled-worker recovery step must match a restricted identifier pattern; either failing is rejected.
     - `BU-0482`: The stalled-worker recovery step refuses to proceed if the named task or repo does not exist in fleet state, or if the recorded worktree is unavailable.

6. **`recover-orphaned-worker`** — `BU-0147` (`docs/using-sergeant.md (docs/using-sergeant.md L182)`)
   - Trigger: a worker is observed in the orphaned state
   - Outcome: orphaned is treated as requiring full reconciliation before any recovery action, not a quick retry
   - Statement: `orphaned` means the expected supervisor identity disappeared without a durable waiting state, and the required operator action is to reconcile process, pane, worktree, branch, task, and handoff before recovery.
   - Stage-context attachments (8):
     - `BU-0176`: An expected dependency-blocked exit must remain blocked; it is not an orphan merely because the process ended, so supported response/recovery is used only after reconciling the record, and its worktree is never cleaned.
     - `BU-0178`: A missing pane plus durable blocked/handoff state is classified as waiting work; a missing pane from `in_progress` without a handoff is orphan evidence.
     - `BU-0306`: If an agent exits before reaching a terminal or waiting state, the supervisor records `orphaned` with durable diagnostics and the task tracker recovery pointers; orphaned work is resumed only through the worker response-delivery step, and its recovery state is never overwritten or discarded.
     - `BU-0357`: When a worker exits orphaned, _finish inspects git log above the dispatch base (or, absent that record, commits not on the upstream tracking branch) and records in the diagnostic whether real committed work exists, so the coordinator can distinguish a clean orphan from an interrupted worker with real work needing reconciliation before re-dispatch.
     - `BU-0596`: When a repo's recorded worktree is missing or absent and its fleet status is in_progress, the interactive fleet-watch loop reclassifies it to orphaned with a diagnostic directing reconciliation of the preserved branch and handoff, rather than leaving the stale in_progress status in place.
     - `BU-0597`: The interactive fleet-watch loop treats a worktree reporting terminal status done with no substantiating .sergeant-result file as an ambiguous terminal condition: it reclassifies both the worktree and fleet status to orphaned with a diagnostic, and hands off via the task-tracker memory step, rather than accepting done at face value.
     - `BU-0599`: An in_progress repo with no recorded worker pane is reclassified to orphaned with a diagnostic instructing resume via the worker response-delivery step, rather than continuing to poll a repo that was never given a pane to check.
     - `BU-0600`: An in_progress repo whose recorded pane fails supervisor-identity verification (dead, or no longer the expected worker) is reclassified to orphaned with a diagnostic instructing resume via the worker response-delivery step, rather than continuing to treat a foreign or dead pane as the live worker.

7. **`enter-waiting-state`** — `BU-0148` (`docs/using-sergeant.md (docs/using-sergeant.md L188-190)`)
   - Trigger: a worker needs to wait on an external condition
   - Outcome: the wait is represented durably (wake-condition file + waiting status) rather than a live sleep loop, permitting clean exit
   - Statement: `waiting` is used instead of sleep loops for CI checks, dependency completion, and time-based delays: the worker writes `.sergeant-wake-condition`, sets `.sergeant-status=waiting`, and may exit cleanly after its durable handoff.
   - Stage-context attachments (2):
     - `BU-0307`: `.sergeant-gate-generation` is a monotonic integer for waiting gates: before every new `needs_input` or `blocked` publication, the worker increments and persists the generation before writing the waiting status and message, so a repeated blocker message only counts as a new gate when the generation actually advances.
     - `BU-0308`: Only the allowlisted wake-condition field names and alphanumeric-safe values are accepted in `.sergeant-wake-condition`; arbitrary shell commands, prompt bodies, response text, tokens, or secrets are never persisted there.

8. **`evaluate-and-resume-wait`** — `BU-0149` (`docs/using-sergeant.md (docs/using-sergeant.md L196-199)`)
   - Trigger: the wake-condition step is invoked for a waiting worker
   - Outcome: resumption happens only for the exact worker whose condition evaluates as met, tagged with a required generation
   - Statement: The wake-condition step evaluates the wake condition and resumes the exact waiting worker through the worker response-delivery step when the condition is met; every condition requires `generation=<int>`.
   - Stage-context attachments (25):
     - `BU-0150`: A `github_check` wake condition requires both `run_id` and `check_name`, and resumes only when that exact check concludes `success`; `failure`, `cancelled`, `skipped`, `timed_out`, and every other non-success conclusion never resume the worker.
     - `BU-0151`: A condition that can no longer be met converts the worker to `needs_input` with the remedy in `.sergeant-message` instead of retrying until its deadline — covering a named check that concluded unsuccessfully, a check absent from an already-completed run, an ambiguous duplicate check name, or a condition missing a required field.
     - `BU-0152`: `human_response` conditions never auto-resume and always convert the worker to `needs_input` for a human response delivered through the worker response-delivery step; `deployment` remains a declared condition kind but also escalates to `needs_input` until an installation-specific deployment adapter is wired.
     - `BU-0453`: The wake-condition step only evaluates a worker whose current status is exactly 'waiting'; any other status refuses evaluation.
     - `BU-0454`: The wake-condition step refuses to proceed unless a wake-condition file exists in the worktree.
     - `BU-0455`: Every field name in a wake-condition file is checked against a strict allowlist; any unknown field name causes the whole evaluation to be rejected outright.
     - `BU-0456`: Even an allowlisted field name is rejected if it matches a secret-like pattern (token, password, key, secret, auth, credential, and variants), as a belt-and-suspenders check.
     - `BU-0458`: Wake-condition field values (other than check_name) are restricted to a pattern that never admits a leading '-', because every value becomes an argument to gh(1) or the task tracker(1) and a dash-leading value would be read as an option rather than as data.
     - `BU-0459`: check_name uses a wider allowlist admitting the punctuation real human-authored GitHub check names contain, but still excludes shell metacharacters, quotes, redirection, and control characters, and is used only as quoted argv/printf data, never eval'd or word-split.
     - `BU-0460`: Both the kind and generation fields are required in a wake condition; either being absent is a hard failure before any condition is evaluated.
     - `BU-0461`: The wake-condition step serializes itself with a per-(task,repo) lock directory; if the lock directory exists but its recorded owner pid is no longer alive, the wake-condition step reclaims it by removing and recreating it, rather than blocking forever.
     - `BU-0462`: If a wake condition's deadline has already passed, the worker is moved to a terminal 'failed: deadline exceeded' state instead of being retried.
     - `BU-0463`: If a wake condition's max_attempts has already been reached, the worker is moved to needs_input for manual intervention instead of being retried further.
     - `BU-0468`: Every GitHub query a wake condition makes is explicitly bound to the worker's own recorded worktree/remote, never the scheduler's own current directory, because gh(1) otherwise silently resolves against the wrong repository.
     - `BU-0469`: Resolving a worktree's origin remote to a GitHub owner/repo slug anchors the hostname match so that a host merely ending in 'github.com' (e.g. mygithub.com) is never mistaken for github.com and queried as the wrong repository.
     - `BU-0470`: A github_check condition with a missing or non-numeric run_id, or a missing check_name, escalates immediately rather than being retried, because no amount of waiting can make a malformed condition well-formed.
     - `BU-0471`: For github_check, zero matching checks in a still-running workflow run is treated as unmet (the check may not have started yet); zero matching checks in an already-completed run is escalated as permanently unsatisfiable, since the named check will never appear.
     - `BU-0472`: For github_check, exactly one matching but still-incomplete check is unmet; a concluded 'success' is met; any other concluded conclusion escalates as permanently unsatisfiable rather than retried.
     - `BU-0473`: For github_check, more than one job sharing the same check_name is escalated as ambiguous rather than guessed at.
     - `BU-0474`: The deployment and human_response condition kinds are always treated as unsupported, and so is any unrecognized kind — none of these are auto-evaluated.
     - `BU-0475`: An 'unsupported' evaluation result sets the worker to needs_input for manual response, and the wake-condition step exits non-zero.
     - `BU-0476`: An 'escalate' evaluation result records the escalation reason to the fleet diagnostic and sets the worker to needs_input describing the condition as no longer satisfiable, rather than retrying it.
     - `BU-0477`: An 'error' evaluation result (an adapter/transient failure) records the reason to the diagnostic but is retried with a recorded attempt and backoff, rather than escalated to needs_input.
     - `BU-0478`: An 'unmet' evaluation result is retried with a recorded attempt and backoff exactly like an error result, but without writing a diagnostic.
     - `BU-0479`: Once a condition is met, the wake-condition step resumes the worker by invoking the worker response-delivery step with a message describing the wake evidence piped to it as the response text.
   - Helper attachments (8):
     - `BU-0450`: The wake-condition step requires exactly two positional arguments, a task ID and a repo; any other count is rejected.
     - `BU-0451`: Each of the task ID and repo arguments to the wake-condition step must match a restricted identifier pattern; either failing is rejected.
     - `BU-0452`: The wake-condition step refuses to proceed if the named repo has no fleet-state directory, or if its recorded worktree does not exist.
     - `BU-0457`: A field present in the wake-condition file with an empty value is rejected with a distinct error from an invalid-character value, so the operator is not misled into hunting for a bad character that is not there.
     - `BU-0464`: Each unmet or errored evaluation attempt is recorded with a pseudorandom jitter (0-9 seconds) added to the configured base backoff, publishing the timestamp before which no subsequent attempt should occur.
     - `BU-0465`: A not_before condition is met once the current time is at or after the recorded timestamp field; before that it is unmet.
     - `BU-0466`: A fleet_dependency condition is met only when the dependency task/repo's own status file reads exactly 'done'; any other recorded status, including a failed or missing one, is unmet, never met.
     - `BU-0467`: A td_dependency condition is met only when the task tracker reports one of done/closed/complete/completed; an empty status is treated as a distinct adapter error (not simply unmet), and any other value is unmet.

9. **`drain-fleet-admission`** — `BU-0153` (`docs/using-sergeant.md (docs/using-sergeant.md L237-243)`)
   - Trigger: a drain is activated, optionally with --wait
   - Outcome: new pane admission is refused immediately; existing workers are allowed to finish cooperatively rather than being force-terminated on timeout
   - Statement: A drain refuses new pane starts for the matching scope while still storing responses generation-safely for later delivery; `--wait` activates the drain, then waits for live workers in scope to finish their current turn and exit, and on timeout leaves the drain active, exits nonzero, and names the unresolved workers without terminating any of them.
   - Stage-context attachments (15):
     - `BU-0154`: A worker is only treated as finished when its exit can be proven, so a worker whose identity was never recorded blocks the wait rather than being silently counted as drained.
     - `BU-0348`: Cooperative drain first performs a durable task tracker handoff and finalizes the accepted action lease (from the agent's own proof, or explicitly recorded as still pending) before publishing the drained status and terminating; the drain path never invents a result, leaving status honestly as the nonterminal "drained" with any prior result file removed.
     - `BU-0423`: Drain admission is checked only at the point a new worker pane/process would be launched; drain never blocks storing a response or delivering it to an already-live pane.
     - `BU-0424`: If drain is active when a relaunch would otherwise occur, the worker response-delivery step records a generation-bound drain_held marker (so an explicit undrain can re-evaluate this exact waiting worker once) and exits successfully having stored the response without relaunching.
     - `BU-0489`: Global or project drain blocks a stall relaunch; on drain, recovery escalates to needs_input without stamping the one-shot recovery marker, so recovery remains retryable once undrain occurs.
     - `BU-0515`: While a drain is active, response-driven relaunches and stall recovery are refused for matching projects; responses due during the drain are stored generation-safely for later delivery rather than dropped.
     - `BU-0516`: --wait activates the drain before it begins waiting, so no new work can be admitted during the wait for live workers in scope to finish their current turn and exit.
     - `BU-0517`: --wait exits nonzero on timeout, leaves the drain active, and names the unresolved workers; it never terminates any worker itself.
     - `BU-0519`: A project name and --global are mutually exclusive scopes for the drain step; combining them is a hard error, because silently letting --global win would escalate a one-project pause into blocking every project.
     - `BU-0522`: --timeout and the drain-wait timeout/interval environment variables must be a non-negative whole number of seconds; an unvalidated value could otherwise silently become 0 inside arithmetic (turning a bounded wait into an immediate timeout) or abort `sleep` midway with the drain already active.
     - `BU-0523`: A worker's exit is never inferred from missing identity: a pane that is running but whose identity was never recorded (the interactive worker harness skips recording it when `ps` cannot report a pgid or start time) blocks the drain wait rather than being silently counted as finished.
     - `BU-0524`: A worker with a currently-live PID is only treated as 'gone' once its recorded process-start token no longer matches the process now holding that PID (proving PID reuse); when either side's start time cannot be obtained, the tokens are incomparable and the worker stays 'unverifiable' rather than being declared gone.
     - `BU-0525`: force-stopped, orphaned, and failed* worker statuses are excluded entirely from --wait's cooperative-drain tracking, because a cooperative wait can never resolve them by waiting — resolving them is the forced-drain step's job, not the drain step --wait's.
     - `BU-0526`: A worker reported 'done' or 'drained' is trusted at its word (treated as resolved) for --wait purposes only when its liveness is unverifiable and not contradicted by an actually-live process; a 'done'/'drained' status accompanied by a provably live process still blocks the wait, because the interactive worker harness writes 'drained' before killing its pane and before its exit-path handoff runs.
     - `BU-0527`: A nonterminal worker with no recorded project cannot be attributed to any drain scope, so it is reported as unresolved by --wait rather than silently skipped.
   - Helper attachments (9):
     - `BU-0518`: --wait is refused unless the mode is 'drain' (not with --undrain or --status, since there is nothing to wait for), and --timeout is refused unless --wait was also given.
     - `BU-0520`: A project name argument to the drain step must match ^[A-Za-z0-9][A-Za-z0-9._-]*$; anything else is refused.
     - `BU-0529`: Locked drain mutations (write/clear) pass caller-supplied text such as --reason as argv elements to the locked helper, never interpolated into a shell string, so it is never re-parsed by the shell.
     - `BU-0539`: Removing a drain explicitly restores admission for that scope; the undrain step is idempotent — undraining a scope that is not currently drained still exits 0.
     - `BU-0540`: --global and an explicit project target are mutually exclusive for the undrain step.
     - `BU-0541`: A project name argument to the undrain step must match ^[A-Za-z0-9][A-Za-z0-9._-]*$; anything else is refused before any drain state is touched.
     - `BU-0542`: _sgt_is_drained treats an empty or syntactically-invalid project name as absent, checking only the global drain in that case, rather than erroring or attempting to match it against a project drain file.
     - `BU-0562`: A drain file is written atomically via a temp-file-then-rename, and drain activation is determined solely by the file's existence — the reason/actor/deadline fields it may also carry are for human inspection only and are never consulted to decide whether a drain is active.
     - `BU-0563`: Removing a global or per-project drain file runs under the admission lock and is idempotent — safe to call when no drain is currently active.

10. **`respond-to-worker`** — `BU-0155` (`docs/using-sergeant.md (docs/using-sergeant.md L255-262)`)
   - Trigger: the worker response-delivery step is about to be used
   - Outcome: the five-step precondition/delivery sequence is followed before and after responding
   - Statement: Before responding to a worker: read the exact finding/question and recommendation, ask only for missing product/risk/security/privacy/destructive/irreversible decisions, record the decision in the owning task tracker task, verify no unconsumed response generation already exists, and require the matching worker to acknowledge/consume the response after sending.
   - Stage-context attachments (17):
     - `BU-0156`: The supervisor nudge includes a scoped token (`notification_id|target_nonce`); the agent writes the acknowledgement but does not act yet, proceeding only after the targeted supervisor sends acceptance and the scoped acceptance file contains the same token, then records completion in the named completion file.
     - `BU-0157`: The notified worker reads `.sergeant-response`, its ID, and gate generation, applies the decision exactly once, restores truthful status, and writes `.sergeant-response-applied` with the matching ID, generation, and status.
     - `BU-0177`: A pending response is never overwritten; the exact waiting worker is resumed with the worker response-delivery step or the caller waits for the current generation to reach a terminal outcome, the stalled-worker recovery step is never used for an active response generation, and if the worker already applied the response with an existing archive entry, the same response-acknowledgement step command is rerun from the recorded worker pane to finish acknowledgement and cleanup.
     - `BU-0179`: Repeated notifications are diagnosed by comparing task, repo, state generation, message digest, and timestamp, since they can indicate stale fleet records, unconsumed responses, or an expected blocked worker incorrectly reclassified orphaned; duplicate tasks or duplicate responses are never created in reaction.
     - `BU-0275`: When a worker escalates, the coordinator reads its context/evidence/exact question/recommendation/options, obtains the human decision without inferring consequential intent, and only then runs the worker response-delivery step; the worker consumes/removes the response, clears its message, logs the decision to the task tracker, and returns to in_progress.
     - `BU-0405`: Before making any state change, the worker response-delivery step acquires this repo's response lock and registers a cleanup trap that releases the lock and removes the private response-input file on exit.
     - `BU-0412`: The worker response-delivery step refuses to publish a response unless the recorded worktree is re-verified, at response time, to still be the exact owned checkout — its actual git pointer and git directory must match the values recorded at dispatch — even after a migration attempt.
     - `BU-0413`: The worker response-delivery step refuses to publish a response unless the worker's canonical intent revision still matches at response time; if it does not, one migration attempt is made and the check is repeated before refusing.
     - `BU-0414`: The worker response-delivery step only accepts a response when the worker's current status is exactly one of needs_input, blocked, waiting, or orphaned; any other status is refused.
     - `BU-0415`: Publishing the local response transport writes the generation marker, response id, response body (fleet-state and worktree copies), atomically via temp-file-then-rename, and clears any stale acknowledgement marker before writing the new response id, so a leftover ack can never be mistaken for acknowledgement of the new response.
     - `BU-0416`: A delivery-escalation marker is considered armed only when it names the exact response id and gate generation currently in play; a marker from an earlier response or generation does not arm escalation.
     - `BU-0417`: For an orphaned worker with no valid recorded gate generation, the worker response-delivery step initializes the generation to 1 and records it, rather than refusing the response for lack of a generation.
     - `BU-0418`: If the response text supplied to the worker response-delivery step differs from an already-stored pending response, the newly supplied text is ignored and the stored response is reused verbatim, with a warning printed.
     - `BU-0419`: A pending response is only redelivered when its recorded generation matches the worker's current gate generation in both fleet state and the worktree, and its recorded response id matches on both sides; any mismatch refuses the operation.
     - `BU-0420`: When the recorded pane is live and its identity matches the worker's dispatch record, the worker response-delivery step notifies it and waits, with the lock released, for acknowledgement within a bound; on timeout it arms the delivery-escalation marker and reports the response as stored and recoverable rather than lost.
     - `BU-0421`: Once a live pane acknowledges delivery within the bound, the worker response-delivery step clears the unacknowledged-delivery marker and exits successfully without relaunching any worker.
     - `BU-0422`: When there is no live local pane and the relaunch metadata (tmux, session, window, agent) is incomplete, the worker response-delivery step leaves the response file ready on disk and exits successfully without attempting a relaunch.
   - Helper attachments (8):
     - `BU-0397`: The worker response-delivery step requires exactly two positional arguments, a task ID and a repo name; an invalid argument count is rejected before any other work happens.
     - `BU-0398`: A task ID supplied to the worker response-delivery step must match a restricted identifier pattern (starts alphanumeric, then alphanumeric/./_/- only); a malformed task ID is rejected.
     - `BU-0399`: A repo name supplied to the worker response-delivery step must match the same restricted identifier pattern as a task ID; a malformed repo name is rejected.
     - `BU-0400`: The worker response-delivery step reads the response body into a privately created temporary file: the file is created under a restrictive umask and explicitly chmod'd to owner-only before any content is written to it.
     - `BU-0401`: The private response-input temporary file is always removed when the worker response-delivery step exits, via a trap registered before the file is created.
     - `BU-0402`: The worker response-delivery step refuses to proceed if the response body read from standard input is empty.
     - `BU-0403`: The worker response-delivery step refuses to proceed if the named task does not exist in fleet state.
     - `BU-0404`: The worker response-delivery step refuses to proceed if the named repo does not exist under the named task in fleet state.

11. **`acknowledge-response`** — `BU-0158` (`docs/using-sergeant.md (docs/using-sergeant.md L275-281)`)
   - Trigger: the response-acknowledgement step is run after a worker applies a response
   - Outcome: sensitive transport is only cleared after private archival succeeds, and a retry after partial failure converges idempotently without re-applying the decision
   - Statement: The response-acknowledgement step validates post-application proof, stages replay evidence in a private archive entry (`0700` directory, `0600` files), records acknowledgement, and only then clears active plaintext transport; if a later archive-marker or transport-cleanup step fails, rerunning the same command with the same response ID must converge existing archive, acknowledgement markers, and active transport without reapplying the decision.
   - Stage-context attachments (14):
     - `BU-0186`: `<repo> has pending or incomplete response acknowledgement` means a response was delivered but never acknowledged; the only path that completes the handshake is resuming the worker and acknowledging with the response-acknowledgement step from that worker's own pane.
     - `BU-0436`: An acknowledgement is only accepted when it runs from the exact tmux pane recorded as owning the worker: both the calling process's own TMUX_PANE and its resolved pane identity must match the recorded dispatch pane.
     - `BU-0437`: The response-acknowledgement step acquires the response lock around the entire acknowledgement, and its exit trap also removes any partially-written archive staging directory and ack staging files left by an interrupted run.
     - `BU-0438`: Acknowledgement is refused unless the pending response id recorded in fleet state exactly equals the RESPONSE_ID argument.
     - `BU-0439`: Acknowledgement is refused unless the pending response's recorded generation is a valid positive integer.
     - `BU-0441`: If an archive entry already exists for this response id, acknowledgement requires it to be a complete, non-symlinked, non-retired directory recording all canonical fields whose recorded response id and generation match the pending response, and whose stored body is byte-identical to any still-present pending response transport.
     - `BU-0442`: When no archive entry exists yet, acknowledgement requires the worktree's own post-application proof file to exist and its recorded response id, gate generation, and status to exactly match the worktree's current live state before anything is archived.
     - `BU-0443`: A worker status of 'done' may only be acknowledged if the worktree also recorded a non-empty result; a done status with no result is refused.
     - `BU-0444`: A worker status of 'failed' must carry a non-blank reason to be acknowledged; a blank reason is refused.
     - `BU-0445`: A needs_input or blocked status may only be acknowledged once the worktree's own gate generation has advanced strictly past the generation the acknowledged response answered; the same generation is not sufficient.
     - `BU-0446`: A post-application proof carrying any status other than in_progress, done, a non-blank failed reason, needs_input, or blocked is refused outright.
     - `BU-0447`: The archive entry is assembled in a private staging directory (created via mkdir/chmod, contents chmod'd owner-only) and only renamed into its final location once every field has been written.
     - `BU-0448`: Acknowledging a response atomically publishes the ack marker to both fleet state and the worktree, each via a temp-file-then-rename, before any consumed response files are removed.
     - `BU-0449`: Once acknowledgement is published, the response-acknowledgement step removes the response body, its worktree mirror, and every worker-side response identity and applied-proof file.
   - Helper attachments (4):
     - `BU-0433`: The response-acknowledgement step requires exactly three positional arguments: task ID, repo, and response ID; any other count is rejected.
     - `BU-0434`: Each of the task ID, repo, and response ID arguments to the response-acknowledgement step must match a restricted identifier pattern; any one failing is rejected.
     - `BU-0435`: The response-acknowledgement step refuses to proceed if the worker's recorded worktree directory is unavailable.
     - `BU-0440`: The response archive directory is created privately: restrictive umask, then explicitly chmod'd to owner-only.

12. **`cleanup-fleet-task`** — `BU-0171` (`docs/using-sergeant.md (docs/using-sergeant.md L403-408)`)
   - Trigger: the fleet cleanup step is invoked for a task
   - Outcome: cleanup proceeds only once every named precondition holds, and never as a shortcut for a nonterminal worker state
   - Statement: Cleanup requires terminal/reconciled state, configured cleanup-owner proof for the repository/worktree or the treehouse session manager lease, preserved evidence, explicit cleanup-phase proof when replaying an interrupted removal or reconciling an already-absent worktree, fully acknowledged response transport, and no uncommitted or in-use worktree state; cleanup is never used to resolve a waiting, blocked, or orphaned worker.
   - Stage-context attachments (59):
     - `BU-0185`: Fleet files are never force-deleted or manually edited when cleanup refuses; cleanup safety depends on terminal proof, staged evidence, exact configured repository/worktree/lease identity, explicit cleanup phases for replayed/already-absent cases, proof of completed removal, and a response handshake that is either fully converged or explicitly retired, and cleanup intentionally refuses while any of that is only partially published.
     - `BU-0191`: The fleet cleanup step requires fleet state and worktree to be on the same filesystem because it uses atomic rename operations to move and restore evidence during terminal worker removal, and atomic rename does not cross filesystems; Sergeant refuses cross-filesystem layouts rather than falling back to a non-atomic copy+delete that could leave evidence in an inconsistent state.
     - `BU-0230`: The fleet cleanup step synchronizes origin tasks and refuses full fleet deletion until the callback-delivery step succeeds for that task.
     - `BU-0231`: Rejected events are intentionally unacknowledged and therefore also block cleanup until an operator repairs the consumer and runs the callback-delivery step.
     - `BU-0611`: If the task's fleet directory no longer exists, the fleet cleanup step reports the task as already cleaned up and exits successfully rather than treating a second cleanup invocation as an error.
     - `BU-0612`: The fleet cleanup step refuses to proceed with verifying that no process still has a worktree as its current directory if the lsof binary is unavailable, dying rather than silently skipping the check.
     - `BU-0613`: Before using a recorded worker PID's process group to terminate escaped descendants, the fleet cleanup step verifies the process's start time still matches the one recorded at dispatch, refusing (dying) if it does not, because the PID may have been reused by an unrelated process since it was recorded.
     - `BU-0614`: If a worker's recorded PID is no longer alive but live processes remain in its recorded process group, the fleet cleanup step only treats them as safe-to-terminate descendants when the worker was its own process-group leader (PID == PGID); otherwise it refuses termination and preserves diagnostics rather than guessing.
     - `BU-0615`: _stop_local_worker requires a stored pane_identity file before attempting to kill a recorded worker pane; if it is absent, the fleet cleanup step skips the pane kill because ownership cannot be verified, falling back to only recovering the escaped-descendant process group.
     - `BU-0616`: If the recorded worker pane is already reported dead, the fleet cleanup step still issues a kill-pane against it to remove it from remain-on-exit lingering, rather than leaving a dead pane sitting in tmux.
     - `BU-0617`: If a recorded pane no longer belongs to the expected worker (identity mismatch), the fleet cleanup step does not kill it — it only attempts to recover any escaped worker descendants by process group, leaving a foreign pane alone.
     - `BU-0618`: The fleet cleanup step gives a terminated worker process tree up to 5 seconds (polled at 0.1s intervals) to exit after TERM before escalating to KILL against both the recorded process-tree PIDs and the pane's process group.
     - `BU-0619`: After escalating to KILL, the fleet cleanup step gives the process tree up to 2 more seconds to exit before it dies with an explicit list of the processes that would not terminate, rather than proceeding with cleanup while they are still alive.
     - `BU-0620`: Even after the recorded worker's process tree is confirmed terminated, the fleet cleanup step separately verifies via lsof that no other process (of any origin) still has the worktree directory as its current working directory, and dies rather than removing a worktree something else is actively using.
     - `BU-0631`: The fleet cleanup step resolves the repository used for an owning td-task lookup from an owner identity the fleet itself recorded (or the currently configured project YAML), never from the directory the command happens to be invoked from, so terminal classification can never depend on caller cwd.
     - `BU-0632`: When a repo's owning repository was previously recorded, the fleet cleanup step re-verifies that the repository standing at that path today is still the identical repository (via a stable identity hash) before trusting it, rejecting a same-path repository that has since been replaced or re-cloned.
     - `BU-0633`: If a worktree is recorded and still present, the fleet cleanup step requires it to actually belong to the resolved owning repository before treating that repository as authoritative; a mismatch fails closed before any pane is stopped or evidence is published, not merely at the final removal gate.
     - `BU-0634`: The fleet cleanup step distinguishes an infrastructural failure to look up an owning task tracker task's status (missing td/jq binaries, an unreadable database, unparseable output, or no status field returned) from a genuine 'not yet closed' answer, reporting the former as its own explicit condition (return code 2) rather than silently downgrading it into a not-closed refusal.
     - `BU-0635`: The fleet cleanup step refuses to treat a repo as safely terminal when its recorded status is done but the result file is missing or empty, requiring a reconciled result before cleanup proceeds.
     - `BU-0636`: The fleet cleanup step only accepts an orphaned repo as cleanup-eligible when its owning task tracker task is verifiably closed, covering the case where a worker's pane exited after merge but before cleanup ran; every other orphaned repo is refused as not terminal.
     - `BU-0637`: If cleanup's persisted phase says a worktree was already removed but the worktree directory exists again on this retry, the fleet cleanup step dies rather than silently attempting to remove it again; a reappeared worktree after recorded removal is treated as an anomaly requiring investigation.
     - `BU-0638`: A retry that finds the cleanup phase mid-removal (worktree still present) but with no recorded retry-owner identity dies rather than proceeding, because the retry cannot be validated against an owner it never recorded.
     - `BU-0639`: Any persisted cleanup-phase value that does not match one of the recognized phase shapes is treated as malformed and fatal, rather than being interpreted loosely or ignored.
     - `BU-0640`: The fleet cleanup step requires the worktree's own terminal-status proof to match the fleet-recorded status exactly, and for a done status, requires the worktree's own result file to be byte-identical to the fleet-recorded result, refusing cleanup on any divergence between the two.
     - `BU-0641`: The first cleanup pass over a repo, with no persisted cleanup-phase yet, additionally verifies the worktree belongs to the currently configured repository owner whenever a project configuration is available, refusing to proceed on a mismatch; this check is skipped only for legacy fleets that predate project-config ownership tracking.
     - `BU-0642`: The fleet cleanup step refuses to proceed with any repo whose worktree has uncommitted changes (tracked or untracked, excluding the .sergeant-* evidence files), checked unconditionally before any pane is stopped or worktree removed.
     - `BU-0643`: The fleet cleanup step requires a repo's fleet-state directory and its worktree to live on the same filesystem, refusing to proceed for any repo (before any destructive action) if they differ, because cleanup relies on atomic same-filesystem rename to preserve and replay terminal evidence.
     - `BU-0644`: When staging a worktree's terminal (.sergeant-*) evidence into fleet state, the fleet cleanup step re-computes the evidence identity for both the source worktree and the staged copy and requires both to match the identity expected at the start of staging, rolling back the staging directory on any mismatch rather than trusting the copy blindly.
     - `BU-0645`: Rolling back a cleanup transaction directory verifies with a follow-up existence check that the path is actually gone after rm -rf, dying with a CRITICAL message naming the preserved artifact if it is not, rather than trusting rm's exit status alone.
     - `BU-0646`: The fleet cleanup step refuses to move a worker-evidence entry onto a destination that is an existing directory or a symlink, because mv would silently move the entry inside the directory (or through the link) rather than replacing it, making the evidence effectively unfindable; an existing plain file at the destination is replaced instead, since the identity-bound evidence being moved should win over an unbound file underneath it.
     - `BU-0647`: Before removing a worktree's live .sergeant-* evidence, the fleet cleanup step re-verifies the evidence's identity still matches what was expected at the point of transition; if removal fails partway through, it rolls back only the specific entries already moved, tracked as it goes, restoring the worktree to its exact prior state rather than leaving a partially-emptied evidence set.
     - `BU-0648`: Restoring persisted terminal evidence back into a worktree on a retry re-verifies the evidence's identity at least three separate times, before staging, after staging, and against the current live state, aborting the whole restore transaction if any of those checks disagree, rather than trusting the evidence unchanged across the whole operation.
     - `BU-0649`: If publishing restored evidence entries fails partway through, the fleet cleanup step rolls back every entry already published back to a fleet-owned backup, so an interrupted evidence restore is never left half-applied in the worktree.
     - `BU-0650`: When fleet state and a worktree are on different filesystems, the fleet cleanup step restores backed-up evidence entry by entry via copy-to-temp-then-rename rather than one atomic move, first checking that neither the destination nor its own temp path already exists, and rolling back every already-published entry if any entry fails.
     - `BU-0651`: Every write to the durable cleanup-phase checkpoint is verified by re-hashing the written temp file, using --no-filters so a repository-level clean/eol attribute over fleet state cannot make the verification disagree with what was written, against the expected hash before the temp file is published to the real path; a mismatch aborts the write and rolls back rather than publishing unverified checkpoint state.
     - `BU-0652`: A cleanup retry rejects a repo whose retry-recorded worktree, root, or removal type diverges from what was originally captured as the cleanup owner for that repo, before doing anything else on the retry path.
     - `BU-0653`: A cleanup retry re-derives the task's currently-resolved project and refuses to proceed if it differs from the project recorded when cleanup was first attempted for that repo.
     - `BU-0654`: A cleanup retry cross-checks the currently configured repository root, when one resolves, against the originally recorded owner root, refusing on mismatch, but tolerates a project whose configuration no longer names any repos at all only when the task itself no longer resolves to a project.
     - `BU-0655`: If the worktree is still present on a cleanup retry, the fleet cleanup step re-derives its full checkout identity (HEAD, index, git-dir metadata, working-tree diff) and refuses to proceed if it no longer matches the identity recorded when cleanup first captured it; a worktree that changed underneath an interrupted cleanup is not trusted.
     - `BU-0656`: The fleet cleanup step re-verifies the identity of the terminal evidence it persisted to fleet state on every retry, refusing to proceed if that persisted evidence no longer matches what was recorded, regardless of whether the live worktree still exists.
     - `BU-0657`: If live worker evidence is still present in the worktree during a retry, its identity must also still match the originally recorded evidence identity, refused otherwise.
     - `BU-0658`: The fleet cleanup step re-derives the owning repository's own stable identity (instance marker, remote URL, root commits) on every retry and refuses to proceed if it no longer matches what was recorded, detecting a repository that was moved, aliased, or replaced since cleanup began.
     - `BU-0659`: For a treehouse-leased worktree, a cleanup retry additionally requires the expected lease-holder name, derived from the task and repo, to match both the live wt_holder file and the value recorded when cleanup first captured ownership, refusing to proceed on any mismatch.
     - `BU-0660`: The fleet cleanup step proves a git worktree removal actually completed by parsing the full 'git worktree list --porcelain -z' output through a strict validator that rejects any record shape it does not recognize; it only accepts the removal as proven when parsing succeeds cleanly and the target path appears exactly zero times among the registered worktrees.
     - `BU-0661`: The fleet cleanup step refuses (dies) rather than attempting to prove a treehouse-leased worktree's removal via the git-worktree-registry check, because that proof mechanism does not apply to the treehouse session manager leases.
     - `BU-0665`: The fleet cleanup step acquires a per-repo response lock before performing any response-safety check or destructive action on that repo, and installs an EXIT trap to release it as a backstop, releasing it explicitly at every normal loop-continuation point as well.
     - `BU-0666`: If a repo's worktree is already absent but its recorded pane is still reported alive after the stop sequence ran, the fleet cleanup step refuses to proceed with absent-worktree cleanup for that repo, treating a live pane it could not verify ownership of as a hazard rather than assuming the stop simply wasn't needed.
     - `BU-0667`: The fleet cleanup step refuses to proceed with absent-worktree cleanup for a repo that still has a recorded validation_pane file, requiring the validation pane to be stopped first.
     - `BU-0668`: When a retry's absent-worktree path can prove an interrupted git worktree removal already completed via the worktree-registry proof, the fleet cleanup step records a 'reconciled-absent' cleanup phase and moves on rather than re-attempting a removal that already succeeded.
     - `BU-0669`: An interrupted treehouse-leased worktree removal has no proof mechanism the fleet cleanup step can use on retry to confirm whether the return actually completed; unlike a git worktree, cleanup dies rather than guessing, leaving the state for manual reconciliation.
     - `BU-0670`: A repo's first cleanup pass, with no persisted retry state yet, requires the current project YAML to resolve that repo's owning repository root; there is no fallback derived from the worktree's own path, so cleanup fails closed if the project configuration is missing or the repo entry has been renamed since dispatch.
     - `BU-0671`: For a treehouse-leased worktree, the fleet cleanup step identifies the lease by reading the worktree's .git file and requires the derived main-repository path to match the resolved cleanup-owner root; a mismatch preserves fleet state and refuses to proceed rather than attempting to return a lease against the wrong repository.
     - `BU-0672`: When a worktree removal command reports failure, the fleet cleanup step distinguishes two outcomes: if the worktree directory is actually gone, it durably records a retryable 'partial-removal' phase and dies for a retry to pick up; if the directory still exists, it restores the moved-out worker evidence and preserves fleet state entirely, refusing to proceed.
     - `BU-0673`: A worktree-removal command reporting success is not trusted without also verifying the directory is actually gone afterward; if it remains, the fleet cleanup step restores the moved-out worker evidence and dies rather than reporting cleanup complete.
     - `BU-0674`: After a worktree removal completes, the fleet cleanup step re-runs the same retry-owner validation and removal-proof check a cold retry would use before recording the final reconciled-absent phase, so the checkpoint it leaves behind is provably consistent with what a future retry would independently re-derive.
     - `BU-0675`: The fleet cleanup step only removes the task's fleet-state directory when cleaning up every repo in the task (FILTER_REPO unset); invoking cleanup for a single named repo never removes the shared task directory even after that repo's own state is fully cleaned.
     - `BU-0678`: The fleet cleanup step stops a task's registered background monitor bound to the exact invocation id stored for it, so it can never terminate a different unit that has since taken the same name; this stop is skipped entirely when cleaning a single filtered repo, because the monitor is task-scoped rather than repo-scoped.
     - `BU-0679`: The fleet cleanup step calls the callback-delivery step sync, and if not filtering by repo also the callback-delivery step check-acked, at the very start of the run whenever the task carries a callback origin marker, before any worktree or pane state is touched.
     - `BU-0805`: A task is only reported as fully acknowledged if every one of its callback events has reached the acknowledged state — any pending, delivering, or rejected event makes the whole check fail, naming how many events remain outstanding.
   - Helper attachments (3):
     - `BU-0608`: The fleet cleanup step refuses to proceed if the task-id argument is empty, '.', '..', an absolute path, or contains a path separator, before that value is ever used to build a fleet-state path.
     - `BU-0609`: The fleet cleanup step refuses a task-id whose fleet directory entry is a symlink, rather than following it into fleet state elsewhere.
     - `BU-0610`: The fleet cleanup step verifies the task directory's canonicalized real path is exactly $FLEET_ROOT/$TASK_ID before proceeding, rejecting any traversal or symlink aliasing that would make the resolved path diverge from the literal task-id.

13. **`retire-response-handshake`** — `BU-0187` (`docs/troubleshooting.md (docs/troubleshooting.md L173-180)`)
   - Trigger: cleanup considers retiring a handshake because the worker appears gone
   - Outcome: retirement requires two independently re-verified conditions (closed owning task, provably dead worker) on every attempt, not a one-time check
   - Statement: When the worker is gone for good, cleanup can retire the handshake instead, but only when, re-checked on every attempt, both hold: the owning task tracker task is closed, and the recorded worker is provably dead (pane gone/dead with matching identity, recorded PID not running, no process in its recorded process group, and worker_pid/worker_process_start/worker_process_group all recorded).
   - Stage-context attachments (10):
     - `BU-0188`: The refusal names which condition failed (e.g. process still alive, PID reused, pane identity mismatch, owning task tracker task not closed); a live, PID-reused, or identity-mismatched owner is always refused, since it is never correct to retire a handshake underneath a worker that might still finish it.
     - `BU-0189`: Retirement records the exact partial state under `~/.local/share/sergeant/fleet/<task>/<repo>/response-retirement/` before mutating anything (verbatim copies of both sides, owner death evidence, and provable response-archive fields), never writes an acknowledgement, and marks a `retired` directory so the archive can never be read as one; the archive shares the fleet task's lifetime so a retried cleanup converges.
     - `BU-0190`: Cleanup refuses a retirement archive that no longer describes the state it preserved — a changed response, a tampered or symlinked copy, or a drifted recorded owner — rather than trusting stale evidence.
     - `BU-0624`: The fleet cleanup step only retires an unfinished response handshake without an acknowledgement when the owning task tracker task is closed and the worker that owned the handshake is provably dead by every check available (recorded pane identity, process liveness, process group); this is the single path in the file allowed to bypass the acknowledgement requirement.
     - `BU-0625`: Every refusal to retire a response handshake is fail-closed, including cases that merely cannot be proven: incomplete worker process provenance, a still-live or non-matching recorded pane, a live or PID-reused recorded process, or live processes remaining in the recorded worker's process group.
     - `BU-0626`: Before publishing a response-handshake retirement archive, the fleet cleanup step re-computes the digest of both the fleet-side and worktree-side response state and refuses to publish (rolling back instead) if either digest no longer matches what was captured, so a state change mid-archive can never be published as the preserved evidence.
     - `BU-0627`: On a retry, the fleet cleanup step treats an existing response-retirement archive as valid evidence only if re-deriving its fleet-state digest, worktree-state digest, archive-fields digest, full manifest, and owner record from the files on disk right now reproduces exactly what the archive recorded; any divergence is refused rather than trusted.
     - `BU-0628`: The fleet cleanup step writes an explicit 'retired, not an acknowledgement' marker file as part of every response-retirement archive, covers it by the same digest as the archive's real fields, and applies the same 0600 permission discipline the response-acknowledgement step uses, so a partial handshake that happens to carry every field a real acknowledgement has can still never be read as one.
     - `BU-0629`: If rolling back a failed retirement-archive transaction itself fails, the fleet cleanup step reports a distinct CRITICAL exit path (return code 3) naming the preserved artifact to inspect, rather than folding the failure into the ordinary 'handshake not retired' refusal message.
     - `BU-0630`: The fleet cleanup step only allows cleanup of a repo carrying any response-handshake artifact (fleet or worktree side) to proceed when that handshake is either fully acknowledged (matching response_id, generation, and both ack markers, and a validated archive entry) or has gone through explicit retirement; every other shape is refused.
   - Helper attachments (1):
     - `BU-0501`: A response-archive entry is only considered complete when it is an unsymlinked directory containing every one of the four canonical fields (body, gate_generation, applied_status, proof) as unsymlinked regular files, and carries no retirement marker.

14. **`seal-before-deletion`** — `BU-0232` (`docs/callbacks.md (docs/callbacks.md L176-178)`)
   - Trigger: the fleet cleanup step is about to delete fleet state for a task with callback events
   - Outcome: a re-verified, locked, sealed check closes the race window between checking and actually deleting
   - Statement: Immediately before fleet deletion, the fleet cleanup step takes the callback lock, verifies the acknowledgement condition again, and writes a terminal seal that rejects new event generations and closes the acknowledgement-check/deletion race.
   - Stage-context attachments (8):
     - `BU-0233`: If cleanup fails after sealing and the fleet must resume, an operator removes only the seal with the callback-delivery step.
     - `BU-0676`: Before deleting the task's fleet state, the fleet cleanup step seals the task's callback origin, only if an origin.json marker is present, so callback finalization happens while the evidence it depends on still exists.
     - `BU-0677`: The fleet cleanup step writes a wiki activity log entry recording the task's final status and result immediately before deleting fleet state, because fleet state is the only place that status and result live; waiting any later would lose the ability to report them.
     - `BU-0779`: A callback event cannot be enqueued for a task whose callback directory has been sealed for cleanup.
     - `BU-0806`: A callback event cannot be retried inside a task whose callback directory has been sealed for cleanup.
     - `BU-0810`: A task's callback directory cannot be sealed for cleanup while any of its callback events remains unacknowledged.
     - `BU-0811`: Sealing a task's callback directory is idempotent given an existing valid seal, but an existing seal with unexpected content is treated as an error rather than silently accepted or overwritten.
     - `BU-0812`: Unsealing a task validates the existing seal's content before removing it — an unrecognized seal value is rejected rather than blindly removed — and unsealing a task that is not currently sealed is a no-op.

15. **`terminate-worker-process`** — `BU-0347` (`bin/sgt-interactive-worker (bin/sgt-interactive-worker L174-182)`)
   - Trigger: _drain_terminate signals the process group
   - Outcome: termination never signals processes outside the worker's own ownership just because they share an ambient process group id
   - Statement: The termination handler only sends a signal to the worker's own process group when this worker process actually leads that group; if it does not, it falls back to terminating only the worker shell and lets that shell's own EXIT path clean up, because a group it does not lead may contain processes it does not own.
   - Stage-context attachments (7):
     - `BU-0349`: The termination watcher's double guard mirrors _finish's own rule: it only stops the Claude background session on status=done together with a non-empty result; a done status with an empty result is treated as an orphaned mission still in progress, never as completion.
     - `BU-0350`: The termination watcher does not stop the background session on a done status with an empty result; it explicitly lets _finish handle that empty-result (orphaned) case whenever attach exits by other means.
     - `BU-0351`: Any status matching failed:<reason> is treated as terminal by the termination watcher regardless of the result field's state, and stops the background session.
     - `BU-0491`: The stalled pane's current identity is re-verified to still match its recorded owner before it is allowed to be killed; a mismatch refuses recovery without killing anything.
     - `BU-0564`: Before stopping a recorded Claude background session, if both a recorded and a live session id are available for that background id, a mismatch — meaning the recorded id has since been reused by an unrelated session — skips the stop call, so an unrelated live session is never accidentally stopped.
     - `BU-0565`: The background-session id cross-check is deliberately best-effort, not fail-closed: any unresolvable verification (no session id yet persisted, jq unavailable, or the live-session query itself failing) still lets the stop call proceed, because preventing a genuinely leaked live session from running forever must not be defeated by an unrelated, transient verification failure — only a confirmed, positive id mismatch skips the call.
     - `BU-0566`: Every termination path that kills a worker's pane or process must also call the background-Claude-session stop helper, because a background Claude session is not a child of the worker's process group and is invisible to process-tree or tmux kill-pane signals — omitting the call would silently leak the session.

16. **`worker-exit-cleanup`** — `BU-0354` (`bin/sgt-interactive-worker (bin/sgt-interactive-worker L458-476)`)
   - Trigger: the worker process is exiting for any reason
   - Outcome: every background loop and the Claude background session are stopped on every exit path, including a clean completion no external script observes
   - Statement: On exit, the worker's _finish handler kills every background loop it started (notification delivery, progress watch, drain watch, termination watch) and stops any Claude background session as a ninth, in-process termination backstop independent of the eight external termination paths, so no background process leaks regardless of which exit path fired.
   - Stage-context attachments (3):
     - `BU-0355`: At the worker's exit boundary, the accepted action lease is always settled — finalized from the agent's own completion proof, or explicitly recorded as pending with the exit reason — covering every terminal status (done, failed, drained, needs_input, blocked, waiting, orphaned) so no branch can exit leaving the lease silently outstanding.
     - `BU-0356`: On exit, a status of done together with an empty .sergeant-result is downgraded to orphaned rather than accepted as a completed mission.
     - `BU-0358`: When the agent itself publishes a completion proof for a notification, the worker settles the accepted action lease through the one shared finalizer (which re-verifies identity, lease, and proof under the response lock) rather than a second, separately-maintained inline implementation.

17. **`claim-action-lease`** — `BU-0359` (`bin/sgt-interactive-worker (bin/sgt-interactive-worker L608-626)`)
   - Trigger: the acceptance path is about to claim the action lease for a notification target
   - Outcome: at most one nonce ever holds the action lease for a given notification — a second target cannot silently steal or duplicate acceptance
   - Statement: When the notification delivery loop is about to accept a notification target, it re-checks whether an action lease already exists for a different nonce and refuses to overwrite it; only when no lease exists does it atomically claim the lease for the current nonce via mktemp+mv.
   - Stage-context attachments (5):
     - `BU-0425`: Before relaunching, an outstanding notification action lease from a prior supervisor blocks relaunch unless the shared finalizer can prove the agent's own durable completion proof for that exact turn; an unprovable lease refuses the relaunch rather than fabricating completion.
     - `BU-0426`: Before overwriting notification state for a relaunch, any existing pending notification target's pane identity is preserved as superseded evidence; if that evidence conflicts with a previously recorded record, the relaunch is refused.
     - `BU-0493`: Before overwriting notification target state for the relaunch, any existing pending notification target's pane identity is preserved as superseded evidence; a conflict with previously recorded evidence refuses the recovery.
     - `BU-0909`: Before the active-notification pointer advances to a new notification id, evidence that the outgoing notification was acknowledged and delivered (including which pane identity received it) is durably captured into per-notification proof files, written once and only once (guarded by the proof file's own existence).
     - `BU-0910`: When the active notification id changes, any stale worktree acceptance marker left over from the prior notification is deleted, so it can never be misread as consent for the new notification.
   - Helper attachments (8):
     - `BU-0500`: The response-archive field reader accepts a record only when the requested key appears in it exactly once with a non-empty value; a missing, duplicated, or empty key causes the read to fail rather than returning a guessed or partial value.
     - `BU-0502`: A response-archive entry only 'matches' a given response id and gate generation when every one of the entry's own recorded fields and its embedded proof's own fields agree with each other and with the caller's values; a field defaulting to empty is explicitly range-checked so it can never silently satisfy the comparison.
     - `BU-0509`: An action lease is considered complete only when the worktree's own durable per-nonce completion file contains the exact literal '<notification_id>|<nonce>' token; nothing else, however plausible, satisfies completion.
     - `BU-0510`: Finalizing an action lease never fabricates completion: a malformed lease nonce, a missing notification-target directory, or an agent proof that does not exactly match the expected token each fails closed and records the specific pending reason, rather than guessing at completion.
     - `BU-0511`: Finalization is idempotent: a lease whose completion token is already recorded as matching the expected value is reported as finalized without attempting to write anything again.
     - `BU-0512`: Every premise for publishing lease completion (the notification id, the lease nonce, and the agent's proof) is re-verified under the response lock immediately before the write, because a concurrent supersede could have replaced any of them since the caller's own earlier checks.
     - `BU-0513`: If the response lock is already held by another context within this same process, finalization does not attempt to wait for it — since the liveness check would see its own live PID forever — and instead records the pending reason for a later exit-boundary call to settle.
     - `BU-0514`: A lease-outcome record (pending or finalized) is written exactly once per record name and is never overwritten by a later finalization attempt, so the first, most-proximate reason for an outstanding lease survives every later retry.

18. **`migrate-legacy-response-state`** — `BU-0406` (`bin/sgt-respond (bin/sgt-respond L70-73)`)
   - Trigger: the recorded worktree is not yet recognized as an owned checkout and migration is attempted
   - Outcome: migration only proceeds from a worktree whose identity is provably genuine
   - Statement: Legacy response-state migration requires the recorded worktree path to be an absolute, existing, symlink-free directory whose canonical path is itself, containing an unsymlinked .git file; any deviation aborts migration without changing state.
   - Stage-context attachments (5):
     - `BU-0407`: Legacy response-state migration requires the branch recorded in fleet state to exactly equal the worktree's actual checked-out branch; a mismatch aborts migration.
     - `BU-0408`: Legacy response-state migration requires the worktree's own brief file to literally contain matching Task ID, Project, Repo, Branch, Worktree, and Fleet-state lines; any missing or differing line aborts migration.
     - `BU-0409`: Legacy response-state migration requires the project configuration's repo entry to resolve to a path sharing the exact same git common directory as the worktree; a mismatch aborts migration.
     - `BU-0410`: Before mutating any state, legacy migration records durable evidence of the exact identity facts it verified, and if evidence was already recorded for this repo, migration proceeds only when the newly computed evidence is byte-identical to it, refusing otherwise.
     - `BU-0411`: Legacy response-state migration only fills in tracked pointer and intent files that are currently absent; it never overwrites a pointer or intent file that already exists.

19. **`relaunch-superseded-worker`** — `BU-0427` (`bin/sgt-respond (bin/sgt-respond L469-480)`)
   - Trigger: a relaunch is about to start a new worker supervisor pane
   - Outcome: two Claude processes are never running concurrently against the same worktree
   - Statement: Before dispatching a replacement worker, any live Claude background session left by the superseded worker is stopped first — the stop always happens before the new dispatch, never after.
   - Stage-context attachments (11):
     - `BU-0428`: If tmux fails to create the relaunch window/pane, the worker response-delivery step records the worker as orphaned (with the failure reason in the diagnostic) and releases the drain admission lock before dying.
     - `BU-0429`: The relaunched pane's identity is published and a notification target is created for it; if that creation detects a race, the just-started background session is stopped, the new pane is killed, the worker is recorded orphaned, and the worker response-delivery step dies.
     - `BU-0430`: A superseded live pane is killed only after the replacement pane is fully live, its identity published, and its notification target created.
     - `BU-0431`: If the relaunched worker's pane does not acknowledge its delivery notification within the bound, the background session and pane are torn down, the worker is recorded orphaned, and the worker response-delivery step dies rather than declaring the relaunch successful.
     - `BU-0432`: On a fully successful relaunch, the worker response-delivery step clears both the drain_held and response_delivery_unacked markers.
     - `BU-0494`: Before launching the replacement worker pane, any live Claude background session left by the stalled worker is stopped first, always before the new dispatch, never after.
     - `BU-0495`: The replacement pane is launched before the stalled pane is ever killed; if the launch fails, the original stalled pane is left completely untouched and recovery escalates to needs_input for investigation.
     - `BU-0496`: New pane identity publication and notification-target creation are validated before anything else proceeds; on any failure the new pane is killed, the pane record is restored to the original stalled pane, and recovery escalates — the worker is never left pointing at a pane that was never confirmed live.
     - `BU-0497`: The old stalled pane is only killed once the replacement pane is fully launched, identity-confirmed, and holds an active notification target.
     - `BU-0498`: If the relaunched worker's pane fails to acknowledge the recovery notification within the bound, its background session and pane are torn down and recovery escalates to needs_input rather than being declared successful.
     - `BU-0567`: A recorded Claude background session is only treated as provably live when its queried state is exactly 'working' or 'blocked'; a missing id, an unresolvable binary, jq being unavailable, or any other state is treated as 'not provably live', which is what authorizes proceeding (e.g. dispatching a replacement worker) rather than blocking on an unconfirmable session.

20. **`force-stop-worker`** — `BU-0530` (`bin/sgt-drain-force (bin/sgt-drain-force L48-56)`)
   - Trigger: the forced-drain step is invoked
   - Outcome: the command dies with an error if no matching drain is active
   - Statement: The forced-drain step refuses to run unless a matching drain (global, or the named project) is already active; force never operates without an antecedent cooperative drain having been set.
   - Stage-context attachments (8):
     - `BU-0531`: Force-stopping requires either --dry-run (preview only, no termination) or an explicit --yes; invoking it with neither is refused.
     - `BU-0532`: Workers whose recorded status is already done, failed:*, drained, force-stopped, or orphaned are excluded from force-stop eligibility entirely.
     - `BU-0533`: --dry-run prints the exact set of eligible force-stop targets (task, repo, worktree, pid) without terminating anything.
     - `BU-0534`: Before signalling a worker's recorded PID, the forced-drain step always stops any recorded Claude background session first, idempotently, because a background Claude session is not a child of the worker's process group and is invisible to process-group signalling.
     - `BU-0535`: Before sending any kill signal to a worker's recorded PID, the forced-drain step re-verifies that PID's recorded process-start time against its actual current start time, to prevent killing an unrelated process that has since reused the same PID.
     - `BU-0536`: When a PID-reuse mismatch is detected, the original worker is marked force-stopped without any signal being sent to the now-unrelated process holding that PID.
     - `BU-0537`: Force-stop escalates from SIGTERM to SIGKILL only if the process is still alive after waiting up to five seconds (fifty 0.1s polls); SIGKILL is never sent immediately.
     - `BU-0538`: A SIGTERM send failure for one worker is recorded and that worker is marked failed, but force-stop continues attempting the remaining eligible workers rather than aborting the whole batch; the run still exits nonzero overall so the operator knows manual recovery is needed.

21. **`recycle-terminal-worker-pane`** — `BU-0581` (`bin/sgt-watch (bin/sgt-watch L282-298)`)
   - Trigger: the interactive fleet-watch loop evaluates whether a terminal worker's pane has already been recycled
   - Outcome: a relaunch that rebinds pane/pane_identity is correctly treated as needing its own recycling, rather than being permanently suppressed by an older marker
   - Statement: A repo's worker_recycled evidence only counts as covering the current pane if it names that exact pane identity; a marker written for an earlier pane never suppresses recycling of a later, different pane that replaced it.
   - Stage-context attachments (9):
     - `BU-0582`: Recording recycle evidence both rebinds the current pointer file (pane, identity, outcome, timestamp) and appends an entry to an append-only worker_recycled_log, so no earlier recycling record is ever lost even though the pointer file itself is overwritten on each recycle.
     - `BU-0583`: Before a terminal worker's pane is recycled, the interactive fleet-watch loop settles any outstanding accepted action lease on the worktree first, because recycling used to stop the only process that could ever publish completion, which made a completed-but-unpublished turn permanently unrecoverable.
     - `BU-0584`: The interactive fleet-watch loop stops any Claude background session associated with a repo before recycling its pane, because a background Claude session is not a child of the pane's process group and is invisible to tmux kill-pane; the stop call is idempotent so repeated recycling attempts are safe.
     - `BU-0585`: If tmux is unavailable, the interactive fleet-watch loop cannot recycle a terminal worker's pane; it records a diagnostic explaining why and reports failure rather than silently treating the pane as already retired.
     - `BU-0586`: The interactive fleet-watch loop determines a recorded pane is truly gone by comparing the pane id tmux display-message actually returns against the expected pane id, not by trusting the command's exit status, because display-message against a gone pane silently falls back to a default target instead of failing.
     - `BU-0587`: The interactive fleet-watch loop refuses to kill a recorded pane unless its live identity still verifies as the expected supervisor, recording a diagnostic and refusing recycling rather than killing an unverified pane.
     - `BU-0588`: After issuing kill-pane, the interactive fleet-watch loop re-checks that the pane id is actually gone before recording the recycle as successful, rather than trusting the kill command's exit status alone.
     - `BU-0598`: The interactive fleet-watch loop only recycles a worker's pane for the terminal states done, failed, and drained; the nonterminal and unreconciled states in_progress, needs_input, blocked, waiting, and orphaned are deliberately excluded because they are still resumable and must fail closed rather than lose their pane.
     - `BU-0605`: A pane no longer matching its recorded identity is never killed by the recycler — the interactive fleet-watch loop only attempts to recover any escaped worker descendants by process group, leaving a foreign pane untouched.

22. **`classify-stalled-worker`** — `BU-0591` (`bin/sgt-watch (bin/sgt-watch L416-426)`)
   - Trigger: the interactive fleet-watch loop classifies an in_progress worker as stalled or active
   - Outcome: a worker that is actually producing tool-call or streamed output is never misclassified as stalled merely because progress_ts happens to be older
   - Statement: Stall detection's authoritative liveness signal is live tmux pane-activity output (any terminal output from the agent, including tool calls), taking precedence over the progress_ts file, because a process-tree-based check would incorrectly count the interactive worker's own delivery loop as meaningful activity.
   - Stage-context attachments (3):
     - `BU-0590`: Clearing a stall diagnostic only removes it when the current diagnostic text is exactly an owned 'live worker stalled:' marker; any other diagnostic (orphan, dispatch-failure, etc.) is left untouched.
     - `BU-0592`: Stall classification never changes a repo's status field — it only ever rewrites the diagnostic for an in_progress repo whose pane has already passed live identity verification, so a stalled worker remains resumable rather than being forced into a terminal state.
     - `BU-0593`: The interactive fleet-watch loop only rewrites the 'live worker stalled' diagnostic when the stall is newly detected or the elapsed time crosses into a new bucket (default 60s, SERGEANT_STALL_DIAG_BUCKET), rather than on every reconciliation pass, to avoid a full watch redraw on every sync-interval tick.

23. **`reconcile-incomplete-dispatch`** — `BU-0595` (`bin/sgt-watch (bin/sgt-watch L484-509)`)
   - Trigger: a dispatched repo's grace period expires and no owned live pane can be found
   - Outcome: evidence of committed work is surfaced to the operator rather than silently discarded behind a generic failure message
   - Statement: When an in-progress dispatch's grace period expires with no owned live pane, the interactive fleet-watch loop checks the worktree for commits made above the dispatch base; if any exist, it marks the repo failed with a message directing the operator to reconcile the preserved branch before re-dispatch, rather than the generic 'no worktree or pane acquired' failure.
   - Stage-context attachments (2):
     - `BU-0589`: The grace period before an incomplete dispatch is considered expired is configurable via SERGEANT_DISPATCH_GRACE_SECONDS (default 300) and the interactive fleet-watch loop dies if that value is not a non-negative integer, rather than silently falling back to a default.
     - `BU-0594`: The interactive fleet-watch loop's committed-work log for a dispatch prefers the range from the dispatch-recorded initial_sha to HEAD, and only falls back to diffing against the branch's upstream tracking ref for fleet state that predates initial_sha being recorded; it produces nothing (not an error) when git is unavailable or no upstream is configured.

24. **`stop-validation-pane`** — `BU-0621` (`bin/sgt-cleanup (bin/sgt-cleanup L314-318)`)
   - Trigger: _stop_validation_pane is called for a repo with a recorded validation_pane
   - Outcome: a validation pane with incomplete ownership provenance is never terminated on an assumption
   - Statement: The fleet cleanup step refuses to stop a recorded validation pane whose PID, process-group, and start-time provenance are not all recorded together, dying rather than guessing which process to signal.
   - Stage-context attachments (5):
     - `BU-0622`: Before terminating a live validation pane, the fleet cleanup step re-verifies the pane's live identity against the recorded validation-pane identity and dies immediately on any mismatch, rather than terminating a pane that may no longer be the validation worker.
     - `BU-0623`: Before terminating a validation checkout's process group, the fleet cleanup step verifies the recorded owning PID's start time still matches what was recorded, refusing (dying) if the PID appears to have been reused.
     - `BU-0662`: The fleet cleanup step validates a recorded validation checkout's ownership provenance before any destructive step (pane kills, worktree removal) runs for that repo, so that rejecting an invalid validation checkout leaves the live worker pane and fleet evidence completely unchanged.
     - `BU-0663`: The fleet cleanup step recognizes exactly two shapes of validation-checkout ownership provenance, a full four-field 'exact' record or a legacy single-head record with none of the four fields, and treats any other combination of present and absent provenance fields as invalid rather than guessing which shape was intended.
     - `BU-0664`: Verifying a validation checkout's identity requires its own git-common-dir to carry a sergeant-validation-owner file containing the exact expected owner string, in addition to matching path identity, git-dir identity, and HEAD; path or content equivalence alone is not sufficient to prove ownership.

25. **`start-background-monitor`** — `BU-0919` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L1085-1091 (_sgt_background_watch))`)
   - Trigger: _sgt_background_watch runs on a host without systemd user services
   - Outcome: the call dies immediately with an actionable alternative rather than partially starting a monitor it cannot manage
   - Statement: Starting a managed background monitor fails fast, before touching any state, if the required systemd-run or systemctl tooling is not available, naming the foreground the interactive fleet-watch loop command as the alternative.
   - Stage-context attachments (3):
     - `BU-0920`: Starting a background monitor is idempotent: if a unit with the deterministic per-task name is already active, its live InvocationID is adopted and the ownership files are refreshed, rather than attempting to start a duplicate unit (which would fail with 'unit already exists').
     - `BU-0921`: A monitor's ownership files are written invocation-id-first, then unit-name second, so that a crash between the two writes leaves monitor_unit absent; a later cleanup pass that finds monitor_unit missing silently skips rather than dying on an incomplete/missing invocation id.
     - `BU-0922`: After starting a new monitor unit, its InvocationID is read with a bounded retry (up to 20 attempts at 0.1s intervals) because systemd assigns the InvocationID asynchronously after the unit becomes active; if it never appears, the call dies.

26. **`stop-background-monitor`** — `BU-0923` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L1182-1185 (_sgt_stop_background_monitor))`)
   - Trigger: _sgt_stop_background_monitor finds a registered unit name but no stored invocation id
   - Outcome: the call refuses to stop anything and reports the inconsistency instead of guessing
   - Statement: Stopping a background monitor dies rather than proceeding when a monitor unit is registered but its stored invocation id is missing, because the identity needed to safely target the stop cannot be established.
   - Stage-context attachments (1):
     - `BU-0924`: Stopping a background monitor refuses to act, and dies with a diagnostic, if the unit's current live InvocationID differs from the one stored when the monitor was started, because a different process instance now holds that unit name (a TOCTOU/unit-reuse hazard, analogous to PID reuse).

### `adopt-external-skill`

**Single-behavior workflow candidate**, established entirely by `BU-0119`
(`docs/skills.md (docs/skills.md L124-131)`) — no `stage`/`stage-context`/`helper` record in this
corpus carries a `workflow` field matching `adopt-external-skill` (checked against
that source file's other extracted units and against every `workflow`
field value used corpus-wide). Reported as a single-behavior candidate
per synthesis-method.md's "what must not happen" rather than padded out
or merged into an adjacent-topic cluster it does not actually name.

- **Trigger:** an external skill is being adopted
- **Outcome:** the six-step vetting procedure is completed before broad installation
- **Statement:** Before adopting an external skill: read its complete SKILL.md and referenced scripts, confirm its source and update mechanism, check its filesystem/shell/network/Git/credential actions, verify no conflict with `AGENTS.md` or safety policy, pin or lock its source where supported, and test it in a disposable repository or worktree before broad installation.
- **Completion condition:** the full six/five-part procedure named in the
  statement above has been executed, per `BU-0119`.

No member stage candidates (none exist for this workflow name).

### `invoke-grill-with-docs`

**Single-behavior workflow candidate**, established entirely by `BU-0969`
(`.agents/skills/grill-with-docs/SKILL.md (.agents/skills/grill-with-docs/SKILL.md L7-7)`) — no `stage`/`stage-context`/`helper` record in this
corpus carries a `workflow` field matching `invoke-grill-with-docs` (checked against
that source file's other extracted units and against every `workflow`
field value used corpus-wide). Reported as a single-behavior candidate
per synthesis-method.md's "what must not happen" rather than padded out
or merged into an adjacent-topic cluster it does not actually name.

- **Trigger:** the grill-with-docs skill is invoked
- **Outcome:** the resulting interview both stress-tests the plan and leaves behind ADR/glossary docs
- **Statement:** Invoking grill-with-docs runs a /grilling session using the /domain-modeling skill, so the interview it drives also produces ADR and glossary documentation as it goes.
- **Completion condition:** the full six/five-part procedure named in the
  statement above has been executed, per `BU-0969`.

_Relation note (not a merge):_ this candidate's own statement names
both the `grilling` and `domain-modeling` candidates as what it
invokes; no record's `workflow` field points at
`invoke-grill-with-docs`, so it stays its own candidate rather than
being folded into either.

No member stage candidates (none exist for this workflow name).

### `register-project`

**Single-behavior workflow candidate**, established entirely by `BU-0131`
(`docs/getting-started.md (docs/getting-started.md L102-110)`) — no `stage`/`stage-context`/`helper` record in this
corpus carries a `workflow` field matching `register-project` (checked against
that source file's other extracted units and against every `workflow`
field value used corpus-wide). Reported as a single-behavior candidate
per synthesis-method.md's "what must not happen" rather than padded out
or merged into an adjacent-topic cluster it does not actually name.

- **Trigger:** a new project YAML is being registered
- **Outcome:** the project file satisfies all six named field-shape requirements
- **Statement:** Registering a project requires the project file's `name` field to match its filename, every repository to have a unique name and correct path, clone URLs present for repositories the sync step may clone, roles/groups identifying ownership, agent instructions containing commands and observable constraints rather than vague quality slogans, and `graphify.output` (when used) to be one project-level path outside source repos.
- **Completion condition:** the full six/five-part procedure named in the
  statement above has been executed, per `BU-0131`.

No member stage candidates (none exist for this workflow name).

## Unattached records (synthesis-time defects surfaced here, not silently resolved)

Per synthesis-method.md buckets 1 and 3: a `helper`/`stage`/`stage-context`
record with no `workflow` value, or naming a `workflow`+`stage` pair with no
matching `stage` candidate, is a `40-classify`-stage defect surfacing here —
recorded plainly, not invented a home for.

### Missing `workflow` field entirely

- `BU-0878` (`helper`, `workflow=null`, `stage=null`) —
  `bin/_sgt-lib.sh (bin/_sgt-lib.sh L12-13)`: Sourcing the shared library helper file more than once in the same shell process is a no-op after the first source.
- `BU-0890` (`helper`, `workflow=null`, `stage=null`) —
  `bin/_sgt-lib.sh (bin/_sgt-lib.sh L419-420 (_die / _info))`: An unrecoverable error in any sgt-* script prints an ERROR-prefixed message to stderr and terminates the process with a non-zero exit code.
- `BU-0892` (`helper`, `workflow=null`, `stage=null`) —
  `bin/_sgt-lib.sh (bin/_sgt-lib.sh L438-449 (_sgt_wiki_write))`: Wiki capture only runs at all if the configured wiki-writer script exists and is executable and the operator has not set SGT_WIKI_DISABLED=1; otherwise the capture call is a silent no-op.
- `BU-0893` (`helper`, `workflow=null`, `stage=null`) —
  `bin/_sgt-lib.sh (bin/_sgt-lib.sh L452-460 (_require_yq / _require_tmux / _require_git))`: Sergeant dies with an actionable installation hint (e.g. 'brew install yq') when a required external tool (yq, tmux, or git) is not found on PATH.

### `workflow`+`stage` names a checkpoint with no matching `stage` candidate

- **`standard-workflow` / `monitor-progress`** — no `stage`-rung record in this corpus
  classifies this checkpoint; the following record(s) name it anyway:
  - `BU-0032` (`stage-context`) — `AGENTS.md (AGENTS.md L144)`: Step 7 of the standard workflow: monitoring real progress requires recent meaningful events or an active child operation plus exact pane/process identity; parent-process liveness alone is insufficient.
  - `BU-0033` (`stage-context`) — `AGENTS.md (AGENTS.md L144)`: In OpenCode, run the interactive fleet-watch loop and verify the monitor started (unit identity printed); if managed background execution is unavailable, use bounded one-shot status checks rather than a blocking watch call.
  - `BU-0144` (`stage-context`) — `docs/using-sergeant.md (docs/using-sergeant.md L161-166)`: `in_progress` is never equated with health; the interactive fleet-watch loop requires exact live worker-pane identity plus recent meaningful progress evidence, preferring tmux `pane_activity`, falling back to the worker's recorded `progress_ts`, and using `.sergeant-status` mtime only when no better timestamp exists.
  - `BU-0145` (`stage-context`) — `docs/using-sergeant.md (docs/using-sergeant.md L165-168)`: When progress evidence stays older than the default 300-second grace window, the interactive fleet-watch loop keeps the repo `in_progress` and records a nonterminal `live worker stalled` diagnostic instead of declaring it done, failed, or orphaned.
  - `BU-0174` (`stage-context`) — `docs/troubleshooting.md (docs/troubleshooting.md L61-68)`: A live parent process is insufficient evidence of progress; `in_progress` plus a `live worker stalled` diagnostic is still nonterminal and must be reconciled through the progress rules before killing or relaunching anything, preserving worktree/branch/task/response-generation/handoff first, and using the stalled-worker recovery step only for that exact stall classification.
  - `BU-0274` (`stage-context`) — `skills/dispatch/SKILL.md (skills/dispatch/SKILL.md L92)`: `needs_input` and `blocked` are distinct nonterminal states; a worker waiting on CI, review threads, or dependencies remains `in_progress` unless it needs to escalate.
  - `BU-0568` (`helper`) — `bin/sgt-watch (bin/sgt-watch L14-23)`: The interactive fleet-watch loop accepts the --background flag either before or after the task-id argument when entering background-watch mode, so both call-site conventions resolve to the same task.
  - `BU-0569` (`stage-context`) — `bin/sgt-watch (bin/sgt-watch L42-44)`: The --snapshot observation path is strictly read-only: it runs before any reconciliation, never writes fleet state, and deliberately avoids the pane-identity migration side effect that ordinary reconciliation performs.
  - `BU-0572` (`helper`) — `bin/sgt-watch (bin/sgt-watch L136-146)`: --snapshot validates every caller-supplied task-id and repo scope value against a fixed identifier pattern before observing any fleet state, so a malformed scope value can never reach the emitted document and cannot make the document unbounded.
  - `BU-0573` (`stage-context`) — `bin/sgt-watch (bin/sgt-watch L63-77)`: A --snapshot repo or task observation only counts a pane as a verified worker when the live tmux pane identity exactly matches the identity recorded in that repo's pane_identity file; an ambient pane occupying the recorded pane id is not sufficient.
  - `BU-0574` (`stage-context`) — `bin/sgt-watch (bin/sgt-watch L84-86)`: Only a repo whose recorded status is exactly in_progress can ever count as an active witness in a --snapshot observation; terminal, waiting, and unreconciled statuses never do.
  - `BU-0575` (`stage-context`) — `bin/sgt-watch (bin/sgt-watch L90-100)`: A --snapshot active-witness determination treats the more recent of the recorded progress_ts and the live tmux pane-activity timestamp as the last-event time, and only counts the witness as active if that time falls within a configurable recent-seconds window (default 300s).
  - `BU-0577` (`stage-context`) — `bin/sgt-watch (bin/sgt-watch L239-242)`: Once a repo has a persisted pane_identity file with content, the interactive fleet-watch loop verifies pane ownership against that exact recorded identity rather than recomputing eligibility criteria from scratch.
  - `BU-0578` (`stage-context`) — `bin/sgt-watch (bin/sgt-watch L256-263)`: The first time the interactive fleet-watch loop verifies a pane against inferred (pre-migration) worker criteria, it persists the verified identity to pane_identity, and a parallel pane_identity_migration record, so future checks use the exact recorded identity instead of re-deriving it every time.
  - `BU-0601` (`helper`) — `bin/sgt-watch (bin/sgt-watch L694-715)`: The interactive fleet-watch loop only reprints task status when the aggregated per-repo snapshot (status, stage, validation_status, message, diagnostic, plus the task's notify marker) actually differs from the last printed snapshot, rather than redrawing on every poll tick.
  - `BU-0602` (`stage-context`) — `bin/sgt-watch (bin/sgt-watch L716-735)`: The interactive fleet-watch loop exits 0 once every repo in the task reaches a terminal, non-failed status, and exits 1 as soon as any repo is failed or orphaned, even while others are still nonterminal; otherwise it keeps polling at POLL_INTERVAL (default 5s, SERGEANT_WATCH_INTERVAL).

## Bucket 4: Permanent-instruction candidates (`agents-invariant`)

**126** records. Listed, not drafted into any workflow package — per
synthesis-method.md bucket 4 and the ladder's own framing, `AGENTS.md`
changes are the promotion reviewer's call, not this run's.

- `BU-0001` (`AGENTS.md (AGENTS.md L3-5)`): Before acting on a project, resolve its repositories, roles, inherited instructions, and configured paths with the project context-resolution step.
- `BU-0002` (`AGENTS.md (AGENTS.md L3-5)`): Ownership of a project is never inferred from the current working directory.
- `BU-0003` (`AGENTS.md (AGENTS.md L11-13)`): The primary Sergeant session coordinates multi-repository work by default rather than implementing directly.
- `BU-0004` (`AGENTS.md (AGENTS.md L11-13)`): Direct implementation in the primary session is permitted only when the user explicitly asks to work in-session (or says not to dispatch) and one repository owns the complete outcome.
- `BU-0005` (`AGENTS.md (AGENTS.md L15-20)`): Dispatch mode is used when work spans repositories, contains two or more independent repository-owned tasks, needs an isolated independent review worker, or the user asks for workers.
- `BU-0009` (`AGENTS.md (AGENTS.md L22-36)`): Direct mode is used only when the user explicitly requests it and the work has one clear owning repository.
- `BU-0017` (`AGENTS.md (AGENTS.md L38-41)`): The coordinator role is never used as a reason to stop at a plan, status report, or dispatch suggestion when the user asked for an implemented outcome.
- `BU-0018` (`AGENTS.md (AGENTS.md L38-41)`): Direct mode is never used to edit several repositories in one checkout, or to bypass repository instructions, task ownership, review independence, or shipping gates.
- `BU-0019` (`AGENTS.md (AGENTS.md L60-61)`): When a toolbelt command covers an operation, it is used instead of reproducing the operation with ad hoc shell commands.
- `BU-0020` (`AGENTS.md (AGENTS.md L99-103)`): The bare `sgt-*` command name is used when it resolves on PATH; otherwise the matching script is run from this repository's `bin/` directory.
- `BU-0021` (`AGENTS.md (AGENTS.md L99-103)`): Manual fallback operations are used only when no toolbelt command covers the operation, or the command returns an explicit unsupported-case error; the fallback and the original error evidence are reported.
- `BU-0022` (`AGENTS.md (AGENTS.md L120-124)`): For every listed procedural-skill trigger, the repository-local SKILL.md file is read directly; it is canonical and takes precedence over any same-named registry skill.
- `BU-0023` (`AGENTS.md (AGENTS.md L125-126)`): A harness registry's omission of a skill does not make the skill unavailable, and the owner is not asked or the task stopped solely because the registry omits it.
- `BU-0024` (`AGENTS.md (AGENTS.md L127-128)`): The session stops and reports the exact repository-local skill path only when that file is absent or unreadable, and does not reconstruct a partial protocol from memory in that case.
- `BU-0036` (`AGENTS.md (AGENTS.md L148)`): `in_progress`, `needs_input`, `blocked`, and `waiting` are treated as nonterminal worker states; a waiting worker may remain alive or may exit after a durable handoff.
- `BU-0037` (`AGENTS.md (AGENTS.md L148)`): Deferred waits are resumed through the wake-condition step when a durable `.sergeant-wake-condition` has been published; human decisions are resumed through the worker response-delivery step.
- `BU-0038` (`AGENTS.md (AGENTS.md L148)`): Progress is never inferred from liveness alone, an expected blocked exit is never rewritten as orphaned, and a waiting worktree is never cleaned.
- `BU-0039` (`AGENTS.md (AGENTS.md L148)`): The worker response-delivery step, the wake-condition step, or supported recovery are used only after reconciling status, response generation, pane identity, and handoff evidence.
- `BU-0040` (`AGENTS.md (AGENTS.md L150-153)`): Every dispatched implementation, independent review, PR description, successor, recovery, and final shipping gate must use the same canonical intent revision from `.sergeant-intent.md`.
- `BU-0041` (`AGENTS.md (AGENTS.md L150-153)`): Workers and remediation loops never run the validation pipeline themselves.
- `BU-0044` (`AGENTS.md (AGENTS.md L161-162)`): A plan, task, finding, or worker launch is not treated as the requested outcome unless the user asked only for planning or dispatch.
- `BU-0045` (`AGENTS.md (AGENTS.md L163-164)`): A known blocker is not repeatedly reported once its decision and remediation path are approved; the next safe step is executed instead.
- `BU-0046` (`AGENTS.md (AGENTS.md L165-166)`): Duplicate tasks, findings, PRs, workers, or review passes are not created when a canonical preserved owner already exists.
- `BU-0047` (`AGENTS.md (AGENTS.md L167-168)`): A worker is not called active solely because its process or pane exists; recent meaningful progress evidence is required.
- `BU-0048` (`AGENTS.md (AGENTS.md L169-170)`): A completed, merged, blocked, or abandoned task is never left recorded as `in_progress`; the task tracker and fleet state are reconciled truthfully.
- `BU-0049` (`AGENTS.md (AGENTS.md L171-172)`): Tool absence produces an actionable fallback or explicit blocker, never a silent skip, false success, or indefinite wait.
- `BU-0050` (`AGENTS.md (AGENTS.md L173-175)`): Standing authorization may remove repetitive dispatch confirmation, but never authorizes risk acceptance, gate skipping, force operations, secret exposure, or destruction of preserved state.
- `BU-0054` (`AGENTS.md (AGENTS.md L182)`): Repositories under `~/.config/sergeant/` are never modified — that location is config, not code.
- `BU-0055` (`AGENTS.md (AGENTS.md L183)`): Secrets are never committed; project YAMLs may contain paths but must not contain credentials.
- `BU-0056` (`AGENTS.md (AGENTS.md L184-185)`): A bare `sgt-*` command is used when `command -v <name>` succeeds; otherwise the equivalent `bin/<name>` from this repository is run.
- `BU-0061` (`README.md (README.md L145-149)`): The interactive fleet-watch loop and `--sync-all` reconcile lifecycle state and may kill panes, so neither is safe for a coordinator or bridge that only wants to observe.
- `BU-0065` (`README.md (README.md L172-178)`): The dispatch step/`SERGEANT_AGENT` selects the harness executable and the dispatch step/`SERGEANT_MODEL` pins what that harness runs, and the two are orthogonal, with model precedence `--model` > `SERGEANT_MODEL` > the harness's own ambient default.
- `BU-0078` (`README.md (README.md L257-260)`): The managed coordinator pane is reused across dispatches and runs a reader that displays each line it receives and never executes it, so a tmux-injected notification can never become a shell command in the coordinator's pane.
- `BU-0106` (`docs/README.md (docs/README.md L28-36)`): Documentation authority is layered by ownership: `AGENTS.md` owns always-on agent execution/safety policy, `skills/*/SKILL.md` and `.agents/skills/*/SKILL.md` own trigger-specific procedures, `docs/schema.md` owns project configuration fields and path resolution, and the rest of this documentation set owns user installation/operating instructions.
- `BU-0107` (`docs/README.md (docs/README.md L34-36)`): Command `--help` output wins when the command implements it; otherwise the command's emitted usage/error contract and its tests win, and a task is filed when prose disagrees with released behavior.
- `BU-0108` (`docs/README.md (docs/README.md L38-39)`): Documentation examples must not contain real credentials, private repository names, prompt bodies, response bodies, or secret-bearing environment values.
- `BU-0109` (`docs/what-is-sergeant.md (docs/what-is-sergeant.md L10-12)`): Sergeant is designed for one developer per installation; adoption by a larger organization means each developer installs Sergeant independently — it does not turn one installation into a shared team service.
- `BU-0110` (`docs/what-is-sergeant.md (docs/what-is-sergeant.md L23-24)`): Sergeant does not provide central tenancy, organization RBAC, shared credentials, cross-machine worker leases, or a team-wide fleet database.
- `BU-0111` (`docs/what-is-sergeant.md (docs/what-is-sergeant.md L51-52)`): A worker is an agent running in an isolated worktree and tmux pane; a live process is not proof of progress, and recent meaningful progress evidence is required.
- `BU-0112` (`docs/what-is-sergeant.md (docs/what-is-sergeant.md L54-58)`): A decision request is a `needs_input`, `blocked`, or validation ask-user gate that requires a human product, security, privacy, destructive-action, or risk decision; mechanical findings are not human decision requests.
- `BU-0113` (`docs/what-is-sergeant.md (docs/what-is-sergeant.md L64-66)`): Direct mode still requires a task, TDD, repository-native checks, independent review, shipping validation, and handoff even though it runs in the current session.
- `BU-0114` (`docs/what-is-sergeant.md (docs/what-is-sergeant.md L78)`): Sergeant is not permission to push directly to default branches.
- `BU-0115` (`docs/what-is-sergeant.md (docs/what-is-sergeant.md L79-82)`): Sergeant does not make a worker healthy merely because its process exists, does not treat a plan, task, worker launch, or finding as delivered work, and does not authorize validation agents to modify source while reporting findings.
- `BU-0116` (`docs/skills.md (docs/skills.md L19-20)`): Skill provenance is never inferred from a folder name; `.skill-lock.json`, a package lock, plugin metadata, or the source repository is checked instead.
- `BU-0117` (`docs/skills.md (docs/skills.md L55-57)`): The Claude plugin route installs a managed read-only bundle; plugin-owned files are never edited, since updates are not expected to preserve those edits.
- `BU-0118` (`docs/skills.md (docs/skills.md L119-122)`): Every directive in a Sergeant-owned skill must contain a trigger, action, prohibition, observable evidence, or stop condition; slogans such as 'be thorough' are replaced with commands, failure behavior, acceptance criteria, ownership, or review evidence.
- `BU-0120` (`docs/skills.md (docs/skills.md L142-144)`): Sergeant-owned skills are updated through this repository via a reviewed PR and by running `bash tests/instruction-policy-test.sh` plus the full Sergeant test suite.
- `BU-0121` (`docs/repo-scoped-skills.md (docs/repo-scoped-skills.md L4-10)`): `.agents/skills/` is the canonical Agent Skills tree discovered directly by Codex; OpenCode discovers the same tree through `opencode.json`; Claude discovers it through repository-local links in `.claude/skills/`, which resolve only to `.agents/skills/` — no install step writes to a user's global agent configuration.
- `BU-0122` (`docs/repo-scoped-skills.md (docs/repo-scoped-skills.md L38-40)`): Workers are instructed never to invoke the validation pipeline directly; the validation pipeline skill is vendored only so workers can load and understand the coordinator-owned shipping gate contract when a brief references it.
- `BU-0130` (`docs/getting-started.md (docs/getting-started.md L82-83)`): Sergeant does not install harness-specific conversation-injection plugins; worker updates are surfaced from durable fleet state through the interactive fleet-watch loop.
- `BU-0172` (`docs/troubleshooting.md (docs/troubleshooting.md L3-4)`): Supported Sergeant commands are used before manual process, tmux, Git, or fleet-file operations, and exact errors and state are preserved before recovery.
- `BU-0183` (`docs/troubleshooting.md (docs/troubleshooting.md L144-146)`): Parsing proof of Bash 3.2 compatibility does not replace runtime proof unless the task acceptance explicitly permits parsing only.
- `BU-0194` (`docs/schema.md (docs/schema.md L21-24)`): Durable callback implementations are executable profiles under `~/.config/sergeant/callbacks/`; they are not project YAML fields, and fleet requests cannot supply paths.
- `BU-0259` (`skills/load-project/SKILL.md (skills/load-project/SKILL.md L39)`): Project YAML files never contain credentials, tokens, or secret values.
- `BU-0265` (`skills/load-project/SKILL.md (skills/load-project/SKILL.md L74)`): When a required executable is missing, the skill reports the executable and a platform-neutral installation requirement rather than inventing a fallback parser.
- `BU-0266` (`skills/load-project/SKILL.md (skills/load-project/SKILL.md L75)`): If the project context-resolution step output and the raw YAML disagree, the project context-resolution step failure is treated as blocking and the YAML is preserved for diagnosis.
- `BU-0817` (`skills/wiki/SKILL.md (skills/wiki/SKILL.md L19-21)`): Curated wiki pages never contain raw prompts, response bodies, credentials, tokens, or secrets copied from source material.
- `BU-0818` (`skills/wiki/SKILL.md (skills/wiki/SKILL.md L19-21)`): Task, repository, PR, merge, decision, and blocker facts are preserved into curated pages only when the wiki schema permits them.
- `BU-0819` (`skills/wiki/SKILL.md (skills/wiki/SKILL.md L27-31)`): Automatic wiki captures are owned exclusively by three commands, each for its own event: the dispatch step captures fleet launch/task/project/branch/repository/brief metadata, the notify step captures escalation or terminal outcome plus any PR URL, and the fleet cleanup step captures worktree/fleet cleanup and final status.
- `BU-0820` (`skills/wiki/SKILL.md (skills/wiki/SKILL.md L33-34)`): A missing automatic capture is fixed by reproducing the owning command in a fixture or repairing its capture adapter; it is never fixed by manually synthesizing a capture as a substitute.
- `BU-0877` (`bin/_sgt-bash-version.sh (bin/_sgt-bash-version.sh L4-19 (_sgt_bash_version_supported / _sgt_require_bash_version))`): Sergeant's Bash entry points refuse to continue when the running Bash interpreter is older than 3.2, printing an error to stderr and returning failure instead of proceeding under an unsupported interpreter.
- `BU-0891` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L426)`): A failure while writing a wiki capture document never fails or blocks the Sergeant operation it was documenting; wiki-write failures are silently swallowed.
- `BU-0898` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L501-506 (SGT_MANAGED_COORDINATOR_COMMAND))`): The managed coordinator pane runs a reader loop that only echoes every line it receives back out and never executes it, so a tmux-injected notification can never become a shell command running in the coordinator's own pane.
- `BU-0942` (`.agents/skills/diagnosing-bugs/SKILL.md (.agents/skills/diagnosing-bugs/SKILL.md L8-8)`): A phase of the diagnosing-bugs discipline may be skipped only when there is an explicit justification for skipping it.
- `BU-1001` (`.agents/skills/wayfinder/SKILL.md (.agents/skills/wayfinder/SKILL.md L17-17)`): In human-facing narration and the map's Decisions-so-far, a map or ticket is referred to by its name (title), never by a bare id, number, or slug — the id and URL still exist but ride inside the name link rather than standing in for it.
- `BU-1002` (`.agents/skills/wayfinder/SKILL.md (.agents/skills/wayfinder/SKILL.md L21-21)`): The map is a single issue on the repo's issue tracker labelled `wayfinder:map`, and its tickets are child issues of that map.
- `BU-1003` (`.agents/skills/wayfinder/SKILL.md (.agents/skills/wayfinder/SKILL.md L23-23)`): The map itself only gists a decision and links to it; the map is an index, not a store, so the decision's actual detail lives in exactly one place — its ticket.
- `BU-1005` (`.agents/skills/wayfinder/SKILL.md (.agents/skills/wayfinder/SKILL.md L29-29)`): Open tickets are not listed inline in the map body — they are found by querying open child issues instead, keeping the loaded map view low-resolution.
- `BU-1006` (`.agents/skills/wayfinder/SKILL.md (.agents/skills/wayfinder/SKILL.md L65-65)`): Every ticket carries exactly one `wayfinder:<type>` label from the set research, prototype, grilling, task.
- `BU-1008` (`.agents/skills/wayfinder/SKILL.md (.agents/skills/wayfinder/SKILL.md L69-69)`): Ticket dependencies use the tracker's native dependency relationship (so the frontier renders visually in the tracker's own UI) unless the tracker lacks native blocking, in which case a body convention is the fallback; a ticket is unblocked once every ticket blocking it is closed, and the frontier is the set of open, unblocked, unclaimed children.
- `BU-1010` (`.agents/skills/wayfinder/SKILL.md (.agents/skills/wayfinder/SKILL.md L75-75)`): Every ticket is either HITL — resolvable only through a live exchange with the human, who the agent never stands in for — or AFK, driven by the agent alone; an agent answering its own HITL questions has broken this.
- `BU-1015` (`.agents/skills/wayfinder/SKILL.md (.agents/skills/wayfinder/SKILL.md L84-84)`): The map does not chart what can't yet be seen (the fog of war); resolving a ticket clears the fog ahead of it, graduating whatever becomes specifiable into fresh tickets one at a time.
- `BU-1033` (`.agents/skills/codebase-design/SKILL.md (.agents/skills/codebase-design/SKILL.md L12)`): When discussing or designing module boundaries, use the codebase-design glossary terms (module, interface, implementation, depth, seam, adapter, leverage, locality) exactly, rather than substituting generic terms like component, service, API, or boundary.
- `BU-1034` (`.agents/skills/codebase-design/SKILL.md (.agents/skills/codebase-design/SKILL.md L54-58)`): When designing an interface, ask whether the number of methods can be reduced, whether the parameters can be simplified, and whether more complexity can be hidden inside.
- `BU-1035` (`.agents/skills/codebase-design/SKILL.md (.agents/skills/codebase-design/SKILL.md L63)`): To judge whether a module earns its keep, apply the deletion test: imagine deleting the module — if the complexity it held simply vanishes, it was a pass-through; if that complexity reappears spread across its N callers, the module was earning its keep.
- `BU-1036` (`.agents/skills/codebase-design/SKILL.md (.agents/skills/codebase-design/SKILL.md L65)`): Do not introduce a seam unless something actually varies across it: one adapter means the seam is only hypothetical, two adapters means it is real.
- `BU-1037` (`.agents/skills/codebase-design/SKILL.md (.agents/skills/codebase-design/SKILL.md L71)`): For testability, a module should accept its dependencies as parameters rather than constructing them internally.
- `BU-1038` (`.agents/skills/codebase-design/SKILL.md (.agents/skills/codebase-design/SKILL.md L83)`): For testability, a module should return results rather than produce side effects.
- `BU-1039` (`.agents/skills/codebase-design/SKILL.md (.agents/skills/codebase-design/SKILL.md L95)`): Interfaces should be kept to a small surface area, because fewer methods require fewer tests and fewer parameters require simpler test setup.
- `BU-1040` (`.agents/skills/codebase-design/DEEPENING.md (.agents/skills/codebase-design/DEEPENING.md L9-11)`): A candidate module whose dependencies are in-process (pure computation, in-memory state, no I/O) is always deepenable: merge the modules and test through the new interface directly, with no adapter needed.
- `BU-1041` (`.agents/skills/codebase-design/DEEPENING.md (.agents/skills/codebase-design/DEEPENING.md L13-15)`): A candidate module whose dependency has a local test stand-in (e.g. PGLite for Postgres, an in-memory filesystem) is deepenable if that stand-in exists: the deepened module is tested with the stand-in in the test suite, and the seam stays internal with no port at the module's external interface.
- `BU-1042` (`.agents/skills/codebase-design/DEEPENING.md (.agents/skills/codebase-design/DEEPENING.md L17-19)`): A candidate module whose dependency is a remote but owned service (e.g. an internal microservice) is deepened by defining a port at the seam so the deep module owns the logic while the transport is injected as an adapter; tests use an in-memory adapter and production uses an HTTP/gRPC/queue adapter.
- `BU-1043` (`.agents/skills/codebase-design/DEEPENING.md (.agents/skills/codebase-design/DEEPENING.md L23-25)`): A candidate module whose dependency is a true external, third-party service (e.g. Stripe, Twilio) is deepened by taking the dependency as an injected port, with tests supplying a mock adapter.
- `BU-1044` (`.agents/skills/codebase-design/DEEPENING.md (.agents/skills/codebase-design/DEEPENING.md L29)`): Do not introduce a port at a seam unless at least two adapters are justified (typically production and test); a single-adapter seam is just indirection.
- `BU-1045` (`.agents/skills/codebase-design/DEEPENING.md (.agents/skills/codebase-design/DEEPENING.md L30)`): Internal seams — private to a module's own implementation and used only by its own tests — should not be exposed through the module's external interface just because tests happen to use them.
- `BU-1046` (`.agents/skills/codebase-design/DEEPENING.md (.agents/skills/codebase-design/DEEPENING.md L34)`): Once tests exist at a deepened module's interface, the old unit tests on the shallow modules it replaced become waste and should be deleted.
- `BU-1047` (`.agents/skills/codebase-design/DEEPENING.md (.agents/skills/codebase-design/DEEPENING.md L35-36)`): New tests written for a deepened module are written at its interface and assert on observable outcomes through that interface, not on internal state.
- `BU-1048` (`.agents/skills/codebase-design/DEEPENING.md (.agents/skills/codebase-design/DEEPENING.md L37)`): A test that must change when a module's implementation changes without any corresponding interface change is a signal that the test is testing past the interface rather than describing behavior.
- `BU-1064` (`.agents/skills/domain-modeling/SKILL.md (.agents/skills/domain-modeling/SKILL.md L64)`): CONTEXT.md is restricted to glossary content: it must not be treated as a spec, a scratch pad, or a repository for implementation decisions, and must stay devoid of implementation details.
- `BU-1080` (`.agents/skills/prototype/SKILL.md (.agents/skills/prototype/SKILL.md L21)`): Prototype code is located close to where it will actually be used, but named so a casual reader can tell it is a prototype, not production, and throwaway UI routes follow the project's existing routing convention rather than inventing a new top-level structure.
- `BU-1081` (`.agents/skills/prototype/SKILL.md (.agents/skills/prototype/SKILL.md L22)`): A prototype must be runnable with one command, using whatever task runner the project already supports, so the user can start it without having to think about how.
- `BU-1082` (`.agents/skills/prototype/SKILL.md (.agents/skills/prototype/SKILL.md L23)`): A prototype has no persistence by default (state lives in memory); if the question explicitly involves a database, the prototype hits a scratch DB or local file with a clear "PROTOTYPE — wipe me" name rather than a real data store.
- `BU-1083` (`.agents/skills/prototype/SKILL.md (.agents/skills/prototype/SKILL.md L24)`): A prototype skips polish: no tests, no error handling beyond what's needed to make it runnable, and no abstractions, because the point is to learn something fast.
- `BU-1084` (`.agents/skills/prototype/SKILL.md (.agents/skills/prototype/SKILL.md L25)`): A prototype surfaces its full relevant state after every action (logic branch) or on every variant switch (UI branch), so the user can see what changed.
- `BU-1126` (`.agents/skills/tdd/SKILL.md (.agents/skills/tdd/SKILL.md L8)`): This skill's sections on what a good test is, where tests go, the anti-patterns, and the rules of the loop are consulted before and during every TDD cycle, not only afterward.
- `BU-1128` (`.agents/skills/tdd/SKILL.md (.agents/skills/tdd/SKILL.md L14)`): A good test verifies behavior through public interfaces rather than implementation details, reads like a specification of a capability, and survives refactors because it does not depend on internal structure.
- `BU-1129` (`.agents/skills/tdd/SKILL.md (.agents/skills/tdd/SKILL.md L20)`): A test lives at a seam — the public boundary where behavior is observed without reaching inside — and never against internals.
- `BU-1131` (`.agents/skills/tdd/SKILL.md (.agents/skills/tdd/SKILL.md L28)`): An implementation-coupled test — one that mocks internal collaborators, tests private methods, or verifies through a side channel like querying the database instead of the interface — is an anti-pattern, tellingly breaking on refactors that don't change behavior.
- `BU-1132` (`.agents/skills/tdd/SKILL.md (.agents/skills/tdd/SKILL.md L29)`): A tautological test — one whose expected value is recomputed the same way the code computes it, so it passes by construction — is an anti-pattern; expected values must instead come from an independent source of truth such as a known-good literal, a worked example, or the spec.
- `BU-1137` (`.agents/skills/tdd/mocking.md (.agents/skills/tdd/mocking.md L3-8)`): Mocking is used only at system boundaries: external APIs, databases (sometimes, a test DB is preferred), time/randomness, and the filesystem (sometimes).
- `BU-1138` (`.agents/skills/tdd/mocking.md (.agents/skills/tdd/mocking.md L10-14)`): Your own classes/modules, internal collaborators, and anything you control are never mocked.
- `BU-1139` (`.agents/skills/tdd/mocking.md (.agents/skills/tdd/mocking.md L20-22)`): For mockability, external dependencies at system boundaries are passed into a function/module (dependency injection) rather than being constructed internally by it.
- `BU-1140` (`.agents/skills/tdd/mocking.md (.agents/skills/tdd/mocking.md L37-39)`): For mockability, SDK-style interfaces (a specific function per external operation) are preferred over one generic fetcher with conditional logic, because each mock then returns one specific shape, test setup needs no conditional logic, it's easier to see which endpoints a test exercises, and type safety is per-endpoint.
- `BU-1141` (`.agents/skills/tdd/tests.md (.agents/skills/tdd/tests.md L17-23)`): A good test exhibits five characteristics together: it tests behavior users/callers care about, uses only the public API, survives internal refactors, describes WHAT rather than HOW, and makes one logical assertion.
- `BU-1142` (`.agents/skills/tdd/tests.md (.agents/skills/tdd/tests.md L38-45)`): A bad, implementation-detail test is recognized by any of six red flags: mocking internal collaborators, testing private methods, asserting on call counts/order, breaking on refactors without a behavior change, a name that describes HOW rather than WHAT, or verifying through external means instead of the interface (e.g. querying the database directly rather than using the createUser/getUser interface).
- `BU-1143` (`.agents/skills/triage/SKILL.md (SKILL.md L11)`): When the subject repository treats external pull requests as a request surface, triage handles a PR through the same category/state roles and the same state machine as an issue, with only a small set of PR-specific deltas.
- `BU-1145` (`.agents/skills/triage/SKILL.md (SKILL.md L13-17)`): Every comment or issue the triage skill posts to the issue tracker must begin with a disclaimer stating it was generated by AI during triage.
- `BU-1146` (`.agents/skills/triage/SKILL.md (SKILL.md L41)`): Every triaged issue carries exactly one category role and exactly one state role.
- `BU-1196` (`.agents/skills/no-mistakes/SKILL.md (SKILL.md L8-13)`): Workers and remediation loops must never invoke the validation pipeline; the Sergeant coordinator alone owns every validation pipeline gate.
- `BU-1260` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L9-10)`): The setup skill orchestrates only supported Sergeant and bootstrap commands and must not substitute undocumented workarounds for a capability that is missing.
- `BU-1261` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L9-10)`): Missing capabilities encountered during setup are surfaced as separate task tracker issues rather than worked around.
- `BU-1262` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L20-21)`): This skill must not be loaded when the user wants documentation only or is asking about a specific command; `sergeant-help` is used instead in both cases.
- `BU-1263` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L25-29)`): This skill writes only to Sergeant-owned paths: `~/.config/sergeant/config.yaml` (global config) and `~/.config/sergeant/<project>.yaml` (project YAML files).
- `BU-1264` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L30-37)`): This skill must never write to opencode's config, Claude's config or `CLAUDE.md` or any `.claude/` directory, Codex's config, Goose's config, any repository's `AGENTS.md`/`.github/`/other agent configuration paths, or any path outside `~/.config/sergeant/` the user has not explicitly named.
- `BU-1265` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L39-41)`): The skill does not automatically initialize the task tracker, Graphify, or Treehouse; each requires an explicit confirmation prompt before any command runs, and if consent is declined the skill leaves state unchanged and reports what was skipped.
- `BU-1295` (`.agents/skills/sergeant-setup/SKILL.md (SKILL.md L296-298)`): Re-running this skill after a successful setup must produce the same final state: each phase skips steps that already pass verification, and no phase destroys existing working configuration to reach the same end state.
- `BU-1297` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L15)`): Prefer vertical slices that produce independently verifiable behavior when drafting tickets.
- `BU-1298` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L16)`): Keep each ticket small enough for one fresh agent context.
- `BU-1299` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L17)`): Assign exactly one owning repository to each implementation ticket.
- `BU-1300` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L18-19)`): Represent cross-repository delivery with counterpart tickets and explicit merge order, not one ambiguous shared ticket.
- `BU-1301` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L20-21)`): Use expand-migrate-contract for mechanical changes that cannot remain green as a vertical slice.
- `BU-1302` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L22)`): Create epics for coherent programs of work, not as substitutes for executable tickets.
- `BU-1303` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L23)`): Never duplicate an existing task tracker task or GitHub issue.
- `BU-1304` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L24)`): Preserve stable finding IDs such as `RBAC-P1-004` or `DATA-P0-002` in ticket titles.
- `BU-1305` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L25-26)`): A ticket is not ready unless its acceptance criteria are observable and its blockers are accurate.
- `BU-1311` (`.agents/skills/to-tickets/SKILL.md (SKILL.md L46)`): Do not automatically add the task tracker instructions to repository guidance files.

## Bucket 5: Shared helper/context candidates

`shared-helper`: **23** records. `shared-context`: **0** records (none this run).

No `.sergeant/common/contexts/` or `.sergeant/common/scripts/` directory
exists in this worktree yet (checked directly) — every candidate below is a
new promotion candidate, not a name collision to reconcile against an
existing entry.

**Over-promotion tell check** (`../_config/icm-ladder.md` §6.6): grouping the
23 `shared-helper` records by contract below yields 9 groups; the only
groups whose membership is entirely drawn from one source file
(`bin/_sgt-lib.sh`) are "tmux pane-identity verification" (2 of that file's
8 records), "owned state-file read validation" (3 of 8), and half of
"owned state-file atomic write / publish" (1 of 8, paired with a
`bin/sgt-watch`-adjacent record) — none is *all* of that file's records, so
the tell (a bucket-5 group == one whole source file's own unit set) is not
triggered for this corpus.

### dev_root-relative repo path resolution

Contract: given a repo `path` value from a project YAML (or a CLI path argument), resolve it to an absolute path — used verbatim if already absolute or home-relative (`~...`), otherwise resolved relative to `dev_root`. Same contract stated three times from three source files (`AGENTS.md`, `docs/schema.md`, `bin/_sgt-lib.sh`) — genuine cross-file behavior-shape clustering, not file mirroring.

Consuming candidates (workflows whose stage/stage-context/helper records
plausibly rely on this contract, by shared topic — not asserted as a
closed list):

- `BU-0051` (`AGENTS.md (AGENTS.md L179)`): `dev_root`, set in `~/.config/sergeant/config.yaml`, is the base against which repo paths in project YAMLs are resolved as relative paths.
- `BU-0193` (`docs/schema.md (docs/schema.md L19)`): Repo `path` values that are not absolute or home-relative are resolved relative to `dev_root`, so project YAMLs stay portable across machines by changing `dev_root` in one place.
- `BU-0889` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L401-410 (_resolve_path))`): A repository path argument that is already absolute or home-relative (~...) is used verbatim; any other form is resolved relative to the configured development root (DEV_ROOT).

### project name/filename identity rule

Contract: a project's `name` field (or identity) must equal its YAML filename without extension. Same contract from `AGENTS.md` and `docs/schema.md`.

Consuming candidates (workflows whose stage/stage-context/helper records
plausibly rely on this contract, by shared topic — not asserted as a
closed list):

- `BU-0052` (`AGENTS.md (AGENTS.md L180)`): A project's name is the YAML filename without its extension.
- `BU-0195` (`docs/schema.md (docs/schema.md L40)`): A project's `name` field must match its filename.

### agent-instruction layer concatenation order

Contract: instruction layers concatenate in a fixed order — defaults, then group, then repo — with later layers appearing later (later-wins on conflict). Same contract from `AGENTS.md` and `docs/schema.md`.

Consuming candidates (workflows whose stage/stage-context/helper records
plausibly rely on this contract, by shared topic — not asserted as a
closed list):

- `BU-0053` (`AGENTS.md (AGENTS.md L181)`): The project context-resolution step resolves instructions in order — `defaults.agent_instructions`, then group instructions, then repo instructions — with later layers overriding earlier ones for the same repo.
- `BU-0200` (`docs/schema.md (docs/schema.md L101-107)`): Agent instruction layers are concatenated in order (defaults, group, repo) with later layers appearing later in the block, and when directives conflict the later, more specific repository-level directive is the intended authority; Sergeant does not structurally merge or deduplicate the free-form prose.

### fleet-watch snapshot busy/basis contract

Contract: the `--snapshot` / interactive fleet-watch loop is read-only, constant-size, and reports `busy` from a closed two-value `basis` allowlist — `busy:true` only on a verified-active match, `busy:null` with `basis: no_verified_active_match` otherwise, any unrecognised condition mapping to the null basis rather than a guess. Same contract documented in `README.md` and implemented in `bin/sgt-watch` — 5 records, 2 source files, genuine behavior-shape clustering.

Consuming candidates (workflows whose stage/stage-context/helper records
plausibly rely on this contract, by shared topic — not asserted as a
closed list):

- `BU-0062` (`README.md (README.md L151-168)`): The interactive fleet-watch loop is strictly read-only and emits constant-size versioned JSON with `busy: true` only when all three hold: a stable `in_progress` status, an exact live Sergeant worker pane identity, and progress attributable to that pane within `SERGEANT_SNAPSHOT_RECENT_SECONDS` (default 300).
- `BU-0063` (`README.md (README.md L151-168)`): Every snapshot outcome other than the verified-busy case reports `busy: null` with `basis: no_verified_active_witness`; version 1 never emits `busy: false`, because absence of a verified witness is not proof of idleness.
- `BU-0064` (`README.md (README.md L151-168)`): `basis` is a closed allowlist of exactly two values, so an unrecognised condition maps to the null basis rather than inventing a new one.
- `BU-0570` (`bin/sgt-watch (bin/sgt-watch L46-49)`): A --snapshot observation reports busy:true only when a stable active status, an exact live worker identity match, and recently attributable progress all hold together; any other outcome reports busy:null, never busy:false, because absence of a verified witness is not proof of idleness.
- `BU-0571` (`bin/sgt-watch (bin/sgt-watch L51-52)`): The --snapshot basis field is restricted to a closed set of exactly two values; an unrecognized condition maps to the null basis rather than a newly invented one.

### worker-brief independent-review-axis contract

Contract: one shared axis/severity definition drives both the brief-rendering and dispatch-instruction halves of the independent-review requirement; an axis with no defined reviewer guidance fails brief-rendering rather than emitting an unreviewed brief. `README.md` + `bin/_sgt-review-axes.sh`.

Consuming candidates (workflows whose stage/stage-context/helper records
plausibly rely on this contract, by shared topic — not asserted as a
closed list):

- `BU-0095` (`README.md (README.md L314)`): Every worker brief requires one independent review per axis named in `SGT_REVIEW_AXES_REQUIRED`, and frontend/UI/visual/interaction/accessibility/user-facing-output language in the mission, repo role, or repo group additionally requires the conditional Accessibility axis; the one definition drives both what the brief demands and what `--axis` values the review-findings router accepts, so the two cannot drift apart.
- `BU-0300` (`bin/_sgt-review-axes.sh (bin/_sgt-review-axes.sh L4-9)`): One shared definition drives both halves of the independent-review contract — the axes/severities dispatch instructs a worker to produce, and the axes/severities the review-findings router accepts for routing — because previously writing them out separately let them drift (dispatch mandated a readiness review the router rejected outright, td-61a0c8).
- `BU-0301` (`bin/_sgt-review-axes.sh (bin/_sgt-review-axes.sh L41)`): An axis with no defined reviewer guidance fails the brief-rendering step rather than silently emitting an unexplained axis name.

### coordinator notify-marker wake contract

Contract: workers wake the coordinator by updating one shared per-task notify marker that the interactive fleet-watch loop polls. Single record (`docs/using-sergeant.md`) — no second record shares this exact contract, so it stays its own group rather than being force-merged into the pane-identity or notify-target-pointer groups below, which cover a related but distinct mechanism (see note).

Consuming candidates (workflows whose stage/stage-context/helper records
plausibly rely on this contract, by shared topic — not asserted as a
closed list):

- `BU-0143` (`docs/using-sergeant.md (docs/using-sergeant.md L157-159)`): Workers wake the coordinator by updating one shared per-task notify marker in fleet state; the interactive fleet-watch loop polls that marker, so simultaneous repo updates can at worst collapse into a delayed wakeup rather than duplicate delivery.

### tmux pane-identity verification

Contract: a tmux pane (or a previously recorded target pane) is only treated as the correct live destination if the live tmux server confirms the pane exists, is not dead, and its identity matches what was recorded — never inferred from pane position/index alone. Both records are from `bin/_sgt-lib.sh`; this group is 2 of that file's 8 `shared-helper` records, not all of them, so the over-promotion tell (a group == one whole source file's unit set) does not apply here.

Consuming candidates (workflows whose stage/stage-context/helper records
plausibly rely on this contract, by shared topic — not asserted as a
closed list):

- `BU-0896` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L484-495 (_sgt_verify_pane_identity))`): A tmux pane's identity is accepted only when the live tmux server confirms the pane exists, is not dead, and reports back the exact pane id that was asked for; an absent, dead, or mismatched pane fails the check rather than being adopted.
- `BU-0905` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L732-744 (_sgt_pane_identity_matches))`): A target pane is treated as still the correct destination only if its live identity matches a previously recorded expected identity (read via the strict or legacy-migration reader), and the live identity is re-checked once more immediately after that lookup before being trusted.

### owned state-file read validation

Contract: a state file is read as 'owned' only if it is a regular file (not a symlink), owned by the current user, and — for a hard-linked pair — both paths resolve to the same inode with matching content; a looser historical permission mode (640/644/660/664) is still accepted for backward compatibility. All 3 records are from `bin/_sgt-lib.sh` (3 of that file's 8 `shared-helper` records — again not the file's full set).

Consuming candidates (workflows whose stage/stage-context/helper records
plausibly rely on this contract, by shared topic — not asserted as a
closed list):

- `BU-0901` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L591-619 (_sgt_read_owned_file))`): A strict-mode owned state file is read only if it is a regular file (not a symlink), owned by the current user, and mode 600; its identity (inode:device) and mode are re-verified immediately after opening and again after the read completes, and the read is rejected if either check reveals the path was swapped or its mode/ownership changed underneath it.
- `BU-0902` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L620-677 (_sgt_read_matching_legacy_pane_identity))`): A state file stored under a looser historical permission mode (640/644/660/664) is still read for backward-compatible migration, but only when its value exactly matches an already-known expected value; once confirmed, it is atomically rewritten at the strict mode 600 and the migrated value is re-verified before being trusted.
- `BU-0903` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L678-724 (_sgt_read_same_owned_files))`): Reading a hard-linked pair of owned state files requires both paths to resolve to the same inode and to yield the same value, with identity (inode:device) and mode re-verified before and after both reads; any mismatch, mode drift, or broken hardlink relationship fails the read.

### owned state-file atomic write / publish

Contract: an owned/published state value is updated by writing to a private (mode 600) temporary file and then atomically renaming it into place — `BU-0904` states the general owned-state-file form, `BU-0906` applies the same temp-file+atomic-rename pattern specifically to publishing the current notification target via a nonce-bearing pointer file. Grouped as the same mechanism applied to two named artifacts, not force-merged into one contract — `BU-0906`'s consumer is the notify-target lookup the pane-identity group above feeds, `BU-0904`'s is any owned state file generally.

Consuming candidates (workflows whose stage/stage-context/helper records
plausibly rely on this contract, by shared topic — not asserted as a
closed list):

- `BU-0904` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L725-731 (_sgt_replace_owned_file))`): An owned state file is updated by writing the new value to a private (mode 600) temporary file and then atomically renaming it into place, so a concurrent reader never observes a partially-written value.
- `BU-0906` (`bin/_sgt-lib.sh (bin/_sgt-lib.sh L748-781 (_sgt_notification_target_create))`): The current notification target is published by atomically renaming a nonce-bearing pointer file into place, and the publish is then re-verified by re-reading that pointer; if a concurrent publisher's write landed after this rename, this attempt's own target-directory state is removed and the call reports failure rather than claiming to have set the target.

`shared-context`: 0 records this run — bucket reported empty, not omitted.

## Bucket 6: Obsolete-mechanism findings

**0** records this run. None — bucket reported empty per "what must not happen."

## Bucket 7: Engine-pressure candidates

**0** records this run. None — bucket reported empty per "what must not happen."

## Coverage accounting

- Total classification records: 1333
- Records with exactly one bucket appearance: 1333
- Records missing (0 appearances): 0
- Records double-counted (>1 appearances): 0

