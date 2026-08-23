# ADR 0021: The OpenCode adapter — run-json + adapter-owned serve,
the permission v1/v2 fork, and an honest no-sandbox posture

**Status:** Accepted, 2026-08-23. Implemented across four waves (W1
`opencode run --format json`, W2 registration + the PATH line, W3
`opencode serve` HTTP+SSE, W4 this record), merged to
`integration/opencode` (#245, #246), on `docs/proposals/
opencode-adapter-2026-08-23.md`.

## Context

Owner commission, same day (2026-08-23): build the OpenCode adapter,
Claude/Codex feature parity at minimum, exceed it where opencode
measurably does better. The commissioning plan is explicit that it is
not re-deriving anything ADR 0020 already settled: ADR 0020 itself
says its pattern is "the pattern any future adapter (opencode, goose,
…) should copy rather than re-derive," and this ADR takes that
instruction literally — every section below either cites an ADR 0020
ruling directly or states the one place opencode's measured behavior
actually diverges from codex's.

Three owner rulings frame this sprint and are cited by name rather
than restated:

- **K1** — dev-test pin is `opencode/big-pickle` (free Zen tier, zero
  cost); the adapter takes model as a parameter and hardcodes nothing.
- **K2** — scope is the adapter; no core changes. The owner offered a
  carve-out for cheap new dependencies if the serve transport needed
  one; **the carve-out went unused** (below).
- **K3** — version target `0.2.2`.

Carried without re-litigation from ADR 0020: **R1** (a measured
version floor is provenance, not a gate — now this repo's stance for
every adapter), **R2** (harness and backend are separate,
user-composable axes), **R3** (core stays; adapter-local evidence
types, not a contract-v2 seam), **R4** (parity is the floor, not the
ceiling; a measured better-than-claude/codex capability is used and
the record amended, `[[repo-is-a-snapshot]]`).

Every behavioral claim below is measured against opencode **1.18.19**
on Cerberus, 2026-08-23, in the probe evidence packet
(`sergeant-rs-workspace`:`knowledge/evidence/
opencode-adapter-probes-2026-08-23.md`, probes 1–11 plus the W3 serve
amendment) or the fixtures/live tests committed alongside
`src/backend/opencode.rs`/`opencode_serve.rs`, and is marked
**[measured]**, **[doc-claimed]**, or **[measured-negative]** exactly
as those modules' own doc comments mark it.

## Decision

### Harness and backend axes: nothing new decided here (R2)

`sgt opencode` has set `SGT_ORIGIN_CLIENT=opencode` since ADR 0006
(2026-08-14), and `router.rs`'s four-tier ladder
(`Explicit > OriginAffinity > WorkspaceDefault > GlobalDefault`) has
resolved a `"codex"`-named backend since before the codex adapter
existed. W2 registered `"opencode"` in `BackendRegistry`
(`daemon.rs`) the same way W2 of the codex sprint did — verification of
already-shipped machinery, not invention. `tests/opencode_routing.rs`
mirrors `tests/codex_routing.rs` line for line: explicit
`--backend opencode`, origin affinity from `sgt opencode`, and estate
`default_backend = "opencode"` all reach the real registered adapter.
No new precedence code, no new env var, nothing to decide.

### Version provenance is R1, unconditionally, from birth

`opencode.rs` never had a refusal branch to strike — it was written
against R1's posture from the start, exactly as ADR 0020 records
codex's `MEASURED_FLOOR` was. `MEASURED_FLOOR = (1, 18, 19)`
(`src/backend/opencode.rs:220`) is recorded, checked, and reported as
**provenance only**: a build below it is `available: true` with an
honest unmeasured-provenance detail, never blocked. What *is* refused
— the A2 split, carried verbatim — is a CLI whose version cannot be
parsed at all, or whose `run --help` does not offer this adapter's own
launch grammar (`--model`, `-s`, `--format json`); neither of those is
a version-policy question.

Two provenance facts worth naming because they differ from codex's
shape: `opencode --version` prints a **bare** `1.18.19\n` — no vendor
token to skip, unlike `codex-cli 0.149.0` — and `run --help`/`--help`
write to **stderr**, not stdout (yargs), so the probe reads both
streams rather than assuming stdout is where help text lives.

### The transport story: `opencode run --format json` and an
adapter-owned `opencode serve` HTTP+SSE child, both per execution; ACP
is an unmeasured third transport, not evaluated

**`opencode run --format json`** (W1): one process per turn, NDJSON on
stdout (`step_start` → (`tool_use`|`text`)* → `step_finish`), no
framing, no daemon — the fallback under every capability this ADR
names, exactly as `codex exec` is codex's. The session id is
**server-minted**, the opposite of claude's client-minted
`--session-id`: it appears on the first event line and nowhere before,
so `PreparedExecution::native_id` is honestly `None` at PREPARE and
LAUNCH waits, bounded (`SESSION_ID_BUDGET`), for that first line before
returning a handle at all — the same fail-closed discipline ADR 0020
applies to claude's ambiguous terminals, applied here to session
*birth* instead of session *end*.

**`opencode serve`** (W3): an adapter-owned child, one per
`OpencodeExecution`, on `127.0.0.1` with an ephemeral port and an
adapter-set `OPENCODE_SERVER_PASSWORD` — the same adapter-owned,
per-execution shape ADR 0020 chose for `codex app-server` over a
shared, host-global daemon, for the same blast-radius reasoning:
`runtime_scope()` stays `RuntimeScope::PerExecution`, so `mod.rs`'s
ENSURE-RUNTIME seam stays untouched (K2), and no shared listener
carries an auth-posture question across every Work on the host. `[R]`
was announced on **stdout** unconditionally
(`opencode server listening on http://127.0.0.1:<port>`)
[measured — W3 serve amendment], which is what the spawn probe reads
rather than guessing a fixed port. run-json remains the fallback at
every capability; a serve child that fails to spawn fails LAUNCH
honestly (§5.3, codex's precedent verbatim) rather than silently
downgrading a registration that already advertised serve-only
capabilities.

**The Agent Client Protocol (ACP) is not this ADR's third transport —
it is an unmeasured one.** opencode's own docs name an ACP surface
(Zed's protocol) alongside `run`/`serve`; nothing in this sprint probed
it, no crate for it exists in `Cargo.lock`, and no admission row claims
anything about it. It is named here only so a future reader does not
mistake silence for a considered refusal — this is the same posture
ADR 0020's A4 records for codex's app-server-daemon path: recorded as
a roadmap lead, reopened only by a wave whose own measured admission
needs it, at R5, with its own facts.

**The WebSocket carve-out: offered by the owner, measured unnecessary,
unused.** The owner's kickoff ruling explicitly offered a cheap-crate
carve-out if opencode's serve transport turned out to need one — the
same kind of gap ADR 0020's A4 flagged as unresolved for codex
(`[measured-negative]`: no WebSocket crate in `Cargo.lock` at the time).
opencode's serve transport measured out as **HTTP + SSE**, not
WebSocket [measured, probe amendment W1/W3], and `reqwest` 0.13
(rustls, json) was already a direct dependency `ApiClient` uses for SSE
reads (`src/api.rs:4300-4630`). W3 therefore needed exactly one
`Cargo.toml` line — the `"blocking"` feature flag, already pulled in
transitively via `opentelemetry-otlp`'s `reqwest-blocking-client`,
`Cargo.lock` verified byte-identical before and after — an R5
(installed-dependency) rung citation, not an R6/R7 new-crate one, and
zero net change to the dependency graph. The carve-out the owner
offered went unused because the facts didn't ask for it, which is
itself worth recording: A4's WebSocket ladder is *still* only reopened
by a wave whose own measured facts require it, and this wave's facts
did not.

Both transports decode into the same `NativeEvent` shapes through one
shared decoder path inside `opencode.rs`/`opencode_serve.rs` (the
`TurnAccumulator`/`decode_export` machinery, reused verbatim across
run-json's `export` and serve's `messages()` via a one-line envelope
shim) — the narration rule holds structurally, not by two decoders
agreeing today, exactly as ADR 0020 states for codex's
`ItemView`/`TurnAccumulator`.

### R4 deltas: four measured firsts, none of them guessed

ADR 0020 names R4 as "parity is the floor, not the ceiling." This wave
cashes it in four times, each with a named admission test — not
prose:

1. **`history: true`**, first landed by W1, via **`opencode export`**
   [measured, probe 6]: `{info, messages:[{info:{role, modelID,
   providerID, finish, tokens, cost}, parts:[…]}]}`, including
   `reasoning` parts the SDK's own documented `Part` union does not
   list — token-free and complete
   (`live_opencode_history_exports_the_whole_session`). Neither
   `claude.rs` nor `codex.rs` claims this flag. W3 re-earns it on
   serve via **`GET /session/{id}/message`**
   (`serve_messages_and_export_decode_to_identical_history`), measured
   structurally identical to `export` on the same rich four-message
   session (an aborted tool call included) — no completeness gap
   either direction, cheaper (no subprocess), so serve's `messages()`
   wins the tie-break while `export` stays run-json's own source.
2. **`approval_flow: true`**, the registry's first true on this flag
   anywhere (claude and codex both `false`). `permission.asked`
   [measured] parks the stage as `NeedsInput{asked_by: Adapter}`; SEND
   relays `once`/`always`/`reject` to the **deprecated-but-live v1**
   endpoint (`POST /session/{id}/permissions/{permissionID}`) — see
   the permission-fork subsection below for why v1, not the OpenAPI
   doc's own v2. `live_opencode_serve_approval_round_trip_
   runs_the_gated_tool` ran for real against installed 1.18.19,
   `-m opencode/big-pickle`, and passed.
3. **`ask: true`** on serve — measured through a genuinely distinct
   mechanism from approval_flow, not a relabeling of it: opencode
   ships a separate `question` tool with its own typed
   `question.asked` event (`que_` ids, disjoint from
   `permission.asked`'s `per_` ids) naming the actor's own
   `tool.callID`. Actor authorship is schema-distinguishable, not
   guessed from a text part, which is exactly what
   `Capabilities::ask` requires. Measured end to end:
   `{answers:[["Blue"]]}` → `question.replied` → the session resumed
   *itself* with no further client call and produced "You prefer
   blue." (`live_opencode_serve_actor_question_parks_and_resumes_on_
   answer`). This is the one place codex's own protocol had a
   comparably clean candidate (`item/tool/requestUserInput`) and never
   produced a measured result at all (ADR 0020's own "open admission
   test, not a claimed negative" section) — opencode's did.
4. **`interrupt`** upgrades from `ProcessTreeTermination` (run-json,
   `Child::kill()` plus `process_group(0)`/negated-pgid SIGKILL — probe
   11 measured a bash-tool grandchild surviving a plain `kill()`) to
   **`NativeSessionAbort`** on serve: `POST /session/{id}/abort` [live
   measured] kills the tool's own subprocess tree cleanly (no surviving
   `sleep 30`, the exact scenario probe 11's raw SIGKILL orphaned) and
   leaves the session usable for a follow-up turn. A **live-measured
   deviation from the wave's own written spec**, recorded rather than
   silently absorbed: the *synchronous* `POST /session/{id}/message`
   response itself settles with `info.error.name ==
   "MessageAbortedError"` on an aborted turn, not only via a separate
   SSE `session.error` frame as the spec assumed —
   `classify_serve_terminal_recognizes_an_abort_signature_on_the_
   post_response_itself` pins the corrected behavior. Any abort-RPC
   failure falls back to the process-group kill and journals
   `phase:"interrupt_downgraded"` (codex's own §7.3 precedent).

A fifth adapter-local capability with no v1 boolean, the same posture
codex's `structured_output` row takes (R3): `format:{type:
"json_schema", schema}` on `POST /session/{id}/message` → a synthetic
`StructuredOutput` tool part, decoded as an ordinary
`tool.requested`/`tool.completed` pair. **Corrects the wave's own
plan's guess**: the result lands at `info.structured`, not a field
named `structured_output`, and `info.finish` is `"tool-calls"` on such
a turn, not `"stop"` — a classifier treating non-stop as abnormal would
have marked every structured turn abnormal. W3 wires the channel and
synthesizes no schema of its own; sergeant has no per-stage
output-schema surface, and inventing one is a core change (K2).

### The permission v1/v2 fork and the auth-username quirk — recorded
with fixture citations because the docs get both wrong

Two doc-contradicting facts, both load-bearing for `approval_flow`,
both measured rather than assumed:

- **The permission fork.** opencode's own OpenAPI document defines
  *two* permission-reply surfaces: a **deprecated v1**
  (`POST /session/{id}/permissions/{permissionID}`, event
  `permission.asked`) and a nominally-current **v2**
  (`/api/session/.../permission/.../reply`, event
  `permission.v2.asked`). On the installed 1.18.19 build, **only v1
  actually fires** — `permission.v2.asked` never appeared in any
  captured event stream, and a reply attempted against the v2 endpoint
  404'd with `PermissionNotFoundError` [measured, W3 serve amendment].
  The adapter therefore relays every reply to the deprecated endpoint,
  on the documented API surface being wrong about which of its own
  contracts is live — a distro-shipped fork the adapter measured
  around rather than trusted. A third endpoint,
  `POST /permission/{requestID}/reply`, matches `permission.replied`'s
  own `{requestID, reply}` vocabulary in the OpenAPI doc and was never
  tried; it is recorded as the alternative to measure, not silently
  preferred, in the `approval_flow`/Serve admission row's own note.
- **The auth-username quirk.** `OPENCODE_SERVER_PASSWORD` sets the
  password, but the doc is silent on the username: it must be the
  **literal string `opencode`** — any other username with the correct
  password still 401s [measured, W3 serve amendment]. The OpenAPI
  document's `securitySchemes` are empty; auth is middleware the schema
  never describes. `get_doc_names_the_username_rule_on_401`
  (`opencode_serve.rs`) pins this fact by name rather than leaving it
  as tribal knowledge the next reader has to rediscover by a failed
  request.

Both facts live in the module doc comments of `opencode.rs`/
`opencode_serve.rs` as well as here, on the same theory ADR 0020 states
for its own sandbox-amendment correction: a superseded or
doc-contradicted claim stays legible where the code lives, not only in
this record.

### No native OS sandbox exists — NORTH-STAR amendment 4 has nothing
to bind, so this ADR does not append to it

ADR 0020's own NORTH-STAR amendment 4 (2026-08-21) says an adapter
*may* use its harness's native enforcement where the harness has one,
scoping core's "no OS sandbox" non-goal to core, not adapters — and
records codex's `workspace-write` sandbox as the first adapter to use
that allowance, "enforcement-claimed, not locally proven" on this
host. **opencode has no analog to claim.** opencode's own
documentation is exhaustively silent on any OS-level sandbox
mechanism; the only controls opencode exposes are its permission
config (`ask`/`allow`/`deny` per tool glob) and per-tool disables —
policy the model's own tool-call layer honors, not a kernel-level
write barrier the way bubblewrap is for codex or `--network=none`
bind-mounts are for docker. Amendment 4's second sentence — "an
adapter MAY use its harness's native enforcement" — is permissive, not
mandatory, and this adapter has nothing that qualifies as "native
enforcement" to reach for. **Sergeant's observation layer therefore
stays the sole source of truth for this adapter exactly as it already
is for core**, with no belt added over the braces, and no dated
amendment is appended here because there is nothing to record beyond
what amendment 4 already says in its own third numbered consequence:
an adapter's enforcement, where none exists, changes nothing about
where sergeant charges dirty evidence from.

### The admission-rows / L8-structural pattern — reused verbatim, one
real divergence

Contract v1 (`src/backend/mod.rs`) is untouched (R3): thirteen
booleans plus `AskAuthor`/`RuntimeScope`, no typed capability enum.
This adapter's `AdmissionRow` ledger (`opencode.rs:535-988`,
`Evidence::{LiveMeasured, LocallyMeasured, DocClaimed, Unmeasured}`)
and its own compile-adjacent structural test
(`admission_rows_agree_with_capabilities`) are ADR 0020's own pattern,
copied rather than re-derived, exactly as that ADR invited a future
adapter to do. **No `Stability` column** — deliberately, unlike
codex's ledger: every row here would carry the identical value
(opencode publishes no API/CLI breaking-change policy for any surface,
and the upstream repo itself moved `sst` → `anomalyco` mid-flight), so
the fact is stated once in `render_admission_rows`'s own header rather
than repeated in every row.

**The one real transport divergence from codex's ledger, named because
it is the interesting fact, not an oversight:** codex's two transports
(`exec`, `app-server`) claim an *identical* `Capabilities` value, so
one struct serves both. opencode's two transports claim **different**
capability sets on purpose — serve adds `approval_flow` and `ask`,
both structurally `false` on run-json (a permission resolving to `ask`
auto-rejects on `run`; there is nobody to approve to) — so
`capabilities_for(transport)` is a pure function of which transport a
registration actually resolved to, and the structural test drives both
independently. A registration that resolved to run-json can never
advertise serve's flags; the type system, not a review discipline,
enforces it.

## K2 exception ledger — every touch outside `src/backend/` this
sprint made, complete, for owner ratification at the head PR

K2's own text is "scope is the adapter; no core changes." Six items
touched something outside `src/backend/opencode*.rs`, each named here
with its wave, its reason, and why it does not read as a core change
in spirit even where it touches a file outside that directory:

| Item | Wave | File(s) | Reason |
|---|---|---|---|
| Required PUT-site + recovery-arm row | W1 | `tests/a4_blob_ref_pinning.rs` | Gate-forced, not discretionary: A4's own blob-ref-pinning suite requires every new blob-capture site (opencode's raw-stream archive) to carry a recoverability row, or the suite itself fails closed. The adapter cannot exist without this row; it is the suite's own admission gate operating exactly as designed, not a scope creep W1 chose. |
| `"opencode"` registered in `BackendRegistry` | W2 | `src/daemon.rs` | `DaemonConfig` gained an `opencode: Option<OpencodeConfig>` field plus the construction/registration and event-sink-wiring block, mirroring the codex block exactly. Named here rather than folded into "adapter work" because it is a real edit to a core file outside `src/backend/`, even though ("Harness and backend axes," above) it decides nothing new: `router.rs`'s precedence ladder and `sgt opencode`'s origin affinity already existed, and registration only makes the name they already resolve toward real. |
| `~/.opencode/bin` added to `toolchain_path_dirs` | W2 | `src/harness.rs` | Pre-ratified by name in the sprint plan itself (§ W2, "this sprint's one touch outside `src/backend/`"): the packet's own measured `command not found` evidence, and `harness.rs`'s module doc already states the list is designed to grow by measured entries — the same one-line remedy that put `~/.cargo/bin`/`~/.local/bin` there. |
| Registered-backend count/list widened in three pre-existing fixtures | W2 | `tests/m3_execution.rs`, `tests/m2_daemon_api.rs`, `tests/m4_backends.rs` | Mechanical, not a design choice: registering a fourth backend moves every fixture that hardcoded "how many backends exist" or used `"opencode"` as a canonical *unregistered* name (swapped to `"goose"`, which stays unregistered). A direct, forced consequence of item 2 (`daemon.rs` registration) in the same commit, never a separate decision. |
| `reqwest`'s own `"blocking"` feature flag | W3 | `Cargo.toml` | The owner's own offered carve-out, ultimately unneeded for a new crate (see "The WebSocket carve-out" above) but used here in its narrower form: making an already-transitively-enabled feature (via `opentelemetry-otlp`'s `reqwest-blocking-client`) an explicit direct one. `Cargo.lock` verified byte-identical before and after — an R5 rung citation, zero net dependency-graph change. |
| #231(b) orphan-suite guard, written from scratch | W2 | `tests/coverage_stage_membership.rs`, `scripts/coverage/c2-suites.sh` | Neither file existed before this sprint's own W2. `coverage_stage_membership.rs` is a new structural test asserting every `tests/*.rs` suite is wired into a coverage stage script or named in its `ALLOWLIST`; that `ALLOWLIST` was seeded with the 18 suites already orphaned at authorship time (2026-08-23), a fact this sprint discovered, not inherited. `opencode_routing`/`opencode_backend` were wired into `c2-suites.sh` in the same commit that created them, so neither is itself an orphan — the guard's first act was to certify this sprint's own new suites, not to widen a ledger some earlier sprint had already built. |

None of the six touch `src/backend/mod.rs`'s contract, the router, or
the engine — K2's actual substance (R3) is intact. They are named here
individually, rather than folded into "adapter work," because K2's own
words promise "no core changes" and a reader auditing that promise
should be able to check each item against the reason it was necessary
rather than take "the adapter" on faith.

## Consequences

OpenCode is a fully registered, routed, capability-honest backend
alongside Claude, Codex, and Docker. Every capability it claims carries
a named admission test or a named, specific negative, never a
schema-only promotion — the same discipline ADR 0020 established and
this adapter's own structural test enforces at compile time. Four
capabilities exceed both existing native adapters
(`history`, `approval_flow`, `ask`, and `interrupt`'s
`NativeSessionAbort` tier), each earned by a measured admission test,
each recorded with the specific mechanism and the specific fixture
that proves it, per R4. The version-provenance posture ADR 0020 made
repo-wide holds here without exception or special pleading — this
adapter never had a refusal branch to strike, because it was written
against R1 from its first commit. No native sandbox exists for this
adapter to claim, which this ADR states as a fact about opencode, not
a gap in the adapter's own coverage. The K2 exception ledger above is
this sprint's honest account of every place work outside
`src/backend/` was actually required, offered whole to the owner
rather than left implicit in a diff.

## Open questions / hand-offs

- **`native_subagents`** is `Evidence::DocClaimed` on both transports:
  opencode has agents (`opencode agent`, `run --agent`,
  `--agent` on `session.create`/`session.prompt`) [doc-claimed]; no
  subagent turn was ever run, on either transport. Documented is not
  supported (§15) — this is an open admission test for a future wave,
  not a claimed negative.
- **#231(a)-style audit**: `tests/coverage_stage_membership.rs` and its
  `ALLOWLIST` did not exist before this sprint — W2 wrote the guard
  from scratch (#231(b)) and, in the same pass, discovered 18 suites
  already orphaned at authorship time (2026-08-23), seeding the
  `ALLOWLIST` with them. That same commit wired this sprint's own two
  new suites, `opencode_routing`/`opencode_backend`, into
  `c2-suites.sh` before the guard could ever see them as orphans — the
  two actions are independent, not the same "widening": wiring a suite
  into `c2-suites.sh` keeps it *out* of the allowlist, it does not add
  to it. Named here so a reader does not assume this sprint's own
  coverage-lift work (W4 Job 1) touched the 18-suite seed list; it did
  not — that count has been unchanged since the commit that first
  wrote it.
- **The 18-suite allowlist itself**: pre-existing orphans discovered
  when this sprint's W2 first wrote the guard, unrelated to what this
  adapter does, carried forward unmodified since. Auditing and closing
  those 18 individually is #231(a)'s own hand-off, not this ADR's.
- **`always`'s persistence** (the approval_flow reply body) is relayed
  by this adapter but its actual durability across a later turn was
  schema-read only, never exercised end to end — recorded in the
  `approval_flow`/Serve row's own note, not re-litigated here.
- **Multi-select questions and `POST /question/{id}/reject`** are
  schema-claimed and unwired: the `ask` admission test's own answering
  path handles exactly one question with one exact label match; a
  second question, or an unmatched label, is a structured refusal
  naming the labels, never a guess.
- **Re-spawning a serve child against a durable session id after
  RESUME** (rather than withdrawing to run-json, this wave's actual
  behavior) is plausible but unmeasured — named in the `resume`/Serve
  row, a genuine future-wave question, not a decision ducked here.
- **ACP** — see "The transport story" above: unmeasured, not refused,
  reopened only by a wave whose own facts require it (A4's own
  standard, reapplied).

## Alternatives considered

**A shared, host-global `opencode serve` daemon** (one listener across
every Work on the host) — refused for the identical reasons ADR 0020
refused `codex app-server daemon`: a blast radius shared across every
execution and every human session on the host, and an auth-posture
question (this wave's own measured auth-username quirk makes that
question sharper, not softer) that belongs to a future J0-scoped
decision, not one wave's adapter work.

**Composing `--auto` on `run`** to avoid the auto-reject-on-`ask`
path — refused: probe 4 measured `--auto` auto-*approves* every
permission not explicitly denied, which is the opposite safety
property this adapter needs for non-interactive Work execution.
`non_blocking_run`'s own admission row (`AutoRejectOnAsk` tier) is the
mechanism actually kept.

**Guessing an actor question from a `text` part** rather than waiting
for opencode's typed `question.asked` event — refused directly: this
is precisely the heuristic `Capabilities::ask`'s own contract forbids,
and the narration rule (§ "The transport story," above) exists to make
that refusal structural rather than a matter of adapter-author
discipline.

**A typed `Capabilities` v2 enum for this adapter alone** — refused by
R3, identically to ADR 0020's own refusal of the same idea for codex;
the adapter-local `AdmissionRow` ledger is what was actually built,
designed to lift whole into a real v2 when one lands, nothing here
thrown away in the meantime.
