# PATH-TO-MAC-1 — assumptions refuter

**Axis:** assumptions. **Artifact under review:** `docs/gauntlet/runs/path-to-mac-2026-08-15/plan.md` at `d86885f`. **Critic file graded:** `docs/gauntlet/runs/path-to-mac-2026-08-15/critics/assumptions.md` (commit `b02846c`), read in full and only after this refuter's own read of the contract and the plan.

## Method note

Default posture: skepticism. Every finding below was independently re-run, not re-argued. No file under review was edited (L5) — every command in this pass was read-only (`grep`, `sed`, `cat -n`, `find`, `git log`/`git show`, `cargo tree`, `gh --version`). No mutation probe was needed, so no disposable worktree was used.

Two lines of attack were assigned specifically:
1. **F4's arithmetic** — decide whether the critic's proposed "shared transitive dependency, counted up to three times" mechanism explains the 4.7+17.3+144≠161 gap, and whether the critic's proposed fix (re-sum to 166) would make the plan more wrong.
2. **F3/F7 extended to the whole platform module** — read `src/platform/disk.rs` and `src/platform/data_dir.rs` in full (in addition to `process.rs`, which the critic already read) and state, for each of #18/#81/#82/#85, whether W1's assigned work is unbuilt, partly built, or already shipped-and-awaiting-macOS-verification.

## Verdicts

### assumptions-F1 — REFUTED on its central claim ("does not exist... simply absent"); DOWNGRADED error → warning with a corrected citation

The critic's grep (`grep -rn "libc-binding\|one syscall" docs/ reference/ GAUNTLET.md LESSONS.md`) was scoped to four locations that exclude `src/`. Re-run unfiltered:

```
$ grep -rn "one syscall" --include="*.rs" --include="*.md" .
docs/gauntlet/runs/path-to-mac-2026-08-15/plan.md:94:declined "adding a libc-binding dependency for one syscall." ...
src/platform/disk.rs:5://! that binding "for one syscall" in favor of the same shell-out posture the
src/cli.rs:1531:    // adding for one syscall's worth of shelling out).
```

`src/platform/disk.rs:4-6`'s module doc reads, in full: *"The module this fact used to live in (`src/backend/docker.rs`) explicitly declined that binding 'for one syscall' in favor of the same shell-out posture the rest of this crate already takes for external facts."* That is the plan's quoted objection, word for word, minus "libc-binding dependency." I traced it further with `git log -p -- src/backend/docker.rs`: the exact phrase **"adding a libc-binding dependency for one syscall"** exists verbatim as a code comment on `free_space` in `src/backend/docker.rs`, present since commit `b9d2050` (and earlier), moved into `src/platform/disk.rs` by ADR 0002's boundary-move commit `4eadc50`.

So the objection is real, traceable, and independently re-verifiable in this repo's own git history — it is not a fabrication or a garbled memory, which is what "does not exist... simply absent" asserts. It is mis-cited: the source is a source-code comment (originally in `src/backend/docker.rs`, now paraphrased in `src/platform/disk.rs`'s module doc), not ADR 0002 D4. The critic's absence claim is a **false absence produced by an under-scoped grep** — exactly the failure mode the contract warns this unit has already produced once.

This also means the critic's correction option (a) — "drop the framing entirely... replacing it with what actually changed" — would make the plan **more wrong**, not less: it would delete a true, sourceable claim on the mistaken belief that no such claim exists. Correction option (b) — "cite where it actually lives" — is the right shape, but the critic's own candidate list for where it might live ("a different ADR, an interview transcript, GAUNTLET.md") did not include the actual answer: a source-code doc comment.

One more consequence worth flagging for the adjudicator: because the real governing artifact is a *code comment*, not an ADR, **"An ADR refresh is owed... Assigned to W1"** may itself be targeting the wrong artifact. There is no ADR to refresh here — `src/platform/disk.rs`'s doc comment is what needs updating (or W1 needs to decide whether this now belongs in ADR 0002 proper, which is a scope question, not an assumptions question).

**What a correction would be:** Replace "**This retires ADR 0002 (D4)'s objection**" with "**This retires the objection recorded in `src/platform/disk.rs:4-6`** (originally a code comment on `src/backend/docker.rs::free_space`, moved by ADR 0002's boundary commit `4eadc50`)" and change "An ADR refresh is owed" to name the actual artifact (the doc comment, not an ADR) unless the owner decides this now belongs in ADR 0002 itself.

**Verified vs believed:** VERIFIED — unfiltered repo-wide grep plus `git log -p` tracing the comment's provenance across the ADR-0002 boundary move.

---

### assumptions-F2 — CONFIRMED exactly

Independently re-read all three governing sources in full:
- `LESSONS.md:70-85` (L21): dead-man test's `.ready`-marker leak, RAII/cleanup-in-code-under-test fix, "**Filed as #108**" — verbatim match to the critic's quote.
- `docs/DEVELOPMENT.md:68-74`: same account, "**not** in the body of the happy-path test" — verbatim match.
- `docs/gauntlet/runs/cross-platform-2026-08-14/retrospective.md:77-89` (§1.2, titled *"The 198 files — the failure-path test is the one that leaks (#108)"*) and §1.3 (*"The 1.7 GB of test rigs"*, SIGKILL/`Drop`/reaper reasoning, **no issue number**) plus the residue table (line 27 region): `#108` row says "dead-man test never removes its `.ready` marker"; the SIGKILLed-rig row's disposition is literally **"sweep"**, not a number.

R6's rationale ("`Drop` does not survive `SIGKILL`") is real prose from this repo, but it is §1.3's rationale, not #108's, and §1.3 has no filed issue number. A Work executing R6 as written builds a reaper and very plausibly leaves #108's actual cause (missing marker cleanup) unfixed while reporting #108 closed.

**Verdict: CONFIRMED**, error severity is right — this directly misdirects W3's dispatched scope.

---

### assumptions-F3 — CONFIRMED exactly, and UPGRADED via the assigned second line of attack

Re-read `src/daemon.rs:928-934`, `src/backend/claude.rs:594-602`, `tests/support/mod.rs:198-204` directly:
- `daemon.rs::pid_alive` calls `crate::platform::process::process_alive(pid)` — routed through the boundary, exactly as the critic found. Line 1128's `/proc/self/task` read is thread-naming, unrelated to #18.
- `backend/claude.rs::session_liveness_excluding` calls `crate::platform::process::running_processes()` — routed through the boundary.
- `tests/support/mod.rs:201` (`std::fs::read_dir("/proc")`) is the one genuine direct read.

Critic's characterization is accurate. But reading `src/platform/process.rs` in full (per the assigned second line of attack) shows the finding understates the situation further than the critic's own "warning" framing captures. See the combined platform-module analysis below — this is folded into that write-up rather than repeated twice.

**Verdict: CONFIRMED**, and its severity should be read together with the module-wide finding below, which is the more consequential fact.

---

### assumptions-F4 — arithmetic gap CONFIRMED; the assigned hypothesis REFUTED as the explanation; critic's proposed fix flagged as risky

**The arithmetic fact:** re-verified. 4.7 + 17.3 + 144 = 166.0, not 161. Not in dispute.

**The assigned mechanism — "the three crates share transitive dependencies (libc, rustix, bitflags, memchr); shared code counted once in the combined build is counted up to three times across the individual builds" — does not hold up against this repo's actual dependency graph, and I could not confirm it as even a partial explanation:**

The plan's own §4 states 5 of the 10 candidate-tree crates are **already present** in the current binary: `bitflags`, `libc`, `linux-raw-sys`, `memchr`, `rustix` — the exact four (plus one) the assigned hypothesis names as "shared." I mapped every consumer of each in `Cargo.lock`:

```
$ awk '/^\[\[package\]\]/{pkg=""} /^name = /{pkg=$0} /rustix 1\.1\.4/{print pkg}' Cargo.lock
name = "crossterm"
name = "tempfile"
name = "termina"
name = "xattr"
```

`rustix` 1.1.4 (the version `fs4`'s `rustix = "1"` constraint resolves to) is **already** a live dependency of `tempfile`/`termina`/`xattr`/`crossterm` today — before `fs4` is added at all. `libc` and `memchr` are even more ubiquitous already-present crates. This means these four crates cost approximately **zero** marginal bytes in *each* of the three individual per-crate measurements to begin with (the baseline they were measured against already contains them) — they were never "newly added" by `fs4`, `sysinfo`, or `directories` individually, so there is nothing for the combined build to only-count-once that the individual builds counted redundantly. The hypothesis requires the shared crates to be *genuinely new, shared* cost; they are neither new nor, on this evidence, meaningfully shared cost across the three deltas.

I also checked whether cross-crate linker deduplication (a more general version of the same idea) is plausible: `Cargo.toml` has no `[profile.release]` section, so this build uses Cargo's stock release profile (`lto = false`, 16 codegen units) — there is no LTO/ICF configured that would fold identical code across the three crates' TUs at link time either.

**Conclusion:** the assigned mechanism does **not** fully explain the 5 KiB gap — on the evidence available, it does not clearly explain *any* of it. The gap remains genuinely unresolved by reasoning alone, exactly as the critic already scoped it (this axis's non-goal is not to re-derive the underlying binary-size measurement, so I did not rebuild and re-measure).

**On the critic's proposed correction ("re-sum the total to 166 KiB"):** this is not obviously safer than leaving 161 KiB as the reported total, and could make the plan *more* wrong. The 161 KiB figure is described as measured from **the combined build** — a real, single measurement of the actual thing being shipped. 166 KiB is a **derived sum of three independent measurements**, each of which is its own separate build with its own opportunity for noise (stripped-ELF section alignment/padding differs run to run; the three figures' precision is inconsistent — `4.7` and `17.3` are given to one decimal KiB, `144` is a bare integer, which is itself worth a KiB or so of rounding slop, not enough alone to explain 5 KiB but a sign the line items were not captured with uniform care). Mechanically replacing a real combined measurement with a summed approximation trades a more-trustworthy number for a less-trustworthy one without evidence either individual line item is the error.

**What the plan should actually say:** state the gap explicitly rather than silently presenting 161 as if it summed cleanly, and name the reconciling command rather than guessing which number is right:
```
cargo clean -p fs4 -p sysinfo -p directories -p dirs-sys -p option-ext
PATH="$HOME/.cargo/bin:$PATH" cargo build --release   # combined, re-measure total
# then repeat individually for each of the three crates against the same baseline
```
Until that is run, the honest statement is "three independently-measured per-crate deltas summing to 166.0 KiB vs. a combined-build measurement of 161 KiB — a 5 KiB / 3% reconciliation gap not yet explained," not a silently-accepted 161.

**Verdict:** arithmetic finding CONFIRMED as real; the assigned candidate mechanism REFUTED as an explanation (verified via dependency-consumer mapping, not reasoning); severity holds at warning; the critic's specific proposed fix (mechanical re-sum) is flagged as a plausible new error rather than adopted.

---

### assumptions-F5 — CONFIRMED exactly

Read `sergeant.toml` at the estate root (`/home/miztertea/sergeant-rs/sergeant.toml`) directly. Lines 4-7 are a comment about `default_backend` precedence; the `[[profile]] name = "sonnet"` block is at lines 11-14, exactly as the critic found.

---

### assumptions-F6 — CONFIRMED exactly

Read `docs/DEVELOPMENT.md:68-74` directly. The rule ("A workflow stage or actor executing inside a worktree never invokes `scripts/gate.sh`/no-mistakes itself...") spans lines 70-73; line 71 alone is a mid-sentence fragment, exactly as the critic found.

---

### assumptions-F7 — CONFIRMED exactly, and CONFIRMED as the sole genuinely-unbuilt item via the assigned second line of attack

```
$ grep -rn "/proc/mounts" src/          → no hits
$ grep -rln "flock\|LOCK_EX\|advisory lock" src/
src/runtime/fsutil.rs src/daemon.rs src/runtime/journal.rs src/domain/manifest.rs
```
`src/runtime/fsutil.rs::take_exclusive_lock` uses `std::fs::File::try_lock` — already cross-platform via `std` (flock on Unix, `LockFileEx` on Windows) — and backs the journal/manifest lock, unrelated to #85's filesystem-type-detection need. No mount-parsing or filesystem-type code exists anywhere in `src/`; every "mount" hit in `docker.rs`/`cli.rs`/`manifest.rs`/`workflow.rs`/`api.rs` is about Docker bind-mounts, confirmed by direct inspection of each hit. `docs/gauntlet/runs/cross-platform-2026-08-14/plan.md:36` and `close-out.md:108` independently confirm #85 was still queued, unbuilt, at that sprint's close. CONFIRMED.

---

### assumptions-F8 — CONFIRMED (labeling gap real); underlying historical claim remains PLAUSIBLE, not upgraded

`gh --version` on this host reports `2.97.0 (2026-07-31)` — matches the critic's verification of the post-upgrade state exactly. The pre-upgrade "51 minor versions behind... failed every call on 2.46.0" claim remains unverifiable from this session (no prior version is observable now); left PLAUSIBLE, not dropped, matching the critic. The labeling-convention gap itself (three same-confidence claims, only one tagged `[measured]`) is a real, independently-checkable inconsistency in the plan's own text. CONFIRMED.

---

## Second line of attack — full platform-module status for #18, #81, #82, #85

Read `src/platform/mod.rs`, `process.rs`, `disk.rs`, `data_dir.rs` in full. This is the single fact the contract's assignment says "no seat established it completely," so it is reported in full even though it extends rather than replaces F3/F7:

| Issue | File | Status | Evidence |
|---|---|---|---|
| **#18** (process liveness) | `src/platform/process.rs` | **Shipped, both platforms implemented and tested; macOS arm UNVERIFIED** | Linux arm: raw `/proc` read (`raw_running_processes`, `raw_process_alive`). macOS arm: `ps -axo pid=,command=` + `kill -0`, both marked `**UNVERIFIED**` in their own doc comments, both covered by unit tests (`parse_ps_output` pinned against ordinary shape, header-row noise, empty input, and the known quoted-argument-splitting weakness). Module doc: *"They close #18 when someone measures them there, not when this lands."* Callers `daemon.rs` and `backend/claude.rs` already route through this boundary (F3, above); only `tests/support/mod.rs` still reads `/proc` directly. |
| **#81** (free disk space) | `src/platform/disk.rs` | **Shipped, both platforms implemented and tested; macOS arm UNVERIFIED** | Linux arm: `df -k --output=avail` (GNU). macOS arm: `df -k` + positional `Available`-column parsing (BSD shape), marked `**UNVERIFIED**`, pinned by a test (`bsd_shape_parses_positionally`) using the exact macOS `df` column layout. Module doc explicitly states the mechanism is **not** `fs4`/`statvfs` today and names the historical objection (see F1, above) — the GNU-only failure #81 was filed over is already fixed by the added POSIX-portable arm. |
| **#82** (data-dir fallback tail) | `src/platform/data_dir.rs` | **Shipped, both conventions implemented and tested; macOS arm UNVERIFIED** | `FREEDESKTOP` (`$XDG_DATA_HOME` / `~/.local/share`) and `MACOS` (`~/Library/Application Support`) are both unconditionally compiled and both unit-tested (including the macOS convention's XDG-ignoring behavior, exercised from this Linux host per ADR 0002 D3). Module doc: *"Closes when measured there (#82), not when this lands."* |
| **#85** (portable flock + filesystem type) | *(no file — does not exist)* | **Unbuilt** | No mount-parsing, filesystem-type-detection, or `statfs`/`f_type` code anywhere in `src/` (grepped clean, every "mount" hit is Docker bind-mount-related). The only existing flock-shaped code (`fsutil.rs`) is `std`-portable already and serves a different lock (journal/manifest), not #85. Cross-platform sprint's own planning docs confirm #85 was still queued at close. |

**Why this matters more than the plan's framing suggests:** three of the four "W1 · platform crates" line items (#18, #81, #82) are not build work — they are **complete, dual-platform, unit-tested implementations that already ship in this binary today**, each blocked only on a macOS host to flip its `UNVERIFIED` doc-comment marker. Adopting `fs4`/`sysinfo`/`directories` for those three is a **replace-working-code-with-a-dependency** decision (which R1's "reimplementing `fs4` worse" language gestures at but does not spell out), materially different in size and risk from #85, which is the only one of the four requiring net-new implementation from nothing. The plan's §4 table and §5 wave assignment currently present all four as one undifferentiated scope line ("#81, #82, #18, #85") with no signal that three already have shipped, tested answers and one has none. **This is a plan-shape correction, not merely an assumptions correction** — W1's brief should say explicitly which of its four items are "swap a working implementation for a crate" (#18/#81/#82) versus "build from scratch" (#85), because the acceptance bar and realistic effort differ sharply between the two.

## PLAUSIBLE items — left undisturbed, not re-litigated

`#96`'s characterization, the 3.90s compile-time figure, and the 64,023,360 B binary-size figure are all explicitly out of scope for re-derivation per the contract's non-goal ("crate measurements in plan §4 are not re-derived") and per `gh`'s confirmed inability to reach the tracker from this host. I did not attempt to re-open these. They remain PLAUSIBLE per the critic's own recording, per the method doc's fail-closed rule.

## Summary for the adjudicator

| Finding | Verdict | Move |
|---|---|---|
| F1 | REFUTED (central "absence" claim) | DOWNGRADE error → warning; correct the citation, don't delete the claim |
| F2 | CONFIRMED | none |
| F3 | CONFIRMED | UPGRADE — read together with the platform-module table above |
| F4 | Arithmetic CONFIRMED; assigned mechanism REFUTED | none to severity; correction path changed (re-measure, don't re-sum) |
| F5 | CONFIRMED | none |
| F6 | CONFIRMED | none |
| F7 | CONFIRMED | UPGRADE — read together with the platform-module table above |
| F8 | CONFIRMED | none |

Refs #18, #81, #82, #85, #90, #94, #95, #96, #108, #109
