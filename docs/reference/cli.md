# CLI reference

The executable command graph is authoritative. Run `sgt --help` and `sgt <command> --help` for exact arguments, defaults, and closed vocabularies.

Global options are `-C <ESTATE_ROOT>`, `--data-dir <PATH>`, and `--json`. Only help/version, `init`, and `doctor` are valid outside an estate. Harness launchers exec the native harness; estate client commands use the estate daemon and may auto-start it according to their help contract.

| Family | Purpose |
|---|---|
| `daemon` | run in the foreground, stop, or rebuild daemon state |
| `status`, `doctor`, `tui` | health and human operation |
| `init`, `repo`, `group` | estate topology |
| `workflow` | fork a stock package into a local, editable copy |
| `run` | submit accepted intent and scope |
| `work` | list, show, transcript, retained-state, reap, and sweep operations |
| `respond`, `retry`, `extend`, `cancel` | legal Work transitions |
| `watch` | blocking attention/terminal notifications |
| `analytics` | query operational projections |
| `claude`, `codex`, `opencode`, `goose`, `agy` | launch Captain harnesses |

Human errors use a nonzero exit status. Successful `--json` output is JSON; streaming surfaces use JSONL where documented. Do not assume every error shape or exit status is stable across pre-1.0 releases unless a page explicitly promises it.
