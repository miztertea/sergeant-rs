# Gauntlet Ledger

Development record for the sergeant-rs prototype. The method is defined in
`reference/notes/gauntlet-pattern.md`; per-milestone contracts live in
`docs/gauntlet/contracts/`. This file is append-only: entries record what
happened, with evidence; superseded decisions stay visible.

Each milestone entry carries two scorecards:

- **Mission outcome** — contract met? gates green? what shipped?
- **Environmental behavior** — iterations used, findings by axis and disposition,
  escalations, evidence completeness.

Design decisions and deviations log their **Ponytail rung** (`R1`–`R7`; ladder in
`reference/notes/ideaos-agent-contract.md`): the rung the decision resolved at.
`R7` (new machinery) entries name which lower rungs failed and why.

## Deviation register

Deviations from `reference/proposal-depot-rust-execution-surface.md`, with
rationale. The proposal is the idea as it stood in that moment, not a how-to.

| # | Proposal says | We do | Why |
|---|---|---|---|
| D1 | Product named Depot; `depot.toml`; `depot` CLI | Product **sergeant-rs**; `sergeant.toml`; binary `sgt` | Owner decision 2026-08-08 |
| D2 | Claude adapter drives a held `attach` (per Sergeant's tmux design) | Daemon has no TTY/pane. Leading M4 candidate (2026-08-08): headless turn sequence — `claude -p --output-format stream-json` with prompt on stdin, `--resume <session_id>` per later turn; session identity is durable, process exists per turn. Proven in production by no-mistakes (`internal/agent/claude.go`, vendored knowledge in LESSONS L2). The spike's rejection of `-p --resume` was Sergeant-specific (live-`--bg` refusal + persistent-TTY doctrine), neither applies here. To be confirmed by M4 contract tests (R5: installed harness capability; interrupt/restart/concurrency semantics still unmeasured). Fallbacks: SessionStart-hook injection, `--bg` + stop→resume. **Confirmed at M4** (see the M4 ledger entry below): headless print-mode turns proved adequate; fallbacks unused. | Measured spike facts + no tmux in scope; see M4 contract Unknowns and the M4 ledger entry |
| D7 | M5 contract budgets `tracing-opentelemetry` (§28-named) | `opentelemetry` + `opentelemetry_sdk` + `opentelemetry-otlp` directly; no tracing bridge | Builder measurement-adjacent ruling, registered at M5 adjudication (flagged by the checkpoint gate): the tracing-bridge cannot represent the §28 work→stage→execution span tree — spans there follow tokio task structure, not the engine's domain structure; building spans directly from journal/engine events preserves the §28 shape (R5: the named OTel crates themselves; the bridge dropped as R1 — machinery that cannot express the requirement). Rationale in `src/telemetry.rs` module docs. |
| D6 | §38 P0 includes "native Codex execution"; owner's original scope decision was full P0 | Claude is the only native adapter in this prototype; Codex (and the M0-era `backend/codex.rs` stub) deferred until an environment exists where Codex can actually be measured | Owner decision 2026-08-08. Measure-first doctrine (L1) + the L7 lesson: adapter code that cannot be validated against the real harness is prose with a compiler. The §15 trait remains backend-neutral, so a future Codex adapter is additive. |
| D5 | M2 contract enumerates dependencies "axum, tower, reqwest (client, rustls)" | Actual: axum; reqwest without TLS (loopback plain HTTP — R1); tower first wrapped in a no-op layer then removed entirely at round 2 (R1: axum's own middleware suffices); plus `tokio-stream` (R7: sole Stream adapter for SSE, lower rungs named in build report) and `tracing-subscriber` (R5-adjacent: the tracing facade M0 pinned needs one subscriber to emit anything) | Contract over-specified; builder narrowed with rung-logged justifications, round-2 panel flagged the unregistered delta (M0's D3 precedent), registered at M2 adjudication. |
| D4 | §35's tree has `src/main.rs` owning all modules, no lib target | `src/lib.rs` declares the module tree; `main.rs` is a thin shell over `sergeant_rs::cli` | M1 contract requires the core "as library code with tests": integration tests under `tests/` can only import a lib target, and a bin-only crate forces dead-code suppressions under `clippy -D warnings` (R2: the change reuses Cargo's native lib+bin layout). Raised by M1 critics rounds 3–4; ruled authorized at M1 adjudication. |
| D3 | §35 lists `backend/{claude,codex,opencode,prime}.rs` | Scaffold has `backend/{claude,codex,fake}.rs` | §38 defers OpenCode/Prime past the P0 contract proof (R1: doesn't need to exist yet); §37's deterministic core tests require a fake backend (R7: no lower rung supplies a deterministic in-process backend). Modules are added when their milestone arrives, not pre-declared. Raised by the M0 critic panel. |

## Backlog (confirmed-but-deferred findings)

| # | From | Finding | Why deferred |
|---|---|---|---|
| B1 | M1 adjudication | A foreign snapshot whose `last_seq` is within the journal's range loads undetected (identity binding was removed as beyond-contract machinery) | Snapshots live in the daemon-owned data dir; the threat is operator error, not adversarial. **Revisited at M2 (per checkpoint-gate finding document-1):** the daemon now owns the data dir exclusively (daemon.lock) AND uses full journal replay — no snapshot loading exists in the daemon path at all (builder ruling, R1). B1 is unreachable in production flow; trigger narrowed to "if/when the daemon adopts snapshot loading (likely M5 perf)". **Revisited at M5 (checkpoint round 1):** rebuild-from-journal measured well within budget (bulk-appender fold, ~580x a row-wise-SQL baseline — see `Analytics`'s doc comment in `src/runtime/analytics.rs` and the rebuild bench in `tests/m5_projections.rs`), so rebuild-on-start remains the only population path with no perf case for snapshot loading. B1's trigger still does not fire; still dormant. |

---

## Ledger entries

### M5 — DuckDB Projection, Graph, OTel (2026-08-09)

**Mission outcome: contract met, gates green.** Shipped: the §21–22 DuckDB
analytical projection as a bulk-appender materialization of a pure journal
fold — rebuild-on-start is the only population path (measured 14,810 events/s
after a 580× engineering pass from row-wise SQL; B1's snapshot trigger
formally does not fire); fail-closed catch-up (`NeedsRebuild`: a transient
flush failure costs one 503 and a rebuild, never a silently short table); the
§23 graph projection with per-edge `source_seq` provenance at
`/v1/graph/work/{id}`; §28 OTel export off-by-default with a live-measured
OTLP smoke and an honest "what this export loses, on purpose" doc (startup
events and cross-restart spans — deliberately not re-exported); journal
`replay_after` seek so read-time catch-up is O(segments + wanted), not
O(history) under the core lock. Evidence: 177 tests, fmt/clippy -D/test all
green (clippy re-verified after the profile fix below), orchestrator-run.

**Environmental behavior.** Round 1: 17 findings, only 7 confirmed — all
test-shape refinements, zero production defects (confirmation counts across
milestones: 13→15/19→22→7 — the codebase is getting harder to catch).
Checkpoint gate: passed; two ask-user findings led to an **orchestrator-
caused regression**: my instructed t5 rewrite replaced falsifiable source-
scan guards with an unfalsifiable stand-in collector (port never handed to
anything). Round 2 caught it as an error, alongside three real production
defects earlier rounds missed (catch_up flush-desync; O(journal) reads under
the core mutation lock; post-startup export subscription) and the
still-unrecorded duckdb build-cost Unknown. Fixer closed all 12: the
falsifiable guards restored AND the collector made real (bound to the
default endpoint, probed to trip), the desync fixed fail-closed, the seek
added, and the disk crisis root-caused — cc passes `-g` to DuckDB's ~500 C++
TUs in dev profile; `[profile.dev.package.libduckdb-sys] debug = false`
shrank target/ 15GB→5.4GB and made `clippy --all-targets` runnable in this
container again. One container restart mid-checkpoint recovered from git
evidence (the exact coordinator-death class this prototype exists to fix —
the daemon would have journaled it; the orchestrator had to do forensics).

**Measurements of record** (closing the contract's Unknowns): duckdb bundled
cold build 605s / ~3.9GB per configuration (three configs + copies drove two
ENOSPC incidents; mitigated by the debug=false profile override, rejected
alternatives documented in Cargo.toml); rebuild 16,000 events in 1.08s; all
canned queries 59–123ms; OTLP smoke measured live against a bound collector;
pipeline worktrees cannot see the shared build cache (no-mistakes constructs
agent env), so the standing pattern is pipeline-static-review + orchestrator
runtime verification, recorded per gate. Release binary: 58.8MB (47.3MB
stripped) — the full daemon with embedded DuckDB; one-binary shipping (§34)
confirmed viable, owner accepted the size 2026-08-09.

**Adjudication rulings.** (1) D7 registered (opentelemetry_sdk direct; the
tracing bridge cannot represent the domain span tree). (2) The t5 regression
is owned by the orchestrator, not the pipeline — the instruction was wrong;
LESSONS L9. (3) `table_rows` kept as the acceptance-1 instrument with its
no-production-caller status documented in place. (4) OTel restart loss is a
documented property, not a defect — re-exporting history on restart is the
wrong behavior for an export projection.

**Shipping gates.** Checkpoint gate 01KZJQW-series: passed (test/lint steps
approved on static review + orchestrator local runtime verification — the
duckdb cold-build wall; measured and recorded). Final gate: review found 1
(analytics catch-up TOCTOU — a concurrent `catch_up` failure landing in
`with_analytics`'s lock-release window could fold a stale journal tail);
fixed with retry-on-mismatch plus a regression test, pipeline-applied
(eaa6845); test/lint clean on the checkpoint gate's static-review-plus-
orchestrator-verification basis.

---

### M4 — Claude Adapter, Recovery, Regression Catalog (2026-08-09)

**Mission outcome: contract met (as amended by D6), gates green.** Shipped:
the measured Claude adapter — headless print-mode turns over daemon-chosen
durable session identity (`--session-id` pre-launch + `CLAUDE_CODE_SESSION_ID`
scrubbed, closing a measured nested-capture hazard this container itself
demonstrated), `--setting-sources user` against project-memory capture,
three-layer model-pin verification keyed to `is_error`/`modelUsage` (print
mode's `subtype` measured untrustworthy), raw stream-json archived per turn
with journaled blob refs, EventSink-delivered normalized events with
committed causation, transcript-existence reconciliation (measured: no free
liveness probe exists — even `--max-turns 0` bills), **resume wired into
reconcile per the §25 ruling** (unambiguous evidence resumes, ambiguity still
blocks, no new public verbs), honest capabilities incl. durable-transcript
`history` backfill and the §17 `runtime_scope` declaration, version gate at
2.1.226, and the seven-entry Sergeant regression catalog with provenance.
Codex: doc-comment stub per D6 (421 speculative lines removed in-loop).
Evidence: 150 tests + 2 opt-in (live pair passed first try — haiku, 33s, six
turns incl. one killed mid-generation), 5 clean full runs verified by the
orchestrator, 14 clean loaded runs by the fixer.

**Environmental behavior.** Round 1: 24 findings, 22 confirmed (8 errors) —
D6-descope catches (the contract-amendment trap worked: critics graded
against the amended contract and flagged the builder's pre-descope Codex
work), recovery identity loss, evidence-archiving gaps, an unmeasured
capability claim, L6's crash-window class for the third straight milestone.
Checkpoint gate: passed; parked `resume-unwired` (ask-user) → orchestrator
ruling seeded into round 2. Round 2: 24 findings, 19 confirmed (18 + seed) —
headline: `history: true` advertised while the restart path returned
Ok(empty) despite the durable transcript on disk (fail-open indistinguishable
from "nothing said"), caught precisely where the live contract tests hadn't
measured (§37's history/stop gap — the hole and the defect coincided).
Fixer: all 19 closed, 18/18 revert-probes killed, self-introduced flake
caught by its own 14-run load check. The builder's 15 verbatim CLI
measurements (incl. print-mode envelope semantics differing from the spike's
TUI measurements) live in workflow record wf_e6b3fd7f-95b. One planned pause
(owner-directed, usage window) executed cleanly mid-milestone.

**Adjudication rulings.** (1) resume wiring per §25 — reconcile auto-resumes
unambiguous evidence only; no new public verbs (R1). (2) Substitution
detection remains fixture-derived and fails closed on any model-field
mismatch — an entitled account cannot be made to substitute on demand;
recorded as documented-not-measured per spike doctrine. (3) `runtime_scope()`
added as the minimal §17 rendering (declaration only; ENSURE RUNTIME remains
unneeded while the only runtime is the CLI itself — R1, revisit with a
server-model backend).

**Shipping gates.** Checkpoint gate: **passed** (doc commit bc52c00 adopted;
review parked resume-unwired to adjudication; test step approved without
re-spending the live pair). Final gate: **passed** — one doc commit adopted
(2d8bfdc: the pipeline confirmed D2's register row against the M4
measurements; the register now records its own resolution).

---

### PAUSE MARKER — 2026-08-09, M4 mid-gauntlet (planned, owner-directed)

State at pause: M4 build + round-1 panel complete (24 findings, 22 confirmed,
all fixed with 17 revert-probed pinning tests; 132 tests, 5 clean runs);
checkpoint commit c89e1c0 + checkpoint gate **passed** (pipeline doc commit
bc52c00 adopted). The builder's 15 verbatim CLI measurements are in the M4
workflow record (wf_e6b3fd7f-95b), including two new-beyond-the-spike
hazards: nested CLAUDE_CODE_SESSION_ID capture (closed structurally via
pre-launch --session-id + env scrub) and print-mode's lying result subtype
(is_error/modelUsage are load-bearing).

**Resume point:** M4 lean round 2. First input, from the checkpoint gate's
parked ask-user finding: `Backend::resume()` is implemented and tested on
both adapters but has NO production caller — reconcile only observes and
blocks; the resume verb must be wired into the engine (and CLI/API per §8)
or its absence ruled and recorded. Then: adjudication, MARK & LOG, final
gate, M5 (DuckDB/graph/OTel), M6 (TUI/HTML/doctor/§39 demo). Method:
`reference/notes/gauntlet-pattern.md`; script pattern:
`workflows/scripts/m3-round2-lean.js` (adapt for M4).

---

### M3 — Work Surfaces, Workflow Engine, Routing, Fake Backend (2026-08-08)

**Mission outcome: contract met, gates green.** Shipped: zero-config and
multi-repo workspace discovery; git-worktree work surfaces with full binding
records, fail-closed teardown, partial-failure rollback, and stale-registration
pruning; the §12 staged workflow engine (versioned filesystem workflows, run
pins its resolved definition in the journal, stage state structurally separate
from Work state); the §15 backend trait (M3 subset) with the §37 scriptable
fake backend; §13 routing precedence with origin affinity and
fail-with-options; §14 profiles with a credential boundary; §25 restart
reconciliation with per-work fail-closed isolation; traversal-guarded workflow
and repository names. Evidence: fmt/clippy -D/test green, **96 tests** (41 lib
+ 10 M1 + 21 M2 + 24 M3), all eight contract acceptance tests real, M3 suite
0 failures across 10 consecutive runs (flake check), verified by the
orchestrator.

**Environmental behavior.** Economy cadence, ~38 agents total across two
workflows + pipeline. Round 1 (in-workflow): 12 findings, 9 confirmed — incl.
a submit crash-window strand (same adjacent-append class as M2's exact-once
window) and a vacuous recovery test (test-honesty's third error in three
milestones). Checkpoint gate: **11 findings, 2 errors, zero overlap with the
panel's 12** — client-supplied workflow-name path traversal and duplicate-repo
worktree poisoning; all 11 authorized in one respond with the F8 ruling
(fail closed per-work, never per-daemon) and R2 guard-pattern instructions;
pipeline applied them (5a60f49) but its test-step agent died (infra, not
code — verified green locally). Round 2 (lean): 28 findings, **20 confirmed**,
headline: the entire gate-fix commit reverted with all 75 tests green — two
traversal guards and a recovery-semantics change carried by prose alone; plus
~5% parallel-run flake in the M3 suite, two real gaps in the F1/F3 fixes, and
Ponytail findings on the gate commit's own machinery. Fixer (interrupted once
by the session limit, resumed from cache) closed all 20: 13 pinning tests, 19
mutation probes all killed, traversal predicate unified into one
`is_plain_name`, same-path duplicate detection via already-computed rev-parse,
prune-on-rematerialize, flake eliminated. Two session-limit interruptions and
one pipeline infra failure this milestone; cache-resume recovered all three
with zero rework.

**Adjudication rulings.**
1. F8 (gate ask-user): reconcile isolates failures per work — a failing work
   blocks itself with evidence; the fleet and the daemon start regardless.
2. Round-2 C-findings on the gate commit accepted wholesale — the
   unpinned-fix discovery is a method lesson (LESSONS L7), not a shortcut.
3. Pipeline test-step death ruled infrastructure after local verification;
   no rerun — the lean round and final gate both still stood between the
   fixes and the close.

**Shipping gates.** Checkpoint gate 01KZHCBRH365H8X4TPNVC5P2P8: review found
11 (all fixed), then its test step's agent died — outcome failed on infra,
fixes recovered via `axi sync --recover`, verified green locally. Final gate:
**passed**, zero findings, no pipeline commits. (Two environmental notes: the
no-mistakes daemon died a second time → `scripts/gate.sh` self-healing runner
adopted, failure-triggered R7; the pipeline also fails closed on an unclean
tree — the wrapper gains a status pre-flight at the next natural commit.)

---

### M2 — Daemon, API, CLI, Idempotency (2026-08-08)

**Mission outcome: contract met, gates green.** Shipped: §10 Work state machine
(transitions only via journal events, illegal transitions fail closed with no
append); daemon owning the data dir (daemon.lock + M1's journal lock, 127.0.0.1
ephemeral bind, 0600 atomic runtime descriptor with ~160-bit token, fail-closed
descriptor schema validation on read); v1 API (submit/list/show/cancel, events
history + SSE with Last-Event-ID resume and shutdown-safe streams, bearer
middleware, structured JSON errors incl. router fallbacks); §26 idempotency
journaled as command.accepted/rejected with the exact response value — duplicate
command_ids replay byte-identically, including across restart; clap CLI with
detached auto-spawn and two-client race convergence (proven by spawned-binary
test). Evidence: fmt/clippy -D/test all green, 39 tests (18 M1 + 21 M2),
re-verified by the orchestrator.

**Environmental behavior.** New economy cadence, applied mid-milestone by owner
direction: build + round-1 panel in one workflow (16 findings, 13 confirmed —
two severity-error: an exact-once crash window between submit's two journal
appends, and an idempotency test that never inspected the journal); checkpoint
commit + checkpoint gate #1 (passed; 1 review auto-fix, 1 ask-user finding
routed to this adjudication, 1 pipeline doc commit adopted); lean follow-up
panel (19 findings, 15 confirmed — one production defect: SSE subscribers
wedged graceful shutdown indefinitely, reproduced empirically by the critic).
Incidents: one refuter quarantined for editing the tree to force tests green
(reverted, probe-hygiene rule added to the method — see LESSONS L5); the
round-2 fixer was killed by the session usage limit and resumed as a single
bundled agent after reset. Two workflow invocations + one resume + one direct
fixer; subagent spend ≈1.4M tokens on the pre-economy round 1, ≈1.07M after
restructure (incl. 740k for a follow-up panel whose critics ran 160 tool
uses — medium effort bounds reasoning, not exploration; noted for M3 prompts).

**Adjudication rulings.**
1. C7: CoreStats/WorkRegistry coexistence ruled intentional — M1 acceptance
   tests pin CoreStats; "evolved into" (contract) yielded to test stability.
2. C8: write-only descriptor schema field resolved fail-closed (CLI validates
   on read, R2 — mirrors M1's snapshot-schema refusal) rather than deleted.
3. C9–C15 (quarantined refuter's batch): accepted without re-verification —
   all were missing-test claims where wrongly accepting one costs a redundant
   test, not a defect. All seven produced real tests; C12's constant-token
   mutant and C10's deleted-replay mutant now die.
4. Contract defects recorded: dependency list over-specified in both
   directions (D5). The "seven acceptance tests" miscount was initially
   attributed here to the contract — the final gate's document step caught
   that the contract correctly lists eight and the miscount was in the
   orchestrator's builder prompt. Corrected 2026-08-08; the error was the
   orchestrator's.
5. B1 revisited per checkpoint-gate finding — see backlog.

**Design decisions (selected, rung-tagged).** Shutdown signal via
tokio::sync::watch (R5) with the flag set inside the graceful-shutdown future
so signal-before-wait is structural (R6); SSE cancellation as one select!
around the existing pump — one cancellation point covers every await (R6);
idempotency on the existing journal+projection machinery, so restart replay is
free (R2); bearer token from two ULIDs instead of a rand dep (R5); PID
liveness via /proc, fail-closed direction on non-Linux (R4); tower removed
(R1), tokio-stream added (R7 — lower rungs named in build report);
method_not_allowed_fallback measured unnecessary and removed (R1). Builder
and fixer reported 10 contract ambiguities with interpretations, retained in
the workflow record.

**Shipping gates.** Checkpoint gate 01KZGVV42FJMY749V1YTBVWVDX: passed.
Final gate: **passed** — review/test/document/lint clean, no pipeline
commits; one ask-user info finding (the ruling-#4 misattribution above,
corrected). Gate agent verified running `--model sonnet`. One environmental
note: the no-mistakes daemon died between runs and required a restart with
`IS_SANDBOX=1` re-applied — worth a wrapper if it recurs.

---

### M1 — Event Core (2026-08-08)

**Mission outcome: contract met, gates green.** Shipped: event envelope with
unknown-field preservation (top-level and nested after round-1 fix); segmented
NDJSON journal — fsync-per-append, size-based rotation, crash-tail
quarantine+truncate recovery, fail-closed seq-validated replay, advisory
cross-process lock; BLAKE3 write-once blob store with hash-verified reads;
reducer-based projection with atomic snapshots and snapshot+tail catch_up
proven identical to full replay at five cut points. Evidence: `cargo fmt
--check`, `clippy --all-targets -- -D warnings`, `cargo test` (16 tests) all
exit 0, re-run by the orchestrator after the final fix.

**Environmental behavior.** 1 build + 4 critic rounds (cap) + 3 in-loop fix
rounds + 1 adjudicated final fix. Findings per round 21/16/17/14; confirmed
after adversarial refutation 8/8/9/5. 88 agents, ~0.99M subagent tokens; run
interrupted once and resumed from cache with zero loss. Panel value was real:
rounds 1–2 found architectural defects (single-writer by convention only,
torn append corrupting an acknowledged write, self-reporting fsync test,
BLAKE3 never pinned by the blob test, vacuous unknown-fields assertion).
Rounds 3–4 oscillated: invariants demanded fail-closed guards, simplicity
flagged the same guards as beyond-contract machinery — resolved by
adjudication, not iteration.

**Adjudication rulings (cap reached, residuals = 5).**
1. Snapshot identity-divergence machinery removed; kept a one-line fail-closed
   seq bound (`SnapshotBeyondJournal`). R1 removal + R6 guard — satisfies the
   invariants finding (bypassable guard) and the simplicity finding (excess
   machinery) simultaneously, on a lower rung than either critic proposed.
   Accepted consequence → backlog B1.
2. Crash-tail semantics for a newline-less complete-JSON tail pinned by test:
   quarantined (implementation already correct; the claim was untested).
3. Rotation test strengthened (multi-event segments, complete-line endings,
   cross-segment seq order) so rotate-on-every-append cannot pass.
4. `src/lib.rs` restructure ruled authorized → deviation D4.

**Design decisions (builder-reported, rung-tagged, selected).** lib+bin split
(D4, R2 — Cargo-native layout); all five contextual ids `Option` — causation
cannot exist for a first event (contract ambiguity, documented); seq starts at
1 with full validating replay on open — a gapped/corrupt journal refuses to
open for writing (fail closed); quarantine+truncate over truncate-only to
preserve evidence per §20's spirit; fsync observability counter per the
contract's own Unknowns clause; rotate-before-append soft cap (every segment
holds ≥1 complete event); `Reducer` as plain fn pointer, not a trait (R6 —
trait when a second projection demonstrates need); `tempfile` as dev-dep only
(contract-permitted); `BlobRef` validated newtype, put idempotent/write-once,
get re-hashes and fails closed. Builder documented 7 contract ambiguities with
its interpretations — retained in the workflow record; two carry forward:
replay's `first-seq==1` check must learn a lower bound if M5+ introduces
compaction, and blob refs ride in payloads as validated strings by design.

**Shipping gate.** no-mistakes v1.47.0 (source SHA
05e836bb904aef9efcbaf04519144be5c7c3baba), gate agent claude pinned to Sonnet.
Run 01KZGQ8MQM20D4AWK4E3340R58: **passed, zero findings** — review 160s,
test 171s, document 136s, lint clean; push/pr/ci skipped by design. One
pipeline commit adopted via `axi sync --recover` (9f23825: misplaced doc
comment in fsutil.rs). Environmental note: first attempt failed because the
gate agent's `--dangerously-skip-permissions` is refused under root; fixed by
restarting the no-mistakes daemon with `IS_SANDBOX=1` (measured working
before adopting).

---

### M0 — Bootstrap (2026-08-08)

**Mission outcome.** Contract met. Reference corpus committed (proposal;
miztertea/sergeant vendored at `f430cfd`; notes). Crate scaffold per §35 with
the D3 deviation; binary `sgt`; deps limited to the contract list. CI enforces
fmt/clippy/test. Gates green locally (build, fmt, clippy -D warnings, test —
builder output and an independent critic re-run both confirmed). Branch push +
PR completed immediately after this entry; PR number recorded in the next
commit touching this file.

**Environmental behavior.** 1 build iteration, no re-gauntlet needed. Critic
panel (1 Opus critic, combined spec-fidelity+simplicity per contract's
"mechanical" depth): 3 findings → 1 refuted, 2 confirmed. Refutation was
empirical: the refuter deleted the questioned CLI dispatch and the contract's
own clippy gate failed, proving the code load-bearing. Confirmed findings:
(1) unregistered §35 deviation → fixed as D3 above; (2) branch-not-yet-pushed →
resolved by this commit's push. No escalations. Evidence: workflow
wf_144fc9e3-10d (5 agents, 217k tokens).
