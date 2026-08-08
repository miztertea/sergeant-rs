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
| D3 | §35 lists `backend/{claude,codex,opencode,prime}.rs` | Scaffold has `backend/{claude,codex,fake}.rs` | §38 defers OpenCode/Prime past the P0 contract proof (R1: doesn't need to exist yet); §37's deterministic core tests require a fake backend (R7: no lower rung supplies a deterministic in-process backend). Modules are added when their milestone arrives, not pre-declared. Raised by the M0 critic panel. |

## Backlog (confirmed-but-deferred findings)

(none yet)

---

## Ledger entries

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
