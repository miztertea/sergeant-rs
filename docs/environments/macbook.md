# MacBook Pro M3 Pro (18GB RAM)

Measured 2026-08-15, first contact. Host `US-DWDVHVG44T`, model `Mac15,7`
(Apple M3 Pro, confirmed via `sysctl -n hw.model` / `machdep.cpu.brand_string`),
12 cores, 18 GB RAM (`sysctl -n hw.memsize`), Darwin kernel 25.6.0
(`uname -a`: `Darwin ... 25.6.0 ... RELEASE_ARM64_T6030 arm64`), macOS 26.6.1
(build 25G76, `sw_vers`).

Two of `scripts/probe-env.sh`'s own rows below are **corrected** rather than
pasted verbatim — the probe reported them wrong on this host, not merely
`unmeasurable`, and per this file's own house rule ("measured-not-assumed",
`docs/handoff/path-to-mac.md` step 1) a known-false line does not belong in a
durable record. Root cause and independent verification for both are in
"Probe defects found on this host" below the table.

Facts measured 2026-08-15 on host "US-DWDVHVG44T"

| Fact | Measured value | Evidence |
|---|---|---|
| uid / user / groups | uid=501 (thawes); groups: staff,DPusers,everyone,localaccounts,_appserverusr,admin,_appserveradm,_developer,_appstore,_lpadmin,_lpoperator,_analyticsusers,com.apple.access_ftp,com.apple.access_screensharing,com.apple.access_remote_ae,com.apple.sharepoint.group.1 | `id -u; id -un; id -Gn`, probe-env.sh, 2026-08-15 |
| DAC / permission-bit enforcement | enforced — `chmod 000` self-read FAILED (permission denied) | probe-env.sh, 2026-08-15 |
| CAP_LINUX_IMMUTABLE | unmeasurable: `chattr` not present on this host (Linux-only capability; macOS has no equivalent syscall) | probe-env.sh, 2026-08-15 |
| Disk free ($HOME fs) | 460Gi total, 237Gi avail (46% used) on `/System/Volumes/Data` | `df -Ph "$HOME"`, probe-env.sh, 2026-08-15 |
| Disk free ($TMPDIR fs) | 460Gi total, 237Gi avail (46% used) on `/System/Volumes/Data` (same physical volume as $HOME on this host) | `df -Ph "$TMPDIR"`, probe-env.sh, 2026-08-15 |
| Quota binaries on PATH | present: `quota`, `repquota` (presence only — not confirmed to apply to this host's filesystem) | `command -v`, probe-env.sh, 2026-08-15 |
| Writable allowance (session quota, distinct from FS capacity) | unmeasurable: `quota -s` exited 111, no usable output — no per-user limit visible via quota; an allowance may still be enforced elsewhere (session sandbox), above the filesystem | probe-env.sh, 2026-08-15 |
| O_DIRECT open+unaligned-write ($TMPDIR fs) | unmeasurable: `os.O_DIRECT` not exposed by this host's python3 build (macOS has no `O_DIRECT` flag at all — the nearest equivalent is `fcntl(F_NOCACHE)`, a different mechanism the probe doesn't test) | probe-env.sh, 2026-08-15 |
| O_DIRECT open+unaligned-write ($HOME fs) | unmeasurable, same reason as above | probe-env.sh, 2026-08-15 |
| Proxy env vars | none set | probe-env.sh, 2026-08-15 |
| HTTPS reachability: api.github.com | HTTP 200 | probe-env.sh, 2026-08-15 |
| HTTPS reachability: raw.githubusercontent.com (bare host) | HTTP 301 | probe-env.sh, 2026-08-15 |
| HTTPS reachability: no-mistakes install.sh (real asset path) | HTTP 200 | probe-env.sh, 2026-08-15 |
| Docker: presence / daemon / storage driver / cgroup / group | **corrected** — present; daemon **reachable** (`docker info` succeeds directly); user NOT in the `docker` group per `id -nG`, but Docker Desktop on macOS doesn't require that membership (it proxies through a VM, not a shared socket-group model) | `docker info`, `docker --version`, `id -nG`, run directly (not through the probe's `bounded` helper), 2026-08-15 — see "Probe defects" below for why the probe itself misreported this |
| Docker runtime lifecycle (pull/run/network/cleanup) | **verified end to end**: `docker run --rm alpine:3 echo ok` pulled the image (5 layers, cold) and printed `ok` | manual run, this session, 2026-08-15 |
| claude CLI | **corrected** — present: `claude --version` → `2.1.233 (Claude Code)` | run directly, 2026-08-15 |
| claude auth status | **corrected** — `loggedIn=true`, `authMethod=claude.ai`, `apiProvider=firstParty`, org `Deloitte`, subscription `enterprise` | `claude auth status --json` run directly, 2026-08-15 (token/email redacted from this durable record beyond what's shown) |
| cargo | `cargo 1.97.1 (c980f4866 2026-06-30)` (on PATH via `~/.cargo/bin`) | probe-env.sh, 2026-08-15 |
| rustc | `rustc 1.97.1 (8bab26f4f 2026-07-14)` (on PATH) | probe-env.sh, 2026-08-15 |
| Cores | **corrected** — 12 (`sysctl -n hw.ncpu`; macOS has no `nproc`) | run directly, 2026-08-15 |
| Kernel | 25.6.0 | `uname -r`, probe-env.sh, 2026-08-15 |
| Container heuristic (evidence, not a verdict) | `/.dockerenv` absent; `/proc/1/cgroup` unreadable (expected — not a container, no `/proc` on macOS) | probe-env.sh, 2026-08-15 |
| IS_SANDBOX env var | unset | probe-env.sh, 2026-08-15 |
| bash version (PATH-resolved) | `GNU bash, version 5.3.15(1)-release (aarch64-apple-darwin25.4.0)` — Homebrew bash, first on `$PATH` | `bash --version`, 2026-08-15 |
| bash version (system, `/bin/bash`) | `GNU bash, version 3.2.57(1)-release (arm64-apple-darwin25)` — confirms ADR 0004 (D7)'s assumption for the system shell specifically | `/bin/bash --version`, 2026-08-15 |

## Probe defects found on this host

`scripts/probe-env.sh`'s `bounded()` helper (script line ~62) wraps
wall-clock-bounded checks in `timeout "$secs" "$@"`, falling back to printing
`"unmeasurable: no timeout(1) available to bound this probe"` **as the
command's own stdout** when `timeout` isn't on `PATH` — which it is not on a
stock macOS install (`timeout`/`gtimeout` are GNU coreutils, not shipped by
Apple). Three rows depend on `bounded()`:

- **Docker daemon**: `bounded 5 docker info ...` never actually invoked
  `docker info` — it printed the fallback sentinel instead, which the script
  then parsed as a failed JSON response and reported **"not reachable"**.
  Independently confirmed false: `docker info` and `docker run --rm alpine:3
  echo ok` both succeed immediately, directly, on this host.
- **claude CLI / claude auth status**: same mechanism, reported
  `"unmeasurable... exited 111"`. Independently confirmed the underlying
  commands work fine (`claude --version`, `claude auth status --json` both
  succeed directly, exit 0).

Separately, **Cores** used `nproc`, which macOS does not ship (GNU coreutils
again) — the script correctly fell back to reporting `unmeasurable` rather
than guessing, but the real value (`sysctl -n hw.ncpu` → 12) was one command
away.

This is the same class of defect as #81/#82 (GNU-coreutils-only tooling
assumed portable) but in `scripts/probe-env.sh` itself rather than
`src/platform/`, and it produces a **false negative**, not just a gap — a
session trusting this probe's raw output at face value would have believed
Docker was unreachable and gone looking for a Docker Desktop problem that
doesn't exist. Worth filing separately from #18/#81/#82/#95 (see session
notes); not fixed in this pass per the path-to-mac scope (measurement, not
build work), but recorded per this file's own "anything unanticipated gets
recorded, not worked around" rule.

Paste destination: docs/environments/macbook.md

Repo invariants (target size, build times, test counts) intentionally NOT
here — they live in `docs/DEVELOPMENT.md`. Sibling files: `cerberus.md`,
`claude-code-cloud.md`, `github-runner.md`.
