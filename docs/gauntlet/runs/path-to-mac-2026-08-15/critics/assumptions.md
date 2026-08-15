# PATH-TO-MAC-1 — assumptions critic

**Axis:** assumptions. **Artifact:** `docs/gauntlet/runs/path-to-mac-2026-08-15/plan.md` at `d86885f`.

## Method note

Read, in full: the contract (`docs/gauntlet/contracts/PATH-TO-MAC-1.md`),
`plan.md` at `d86885f`, `reference/notes/gauntlet-pattern.md`,
`docs/DEVELOPMENT.md`, ADR 0001/0002/0005 in full, ADR 0007 (for #94),
`GAUNTLET.md`'s deviation register (lines 18–56) and the relevant ledger
rows, `LESSONS.md` L20/L21, and the cross-platform sprint's
`retrospective.md` (§1.1–1.4, §2.4, §4) and `close-out.md`. I did **not**
read `plan-review.md`, any other critic's output, or Work
`01M01XGMY8JJ4M4RJDN1RSZXJC`'s output — confirmed absent from this run's
directory at the time I ran (`ls critics/` was empty).

Commands run (read-only, no artifact under review touched, nothing written
outside this findings file):
- `git show d86885f --stat`, `git remote -v`, `git log main -1` — provenance.
- `mount`, `df -h /tmp`, `findmnt /tmp` — tmpfs claim.
- `gh --version`, `gh issue view 18`, `gh repo view` — tracker reachability.
- `sed -n` over `src/runtime/surface.rs`, `src/daemon.rs`, `src/backend/claude.rs`,
  `src/platform/process.rs`, `src/backend/docker.rs`(indirectly via ADR 0001) —
  code citations.
- `grep -n '^name = "..."' Cargo.lock`, `cargo tree --edges normal --prefix
  none | sort -u | wc -l` (metadata resolution only, no compile) — crate
  claims, dependency count.
- `find` / `grep` over `~/.cargo/registry/src/.../{fs4-0.13.1,sysinfo-0.37.2,directories-6.0.0}`
  — these three crates' source happened to be already cached locally, which
  let me check documentation claims against real source rather than memory.
- `find /home/miztertea -maxdepth 3 -iname sergeant.toml` — this run's
  worktree (`.../surfaces/01M01Y27TJ88N61FA0GJPPHDZR/sergeant-rs`) does not
  contain `sergeant.toml`; it lives one level up, at the estate root
  (`/home/miztertea/sergeant-rs/sergeant.toml`), outside the git-tracked
  checkout. I read it there. Flagging this so the adjudicator can judge
  whether "the estate root, not the repo" is a reasonable place for a Work
  to expect this file — I did not find it originally and want that near-miss
  on the record rather than silently corrected away.

**What I could not check:** `gh issue view` does not resolve on this host —
`origin` is a local path, not a GitHub host (same limitation ADR 0001's own
Open Questions section records for #81). I could not independently confirm
the ten issues' tracked titles/bodies against GitHub. Where another
governing document in this repo (ADR, LESSONS, the cross-platform
retrospective) independently describes the same issue, I used that as the
best available secondary source and say so per finding. Where none exists
(#96), I could not confirm or refute the plan's characterization —
recorded PLAUSIBLE, not dropped. I did not re-run the crate binary-size/
compile-time measurements (non-goal, explicitly out of scope) or the gate.

## Findings

### assumptions-F1 — ADR 0002 (D4) does not contain the objection the plan retires

**Severity argued: error.** This is a citation to a specific ADR decision
that, on inspection, is not there — not "true in a different frame," simply
absent.

**Plan text (§4, lines 93–98):**
> **This retires ADR 0002 (D4)'s objection on its own terms.** That decision
> declined "adding a libc-binding dependency for one syscall." `libc` and
> `rustix` are already dependency edges this crate carries, and the count is
> now four facts rather than one. **An ADR refresh is owed** — this repo's
> gate caught ADR staleness three times in the last sprint, and R8 above is
> the same failure one document over. Assigned to **W1**.

**Governing text it contradicts:** `docs/adr/0002-platform-boundary-shape.md`,
D4 (lines 46–64), in full:
> **What is behind the boundary (D4).** Platform facts are in scope for this
> boundary. Docker's and the `claude` CLI's behavior are explicitly out of
> scope — those remain `Backend` capabilities with runtime withdrawal...

D4 is about which facts belong behind the `#[cfg]`-module platform boundary
(platform facts in, Docker/claude-CLI behavior out) — it says nothing about
libc, statvfs, one syscall, or declining a dependency for either. I grepped
the whole ADR corpus (`grep -n "libc" docs/adr/*.md`; `grep -rn
"declined|decline" docs/adr/*.md`) and the exact phrase "adding a
libc-binding dependency for one syscall" appears **nowhere in this repo
except the plan itself** (`grep -rn "libc-binding\|one syscall" docs/
reference/ GAUNTLET.md LESSONS.md` → one hit, `plan.md:94`). Nothing in
ADR 0001, 0002, or any other ADR records a decision resembling this. It may
be a garbled memory of the *rejected* "Platform trait with four
implementations" alternative in ADR 0002 (also not about libc), or of a
decision that was cut before the ADRs were written — either way, the plan
asserts a specific, quoted governing decision that does not exist to be
retired.

**Verified vs believed:** VERIFIED absent. I read all of ADR 0002 and
searched the whole tracked doc corpus for the quoted phrase and its
constituent terms.

**What a correction would be:** Either (a) drop the "retires ADR 0002 (D4)"
framing and the quoted sentence entirely, replacing it with what actually
changed (`libc`/`rustix` are already dependency edges — true and separately
supportable, see assumptions-F-verified below), or (b) if the owner recalls
a real prior objection, cite where it actually lives (a different ADR, an
interview transcript, GAUNTLET.md) instead of ADR 0002 (D4). Either way,
**W1's "ADR refresh is owed" acceptance criterion is currently unenactable
as written** — a Work told to refresh "ADR 0002 (D4)'s objection" will not
find one to refresh.

---

### assumptions-F2 — R6 attaches the wrong issue's rationale to #108

**Severity argued: error.** This directly shapes W3's dispatched scope
(§5) and would misdirect the fix.

**Plan text (§2, R6, line 43):**
> | R6 | #108 is fixed by a **start-of-run reaper**, using standard patterns
> | `Drop` does not survive `SIGKILL`, and SIGKILL is how these die |

Echoed at §3 ("#95, #108 (harness hygiene)") and §5 ("W3 · harness hygiene |
#108 (start-of-run reaper), #95 | `main`").

**Governing text it contradicts — three independent sources, all naming
the same thing for #108:**

1. `LESSONS.md` L21 (lines 71–85), in full: *"198 zero-byte files named
   `sgt-watch-test-hold-never-released-*` had accumulated in `/tmp`...
   `test_hold_wait` writes `<path>.ready` as its rendezvous marker; the
   happy-path test removes it, and the dead-man test — whose whole premise
   is that the release path never appears — removes nothing... Cleanup
   belongs in the code under test or an RAII guard, never only in the body
   of the test that happens to succeed... **Filed as #108.**"*
2. `docs/DEVELOPMENT.md:53`: *"Test artifacts follow the same placement
   rule as build dirs: nothing a suite creates may be left in
   `std::env::temp_dir()`... a suite that leaks one file per run leaks it
   there (**#108**). Cleanup belongs in the code under test or an RAII
   guard — **not** in the body of the happy-path test, because the
   failure-path test is then the one that leaks."*
3. `docs/gauntlet/runs/cross-platform-2026-08-14/retrospective.md`, §1.2
   (lines 77–89), titled *"The 198 files — the failure-path test is the
   one that leaks (**#108**)"* — same dead-man-test-marker-leak account,
   generalized as L21, with **no mention of SIGKILL, `Drop`, or a reaper**.

The "`Drop` does not survive `SIGKILL`... a reaper that runs at suite start"
reasoning is real, but it belongs to a *different, adjacent* finding in the
same retrospective — §1.3 (lines 91–101), *"The 1.7 GB of test rigs — a
known cause, still uncaught."* That finding's own residue table (line 27)
gives its disposition as **"sweep"**, not an issue number: `| /var/tmp/sgt-rs-tests/.tmp* × 8 | 1.7 GB | disposable test rigs; Drop never ran on SIGKILLed runs | sweep |`.
§1.3 explicitly recommends extending `tests/support/mod.rs`'s existing
daemon reaper to run at start-of-run "for rigs as well as daemons" — that
recommendation has no filed issue number in this repo's tracker per the
retrospective's own accounting.

**Verified vs believed:** VERIFIED. All three governing citations quoted
above were read directly, and the retrospective's own residue table
(§1, line 27) explicitly separates #108 (marker-file leak, RAII fix) from
the reaper-shaped finding (SIGKILL/Drop, "sweep" disposition, no number).

**What a correction would be:** R6 should read something like: "#108 is
the dead-man test's `.ready`-marker leak (L21) — fixed by adding cleanup to
the code under test or an RAII guard, not by a reaper." If the owner also
wants the start-of-run reaper built (a reasonable, separately-justified
idea — it would close the *unnumbered* §1.3 finding too), that is a second,
additional piece of W3's scope and should be labeled as such rather than
presented as #108's fix. A Work executing R6 as written would build a
process/rig reaper and very plausibly leave #108's actual failure (the
dead-man test never calling cleanup) unfixed while reporting #108 closed.

---

### assumptions-F3 — #18's "direct /proc reads" citation is wrong for two of its three named files

**Severity argued: warning.** Doesn't invalidate adopting `sysinfo`, but
misstates the current blast radius and undersells work ADR 0002 already
did.

**Plan text (§4 table, line 77):**
> | #18 | `sysinfo` — `process(pid)`, `start_time()`, `cmd()` | direct
> `/proc` reads in `daemon.rs`, `backend/claude.rs`, `tests/support/mod.rs` |

**Governing text (the code itself) it contradicts:**
- `src/daemon.rs:930-933`: `process_alive` calls
  `crate::platform::process::process_alive(pid)` — routed through the
  platform boundary, not a direct read. (The one literal `/proc` read left
  in `daemon.rs`, at line 1128, is `read_dir("/proc/self/task")` for thread
  naming — unrelated to #18's process-liveness question.)
- `src/backend/claude.rs:597-601`: `session_liveness_excluding` calls
  `session_liveness_among(session_id, skip_pid,
  crate::platform::process::running_processes())` — also routed through the
  platform boundary, not a direct read. I found no direct `/proc` read for
  process listing anywhere in `claude.rs`.
- `src/platform/process.rs:1-95` (module doc, `raw_running_processes`,
  `raw_process_alive`): this module **already exists**, already follows
  ADR 0002's D2/D3 shape (`#[cfg]`-gated raw fact, injected-probe decision
  logic), and **already has a macOS arm** (`ps -axo pid=,command=`,
  documented UNVERIFIED). Its own module doc reads: *"Both are Linux-only
  via `/proc` today; both get a macOS arm here, and both macOS arms are
  UNVERIFIED... They close #18 when someone measures them there, not when
  this lands."*
- `tests/support/mod.rs:201` (`std::fs::read_dir("/proc")`) is the one file
  of the three that genuinely does a direct, unabstracted `/proc` read.

**Verified vs believed:** VERIFIED by reading the cited files directly.

**What a correction would be:** Narrow the "Replaces" column to what is
actually true: `sysinfo` would replace `src/platform/process.rs`'s
hand-rolled Linux/macOS split (both arms) plus the direct read in
`tests/support/mod.rs`, not "`daemon.rs`, `backend/claude.rs`" — those two
already call through the existing boundary. This also means R1's "the crate
answers already exist; rediscovering them on the Mac is reimplementing
`fs4` worse" undersells #18 specifically: some of the cross-platform work
already shipped (hand-rolled, ADR-0002-shaped, UNVERIFIED on macOS) in the
cross-platform sprint per `close-out.md`'s "Deliberately not closed" list
("#18... close on a real macOS host, not on Linux. Every macOS arm shipped
this round is marked UNVERIFIED").

---

### assumptions-F4 — the three crate deltas do not sum to the stated total

**Severity argued: warning.** A `[measured]` figure whose arithmetic does
not check is exactly the failure mode this axis exists to catch (per the
contract's own framing, "a figure and its percentage disagree").

**Plan text (§4, lines 86–88):**
> Stripped release binary delta, attributed: `fs4` **+4.7 KiB**,
> `directories` **+17.3 KiB**, `sysinfo` **+144 KiB**; all three **+161 KiB**
> — **0.26%** of the shipped binary.

**Internal inconsistency:** 4.7 + 17.3 + 144 = **166.0 KiB**, not 161 KiB —
a 5 KiB (≈3%) gap between the sum of the three attributed parts and the
stated total. I recomputed both:
- 161 KiB / 64,023,360 B → 0.2575% → rounds to the stated **0.26%**.
- 166.0 KiB / 64,023,360 B → 0.2655% → rounds to **0.27%**.

So the percentage is internally consistent *with the 161 KiB total*, but
that total does not match the three line items given as its components.
Either one of the three per-crate figures is wrong, the 161 KiB total was
computed before a rounding correction was applied to one line item and
never re-summed, or there is a fourth (uncredited, presumably small)
component netting the difference — none of which the plan states.

**Verified vs believed:** VERIFIED as an arithmetic fact
(`4.7+17.3+144=166.0`, not re-deriving the underlying binary-size
measurement itself, per the contract's non-goal).

**What a correction would be:** Re-sum the three attributed deltas and
either correct the total to 166 KiB (0.27%) or correct whichever per-crate
figure is wrong, and re-verify to the exact command that produced it. The
conclusion ("small, ~0.26–0.27% of the binary") is unaffected either way,
so this is not a re-litigation of R2/R3 — it is a request that the
`[measured]` figures agree with themselves.

---

### assumptions-F5 — R7's `sergeant.toml` citation points at the wrong lines

**Severity argued: info.** The underlying claim is true; only the cited
line range is wrong.

**Plan text (§2, R7, line 44):**
> | R7 | Sonnet for every dispatched Work | `--profile sonnet`, declared at
> `sergeant.toml:4-7` |

**Governing text:** `sergeant.toml` (estate root,
`/home/miztertea/sergeant-rs/sergeant.toml` — see method note), read in
full:
```
1  [estate]
2  name = "sergeant-rs"
3  # §13's third precedence tier (WorkspaceDefault), which outranks the daemon's
4  # own hardcoded GlobalDefault of "fake" (src/daemon.rs:251). Set 2026-08-15:
5  # every Work in this estate is real work on a real harness, and `--profile
6  # sonnet` launches "claude", so leaving the default at "fake" made every
7  # dispatch a 422 refusal until the backend was named twice. Explicit
8  # `--backend` still wins (RouteSource::Explicit is the first tier).
9  default_backend = "claude"
10
11 [[profile]]
12 name = "sonnet"
13 backend = "claude"
14 default_model = "sonnet"
```
Lines 4–7 are a comment about `default_backend`'s precedence and
`src/daemon.rs:251` (which I separately verified: `default_backend:
Some(FAKE_BACKEND_NAME.to_string())` — accurate). The `[[profile]] name =
"sonnet"` block the plan is actually pointing at is at **lines 11–14**.

**Verified vs believed:** VERIFIED by reading the file directly.

**What a correction would be:** `sergeant.toml:11-14`.

---

### assumptions-F6 — R8's `docs/DEVELOPMENT.md` line citation is a fragment of the rule, not the rule

**Severity argued: info.**

**Plan text (§2, R8, line 49):**
> `docs/DEVELOPMENT.md` line 71 states the rule; **ADR 0005** ... dissolved
> it

**Governing text:** `docs/DEVELOPMENT.md:70-73`:
> A workflow stage or actor executing inside a worktree never invokes
> `scripts/gate.sh`/no-mistakes itself — only the top-level orchestrating
> session owns a shipping-gate run, matching the single-owner posture the
> engine itself enforces on the data dir. <!-- BU-0041, BU-0122, BU-1196 -->

Line 71 alone is a mid-sentence fragment ("`scripts/gate.sh`/no-mistakes
itself — only the top-level orchestrating"). The rule as a complete
sentence spans lines 70–73. The claim itself (ADR 0005 dissolves this
rule; the quoted ADR 0005 text is accurate — I verified *"Gating becomes a
dispatched Work: Captain adjudicates findings; Sgt executes"* is an exact
quote from `docs/adr/0005-gating-becomes-a-dispatched-work.md`'s D1) is
correct; only the line-range precision is off, in a repo whose own
convention (see every ADR's own citations) is line **ranges** for anything
longer than one line.

**Verified vs believed:** VERIFIED.

**What a correction would be:** `docs/DEVELOPMENT.md:70-73`.

---

### assumptions-F7 — #85's "Replaces" column describes code that does not exist yet

**Severity argued: warning.** Not wrong that #85 needs a filesystem-type
answer; wrong that there is existing `/proc/mounts` parsing to replace.

**Plan text (§4 table, line 78):**
> | #85 | `fs4` for portable `flock(2)`; `sysinfo` Disks API for per-mount
> filesystem type | `/proc/mounts` parsing (Linux-only) |

**Governing text it contradicts:** I searched the tracked source tree for
any existing filesystem-type/mount-parsing implementation
(`grep -rn "/proc/mounts" src/` → no hits; `grep -rln
"mounts|statfs|f_type|filesystem type|fs_type" src/` → two files, neither
containing mount-parsing code — `src/domain/manifest.rs` and `src/cli.rs`
only use the word "mounts" in unrelated doc comments about repository
worktree mounts). `docs/gauntlet/runs/cross-platform-2026-08-14/plan.md:36`
describes #85 as detecting filesystem type from `/proc/mounts` as the
*intended, not-yet-built* approach ("G3 → G5: #85 detects filesystem type
from `/proc/mounts` — a platform [boundary concern]"), and
`close-out.md:108` lists #85 under "the queued perf work" — i.e. still
open, unimplemented, not merely undermeasured.

**Verified vs believed:** VERIFIED (absence of the code) via direct grep of
`src/`; the characterization of #85 as still-open is BELIEVED, sourced from
the cross-platform sprint's own planning/close-out docs (best available
secondary source given `gh` cannot reach the tracker here — see method
note).

**What a correction would be:** "Replaces the Linux-only `/proc/mounts`-
parsing approach originally planned for #85" (intent, not shipped code), or
simply drop "Replaces" in favor of "answers #85's filesystem-type need
directly, without a hand-rolled Linux path first."

---

### assumptions-F8 — the gh-version history claim carries no evidence-vs-belief label

**Severity argued: info.** A labeling-consistency gap, not a factual error
— I could not confirm or refute the historical claim itself.

**Plan text (§8, item 4, lines 189–191):**
> **`gh` was 51 minor versions behind** and `gh pr edit` failed every call
> on 2.46.0. **[measured]** upgraded this session to 2.97.0 from GitHub's
> own apt repo; `cerberus.md`'s row is now stale and is W5's to supersede.

The `[measured]` tag is attached only to the *upgrade action*. The two
preceding claims — "51 minor versions behind" and "failed every call on
2.46.0" — are stated with the same flat confidence but carry no tag at all,
in a document whose own stated convention (line 11-12: measured claims get
**[measured]**, artifact-sourced claims get a citation, beliefs "say so")
leaves no fourth, unlabeled category. The current version (2.97.0,
2026-07-31) matches what `gh --version` reports on this host now — I
VERIFIED that part. I could not verify "51 minor versions" or "failed every
call" against anything in this session (no prior `gh` version is
observable now, and no failure log is cited) — these are plausible,
first-person session claims from the plan's authoring session, not
independently checkable by me. Recorded PLAUSIBLE, not dropped.

**What a correction would be:** Tag the historical claim explicitly, e.g.
"**[measured, this session, before the upgrade]** `gh` was 51 minor
versions behind..." — or, if it is being carried from memory of the
session rather than a captured version string, say so per the plan's own
labeling convention.

## What I verified as accurate (no finding — stated per the contract: a truthful "this checks out" is a successful result)

- `tmpfs /tmp ... 15.3G ... usrquota` (§6, line 146-148): **VERIFIED exact**
  — `findmnt /tmp -o SIZE,AVAIL,USED,FSTYPE,OPTIONS` on this host reports
  `15.3G ... usrquota` verbatim.
- "258 normal dependencies" (§4, line 82): **VERIFIED exact** —
  `cargo tree --edges normal --prefix none | sort -u | wc -l` → 258
  (metadata resolution only, no build, so this doesn't touch the §4
  non-goal on binary-size/compile-time re-derivation).
- "5 of the 10 crates already present — `bitflags`, `libc`,
  `linux-raw-sys`, `memchr`, `rustix`... genuinely new: `fs4`, `sysinfo`,
  `directories`, `dirs-sys`, `option-ext`" (§4, lines 83-85): **VERIFIED
  consistent** — all five "already present" names are in `Cargo.lock`;
  none of the five "genuinely new" names are direct deps in `Cargo.toml`.
- `fs4`'s Unix path uses `rustix` for both `flock` and `statvfs` (§4, line
  75): **VERIFIED** against the cached crate source at
  `~/.cargo/registry/src/.../fs4-0.13.1/{src/unix.rs,src/lib.rs,Cargo.toml}`
  — `rustix::fs::flock`, `rustix::fs::FlockOperation`, and a `rustix = "1"`
  dependency edge are all present.
- `sysinfo` exposes `process(pid)`, `start_time()`, `cmd()` as public API
  (§4, line 77): **VERIFIED** — all three found as `pub fn` in
  `~/.cargo/registry/src/.../sysinfo-0.37.2/src/common/system.rs`.
- `directories::ProjectDirs` documents `~/Library/Application Support` for
  macOS (§4, line 76): **VERIFIED** — appears repeatedly in
  `~/.cargo/registry/src/.../directories-6.0.0/src/lib.rs`'s doc comments.
- §10's framing that macOS crate behavior is "cheap, not measured" and §4's
  "not measured, and it is the number that matters for #18" (`sysinfo`
  runtime cost): honestly labeled throughout — I did not find a place
  where a documentation-sourced crate claim was asserted as if it had been
  measured.
- #90 ("ceiling interrupt wedges a Work in `active` with no verb that
  reaches it", §6 line 138) and #94 ("`finalize_commit` ≠ base and `git log
  base..sergeant/<id>` non-empty... exact mechanism", §7 lines 161/171):
  **VERIFIED** against `docs/adr/0007-actor-runtime-contract.md` (D3, parts
  a/b) and `close-out.md`'s "#90 (ceiling interrupt wedges a Work with no
  exit door)" — both accurate characterizations, correctly dated
  2026-08-14.
- The retrospective quotes at §5 ("§4 of the cross-platform retrospective
  requires: 'anything that survives as a commit is a Work, with no
  exception for size'") and §6 (the §2.4 bullet list: prior-art-as-settled,
  trailer-naming, the two overridden-briefs precedent, the backtick/command-
  substitution incident): **VERIFIED exact**, all found verbatim in
  `retrospective.md` §2.4 and §4.
- ADR 0005 D1's quoted line, *"Gating becomes a dispatched Work: Captain
  adjudicates findings; Sgt executes"* (§2, R8): **VERIFIED exact**.

## PLAUSIBLE (unable to confirm or refute)

- **#96's characterization** ("operator surfaces", §3/§5): no independent
  description of #96 exists anywhere in this repo's tracked docs
  (`grep -rn "#96\b"` outside `plan.md` found only a passing group mention
  in the cross-platform `plan.md`, with no detail). `gh` cannot reach the
  tracker from this host. Cannot confirm or refute against the actual issue
  body. Recorded PLAUSIBLE per the method doc's fail-closed rule, not
  dropped.
- **The "3.90 s" clean-compile figure and its "0.02 s cached-rerun control"**
  (§4, lines 89-91): plausible and consistent with the L23 citation
  (silently-failed `cargo clean` producing a cache-hit timing), but I did
  not re-derive it (explicit non-goal). The exact command that would settle
  it: `cargo clean -p fs4 -p sysinfo -p directories -p dirs-sys -p
  option-ext && PATH="$HOME/.cargo/bin:$PATH" time cargo build --release
  --timings` run twice back to back.
- **The 64,023,360 B current binary size** (§4, line 82): no
  `target/release/sgt` exists in this worktree to spot-check without a full
  release build (~10 min cold per `docs/DEVELOPMENT.md`'s own note on
  DuckDB), which is both the §4 non-goal and would burn most of this
  session's budget on one figure. The exact command that would settle it:
  `PATH="$HOME/.cargo/bin:$PATH" cargo build --release && strip
  target/release/sgt -o /var/tmp/sgt-stripped && stat -c%s
  /var/tmp/sgt-stripped` (plan doesn't state whether 64,023,360 B is
  stripped or unstripped — worth the correction to specify, separate from
  re-deriving the number itself).
