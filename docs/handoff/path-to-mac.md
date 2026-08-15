# Path to Mac — arrival checklist

For a session landing on the MacBook Pro M3 Pro (18GB RAM, ADR 0001) with
this repo. Owner's framing for what this session is for: *"We shouldn't
carry land mines with us."* You should be able to **arrive, measure, close,
and return** — not arrive and start debugging. Read `docs/DEVELOPMENT.md`
first regardless; this file assumes it.

## The bar

Quoted verbatim from ADR 0001 (D8) — this is the bar, not a paraphrase of it:

> A platform earns that label only once two things are both true: (a)
> `scripts/probe-env.sh` has run there and its output is recorded in
> `docs/environments/<host>.md`, the existing per-host fact convention
> (`docs/environments/README.md`); and (b) the full test suite has run there
> with a **published skip count**. (b) exists because this repo's fixtures
> probe-gate with a loud `SKIPPED-ENV` under the two-environment rule in
> `docs/DEVELOPMENT.md`... at four environments instead of two, a run can go
> fully green having skipped most of what actually matters on that host, and
> a published count is what keeps "measured" from quietly becoming a soft
> claim.

## What the Mac owns

Three issues ship today with both platform arms already built and
unit-tested; they are blocked on nothing but a macOS host to flip their
`UNVERIFIED` marker. One issue is genuinely undecided and needs real
hardware to settle.

| Issue | State | What the Mac does |
|---|---|---|
| #18 | `src/platform/process.rs` — both arms shipped, unit-tested | **Verify**, flip UNVERIFIED |
| #81 | `src/platform/disk.rs` — GNU and POSIX arms shipped, test-pinned | **Verify**, flip UNVERIFIED |
| #82 | `src/platform/data_dir.rs` — both conventions shipped, unit-tested | **Verify**, flip UNVERIFIED |
| #95 | fail-loud guard shipped (`scripts/perf/common.sh`); clock **unchosen** | **Choose the clock** on real hardware |

None of this is build work — it's measurement. Do not reach for `fs4` /
`sysinfo` / `directories` to replace any of #18/#81/#82's hand-rolled code;
that premise was already raised and rejected this sprint
(`docs/gauntlet/runs/path-to-mac-2026-08-15/adjudication.md`,
`src/platform/disk.rs:1-21`). A change that re-litigates that ruling must
argue it's wrong, not merely note the hand-rolled code exists (**L3**).

Aside: `src/platform/fs_locking.rs` (#85) also carries a macOS
`UNVERIFIED` arm, built in a later wave of this same sprint. It isn't one of
the four issues this sprint tracked as blocking, but the full suite run
below exercises it too, and it gets the same "flip on measurement" treatment
as a matter of course.

## Pre-flight — before step 1

This checklist has a gap: it tells you to "install cargo/rustc if missing"
in passing (step 2) and discusses Docker Desktop's *semantics* (step 9), but
it never states what must already be true before you start. Without it, a
session can reach step 4 — after the ~10-minute cold DuckDB build in step 3
— before discovering a missing dependency it could have caught in seconds.

Run **`docs/handoff/pre-flight.md`** top to bottom first. It's written as a
generic checklist (it's meant to serve the next new host too, per ADR 0001
D1's Hades/WSL2 target, not just this one), so two of its judgment calls are
worth restating in this trip's specific terms:

- **Docker is not optional here.** This trip's step 4 runs the full suite,
  and six of its suites are Docker-gated (`m2`, `m3`, `m4`, `m6`, `m7`,
  `m8` — the pre-flight file's Docker row names them). If Docker Desktop is
  installed but not running, those suites probe-gate to `SKIPPED-ENV`
  instead of failing — the run goes green having quietly skipped most of
  what actually matters on this host, which is the exact failure mode ADR
  0001 (D8)(b) exists to catch. Treat the pre-flight file's Docker
  reachability row as a hard stop for this trip, and re-check it right
  before step 4 starts, not only at session start.
- **`claude`/`gh` auth are degrade-not-hard-stop for this trip specifically**,
  because closing #18/#81/#82 and choosing #95's clock (this file's whole
  point) only need `cargo build`/`cargo test` to succeed — not a live
  `claude` turn or `gh` working. If you do plan to also read #18/#81/#82/#95
  through `gh` (the natural way to orient before step 1), the pre-flight
  file's `gh` row already covers the `--repo miztertea/sergeant-rs` fix for
  #112's misleading auth remedy.

## Steps

1. **Land and orient.** Check out the branch you were handed. Read
   `docs/DEVELOPMENT.md` if you haven't. Do *not* trust any platform claim
   in prose over what you measure this session (measured-not-assumed is the
   house rule).

2. **Probe first, before anything else.**
   ```sh
   PATH="$HOME/.cargo/bin:$PATH" bash scripts/probe-env.sh
   ```
   Paste its output into `docs/environments/macbook.md`, replacing every
   `NOT YET MEASURED` cell. This is ADR 0001 D8(a). If `cargo`/`rustc` aren't
   installed yet, install them first — the probe script reports their
   absence rather than failing, but you want real values recorded, not
   "unmeasurable."

3. **Cold build.** `cargo build`. Expect a genuinely slow first build —
   bundled DuckDB compiles ~500 C++ translation units (~10 min cold on the
   Linux hosts this was last timed on; record how long it actually takes
   here, it hasn't been measured on Apple Silicon).

4. **Run the gates, capturing output.**
   ```sh
   cargo fmt --check
   cargo clippy --all-targets -- -D warnings
   cargo test 2>&1 | tee /tmp/macos-suite-run.log
   ```
   This is the first time `src/platform/{process,disk,data_dir,fs_locking}.rs`'s
   `#[cfg(target_os = "macos")]` arms compile and run on real hardware — the
   Linux-side unit tests already exercise the *decision logic* (ADR 0002 D3),
   but not the actual `ps`/`df`/`kill`/`flock` syscalls those arms shell out
   to or call directly.

5. **Publish the skip count.** Count the `SKIPPED-ENV` lines in the log
   (`grep -c SKIPPED-ENV /tmp/macos-suite-run.log` or equivalent) and record
   it alongside the run — this is ADR 0001 D8(b), and it's not optional: a
   green run with an unpublished skip count doesn't clear the bar. Put the
   count and the log's key lines wherever this session records its own
   summary (issue comment, PR body, or a dated note — your call, but it must
   be recorded somewhere durable, not just in a terminal that closes).

6. **Flip `UNVERIFIED`, but only on a green run.** If step 4 passed clean
   for `src/platform/process.rs`, `disk.rs`, and `data_dir.rs`, edit each
   file's `UNVERIFIED` doc comment(s) to record the measurement (date, what
   was run) and close #18/#81/#82 by that measurement — the files' own
   module docs say this explicitly ("They close #18 when someone measures
   them there, not when this lands"). If a macOS arm actually fails, do
   **not** flip it — file what broke instead; a failing measurement is a
   real finding, not a checklist item to force green.

7. **Choose #95's clock.** `scripts/perf/common.sh`'s `perf_now_ns`/
   `perf_mark` already fail loudly rather than feeding a malformed
   timestamp into arithmetic (macOS bash 3.2 has no `EPOCHREALTIME`, and
   `date +%s%N` — a GNU coreutils extension — emits a literal trailing `N`
   on BSD/macOS `date`). What's still open is which replacement clock to
   use. Candidates, and why none was picked without hardware to measure on:
   - `perl -MTime::HiRes` — preinstalled on macOS, but forks a subprocess.
   - `python3 -c 'import time; print(time.time_ns())'` — portable, but also
     forks.
   - Accept millisecond resolution (whatever `date` without `%N` gives you)
     and **record the degradation rather than hiding it** — the perf
     harness's numbers would then carry coarser timing on macOS than on
     Linux, and callers need to know that, not discover it.

     `perf_mark` deliberately avoids forking on the fast path (a
     `date +%s%N` fork costs ~2ms, the same order of magnitude as what's
     being measured) — so before picking `perl` or `python3`, **measure**
     their fork cost on this hardware rather than assuming it's fine.
     Implement the choice in `scripts/perf/common.sh`, keep it bash-3.2-clean
     (see below), and record the measurement that justified it.

8. **Stay bash-3.2-clean.** macOS ships bash 3.2.57, frozen there for
   licensing reasons (GPLv2 vs GPLv3 — ADR 0004 D7). Confirm with
   `bash --version`. Anything you run or add here — one-off commands, not
   just `scripts/perf/common.sh` — must avoid: `EPOCHREALTIME` (bash 5.0+),
   `local -n` namerefs (bash 4.3+, already fixed once at `common.sh` per
   ADR 0004's Consequences), associative arrays, `mapfile`/`readarray`,
   `${x,,}`/`${x^^}` case conversion, and `&>>`. `date +%s%N` is a GNU
   extension too — BSD/macOS `date` silently returns a literal `N` suffix
   instead of nanoseconds if you call it that way outside the guarded
   helper.

9. **Watch for these, unmeasured going in:**
   - **Docker Desktop's bind-mount and uid semantics differ from Docker on
     Linux.** ADR 0002 (D4) puts this outside the platform boundary
     deliberately — it's the `Backend`'s problem, not a platform fact — but
     it's real and nobody has measured it here yet. If you exercise
     `DockerBackend` (`src/backend/docker.rs`), the worktree-ownership fix
     there (`--user <uid>:<gid>` sourced from the mounted worktree's host
     owner) was proven correct on Linux/overlayfs only; Docker Desktop's own
     bind-mount layer may behave differently.
   - **#18's macOS arm shells out to `ps -axo pid=,command=`, tokenized on
     whitespace.** Its own doc comment and test
     (`quoted_argument_with_a_space_is_split_current_known_weakness`,
     `src/platform/process.rs`) already name the failure mode: a quoted
     argument containing a space defeats the tokenizer. It's judged
     sufficient today because the launch grammar it actually matches
     (`--session-id <uuid>` / `--resume <uuid>`) never quotes — but if you
     see anything that looks like a spurious argv split during the suite
     run, this is the first place to look.

## Done looks like

- `docs/environments/macbook.md` has real measured values, not
  `NOT YET MEASURED`, dated this session.
- The full suite ran, went green (or every red is a real, filed finding —
  not a flipped marker forcing green), and its skip count is published
  somewhere durable.
- #18, #81, #82 are closed by measurement (their `UNVERIFIED` markers are
  gone, replaced with what was actually run and when) — or, if something
  genuinely failed, that failure is filed instead of hidden.
- #95's clock is chosen, implemented, and the choice is justified by a
  measurement taken on this hardware, not assumed from the candidates'
  reputations.
- Anything this checklist didn't anticipate got recorded rather than
  quietly worked around — that list being non-empty is the expected outcome
  of a first contact with new hardware, not a failure of this checklist.
