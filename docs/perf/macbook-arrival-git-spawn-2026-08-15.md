# #128 — submission throughput floor on Apple M3 Pro

WC investigation, `macbook-arrival-2026-08-15` sprint. Governing:
`docs/gauntlet/runs/macbook-arrival-2026-08-15/plan.md` §3/§8; the floor
(t12, `m2_daemon_api`) is derived from A-N3-1's amended 24 works/s at burst
50 divided by 2 for a shared test host.

**Headline: root cause isolated, partially mitigated, floor not met.**

| metric | before | after | verdict |
|---|---|---|---|
| t12 throughput, burst 25 | 4.8 works/s | 10.5 works/s | **fails floor (≥ 12)** |
| git calls under per-repo lock | 7 per work | 2 per work | improved |
| git calls per work total | 7 | 7 (same count, different placement) | unchanged |

Commits: `surface.rs` — pre-lock reads in `materialize_one` and
`teardown_binding`, redundant `head_sha` re-read eliminated.

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

---

## Why the floor cannot be met with the current architecture on this hardware

After both fixes the per-repo lock holds exactly two git calls:

| call | why it must be under the lock | warm time |
|---|---|---|
| `git worktree add -b <branch> <path> <sha>` | writes `.git/worktrees/<name>/` registry entry | 50–57 ms |
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

**Arithmetic of the remaining critical path:**

Measured combined (add + remove), warm, 5 runs on this machine:

```
 1: add=57 ms  remove=31 ms  total=88 ms
 2: add=45 ms  remove=38 ms  total=82 ms
 3: add=56 ms  remove=42 ms  total=98 ms
 4: add=54 ms  remove=32 ms  total=86 ms
 5: add=40 ms  remove=32 ms  total=72 ms
avg: add=50 ms  remove=35 ms  total=85 ms
```

25 works × 85 ms = **2125 ms** in the critical section alone.
Budget for 12 works/s: 25 / 12 = **2083 ms** total.

The critical section by itself already exceeds the total budget. Even if
every other cost — workspace discovery, HTTP round-trip, FakeBackend
processing, journal writes, core-lock acquisition — were instantaneous,
the test cannot pass on this hardware.

Measured after both fixes: **2380–2430 ms → 10.3–10.5 works/s**. The ~300 ms
of overhead above the 2125 ms critical section is largely the non-lock git
calls running concurrently (plan phase `rev-parse --show-toplevel`, the
pre-lock reads for works still queued), plus Tokio scheduling, TCP round-trips,
and journal appends.

---

## What was not tried and why

| candidate | why rejected |
|---|---|
| Implement `git worktree add`/`remove` in Rust via direct file I/O | Violates §11 |
| Deferred teardown (return `completed` before worktree remove) | Significant engine/journal change with untested L6 crash windows; outside this pass's risk envelope |
| `git worktree add --no-checkout` + separate checkout outside the lock | Saves ~8 ms under the lock (checkout is ~25 ms, moves outside); 25 × 8 ms = 200 ms, not enough to cross the gap |
| Per-work repository clone (each work gets a fresh clone, no lock needed) | `git clone` at ~300 ms each is far more expensive than the lock |
| Revising the floor with measured justification | The measurement is honest; the proposal belongs to the contract owners, not to this pass |

---

## Recommendation for forward passes

The floor as written (≥ 12 works/s at burst 25) is not reachable on Apple
M-series hardware with the current git-subprocess design. Three options:

1. **Revise the floor with a hardware annotation.** The linux-container
   baseline (A-N3-1: 24 works/s at burst 50, i.e. 12 works/s floor) remains
   valid. A separate macOS annotation could state the floor at 10 works/s
   (the measured plateau after both optimizations), with a note that the gap
   is git process-spawn overhead, not architectural regression.

2. **Gate t12 only on the CI container.** The test currently runs on every
   `cargo test` invocation, regardless of platform. Adding a `#[cfg_attr(target_os = "macos", ignore)]`
   or a `CARGO_CFG_TARGET_OS` guard would let the floor defend its intended
   budget on Linux without false-failing on macOS.

3. **Accept the gap.** The 2.2× improvement (4.8 → 10.5 works/s) from
   moving reads outside the lock is real value. A note in the test's failure
   message ("this floor is written against Linux container timing; macOS
   git-spawn overhead of ~85 ms/add+remove makes it unreachable") would
   document the gap honestly without changing the floor.

This pass implements options 1 and 3 only to the extent of this document —
it does not edit the floor or the test. A floor revision requires a ruling
from the contract owners with its own adjudication trail.
