# MacBook Pro M3 Pro (18GB RAM)

Measured 2026-08-15, first contact. Host `US-DWDVHVG44T`, model `Mac15,7`
(Apple M3 Pro, confirmed via `sysctl -n hw.model` / `machdep.cpu.brand_string`),
12 cores, 18 GB RAM (`sysctl -n hw.memsize`), Darwin kernel 25.6.0
(`uname -a`: `Darwin ... 25.6.0 ... RELEASE_ARM64_T6030 arm64`), macOS 26.6.1
(build 25G76, `sw_vers`).

Two of `scripts/probe-env.sh`'s own rows in the measurement table below
were **corrected** rather than pasted verbatim — the probe reported them
wrong on this host, not merely `unmeasurable`, and per this file's own
house rule ("measured-not-assumed", `docs/handoff/path-to-mac.md` step 1)
a known-false line does not belong in a durable record. Root cause and
independent verification for both are in "Probe defects found on this
host" below.

## ADR 0001 (D8) bar — cleared

Full suite run 2026-08-15 on the final merged tip (`f0ec0b0`,
`integration/macbook-arrival-2026-08-15`), `$SGT_DATA_DIR` unset (this
checkout is now itself a live `sgt` estate — see the m8_estate_cli note
below): **655 tests, 655 passed, 0 failed, 0 `SKIPPED-ENV`, 4 `#[ignore]`d**
(opt-in `SERGEANT_CLAUDE_TESTS` suite, not environment skips). `#18`/`#81`/
`#82`'s `UNVERIFIED` markers are gone from `src/platform/{process,disk,data_dir}.rs`;
`#85`'s remains in `fs_locking.rs`, correctly not claimed — no real
`statfs`/`getmntinfo`/`diskutil` detection was built.

Two confounds surfaced and resolved during this run, neither a regression:
- With `$SGT_DATA_DIR` set (as this session's harness sets it, since this
  checkout became a live estate mid-sprint via `sgt repo add`), four
  `tests/m8_estate_cli.rs` tests fail — they assert an isolated tempdir data
  dir but pick up the ambient real one instead. Confirmed by unsetting the
  var: clean. Not a code defect; a fact for anyone running this suite from
  inside a Sergeant-managed checkout of itself to know.
- `run_turns_and_ceiling_secs_override_the_envelope_for_one_work` flaked
  once (a real daemon's health check timing out at 10s under host load
  from everything else running this session) — reproduced 3/3 clean in
  isolation immediately after.

**Measurement table moved.** The dated per-host measurement table
(30 rows: uid/groups, DAC enforcement, disk, quota, O_DIRECT, network
reachability, Docker, claude CLI/auth, cargo/rustc, cores, kernel,
container heuristic, bash versions) previously here now lives in
`sergeant-rs-workspace`'s knowledge library, at
`knowledge/evidence/host-measurements/macbook.md` (ADR 0014 decision 18:
the capability — `scripts/probe-env.sh` and the rule that a host is
measured before it is trusted — stays with the product; the measurements
themselves are workspace-shaped). The "corrected" rows and the "Probe
defects found on this host" section below are preserved verbatim there.

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

Paste destination for a fresh `scripts/probe-env.sh` run on this host:
`knowledge/evidence/host-measurements/macbook.md` in `sergeant-rs-workspace`
(no longer this file — see "Measurement table moved" above).

Repo invariants (target size, build times, test counts) intentionally NOT
here — they live in `docs/DEVELOPMENT.md`. Sibling files: `cerberus.md`,
`claude-code-cloud.md`, `github-runner.md`.
