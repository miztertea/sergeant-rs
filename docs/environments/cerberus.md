# Cerberus (owner's home server, cerberus.what-it.be on the local network)

Facts measured 2026-08-11, first session on this host (N-series close-out
handoff). Persistent host — no container-reset hazard — but facts still
drift on OS/tool updates; re-measure on suspicion and date any change.

| Fact | Measured value | Evidence |
|---|---|---|
| uid | 1001 (`miztertea`), groups include `sudo`, `docker` | `id`, 2026-08-11 session probe |
| DAC / permission-bit fixtures | **enforced** (chmod 000 read fails EACCES) — opposite of the root container; permission-bit fault fixtures work here | session probe 2026-08-11 |
| `CAP_LINUX_IMMUTABLE` | not available without sudo (`chattr +i` → EPERM); same shape as the GH runner | session probe 2026-08-11 |
| `claude --dangerously-skip-permissions` under this uid | expected viable (non-root; the CLI's refusal is root-specific) — **hypothesis until the opt-in suite runs**; direct probe blocked by session sandbox policy | root-refusal measured in Run B attempt 1; this host's confirmation pending task: opt-in m4 suite |
| `IS_SANDBOX=1` | **not required** (exists only to work around the root refusal) | gate.sh portability commit c585589 analysis |
| Disk | 935 GB LVM/ext4 root, ~864 GB free at first contact; no user quota tooling installed (`quota` absent) | `df -h`, 2026-08-11 |
| Cores / kernel | 20 cores; Linux 7.0.0-29-generic | `nproc`, `uname -r`, 2026-08-11 |
| /tmp | tmpfs, 16 GB; **O_DIRECT open SUCCEEDS on tmpfs here** (kernel 7.0) — differs from the cloud container where tmpfs refused it | python O_DIRECT probe, 2026-08-11 |
| O_DIRECT on ext4 (home fs) | open succeeds | python O_DIRECT probe, 2026-08-11 |
| Outbound network | open, no proxy vars; GitHub release/raw fetches return 200 (the cloud container's 403 does not apply — release installers work here) | curl probes, 2026-08-11 |
| Docker | 29.7.2, storage driver **overlayfs**, cgroup v2 (systemd driver), user in `docker` group (no sudo needed). Full lifecycle measured: registry pull (`alpine`), bind-mount read+write, `--network=none`, bridge egress to 1.1.1.1, image removal — all green. §22.7's cold-pull/digest/registry tests are runnable on this host | docker lifecycle probe, 2026-08-11 session |
| Claude CLI | v2.1.227 at `~/.local/bin/claude` — **one above the measured floor 2.1.226**; version gate accepts it but L1 requires re-measurement (opt-in m4 suite) before real-backend trust | `claude --version`, 2026-08-11 |
| no-mistakes | preinstalled at `~/.local/bin/no-mistakes`, runs under a systemd --user unit (not a direct fork — inline env prefixes are discarded; gate.sh detects and handles this) | c585589 commit message |
| Rust toolchain | cargo/rustc 1.97.1 in `~/.cargo/bin` — **not on non-interactive shells' default PATH**; prefix `PATH="$HOME/.cargo/bin:$PATH"` in scripts | `which cargo` miss + explicit path check, 2026-08-11 |
| Build deps | gcc/g++/make/pkg-config present; `cmake` absent (bundled DuckDB builds without it — warm build verified) | `which` probe + green `cargo build`, 2026-08-11 |

Repo invariants (target size, build times, test counts) intentionally NOT
here — they live in CLAUDE.md. Sibling files: `claude-code-cloud.md`,
`github-runner.md`.
