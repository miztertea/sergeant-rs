# Environment fact files

One file per named execution environment this repo runs in. **An environment
fact is a measured property of a host** (capabilities, quotas, network
posture, uid, filesystem semantics) — never assumed, never inherited from
another environment's file, and kept separate from repo invariants (which
live in `docs/DEVELOPMENT.md` because they hold everywhere the code builds).

Rules:

- Facts carry their **measurement**: date, how measured, evidence pointer
  (run ID, journal path, command output). An undated fact is a rumor.
- Sessions **re-measure cheaply on wake** where a probe exists rather than
  trusting a stale file; the record holds the last measurement, not eternal
  truth. Run `scripts/probe-env.sh` once at session start on any host and
  paste its table into that host's record — as of Phase 4 step 4 (ADR 0014
  decision 18), that's `knowledge/evidence/host-measurements/<host>.md` in
  `sergeant-rs-workspace`, not a file in this directory. It emits exactly
  this format, dated, never a blank or assumed cell.
- This directory keeps the *capability* — `scripts/probe-env.sh` and the
  rule below — and one file per known host with a short pointer to where
  that host's dated measurements now live. It is not itself where
  measurements are recorded anymore.
- Test fixtures asserting environment facts must **probe-gate** (skip
  loudly where the fact doesn't hold) — the two-environment matrix in
  docs/DEVELOPMENT.md is the root-container/GH-runner instance of this general rule;
  every environment added here widens that matrix.
- Known environments: `claude-code-cloud.md` (this container class),
  `github-runner.md` (CI), and — anticipated — Cerberus, Hades, MacBook,
  each added the first time a session runs there, by measurement.
