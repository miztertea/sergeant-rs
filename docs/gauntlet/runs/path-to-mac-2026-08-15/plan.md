# Path-to-Mac sprint — plan

Authored 2026-08-15 (Cerberus) by the orchestrating session, from a live
`grilling` interview with the owner. Per **L19**, this document directs what
gets built and is therefore executable through the program that obeys it: it
takes the review loop before it governs. The reviewing pass is a dispatched
`code-review` Work against this file's own diff — see §9.

Provenance of every ruling below is the interview itself; nothing here was
produced by a workflow run. Where a claim is a *measurement taken this
session*, it is marked **[measured]**; where it is read from a repo artifact,
the artifact is cited; where it is a belief, it says so (**L15**).

---

## 1. Destination

Get sergeant-rs to the point where a MacBook Pro M3 Pro session can **arrive,
measure, close, and return** — not arrive and start debugging the estate. The
Mac trip's output is `docs/environments/macbook.md` plus closed portability
issues; anything that would strand a session there is a landmine to be removed
on Cerberus first.

Owner's framing, verbatim: *"We shouldn't carry land mines with us, we want to
go there, measure and close and get back to Cerberus."*

The bar for "measured on macOS" is not this document's to set — **ADR 0001
(D8)** already sets it: `scripts/probe-env.sh` run there and recorded in
`docs/environments/macbook.md`, plus a full suite run **with a published skip
count**. This sprint exists to make both reachable in one sitting.

## 2. Owner rulings taken in the interview

These are decisions, not derivations. Each was put to the owner and answered.

| # | Ruling | Consequence |
|---|---|---|
| R1 | #18, #81, #82 come back to Cerberus rather than being "debugging exercises" on the Mac | The crate answers already exist; rediscovering them on the Mac is reimplementing `fs4` worse |
| R2 | **Parity**: crate on both platforms or neither — no hand-roll on Linux with a crate on macOS | Kills the "keep `/proc` on Linux if it's faster" split this plan originally proposed |
| R3 | Latest stable crate versions, pinned at what is actually current | Measurement below used older resolutions; the real pin is the Work's to take |
| R4 | #109: retain the dirty **state**, never the **directory**; `target/` is never in scope | *"The journal is the real artifact. Let's keep the disk clean."* Turns 30 GB into megabytes |
| R5 | #85 is **in** scope, on R2's reasoning | Its macOS half was "unmeasured" only under the hand-roll assumption |
| R6 | #108 is fixed by a **start-of-run reaper**, using standard patterns | `Drop` does not survive `SIGKILL`, and SIGKILL is how these die |
| R7 | Sonnet for every dispatched Work | `--profile sonnet`, declared at `sergeant.toml:4-7` |
| R8 | `DEVELOPMENT.md`'s "an actor never invokes the gate" rule is **stale evidence, not law** | Superseded by ADR 0005; the gate is a dispatched Work |
| R9 | The orchestrator never merges to `main` | Integration branch + head PR; the owner merges |

**R8 is a correction to a governing document and is the sharpest item here.**
`docs/DEVELOPMENT.md` line 71 states the rule; **ADR 0005 (Accepted,
2026-08-14)** dissolved it — *"Gating becomes a dispatched Work: Captain
adjudicates findings; Sgt executes."* The prose contradicts an accepted ADR.
This is **L20**'s shape for the second recorded time: the prose was true when
written (there was no `sgt` able to run a gate), nothing contradicted it, so
nothing fired. Superseding that line is owed by this sprint; it is assigned to
**W5**.

## 3. Scope

**In:** #18, #81, #82, #85 (platform crates) · #90, #94 (engine honesty) ·
#96, #109 (operator surfaces) · #95, #108 (harness hygiene).

**Out, and deliberately so:** nothing from the portability set is deferred to
the Mac any more. What the Mac still owns is **verification** — that the
crates' cross-platform claims hold on a real host — plus `#95`'s clock choice,
which needs timing on real hardware to pick between `perl -MTime::HiRes`,
`python3 time.time_ns()`, and accepting millisecond resolution.

## 4. The crate adoption, with measurements

Researched via Context7 against the crates' own documentation rather than from
training data, at the owner's direction.

| Issue | Crate answer | Replaces |
|---|---|---|
| #81 | `fs4::available_space` / `statvfs` — Unix path is `rustix` v1 → `statvfs(2)` | `df -k --output=avail` shell-out (GNU-only) |
| #82 | `directories::ProjectDirs` — Linux `$XDG_DATA_HOME`, macOS `~/Library/Application Support` | hand-rolled freedesktop-specific tail |
| #18 | `sysinfo` — `process(pid)`, `start_time()`, `cmd()` | direct `/proc` reads in `daemon.rs`, `backend/claude.rs`, `tests/support/mod.rs` |
| #85 | `fs4` for portable `flock(2)`; `sysinfo` Disks API for per-mount filesystem type | `/proc/mounts` parsing (Linux-only) |

**[measured] this session, on Cerberus:**

- Current `target/release/sgt`: **64,023,360 B**; 258 normal dependencies.
- Of the 10 crates in the candidate tree, **5 are already present** —
  `bitflags`, `libc`, `linux-raw-sys`, `memchr`, and **`rustix`**. Genuinely
  new: `fs4`, `sysinfo`, `directories`, `dirs-sys`, `option-ext`.
- Stripped release binary delta, attributed: `fs4` **+4.7 KiB**,
  `directories` **+17.3 KiB**, `sysinfo` **+144 KiB**; all three **+161 KiB**
  — **0.26%** of the shipped binary.
- Clean single-threaded compile of all five new crates: **3.90 s** (verified
  against a cached-rerun control at 0.02 s, because the first timing was a
  cache hit from a silently-failed `cargo clean` — **L23**).

**This retires ADR 0002 (D4)'s objection on its own terms.** That decision
declined "adding a libc-binding dependency for one syscall." `libc` and
`rustix` are already dependency edges this crate carries, and the count is now
four facts rather than one. **An ADR refresh is owed** — this repo's gate
caught ADR staleness three times in the last sprint, and R8 above is the same
failure one document over. Assigned to **W1**.

**Not measured, and it is the number that matters for #18:** `sysinfo`'s
*runtime* cost for a single-pid refresh versus the current `/proc` read. Binary
size is not the risk; #12 (doctor's ~450 ms floor) and #10 (cold-call latency
scaling) are. W1's acceptance requires measuring it. Under R2, a slower result
does **not** license a Linux/macOS split — it licenses raising the finding.

## 5. Waves

Waves exist because of **file ownership**, not logic. Three Works editing
`Cargo.toml` would conflict by construction.

**[measured]** `src/runtime/surface.rs:332` and `431-441`: a work branch is cut
from the repository's **current HEAD**. `repos/sergeant-rs` is a separate clone
(origin = the main checkout, no GitHub remote), clean on `main` at `242abe3`.
Advancing *its* HEAD to the integration tip between waves means every later
Work cuts from the previous wave's result — so conflicts do not arise rather
than being resolved. This costs wall-clock and buys zero hand-authored merges,
which **§4 of the cross-platform retrospective** requires: *"anything that
survives as a commit is a Work, with no exception for size."*

| Wave | Work | Scope | Cut from |
|---|---|---|---|
| 1 ‖ | **W1 · platform crates** | #81, #82, #18, #85; ADR 0002 refresh | `main` |
| 1 ‖ | **W3 · harness hygiene** | #108 (start-of-run reaper), #95 | `main` |
| 2 | **W2 · engine honesty** | #90, #94 | wave-1 tip |
| 3 | **W4 · operator surfaces** | #109 (R4), #96 | wave-2 tip |
| 4 | **W5 · handoff + governing-doc corrections** | Mac handoff, `macbook.md` skeleton, `cerberus.md` re-measurement, R8's supersession | wave-3 tip |
| 5 | **W6 · gate** | `validate-and-ship` | integration tip |

W1 and W3 are the only parallel pair; their file sets are disjoint (`src/` +
`Cargo.toml` vs `tests/` + `scripts/perf/`).

## 6. Dispatch parameters

Every Work: `--repo sergeant-rs --profile sonnet --turns 40 --ceiling-secs 5400`.

The generous ceiling is deliberate and its reasoning is uncomfortable: **#90's
fix cannot protect its own sprint.** A ceiling interrupt wedges a Work in
`active` with no verb that reaches it, and W2 runs under the currently
installed binary. Fewer ceiling firings is the only lever available tonight.
The cost is that a genuinely stuck turn burns 90 minutes before it surfaces.

**Briefs are files, passed as `"$(cat <path>)"`**, under `/var/tmp/`. Two
reasons: backticks in an inline intent previously caused command substitution
to execute `sgt work list` and embed its output into a brief (retrospective
§2.4); and **[measured]** `/tmp` on this host is still a 16 GB tmpfs
(`tmpfs /tmp tmpfs 15.3G rw,nosuid,nodev,...,usrquota`) — re-measured this
session at the owner's challenge, and the `usrquota` mount option corroborates
#70's `EDQUOT` signature rather than `ENOSPC`.

Every brief carries, per retrospective §2.4's *confirmed* approaches:

- the prior art named as **settled**, so turns are not spent re-deriving it;
- the commit trailer stated explicitly (`Refs` vs `Fixes`);
- **why**, not only what — two Works last sprint overrode their briefs on
  technical grounds and were right both times; a brief that only issues
  instructions forecloses that;
- a pointer at `GAUNTLET.md`'s deviation register (**L3**);
- evidence-vs-hypothesis labels on every factual claim (**L15**);
- `PATH="$HOME/.cargo/bin:$PATH"` — cargo is not on non-interactive PATH here;
- **run long commands in the foreground.** A headless turn has no
  background-completion callback. This is the exact mechanism of #94.

## 7. Orchestrator duties

- **Watchers armed in the same response as the dispatch**, backgrounded from
  the start. Retrospective §2.2: "I forgot to set a monitor" is the wrong
  framing — the monitor was armed *after* the wait began, so anything landing
  in the gap was invisible. §2.3: any wait that *might* exceed ten minutes is
  backgrounded rather than promoted after it stalls.
- **No "completed" is believed on its own.** For each Work: `finalize_commit`
  ≠ base **and** `git log base..sergeant/<id>` non-empty. This is #94's exact
  signature, observed live on this repo on 2026-08-14.
- **Gate findings**: ADR 0005 leaves `validate-and-ship`'s authority split
  intact. The Work may authorize `auto-fix`, records `no-op`, and relays
  `ask-user` **verbatim**. An `ask-user` finding waits for the owner; it is
  never resolved autonomously.
- **No merge to `main`** (R9). The integration branch carries a head PR opened
  early, so the owner watches it accumulate rather than meeting it finished.

## 8. Risks, stated rather than hidden

1. **#90 is unprotected tonight.** Mitigated only by envelope sizing (§6).
2. **`sysinfo` runtime cost is unmeasured** (§4). Could make #12/#10 worse.
3. **Review is Captain-serial.** ADR 0005's items 3 and 4 — a submission shape
   for "this Work reviews that Work" — remain unbuilt, so a gate Work cannot
   be bound to another Work's branch. This sprint sidesteps it: W6 cuts from
   the integration tip and therefore already holds the content, never needing
   `surface::attach`. That works here and does not generalize.
4. **`gh` was 51 minor versions behind** and `gh pr edit` failed every call on
   2.46.0. **[measured]** upgraded this session to 2.97.0 from GitHub's own apt
   repo; `cerberus.md`'s row is now stale and is W5's to supersede.

## 9. How this plan is reviewed

Committed to `integration/path-to-mac-2026-08-15`, then graded by a dispatched
**`code-review`** Work against this file's diff — two non-contaminating axes,
Standards and Spec, reported side by side and never merged or reranked.

Spec sources for the Spec axis, in priority order: `NORTH-STAR.md`, ADR 0001 /
0002 / 0005, `docs/DEVELOPMENT.md`, `LESSONS.md`, `GAUNTLET.md`'s deviation
register and backlog.

**Known narrowing, recorded rather than smoothed over:** `code-review` gives
two axes. FOUNDATION-1 graded a comparable artifact — a proposal rather than an
implementation — with a four-axis panel plus per-axis adversarial refuters, and
its own method note records that the three axes whose refuter was given a
*specific line of attack* produced the unit's only refutation and all three
severity downgrades. A two-axis pass is the published workflow and is what runs
here; it is weaker than that precedent, and this paragraph exists so nobody
later mistakes "reviewed" for "reviewed to FOUNDATION-1's depth."

## 10. What the Mac still owns

- Running `scripts/probe-env.sh` and recording `docs/environments/macbook.md`.
- A full suite run **with a published skip count** (ADR 0001 D8).
- Verifying that `fs4`, `sysinfo`, and `directories` behave on macOS as their
  documentation claims — the crates make the claim cheap, they do not make it
  measured.
- Choosing #95's clock on real hardware.
- Anything the above surfaces that this plan did not anticipate. That list
  being non-empty is the expected outcome, not a failure of this plan.
