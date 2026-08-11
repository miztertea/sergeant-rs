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

---

## Ledger entries

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
