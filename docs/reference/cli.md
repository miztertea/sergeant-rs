# CLI reference

The executable command graph is authoritative. Run `sgt --help` and `sgt <command> --help` for exact arguments, defaults, and closed vocabularies.

Global options are `-C <ESTATE_ROOT>`, `--data-dir <PATH>`, and `--json`. A daemon is host-scoped (one long-lived process serving every estate ever admitted to it) — `sgt` verbs split into three sets accordingly:

- **Unscoped** — `--help`, `--version`, `init`, `doctor` — valid anywhere, touch no daemon.
- **Host-scoped** — `tui`, `status`, `work show`/`list`/`transcript`, `watch`, and every `daemon` verb (foreground start, `stop`, `install-service`) — valid from any directory, no estate root required. `sgt watch` still consults `-C`/cwd opportunistically: inside a valid estate root it defaults to that estate's events; `--all`, or a cwd that is not one, watches every estate the daemon has admitted.
- **Estate-scoped** — everything else (`run`, `repo`, `group`, `workflow`, `respond`, `retry`, `extend`, `cancel`, `work reap`/`sweep`/`retained`, `analytics`, and the harness launchers) — requires an exact estate root (`-C <ESTATE_ROOT>` or a cwd that is one); refused outside one.

Harness launchers exec the native harness; estate client commands use the host daemon and may auto-start it according to their help contract — the spawned daemon addresses no estate itself (H1: every estate it ever serves is admitted per-request, over the wire, once it is already running).

| Family | Purpose | Scope |
|---|---|---|
| `daemon` | run in the foreground, stop, install the native service unit, or rebuild daemon state | host |
| `status`, `tui` | health and human operation | host |
| `doctor` | diagnose this installation | unscoped |
| `init`, `repo`, `group` | estate topology | unscoped (`init`) / estate (`repo`, `group`) |
| `workflow` | fork a stock package into a local, editable copy | estate |
| `run` | submit accepted intent and scope | estate |
| `work` | list, show, transcript, retained-state, reap, and sweep operations | host (`list`/`show`/`transcript`) / estate (`retained`, `reap`, `sweep`) |
| `respond`, `retry`, `extend`, `cancel` | legal Work transitions | estate |
| `watch` | blocking attention/terminal notifications | host, `-C`/cwd-scoped by default (D6) |
| `analytics` | query operational projections | estate |
| `claude`, `codex`, `opencode`, `goose`, `agy` | launch Captain harnesses | estate |

Human errors use a nonzero exit status. Successful `--json` output is JSON; streaming surfaces use JSONL where documented. Do not assume every error shape or exit status is stable across pre-1.0 releases unless a page explicitly promises it.
