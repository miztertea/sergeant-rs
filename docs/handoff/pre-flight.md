# Pre-flight checklist

For any session about to run this repo's full trip on a host it hasn't
gated before — the MacBook today, per ADR 0001 (D1) a Hades/WSL2 host is
the next measured target, and both point here rather than duplicating this
file. If you're on the Mac specifically, `docs/handoff/path-to-mac.md`
links to this file and adds what's specific to that trip; read it too.

**Run `scripts/probe-env.sh` first** (`docs/DEVELOPMENT.md`'s "Environments"
section — it's the session-start convention on any host, not something this
file replaces). Most of what a pre-flight would check, the probe already
measures and prints as a dated table. **This checklist refers to those rows
instead of restating them**, and only adds its own checks for what the probe
deliberately does not cover: `gh` auth, Docker's live runtime lifecycle (the
probe checks the daemon answers, not that it can actually run a container
end to end), and a couple of resolution gaps — crates.io reachability and
whether measured disk headroom is *enough* for this repo specifically, not
just a raw number.

Every row below is a command with a checkable expected answer, and is
marked as a **hard stop** (do not proceed past it) or a **degrade**
(record it and continue — the trip still completes, just with a gap).

| Check | Command | Expected answer | Hard stop or degrade |
|---|---|---|---|
| Rust toolchain | `PATH="$HOME/.cargo/bin:$PATH" bash scripts/probe-env.sh` → read the `cargo` / `rustc` rows | Both show a real version, not "absent" | **Hard stop.** Nothing in `docs/DEVELOPMENT.md`'s command table runs without them; install via rustup and re-run the probe before continuing. |
| git | `git --version` | Prints a version | **Hard stop.** Not measured by `scripts/probe-env.sh` (no row for it) — you can't check out the handed branch without it. |
| Docker: daemon reachable **right now** | `docker info >/dev/null 2>&1 && echo REACHABLE \|\| echo NOT_REACHABLE` (also see the probe's "Docker: presence / daemon / storage driver / cgroup / group" row) | `REACHABLE` | **Hard stop**, and re-run this exact check again immediately before the suite (step 4 of `path-to-mac.md`), not just at session start — Docker Desktop's state can drift across the ~10-minute cold build. Installed-but-stopped is the dangerous case: it does not error, it makes every Docker-gated test skip. Six suites depend on it — `m2`, `m3`, `m4`, `m6`, `m7`, `m8` (`grep -l -i docker tests/*.rs` also turns up `tests/m9_watch.rs`, but its only hit is the string literal `"DockerBackend"` in a backend-name assertion, not a Docker-gated test; `tests/m7_docker_executor.rs` alone is 1,557 lines) — and ADR 0001 (D8)(b) exists precisely because a run can go green having silently skipped most of what matters. A `NOT_REACHABLE` here means any later green run's skip count is meaningless until you start Docker Desktop and re-check. |
| Docker: runtime lifecycle (not just daemon reachability) | `docker run --rm alpine:3 echo ok` | Prints `ok` | **Degrade if it fails while the row above says `REACHABLE`** — that combination is itself a finding to file, not to paper over: the probe's own "Docker runtime lifecycle" row is deliberately left unprobed at session start ("must stay cheap and offline-safe"), and this is the first real exercise of Docker Desktop's VM/bind-mount path, which `path-to-mac.md` step 9 already flags as unmeasured here. |
| `claude` CLI: present and authenticated | `PATH="$HOME/.cargo/bin:$PATH" bash scripts/probe-env.sh` → read the `claude CLI` and `claude auth status` rows | A version, plus `loggedIn=True` and some `authMethod` | **Degrade**, conditional on scope (see "What this assumes" below) — the default `cargo test` run doesn't invoke the real `claude` CLI at all. Promote to a hard stop only if this session's brief calls for the opt-in `SERGEANT_CLAUDE_TESTS=1` suite or a live `sgt` run against the real backend. Whichever `authMethod` the probe reports, record it as-is rather than assuming it matches another host — `docs/environments/cerberus.md` records `authMethod` differing between hosts (`claude.ai` vs `oauth_token`) as a plausible gating variable for a measured capability difference (the `a5` finding), not a detail safe to skip. |
| `gh`: present, authenticated, with scopes | `gh --version`; `gh auth status`; `gh issue view 18 --repo miztertea/sergeant-rs` | `gh auth status` reports a logged-in account and its token scopes; the issue view prints #18's body | **Degrade.** Not measured by `scripts/probe-env.sh` at all — no row for it. A bare `gh issue view 18` (no `--repo`) can fail with a misleading auth remedy even when `gh auth status` is fine, if this checkout's `origin` isn't a GitHub host (#112) — always pass `--repo miztertea/sergeant-rs` here. You can build, test, and flip `UNVERIFIED` markers without `gh`; you just lose the convenient path to #18/#81/#82/#95's bodies and fall back to the GitHub web UI. |
| Network: crates.io | `curl -s -o /dev/null -w '%{http_code}' --max-time 5 https://index.crates.io/config.json` | `200` | **Hard stop for the cold build** (step 3 of `path-to-mac.md`) unless the full dependency graph is already vendored or cached locally, which a first-contact machine won't have. Not measured by `scripts/probe-env.sh` — its network rows check GitHub hosts only (`api.github.com`, `raw.githubusercontent.com`, the no-mistakes install script), which cover git/toolchain reachability, not the crate registry a `cargo build` actually fetches from. |
| Disk headroom | `PATH="$HOME/.cargo/bin:$PATH" bash scripts/probe-env.sh` → read the `Disk free ($HOME fs)` and `Writable allowance` rows | Free space clearly above what this repo needs (see next column) | **Hard stop if headroom reads under roughly 20 GB free.** The raw number the probe reports isn't self-interpreting: `target/` with bundled DuckDB runs ~5 GB normally and can reach ~15 GB if `Cargo.toml`'s `[profile.dev.package.libduckdb-sys] debug = false` pin is ever lost (`docs/DEVELOPMENT.md:27`), on top of the clone itself and any disposable worktrees this trip creates on `/var/tmp` (this repo's own placement rule — never `/tmp`). An 18 GB-RAM laptop is not necessarily a spacious-disk one; check this before the cold build, not after it fails ~10 minutes in on `ENOSPC`. |

## What this checklist assumes

It assumes the trip's primary deliverable is the ADR 0001 (D8) measurement
pass — `scripts/probe-env.sh`'s output recorded, plus a full `cargo test`
run with a published skip count — not necessarily running `sgt` itself as a
live daemon. `cargo build`/`cargo test` don't need `claude`/`gh`
authentication to succeed; only the opt-in `SERGEANT_CLAUDE_TESTS=1` suite
and any live `sgt` run against the real backend do. That's why those two
rows are marked degrade rather than hard stop above. If a session's actual
brief is different — e.g. it explicitly asks to run `sgt` for real against
Docker or Claude, not just `cargo test` against them — promote the relevant
degraded rows to hard stops before starting, rather than discovering the gap
mid-trip.
