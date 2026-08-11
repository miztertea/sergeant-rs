# Claude Code cloud container (remote sessions on this repo)

Facts measured 2026-08-09 → 2026-08-11 across the M/N/S-series sessions.
Ephemeral container class: resets without warning; everything below can
drift on image updates — re-measure on suspicion, and date any change.

| Fact | Measured value | Evidence |
|---|---|---|
| uid | root (uid 0) | Run B attempt 1, 2026-08-11 |
| `claude --dangerously-skip-permissions` | **refused under root** ("cannot be used with root/sudo privileges") | Run B attempt-1 journal, `docs/gauntlet/runs/runB/attempt-1/` |
| Real claude turns | require `IS_SANDBOX=1` in the spawning daemon's env | Run B attempt 2 (working turn); `src/backend/claude.rs` module docs |
| Writable disk allowance | ~40 GB per session; at quota, writes fail but **deletes still work** | two ENOSPC events 2026-08-10/11; freed via foreign `target/` + both `incremental/` dirs |
| Outbound network | HTTPS via agent proxy; **GitHub release-API/asset fetches 403** (source builds work) | no-mistakes install, M1 + S-series retro |
| DAC / permission-bit fixtures | silently pass (root ignores mode bits); `CAP_LINUX_IMMUTABLE` present; O_DIRECT alignment enforced on the scratch FS, refused on tmpfs | S2 fixture matrix (GAUNTLET S2 entry, issue #31); probe-gates in `tests/m1_event_core.rs`/`m3_execution.rs` |
| Container resets | wipe installed tools and `target/` (~10 min DuckDB rebuild); repo restored from remote is the only safe state | three resets in one S-series day; CLAUDE.md ops section |
| Claude CLI | v2.1.226 at last measurement; auth present for the owner's account; CLI default model for unpinned turns was sonnet-5 | M4 gate; Run B usage capture |

Repo invariants (target size, build times, test counts) intentionally NOT
here — they live in CLAUDE.md. GH-runner facts: `github-runner.md`.
