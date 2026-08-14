# ADR 0001: Platform targets and measurement posture

**Status:** Accepted, 2026-08-14.

## Context

Sergeant is a per-developer distro (`NORTH-STAR.md`: "Sergeant is an AgentOS
distro"), and the owner's colleagues work primarily on Windows and Mac —
cross-platform is a product requirement for `sgt`, not portability debt to
pay down opportunistically. The owner has exactly four environments where a
platform claim can actually be measured rather than assumed: the Claude Code
cloud container (Linux, the original dev environment), Cerberus (Linux, the
current primary host — `docs/environments/cerberus.md`), Hades (hardware
identical to Cerberus, running Windows 11, reachable for WSL2 measurement),
and a 2024 MacBook Pro M3 Pro with 18GB RAM. This decision, and the two
below it, were made in a live `grilling`/`grill-with-docs` interview on
2026-08-14 — North Star ruling R-NS-6 holds that such interviews are
conversation, never dispatched Work, so nothing here was produced by a
workflow run. The project is alpha; `docs/DEVELOPMENT.md` and `AGENTS.md`
make no platform-support claims anywhere today, and this ADR does not
introduce one.

## Decision

**Targets (D1).** The measured targets are Linux, macOS, and Windows only
via WSL2. There is no native Windows target. WSL2 runs a real Linux kernel,
so `/proc`, `SIGTERM`, `flock`, and `bash` all work as they do on Linux —
which collapses what would otherwise be a full second process-model port
down to nothing.

**Vocabulary: "measured", not "supported" (D8).** While the project is
alpha, no support claims are made about any platform at all. The vocabulary
this repo uses instead is "measured on `<host>`". A platform earns that
label only once two things are both true: (a) `scripts/probe-env.sh` has
run there and its output is recorded in `docs/environments/<host>.md`, the
existing per-host fact convention (`docs/environments/README.md`); and (b)
the full test suite has run there with a **published skip count**. (b)
exists because this repo's fixtures probe-gate with a loud `SKIPPED-ENV`
under the two-environment rule in `docs/DEVELOPMENT.md` ("Tests run in two
known environments with opposite constraints... design fault-injection
fixtures for both or probe-gate with a loud `SKIPPED-ENV`"); at four
environments instead of two, a run can go fully green having skipped most
of what actually matters on that host, and a published count is what keeps
"measured" from quietly becoming a soft claim. An earlier proposal, floated
in the same interview, for a "supported" tier gated on product-promise
probes was considered and **rejected by the owner as premature for alpha**;
the tier is not built and the vocabulary for it does not exist yet, but the
possibility of it returning post-alpha is recorded rather than foreclosed.

**Unsupported build targets: leave alone (D10).** No `compile_error!` guard
refuses a build on a platform outside the measured set, and the existing
`cfg(not(unix))` stubs stay in the tree. The posture is: this repo states
what is measured; if someone wants to build `sgt` on native Windows anyway,
that is their business.

## Alternatives considered

**Native Windows support** was the alternative to D1's WSL2-only stance,
and was rejected on a cost/benefit argument, not a philosophical one:
signal- and process-lifecycle code is genuinely pervasive here. Re-measured
2026-08-14 (`grep -rn 'SIGTERM\|SIGKILL\|\.kill(\|libc::kill'` across
`src/`): 44 references across the same 6 files the interview's figure of
"~50 across 6 files" named — `src/daemon.rs` (10), `src/tui.rs` (15),
`src/cli.rs` (8), `src/backend/claude.rs` (6), `src/backend/fake.rs` (4),
`src/backend/mod.rs` (1). Add `daemon.lock`'s advisory-locking semantics
(`src/daemon.rs:49-50`, `src/runtime/fsutil.rs:24-29`), the detached-daemon
lifecycle, and `TtyWatch`'s pty hangup detection (`src/tui.rs`), and a
native-Windows port would need a second process model built and maintained
alongside the Unix one — for no benefit once WSL2 is available as a real
Linux kernel underneath Windows hardware.

**A "supported" tier gated on product-promise probes**, the second
alternative floated in the interview, was rejected as premature: the
project has not yet earned the right to make a support claim to strangers
while still alpha. It is recorded here as considered, not dismissed outright
— it is explicitly a candidate for reintroduction once the project is past
alpha.

**Refusing unsupported builds at compile time and deleting the
`cfg(not(unix))` stubs** was the alternative to D10's leave-it-alone
posture, and was rejected as unnecessary machinery for what this repo is
today: an alpha project that states what it measures rather than policing
what others attempt.

## Consequences

WSL2-as-real-Linux is what makes D1 cheap: no second process model, no
native-Windows-specific signal or lockfile handling to build or keep green.

D10's leave-alone posture has a consequence that must survive review, not
just theory: platform-fact stubs can still produce misleading runtime
messages on a platform outside the measured set. At the time of this
interview, `free_space` in `src/backend/docker.rs:1308-1311` returned
`None` unconditionally when not `cfg(unix)`; ADR 0002 has since moved that
fact to `src/platform/disk.rs` and added an UNVERIFIED macOS arm, so today
the unconditional `None` is confined to platforms outside both Linux and
macOS. `src/cli.rs:2025` and `2055` render a `None` result as "free space
could not be measured on this platform" in `sgt doctor`'s disk-pressure
check — a message that (per the comment at `src/cli.rs:2005-2010`, added
for a related but distinct fix, #67) has already been flagged once as
potentially misdirecting an operator. This gap is tracked separately as
**issue #81**, not closed by ADR 0002's macOS arm since it remains
unmeasured on a real host, and was explicitly not fixed by this ADR or
D10's decision to leave the stubs standing.

## Open questions

- The "supported" tier's exact gating criteria (which product-promise
  probes, what threshold) were not specified in the interview beyond
  "premature for alpha" — this is deferred, not designed, and whoever picks
  this back up post-alpha should not assume the shape sketched informally
  in the interview was ever ratified.
- This ADR could not independently confirm issue #81's tracked title or
  body against GitHub (this checkout's `origin` remote is a local path, not
  a GitHub host, so `gh issue view` cannot resolve it here). The code
  artifact the issue is said to describe — the `free_space` stub and its
  "could not be measured on this platform" message — is real and verified
  above; the issue number itself is taken on the interview's word.
