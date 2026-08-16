# Path-to-Mac sprint — plan

Authored 2026-08-15 (Cerberus) by the orchestrating session from a live
`grilling` interview.

**[v2] Revised in place 2026-08-15 after PATH-TO-MAC-1 graded v1 and sent §4
and §5 back.** The graded version is pinned at `d86885f`; every verdict and
correction is in `adjudication.md` beside this file. The substantive change:
**#18, #81 and #82 are already built** — v1 scoped a Work to build them. Per
`reference/notes/`' own convention, this document is revised in place with a
dated entry rather than superseded by a sibling.

Marks used below: **[measured]** = measured this session; a citation = read
from that artifact; anything else says it is a belief (**L15**).

---

## 1. Destination

Get sergeant-rs to where a MacBook Pro M3 Pro session can **arrive, measure,
close, and return** — not arrive and start debugging. Owner's framing: *"We
shouldn't carry land mines with us."*

The bar is **ADR 0001 (D8)**'s, not this plan's: `scripts/probe-env.sh` run
there and recorded in `docs/environments/macbook.md`, plus a full suite run
**with a published skip count**.

## 2. Owner rulings

| # | Ruling | Consequence |
|---|---|---|
| R1 | Portability work returns to Cerberus rather than being "debugging exercises" | **[v2] Largely moot — the work already shipped. See §4.** |
| R2 | **Parity**: crate on both platforms or neither | **[v2] Already satisfied.** The existing code is hand-rolled on *both* platforms, not split |
| R3 | Latest stable crate versions | Applies only to #85 now |
| R4 | #109: retain the dirty **state**, never the **directory**; `target/` never in scope | *"The journal is the real artifact."* |
| R5 | #85 is in scope | **[v2] The only net-new platform work** |
| R6 | #108 gets a start-of-run reaper | **[v2] CORRECTED — see below** |
| R7 | Sonnet for every Work | `--profile sonnet`, `sergeant.toml:11-14` |
| R8 | `DEVELOPMENT.md`'s "actor never invokes the gate" is stale evidence | Superseded by ADR 0005; the prose at `docs/DEVELOPMENT.md:70-73` still carries it |
| R9 | The orchestrator never merges to `main` | Integration branch + head PR |
| **R10** | **[v2] Verify #18/#81/#82 on the Mac; do not swap them for crates** | W1 shrinks to #85 alone |
| **R11** | **[v2] #108 and #113 are separate fixes, separately labeled** | Both in W3 |
| **R12** | **[v2] #109 is narrowed to R4's clause and says so** | #109 will not fully close |

**R6 corrected.** v1 justified a reaper for #108 with *"`Drop` does not survive
`SIGKILL`."* That reasoning is real but belongs to a **different** leak. The
enactability refuter settled it by reading `src/watch.rs`: `test_hold_wait`
returns an ordinary `Err(WatchError::TestHoldTimedOut)` and the dead-man test
is a plain `#[tokio::test]` that awaits, asserts, and returns. **No signal is
involved**, so `Drop` runs. #108 needs guard-shaped cleanup (`LESSONS.md` L21).
The reaper fixes the `SIGKILL`ed-rig leak, now filed as **#113**.

## 3. Scope

**Built here:** #90, #94 (engine honesty) · #108, #113, #95-guard (harness
hygiene) · #85 (filesystem-type detection) · #109-narrowed, #96 (operator
surfaces).

**Closed on the Mac by measurement, not built:** **#18, #81, #82.** All three
ship today with both platform arms implemented and unit-tested, each marked
`UNVERIFIED` on macOS. `src/platform/process.rs`'s own module doc: *"They close
#18 when someone measures them there, not when this lands."*

**Also the Mac's:** #95's clock choice — `perl -MTime::HiRes` vs
`python3 time.time_ns()` vs accepting millisecond resolution needs timing on
real hardware. W3 builds only the fail-loudly guard.

## 4. Platform state — corrected

**[v2] This section previously proposed adopting `fs4`/`sysinfo`/`directories`
to replace hand-rolled code. That premise was disproved.**

| Issue | State | Evidence |
|---|---|---|
| #18 | **Shipped**, both arms, unit-tested; macOS `UNVERIFIED` | `src/platform/process.rs`; `daemon.rs` and `backend/claude.rs` already route through it; only `tests/support/mod.rs:201` reads `/proc` directly |
| #81 | **Shipped**; the filed GNU-only defect is fixed | `src/platform/disk.rs:71` (GNU) and `:86` (POSIX `df -k <path>` + positional parsing, test-pinned) |
| #82 | **Shipped**, both conventions, unit-tested | `src/platform/data_dir.rs` |
| #85 | **Unbuilt** | No mount-parsing, filesystem-type, or `statfs`/`f_type` code anywhere in `src/` |

All three landed in `4eadc50`, "Build the platform boundary (ADR 0002) and move
three platform facts behind it."

**Root cause of v1's error is L18**, which names itself: *"R1's 'already
exists' includes the product you are building… the comparison list is a moving
target that every future pass re-derives from the product, never from
memory."* v1 was derived from the issue tracker, never from `src/platform/`.

**The objection v1 mis-cited.** v1 attributed *"adding a libc-binding
dependency for one syscall"* to ADR 0002 (D4). It is not there. It is a
**source-code comment**, verbatim, originally on `src/backend/docker.rs::free_space`
and now paraphrased at `src/platform/disk.rs:4-6`. Two critic seats called it
invented on greps that excluded `src/`; the fidelity refuter re-ran the command
and traced it through git history. The quote is real — the citation was wrong,
and there is **no ADR to refresh**.

`src/platform/disk.rs:1-21`, written 2026-08-14 — *one day before v1* — already
re-examines this and concludes **"It still holds."** R10 accepts that.

**If a crate is proposed for #85**, the brief must record a Ponytail rung
(`GAUNTLET.md` lines 14-15) and state why raw `rustix` was rejected:
**[measured]** `rustix` v1.1.4 is transitive-only and resolved **without the
`fs` feature**, so calling `rustix::fs::statvfs`/`flock` needs a `Cargo.toml`
line *and* a feature flag. "No new dependency edge" is false.

**Measurement reconciliation gap, stated rather than smoothed.**
**[measured]** three per-crate deltas (4.7 + 17.3 + 144 KiB) sum to 166.0 KiB
against a combined-build measurement of 161 KiB — a 5 KiB / 3% gap that is
**not yet explained**. The shared-transitive-dependency hypothesis was tested
and refuted: those crates are already in the baseline, and no LTO is
configured. The total is not silently re-summed — 161 KiB is a real combined
measurement; 166 KiB is a derived sum of three noisier ones. The command that
would settle it is in `refuters/assumptions.md`.

## 5. Waves

**[measured]** `src/runtime/surface.rs:332,431-441`: a work branch is cut from
the repository's **current HEAD**. Advancing the estate clone's HEAD between
waves means later Works cut from earlier results, so conflicts do not arise
rather than being resolved — required, because *"anything that survives as a
commit is a Work, with no exception for size."*

| Wave | Work | Scope | Cut from |
|---|---|---|---|
| 1 ‖ | **W2 · engine honesty** | #90, #94 | integration tip |
| 1 ‖ | **W3 · harness hygiene** | #108 guard, #113 reaper, #95 guard | integration tip |
| 2 | **W1′ · #85** | filesystem-type detection | wave-1 tip |
| 3 | **W4 · operator surfaces** | #109 (R4 clause only), #96 | wave-2 tip |
| 4 | **W5 · handoff + doc corrections** | Mac handoff, `macbook.md`, `cerberus.md`, `DEVELOPMENT.md:70-73`, the `disk.rs` citation | wave-3 tip |
| 5 | **W6 · gate** | `validate-and-ship` | integration tip |

W2 and W3 are disjoint (`src/runtime/` + `cli.rs` vs `tests/` + `scripts/perf/`).

## 6. Dispatch

`--repo sergeant-rs --profile sonnet --turns 40 --ceiling-secs 5400`.
**[v2]** No `--backend` flag: the estate manifest now declares
`default_backend = "claude"` (§13's WorkspaceDefault tier), so routing resolves
as `workspace_default` and the setting proves itself in the journal. v1 omitted
this and every dispatch 422'd.

The generous ceiling is deliberate: **#90's fix cannot protect its own sprint**
— W2 runs under the installed binary.

**Briefs are files, passed `"$(cat …)"`**, under `/var/tmp/`. **[measured]**
`/tmp` is a 16 GB tmpfs mounted `usrquota`, which corroborates #70's `EDQUOT`
signature.

Every brief carries: prior art named as **settled**; the commit trailer;
**why**, not only what; a pointer at `GAUNTLET.md`'s register (**L3**);
evidence-vs-belief labels (**L15**); `PATH="$HOME/.cargo/bin:$PATH"`; **run
long commands in the foreground** (#94's mechanism); **[v2]** `gh … --repo
miztertea/sergeant-rs` (#112); and **[v2]** for any journal-touching change,
the adjacent-append crash-window check (**L6**, `DEVELOPMENT.md:40`).

## 7. Orchestrator duties

- Watchers armed **in the same response** as the dispatch, backgrounded.
- **No "completed" believed on its own**: `finalize_commit` ≠ base **and**
  `git log base..sergeant/<id>` non-empty. That is #94's exact signature.
- **[v2] A wave Work landing `failed`:** retry once against the same base; if
  it fails again the wave **blocks** and Captain escalates to the owner. The
  next wave never proceeds from a stale base, because that would silently drop
  a Work's scope from the sprint.
- Gate findings: the Work may authorize `auto-fix`, records `no-op`, relays
  `ask-user` **verbatim**. An `ask-user` finding waits for the owner.
- **No merge to `main`** (R9).

## 8. Risks

1. **#90 is unprotected tonight** — mitigated only by envelope sizing.
2. **[v2]** The 5 KiB measurement gap is unexplained (§4).
3. **Review is Captain-serial.** ADR 0005's items 3–4 are unbuilt, so a gate
   Work cannot bind to another Work's branch. W6 sidesteps this by cutting from
   the integration tip, which already holds the content. That works here and
   does not generalize.
4. **[v2]** `#112` is worked around in this estate, not fixed.

## 9. How this plan was reviewed

`PATH-TO-MAC-1` — 4 blind critic seats, 4 per-axis refuters, all Sonnet, 1 turn
of 25 each. **22 findings · 18 confirmed · 6 PLAUSIBLE · 1 central claim
refuted · 5 severity moves.** §4 and §5 sent back. Full record in `critics/`,
`refuters/`, and `adjudication.md`.

## 10. What the Mac owns

- `scripts/probe-env.sh` → `docs/environments/macbook.md`.
- A full suite run **with a published skip count** (ADR 0001 D8).
- **Flipping `UNVERIFIED` on #18, #81, #82** — measurement, not implementation.
- Choosing #95's clock on real hardware.
- Whatever the above surfaces that this plan did not anticipate. That list
  being non-empty is the expected outcome.
