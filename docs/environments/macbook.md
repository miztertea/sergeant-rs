# MacBook Pro M3 Pro (18GB RAM)

Skeleton created 2026-08-15, in advance of first contact — no session has run
here yet. Every cell below is deliberately `NOT YET MEASURED`: per
`docs/environments/README.md`, *"an undated fact is a rumor,"* and a
plausible-looking guessed value would be worse than a blank. Do not fill any
cell by inference from another host's file (Cerberus, the cloud container) or
from documentation — only `scripts/probe-env.sh`'s own output, run on this
host, belongs here. See `docs/handoff/path-to-mac.md` for the arrival
checklist this file is step 2 of.

Facts measured NOT YET MEASURED on host "NOT YET MEASURED"

| Fact | Measured value | Evidence |
|---|---|---|
| uid / user / groups | NOT YET MEASURED | NOT YET MEASURED |
| DAC / permission-bit enforcement | NOT YET MEASURED | NOT YET MEASURED |
| CAP_LINUX_IMMUTABLE | NOT YET MEASURED | NOT YET MEASURED |
| Disk free ($HOME fs) | NOT YET MEASURED | NOT YET MEASURED |
| Disk free ($TMPDIR fs) | NOT YET MEASURED | NOT YET MEASURED |
| Quota binaries on PATH | NOT YET MEASURED | NOT YET MEASURED |
| Writable allowance (session quota, distinct from FS capacity) | NOT YET MEASURED | NOT YET MEASURED |
| O_DIRECT open+unaligned-write ($TMPDIR fs) | NOT YET MEASURED | NOT YET MEASURED |
| O_DIRECT open+unaligned-write ($HOME fs) | NOT YET MEASURED | NOT YET MEASURED |
| Proxy env vars | NOT YET MEASURED | NOT YET MEASURED |
| HTTPS reachability: api.github.com | NOT YET MEASURED | NOT YET MEASURED |
| HTTPS reachability: raw.githubusercontent.com (bare host) | NOT YET MEASURED | NOT YET MEASURED |
| HTTPS reachability: no-mistakes install.sh (real asset path) | NOT YET MEASURED | NOT YET MEASURED |
| Docker: presence / daemon / storage driver / cgroup / group | NOT YET MEASURED | NOT YET MEASURED |
| Docker runtime lifecycle (pull/run/network/cleanup) | NOT YET MEASURED | NOT YET MEASURED |
| claude CLI | NOT YET MEASURED | NOT YET MEASURED |
| claude auth status | NOT YET MEASURED | NOT YET MEASURED |
| cargo | NOT YET MEASURED | NOT YET MEASURED |
| rustc | NOT YET MEASURED | NOT YET MEASURED |
| Cores | NOT YET MEASURED | NOT YET MEASURED |
| Kernel | NOT YET MEASURED | NOT YET MEASURED |
| Container heuristic (evidence, not a verdict) | NOT YET MEASURED | NOT YET MEASURED |
| IS_SANDBOX env var | NOT YET MEASURED | NOT YET MEASURED |
| bash version | NOT YET MEASURED | NOT YET MEASURED |

Paste destination: docs/environments/macbook.md

The `bash version` row is not one of `scripts/probe-env.sh`'s own emitted
facts — added here because ADR 0004 (D7) pins macOS's bash at 3.2.57 and this
file is the natural place to confirm that assumption on the actual hardware
rather than carry it as an unmeasured belief.

Repo invariants (target size, build times, test counts) intentionally NOT
here — they live in `docs/DEVELOPMENT.md`. Sibling files: `cerberus.md`,
`claude-code-cloud.md`, `github-runner.md`.
