# PATH-TO-MAC-1 — enactability refuter

## Method note

Read, in order: `docs/gauntlet/contracts/PATH-TO-MAC-1.md`; the plan at
`docs/gauntlet/runs/path-to-mac-2026-08-15/plan.md` pinned to `d86885f`; and
**only** `docs/gauntlet/runs/path-to-mac-2026-08-15/critics/enactability.md`
(this axis's own critic file — the other three axes' critic files were not
opened). Default posture is skepticism: every finding below was independently
re-derived from primary sources, not accepted on the critic's say-so.

Given a specific line of attack (per the contract's requirement that every
refuter get one): test the critic's stated reason for declining to flag R6
against the actual test code, and re-verify F1 independently.

**What I ran**, all read-only except one local `git merge --ff-only` to pull
the already-published critic files into this worktree (no content mutated,
no artifact under review touched):

- `sed -n` / `grep -n` against `src/watch.rs` — read `test_hold_wait`
  (lines 472–486) and both tests reaching it (lines 739–780) in full.
- `gh issue view {90,94,95,108,109,10,12} --repo miztertea/sergeant-rs
  [--comments]` — full issue bodies, matched against the critic's quotes.
- `grep -n "fails closed into" docs/DEVELOPMENT.md` — line-number check on
  the critic's DEVELOPMENT.md:40 citation.
- `git show d86885f:docs/gauntlet/runs/path-to-mac-2026-08-15/plan.md` —
  read the pinned plan directly rather than trusting the critic's quotes.
- `grep -n` against `docs/gauntlet/runs/cross-platform-2026-08-14/retrospective.md`
  §1.3/§3.3 — traced where R6's stated justification actually originates.
- `gh issue view 90` **with no `--repo` flag**, plus `GH_DEBUG=api gh issue
  view 90` (no flag), plus `git remote -v`, repeated in this worktree and in
  three sibling worktrees (`repos/sergeant-rs`,
  `.sergeant/data/surfaces/01M01ZKWPA5KT74WYPJ6DD73K2/sergeant-rs`,
  `.sergeant/data/surfaces/01M01ZKWW96MMH67RE5M5YJMRS/sergeant-rs`,
  `.sergeant/data/surfaces/01M01ZKX8SW05QJWB8P3C4H3E2/sergeant-rs`) — testing
  F5 by running the command, not reasoning about it.
- `grep -n "wave.*retry\|retry.*wave\|Work.*failed.*wave\|wave.*failed"` over
  `GAUNTLET.md` and the cross-platform retrospective — re-running the
  critic's own absence check for F6.

No mutation of the plan or any critic file. No worktree probe was needed
(nothing here proposes a code change).

---

## R6/#108 — UPGRADED: the critic's "not a finding" verdict rests on a false premise

The critic's own words, in "What I checked and found nothing on": *"a
start-of-run reaper... is not a contradiction: ... this is a coherent, arguably
more robust alternative precisely because (per R6's own stated reasoning) an
RAII guard's `Drop` would not survive a `SIGKILL`'d test process either."*

**This is factually wrong about #108's actual mechanism, verified by reading
the code, not by reasoning about it.**

`src/watch.rs:472–486`, `test_hold_wait`:

```rust
async fn test_hold_wait(path: String, deadman: Duration, poll: Duration) -> Result<(), WatchError> {
    let ready = format!("{path}.ready");
    let _ = std::fs::write(&ready, b"");
    let deadline = Instant::now() + deadman;
    loop {
        if std::path::Path::new(&path).exists() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(WatchError::TestHoldTimedOut { path });
        }
        tokio::time::sleep(poll).await;
    }
}
```

And the dead-man test itself, `src/watch.rs:739–774`
(`test_hold_wait_times_out_with_a_distinct_error_when_the_release_path_never_appears`):
a plain `#[tokio::test]` that `.await`s the call above, gets back
`Err(WatchError::TestHoldTimedOut { .. })`, asserts on the path field, and
returns. **No signal is involved anywhere in this path.** The loop polls,
hits its own internal deadline, returns an ordinary `Err`, and the test
function completes normally — this is exactly the designed dead-man branch
(the comment at the top even calls it that: "proving the dead-man branch").
`Drop` runs on every normal return path in Rust, including a `return Err(..)`
out of a loop inside an `async fn` that completes cleanly. There is no
`SIGKILL` here for a `Drop` guard to fail to survive.

#108's own issue body (`gh issue view 108 --repo miztertea/sergeant-rs`)
confirms this independently: its "Mechanism" section describes the leak as
"the dead-man test's premise is that the release path never appears... but
the code under test creates the `.ready` sibling regardless, and only the
happy-path test knows to remove it" — an ordinary code-coverage gap (nobody
wrote the `remove_file` call on this path), not a process-death problem.
`LESSONS.md` L21 says the same thing in the same words. Neither mentions
`SIGKILL`.

**Where the critic's reasoning actually comes from:** the "Drop does not
survive SIGKILL, and SIGKILL is how these die" language is lifted almost
verbatim from `docs/gauntlet/runs/cross-platform-2026-08-14/retrospective.md`
§1.3/§3.3 item 3 — but that passage is about a **different** leak: "Eight
`TempDir`-shaped rigs under `/var/tmp/sgt-rs-tests`... accumulated anyway,
and the cause is... when a run is SIGKILLed mid-suite — the R-MVP1-7 ceiling
(#90), or the harness killing a backgrounded `cargo test` — `Drop` never
runs." That mechanism — a whole `cargo test` **process** killed externally
by a ceiling interrupt or the harness — is real, and a start-of-run reaper is
the right fix for *it*. It is not #108's mechanism. #108's dead-man test
runs to completion every single time (that is the entire point of a
dead-man timer: it fires internally, on schedule, without external help),
and the plan's own R6 (`docs/gauntlet/runs/path-to-mac-2026-08-15/plan.md`
line 43) applies that retrospective's justification to #108 as if the two
were the same mechanism. They aren't.

**Consequence for enactability, not just fidelity:** this matters here
because §6 states every brief carries "the prior art named as settled, so
turns are not spent re-deriving it." A W3 brief carrying R6's stated
reasoning as settled prior art is being hand ed a false premise about why
`.ready`-marker cleanup must be reaper-shaped rather than guard-shaped — a
Work that reads `src/watch.rs` directly (as it must, to build anything) will
find the premise doesn't match what it can see in the code, and either
silently ignores the discrepancy or burns turns reconciling it. That is
exactly the enactability axis's charter: "does confident prose hide an
undecided question" — here it hides a *wrong* one dressed as settled.

**What I am not claiming:** R6's outcome (build a start-of-run reaper) may
still be the right call on its own merits — a reaper is more robust against
the TempDir-rig class the retrospective actually describes, and nothing
stops W3 from building one that also sweeps `.ready` files. I am not
re-litigating R2/R6 as a decision, which the contract places out of scope.
I am refuting the critic's specific stated *reason* for treating this as
"not a finding," because that reason does not hold for #108's actual
mechanism, and the plan states the same wrong reason as R6's justification.

**Severity argued:** warning. Not "error," because a competent Work reading
`src/watch.rs` directly will likely notice the mismatch on its own and can
still build a working reaper (or a guard) regardless of which stated reason
it trusts — this doesn't stall dispatch outright. But it is more than
"info": it is a plan-stated justification for an owner ruling that is
verifiably false about the issue it's justifying, carried into every W3
brief as settled fact.

**What a correction would be:** either fix R6's stated reason (cite the
TempDir-rig mechanism from the retrospective, which is real, rather than
implying #108's dead-man test is SIGKILLed) or, if the reaper is kept as the
implementation choice regardless, drop the SIGKILL justification for #108
specifically and note that an RAII guard would also have worked here — the
reaper's actual justification is "closes the class in one place" (per the
retrospective's own §1.3 framing), not "the guard can't survive how this
test dies."

**Verified vs. believed:** verified — `src/watch.rs` read directly at the
cited line ranges; #108's issue body and `LESSONS.md` L21 read directly;
the retrospective §1.3/§3.3 passage read directly and compared side by side
with plan §2's R6 row.

---

## enactability-F1 — CONFIRMED

Independently re-verified against `git show d86885f:...plan.md` directly
(not the critic's quotes): §3 line "**In:** ... #95, #108 (harness
hygiene)"; §3's "Out" paragraph naming "#95's clock choice" as something
"the Mac still owns"; §5's Work-scope table row `W3 · harness hygiene |
#108 (start-of-run reaper), #95 | main`; §10's bullet "Choosing #95's clock
on real hardware" — all four passages match the critic's quotes exactly,
character for character where compared.

Also re-ran `gh issue view 95 --repo miztertea/sergeant-rs` independently:
its "Scope" section indeed frames the clock pick and its implementation as
one undivided task ("A nanosecond-resolution clock that works on macOS bash
3.2... Verify on the MacBook before closing"), with no language splitting a
Cerberus-buildable part from a Mac-only part. The critic's citation is
accurate.

This is a plain self-contradiction internal to the plan's own text — #95 is
simultaneously inside W3's Cerberus-wave-1 scope and named twice (§3 Out,
§10) as something only the Mac can settle. A second seat (per the
dispatch brief) independently reporting the same tension is consistent with
this being a genuine textual fact rather than an artifact of one reader's
interpretation. **Confirmed, severity as argued (error) stands** — this
isn't a judgment call two readers could resolve differently from the same
text; it's the same document asserting both things.

**Verified vs. believed:** verified — all quoted plan passages and the
issue body read directly in this session.

---

## enactability-F2 — CONFIRMED

Re-ran `gh issue view 90 --repo miztertea/sergeant-rs --comments` and
`gh issue view 94 --repo miztertea/sergeant-rs --comments` in full. #94 ends
with an explicit "Suggested shape" naming the exact check ("if
`finalize_commit` equals the base sha and the worktree is dirty..."). #90's
body — including its own author's later correction comment — ends on
"the actual defect: after a ceiling interrupt the Work sits in `active`,
`retry` refuses it with a 409, `extend` is inert without a retry to gate,
and the only reachable verb is `cancel`... remains the thing to fix," with
a "Suggested shape" section that names three options (land in `blocked`,
add a fault-injection test, consider committing WIP) rather than a single
ruled target — genuinely less decided than #94's.

`grep -n "fails closed into" docs/DEVELOPMENT.md` confirms the citation:
line 40, exact text as quoted. **Confirmed as argued (warning).**

**Verified vs. believed:** verified — both issue bodies and the
DEVELOPMENT.md line read directly.

---

## enactability-F3 — CONFIRMED

`gh issue view 12` and `gh issue view 10` (both `--repo
miztertea/sergeant-rs`) return titles matching the critic's quotes exactly:
"`sgt doctor` has a fixed ~450 ms floor..." and "`blocked_time_per_work`
cold-call latency scales with journal size (153→792 ms at 10k→50k
events)...". The plan's own §4 text (re-read from the pinned commit) states
"a slower result does **not** license a Linux/macOS split — it licenses
raising the finding" with no numeric or comparative threshold attached, and
the contract's own enactability axis brief names exactly this pattern as
the target. **Confirmed as argued (warning).**

**Verified vs. believed:** verified — both issue titles and the plan
passage read directly.

---

## enactability-F4 — CONFIRMED

`gh issue view 109 --repo miztertea/sergeant-rs --comments`, re-read in
full: the latest comment does reframe #109 around "the top-level item —
write the disk-footprint contract" with retention scope named as one clause
among three ("an inspection verb, a fail-closed disposal verb, and the
retention-scope ruling... under the contract framing it is the contract's
central clause"). Plan §5's scope line is "#109 (R4)" only. The critic's
reading — that the plan doesn't say whether W4 is meant to fully close #109
or deliberately narrow it to R4's clause — holds up against the primary
text. **Confirmed as argued (warning).**

**Verified vs. believed:** verified — #109's full comment thread read
directly.

---

## enactability-F5 — REFUTED

The critic's claim: *"every issue-numbered scope item is unreachable via the
natural first move (`gh issue view <N>`) in this checkout"* — evidenced by
`git remote -v` showing `origin` as a local path and `gh issue view 90` (no
flag) failing with a misdirecting auth remedy.

**I ran the identical command and got the opposite result**, in this
worktree and in three sibling worktrees of the same repository:

```
$ git remote -v
github	https://github.com/miztertea/sergeant-rs.git (fetch)
github	https://github.com/miztertea/sergeant-rs.git (push)
origin	/home/miztertea/sergeant-rs (fetch)
origin	/home/miztertea/sergeant-rs (push)

$ gh issue view 90
title:  [core] A turn killed by the R-MVP1-7 ceiling leaves the Work wedged...
... (full issue body) ...
$ echo $?
0
```

Confirmed with `GH_DEBUG=api gh issue view 90` (no flag): `gh` resolves the
repository via the `github` remote automatically and issues the GraphQL
query with no `--repo` needed; no `GH_REPO`/`GH_HOST` env vars are set.
Repeated in `repos/sergeant-rs` and both other currently-live surface
worktrees (`01M01ZKWPA5KT74WYPJ6DD73K2`, `01M01ZKWW96MMH67RE5M5YJMRS`) —
all four show the identical two-remote configuration and all four resolve
`gh issue view <N>` without `--repo`.

Because these are `git worktree`s of one shared repository (confirmed via
`git worktree list`), remote configuration lives in one shared `.git/config`
across all of them — it is not a per-worktree property that could have
differed for the critic's own (now-torn-down) worktree at the time it ran.
The critic's method note shows it ran `git remote -v`, saw `origin` resolve
to a local path, and concluded `gh` needs `--repo` — without checking
whether a second, GitHub-resolving remote was also present. This is exactly
the failure mode the contract itself warns about for this unit ("A grep
that searched the wrong subtree reports a false absence — this has already
happened once in this very unit"): a look that stopped at the first (wrong)
remote rather than the full remote list.

**Note for the record, not part of this axis's grade:** this same now-false
belief — "`gh` needs `--repo miztertea/sergeant-rs`... or it fails with a
misleading auth remedy" — was repeated verbatim in this refuter's own
dispatch instructions, sourced from ADR 0001's day-earlier note that
`origin` alone couldn't resolve GitHub. The `github` remote was evidently
added to this repository after ADR 0001 was written, and that fact has not
propagated to either the plan's downstream guidance or (evidently) this
critic's checkout. Worth carrying to the ledger as a stale-belief instance
in L20's shape, independent of this axis's finding.

**What this means for F5 itself:** the underlying suggestion (add
`gh issue view <N> --repo miztertea/sergeant-rs` or `export GH_REPO=...` to
§6's brief checklist) is harmless if applied, but the finding as argued —
that this is a live failure mode a dispatched Work will hit — does not
survive re-verification. `gh issue view <N>` with no flag works today, in
every worktree checked.

**Verified vs. believed:** verified — commands re-run in this session, in
four independent worktrees, with `GH_DEBUG=api` confirming the resolution
mechanism.

---

## enactability-F6 — CONFIRMED

Re-read plan §7 (orchestrator duties) and §8 (risks) directly from the
pinned commit: §7 names watcher-arming, `finalize_commit`-based completion
verification, the gate-finding authority split, and no-merge-to-`main`; §8
names exactly one failure mode (the #90 ceiling wedge) and mitigates it via
envelope sizing. Neither names a response to a wave-1 Work (W1 or W3)
landing in `failed`.

Re-ran the critic's absence check: `grep -n
"wave.*retry\|retry.*wave\|Work.*failed.*wave\|wave.*failed"` over
`GAUNTLET.md` and every file in
`docs/gauntlet/runs/cross-platform-2026-08-14/` — zero hits, confirming no
standing convention is recorded in the accessible ledger/retrospective
corpus that this plan could be read as silently inheriting. **Confirmed as
argued (warning) — gap named correctly, absence-of-evidence stated as such
by the critic, not overclaimed as proof.**

**Verified vs. believed:** verified — plan text read directly; grep
re-run and returned empty.

---

## Summary

| Finding | Verdict |
|---|---|
| R6/#108 non-finding | **UPGRADED** to a finding (warning) — stated reason is false for #108's actual (non-SIGKILL) mechanism |
| F1 | CONFIRMED (error, as argued) |
| F2 | CONFIRMED (warning, as argued) |
| F3 | CONFIRMED (warning, as argued) |
| F4 | CONFIRMED (warning, as argued) |
| F5 | **REFUTED** — `gh issue view <N>` resolves without `--repo` in this checkout and three sibling worktrees; the critic checked only one of two configured remotes |
| F6 | CONFIRMED (warning, as argued) |
