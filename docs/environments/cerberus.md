# Cerberus (owner's home server, cerberus.what-it.be on the local network)

Facts measured 2026-08-11, first session on this host (N-series close-out
handoff). Persistent host — no container-reset hazard — but facts still
drift on OS/tool updates; re-measure on suspicion and date any change.

**Measurement table moved.** The dated per-host measurement table
previously here now lives in `sergeant-rs-workspace`'s knowledge library,
at `knowledge/evidence/host-measurements/cerberus.md` (ADR 0014 decision
18: the capability — `scripts/probe-env.sh` and the rule that a host is
measured before it is trusted — stays with the product; the measurements
themselves are workspace-shaped). See also
`knowledge/views/host-wing/index.md` for the scoped-retrieval entry point
into all four hosts.

**Host convention — the inbox (owner-declared 2026-08-11):** `~/inbox/`
on this host is where the owner drops files for the repo (proposals,
research). A file is deleted from the inbox once accepted/vendored into
the repo — the inbox holds ONLY not-yet-accepted material. Check it on
wake.

Repo invariants (target size, build times, test counts) intentionally NOT
here — they live in `docs/DEVELOPMENT.md`. Sibling files: `claude-code-cloud.md`,
`github-runner.md`.
