# PATH-TO-MAC-1 — refuter: invariants

Refuting `docs/gauntlet/runs/path-to-mac-2026-08-15/critics/invariants.md`
against `docs/gauntlet/runs/path-to-mac-2026-08-15/plan.md` at `d86885f`, per
`docs/gauntlet/contracts/PATH-TO-MAC-1.md`. I did not write the plan or the
critic findings. Given a specific line of attack on `invariants-F1` (test the
`rustix`-vs-`fs4` rung claim by running commands, not by arguing); the other
three findings I re-verified independently by re-reading cited text and
tracing line numbers, per the general refuter mandate. No mutation probe was
needed — every check below is read-only (`cargo tree`, `grep`, `git log`,
reading cached crate source and in-repo files); nothing under review was
edited.

---

## Finding invariants-F1: `fs4` is proposed as new machinery (R7) without the plan checking whether `rustix` (R5) already does the job

**Verdict: CONFIRMED, but the finding's own "no new dependency edge" claim is
REFUTED, and its correction (a) is opposed by in-repo evidence the critic did
not check.**

### 1. Is `rustix` a resolvable dependency edge here, and at what version/features?

Ran directly against this checkout (not the integration branch — same
`Cargo.lock`):

```
$ grep -n '^name = "rustix"' -A3 Cargo.lock
2559: name = "rustix"  version = "0.38.44"
2572: name = "rustix"  version = "1.1.4"
```

Two `rustix` resolutions coexist. Traced both with `cargo tree -e normal -i`:

- `rustix v1.1.4` ← `crossterm v0.29.0` ← `ratatui-crossterm v0.1.2` ←
  `ratatui v0.30.2` ← `sergeant-rs`. This is the one the plan means by
  "`rustix` v1" (`plan.md` line 75).
- `rustix v0.38.44` ← `crossterm v0.28.1` ← `comfy-table v7.1.4` /
  `duckdb v1.10505.0` ← `sergeant-rs`. A separate, semver-incompatible
  resolution the critic did not distinguish from the v1 line — worth noting
  but not load-bearing for this finding, since the plan explicitly names
  "`rustix` v1."

**CONFIRMED**, correcting the critic's imprecision: `rustix` v1.1.4 is real,
resolvable, and matches the plan's own citation. But it is **not a declared
dependency** — `grep -n "rustix" Cargo.toml` and a scan of `[dependencies]`
return nothing. It is transitive only, brought in for `crossterm`'s terminal
handling.

### 2. Do `rustix::fs::statvfs` and `rustix::fs::flock` exist in v1.1.4?

Not taken from documentation — read the cached source directly, per the
brief's own instruction that "the cached crate source under
`~/.cargo/registry/src/` is authoritative; documentation is not":

```
$ RUSTIX_SRC=~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustix-1.1.4
$ grep -n "pub fn statvfs\|pub fn flock" -r "$RUSTIX_SRC/src/fs"
src/fs/abs.rs:288:pub fn statvfs<P: path::Arg>(path: P) -> io::Result<StatVfs>
src/fs/fd.rs:317:pub fn flock<Fd: AsFd>(fd: Fd, operation: FlockOperation) -> io::Result<()>
```

**CONFIRMED**: both functions exist, exactly as the critic's Context7 lookup
claimed, now verified against the actual cached source rather than docs.

### 3. The critical question: does using them require a new manifest edge, and is the critic's "no new dependency edge is added at all" claim true?

Checked what features this tree currently resolves `rustix` 1.1.4 with:

```
$ cargo tree -e normal -i rustix@1.1.4 -f "{p} [{f}]"
rustix v1.1.4 [alloc,std,stdio,termios]
```

No `fs` feature. And in `rustix`'s own `Cargo.toml`:

```
fs = []
...
```
```
src/lib.rs:224: #[cfg(feature = "fs")]
src/lib.rs:226: pub mod fs;
```

`rustix::fs` is compiled out of this tree as currently resolved. To call
`rustix::fs::statvfs`/`flock` directly, `src/platform/` would need:

- `rustix` added as an **explicit `[dependencies]` line in `Cargo.toml`**
  (Rust does not let a crate call a transitive dependency's API directly —
  only a direct manifest dependency's public items are visible to `use`), and
- the `fs` feature turned on for it.

**This is a new dependency edge in the manifest sense**, even though it adds
zero new *crates* to `Cargo.lock` (`fs = []` pulls in nothing further, and
Cargo's feature unification would fold the new `fs` request into the single
already-resolved `rustix v1.1.4` build alongside `crossterm`'s `termios`
request — no version conflict). So the critic's claim in "what a correction
would be," option (a) — **"no new dependency edge is added at all"** — is
**REFUTED as literally stated**. The accurate version is narrower: no new
crate enters the build graph, but `Cargo.toml` does gain a new declared
dependency with its own feature flag, which is exactly the kind of change
`GAUNTLET.md`'s rung-logging convention (quoted by the critic in F2) asks to
be recorded, and option (a) as written obscures that it's making a manifest
change of the same *kind* (new `Cargo.toml` line) it's trying to avoid, just
not the same *cost* (no new crate weight).

### 4. Does `src/platform/disk.rs`'s own recorded reasoning oppose correction (a)?

Read `src/platform/disk.rs:1-21` in full — its own module-level doc comment,
authored **2026-08-14** (`git log -1 --format=%ai -- src/platform/disk.rs`),
one day before this plan and by the same lineage of work (the ADR 0002
platform-boundary commit):

> "`df` remains the mechanism — not a `libc`/`statvfs` binding. The module
> this fact used to live in (`src/backend/docker.rs`) explicitly declined
> that binding 'for one syscall' in favor of the same shell-out posture the
> rest of this crate already takes... It still holds..."

This is not a restatement of ADR 0002 (D4) at one remove — it is the
codebase's **most recent, most specific, and still-standing** statement on
the exact question correction (a) proposes to reopen, written the day before
this plan and never mentioned by the critic. Correction (a) — "have W1 call
`rustix::fs::statvfs`/`flock` directly" — is precisely the "libc/`statvfs`
binding... for one syscall" this comment says was considered and rejected,
and whose rejection "still holds." The critic's own hedge ("BELIEVED, not
verified... I did not run a build or `cargo tree` against this checkout") was
the right instinct; run against the checkout, it surfaces a second, sharper
objection the critic didn't have: **correction (a) is opposed by the
platform module's own contemporaneous recorded reasoning**, not merely
unconfirmed.

Correction (b) — keep `fs4`, add one sentence naming why the raw-`rustix`
path was rejected — survives this check and is in fact strengthened by it:
`disk.rs`'s comment already *is* that sentence, one module and one day early.
The plan's own §4 argument ("This retires ADR 0002 (D4)'s objection... An ADR
refresh is owed," assigned to W1) is the mechanism by which `disk.rs`'s
comment gets superseded too — W1's brief should point at `disk.rs:1-21`
directly, not only at the ADR, since the module comment is the more specific
and more recent artifact making the same claim.

### Net verdict on F1

The core observation — the plan proposes `fs4` (R7) without documenting that
the already-present `rustix` was checked and found wanting — **CONFIRMED**.
Severity: **warning** is right, not upgraded — nothing breaks, and the
correct resolution (keep `fs4`, cite why) is cheap. But the finding's
"what a correction would be" section is only half right: option (a) should
be **struck**, not offered as a coequal alternative — it is factually wrong
about adding no dependency edge, and it is opposed by `src/platform/disk.rs`'s
own reasoning, written the day before this plan, which the critic did not
check. Option (b) is the only correction that survives and should be the
plan's actual fix, strengthened with a direct pointer to `disk.rs:1-21`
rather than only the ADR.

---

## Finding invariants-F2: none of §4's five new dependencies records a Ponytail rung

**Verdict: CONFIRMED.**

Re-read `plan.md` §4 in full (lines 70-97): no `R1`–`R7` annotation appears
anywhere for `fs4`, `sysinfo`, `directories`, `dirs-sys`, or `option-ext`.
Re-checked the governing text independently rather than trusting the critic's
quotes:

```
$ sed -n '14,15p' GAUNTLET.md
Design decisions and deviations log their **Ponytail rung** (`R1`–`R7`; ...)
`R7` (new machinery) entries name which lower rungs failed and why.
```

Matches verbatim. `reference/notes/ideaos-agent-contract.md`'s "Rung logging
convention" paragraph (lines 49-53 in the current file) also matches verbatim
— "every new dependency... records the rung it resolved at."

The critic's own uncertainty ("BELIEVED, not proven: that this convention is
meant to bind a *plan* document") is honest and I found nothing to settle it
either way — no enforcement mechanism in-repo that would reject a plan for
this gap specifically (as opposed to a ledger entry). That uncertainty
attaches to *enforceability*, not to whether the gap exists, and the finding
is framed (info/warning, process-conformance) consistent with that. No basis
to upgrade or downgrade.

---

## Finding invariants-F3 (PLAUSIBLE): adjacent-append crash-window check not routed to W2/W4 briefs

**Verdict: PLAUSIBLE, unchanged.**

Verified both citations independently:

```
$ grep -n "Adjacent-append crash windows" docs/DEVELOPMENT.md
40:- **Adjacent-append crash windows** are this architecture's recurring hazard (LESSONS L6): ...
```

Matches (critic cited line 41; the bullet's text starts on 40 in this
checkout — a one-line drift, not a misquote). `LESSONS.md` L6 exists at line
311 with the header the critic quotes. §6's per-brief checklist (`plan.md`
lines 150-161, verified against this checkout at 152-160) indeed lists seven
items and none references L6, "adjacent-append," or `DEVELOPMENT.md`'s
architecture-invariants section by name.

This finding turns on whether W2's/W4's *actual diffs* will in fact add a new
multi-append sequence — that code does not exist yet, and the contract's
non-goals bar deriving it in this unit. I did not find a way to settle this
from available evidence, same as the critic. Recorded PLAUSIBLE, not dropped,
per the contract's explicit rule for exactly this case.

---

## Finding invariants-F4 (PLAUSIBLE): §3/§10 reserve #95's clock choice for the Mac, but §5 dispatches all of #95 to Cerberus with no stated boundary

**Verdict: PLAUSIBLE, unchanged.**

Verified all four citations directly:

```
$ grep -n "harness hygiene\|clock choice\|W3 · harness" plan.md
60:#96, #109 (operator surfaces) · #95, #108 (harness hygiene).
64:crates' cross-platform claims hold on a real host — plus `#95`'s clock choice,
123:| 1 ‖ | **W3 · harness hygiene** | #108 (start-of-run reaper), #95 | `main` |
```

§10's line 219 restates the Mac-side reservation ("Choosing #95's clock on
real hardware") — confirmed by reading §10 in full. The tension is real and
textual: §3/§10 say the clock *choice* needs real Mac hardware; §5 assigns
all of "#95" undifferentiated to a Cerberus-cut Work. I looked for a
disambiguating sentence elsewhere in the plan (searched the full text for
"pluggable," "abstraction," "clock" outside §3/§10) and found none — the
plan genuinely does not state which of the two readings (W3 builds a
pluggable abstraction and leaves selection open, vs. W3 is expected to fully
resolve #95 and has nothing to select from without Mac hardware) governs.
Cannot be settled from the text as written. PLAUSIBLE stands.

---

## Summary

| Finding | Verdict | Severity |
|---|---|---|
| F1 | CONFIRMED (core claim); correction (a) REFUTED — false "no new dependency edge" claim, and opposed by `src/platform/disk.rs:1-21`'s own reasoning (written 2026-08-14, one day prior, uncited by the critic) | warning, unchanged |
| F2 | CONFIRMED | info/warning, unchanged |
| F3 | PLAUSIBLE | warning, unchanged |
| F4 | PLAUSIBLE | info, unchanged |

No refutations in the sense of "finding is wrong"; one finding (F1) had a
factual error in its own proposed correction that a same-checkout `cargo
tree` run and a read of `disk.rs` surfaces — exactly the shape of check this
axis's specific line of attack was designed to force. Recommend the
adjudicator carry F1's correction forward as **option (b) only**, with a
direct citation to `src/platform/disk.rs:1-21` added to W1's brief.
