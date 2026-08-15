# #128 — submission throughput floor on Apple M3 Pro

WC investigation, `macbook-arrival-2026-08-15` sprint. Governing:
`docs/gauntlet/runs/macbook-arrival-2026-08-15/plan.md` §3/§8; the floor
(t12, `m2_daemon_api`) is derived from A-N3-1's amended 24 works/s at burst
50 divided by 2 for a shared test host.

**Headline: root cause isolated, partially mitigated, floor not met.**

| metric | baseline | fix 1+2 (f25de06) | fix 3 (`--no-checkout`) | verdict |
|---|---|---|---|---|
| t12 throughput, burst 25 | 4.8 works/s | 10.5 works/s | **11.6 works/s** | **fails floor (≥ 12)** |
| git calls under per-repo lock | 7 per work | 2 per work | 2 per work | unchanged |
| lock-held time per add | ~57 ms | ~57 ms | **~43 ms** | improved |
| git calls per work total | 7 | 7 | 7 | unchanged |

Commits: `surface.rs` — pre-lock reads in `materialize_one` and
`teardown_binding`, redundant `head_sha` re-read eliminated; `--no-checkout`
optimization moving file population outside the per-repository lock.

---

## Root cause

macOS process spawn overhead. Every call to `git` costs roughly **28 ms
warm** on this M3 Pro; `git worktree add` costs roughly **52 ms** (it
creates a directory, writes the registry entry, and checks out the commit).
Linux containers return the same numbers at **~5–10 ms**.

Before this investigation the per-repository lock (`with_repository`,
`src/runtime/surface.rs`) held 7 git spawns per work — all of them:

| call | purpose | ms |
|---|---|---|
| `git rev-parse HEAD` | base SHA | 28 |
| `git symbolic-ref HEAD` | base branch | 28 |
| `git worktree add -b <branch> <path> <sha>` | create worktree | 52 |
| `git rev-parse HEAD` *(in worktree)* | verify head_sha | 28 |
| `git rev-parse refs/heads/<branch>` | branch tip for record | 28 |
| `git status --porcelain` | dirty check | 28 |
| `git worktree remove <path>` | registry cleanup | 28 |

7 × 28 ms average = **196 ms** per work held under a single-threaded lock,
times 25 concurrent submissions against the same repository:
25 × 196 ms = **4900 ms** minimum, measured at 5208 ms → **4.8 works/s**.

---

## What was moved, and why it was safe to move

### Fix 1 — `materialize_one`

`git rev-parse HEAD` and `git symbolic-ref HEAD` read `.git/HEAD` and ref
storage only. They do not touch `.git/worktrees/`, which is the registry
`with_repository` protects. Moving them before the lock lets concurrent
submissions for the same repository do those reads in parallel.

The fourth call — `git rev-parse HEAD` run *in the new worktree* — was
removed entirely. `git worktree add -b <branch> <path> <sha>` creates the
branch at `<sha>` and checks it out, so the new worktree's HEAD is
`base_sha` by construction. No post-checkout hook is expected to repoint it.
`head_sha = base_sha.clone()` is recorded instead.

TOCTOU: `base_sha` is passed explicitly to `git worktree add`, not as
`"HEAD"`, so a concurrent push between the read and the add does not affect
what is checked out — the worktree lands at the recorded commit regardless.

Result: 4 git calls under the lock → 1 (the worktree add alone).
**4.8 → 6.7 works/s.**

### Fix 2 — `teardown_binding`

`git rev-parse refs/heads/<branch>` reads the branch tip in ref storage, not
the registry. `git status --porcelain` runs in the linked worktree, reading
the worktree-local index and the shared object store (read-only). Neither
touches `.git/worktrees/`.

Both were moved before `with_repository`. Only `git worktree remove` (and
its `--force` retry for submodule-bearing worktrees) remains under the lock.

TOCTOU on the dirty check: if the worktree becomes dirty between the status
read and the remove, `git worktree remove` (without `--force`) refuses the
removal — the same fail-closed outcome the explicit check produces. No
content risk in the window.

The old `teardown_binding_locked` helper was inlined and removed; both
functions are now one.

Result: 3 git calls under the lock → 1 (the worktree remove alone).
**6.7 → 10.5 works/s.**

### Fix 3 — `--no-checkout` (`materialize_one`)

`git worktree add` checks out the commit into the working directory as part
of its single git invocation. That checkout — reading tree objects from the
object store and writing files to the worktree directory — takes roughly
14 ms of the ~57 ms the call costs, and it runs inside the per-repository
lock even though it touches neither `.git/worktrees/` nor any other
shared-registry state.

`git worktree add --no-checkout` writes only the registry entry
(`.git/worktrees/<name>/`) and exits. A subsequent `git checkout HEAD` in the
new worktree populates the files outside the lock, running concurrently with
the next submission's `worktree add --no-checkout`.

Timing on this machine (5 runs):
```
add --no-checkout: 38–49 ms  avg 43 ms   (vs 50–57 ms for full add, avg 57 ms)
git checkout HEAD: 44–55 ms  avg 50 ms   (runs outside lock, concurrent)
```

The lock-held time for the add drops by ~14 ms per work. For 25 serialized
submissions against the same repository: 25 × 14 ms = **350 ms reduction** in
the theoretical serialized chain; measured improvement ~**230 ms** (67 %
realisation — the pipeline sees partial overlap but the startup ramp and
scheduling granularity absorb some of the gain).

Correctness: `--no-checkout` still creates the branch at `base_sha` and sets
HEAD in the new worktree to that branch, so `checkout_worktree` populates
exactly the same content a full add would have. The checkout completes before
`materialize_one` returns; no caller ever sees a worktree with its branch set
but its files absent.

`head_sha` is recorded as `base_sha.clone()` (unchanged from fix 1, where the
redundant in-worktree `rev-parse HEAD` was already eliminated).

Result: lock-held add time ~57 ms → **~43 ms**.
**10.5 → 11.6 works/s.**

---

## Why the floor still cannot be met after all three fixes

After all three fixes the per-repo lock holds exactly two git calls:

| call | why it must be under the lock | warm time |
|---|---|---|
| `git worktree add --no-checkout -b <branch> <path> <sha>` | writes `.git/worktrees/<name>/` registry entry | 38–49 ms |
| `git worktree remove <path>` | deletes `.git/worktrees/<name>/` registry entry | 28–42 ms |

These cannot be moved outside the lock. `git worktree add` reads all
existing registry entries to allocate a unique name; `git worktree remove`
deletes one of them. A concurrent add and remove against the same `.git` can
collide: the add reads the entry the remove is mid-deleting and fails with
`fatal: failed to read .git/worktrees/<other-work>/commondir`. This is not
theoretical — it is the exact failure the per-repo lock was introduced to
prevent, measured at 2 failures in 51 attempts in the N3 wave-2 run
(N3 two-phase-boundary doc, "A bug the fix uncovered").

A pure-Rust worktree registry manipulation (bypassing the `git` subprocess
entirely) would eliminate the subprocess overhead, but §11 explicitly
forbids re-implementing git internals: "shell out to the installed Git rather
than embedding libgit2."

**Arithmetic of the remaining critical path (after fix 3):**

Measured (add --no-checkout + remove), warm, 5 runs on this machine:

```
 1: add=49 ms  remove=31 ms  total=80 ms
 2: add=38 ms  remove=38 ms  total=76 ms
 3: add=46 ms  remove=42 ms  total=88 ms
 4: add=44 ms  remove=32 ms  total=76 ms
 5: add=38 ms  remove=32 ms  total=70 ms
avg: add=43 ms  remove=35 ms  total=78 ms
```

25 works × 78 ms = **1950 ms** in the serialized lock chain.
Budget for 12 works/s: 25 / 12 = **2083 ms** total.

After fix 3, **the lock chain alone now fits within the budget** (1950 ms <
2083 ms). The bottleneck has shifted: it is no longer the lock chain itself
that exhausts the budget, but the overhead above it:

- `git checkout HEAD` (~50 ms per work, outside the lock but on the critical
  path between `worktree add` and the worktree being usable by the runner).
  In steady state this overlaps with the next work's add, but the pipeline
  startup/teardown ramp and scheduling granularity absorb part of the benefit.
- HTTP round-trips, Tokio scheduling, journal appends, core-lock handoffs.

Measured after all three fixes: **2128–2177 ms → 11.5–11.7 works/s**.
Remaining gap to floor: ~45–94 ms (2–5 %) above the 2083 ms budget.

For comparison, after fix 2: **2380–2430 ms → 10.3–10.5 works/s**.
Fix 3 saves ~230 ms from the critical path.

**Historical comparison across all three fixes:**

| fix | lock time/work | total locked | wall time (25 works) | throughput |
|---|---|---|---|---|
| baseline | ~196 ms | ~4900 ms | 5208 ms | 4.8 works/s |
| fix 1 (pre-lock rev-parse/symbolic-ref) | ~52 ms | ~1300 ms | ~3731 ms | 6.7 works/s |
| fix 2 (pre-lock status/branch-tip) | ~52 ms | ~1300 ms + 35 ms/teardown | ~2380 ms | 10.5 works/s |
| fix 3 (--no-checkout) | ~78 ms | ~1950 ms | ~2150 ms | 11.6 works/s |

---

## What was not tried and why

| candidate | why rejected |
|---|---|
| Implement `git worktree add`/`remove` in Rust via direct file I/O | Violates §11 |
| Deferred teardown (return `completed` before worktree remove) | Significant engine/journal change with untested L6 crash windows; outside this pass's risk envelope |
| `git read-tree -u HEAD` instead of `git checkout HEAD` (fix 3 variant) | Tested: does not populate working-directory files in a `--no-checkout` worktree; the index is updated but the tree objects are not written to disk |
| Per-work repository clone (each work gets a fresh clone, no lock needed) | `git clone` at ~300 ms each is far more expensive than the lock |
| Revising the floor with measured justification | The measurement is honest; the proposal belongs to the contract owners, not to this pass |

---

## Recommendation for forward passes

After fix 3 the lock chain (1950 ms) fits within the budget (2083 ms), but
the measured wall time (2128–2177 ms) still exceeds it by 45–94 ms. The gap
is no longer structural (the lock chain itself) — it is scheduling overhead
plus the `git checkout HEAD` call which sits between the lock and the
worktree being usable. Three options:

1. **Revise the floor with a hardware annotation.** The linux-container
   baseline (A-N3-1: 24 works/s at burst 50, i.e. 12 works/s floor) remains
   valid. A separate macOS annotation could state the floor at 11.5 works/s
   (the measured plateau after all three optimizations), with a note that the
   remaining gap is git process-spawn overhead, not architectural regression.

2. **Gate t12 only on the CI container.** The test currently runs on every
   `cargo test` invocation, regardless of platform. Adding a `#[cfg_attr(target_os = "macos", ignore)]`
   or a `CARGO_CFG_TARGET_OS` guard would let the floor defend its intended
   budget on Linux without false-failing on macOS.

3. **Accept the gap.** The 2.4× improvement (4.8 → 11.6 works/s) from all
   three fixes is real value. A note in the test's failure message ("this
   floor is written against Linux container timing; macOS git-spawn overhead
   makes it unreachable on M-series hardware") would document the gap honestly
   without changing the floor.

This pass implements options 1 and 3 only to the extent of this document —
it does not edit the floor or the test. A floor revision requires a ruling
from the contract owners with its own adjudication trail.
