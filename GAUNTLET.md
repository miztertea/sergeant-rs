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
| D2 | Claude adapter drives a held `attach` (per Sergeant's tmux design) | Daemon has no TTY/pane. Leading M4 candidate (2026-08-08): headless turn sequence — `claude -p --output-format stream-json` with prompt on stdin, `--resume <session_id>` per later turn; session identity is durable, process exists per turn. Proven in production by no-mistakes (`internal/agent/claude.go`, vendored knowledge in LESSONS L2). The spike's rejection of `-p --resume` was Sergeant-specific (live-`--bg` refusal + persistent-TTY doctrine), neither applies here. To be confirmed by M4 contract tests (R5: installed harness capability; interrupt/restart/concurrency semantics still unmeasured). Fallbacks: SessionStart-hook injection, `--bg` + stop→resume. | Measured spike facts + no tmux in scope; see M4 contract Unknowns |
| D4 | §35's tree has `src/main.rs` owning all modules, no lib target | `src/lib.rs` declares the module tree; `main.rs` is a thin shell over `sergeant_rs::cli` | M1 contract requires the core "as library code with tests": integration tests under `tests/` can only import a lib target, and a bin-only crate forces dead-code suppressions under `clippy -D warnings` (R2: the change reuses Cargo's native lib+bin layout). Raised by M1 critics rounds 3–4; ruled authorized at M1 adjudication. |
| D3 | §35 lists `backend/{claude,codex,opencode,prime}.rs` | Scaffold has `backend/{claude,codex,fake}.rs` | §38 defers OpenCode/Prime past the P0 contract proof (R1: doesn't need to exist yet); §37's deterministic core tests require a fake backend (R7: no lower rung supplies a deterministic in-process backend). Modules are added when their milestone arrives, not pre-declared. Raised by the M0 critic panel. |

## Backlog (confirmed-but-deferred findings)

| # | From | Finding | Why deferred |
|---|---|---|---|
| B1 | M1 adjudication | A foreign snapshot whose `last_seq` is within the journal's range loads undetected (identity binding was removed as beyond-contract machinery) | Snapshots live in the daemon-owned data dir; the threat is operator error, not adversarial. Revisit when M2's daemon owns the data dir end-to-end or if snapshots ever travel. |

---

## Ledger entries

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
