# Claude Code cloud container (remote sessions on this repo)

Facts measured 2026-08-09 → 2026-08-11 across the M/N/S-series sessions.
Ephemeral container class: resets without warning; everything below can
drift on image updates — re-measure on suspicion, and date any change.

**Measurement table moved.** The dated measurement table previously here
now lives in `sergeant-rs-workspace`'s knowledge library, at
`knowledge/evidence/host-measurements/claude-code-cloud.md` (ADR 0014
decision 18: the capability — `scripts/probe-env.sh` and the rule that a
host is measured before it is trusted — stays with the product; the
measurements themselves are workspace-shaped).

Repo invariants (target size, build times, test counts) intentionally NOT
here — they live in `docs/DEVELOPMENT.md`. GH-runner facts: `github-runner.md`.
