# Fidelity refuter — PATH-TO-MAC-1

Adversarial verify, axis: fidelity. Batched over
`docs/gauntlet/runs/path-to-mac-2026-08-15/critics/fidelity.md`. Given a
specific line of attack on fidelity-F1 (below); F2–F4 re-verified on general
skepticism, not on a directed attack.

## fidelity-F1 — DOWNGRADED

**Critic's claim:** the plan's quote — `"adding a libc-binding dependency for
one syscall"`, attributed to "ADR 0002 (D4)" — is invented; a repo-wide grep
"found it nowhere outside the plan itself."

**Attack run, from the repository root, exactly as specified:**

```
$ grep -rn "one syscall" src/
src/platform/disk.rs:5://! that binding "for one syscall" in favor of the same shell-out posture the
src/cli.rs:1531:    // adding for one syscall's worth of shelling out).
```

```
$ sed -n '1,14p' src/platform/disk.rs
//! Free disk space (#81).
//!
//! `df` remains the mechanism — not a `libc`/`statvfs` binding. The module
//! this fact used to live in (`src/backend/docker.rs`) explicitly declined
//! that binding "for one syscall" in favor of the same shell-out posture the
//! rest of this crate already takes for external facts (`kill` for signals,
//! `docker` itself for the container adapter); `df` is present on every
//! measured environment. ...
```

Then I reproduced the critic's **own stated command** verbatim
(`grep -rn "libc-binding dependency\|one syscall" .` from the repo root,
right now): it returns **three** hits, not one —
`docs/gauntlet/runs/path-to-mac-2026-08-15/plan.md:94`,
`src/platform/disk.rs:5`, and `src/cli.rs:1531`. The critic's absence claim
is **false as written** — this is exactly the false-absence failure the
dispatch brief warned is not hypothetical ("this has already happened once
in this very unit"), and it is also a direct violation of the rule the
critic's own report cites elsewhere in this same file (L23: "confirm a
claimed absence against the unfiltered artifact before reporting it as a
finding").

**Chasing the real source, since the critic's method note says it read
`disk.rs` lines 1–105 and yet missed line 5:** `disk.rs`'s doc comment
attributes the objection not to itself but to "the module this fact used to
live in (`src/backend/docker.rs`)". `git log --follow -p -- src/backend/docker.rs`
turns up the objection **verbatim**, predating the ADR entirely:

```
// `statvfs` is the portable POSIX call for this; sergeant is Linux-first
// (`docs/DEVELOPMENT.md`) and this crate otherwise reaches such facts through
// `std`, so a `df`-equivalent shell-out is used instead of adding a
// libc-binding dependency for one syscall — the same "smaller direct
// client" posture the module docs already take for Docker itself. `df`
// is present on every measured environment (`docs/environments/`).
```

That is the plan's exact phrase, word for word, sitting in this repo's own
history — not paraphrased, not reconstructed, not invented. `docs/adr/0002-platform-boundary-shape.md`
has exactly one revision (`8ee7b2b`, confirmed independently — `git log
--oneline -- docs/adr/0002-platform-boundary-shape.md`), and its D4 section,
read in full, is about what falls inside the platform-facts boundary
(Docker/`claude` CLI excluded); it says nothing about declining a
dependency for a syscall. On that narrow point the critic is right: **ADR
0002 D4 does not contain this objection**, so "This retires ADR 0002 (D4)'s
objection" is a wrong citation and a Work opening D4 to find it will not.

**Verdict: DOWNGRADED, not CONFIRMED as written.** The critic's core
citation complaint survives — ADR 0002 is the wrong source — but the
finding is wrongly argued as fabrication ("the artifact under review is the
one doing the inventing... an objection ADR 0002 never raised" — true of
ADR 0002, false of the claim in general) when the objection is real,
verbatim, and locatable in this repository's own git history and current
`disk.rs` doc comment. This is not a cosmetic distinction: the critic's own
proposed correction offers, as one live option, "drop the... invented quote
entirely" — applying that option would delete a **true, sourced statement**
from the plan. The correction that survives this refutation is **re-cite,
not delete**: point "This retires [...]'s objection" at
`src/platform/disk.rs:3-6` (current) and/or the original
`src/backend/docker.rs` comment in its git history, drop "ADR 0002 (D4)",
and keep the quote — it is accurate.

Severity: I'd hold this at **error**, same tier as the critic, but for a
narrower reason than the critic gave — a Work acting on "ADR 0002 refresh
owed" (assigned to W1) is sent to open the wrong document and, on finding
nothing there, has no way to know from the plan alone that the real
objection lives in code history rather than nowhere. That is still a
correction a Work cannot self-serve from the plan as written, which is what
makes it error rather than warning.

## fidelity-F2 — CONFIRMED

Re-ran independently: located `/home/miztertea/sergeant-rs/sergeant.toml`
(the only file of that name on the filesystem — `find / -xdev -iname
sergeant.toml`), read directly. Lines 4-7 are the `default_backend`
precedence comment; the `[[profile]] name = "sonnet"` block is at lines
11-14. Matches the critic exactly. No refutation available.

## fidelity-F3 — CONFIRMED

Re-ran independently: `docs/DEVELOPMENT.md:70-73` read directly — the
sentence spans all four lines, line 71 alone is the middle clause. ADR
0005's D1 quote checked verbatim (`grep -n "dissolves rather than being
amended\|Gating becomes a dispatched Work"
docs/adr/0005-gating-becomes-a-dispatched-work.md` → lines 26 and 38).
Matches the critic exactly, including its own caveat that this is info-tier
because the substantive claim is accurate and only the pointer is
imprecise. No refutation available.

## fidelity-F4 — PLAUSIBLE (confirmed as PLAUSIBLE, not resolved)

Read `LESSONS.md` L23 in full (lines 9-43). The critic's characterization
holds: every tabled instance is a *narrowed view* of real output, the two
pipe rows are separately called out as the "dangerous class" for a
*mutating* command whose failure channel was discarded, and the plan's
"silently-failed `cargo clean`" claim fits that broader class only if the
clean's own exit status was discarded via a pipe/`&&` — which the plan does
not state. No repo artifact records the actual `cargo clean` invocation
from that working session, so this cannot be settled from evidence
available to either the critic or this refuter. I did not find a way to
move it off PLAUSIBLE; recording it as such rather than dropping it, per
the contract.

## Summary for the adjudicator

| Finding | Critic severity | Refuter verdict | Note |
|---|---|---|---|
| F1 | error | **DOWNGRADED** | citation is wrong (ADR 0002), but the quote is real and verbatim in `src/backend/docker.rs` git history / `src/platform/disk.rs:3-6` — correction is re-cite, not delete; critic's "nowhere outside the plan" grep is independently reproducible as false |
| F2 | warning | CONFIRMED | |
| F3 | info | CONFIRMED | |
| F4 | PLAUSIBLE | PLAUSIBLE | unresolved, correctly not dropped |

No mutation probe was needed or run; nothing here required testing a git
constraint empirically beyond the read-only `grep`/`sed`/`git log` commands
shown above, all run directly against the working tree, none piped into a
mutating command.
