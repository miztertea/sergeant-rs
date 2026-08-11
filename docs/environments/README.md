# Environment fact files

One file per named execution environment this repo runs in. **An environment
fact is a measured property of a host** (capabilities, quotas, network
posture, uid, filesystem semantics) — never assumed, never inherited from
another environment's file, and kept separate from repo invariants (which
live in CLAUDE.md because they hold everywhere the code builds).

Rules:

- Facts carry their **measurement**: date, how measured, evidence pointer
  (run ID, journal path, command output). An undated fact is a rumor.
- Sessions **re-measure cheaply on wake** where a probe exists rather than
  trusting a stale file; the file records the last measurement, not eternal
  truth. Run `scripts/probe-env.sh` once at session start on any host and
  paste its table into that host's file here before doing anything else
  (see the N-series retro, item 1) — it emits exactly this format, dated,
  never a blank or assumed cell.
- Test fixtures asserting environment facts must **probe-gate** (skip
  loudly where the fact doesn't hold) — the two-environment matrix in
  CLAUDE.md is the root-container/GH-runner instance of this general rule;
  every environment added here widens that matrix.
- Known environments: `claude-code-cloud.md` (this container class),
  `github-runner.md` (CI), and — anticipated — Cerberus, Hades, MacBook,
  each added the first time a session runs there, by measurement.
