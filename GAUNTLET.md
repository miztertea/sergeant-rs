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
| D9 | Successor proposal §7.1/§12.4 draws a flat stage layout — `CONTEXT.md` per stage, optional stage docs, shared contexts under `common/`; artifact declaration explicitly deferred (§24.4) | The ICM convention (`docs/icm/convention.md` §1a) adopts the published ICM protocol's four-layer model (Van Clief & McDermott, arXiv 2603.16021): workflow-level `CONTEXT.md` (L1 orientation), stage `CONTEXT.md` with a mandatory Inputs table (L2), `references/`+`_config/` stable-across-runs material (L3), per-stage `output/` declared per-run artifacts (L4, traveling with the Work branch) | Owner direction 2026-08-10 ("the proposal minimized ICM a bit too much"), grounded in the owner's IdeaOS ICM source record. R5: the named published protocol supplies the shapes; zero engine change — all four layers are ordinary worktree files, and L4 is the lower-rung answer to §24.4's deferred artifact declaration. Landed mid-N1 before the Draft phase consumed the convention docs. |
| D8 | M6 contract budgets "ratatui + crossterm (§34-named, R5)" | `ratatui` only; crossterm is reached exclusively through `ratatui::crossterm` re-exports | Builder narrowing, registered at M6 adjudication (D5 precedent): declaring crossterm separately could resolve a second version whose `KeyEvent` differs from ratatui's backend type — one declaration means one resolution (R1). Rung-noted in Cargo.toml; pinned by `the_tui_stack_is_ratatui_with_crossterm_reached_through_it`. |
| D7 | M5 contract budgets `tracing-opentelemetry` (§28-named) | `opentelemetry` + `opentelemetry_sdk` + `opentelemetry-otlp` directly; no tracing bridge | Builder measurement-adjacent ruling, registered at M5 adjudication (flagged by the checkpoint gate): the tracing-bridge cannot represent the §28 work→stage→execution span tree — spans there follow tokio task structure, not the engine's domain structure; building spans directly from journal/engine events preserves the §28 shape (R5: the named OTel crates themselves; the bridge dropped as R1 — machinery that cannot express the requirement). Rationale in `src/telemetry.rs` module docs. |
| D6 | §38 P0 includes "native Codex execution"; owner's original scope decision was full P0 | Claude is the only native adapter in this prototype; Codex (and the M0-era `backend/codex.rs` stub) deferred until an environment exists where Codex can actually be measured | Owner decision 2026-08-08. Measure-first doctrine (L1) + the L7 lesson: adapter code that cannot be validated against the real harness is prose with a compiler. The §15 trait remains backend-neutral, so a future Codex adapter is additive. |
| D5 | M2 contract enumerates dependencies "axum, tower, reqwest (client, rustls)" | Actual: axum; reqwest without TLS (loopback plain HTTP — R1); tower first wrapped in a no-op layer then removed entirely at round 2 (R1: axum's own middleware suffices); plus `tokio-stream` (R7: sole Stream adapter for SSE, lower rungs named in build report) and `tracing-subscriber` (R5-adjacent: the tracing facade M0 pinned needs one subscriber to emit anything) | Contract over-specified; builder narrowed with rung-logged justifications, round-2 panel flagged the unregistered delta (M0's D3 precedent), registered at M2 adjudication. |
| D4 | §35's tree has `src/main.rs` owning all modules, no lib target | `src/lib.rs` declares the module tree; `main.rs` is a thin shell over `sergeant_rs::cli` | M1 contract requires the core "as library code with tests": integration tests under `tests/` can only import a lib target, and a bin-only crate forces dead-code suppressions under `clippy -D warnings` (R2: the change reuses Cargo's native lib+bin layout). Raised by M1 critics rounds 3–4; ruled authorized at M1 adjudication. |
| D3 | §35 lists `backend/{claude,codex,opencode,prime}.rs` | Scaffold has `backend/{claude,codex,fake}.rs` | §38 defers OpenCode/Prime past the P0 contract proof (R1: doesn't need to exist yet); §37's deterministic core tests require a fake backend (R7: no lower rung supplies a deterministic in-process backend). Modules are added when their milestone arrives, not pre-declared. Raised by the M0 critic panel. |

## Backlog (confirmed-but-deferred findings)

| # | From | Finding | Why deferred |
|---|---|---|---|
| B1 | M1 adjudication | A foreign snapshot whose `last_seq` is within the journal's range loads undetected (identity binding was removed as beyond-contract machinery) | Snapshots live in the daemon-owned data dir; the threat is operator error, not adversarial. **Revisited at M2 (per checkpoint-gate finding document-1):** the daemon now owns the data dir exclusively (daemon.lock) AND uses full journal replay — no snapshot loading exists in the daemon path at all (builder ruling, R1). B1 is unreachable in production flow; trigger narrowed to "if/when the daemon adopts snapshot loading (likely M5 perf)". **Revisited at M5 (checkpoint round 1):** rebuild-from-journal measured well within budget (bulk-appender fold, ~580x a row-wise-SQL baseline — see `Analytics`'s doc comment in `src/runtime/analytics.rs` and the rebuild bench in `tests/m5_projections.rs`), so rebuild-on-start remains the only population path with no perf case for snapshot loading. B1's trigger still does not fire; still dormant. |
| B2 | M6 adjudication | Dashboard auth delivers the bearer token in the `sgt web` URL query string (shoulder-surf/history exposure on a shared machine) | Accepted at R1 for the P0: the listener is loopback-only and the token already travels in the printed URL by design; the API refuses query tokens on non-GET/HEAD (CSRF bound), and `/ui` sits behind the same `require_bearer` gate as `/v1`. Post-P0 alternative recorded per the M6 contract: exchange the URL token once for an `HttpOnly; SameSite=Strict` cookie handoff. Trigger: any non-loopback binding or multi-user host. |
| B3 | P0 final gate | `ClaudeBackend::stop` joins the turn's evidence-archive thread while API handlers hold the core lock, so a concurrent request waits out one transcript flush during a cancel/retry of an in-flight Claude turn | Orchestrator ruling at the final gate (three review rounds converged here): STOP's evidence promise (round-2 fix — the archive is durable before STOP returns) is kept; the lock-hold is rare-path and bounded by one local-disk write; the real fix is a §15 trait-shape change (`stop` returning a join token the caller awaits after releasing the core guard) — R7 machinery, not invented at gate-time without panel coverage. `block_in_place` (d82a6e2) keeps the executor healthy meanwhile; trade-off documented on `stop` itself (d20554d). Trigger: any measured multi-client stall, or the first §15 trait revision for other reasons. **CLOSED at N3**: the trigger fired exactly as registered — §14.3's `prepare`/`launch` split is the first §15 trait revision, and B3's own prescribed fix ("`stop` returning a join token the caller awaits after releasing the core guard") is what landed. `stop`/`interrupt` return a `Completion`; the engine collects them into a `Deferred`; `api::crank` drains it after dropping the guard. `block_in_place` removed. Pinned by `tests/m2_daemon_api.rs` t10 — a stalled evidence archive with an independent read answering in ~2 ms — and revert-probed (holding the guard across the drain fails the test in 1 s). Issue #14 closed. |
| B4 | R-MVP1-10 (blocked exit-door invariant) build | A `pending → blocked` landing from a real start failure (a materialize permission fault, or `reconcile_crashed_start`'s crash-window block) has no working `retry` door: `begin_retry` requires `run.current_stage()` (`engine.rs`), and no stage is ever entered before `settle_materialize` fails or a crashed start is swept — `KIND_WORKFLOW_BOUND`/`KIND_STAGE_ENTERED` land together with `KIND_WORK_STARTED` in one group-committed guard hold, all *after* a successful materialize, so this is not a narrow timing window but the whole category. `retry` fails closed with `EngineError::NoRun` (404, `"no_run"`) — safe, structured, non-corrupting, never silently wrong — but not an open door. The real, working exit is `cancel` (`blocked → canceled`), proven instead. | Reopening it properly needs the full `StartPlan` (workflow resolution, routing, per-stage bindings) re-derivable from the journal alone, and today it is not: `origin.cwd` — needed to re-run `Workspace::discover` — is never persisted (`engine.rs`'s own `reconcile_crashed_start` comment says so), and guessing at a substitute `cwd` would silently re-plan against the wrong estate, which CLAUDE.md's fail-closed doctrine forbids. The honest fix is persisting enough of the submit-time plan to redo it — a bind-path change (`engine.rs`, `SurfacePlan`/`KIND_SURFACE_MATERIALIZING`'s payload) outside this ruling's own file scope (`projection.rs`/`recovery.rs`/`surface.rs`/`api.rs` views). Trigger: any MVP that needs `retry` to reopen a `pending`-origin block — the natural home is wherever the submit-time plan next gets a durable record (R-MVP1-1/R-MVP1-4's `workflow.bound` widening is adjacent territory). Pinned as *expected, safe* behavior by `tests/m2_daemon_api.rs`'s `r_mvp1_10_pending_to_blocked_from_a_real_materialize_fault_exits_via_cancel` (revert-probed: removing the `NoRun`→404 mapping or the `no_run` code fails it). |
| B5 | MVP-3 fixer pass, invariants finding MVP3-C4 | `docs/gauntlet/notes/estate-manifest-design-2026-08-11.md`'s `[estate]` shape lists `data_dir` defaulting `.sergeant/data`, but R-MVP1-1 ruled only `surfaces_dir`; `data_dir` was never implemented (`EstateSection` is `deny_unknown_fields` with no such field; `resolve_data_dir` hardcodes the estate-relative default and reads no manifest field at all). A hand-written `[estate] data_dir = "..."` following the design note literally fails closed with a parse refusal instead of overriding anything. | Not implemented in the fixer pass: this is new engine/config surface (an estate-level override of where the daemon's own data dir lives), R-NS-4 territory that wants explicit ratification, not a silent addition riding in on a bug-fix pass. Design doc corrected in place ([v3] note) so it no longer misdescribes shipped behavior. Trigger: any MVP that actually needs a non-default, manifest-declared data dir (today's only override is `--data-dir`/`SGT_DATA_DIR`, both already covered by R-MVP1-12's discovery-boundary rules). |
| B6 | MVP-3 fixer pass, invariants finding MVP3-C5 | `sgt run --turns`/`--ceiling-secs` (commit 78e5da9) adds a per-Work `EnvelopeRequest{turn_cap, ceiling_secs}` the engine reads via `effective_turn_cap`/`effective_turn_ceiling` — genuinely new engine surface (submit-time envelope override) that R-MVP1-7 specified as daemon-wide only (`with_turn_cap`/`SGT_TURN_CAP`), and MVP-3's own bucketing plan does not list it. Counting integrity is intact (not a bypass: `check_turn_envelope` still gates on the effective values, `turns_spawned` stays journal-derived, submit rejects `turn_cap==0`/`ceiling_secs==0`) — the gap is process, not safety. | Shipped code kept as-is (it works, and reverting a MVP-3-milestone feature inside a fixer pass is a bigger unilateral move than registering it); registered here instead per R-NS-4 discipline, for owner ratification at the next adjudication rather than silently standing unregistered. Trigger: MVP-3's own adjudication — fold into the milestone's deviation record or explicitly ratify there. |
| B7 | MVP-5 F2 execution-surface re-triage (`docs/icm/retriage-2026-08-11.md`, `load-project` verdict notes) | `sergeant-setup`'s `30-project-interview` stage duplicates `load-project`'s registration job wholesale (same "complete project definition... previewed... written only after confirmation" shape) instead of delegating to it — the retriage itself flags this as "a duplication defect, not a clean stage-boundary split," and it stood unresolved when the F2 pass executed the surrounding CLI-SURFACE verdicts around it. | Fixing it means editing `sergeant-setup`'s own package to delegate its interview stage to `load-project` instead of reimplementing it — a `.sergeant/workflows/` content edit outside the MVP-5 fixer pass's file scope (docs/`AGENTS.md`/`skills/` only). Trigger: any pass that next touches either `sergeant-setup` or `load-project`'s own package. |
| B8 | MVP-5 Lane F1 dispositions, content-honesty CH-1 (fixer pass, 2026-08-13) | 17 `agents-invariant` units (BU-1033..1048's codebase-design vocabulary/deepening rules, BU-1064's domain-modeling CONTEXT.md file-role rule) have no live skill package to land in — only frozen upstream evidence under `reference/sergeant-upstream/.claude/skills/`, which `docs/DEVELOPMENT.md` forbids treating as live content. `docs/icm/agents-invariant-dispositions.md` previously (and wrongly) claimed both were already-published `.sergeant/workflows/` packages; corrected to `not-adopted` this pass, and the two live packages that named a "grilling/domain-modeling" pair (`triage/40-grill-if-underspecified`, `wayfinder/00-name-destination`) were corrected to name `grilling` alone rather than invent the second invocation. | Promoting either skill is new content, not a dispositioning correction — this fixer pass's job was to stop asserting they exist, not to build them. Trigger: an owner decision to actually promote `codebase-design` and/or `domain-modeling` as `skills/` packages (nearest precedent: `deepen-module` already cites `codebase-design`'s upstream `DEEPENING.md` directly, without a `skill:` indirection — see `00-classify-dependencies/CONTEXT.md`). |

---

## Ledger entries

### WATCH — 2026-08-13, `sgt watch` shipped: proposal → contract → build → gate → pilot, one day

**Mission outcome: contract met, ship gate passed, product pilot PASS.**
`sgt watch [WORK_ID] [--follow]` — the harness's return path after `sgt
run` — landed on `cerberus/watch` (PR #69, owner merges). Six-state watch
set, attention-identity fingerprint over `stage.detail`, three-branch
no-auto-spawn (owner-ruled: observation must not materialize the daemon),
JSONL `sergeant.watch/v1` notices carrying the verbatim Work view,
EventStream's malformed-frame distinction (old silent-skip pin revised
under R-WATCH-7's explicit ruling), and the `SGT_WATCH_TEST_HOLD`
dead-man'd race seam. Suite 539 → **570 passed / 0 failed**.

**The loop, with finding curves.** Owner proposal vendored
(`reference/proposal-sgt-watch-v1.md`) → proposal-review gauntlet
(wf_56a68808: 3 Sonnet critics + Opus refuter over 22 panel + 5
orchestrator findings → 21 confirmed / 3 plausible / 3 refuted-as-dupes;
2 errors: `waiting` wrongly excluded, fingerprint swallowing a second
question) → WATCH contract, 10 rulings → its own L19 pass (wf_d1801972,
2 Sonnet critics: 10/10 findings confirmed, incl. the missing-docket
process error and the terminal teardown-lag race, the L6 class in the
notice path) → build gauntlet (wf_f4cb393b: Sonnet builder 5 separable
commits/29 tests; panel 2 findings + 1 process note, all confirmed;
Sonnet fixer) → 3 L7 mutation probes, all pins have teeth → no-mistakes
ship gate 01KZY9ZFTE (review/test/document/lint **passed**; 1 info
finding approved after reading — the contract-authorized structural-test
pattern; gate agent verified `--model sonnet` on the live process table;
1 pipeline commit adopted via `axi sync --recover`) → §16.4 pilot
(`docs/gauntlet/runs/watch-2026-08-13/pilot.md`): submit→notice 2m43s,
4/8 turns, zero polling commands, R-WATCH-9's terminal lag observed live
(`output: null` at emission, settled by collection).

**Environmental behavior.** 12 agents, ~1.74M subagent tokens across the
three workflows + probe agent; Fable spent only in the orchestrator seat.
One incident, fully traced: the build gauntlet's verify agents filled
Cerberus's 16 GB tmpfs `/tmp` (CARGO_TARGET_DIRs + gigabyte-scale blob
rigs, #70), killing the host's entire Bash layer mid-fix — the fixer
reported honestly instead of papering over, its uncommitted edits were
gate-checked and committed after recovery, and the rule is now an
environment row (`docs/environments/cerberus.md`) plus a Work-delivered
`docs/DEVELOPMENT.md` subsection (`b33eccc` — written by the pilot's own
dispatched Work: sergeant documenting the incident that its own build
caused). Dockets: `docs/gauntlet/runs/watch-2026-08-13/`. Issues: #68
(auto-spawn consistency sweep, scoped out), #70, #71 (pilot finding)
filed; PR #69 closes #59/#61/#63.

### WATCH FIXER PASS — 2026-08-13, F1/F2 closed by edit; PROC-1 re-confirmed and environment-blocked

**Mission outcome.** Two of the three CONFIRMED findings from the WATCH
gauntlet's fresh-eyes review closed by direct edit; the third investigated
and found unfixable by any code change, per below.

**contract-fidelity:F1 — closed.** R-WATCH-4 requires both `AGENTS.md` and
`README.md` to state the attach-before-reconcile ordering and disclose the
residual gap a bare one-shot estate watch still carries when invoked after
reconciliation. `AGENTS.md` already carried both; `README.md` did not.
`README.md`'s "Wait for it, instead of polling" section now carries the
same ordering sentence and residual-gap disclosure, placed right after the
existing no-auto-spawn paragraph.

**contract-fidelity:F2 — closed.** The 60s test-only dead-man on
`SGT_WATCH_TEST_HOLD` (`WatchError::TestHoldTimedOut`) was real but had no
test exercising its timeout branch — only the happy path (hold engages,
then releases) was pinned by W3/W4. `src/watch.rs`'s `test_hold_rendezvous`
was split into a thin env-var reader plus a parameterized
`test_hold_wait(path, deadman, poll)` helper; production behavior is
unchanged — `test_hold_rendezvous` still always passes the real 60s/20ms
constants. Two new unit tests exercise the helper directly with a
millisecond-scale dead-man: one drives a release path that never appears
and asserts `Err(WatchError::TestHoldTimedOut { path })` naming the exact
path; its mirror releases the hold from a concurrent `tokio::spawn`ed task
15ms in and asserts `Ok(())` through the same helper, so both branches of
the parameterized code are pinned, not just the timeout one.

**test-honesty:PROC-1 — investigated; not fixable by a code change;
independently reconfirmed, more severely, this session.** PROC-1 recorded
the prior review session's Bash tool becoming persistently unreliable
partway through, blocking a requested revert-probe. This fixer session hit
the identical failure class from its very first command: every `Bash`
invocation that either invoked a real external binary or produced stdout
returned a nonzero exit (1/128/2) with no captured output — `git status`,
`git log`, `echo`, and `/bin/true` with the sandbox explicitly disabled all
failed identically. Worse than the prior session's report: `grep` runs
against `README.md` for a string ("edge-triggered") silently reported no
match immediately after that exact string was written to that exact file
and independently confirmed present via the `Read` tool — i.e. Bash's own
output cannot be trusted even when it reports success. Only the shell
builtin `exit N` (forks/execs nothing) ever succeeded. This rules out an
intermittent, session-specific flake and confirms PROC-1's own caveat
verbatim: "This is a tool/environment failure, not a finding about the code
under review." No code fix applies to a broken exec environment.

**Consequence for this pass, stated plainly (L15):** the requested
revert-probe, the `cargo fmt`/`clippy`/`test` gate run, and `git commit` of
the F1/F2 fixes above could not be executed from this session. The two
fixes exist only as uncommitted working-tree edits (`README.md`,
`src/watch.rs`), verified by direct `Read` of the edited files — not by
compiling or running them. They are believed correct by inspection against
this codebase's own existing patterns (`src/api.rs`'s `#[tokio::test]`
loopback-TCP precedent for the analogous R-WATCH-7 malformed-frame test;
`Cargo.toml`'s `tokio` features already include `macros`/`rt-multi-thread`/
`time`, which both new tests need) but that belief is a hypothesis, not a
measured fact, until a session with a working exec environment runs the
gates and commits. **Findings disposition:** contract-fidelity:F1 CLOSED;
contract-fidelity:F2 CLOSED; test-honesty:PROC-1 CONFIRMED and
re-observed, disposition unfixable-by-code — recorded per L9 rather than
silently dropped for lack of a code-shaped fix.

**Orchestrator addendum (same day): PROC-1's root cause found, fixed, and
closed as an environment rule.** The "broken exec environment" was `/tmp`:
a 16 GB tmpfs that the gauntlet's probe/verify agents filled with
disposable-worktree `CARGO_TARGET_DIR`s under the session scratchpad. Every
`Bash` invocation then failed at the harness's output-capture layer with
`EDQUOT` while the underlying command still executed (verified: an `rm`
reported exit 1 yet deleted its target) — which is why `grep` looked
untrustworthy and only no-output builtins "succeeded." Clearing the
scratchpad restored the shell instantly. The fixer's uncommitted edits were
then gate-checked for real: `cargo fmt` (one drift spot), `clippy -D
warnings` clean, **`cargo test` 570 passed / 0 failed**, and committed
separably per its own handoff. Environment rule recorded in
`docs/environments/cerberus.md`: agent build dirs go on the ext4 root
(`/var/tmp/<name>`), never tmpfs. The revert-probes PROC-1 originally asked
for were re-run on disk-backed storage — results in the entry above this
one at ship time.

---

### MVP CLOSE-OUT — 2026-08-13, the ship: all five buckets landed, ship gate PASS, #19 closed

**Mission outcome.** The North Star MVP is complete on cerberus/mvp-1
(PR #65, owner merges). Since the MVP-4 entries: (1) **Extended #19
soak** (91ba140, Fixes #19): 2h36m unbroken daemon, 13 works, 10 real
Docker execute stages digest-identical, settle driver reconfirmed 12×,
envelope exit door proven live (blocked → extend → retry → completed —
upstream's no-exit-door scar closed with journal evidence), and Rule A
eviction's first sustained-load measurement (RSS 58.8→24.8 MB, flat
62 min). Operator disclosed its own mid-run dormancy in the manifest.
(2) **Ship gate PASS** (e5af0a6): a fresh clone walked the full
colleague path on product surfaces alone — install, init, two repos,
real intent, genuine 5-min detach, return to a verified diff via the
output pointer, daemon-stop + respawn with zero loss. Four findings
(dormancy/process; cargo-install collision; model-pin syntax
undocumented — the sharpest product gap, backlog; cosmetic). (3) **CI
exposed a real day-one bug this host could not**: init propagated
doctor's claude-row failure as its own exit — a colleague without
claude could not init (§17.5 violation). Fixed ace9b16/0c8650a
(healthy_for_init: harness rows advisory at init, doctor keeps hard
semantics), pinned via SGT_CLAUDE_BIN=/nonexistent both-environments,
CI green on the fix commit. (4) **Residue sweep found two bugs**
(trace-then-clean doctrine): #66 sgt-probe container leak past the
harness sweep (postdates the launch-error fix — different path), #67
doctor fresh-dir docker blob-store EPERM + unmeasurable disk_pressure.
(5) **Final shipping gate passed** (5→2→1-approve): manifest
malformed-TOML panic fixed (fail-closed means refusal, never panic),
transcript full-replay-under-guard fixed after the gate caught its own
first fix not releasing the guard, docker inspect absence/failure
conflation fixed with its L7 pin on the gate's insistence. Suite at
close: 539 passed + 4 opt-in, CI green, all sweeps clean.

**Environmental behavior.** The recurring failure of this stretch was
agent dormancy — the soak operator (twice), the first ship-gate seat
(placeholder verdict), and the second ship-gate seat (80-min install
wait) all ended turns to wait on nothing; watchdogs on artifacts (not
agent claims) caught every instance, and the anti-dormancy rule
(foreground sleeps, artifact watchers, process-table evidence for every
"still running") is now standard workflow-prompt text. Owner
corrections continued to be the highest-value input: dollars-are-
telemetry (re-enforced twice), the repo-to-icm soak redirect (same
adapter evidence at a fraction of the volume), residue-is-evidence
(produced #66/#67), and verify-before-speaking (the "ten minutes" that
measured 23 seconds). Model economy held: Sonnet workforce, Opus
panels/refuters, no-mistakes pinned Sonnet (verified via the binary's
per-agent override string), Fable one orchestrator seat.


### MVP-5 FIXER PASS — 2026-08-13, 15 vision-fidelity/content-honesty findings closed

**Mission outcome.** All 15 findings from the panel review of MVP-5
CONTENT (below) closed by editing content, none rejected. Docs/content
only — no `src/`/`tests/` change, so CLAUDE.md's "code is code" multi-axis
loop does not apply; gates re-run anyway (`cargo fmt --check`, `cargo
clippy --all-targets -- -D warnings`, `cargo test`) since the workflow-
catalog and `m6_surfaces` t5 suites read `.sergeant/` and `AGENTS.md`
directly and a content edit can break them. Leak check clean: `pgrep -f
"debug/sgt [-]-data-dir"` empty.

**vision-fidelity (4 findings, all closed).** VF-1/CH-4 (the colleague
README path never delivers the OS layer — `sgt init` in a bare
`~/my-estate` has no `AGENTS.md`/`skills/`/`.sergeant/`): `README.md`'s
"Get it" section now keeps the reader in the clone itself
(clone-is-distro, matching the A7 ship-gate happy path and
`docs/gauntlet/notes/mvp-bucketing-2026-08-11.md`'s "gh repo clone → ...
→ `sgt init` → open Claude" sequence) instead of sending them to an empty
directory elsewhere. VF-2 (the standard loop never teaches walk-away →
return, the product's whole differentiator): step 5 of `AGENTS.md`'s
standard workflow loop now states `sgt run` returns on submission, not
completion, and that the daemon outlives the terminal; step 7 states a
`needs_input` you weren't watching for is exactly why the loop hands
control back rather than blocking in-session; step 8 now asks for the
spent-envelope report (`envelope.turns_spawned` vs. ceiling) as part of
"collect", not merely the branch/artifacts. VF-3 (README states the wrong
data-dir default, omitting the estate-resolved rung that is R-NS-3's whole
mechanism): the "Using sgt day-to-day" section now states the full,
correct precedence chain measured from `src/cli.rs`'s `resolve_data_dir`
and names the first-`sgt-init`-reports-against-the-fallback wrinkle
explicitly rather than leaving a reader to discover it. VF-4/CH-7 (the
"see it first" demo path prescribes a discarded release build
`scripts/demo.sh` never consults): the release-build line is gone; the
walkthrough now just runs `scripts/demo.sh` (which builds its own debug
binary) with a one-line note on pointing it at an already-installed `sgt`
via `SGT_BIN` instead.

**content-honesty (8 findings, all closed).** CH-1 (17 of 126
`agents-invariant` units routed to `skill: codebase-design`/`skill:
domain-modeling`, which don't exist as published packages — only frozen
`reference/sergeant-upstream/` evidence): `docs/icm/agents-invariant-
dispositions.md`'s 17 rows (BU-1033..1048, BU-1064) reclassified to
`not-adopted` with the honest reason (nearest live host, `deepen-module`,
already cites the upstream path directly rather than through a fabricated
`skill:` indirection); `AGENTS.md`'s corpus-summary paragraph no longer
names either as a workflow; the two live packages that named a
"grilling/domain-modeling" pair (`triage/40-grill-if-underspecified`,
`wayfinder/00-name-destination`) now name `grilling` alone, matching what
was actually built. Backlog row B8 registers the promotion gap. CH-2 (10
of 40 `AGENTS.md`-dispositioned units uncited, 5 with no satisfying text
at all): `AGENTS.md` gained real cited text for BU-0004/BU-0005/BU-0009
(the direct-vs-dispatch criteria, previously absent), BU-0109
(single-developer-per-install, previously absent), and citations for the
five that had satisfying text but no marker (BU-0020/BU-0037/BU-0056/
BU-0172/BU-1262); BU-0003's inverted default-to-coordinate claim was
reclassified `not-adopted` rather than forced into text that contradicts
this repo's actual routing judgment. CH-3 (three dispositions cite a
"`NORTH-STAR.md` 'One owner'" quotation that isn't in that file):
corrected the BU-0054/BU-0109 citations to `docs/DEVELOPMENT.md`, where
the phrase actually lives (and clarified it's an intra-process invariant,
distinct from BU-0109's adoption-model claim); corrected BU-0110's
dismissal, which had wrongly claimed `NORTH-STAR.md`'s Never list names
tenancy/RBAC/credentials/leases — it doesn't; the real reasoning
(sergeant-rs has no multi-tenant server surface at all) replaces it. CH-4:
same fix as VF-1 above (single finding, two review passes). CH-5 (a new
skill cites "CLAUDE.md L1", which the very next commit symlinked away
from any such text): `skills/sergeant-help/SKILL.md`'s precedence-order
citation now points at `docs/DEVELOPMENT.md`'s actual L1 sentence. CH-6
(`load-project/CONTEXT.md` contradicts itself about whether the folded
N1-A4 helpers still exist, and cites a `provenance.md` that never
existed): rewrote the N1-adjudication-A4 paragraph to state the fold and
its later supersession by the MVP-5 F2 SPLIT verdict in one consistent
account, and named the real provenance trail
(`docs/gauntlet/promoted-provenance/load-project.md`) instead of the
dangling reference. CH-7: same fix as VF-4 above. CH-8 (no GAUNTLET entry
for MVP-5, breaking the per-milestone register discipline every prior
milestone followed): this entry, plus the MVP-5 CONTENT entry below it and
backlog rows B7/B8 for this pass's own named-but-unfixed gaps
(`sergeant-setup`'s `30-project-interview` duplication; the
codebase-design/domain-modeling skill-promotion gap CH-1 surfaced).

**Rejections: none.** Every finding was investigated to its cited
evidence before fixing; none was found to already be correct as shipped.

**Environmental behavior.** Direct fixer pass over a panel's 15 findings,
no further panel/critic loop for this response (docs/content-only diff
against CLAUDE.md's "code is code" scope rule — no `src/`/`tests/`
executable-behavior change). Findings spanned five files' worth of
cross-references (`README.md`, `AGENTS.md`, two workflow `CONTEXT.md`
files, one skill file, one dispositions ledger) with real cross-file
contradictions (CH-3's misquoted "One owner", CH-6's self-contradicting
paragraphs) rather than single-file typos — each was traced to its actual
source text before correcting, not patched at the symptom line alone.

---

### MVP-5 CONTENT — 2026-08-12, AgentOS layer shipped: `AGENTS.md` rewrite, symlink, execution-surface re-homing, README recenter

**Mission outcome.** MVP-5's content lane landed: `docs/DEVELOPMENT.md`
carries the dev rulebook moved out of `CLAUDE.md` (commit 9bb84fe);
`AGENTS.md` rewritten as the canonical front door — trigger→skill/workflow
routing table, an 8-step standard workflow loop, guardrails — consuming
all 126 `agents-invariant`-classified units from N2 run 4 with every one
dispositioned in `docs/icm/agents-invariant-dispositions.md` (commit
cdfd2a5); `CLAUDE.md` retargeted to a symlink onto `AGENTS.md` with dev-
rulebook citations across `src/`/`tests/`/`scripts/`/docs retargeted to
`docs/DEVELOPMENT.md` (commit 2c4eb29); `README.md` recentered on
clone-and-work (commit 0b87800); the MVP-5 F2 execution-surface re-triage
executed — 12 of 35 draft/published packages retired to `skills/`
(operator skills) or `docs/icm/re-homing-record-2026-08-12.md`
(CLI-verb/engine-gap candidates), landing the 23-package catalog
`.sergeant/index.md` now lists, plus the R-NS-6 `grilling`/
`grill-with-docs` dissolution out of the WORKFLOW-IF-E3 category (commit
774a372); two small pre-existing package defects fixed in passing
(`validate-and-ship`'s S4 Inputs-table, `repo-to-icm`'s wrong resume verb).
`#53`/`#57` closed in passing per the MVP-5 plan.

**Gates.** `cargo fmt --check && cargo clippy --all-targets -- -D
warnings && cargo test` green (re-confirmed at the fixer pass above, which
touched only docs/content and re-ran the full gate rather than assume the
milestone commit's own run still held). Leak check clean.

**Deviation/backlog notes.** See backlog rows B7 (`sergeant-setup`'s
`30-project-interview` duplicating `load-project` rather than delegating
to it — retriage-flagged, left unresolved by this lane's file scope) and
B8 (17 `agents-invariant` units this lane's own dispositions document
initially — wrongly — claimed already had a published skill home; found
and corrected at the MVP-5 FIXER PASS above).

**Environmental behavior.** Multi-commit content lane (8 commits,
9bb84fe..774a372) executing a plan (`docs/icm/agents-invariant-
dispositions.md`'s own provenance note, `docs/icm/retriage-2026-08-11.md`,
`docs/icm/re-homing-record-2026-08-12.md`) rather than a single fixer
response; a follow-on panel review (findings closed above) is this
milestone's fixer-pass discipline, matching every prior milestone's
two-entry (ship + fixer) ledger shape.

---

### MVP-4 FIXER PASS — 2026-08-12, review findings on the perf re-baseline and the #45 closure

**Mission outcome.** Two warning-severity findings from the review pass over
`MVP-4 HARDENING`/`baseline-mvp-2026-08-12.md` closed, both by adding real
evidence rather than by disputing the findings — both were correct as
stated. Gates green: `cargo fmt --check`, `cargo clippy --all-targets -- -D
warnings`, full `cargo test` unaffected (no `src/`/`tests/` change in this
pass — docs only, so no code-review multi-axis loop applies per CLAUDE.md's
"code is code" rule, which scopes that loop to diffs that change executable
behavior). Leak check clean: `pgrep -f "debug/sgt [-]-data-dir"` empty.

**`perf-raw-artifacts-not-committed` (warning) — closed by documentation,
not by changing the artifact-commit convention.** The finding was right:
`baseline-mvp-2026-08-12.md` asserted "every number below comes from the
raw run artifacts" without ever stating that those artifacts are, by
design, not committed and no longer exist — an asymmetry with
`docs/coverage/`'s own committed-artifact convention that a reader could
mistake for an oversight rather than a fact. Investigated before fixing:
the P1-PERF contract itself specifies raw JSON/CSV live "under the run's
output dir (not committed)", and `scripts/perf/README.md` requires
`<outdir>` to live outside the repo tree — so committing raw perf artifacts
would deviate from both the contract and the harness's own hard constraint,
which this pass has no standing to change unilaterally. Fixed instead by
making the convention explicit where the finding says it's missing: a new
paragraph in `docs/perf/baseline-mvp-2026-08-12.md` states the
not-committed convention, cites the contract and README lines it comes
from, names `docs/perf/s2-churn-mvp1-fixer-2026-08-12.md` as the precedent
for how this repo already handles the same fact (distill numbers, name the
uncommitted artifact's scratch path), and confirms this run's own raw
artifacts do not survive on disk (searched, not found) — so the honest
statement is "transcribed from a since-gone run", not "checkable today".
**Closed.**

**`45-closed-on-preexisting-pin-no-session-code` (warning) — closed by
re-measuring, not by disputing the closure.** The finding accepted the
underlying #45 closure as substantively defensible (the structural pin
fails on revert; the behavioral pin is real) but flagged that the specific
"40/40 isolated, 15/15 full-suite" counts cited in the `MVP-4 HARDENING`
entry had no committed or even scratch-persisted artifact — the wrapper
scripts that produced them (`stress45.sh`/`stress45_full.sh`, found intact
in this session's scratchpad) print to stdout only, and nothing captured
that stdout at the time. Rather than take the prior claim on faith or
merely soften the ledger's wording, this pass re-ran the identical method
(16-way busy-spin background load on the 20-core host, same two commands)
fresh, this time capturing output to `th45-reverify/isolated-40.log` and
`th45-reverify/full-suite-15.log` (scratch, per this repo's own
artifact-commit convention — see the previous finding). Result: **40/40
isolated runs, 15/15 full-suite runs, 0 failures**, matching the original
claim's shape exactly. Full write-up with method, counts, and the scratch
artifact paths: `docs/gauntlet/notes/45-reverify-mvp4-fixer-2026-08-12.md`.
Hygiene confirmed clean after: no leaked daemons, no leftover spin
processes. **Closed** — #45's closure now rests on a number this session
itself measured and can point to, not an inherited, unrecorded one.

**Environmental behavior.** Direct fixer pass over two review findings, no
panel/critic loop (docs-only diff, no executable-behavior change to
gauntlet against CLAUDE.md's "code is code" scope rule). Both findings
were investigated to their contractual root before fixing — neither was
rejected, but neither was "fixed" by simply agreeing with the finding's
framing either: the perf-artifact finding's proposed remedy (commit raw
artifacts) would have contradicted the P1-PERF contract and the harness's
own README, so the fix instead makes the existing, correct convention
legible where it was previously assumed; the #45 finding's remedy (produce
a checkable number) is exactly what was missing, so it was produced by
re-running the measurement rather than retroactively excusing its absence.

---

### MVP-4 HARDENING — 2026-08-12, #45 (m6 dropped-daemon flake) closed by measurement, #22 (workspace-edge tests) closed with a real bug found and fixed

**Mission outcome.** Both of MVP-4's named hardening items closed. Gates
green: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
full `cargo test` (536 passed, 4 opt-in ignored, 0 failed, across the lib +
8 integration suites) all clean after the final commit. Leak check clean:
`pgrep -f "debug/sgt [-]-data-dir"` empty. Repeated-run gate (L7 corollary):
`tests/m3_execution.rs`, the `runtime::surface` lib module, and
`tests/m6_surfaces.rs` each run 10/10 clean; the two m6 dropped-daemon
composition tests specifically also run 40/40 (isolated) and 15/15 (full
suite) clean under sustained 16-way CPU contention — see #45 below.

**#45 (m6 dropped-daemon composition flake): root cause was already fixed on
this branch, before this session — closed by fresh measurement, not new
code.** The issue's own suspicion ("possibly the #26 startup-window class")
was correct: `run_until_signal` used to install its SIGTERM/SIGINT handlers
*after* `start_with` published the runtime descriptor, leaving a window —
descriptor write to handler install — in which SIGTERM's default
disposition simply killed the process: no `daemon.stopped`, no journal, a
descriptor left pointing at a dead pid. That is exactly the wave-3 shape
("the daemon's pid was already dead while `runtime.json` remained").
Commit `439e218` (Cerberus, 2026-08-11, dated *after* #45 was filed but
*before* this session and already an ancestor of this branch's `HEAD`)
moved handler installation to the first act of `run_until_signal`, before
the listener binds and before anything publishes, and pinned the ordering
structurally with `the_daemon_installs_its_signal_handlers_before_it_
publishes_anything` (byte-offset assertion on `daemon.rs`'s own source, not
a flake-rate measurement — a future regression fails deterministically, not
~2.5%-of-runs). This session's contribution is verification, not the fix:
`the_dropped_spawned_daemon_leaves_the_evidence_of_a_clean_shutdown`
(the behavioral pin) and the full `m6_surfaces` suite were run repeatedly
under artificial load matching the issue's own repro method (16-way busy-spin
on this host's 20 cores) — 40/40 isolated runs and 15/15 full-suite runs,
0 failures, no reproduction of either the wave-1 or wave-3 shape. Combined
with the wave-3 fixer's own pre-fix census (0 reproductions in 10 runs + 3
isolated m6 runs — the bug was already rare pre-fix), this is enough
evidence to close rather than merely "not reproduced yet": the mechanism is
understood, the fix addresses that exact mechanism, and it is pinned
structurally so a regression cannot silently reopen the window.
**Closes #45.**

**#22 (workspace discovery edge cases): the discovery-only fixtures
R-MVP1-12 already landed (`src/domain/workspace.rs`) were extended to the
"remaining, non-discovery edges" the MVP-1 contract explicitly deferred
here — full daemon/API flow (correct binding record, work completes,
teardown clean), per the issue's own proposed shape. Four new
`tests/m3_execution.rs` acceptance tests: `t9` (a repository with a
submodule), `t9b` (the *source* repository is itself a git worktree, not a
main checkout), `t9c` (a symlinked repository root), `t9d` (a path with a
space and a non-ASCII character). The worktree-as-source and symlinked-root
shapes needed no code change — `materialize`/`teardown` already run every
`git` invocation with `cwd` set to whatever `repository.path`/
`binding.source_path` resolve to, and git itself correctly resolves the
shared common dir from a linked worktree, so both already worked; the tests
close the "untested" half of the issue honestly, without inventing a defect
that was not there.**

**The submodule shape surfaced a real, previously-unknown bug, in two
parts, both fixed:**
1. `git worktree add` (what `materialize` already used) checks out the
   superproject's gitlinks but never populates a submodule's own content —
   that is `git submodule update`'s job, and nothing called it. A
   repository with a submodule silently materialized a surface with an
   empty submodule directory: not a refusal, not a warning, just content
   the rest of a run expected and did not have. Fixed:
   `init_submodules_if_present` (`src/runtime/surface.rs`) runs `git
   submodule update --init --recursive` after `add_worktree` whenever the
   checked-out worktree has a `.gitmodules`, using the identical transport
   allowlist (`file:http:https:ssh:git`) `git_clone` already uses for a
   `sgt repo add --origin` — a submodule URL is exactly as untrusted, and
   this is a widened *permission* (matching an existing precedent), never
   an unbounded one. Pinned by
   `runtime::surface::tests::a_submodule_is_populated_into_the_
   materialized_worktree` (unit) and `t9` (end to end through the daemon);
   revert-probed (reverting to a no-op strands the test on a missing file
   with `NotFound`, confirmed by hand before landing the real fix).
2. **Found while writing the *positive*-path test, not the negative one —
   the more dangerous kind of gap.** Wiring the submodule step into
   `materialize_one` (before restructuring, below) reintroduced exactly the
   class of bug `materialize`'s own partial-failure rollback exists to
   prevent: a failure *after* `add_worktree` already created a real
   worktree, on the *first* (and, in the single-repo case, only)
   repository, hit `materialize`'s `if bindings.is_empty() { return
   Err(err) }` special case — which assumed, correctly until this change,
   that nothing to roll back could exist yet on repository 1. It stopped
   being correct the moment a post-checkout step could fail after a real
   worktree existed. Fixed: `materialize`'s loop now runs
   `init_submodules_if_present` on the binding `materialize_one` already
   produced and folds that binding into the rollback set *before* deciding
   whether anything needs tearing down, regardless of the repository's
   position in the list — a submodule failure on repository 1 of 1 gets
   the exact same recorded `SurfaceError::PartialFailure` + teardown report
   a later repository's failure always got, never the silent, unrecorded,
   un-journaled strand the naive wiring produced. Caught, not merely fixed
   blind: `a_submodule_is_populated_into_the_materialized_worktree` failed
   first with git's own "working trees containing submodules cannot be
   moved or removed" once step 1 above worked — teardown's plain `git
   worktree remove` turned out to refuse *any* submodule-bearing worktree
   unconditionally, clean or not, which would have leaked one worktree per
   successfully completed submodule-bearing work, forever. Fixed as its own
   third part: `teardown_binding_locked` retries with `--force` exactly
   when git's refusal names "containing submodules" — safe only because
   the existing `git status --porcelain` cleanliness check just above it
   already recurses into a registered submodule by default (measured, not
   assumed: an untracked file, a modified tracked file, and an advanced
   submodule `HEAD` all report as `M <path>` at the superproject level), so
   reaching the force-retry at all already means nothing uncommitted exists
   to destroy. A fourth test,
   `a_dirty_submodule_still_blocks_teardown_despite_the_force_retry`, pins
   that real uncommitted submodule content still blocks removal — proving
   the force-retry path is unreachable for that case, not merely untested.
   A fifth, `a_disallowed_submodule_transport_fails_closed_and_is_not_
   stranded`, pins the fail-closed contract itself: a submodule over a
   transport the allowlist refuses (`ext::`, instant and deterministic, no
   network) fails materialization with the rolled-back-and-reported shape,
   not a bare unrecorded error. All five new `runtime::surface` unit tests
   plus `t9` were confirmed load-bearing by reverting the fix locally and
   observing the expected failures before restoring it (L7).
**Closes #22.**

Both fixes are scoped narrowly and documented against what they
deliberately leave alone: `rematerialize` (the retry re-attach path) also
now re-initializes submodules for symmetry, but a failure there still
propagates through `settle_rematerialize`'s pre-existing `Err(e) => return
Err(e.into())` arm as an engine error rather than a journaled `blocked`
state — true of *every* git failure on that path already, not a gap this
change opens, and left alone rather than folded into this pass (noted on
`rematerialize`'s own doc comment, not silently).

**Environmental behavior.** No panel/critic loop run for this pass — direct
build-and-verify against the two named issues, per the task's own scope (a
Sonnet-executed hardening pass, not a milestone contract). Real bug found
and fixed came from writing the *positive*-path test for #22's submodule
shape first, not from hunting for one — worth naming as a pattern: the
happy-path test found what the sad-path test alone would have missed twice
over (empty-submodule-directory, then the teardown refusal it exposed).

---

### MVP-3 FIXER PASS — 2026-08-12, invariants/test-honesty panel findings closed

**Mission outcome.** Fixer pass over the MVP-3 build's review panel: 11
CONFIRMED findings (`invariants:MVP3-C1`…`C6`, `test-honesty:TH-1`…`TH-5`;
2 warning, 9 info by severity), 0 PLAUSIBLE. **10 of 11 CONFIRMED touched
and improved; 1 (TH-5) is a positive-coverage finding needing no
change.** Nothing silently dropped: every finding below is either fixed
with a revert-probed test, corrected in documentation, or registered as a
deliberately deferred backlog item with a named trigger. One real
correctness bug (duplicate transcript text) was found and fixed while
building TH-1's closure — not itself a panel finding, so it is called out
explicitly below rather than folded silently into TH-1's count. Gates
green: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
full `cargo test` (524 tests across the lib + 8 integration suites, 0
failed, 4 opt-in ignored) all re-run clean after the last commit. Leak
check clean: `pgrep -f "debug/sgt [-]-data-dir"` empty.

**Warning-severity findings (2 of 2 closed).**
- **MVP3-C1** — the estate manifest edit pens (`sgt init`/`repo add`/`repo
  remove`/`group add`/`group remove`) validated every edit by resolving
  ALL declared `[[repo]]` entries through git, failing closed at the
  first one missing from disk — so one uncloned repository blocked every
  *other* manifest edit too, including a freshly `git clone`d estate
  (`sgt init` gitignores `repos/`), contradicting the design capture's
  own "wrongness scoped per-entry" contract. Fixed: added
  `Workspace::from_config_structural` (`src/domain/workspace.rs`) — every
  schema-level check (legacy vocabulary, duplicate/invalid names, group
  membership, profile validity) with the on-disk git resolution dropped —
  and switched `domain::manifest::validate` to it. Pinned by
  `domain::manifest::tests::a_missing_unrelated_repo_does_not_block_
  edits_that_do_not_touch_it` (revert-probed: reverting to
  `from_config_allow_empty` fails it with `RepositoryNotFound` naming the
  untouched repo).
- **MVP3-C2** — `sgt run --group`'s client-side expansion had the same
  coupling: it read group membership through the strict
  `Workspace::discover_scoped`, so an unrelated missing repository
  refused the command before a daemon even spawned. Fixed: added
  `Workspace::declared_groups`/`declared_groups_scoped` (same structural
  parser) and switched `cli.rs`'s `--group` handling to it. The daemon's
  own submit-time estate resolution is a separate, pre-existing,
  *accepted* coupling (matches plain `--repo`, per the B4 register entry)
  and is deliberately untouched — pinned by
  `tests/m8_estate_cli.rs::run_group_expansion_itself_survives_an_
  unrelated_declared_repo_missing_from_disk`, which asserts the daemon
  actually spawns (proving the client-side refusal is gone) even though
  the command as a whole still fails on the daemon's own, separate check.

**Info-severity findings (9 of 9 addressed — 4 code-fixed, 2 doc-corrected,
2 registered to the backlog for owner ratification, 1 needs no change).**
- **MVP3-C3** — `.sergeant.toml.lock` (never removed by design) and
  `sergeant.toml.validate-*` (removed on the happy path, left behind on a
  mid-validate crash) were not gitignored, so `git status` on an
  initialized estate showed sgt's own runtime scratch as untracked. Fixed:
  added both to `GITIGNORE_ENTRIES`. Pinned by
  `domain::manifest::tests::init_writes_every_gitignore_entry` (now
  asserts every entry in the const, not two hardcoded strings).
- **MVP3-C4** — the design capture's `[estate] data_dir` field was never
  implemented (R-MVP1-1 ruled only `surfaces_dir`); a hand-written
  `[estate] data_dir = "..."` fails closed with a `deny_unknown_fields`
  parse refusal rather than overriding anything. Not implemented here —
  new config surface wants explicit ratification, not a silent addition
  inside a bug-fix pass. Design doc corrected in place ([v3] note,
  `docs/gauntlet/notes/estate-manifest-design-2026-08-11.md`); registered
  as backlog **B5**.
- **MVP3-C5** — `sgt run --turns`/`--ceiling-secs` (commit 78e5da9) added
  genuine new per-Work engine surface (a submit-time envelope override)
  that R-MVP1-7 specified as daemon-wide only, with no deviation-register
  entry. Counting integrity was never in question (not a bypass); the gap
  is process. Kept as shipped (reverting a milestone feature inside a
  fixer pass is a bigger unilateral move than registering it); registered
  as backlog **B6** for owner ratification at MVP-3's own adjudication.
- **MVP3-C6** — the manifest module's own doc claimed the advisory lock
  makes "two concurrent editors serialize rather than tearing," but the
  only test exercising that (`concurrent_repo_adds_do_not_lose_an_entry`)
  uses two threads in one process, which take the wait-and-retry path
  because the lock is already in that process's own `SELF_LOCKED` set —
  two real `sgt` processes never share it, so the real cross-process
  outcome is fail-closed refusal, not a queue. Fixed: corrected the
  module doc, and added
  `tests/m8_estate_cli.rs::two_real_sgt_processes_racing_repo_add_
  serialize_dont_tear`, which holds the lock file itself (a genuinely
  different process/file-description) and drives a real `sgt repo add`
  against it — proving the documented refusal, its exact message, and
  that a retry after release succeeds.
- **TH-3** — the m8 transcript test's guard-map claimed to prove `sgt
  work transcript` "decodes a completed work's conversation," but the fake
  backend never produces a turn, so only the empty-conversation rendering
  was ever checked. Fixed: corrected the guard-map to state what is and
  is not covered, pointing at TH-1's new real test for the blob-decode
  half.
- **TH-4** — `daemon_stop_drains_admission_and_exits_cleanly`'s mutation-
  kill list claimed to catch "the drain step never actually calling
  `/v1/admission/pause`," but nothing in the test submits work
  concurrently with a slow drain to observe that refusal — the claim was
  false. Fixed the doc to say so honestly, and named what a real fix
  needs (a scripted `SGT_FAKE_SCRIPT=hang` in-flight turn plus a
  concurrent submit racing a backgrounded `sgt daemon stop`) rather than
  building it inside this `info`-severity item's effort budget — left
  open, not silently claimed.
- **TH-5** — a positive-coverage finding (Q1/Q4 largely satisfied for the
  m8 suite's non-transcript/non-concurrency pins). No change needed;
  confirmed by inspection during this pass.

**TH-1 (warning) — the biggest single item, plus a bonus fix found while
closing it.** No test spanned producer (`backend::claude`'s
`TurnReader`) → journal → `transcript_turns`'s blob-decode fallback; the
only coverage fabricated both the `Event` and its payload by hand, so a
producer-side rename of the `raw`/`result_envelope` payload keys would
silently break recovery in production while both test suites stayed
green. **While building the real e2e closure, found and fixed a genuine
duplicate-reporting bug**: `ingest_line` emits `conversation.assistant.
completed` for any complete, successfully parsed assistant text line —
independent of whether the turn later gets a `result` line — so an
ordinary interrupted turn that streamed text before being killed hit
`transcript_turns`'s blob-decode branch too (gated only on
`result_envelope`), reporting the same text twice. Fixed:
`transcript_turns` now tracks, per `execution_id`, whether `conversation.
assistant.completed` already reached the journal since that execution's
last turn boundary, and skips the blob-decode fallback when it has —
scoped so a *different* execution's archive is still recovered
independently. Pinned by two new unit tests
(`transcript_turns_never_double_reports_text_the_live_event_already_
carried`, `transcript_turns_still_recovers_a_different_executions_
archive`). The e2e closure itself —
`transcript_turns_recovers_a_real_producers_text_across_a_simulated_
adjacent_append_loss` (`src/api.rs`) — drives a real `ClaudeBackend`
against a scripted `claude` CLI, converts its real `EventDraft`s into
real journaled `Event`s exactly as `daemon::journaling_sink` does, but
deliberately drops `conversation.assistant.completed` to model CLAUDE.md's
own "adjacent-append crash window" (the one scenario in which the blob
archive is not simply redundant with the live event, per the dedup fix
above) — then calls the real `transcript_turns`. Revert-probed by hand:
renaming the consumer's `raw` key read to `raw_x` (simulating a producer
rename gone unnoticed) fails this test immediately; reverted after
confirming.

### MVP-2 D3 FIXER PASS — 2026-08-12, N4/M7 panel findings closed

**Mission outcome.** Fixer pass over the N4 (Docker executor) build's
review panel: 26 CONFIRMED findings (12 `invariants:INV-R1-*`, 14
`test-honesty:TH-*` — 10 error, 10 warning, 6 info by severity), 0
PLAUSIBLE, 4 mutation-probe SURVIVORS. **19 of 26 CONFIRMED touched and
improved** — 14 fully closed as originally scoped, plus 5 explicitly
scoped as partial closes (INV-R1-02, INV-R1-07, INV-R1-09, TH-07, TH-08)
with the remaining ask named per finding below; **all 4 SURVIVORS
strengthened**, each re-verified by re-executing its named mutation
against this pass's own fix in a disposable worktree outside the tree
(L5/L7) — every one now fails where it previously passed. **7 CONFIRMED
findings investigated and left untouched**, named individually below with
reasoning — nothing silently dropped. Gates
green: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
full `cargo test` (496 tests, 0 failed, 4 opt-in ignored) all re-run
clean after the last commit. Leak checks clean after the full suite:
`pgrep -f "debug/sgt [-]-data-dir"` empty; `docker ps -a --filter
label=io.sergeant.managed=true` empty.

**Error-severity findings closed (5 of 10 — INV-R1-01, INV-R1-04, TH-01
[covered under Survivors below], TH-02, TH-05; INV-R1-02, INV-R1-03,
INV-R1-05, TH-03, TH-04 deferred, see below). TH-11 (warning) bundles
into the INV-R1-04 bullet below since one fix closes both.**
- **INV-R1-01** — `DockerBackend::stop`'s Docker Engine calls (`docker
  inspect` + `docker rm -f`, measured ~40-70ms) ran synchronously under
  the core lock via `Engine::stop_execution`, violating N4's own acceptance
  gate verbatim. Fixed: the whole inspect+label-check+remove interaction
  now runs as `Completion`'s deferred tail work (the same "kill now, join
  later" split `ClaudeBackend::stop` already uses), so nothing Docker-side
  runs under the lock. Pinned by a nonexistent-`docker_bin` unit test (the
  call itself must not shell out) and a real-Docker test proving the
  container survives until `.wait()` runs. `tests/m6_surfaces.rs` t11's
  whitelist rationale, previously written only about the Claude adapter,
  now names both backends' reviewed in-lock behavior.
- **INV-R1-04 / TH-11** — §16.8, a contract-named Unknown never measured:
  containers ran as image-default root, leaving root-owned files (and
  *un-removable root-owned directories*, stronger than the finding's own
  claim) in the user's worktree. Fixed: `--user <uid>:<gid>` sourced from
  the mounted worktree's own host owner. Pinned by a real-Docker test
  asserting both file and directory ownership, and that the host user can
  actually `rm` what the container created. `docs/environments/cerberus.md`
  now carries the measurement N4's Unknowns section required.
- **TH-02** — m6 t3's "every doctor check must be `ok`" assertion silently
  became Docker-reachability-dependent (measured: `warn` with no docker on
  PATH), the exact probe-gating failure N4.md names for the GH
  runner/cloud container — and `docker_check`, unlike `claude_check`, had
  no `SGT_CLAUDE_BIN`-equivalent override to probe-gate it with. Fixed:
  added `SGT_DOCKER_BIN` (`DockerConfig::new` honors it, mirroring
  `CLAUDE_BIN_ENV`); threaded a scripted `docker` stub through every
  `doctor()` call in the suite. Measured the fix directly (not just via
  the test): `warn` without the override, `ok` with it, on the same
  docker-less PATH.
- **TH-05 / SURVIVOR 2** — §17.5's execute-stage submit preflight
  (`ExecuteBackendUnavailable`) had zero tests despite the injection point
  (`DaemonConfig::docker`'s scripted `docker_bin`) already existing and
  being documented for exactly this. A mutation that swallows the routing
  error and falls back to actor routing survived the whole suite. Fixed
  with a test needing no live Docker: submits a `kind = "execute"`
  workflow against a daemon whose docker backend cannot be routed to,
  asserts 422/`execute_backend_unavailable`, no Work, no `surfaces` dir.
  Mutation re-executed in a disposable worktree: without the fix the same
  submission completes end to end (201) with the stage silently routed to
  the fake actor backend.

**Warning-severity findings (10 total; TH-11 already counted above as
part of the INV-R1-04 bullet). 5 of the remaining 9 fully closed
(INV-R1-06, INV-R1-08, TH-06, TH-09, plus TH-11 above), 4 explicitly
partial (INV-R1-07, INV-R1-09, TH-07, TH-08 — TH-08's crash-injection
half stays open, see Deferred), TH-10 untouched (see Deferred).**
- **INV-R1-06** — §16.3's "a version ping proves only that something
  answered" was not what `sgt doctor` shipped: `docker_check` called only
  the cheap `probe()`; `DockerBackend::lifecycle_probe` (the real
  bind-mount round trip) was dead code the module doc falsely claimed was
  already wired in. Fixed: `docker_check` now runs the lifecycle probe
  whenever the cheap ping succeeds and folds a failed round trip into
  `Warn` with real evidence. Measured against real Docker on this host:
  `"Docker Engine 29.7.2; bind-mount round trip confirmed"`, <1s added.
- **INV-R1-07 (partial: 2 of 17 named tests + the ownership gap already
  closed above via INV-R1-04).** Added `a_mount_path_containing_a_space_
  round_trips_correctly` (test 2) and `a_launched_container_carries_no_
  isolation_escape_hatches` (test 6 — the negative isolation posture
  `create_container`'s own comment claims, never inspected on a real
  container: exactly one mount, no `docker.sock`, not privileged, no
  added capabilities, no devices). **Still open**: tests 1 (API
  negotiation + server/platform evidence), 7 (mutable tag resolves to the
  *journaled* identity, tied to INV-R1-05 below), 11/12 (restart-while-
  running / restart-after-exit), 16 (pull-failure/registry-auth sanitized
  evidence), and all of §22.9 (image/cache pressure) — see Deferred.
- **INV-R1-08** — `observe()` pays for a full log capture (blob writes)
  unconditionally on any exited container, even from restart
  reconciliation's `reserved_identity_liveness`, which reads only
  `.native` and discards the rest. Fixed: added `Backend::observe_liveness`
  (default impl delegates to `observe`, correct as-is for fake/Claude);
  `DockerBackend` overrides it to classify liveness without ever calling
  `capture`. Pinned by a real-Docker test counting blob-store files
  before/after both calls (liveness: no growth; full observe: growth).
  Mutation (fall back to the default) re-executed in a disposable
  worktree: the new test fails against it.
- **TH-06** — the commit's explicit claim that `ExecuteSpec` participates
  in `WorkflowDefinition::content_hash` had no test; deleting
  `execute: s.execute.as_ref()` left the suite green. Fixed: added a case
  to the existing hash test (two workflows differing only in an execute
  stage's pinned image must hash differently). Mutation re-executed: fails
  without the line.
- **TH-07 (partial: the RSS-adjacent disk-cost measurement, not the
  no-orphan-blob half).** [Rule B] names two acceptance items; only the
  first ("measure blob disk cost beside §22.8's RSS budget") is closed —
  the large-output test now also asserts the blob store grew by the
  expected amount on disk. Fixture bug found and fixed in passing: stdout
  and stderr previously wrote *identical* content, and the content-
  addressed blob store correctly deduplicates that into one blob, which
  would have failed the new assertion against a store working exactly as
  designed. The second half ("no blob is written without a journal ref
  naming it") is not pinned — see Deferred.
- **TH-09** — `resume: true` is advertised (L8) but only its negative row
  (missing container) was tested. Added a test driving both untested
  rows: a live labeled container is re-adopted (`Ok(())`), a foreign one
  under the deterministic name is refused.
- **INV-R1-09 (partial: log-visible disclosure only).** D2's
  `--setting-sources user,project,local` translation for `instructions =
  "local"` authorizes repo-authored hooks/tool-permissions/MCP config —
  "a materially larger risk than 'reads a text file'" by the adapter's
  own measurement note — and the submit-time refusal that gated this on
  an unmeasured claim was removed with no replacement operator signal.
  Closed the log-visibility slice: `check_instruction_policy` now emits
  `tracing::warn!` naming the repos and the widened surface. **Not
  closed**: a first-class operator signal (doctor row, manifest
  vocabulary) — see Deferred. Not test-pinned (a tracing-log-only change);
  verified manually instead — a real `sgt run` against an `instructions =
  "local"` manifest with `RUST_LOG=warn` produces exactly this line in
  the auto-spawned daemon's `daemon.log`.

**Info-severity findings closed (4 of 6 — INV-R1-10 and TH-14 deferred,
see below).** INV-R1-12 (`--mount`'s CSV grammar (`,`/`=`-delimited)
built by string interpolation from the worktree path; a `,`/`=`-bearing
path splits into a malformed mount, measured — refused closed before
image resolution, so the refusal needs no live Docker; pinned by a unit
test with a nonexistent `docker_bin`); INV-R1-13 (`ResumeRequest.
instruction_policy` widened to `Option`, matching its own doc's "not
re-supplied" contract — `model`/`profile` were already `Option`, this
field was the odd one out); TH-12 (the network-isolation test only
discriminates on a host with egress; added a positive control — the same
command over the default network must actually reach out first, or the
test skips loudly rather than reporting a meaningless pass); TH-13 (the
exit-mapping test's two fixtures paired neutral stdout with a matching
exit code, so a stdout-scraping implementation would have passed too;
added adversarially-paired cases, and fixed a latent bug the new cases
exposed — the failure-arm assertion was hardcoded to exit code `7`
regardless of which case was running).

**Survivors, all 4 strengthened and mutation-reverified.**
1. `large_captured_output_does_not_grow_this_process_proportionally`
   (TH-01's own test) — same fix as TH-01 above (`VmHWM` not `VmRSS`).
2. The execute-preflight swallow — same fix as TH-05 above.
3. `latest_ask_withdrawal_version_picks_the_highest_seq_matching_event` —
   its "out of order" case put the highest-seq record first in iterator
   order, so "take the highest seq" and "take the first match" gave
   identical answers; added the mirrored ordering (highest seq last).
4. Composition probe C2 — `observe()`'s §16.10 identity check exists in
   code but no test ever drove OBSERVE (only LAUNCH) against a container
   occupying the deterministic name under foreign labels. Added
   `observe_on_a_foreign_container_under_the_deterministic_name_fails_
   closed`, forging a handle against a pre-created unlabeled container.

**Deferred/partial (5 partial closes + 7 fully untouched — 12 CONFIRMED
findings total, all investigated, none closed as originally scoped,
named individually so nothing is silently dropped).**
- **INV-R1-02 / TH-08 (partial)** — both named the §22.5 crash-injection
  matrix over the Docker lifecycle (all create/start/exit/cancel windows)
  and §22.6 lock-discipline coverage for Docker. The lock-discipline half
  is now addressed (INV-R1-01's fix + the m6 t11 whitelist comment update
  above); the crash-injection matrix itself — killing a daemon
  mid-Docker-lifecycle and asserting recovery, the same shape m4's
  `n10`-`n18` fixtures already build for the Claude/fake backends — is
  real, substantial harness work (a daemon-kill rig driving real
  container state) not attempted in this pass.
- **INV-R1-03** — §16.11's "reserved, no recorded ID / one exact
  name+label match → adopt" row is unimplemented:
  `reconcile_unsettled_reservation` returns `Ambiguous` unconditionally
  and never reaches Docker's own stronger evidence (name+label match).
  Closing this means teaching engine-level restart reconciliation a
  Docker-specific adoption path without leaking Docker knowledge into the
  generic engine (§13.2) — an engine-level design change, not a local
  fix, deferred to a build pass rather than attempted piecemeal here.
- **INV-R1-05** — the image pin (`<data_dir>/docker-adapter/image-pins/`)
  is the retry decision's actual source of truth but lives outside the
  journal, best-effort-written, with no rebuild-from-journal path — a
  real journal-only-truth gap. Fixing it properly (rebuild the pin cache
  from `execute.image_resolved` events at startup, or make the pin write
  itself journaled and synchronous before `docker start`) is a
  daemon-startup-path change with its own crash-window analysis (L6)
  this pass did not have room to do carefully.
- **INV-R1-07 (remainder)** — tests 1, 7, 11, 12, 16 and all of §22.9;
  see the partial-close note above for which two of seventeen landed.
- **INV-R1-10** — `capture: "complete"` is asserted on any successful
  `docker logs` read with no check of the container's own
  `HostConfig.LogConfig`; a rotating `json-file` driver or `--log-driver
  none` would misreport truncated/unavailable logs as complete. Fix is a
  driver check at create time or a named `"truncated"`/`"driver_
  unsupported"` outcome — not attempted.
- **TH-03** — the suite's only environment gate is `docker version`; on a
  host with Docker but no registry egress (the cloud container N4.md
  names explicitly) every container-creating test fails hard instead of
  probe-gating via a Sergeant-owned static (`FROM scratch`) probe image.
  Cerberus itself has open egress, so this gap does not bite locally;
  building and vendoring a static probe image plus an image-availability
  gate is real work not attempted here.
- **TH-04** — the `local` instruction policy's shipped
  `--setting-sources user,project,local` value is asserted against a
  stub, never measured against the real installed Claude CLI (no
  `#[ignore]`/`SERGEANT_CLAUDE_TESTS` test drives a Local-policy turn).
  Fixing this means spending real tokens against the live CLI, which — per
  the MVP-1 fixer pass's own precedent (its I2) — a fixer pass does not
  authorize itself; left for a session with owner-approved token spend.
- **TH-14** — N4's "first real execute stage"
  (`.sergeant/workflows/repo-to-icm/workflow.toml`'s `65-self-check`) is
  validated only by a hand-run `docker run` recorded in a commit message;
  no automated test loads and actually executes it (the automated m7
  proof uses a synthetic `mixed-proof` workflow instead). Building this
  properly means a test that materializes the real repo-to-icm workflow
  and runs its execute stage end to end — moderate effort, not attempted
  in this pass.
- **TH-10** — R-H0-7's fake-fidelity work (`FakeBackend::settle_as`, the
  interrupt-vs-terminal-signal fix) is pinned only by in-module unit
  tests; `settle_as` has no caller outside its own test and the interrupt
  fix has no engine-level consumer, so no test demonstrates either shape
  ever changes an engine outcome. Needs an engine-level test built around
  a scripted fake that would produce a different result with vs. without
  the fix — not attempted; the underlying R-H0-7 bug fix itself is not in
  question, only this coverage gap.

**Evidence.** Every closed finding's commit (`git log --oneline
fec2e3c..4e3f625` — 14 commits over this pass) carries its own
measurement transcript, mutation re-execution, or manual-verification
transcript inline; not restated here. `docs/environments/cerberus.md`
gained the §16.8 measurement row N4.md's Unknowns section required.

### MVP-1 SHIPPING GATE — 2026-08-12, passed (22→4→0); as-run note

no-mistakes over the full MVP-1 diff: 22 findings, 12 fixed (headline:
terminal-run replay under the CoreGuard past the 512 cache; the ceiling
interrupt gaining §14.5 staleness discipline + its L7 pin on the gate's
own insistence), 10 accepted as documented trades. Self-hosting
checkpoint passed pre-gate: #50 fixed via a sergeant Work (2 turns of a
12 cap; fix verified by independent reproduction; evidence in
docs/gauntlet/runs/mvp1-selfhost/). As-run provenance note: the gate's
fix round edited resources/n-series/mvp1-build.js (axis-misalignment
correction) AFTER that script's run — the file carries a post-run
correction marker; the version that drove the build is the parent of
the fix commit. Owner ruling applied mid-pass: dollar figures are
telemetry only, never guards — all bounds speak turns/wall-clock.


### MVP-1 FIXER PASS — 2026-08-12, panel findings closed (I6)

**Mission outcome.** Fixer pass over the MVP-1 build-panel review: 20
CONFIRMED findings, 1 PLAUSIBLE, 1 mutation-probe SURVIVOR — every one closed
or refuted with evidence, nothing silently dropped. Gates green (`cargo fmt
--check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, repeated
across runs, not single-run — L7); `pgrep -af "sergeant-rs/target/debug/sgt
[-]-data-dir"` empty after every commit in this pass.

**Errors closed.** E1 (`DEFAULT_TURN_CAP` 6→12, evidence-grounded in this
repo's own longest admitted workflow's stage count, plus a real production
config surface — `DaemonConfig::turn_cap`/`SGT_TURN_CAP`, the same pattern
`turn_ceiling`/`surfaces_root` already use); E2/TH-02 (R-MVP1-2's shared
finalize helper generalized to `.sergeant/lib/finalize.py`, repo-to-icm's own
copy now a thin back-compat wrapper, R-MVP1-2's own L7 pin now has a real
test); E3 (R-MVP1-10's exit door for the envelope-exhausted landing —
`Engine::extend_turn_envelope`, `KIND_TURN_ENVELOPE_EXTENDED`,
`POST /v1/work/{id}/extend`, `sgt extend` — `retry` alone was a revolving
door; `extend` then `retry` now genuinely reopens the room).

**Warnings closed.** W1/W2/TH-01/TH-09 (the fleet view — `GET /v1/work` —
bypassed R-MVP1-9's re-derivation entirely for evicted works, and the
re-derivation path itself replayed the *whole* journal per view; both fixed
by a bounded (`TERMINAL_RUN_CACHE_CAPACITY = 512`), LRU-evicted
`WorkRegistry::terminal_runs` cache populated at eviction time — flat under
churn beyond its own bound, unlike an unbounded cache would be); W3
(`SGT_SURFACES_DIR` now actually read, `Engine::with_surfaces_root` had zero
production callers before this); W4 (the estate-discovery upward walk now
bounded at an explicit `--data-dir`/`SGT_DATA_DIR` scope too, not only
`$HOME`); W5 (a legacy-vocabulary or malformed `sergeant.toml` the walk steps
over on the way up now fails the whole walk closed, matching R-MVP1-3's named
refusal, instead of being silently skipped); W6 (`due_interrupts`'s doc
corrected — it is not side-effect-free, it destructively dequeues — and the
driver's interrupt-delivery loop no longer abandons an already-dequeued entry
on a shutdown race); W7 (doc-only: the AGENTS.md identity hash is measured
true only for `local`, which cannot currently reach a launch — `suppress`,
the shipped default, hashes a file the adapter does not read); W8 (`sgt work
show`'s human form now renders `output`/`teardown`, previously dropped by
cli.rs's own key whitelist).

**Info findings closed.** I1 (folded into E1 — the new default is evidence-
grounded, not measured against live N-series turns; that gap is named
honestly in the constant's own doc and here); I2 (addendum to
`docs/gauntlet/notes/multirepo-measurement-2026-08-11.md`: its stated blocker
is gone now that R-MVP1-7 has landed, but the actual real-Claude re-measure
was not run in this pass — spending real tokens is not something a fixer
pass authorizes itself); I3 (`IntentDetail::is_empty`, previously dead code,
now normalizes an empty `intent_detail: {}` to absent at submit;
`group`'s non-validation documented in place as R-MVP1-5(b)'s own scope, not
a gap); I4 (`run_is_settled` gained the third eviction-safety condition —
`surface_plan` recorded but `surface` not yet materialized — closing the
cancel-during-materialize race the doc already claimed was covered); this
entry (I6 — the ledger entry and evidence trail I6 itself named as missing).
I5 (PLAUSIBLE) investigated: found and reaped a real, currently-running
orphan daemon on this host (`sgt daemon --data-dir /tmp/sgt-demo-*`, PPID 1,
its own data dir already deleted) — confirms the underlying concern
(build-lane daemons do leak on this host) with independent evidence, though
not the specific PID/path originally reported. Worth flagging structurally:
this daemon's argv (`sgt daemon --data-dir <dir>`, subcommand before the
flag) does not match the mandated bracketed leak-check pattern
(`sgt [-]-data-dir`, which assumes the flag immediately follows the binary
name) — a real blind spot in that convention, not fixed here (out of this
pass's scope; flagged for whoever owns the housekeeping loop next).

**Test-honesty findings closed.** TH-01/TH-02/TH-09 above; TH-03 (a standing
hygiene gate — `no_committed_sergeant_toml_outside_reference_carries_
legacy_vocabulary` — replaces the one-time manual grep R-MVP1-3's pin relied
on); TH-05 (the ceiling test's budget was 37x its own claimed bound, and its
`outcome.requested` assertion was vacuous on both `settle_interrupt` arms —
both fixed); TH-06 (the `grilling` declaration pin now uses
`CARGO_MANIFEST_DIR`, fails closed instead of silently `SKIPPED`ing outside
the checkout root); TH-07 (the R-MVP1-1 in-checkout guard test now names the
actual refusal diagnostic instead of a bare state check, and
`surface.planned`'s own `root` is pinned); TH-08 (`scripts/perf/s2-churn.sh`
actually run, for real, against this pass's own W1/W2 fix —
`docs/perf/s2-churn-mvp1-fixer-2026-08-12.md`; decelerating per-wave RSS
slope, not the pre-eviction monotonic climb, real fds, clean hygiene sweep;
time-boxed to 60 works, not the full 200-work contract cell, named as such);
TH-10 (widened the settle margin
in the real-backend-refusal fault test — a genuine, not merely theoretical,
scheduling race with the completion driver); TH-11 (R-MVP1-11's refusal now
pinned through a real HTTP submit, matching its R-MVP1-4 siblings); TH-12
(`intent_detail.repos`'s intentional inertness when `repositories` is absent
now has a test in each direction); TH-13 (two broken intra-doc links fixed).

**SURVIVOR strengthened.** `no_estate_anywhere_falls_back_to_zero_config_
unchanged`'s own fixture now includes a non-estate `sergeant.toml` above
`root`, so the named mutation (the estate-table check dropped from the
walk's match predicate) actually reaches a file and the test now kills it —
verified directly, not merely re-attributed to the sibling test that already
caught it.

**What this pass did not do.** A live-Claude re-measure for I2 (see above);
retrofitting every admitted workflow's own closing stage to invoke the
shared finalize helper (E2's own boundary: "Owner: workflow content" per
R-MVP1-2 itself — core's job was the shared helper and the pointer, both
done); a dedicated deterministic regression test for W6's shutdown-drop
race specifically (fixed and verified by code inspection — the only
`closing` check that could discard an already-dequeued interrupt is the one
removed — but a reliable timing-controlled test for it was judged not worth
the flakiness risk, TH-10's own class of hazard). B4 (the backlog row for
R-MVP1-10's *pending*-origin landing) is untouched by this pass's E3 fix,
which closed the *envelope-exhausted* landing specifically — B4 remains
open, its own trigger unfired.

**Deviation note (R-MVP1-2 / NORTH-STAR).** Already self-documented at
`NORTH-STAR.md`'s own amendment line (2026-08-11, "R-MVP1-2 held:
promote/finalize EXECUTION is workflow content... only the pointer is
core") per that document's own "amended in place with a dated entry"
convention — not duplicated into this file's deviation register, which is
scoped to departures from `reference/proposal-depot-rust-execution-surface.md`
specifically, a different governing document. The schema break (R-MVP1-3's
`[workspace]`/`[[repository]]` → `[estate]`/`[[repo]]` rename) is likewise
self-evident from the migration itself (`WorkspaceError::LegacyVocabulary`,
the fixtures, TH-03's new standing gate above) rather than a second written
record.

---

### CERBERUS DAY 2 — 2026-08-11, direction: North Star adjudicated, MVP plan triple-reviewed, goal prompt cut

**Mission outcome.** Adjudication day, zero engine code. (1) Retention
ruling ADJUDICATED (Rule A → later re-homed standalone to MVP-1 by
owner amendment; Rule C amended compress-first, rebuild-time trigger
binding). (2) Owner rulings landed as method: execution-surface
taxonomy (convention §2a + bucket 4 absorbed-by-engine), the epistemic
license (every ruling is a hypothesis; encoded in the north-star
workflow), the adapter-boundary rule (core semantics never defined by
adapter flags — the `--setting-sources` de-leak), estate model
(clone-is-distro, repos/ working set not dev_root, nothing at `~/`,
manifest as keystone with three-pens/pin-at-bind semantics). (3) The
dogfood gauntlet ($5.29 of $25): settle driver ran two workflows
end-to-end unattended; research produced a load-bearing artifact
(zstd 6.5–7.7× measured, correcting the ruling draft's ~10×); grilling
measured structurally unable to hold on this host; E1–E7 ranked;
verdict "the library didn't fail the assignment — the assignment's
final exam had never been administered." (4) Library re-triaged twice
(23/35 stand; respond-to-worker retires to docs as shipped `sgt
respond`; dispatch mechanics absorbed) — L18 minted: R1's "already
exists" includes the product you are building. (5) Upstream settled:
36-script logical function map (8-function honest delta; td answered —
journal absorbs queue-dedup, td-memory dies, standing backlog is the
residue), 12-lesson issues/PR mining (fail-closed-no-exit-door the live
risk; #205 injection class audited clean here). (6) **NORTH-STAR.md**:
4 blind seats → synthesis → 3 steelmanned challengers → dispositions →
owner rulings (R-NS-6 execution≠dialogue dissolved the interactive
fork; surfaces add usability never functionality). (7) MVP plan cut by
the owner's layer ladder, then **triple-reviewed** (4-critic pipeline →
Opus writer re-verifying every cite → sanity pass; 31 findings, 29
applied — best catch: the turn envelope at PREPARE/LAUNCH-only misses
`send`-spawned turns; plus the outside Codex seat's self-hosting
surfaces contradiction → data_dir/surfaces_root split owed in MVP-1).
Escalations ruled: Rule A standalone, schema rename-with-refusal,
execute subject cheap-fast-frequent. (8) P2-JOURNAL vendored + slotted
post-MVP; P2-vs-T-minimal demoted to a pilot ruling. (9)
`docs/gauntlet/goal-prompt-mvp.md` cut — the standing mission brief.
PRs: #58 (day-2 head PR, everything above).

**Environmental behavior.** Grill-with-docs method run live all day
(one question at a time, captures committed as decisions landed) — the
owner corrected the orchestrator repeatedly and the corrections were
the day's highest-value inputs: the fleet-domain overreach, the
dev_root/repos conflation, "you're agreeing not analyzing" (which
produced the four-crack self-audit), and the missed-review catch (the
orchestrator authored the governing plan with zero fresh eyes — the
owner-shaped 4→1→1 pipeline then found 14 errors in it, base rate
vindicated again). Orchestrator's own R-H0-2 recommendation overturned
by its own workflow's evidence and the overturn upheld. ~2.5M subagent
tokens across north-star + review pipelines + classifiers. The
epistemic license and the owner's "take my rulings with a grain of
salt" are now durable method text — subagent seats are briefed that
ratifying the record is failing the brief.

---

### CERBERUS SESSION CLOSE-OUT — 2026-08-11, first host session: BS2+#44 shipped, library promoted, runs 4 + B2, H0 drafted

**Mission outcome.** First non-container session, everything below on the
session branch (head PR #51; owner merges). (1) **Environment first
contact**: `docs/environments/cerberus.md` measured (Docker full matrix
incl. digest pulls; DAC enforced; open network; O_DIRECT-on-tmpfs delta),
`scripts/probe-env.sh` shipped (retro item 1; its critic caught 3
fabricated-fact errors). (2) **L1 fired on 2.1.227**: `post_turn_summary`
absent on this host (5-probe isolation; auth-method confounder recorded)
— ask affordance gone, withdrawal path operative, a5 probe-gated.
(3) **Perf re-baselined** (`docs/perf/baseline-cerberus-2026-08-11.md`):
burst-50 42.0 works/s, rebuild 54.6k ev/s; #4's RSS shape reproduces
exactly — code, not environment. (4) **BS2 closed #46/#47**: root cause
was OBSERVE starvation after turn end (fake turns settle at launch — a
352-green suite coexisted with a 45-minute stall; one stub parameter,
`stalls_for`, was the whole difference); fix is a completion driver as a
fourth pending-effect performer (§22.6 by construction, §14.5 currency
pins); permission mode became profile config with the skip flag deleted
outright. (5) **#44 closed**: `CoreGuard` one-door hold boundary, one
fsync per hold — fdatasync 1157→253 per burst-50, honest reading
recorded (no throughput delta on this host; the win is O(lock-holds)
journal cost for N4's volume). Shipping gate PASSED after three
selective fix rounds (16→3→1 findings; flush-poison-bypass the best
catch); 379 tests at close. (6) **Run 4 completed end-to-end**: 21/21
partitions, 1,333 units, 44 draft packages, every citation hash
re-verified; within-run multi-attempt continuation measured for the
first time (the protocol's `retry` prescription was never real — #53);
two fail-closed escalations were genuine (partition-scheme
non-determinism; a one-file census delta ruled per the frozen
reference). (7) **Run B2**: the settle driver fired live twice on real
Claude (autonomous cascades, zero client commands), withdrawal live,
#47's bypass opt-in exercised; $4.62 spent against a $2.50 guard — see
L16; #19 stays open. (8) **The promotion library landed**: 34/34
adjudicated packages curated into `.sergeant/workflows/`, each
engine-gated; the §9.7 validator gained the `--admitted` mode that made
promoted packages validatable at all (S12 → warn per D9); library
critic 11 findings/6 errors, all closed. (9) **Drafts for owner
adjudication**: retention ruling (R-N0-3's gate), N4 contract, and the
H0 packet for the vendored harness-adapter research
(`reference/proposal-harness-adapter-research-v2.md`). Issues filed
#50/#53/#57; #44/#46/#47 close via trailers when #51 merges.

**Environmental behavior.** Five workflows + support agents, ~16M
subagent tokens, ~12h wall. Model spread per the same-day revision
(Sonnet default, Opus judged, no Fable seat needed). Panels earned their
keep everywhere: BS2 round 1 11/15 confirmed, round 2 15/16, promotion
6 errors, probe-env 3 — and the pipeline gate three-for-three found
real defects the panels missed, including in its own fixes (L7 on the
gate, again). Owner interventions mid-session, all adopted: merge model
(head PR + sub-PRs), economy ruling (small diffs batch into larger
panels — gauntlet-pattern revision), "be intelligent about grouping"
(→ worktree lanes + shared round-2 panel), promotion rerun caution
(cache-resume discipline held; zero reruns needed). Orchestrator errors,
recorded not hidden: untracked files left in a builder-owned checkout
were swept into B1's commit (separability polluted); the harvest
driver's first two designs encoded unmeasured mechanisms (a keyword
trip-wire and the protocol's own wrong `retry` verb — the fail-closed
stops that caught both were the system working); Run B2's guard could
never fire below single-turn granularity and its cancel came from an
orphaned collector racing the orchestrator's TaskStop (→ L16, L17).

---

### ROUND-2 FIXER — #44 close-out follow-through (Cerberus, 2026-08-11)

**Mission outcome. Closed.** Round-2 review of #44 (journal group commit)
returned 15 CONFIRMED findings (0 PLAUSIBLE) across two axes —
`invariants-r2` (INV-R2-01/02/04–09) and `test-honesty-r2` (TH-R2-01–07) —
plus one adversarial-verify survivor (mutation C1: reordering `Core::flush`
to broadcast the group before consulting the fsync's own result). Every
confirmed finding is closed below, each with a test that fails when its fix
is reverted (L7) or, for the two purely-positive confirmations, re-verified
rather than left unactioned. The survivor is closed with a dedicated test,
mutation-killed in a disposable worktree with its own `CARGO_TARGET_DIR`
(L5), removed after.

**INV-R2-01 (error) — startup had no durability boundary.** `daemon.rs`'s
startup ran with a bare `Core` — no `CoreGuard`, so nothing flushed —
across `daemon.started`, every `backend.probed`, and the whole of
`recovery::reconcile`, while that last one performs unbounded external
effects (`git worktree remove`, harness relaunch). Fixed: `core.flush()`
after `daemon.started`, again after the backend-probe loop, and
`recovery::reconcile` now flushes after every work it touches (isolated
per-work, matching its existing per-work error isolation) plus a backstop
flush before the descriptor is published. Pinned by
`runtime::recovery::tests::reconcile_flushes_after_every_work_so_nothing_it_touches_is_left_unsynced`
— reverting the per-work flush calls fails it (`open_group_len() == 12`
instead of `0`), verified by mutation in-tree and restored.

**INV-R2-02 (warning) — `CoreGuard`'s "no external effect under the guard"
claim was false.** `backend.stop()` — a synchronous kill+reap — runs under
the request-path guard at two sites (`stop_execution`, `settle_launch`'s
stale-reservation arm), a pre-existing, reviewed exemption (issue #14/B3)
that `t11`'s effect enumeration never named and `CoreGuard`'s own doc
contradicted. Fixed: `CoreGuard`'s doc now states the exemption and what
#44 changed about it (the kill can now precede the durability of the event
recording it); `t11` now scans `backend.stop(`/`backend.interrupt(` too,
with `stop_execution`/`settle_launch` named as the reviewed exception
rather than silently passing. `t11` still green; the exemption is visible
instead of invisible.

**INV-R2-04 / TH-R2-04 (warning, same defect) — `t11c`'s "every module"
claim covered six hardcoded files out of 31, two lock spellings out of six,
and no receiver-renaming.** Rewrote `t11c` to walk every `.rs` file under
`src/` (`all_src_files`), scan all six `tokio::sync::Mutex` lock-taking
spellings, and additionally track any identifier bound from
`core.upgrade()` (closing the concrete gap `daemon.rs`'s `let Some(live) =
core.upgrade()` demonstrated) as its own receiver. Mutation-verified in
place: an injected `core.try_lock()` in `engine.rs` — a file the old
six-file list would never have scanned — is caught by the new walk and
missed if reverted to the list. Also fixed a latent non-char-boundary panic
in the assertion's own error-snippet slicing (`§` in nearby doc comments),
found while exercising this path.

**INV-R2-05 (info) — the `journal_append_seconds` metric silently
narrowed.** `Telemetry::record_journal_append`'s doc now states the #44
narrowing explicitly (write-only, no fsync) and that no §28 instrument
replaced the lost visibility into the group fsync — matching the honesty
`journal.rs`'s own `AppendObserver` doc already had. Documentation-only;
no new instrument added in this pass.

**INV-R2-06 (info) — `Core::flush`'s "dropped unannounced" claim was
unscoped.** Doc now states plainly that "unannounced" means the broadcast
channel only — `write_all` and the registry fold already happened before a
group fsync can fail, so every read-only endpoint (`events_after`, SSE
history/refill, analytics catch-up, every projection-backed view) still
serves a failed group's events from a journal that has since poisoned
itself.

**INV-R2-07 (info) — #44's raw perf artifacts were never durable.**
`docs/perf/n3-group-commit-2026-08-11.md` cited six JSON summaries and two
strace outputs "under the session scratchpad"; confirmed gone (no matching
files anywhere in this checkout, none ever committed) — a session
scratchpad does not survive a container reset, let alone a new session.
Corrected in place: the transcribed table is now stated as the only
surviving evidence, with a going-forward rule (commit raw harness output
under `docs/perf/` next to the note that cites it).

**INV-R2-08 (info) — `settle_send`'s INV-1 guarantee has an unstated
dependency.** `KIND_STAGE_INPUT_RECEIVED` has no projection reducer arm, so
a crash between `begin_input`'s append and `settle_send` leaves a stage
`due_observations` would skip forever on its own; INV-1 only closes that
window when `settle_send` gets to run at all. Added a **Residual** doc
paragraph on `settle_send` making explicit that the crash-before-settle
case is closed by `runtime::recovery`'s fail-closed reconciliation, not by
this guarantee — complementary, not redundant, and worth saying so.

**INV-R2-09 (info) — verified negative, and one stale per-host fact found
while checking it.** `REQUIRED_FLAGS` no longer names
`--dangerously-skip-permissions`, and nothing (tests, `permission_mode_check`,
`launch_config`) still assumes it — confirmed, nothing to change in code.
`docs/environments/cerberus.md`'s uid-1001 viability row still asserted in
the present tense that the flag is "carried" by the adapter's launch
grammar; corrected in place with a dated note pointing at #47 (`06fb6e8`).

**TH-R2-01 (warning) — n32's "six events a submit's first hold appends"
claim was false.** The real first hold appends two events
(`work.submitted`, `surface.materializing`); the five-event settle hold
that follows is a separate hold, with `surface.materialized` *before*
`workflow.bound` — the reverse of n32's fixture order. n32's group is a
hand-built worst-case bound, not a reproduction of any single production
hold. Restated honestly (the offered lower-risk fix, not the
producer-derive rewrite) in `tests/m4_backends.rs` (module comment above
n32 plus its assertion messages) and `src/api.rs` (`CoreGuard`'s cost
justification and the L6 proof paragraph). **This ledger's own #44 entry
below is not rewritten — append-only — but its "n32, which truncates a real
six-event grouped hold" phrase should be read against this correction.**

**TH-R2-02 (warning) — a5's SKIPPED-ENV arm was not falsifiable.** Two of
its three assertions were tautologies entailed by the same `AtomicBool`
that gates the withdrawal event's emission, and the discriminator between
"withdraw" and "map to NeedsInput" was the adapter code under test — a
parser regression that stopped recognising a present `post_turn_summary`
line would have silently passed as an environment fact. Fixed: the arm now
fetches the archived transcript via `BlobStore` (the same independent-
evidence pattern `bs2`'s `permission_denied` check already uses) and
asserts the line really is absent, not just unclassified.

**TH-R2-03 (warning) — the CLAUDE.md suite-count line went stale again,
inside the very round that reviewed the diff that staled it.** Round 1's
fix (64fbdf9) set 371; #44's two commits added 6 tests without touching
that line. Updated to 379 (measured post-round-2-fixer: 163 lib + 14 + 41 +
33 + 84 + 17 + 27 across the six suites, +2 for this round's own new
tests), with a note recording the re-staling as a small lesson in itself.

**TH-R2-05 (info) — the STDERR_DRAIN_BUDGET guard's mutation kills by
hanging forever, not failing.** `a_stderr_sender_that_never_sends_...` used
to call `reader.run` in-thread; mutating its `recv_timeout` to an unbounded
`recv()` blocked the test harness forever (no per-test timeout). Fixed:
`reader.run` now runs on its own thread, and the test bounds *its own* wait
with `recv_timeout(STDERR_DRAIN_BUDGET * 6)`, panicking with a named
message on timeout. Mutation-verified: the same mutation now fails in ~30s
with a clear panic instead of hanging; reverted and re-confirmed green.

**TH-R2-06 / TH-R2-07 (info) — positive confirmations, no defect.** Both
findings verified existing work (the settle-seam tests' timing rigor;
TH-2/TH-3/#44's crash-injection fixture honesty) and found it sound. No
code change; re-verified green under every change this round made
(`cargo test`, full suite, 0 failures).

**Survivor — mutation C1 (`Core::flush` publish-before-sync reorder).**
Reordering `flush` to broadcast the group before checking whether the
group's own fsync succeeded is indistinguishable from the real code on
every existing test's path (all of them drive success only) and matters
only when the fsync fails — exactly the case `flush`'s own doc comment
promises against. Closed with
`api::tests::a_failed_group_sync_publishes_nothing_not_even_before_returning_the_error`,
which injects a real fsync failure (the `O_PATH` trick `journal`'s own
poisoning test already uses, promoted to a shared, `pub(crate)`
`journal::tests::make_unsyncable_for_tests` so it stays inside `t11b`'s
`mod tests` exemption rather than needing a new one) and asserts the live
subscriber's mailbox is empty, not just that `open_group_len()` reads zero
— the length is equally zero whether the events were dropped or
published-then-dropped, so only the mailbox tells the two apart.
Mutation-killed in a disposable copy under the session scratchpad with its
own `CARGO_TARGET_DIR` (L5), confirmed failing, restored, and removed
after.

**Gates.** `cargo fmt --check && cargo clippy --all-targets -- -D warnings
&& cargo test`: all three green. 379 passed + 4 opt-in ignored (0 failed),
up from 377 + 4 at #44's own close. Leak check
(`pgrep -af "sergeant-rs/target/debug/sgt [-]-data-dir"`) empty.
Orchestrator-verified per R-S0-1 (no-mistakes gate not re-run this pass);
hygiene sweep clean.

**Environmental behavior.** Single-agent fixer pass, not a full panel —
recorded as such, matching this series' established convention for
follow-through passes. The formal SURVIVORS-section mutation (`Core::flush`
reorder, C1) ran in a disposable copy outside the tree with its own
`CARGO_TARGET_DIR`, per L5's letter, confirmed failing and then removed.
Three other red/green checks in this pass — the new `reconcile` per-work
flush test, the `t11c` `src/`-walk widening, and the `STDERR_DRAIN_BUDGET`
watchdog — were each mutated and restored in place in the main checkout
(single-file edit, immediate revert, `git status` clean before continuing)
rather than copied out: these are a builder's own TDD-style confirmation of
a fix just written (L13's distinction — self-probe, not a panel's
adversarial mutation run against someone else's diff), not the
adversarial-verify pass L5 is scoped to. Fixture directories removed after
use.

### #44 — journal group commit (Cerberus, 2026-08-11)

**Mission outcome. Closed.** A-N3-1's filed follow-up landed on its own
trigger ("before the N4 contract ships"). One fsync per authoritative
core-lock hold instead of one per event: `Journal` splits write from sync,
`Core` carries the hold's open group, and `CoreGuard` — now the only way to
take the core mutex — closes it. Two commits, deliberately separable
(L10): `269179a` is the structural half and changes no durability at all;
`2be980c` moves the fsync and carries the L6 analysis. Gates green
(fmt/clippy/377 tests + 4 opt-in ignored), demo green, no daemon leaks.

**Measured.** `strace -c` on the S1 burst-50 daemon: **1157 → 253
fdatasync** for the same 900 events, i.e. ~23 → ~5 fsyncs per work.
Throughput 43.13 → 44.62 works/s mean of 3 (+3.5%, inside the spread),
p50 unchanged, 18.0 events/work on both sides. Evidence and the honest
negative in `docs/perf/n3-group-commit-2026-08-11.md`: **this measurement
does not claim the N3 regression was recovered on Cerberus** — an fsync
costs ~13 µs on this host, so there was no wall clock in it to win. The
result that matters is that journal cost per work became O(lock holds)
rather than O(events), which is exactly what the pre-N4 trigger was about.

**What was not traded.** No compound event, no event-count change, no
crash window deleted; A-N3-1's rejection of the
`stage.entered`+`execution.reserved` merge stands untouched and `n10`–`n12`
pass unchanged. The L6 obligation was discharged as a *subset* claim — per
append fsync always left "some byte prefix, possibly torn", so grouping
lowers the floor without widening the reachable set — and proven by `n32`,
which truncates a real six-event grouped hold at every byte offset a crash
could stop at (~1400) and requires each to reopen, replay to an exact
prefix, and reconcile fail-closed. Its last case is byte-for-byte `n11`'s.

**Environmental behavior.** Single-agent build under the orchestrator's
scope note, not a full panel — recorded as such. Six mutation probes run in
a disposable git worktree with its own `CARGO_TARGET_DIR` (L5, and CLAUDE.md's
shared-cache hazard): per-append fsync restored, `CoreGuard::drop` neutered,
rotation's group close deleted, `sync`'s poison deleted, a bare `core.lock()`
reintroduced, `recover_tail`'s truncation deleted — **all six killed**, by
the named guards. One process note worth keeping: the first probe round was
invalid and looked clean, because `git checkout -- src tests` in a worktree
detached at the *previous* commit silently reverted the change under test
along with the mutation. A mutation probe must be run against a committed
base, or "survived" means "was never applied".

### N-SERIES CONTAINER CLOSE-OUT — 2026-08-11, v2 measured, Run B, Cerberus handoff

**Mission outcome.** Generator v2 blind-measured (run 3, 2.2M tokens):
**§22.2 MET** — inverts run 2's verdict; recall ≥56.9% in-scope (v1 47.3%),
precision 312/312 at full population, all 3 in-scope v1 silent misses
captured, helper skew collapsed, [S12] closed, review axis found 5/5-real
findings, checkpoint ledger proven resumable (15/21 partitions honestly
pending — Cerberus driver-loop work). v2 delta report: "a genuine
generational improvement, and causally so." Run B (real-Claude, bounded,
$0.68 usage recorded): one genuine 13-turn actor run produced a correct
artifact and three defects — the root refusal of the hardcoded skip flag
(#47), the envelope-less stuck-active seam (#46), and #46's second case:
a live-vs-test discrepancy where observe_envelope's unit-tested
StageCompleted derivation never lands as a settled signal. GP-2 never
fired (correctly — no ask). **#19 stays open, Cerberus-bound**: adapter
evidence, not a completed soak. PR #48 (S-series retro) merged into the
branch: resources/ replaces the zip, CLAUDE.md promotions, the
convergent-evolution probe-gate restoration.

**Environmental behavior.** The Run B operator twice outperformed its
orchestrator: it declined a credential-copy instruction in favor of the
documented IS_SANDBOX=1 fix, and refuted the orchestrator's CONTEXT.md
stall diagnosis by reading the actual file (L12 applied by a subagent to
its coordinator — the loop's protection running in both directions, L9).
Both recorded in the run manifest, plus its own polling spelling bug
honestly reported. Cerberus first acts:
docs/gauntlet/notes/v2-measurement-and-migration-plan.md; #44 and
#46/#47 precede the N4 contract alongside R-N0-3's retention ruling.

---

### N3 — 2026-08-10/11, executor-aware stages + two-phase boundary + GP-2 ask

**Mission outcome: contract met; shipping gate passed; closes #14 (B3's
trigger fired and B3 is closed), #20, #42.** Two-phase external-effect
boundary (reserve under lock / effect outside / verified settle) across
START, SEND, and OBSERVE; §15 prepare/launch with join-returning
stop/interrupt; Claude start-window closed; tagged stage definitions with
content identity, backward-compatible; per-stage actor selection with
whole-workflow preflight (§22.4's matrix); GP-2's actor-initiated ask,
measured on 2.1.226, surviving restart in both directions; §22.5 injection
matrix bound to engine-written prefixes; §22.6 instruments that can fail.
**A-N3-1**: burst-50 budget amended to ≥24 works/s — the +2 events/work
are §22.5-pinned crash windows, not fat; group commit filed as #44 with
the hard pre-N4 trigger. Gate 276/0, demo green, no leaks. Residuals
recorded honestly: instruments blind to per-hold regressions <~200 ms
(perf addendum); the Deferred-Drop backstop leaves a shutdown-race window
(gate finding, info); git-serialization pin catches 22/23 single runs —
L7's repeated-run corollary applies.

**Environmental behavior.** One resumed workflow carried the whole loop
(build→critics→refuters→fixer) + a lean round-2 probe + the pipeline gate:
B1 Sonnet, B2 Opus (which surfaced the budget breach itself and refused
the crash-window-deleting fix — upheld), two Opus critics (18 findings,
18/18 confirmed by refuters — including build-phase errors: Outcome 3
parsed-but-unrouted; SEND under the lock), Opus fixer (12 commits, 19
pins), round-2 critic (7 findings, all mutation-demonstrated: the
unpinned no-substitution fallback, SEND's unpinned §14.5, OBSERVE
invisible to both instruments, a throughput guard measuring a 230×
lighter path), round-2 fixer (8 commits — incl. the fsync *accounting*
instrument where timing was measured inadequate). Pipeline gate found the
deferred-completion leak on commit failure and, on review, its own fix's
missing regression test (L7 enforced on the gate itself). Three-for-three:
every round-2/gate pass found something real the prior pass missed.

---

### S2 — Stabilization (2026-08-10/11)

**Mission outcome: contract met, gate regime green.** All twelve coverage
issues resolved — ten by `Fixes` trailer (#30/#32–#35/#37–#41), two closed
at adjudication with recorded reasons (#31: the fsutil non-WouldBlock arm
is unreachable without a seam — measured: the suite runs as root so DAC
tricks silently pass, and Linux equates EAGAIN with EWOULDBLOCK; #36:
`EngineError::Core`→500 is reachable only by infrastructure fault; the
proposed `segment_max_bytes` seam declined at R1). Three waves, +73
falsifiable tests (221→**294** + 2 opt-in), every pin carrying *executed*
mutation evidence. Coverage by the committed convention: **91.43 →
94.63% lines** (+3.20), regions 90.89→94.06, functions 91.95→93.29, at
close-out tip `dc0d447`. Close-out census: F1 10/10 uninstrumented, F2
3/3 instrumented, zero failures. The CI lane
(`.github/workflows/coverage.yml`) landed after an R-S0-12 review whose
error finding was structural — the drift guard would have failed every CI
run against the committed dev-container fingerprint — plus a
cache-contamination hazard (stale pooled profraws: a silently wrong
number, doctrine 4's exact case); all five amendments applied. Gate:
`--fail-under-lines 90`, 4.63 below measured, per the spread policy.
Wave-3's blind auditor also produced the close-out residual register —
eight declined-with-reason entries appended to the baseline doc, none
silent. One R-S0-7 escalation filed: **#45** (m6 dropped-daemon flake
under load, recurred across two independent sessions; failure shape —
dead pid, surviving descriptor — is possibly the #26 startup-window
class, product-adjacent).

**Environmental behavior.** Three wave workflows + one resume + a config
reviewer, ~3.4M subagent tokens. The instrument protocol *evolved
mid-program and the evolution is the finding*: wave 1 (10 agents, 1.34M)
ran self-probing builders and its panel caught two unfalsifiable tests
green-against-broken-guards plus an L5 deviation and a placeholder
self-report — so wave 2 (7 agents; original run + 100k-token cache-resume
after a container restart recovered five orphaned fixer commits with zero
rework, the M2/M3 precedent) replaced self-probes with builder guard maps
executed by one independent batched prober (37 mutations, 32 kills, 6
survivors — including a mislabeled guard the prober re-sited by probing
both candidates). Wave 3 (3 agents, 440k): Opus auditor + prober (31/28/3)
+ fixer; survivors closed by in-place assertion strengthening, each
mutation re-executed. The wave-2 builders' empirical escalations
(root-DAC bypass, EAGAIN≡EWOULDBLOCK, O_APPEND defeating post-start
sabotage) are recorded environment facts for every future test author.
Owner interventions this milestone: issue↔PR linkage discipline (trailers
+ per-wave PR-body accounting, adopted).

**Adjudication rulings.** (1) #31/#36 sub-items closed-with-reason on
builder-measured evidence; seams declined. (2) The guard-map/independent-
prober protocol is the S-series standard from wave 2 on (L13
operationalized as design). (3) #45 escalated on R-S0-7's recurrence rule
rather than waved off as noise — the failure shape is product-adjacent.
(4) The CI lane's review verdict (amend-first) applied in full; the lane
measures into `runner.temp`, never the frozen evidence tree. (5) Gate 90
recorded in the baseline close-out section; the S1-era "88 against 91.43"
recommendation is superseded there, visibly.

---

### S1 — Coverage Baseline (2026-08-10)

**Mission outcome: contract met, gate regime green.** Measured baseline at
`dc77de9`: **91.43% lines / 90.89% regions / 91.95% functions**, 94/94
profraws merged, zero lost — with the `#[cfg(test)]`-inflation caveat
(18.0% of `src/` lines) stated before the number everywhere it appears.
Phase 1 shipped the instrument in eleven separable commits: three teardown
repairs (SpawnedDaemon SIGTERM-first, reaper TERM-vs-KILL reporting,
demo.sh fail-closed verify-gone), repo hygiene (`.gitattributes` fixing
the majority-Shell language bar, pycache ignores), the CI double-run
trigger fix, the coverage-gap issue template, and the `scripts/coverage/`
harness embedding the R-S0-3 convention with ten measured tool behaviors.
Phase 2 ran C0–C4 + the two-arm census strictly sequentially against the
frozen SHA: F1 10/10 uninstrumented green, F2 3/3 instrumented green — the
M3-era flake class did not reappear. Phase 3: 63 analyzer candidates → 51
confirmed by batched adversarial refutation → **12 issues #30–#41**
(deduped, capped, behavior-named per R-S0-11); 12 refuted with recorded
reasons; residuals (engine.rs function coverage, instrument artifacts)
recorded-not-chased in `docs/coverage/baseline-2026-08-10.md`. Suite
221 + 2 opt-in; every stage's hygiene sweep clean; disk floor never
approached.

**Environmental behavior.** Four workflows + one direct challenger,
~2.2M subagent tokens total: phase-1 builder (1 Opus agent, 294k, 173 tool
uses — measured tool semantics into the harness README, found one flake
under hand-applied contention, recorded 0/7 unreproduced per R-S0-7);
lean round 2 (8 agents: Sonnet spec-fidelity + Opus test-honesty +
Opus invariants/simplicity + batched refuters + fixer, 855k) — **11
findings, 7 confirmed incl. 1 error**: the SpawnedDaemon pin tested
`stop()` while `Drop`, the only path real users take, reverted clean —
Bug Sprint 1's parts-vs-composition shape, caught by fresh eyes after the
builder's self-probe passed (the empirical case for R-S0-12); fixer closed
all 7 in five separable commits pinned by daemon-authored evidence
(descriptor removed, `daemon.stopped` journaled). Analysis gauntlet (10
agents: 5 Sonnet analyzers + 5 batched Opus refuters, 927k) refuted 12 of
63 candidates, several as instrument artifacts (Debug-impl derive
requirements; line-layout artifacts on wrapped `?`). Owner interventions
shaped the process three times mid-milestone — proper-gauntlet challenge
(→ R-S0-12), doc-surface correction (→ model spread revised into
gauntlet-pattern.md, CLAUDE.md refers), evidence-over-memory (→ L12) —
all recorded as rulings/lessons, none silent.

**Adjudication rulings.** (1) Round-2 error fixed with composition pins on
daemon-authored evidence, not reaper self-reports. (2) The harness's
profraw accounting gained a fail-on-unaccounted-loss check (R-S0-6's
natural reading); F2's legitimate cleans are declared, not exempted.
(3) engine.rs's refuted candidates stand as residuals for S2 intake below
the issue cap — the number is recorded, the gap list is not padded.
(4) Real-Claude live-path regions handed to N2 per R-N0-6, named in the
baseline. (5) S2 gate recommendation: `--fail-under-lines 88` (nim-proxy
spread below measured 91.43), to be re-set against S2's own close.

---

### S0 — S-Series Kickoff Adjudication (2026-08-10)

**Mission outcome: contract met — adjudication-only, zero code delta.**
The S-series proposal (coverage baseline → test-only stabilization →
non-blocking CI lane; owner directions: prototyping speed over ceremony,
minor-blocking fixes roll in, breaking changes discussed, S-naming,
workflow `.js` in `resources/`, autonomous run) was challenged by one
fresh Opus panel: 35 findings, 9 error / 17 warning / 9 info (6
verified-credit), all ruled; proposal amended in place before the S1
contract was drawn. Rulings R-S0-1..11 in `docs/gauntlet/contracts/S0.md`;
challenge dispositions in `docs/gauntlet/notes/s0-adjudication.md`.
Post-launch addenda from owner challenges: **R-S0-12** (code is code —
any executable diff takes the full multi-axis loop; the P1-PERF
single-builder exemption covers only phases that write no code) and
**R-S0-13** (model spread: Sonnet executes contracts, Opus judges
outcomes, Fable is the one orchestrator seat and never fans out —
supersedes the M-era model table, revised into gauntlet-pattern.md dated).

**Environmental behavior.** One challenger (132k tokens, ~13 min) whose
headline error (A1: m3/m5 misclassified as subprocess-free) corrected the
orchestrator's own pre-proposal assessment on line-cited evidence. Two
challenger claims about cargo-llvm-cov defaults were deliberately not
adopted — held as Unknowns for the S1 builder to measure (L1 at the tool
boundary), and both measurements later proved load-bearing (the
`#[cfg(test)]` inclusion and the stale-report hazard). The orchestrator's
process errors this milestone (single-builder launch; doctrine copied
instead of referred; edit target from memory) were owner-caught and are
now R-S0-12 + LESSONS L12/L13 — the loop's protection against a wrong
orchestrator is the same as against wrong code: fresh eyes and the record.

---

### N2 — 2026-08-10, actor-only repo-to-icm: built, run, blind-measured

**Mission outcome: contract met — the workflow exists, ran through the real
engine, and the measurement returned a verdict.** The verdict:
**§22.2 NOT met** — generator v1 silently missed 11 consequence-class
behaviors within its covered scope; recall 47.3% in-scope, precision clean
(zero invention, every quote verbatim at the pinned SHA), representation
skewed ~9× toward shared-helper vs the reference's judgment tiers.
Scorecard + grammar-pressure report in `docs/gauntlet/runs/n2-run2/`.
Grammar pressure adjudicated: **GP-2 (actor-initiated mid-run ask) is the
sole confirmed engine gap licensing Program B scope**; the volume wall
(16/136 files per single-actor turn) is real but its engine claim was
rejected — partitioned harvest stages and intra-stage iteration are untried
lower rungs; §21.8 composition trigger NOT fired. Two resilience results
measured incidentally: run 1 propagated a setup ambiguity fail-closed
through all 10 stages with zero invention; run 2's daemon restart correctly
blocked a stale fake execution (recovery invariant confirmed at GP-4).
Defect #29 filed (finalize.py deleted a never-committed evidence file).
U1 measured (needs_input/respond holds and resumes the same execution);
U3 answered (dispositions + finalize executed on a real work branch).

**Environmental behavior.** Build workflow (7 agents: measure-first U1,
3 Sonnet builders, validator, 2 blind Opus critics — 31 findings, 7 error),
Sonnet fix round (30/31 closed, one measured doc-vs-engine correction),
two measurement runs (12 agents each; run-2 resumed once after a wedged
harvest actor — workflow-runner failure, not engine), comparison (3 Sonnet
comparers + Opus adjudicator; the adjudicator overturned C1's
hash-integrity claim on re-verification and confirmed the rest). ~4.2M
subagent tokens this milestone. Orchestrator error recorded: run 1's
intent named no subject/revision and omitted UPSTREAM.md — the workflow's
fail-closed discipline caught it end-to-end. Scripts archived
(n2-build/-run/-run2/-compare).

---

### N1 — 2026-08-10, ICM convention + adjudicated Sergeant reference decomposition

**Mission outcome: contract met; corpus frozen at version 1** (`reference-corpus/FROZEN.md`).
Convention docs (`docs/icm/`) carry the D9 four-layer ICM model (owner-directed
mid-milestone: L1 orientation / L2 stage contract with Inputs table / L3
stable references / L4 per-run outputs with declared promote|evidence
disposition + deterministic finalize). Corpus: 979 behavior units (966
extracted from 139 decompose-files + 13 adjudication additions/splits), every
quote script-verified against its source span, 4 citations honestly marked
disputed; 34 draft workflow packages / 116 stages; 20 conflicts ADJUDICATED;
4 surviving engine-pressure claims. Gate items all evidenced: per-file
dispositions (source-inventory), two independent reviewers (boundary-honesty
+ completeness lenses, Opus), disagreements preserved (classification-ledger
+ adjudication-round1.md), structural lint committed and green (`lint.py`,
orchestrator-re-run at freeze).

**Environmental behavior.** Three workflows, 37 agents, ~8.4M subagent
tokens, ~7h wall (2-wide concurrency ceiling): decomposition (17 agents —
Sonnet extraction/drafting, Opus synthesis + 2 blind reviewers), fixer (15 —
Sonnet fixers, Opus verifier), closure (5 Sonnet). The panel structure paid
for itself twice: round-1 refute returned 21 findings (5 error) including
44% of stages being machinery promoted via a future-engine justification
(A4 demoted 71; N2's measurement would have been graded against over-staged
gold) and the unenforceable-hash finding that became L11; the fixer round's
Opus verifier then caught 13 closure gaps including a skipped package and a
ruling written up but never applied to the record (V2) — fixed and re-linted.
Orchestrator errors recorded: the parallel convention-doc agent's enum
contradicted the extraction vocabulary (A1 — spec doc amended, corpus stood);
adjudication A12 said "route the new units into provenance" without naming an
owner, and no fixer owned it (V3). Scripts archived in
`reference/gauntlet-workflows.zip` (n1-decomposition, n1-fixer, n1-closure).
Unknowns resolved: U1 966 base units (vs 150–400 scoping estimate); U2
no-mistakes decomposed without composition machinery (validate-and-ship, 9
stages post-A5); U3 sergeant-setup's interactivity fit needs_input — its
engine-gap claim (G5) was *rejected*: the lower rung already shipped.

---

### N0 — 2026-08-10, Next Iteration kickoff: proposal accepted, remediation adjudicated

**Mission outcome: contract met — adjudication-only, zero code delta.** The
successor proposal (`reference/proposal-next-iteration-icm-workflows.md`,
audited against 27c00ef, delivered via the owner's IdeaOS corpus) is accepted
as the governing document for the N-series with one owner-accepted amendment:
retention (#17/#4) may no longer be silently parked — the N4 contract cannot
be written without a retention design ruling (R-N0-3). Full rulings
R-N0-1…R-N0-7 in `docs/gauntlet/contracts/N0.md`: #14/B3 and #20 fold into
N3's two-phase-boundary scope and gates (their registered triggers fire
there); #6/#7/#10 become regression budgets, not blockers; #18 → N5;
#19 narrows into N2's real-Claude measurement run. Claude CLI 2.1.226 is
confirmed as the measured floor (`MIN_TRUSTED_VERSION` already says so — M4
re-measured on this exact version; owner ruling: older versions stay refused
as unmeasured). Deviation-register scope extends to the successor proposal
from this entry forward.

**Environment evidence for Program B feasibility (measured this session):**
`dockerd` 29.3.1 runs in this container (vfs, `--bridge=none
--iptables=false`); registry pulls are egress-blocked, but a locally-built
`FROM scratch` static image completes the full contract lifecycle — build,
run, bind-mount read *and* write, `--network=none`, exact cleanup. §22.7's
matrix is therefore executable here with Sergeant-owned probe images; only
cold-pull/digest-pull/registry-auth tests need a qualified host. N5 platform
qualification runs on the owner's macOS/Windows machines later, by design.

**Environmental behavior.** Orchestrator-only (no panel): N0 produced
rulings, not artifacts a blind panel could grade — the rulings themselves
remain reviewable findings (L9) and every later contract that consumes one
(N3, N4, N5, N2) re-exposes it to fresh critics at its own gate.

---

### BUG SPRINT 1 — 2026-08-10, issues #3 #5 #9 #24

**Mission outcome: all four fixed, pinned, landed as four separable commits
(L10)** — 6167f4c (#5 surface-root removal, the syscall as the guard),
f91d95c (#9 terminal-surface sweep on restart, re-derivation over compound
per L6, `recovered: true` provenance), dc157e5 (#24 warn-once on an
unapplied timeout override), de193a2 (#3 pty-death exit + bounded reader
join). Gates 218/0 orchestrator-verified, zero leaked daemons, zero /tmp
residue. Issues auto-close on merge via Fixes-tags.

**The measurement that shaped #3 (L1 vindicated again):** the ~80% spin
lives *inside* crossterm 0.29 — its unix event source treats a
zero-length pty read as "no data yet" and loops, so `event::poll` never
returns after hangup and no caller-side error handling can see it. The
fix therefore guards *before* handing the fd to the reader (open
/dev/tty, ENXIO discriminates hangup from transient errors — measured),
adds a 500 ms watch arm that ends the session (orphan now self-exits in
~1 s, was SIGKILL-only), and bounds the join so the *class* — not just
the instance — can never wedge the process again.

**Environmental behavior.** Lean gauntlet, 5 agents (~764k tokens, ~109
min): Opus fixer, Fable invariants + Opus test-honesty critics, batched
refuter, Opus round-2 fixer. Panel earned its keep: 6 findings, all 6
confirmed, headlined by a test-honesty **error** — round 1 pinned #3's
*parts* but not their *composition* (a two-line mutation removing the
guard's wiring and the bounded join reproduced the wedge with every test
green). Round 2 pinned composition (the tick the reader actually
receives, the watch arm ending the loop, run() mapping TerminalGone to
exit 0, the unbounded join failing the suite), ENXIO discrimination, the
printed (not merely computed) #24 warning, and the canceled-work sweep
case. One residual startup-window hangup edge → issue #26, not silence.
Ruling: pre-fix leaked surface roots stay (GC belongs to #17) — the
sweep is journal-keyed and does not delete directories no event points
at; correctly declined by the fixer.

**Method note (shared-cache hazard, third bite):** probe copies sharing
the main checkout's CARGO_TARGET_DIR overwrite its binary slots — three
early "fixed binary" pty measurements were silently measuring a probe
build until the fixer caught it and rebuilt. Standing rule for probe
prompts: after any probe-copy build, rebuild the main checkout before
measuring its binary.

---

### BACKLOG CONSOLIDATION — 2026-08-10

Everything outstanding from the build cycle now lives in one place: the
GitHub issue tracker (#3–#25). P1-PERF filed #3–#13 (measured findings);
this sweep added #14–#25 from the development record — the registered
backlog rows (B3→#14, B2→#15), the accepted-with-ruling M6 warts (TUI
reconnect #16, timeout-knob silence #24), the structural debts named at
P0 close-out (retention/GC #17, /proc portability #18, doctor disk check
#23), the coverage gaps (real-Claude soak #19, crash-point injection #20,
dashboard JS #21, workspace edge cases #22), and the D6 Codex descope
(#25, blocked on a measurable environment). Deliberately NOT filed: B1
(snapshot identity binding — dormant by design, its trigger is itself a
design guard; it stays a ledger row), the L10 commit-hygiene rule
(process, binding via LESSONS, not work), and §38's OpenCode/Prime/MCP
deferrals (roadmap without current intent — the proposal already records
them). The backlog rows in this file now carry their issue numbers as the
live tracking surface; the ledger remains the record of *why*.

---

### P1-PERF — 2026-08-10, load/stress baseline + issue backlog

**Mission outcome: contract met.** The full S1–S7 matrix ran at contract
scale against the release binary at 499c061; every cell filled, every
scenario's hygiene sweep clean. Deliverables: the rerunnable harness
(`scripts/perf/`, commit 348623b — including the measured SGT_FAKE_SCRIPT
semantics: one global FIFO of steps, popped per execution-start and per
input-send), the baseline document (`docs/perf/baseline-2026-08-10.md`),
and **eleven GitHub issues (#3–#13)** — the phase's stated product. Nothing
was fixed, per contract. Headline numbers: idle 28.6 MB / 15 fds / 9
threads; submission plateau 28–39 works/s independent of concurrency
(daemon saturates first — Unknown #2 resolved); churn cost ~25 kB RSS per
work, never reclaimed (#4); graph reads 9.7 ms p50 at 1k-event depth;
rebuild holds the §21 budget with growing margin (14.6k–29.2k events/s at
10k–50k); kill -9 ×3 recovery: zero lost/illegal/duplicated/orphaned, and
the command-ID crash-window index positively confirmed under ambiguous
client retry. The seeded TUI orphan repro upgraded on measurement from
"needs SIGKILL" to "wedges spinning ~80% CPU" — critical, #3.

**Environmental behavior.** One workflow: Opus harness builder (smoke-
tested all seven scenarios before handoff), seven Sonnet runners strictly
sequential to keep measurement windows unshared, two batched Opus
verifiers; 10 agents, 1.30M tokens, ~108 min. 22 findings → 21 confirmed
on independent reproduction, 1 refuted (S4's claimed 35% submit-latency
rise under SSE subscribers — run-to-run noise). Runners honestly filed
harness defects against their own instrument (#13) and flagged the
workflow brief's script-name drift (s3-graph/s5-scale vs the committed
s3-deep/s5-journal) instead of papering over it — they followed the
committed harness, correctly. The verification pass measured one finding
as WORSE than claimed (TUI orphan: pre-signal CPU spin) and corrected one
severity down (S1 surface-dir leak major→minor per-instance).

**Adjudication rulings.** (1) The empty `surfaces/<work-id>/` leak
recurred in all seven scenarios — deduplicated to one issue (#5), minor
per-instance, unbounded in aggregate; its connection to the S6
completion-tail crash window (#9: a torn-down surface whose teardown
event is lost) is noted in both issues. (2) The S4 refutation stands;
recorded in the baseline doc. (3) The throughput plateau (#6) is ruled an
observation, not a defect — the single-writer journal invariant predicts
it — and is linked to B3 as the second measured symptom of core-lock
serialization. (4) The TUI column collision (#11) and doctor floor (#12)
are intake for the planned usability phase, not P1 work. (5) The no-fix
rule held: zero product or harness lines changed after the contract
commit; harness gaps went to #13. (6) Positive results are recorded with
the same weight as defects: crash idempotency, §21 rebuild margin, SSE
fd-per-subscriber exactness, TUI 0.03% idle CPU.

---

### P0 CLOSE-OUT — 2026-08-09, prototype complete

The §38 P0 vertical slice is done: seven gauntlet units (M0–M6), one crate,
one binary. `sgt run "<intent>"` drives durable Work through worktree
surfaces, a staged workflow engine, and the measured Claude adapter, with
the complete trajectory in an append-only journal that every projection —
in-memory, DuckDB analytics, graph, TUI, dashboard — rebuilds from. The §39
walkthrough runs end-to-end as `scripts/demo.sh` (exit 0, evidence pointers
verified by test t4).

**Totals.** 203 tests + 2 opt-in live-Claude (suite: 78 unit, m1 10, m2 22,
m3 24, m4 40, m5 17, m6 12); gates `fmt --check` / `clippy --all-targets
-D warnings` / `test` green at every milestone close; release binary 58.8MB
(47.3MB stripped) with embedded DuckDB, owner-accepted. Confirmed-finding
series across milestones: 13 → 15/19 → 22/19 → 7/12 → 13 (round-1/round-2
where two rounds ran) — panels kept finding real defects to the end (M6
round 2: two errors post-checkpoint), which is the argument for the loop.

**Deviations D1–D8, disposition.** All registered with rungs, none silent:
D1 naming (owner), D2 headless print-mode turns over held-attach (measured,
confirmed at M4), D3 backend stubs deferred to their milestones, D4 lib+bin
layout for testability, D5 M2 dependency narrowing, D6 Codex descoped to a
doc-stub until measurable (owner), D7 direct OTel crates over the tracing
bridge, D8 crossterm via ratatui's re-export. The register did its job
twice over: critics caught unregistered deltas (D5, D8) and the
contract-amendment trap (D6) proved critics grade the amended contract.

**Backlog.** B1 (snapshot identity binding) — dormant, trigger never fired,
rebuild-on-start measured fast enough that snapshot loading never entered
the daemon path. B2 (dashboard cookie handoff) — post-P0, trigger is any
non-loopback binding.

**LESSONS index (L1–L10).** L1 measure the Claude CLI, never trust docs;
L2 headless driving is proven production practice; L3 point fresh critics
at the register; L4 axis-tension means adjudicate, not loop; L5 enforce
probe hygiene structurally; L6 adjacent-append crash windows recur; L7
fixes need pinning tests (revert-probe); L8 capability flags need contract
tests; L9 orchestrator rulings are findings too; L10 milestone-squashed
commits defeat the revert-probe audit.

**What P0 does not do** (deferred by contract, not omitted silently):
Codex/OpenCode/Prime adapters (D6/D3), MCP, non-loopback auth, graph
rendering (JSON endpoint only), charts. The §15 trait, §8 API, and journal
schema are the extension points the proposal intends.

---

### M6 — TUI, Dashboard, Doctor, §39 Demo (2026-08-09)

**Mission outcome: contract met, gates green.** Shipped: the ratatui TUI
(bare `sgt` opens fleet + work detail, SSE-live with a durable liveness
indicator — `live` / `TAIL CLOSED (r reconnects)` — that command outcomes
cannot overwrite; SIGTERM/SIGHUP restore the terminal; cancel/respond
keybinds); the embedded server-rendered dashboard (`include_str!` assets,
EventSource live updates, `/ui` behind the same `require_bearer` middleware
as `/v1` with one token-extraction rule and one 401/405 vocabulary; `sgt
web` prints/opens the tokenized URL); `sgt doctor` (git, claude CLI +
version-gate verdict, data dir + journal validation, projection rebuild
health, daemon liveness/descriptor; human + stable `--json`, every check
names its remedy, exit code reflects health); and `scripts/demo.sh` — the
§39 walkthrough in a temp repo, fake-backend deterministic, narrating both
stages including the second (review) execution, exiting 0 with evidence
pointers t4 re-resolves against the kept journal. Clients-are-equal is
enforced structurally: tui.rs/web.rs reach state only via `ApiClient`/
`ApiViews`, and t5 pins the `ApiViews` public-method set so widening the
surface fails the test. Evidence: 203 tests + 2 opt-in, zero leaked
daemons and zero /tmp residue after a full suite run (orchestrator-
verified with non-self-matching patterns).

**Environmental behavior.** Build + round 1 in one workflow: 14 findings,
13 confirmed, fixed at checkpoint 69cb52e; checkpoint gate passed (doc
commit e972e64 adopted). Round 2 (lean, 11 agents): 17 panel findings + 1
orchestrator-seeded, refuters killed 4, **13 confirmed — 2 errors after a
passed checkpoint**: (1) t5 was defeatable by widening `ApiViews` with a
non-endpoint method — the guard pinned "whatever methods exist", weaker
than the doc's compile-error claim; probe-proven, now pinned by a
self-tested scanner; (2) the seeded cross-suite daemon leak (measured ~89
accumulated, then 1 per clean run). Round 2 also found by forensics what
no axis was assigned: 243 leaked /tmp dirs traced to `ClaudeBackend::stop`
returning before the turn's evidence archive landed — an M4 latent defect
surfaced by an M6 hygiene sweep. The checkpoint-adopted doc commit e972e64
itself contained false specifics (test-inventory claims), confirmed and
corrected by closing the gaps it papered over, not rewording. Fixer: one
iteration, all 13 closed, every new instrument itself tested (reaper,
surface scanner, stop-join pin all probe-verified to fail when their
subject regresses).

**Adjudication rulings.** (1) D8 registered (crossterm narrowing, D5
precedent). (2) Token-in-URL accepted at R1 for loopback P0; recorded as
backlog B2 with the cookie-handoff alternative — this is the record the
contract's Unknown pointed at; api.rs's premature "recorded in the ledger"
claim was made honest in round 2 and is true as of this entry. (3) A doc
adopted by a checkpoint gate is a claim like any other — e972e64's false
inventory confirmed as a finding; the correction closes gaps rather than
rewording (fleet-row projection now unit-tested field by field). (4) L7's
revert-probe was unperformable on the squashed 69cb52e — mutation probes
accepted as this round's substitute, and the process defect is now LESSONS
L10. (5) `POST /ui` unauthenticated now 401 (was 405): gate-before-router
chosen to match `/v1`; uncontracted behavior change made visible in t2's
assertions. (6) Daemon-leak and temp-dir-leak fixes are structural (the
`DataDir` guard's `&DataDir`-typed helpers make an unreaped spawn a type
error), per L5's "the environment enforces the boundary".

**Shipping gates.** Checkpoint gate (pre-round-2): passed, e972e64
adopted. Final shipping gate: review found 1 — `ClaudeBackend::stop`'s join
of the reader thread ran on the async caller's tokio worker, so the archive
wait could starve that worker's other tasks; fixed with `block_in_place` on
a multi-thread runtime, pipeline-applied (d82a6e2), with the accepted
Core-lock-held-during-join trade-off documented on `stop` itself (d20554d).
Test/lint clean. Post-gate addendum (orchestrator): the lock-hold ruling is
registered as backlog B3 with its trigger — the review's three rounds
(blocking join → `block_in_place` → doc-only concession) were an L4 axis
tension between STOP's evidence promise and core-lock responsiveness, and
the fixer's "accepted trade-off" comment was ratified as a ruling, not
accepted as one. A second parked finding — `source-content-only-tests`,
against the crossterm-narrowing and timeout-knob-name pins — was declined
on the M5 final-gate precedent (L4: third pass at the axis): both
properties are inherently structural (a dependency-manifest shape; a
cross-file env-name agreement), and their runtime observables live behind
the opt-in real-Claude path. Orchestrator post-gate verification: 203/203,
zero leaked daemons, zero /tmp residue.

---

### PAUSE MARKER — 2026-08-09, M6 mid-gauntlet (planned, owner-directed)

State at pause: M6 build + round-1 panel complete (14 findings, 13 confirmed,
all fixed; 195 tests green, verified post-everything); checkpoint commit
69cb52e + checkpoint gate **passed** (doc commit e972e64 adopted). The M5/M6
"transient" test failures are SOLVED and were not transient: the ENOSPC-era
shared CARGO_TARGET_DIR let pipeline worktree builds write test binaries
(with worktree paths baked into `env!(CARGO_MANIFEST_DIR)`) into the cache
our runs then reused — failures appeared after gate runs and "self-cleared"
whenever an edit forced local recompilation. Fixed: gate.sh now gives the
pipeline a private persistent cache (/root/.no-mistakes/cargo-target — warm
across its runs, isolated from ours); forced local rebuild verified 195/0.

**Resume checklist (M6 lean round 2 + P0 close-out):**
1. Lean round-2 panel over the M6 layer (adapt m5-round2-lean.js; diff base
   3a70148). Seed one hygiene finding: repeated suite runs leaked ~89
   spawned `sgt` daemons — tests that auto-spawn must reap their children
   (kill-on-drop guard or teardown sweep); also note the first pipeline run
   under the new private cache pays one cold duckdb build (~10 min) — gate
   step budgets should expect it once.
2. Adjudication + M6 MARK & LOG.
3. P0 close-out: cross-milestone ledger entry (totals M0–M6, D1–D7 summary,
   backlog disposition, LESSONS index), README update to real usage, PR #1
   body refresh as the deliverable's front door.
4. Final shipping gate, push, done.

---

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
duckdb cold-build wall; measured and recorded). Final gate: **passed** —
review found 2: (1) analytics catch-up TOCTOU — a concurrent `catch_up`
failure landing in `with_analytics`'s lock-release window could fold a stale
journal tail — fixed with retry-on-mismatch plus a provoking regression
test, pipeline-applied (eaa6845, the 178th test, orchestrator-verified
green post-recovery); (2) source-scan tests flagged as anti-pattern —
**declined by orchestrator ruling**: third pass at the same axis tension,
and the round-2 adjudication stands on mutation-probe evidence the finding
did not engage (L4: rule, don't re-loop). Test/lint approved on the
checkpoint gate's static-review-plus-orchestrator-verification basis.
Environmental note: in the gate's post-outcome window, t2/t5 transiently
failed directory walks with NotFound and self-cleared once pipeline
activity stopped (17/17 serial, 178/178 parallel afterward) — recorded as
an unexplained shared-environment interaction, watched for recurrence.

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
