# Assumptions critic — macbook-arrival-2026-08-15

Axis: **Every factual and measured claim** in `plan.md`, with special focus on
§4's "0 repositories declared" / "nothing has ever run" claims, §5's
file-disjointness claim across WA/WB/WC, §6's `sergeant.toml` / `--profile
sonnet` refusal claim, the issue characterizations, the prior-sprint envelope
precedent, and PR #126's state.

**Method:** every verification below was run independently in this session.
No claim is taken from the plan's own word; the plan's `[measured]` tags are
treated as assertions to be confirmed or marked PLAUSIBLE, not as proof.
Verified-in-session and believed are distinguished throughout (L15).

---

## F-A1 · §5 "disjoint file sets" label is self-contradictory — WARNING

**Plan text at issue (§5, paragraph after table):**

> "WA/WB/WC touch disjoint file sets (`scripts/gate.sh` ·
> `src/runtime/{recovery,engine}.rs` + `tests/m3_execution.rs` ·
> `tests/m2_daemon_api.rs` + `src/runtime/{engine,surface}.rs` + `docs/perf/`)
> — each Work gets its own isolated worktree by construction (`sgt`'s
> one-Work-one-surface model), so parallel dispatch is safe; residual risk is
> an ordinary merge conflict if WC's profiling touches the same submit-path
> lines WB's fix does, flagged in §8."

**What was measured in-session:**

The plan's own enumeration lists `src/runtime/engine.rs` inside both WB's
file set and WC's file set. **Verified by reading the file list in the same
paragraph:** WB = `{recovery,engine}.rs` + `tests/m3_execution.rs`; WC =
`{engine,surface}.rs` + `tests/m2_daemon_api.rs` + `docs/perf/`. `engine.rs`
is in both.

That both WB and WC would plausibly need to touch `engine.rs` is consistent
with the issue bodies (verified in-session):

- **#129** (WB): "pure engine/recovery logic (`src/runtime/recovery.rs` /
  `src/runtime/engine.rs`)" — the extra OBSERVE fix sits squarely there.
- **#128** (WC): describes "the whole submit path — workspace discovery,
  workflow bind, `git worktree add`, reservation, launch." A grep of
  `engine.rs` in-session confirms it owns surface materialization, reservation
  records, and the submit pipeline — the most likely profiling target for #128.

**Assessment:** the word "disjoint" is factually wrong as applied to WB and
WC, because `src/runtime/engine.rs` appears in both sets. The plan is
**honest** about this — it qualifies the claim in the same sentence ("residual
risk is an ordinary merge conflict … flagged in §8") and lists the overlap
explicitly in §8 risk 4 — but the label "disjoint" contradicts both the
enumeration that immediately follows it and §8's own risk entry. A Work
executor reading §5's topic sentence and stopping there would form a false
picture; one reading the whole paragraph would not.

**Severity: WARNING.** The plan is self-correcting within §5 and again in §8,
so no executor would be operationally misled if they read both. But the label
is wrong and should be corrected.

**What a correction looks like:** Replace "disjoint" with "largely non-overlapping"
or "mostly-separate" and move the engine.rs exception from the parenthetical
risk flag to the same sentence: e.g., "WA/WB/WC touch largely non-overlapping
file sets — WB and WC both plausibly require edits to `src/runtime/engine.rs`
(flagged in §8), but their primary files and test files do not overlap — so
parallel dispatch is safe."

---

## F-A2 · §6 "no Work used more than 2 turns" understates gate-dispatch costs
for WD — INFO

**Plan text at issue (§6):**

> "matching the prior sprint's envelope sizing precedent — no Work in that
> sprint used more than 2 turns against a well-scoped brief, so 40 is
> deliberate slack, not an estimate"

**What was measured in-session:**

Read `docs/gauntlet/runs/path-to-mac-2026-08-15/retrospective.md` §6
(Calibration) in full. Exact text:

> "**Envelopes were over-provisioned 20×.** Sized at `--turns 40` out of
> concern for #90's wedge; **every implementation Work used exactly 2**."
>
> "**Cost:** 19 Works, all `completed` — zero failures, zero wedges, zero
> `needs_input`. 7 implementation × 2 turns; 8 gauntlet seats × 1; 1
> first-pass review × 5; 3 gate dispatches × 7."

**Comparison:**

| Work class | Turns used | Covered by the plan's "2 turn" claim? |
|---|---|---|
| Implementation (7 Works) | 2 each | Yes |
| Gauntlet seats (8 Works) | 1 each | Yes (< 2) |
| First-pass review (1 Work) | 5 | No |
| Gate dispatches (3 dispatches) | 7 each | No |

The plan's "no Work used more than 2 turns" claim, even with the qualifier
"against a well-scoped brief," does not hold for gate dispatches (7 turns each)
or the first-pass review (5 turns). The qualifier "well-scoped brief" is
undefined and does not obviously apply to exclude the gate dispatches — the
PATH-TO-MAC retrospective §3.1 records that the gate Work's brief had a shared-
preamble defect that caused operational problems, but the plan's qualifier does
not name that exclusion.

**Why this matters for WD:** WD in this plan is `--workflow validate-and-ship`,
the same workflow as PATH-TO-MAC's gate Work. PATH-TO-MAC's gate dispatches
used 7 turns each, not 2. A reader using the plan's "2 turns" reasoning to
predict WD's likely turn consumption would be off by 3.5×. The 40-turn ceiling
remains adequate (7 < 40), so the envelope choice is not wrong — but the
stated justification for why 40 is "deliberate slack" is imprecise specifically
for WD.

**Verified in-session:** read §6 of retrospective (primary source). The 7
gate-dispatch turns figure is **verified**; the first-pass review figure is
**verified**. The envelope of 40 being adequate is **verified** (7 ≪ 40).

**Severity: INFO.** The envelope decision is correct regardless. No Work is
at risk of hitting the ceiling based on the prior data. The justification is
imprecise, not wrong in consequence.

**What a correction looks like:** Add the class breakdown from the
retrospective: "implementation Works used exactly 2 turns each; gate dispatches
used 7 turns each; both are well under 40. WA/WB/WC are `--workflow implement`
(expect ~2 turns); WD is `--workflow validate-and-ship` (expect ~7 turns on
PATH-TO-MAC precedent); 40 is deliberate slack for all four."

---

## F-A3 · §4 pre-Wave-0 measured state — PLAUSIBLE, not independently
verifiable

**Plan text at issue (§4):**

> "**[measured]** `sgt doctor` on this checkout reports `0 repositories
> declared`, and `sgt status`/the journal show **nothing has ever run in this
> data dir** — this Mac has never had a working self-hosted estate for
> sergeant-rs to dispatch Work against."

**What was measured in-session:**

Wave 0 has already run. Running `sgt doctor` and `sgt repo list` from the
estate root (`/Users/thawes/Desktop/projects/sergeant-rs`) now shows:

```
estate: 1 repositories declared, all present on disk, no undeclared
directories under repos/
```
```
sergeant-rs  /Users/thawes/.../repos/sergeant-rs  instructions=suppress
  origin=https://github.com/miztertea/sergeant-rs.git
```

Journal events: 179 (at time of this review). The pre-Wave-0 state cannot be
recovered from the current journal.

**Assessment:** The `[measured]` tags are appropriate — they mark a
contemporaneous measurement, not a belief. The post-Wave-0 state (1 repo
declared, non-empty journal) is exactly what the plan describes as the result
of Wave 0 having run. No counter-evidence exists. The estate being new on this
Mac is internally consistent with `sergeant.toml` lacking a `default_backend`
that PATH-TO-MAC's estate (on a different host) had added.

**Status: PLAUSIBLE.** The claim cannot be confirmed in-session (the
pre-Wave-0 state is gone), but nothing contradicts it and the post-Wave-0
observable state is what the plan predicts.

---

## F-A4 · §6 `sergeant.toml` claim — VERIFIED ✓

**Plan text at issue (§6):**

> "This estate's `sergeant.toml` (§4) has no `[profile.*]` sections and no
> `default_backend` at all — it is a fresh scaffold, not a copy of the prior
> sprint's manifest — so an explicit `--backend claude` on every dispatch is
> the correct replacement, not a manifest edit invented to route around the
> refusal."

**What was measured in-session:**

Found `sergeant.toml` at `/Users/thawes/Desktop/projects/sergeant-rs/sergeant.toml`.
Full contents:

```toml
[estate]
name = "sergeant-rs"

[[repo]]
name = "sergeant-rs"
path = "repos/sergeant-rs"
origin = "https://github.com/miztertea/sergeant-rs.git"
```

No `[profile.*]` sections. No `default_backend` key under `[estate]` or
anywhere else. The `[[repo]]` entry is Wave-0's addition (expected post-Wave-0).

**Status: VERIFIED.** The claim is accurate.

---

## F-A5 · Issue #128 characterization — VERIFIED ✓

**Plan text at issue (§3 table):**

> "#128 | `tests/m2_daemon_api.rs`, submit-path code
> (`src/runtime/engine.rs`/`surface.rs`), `docs/perf/` | Submission throughput
> floor measures ~5 works/s vs. a 12 works/s floor on this hardware — real
> profiling needed, not a guessed fix; may end in a code fix, a justified floor
> revision, or neither if the cause resists this pass"

**What was measured in-session (`gh issue view 128 --repo miztertea/sergeant-rs`):**

Issue body says: "burst 25 (full submit path): 5.0 works/s in 4.994275917s"
and "submission throughput fell to 5.0 works/s at burst 25, below the 12
works/s floor". Measured 3 times: 4.8, 5.0, 5.1 works/s. Issue explicitly
states profiling is needed: "genuinely isolating which part of the submit path
costs more on macOS … needs actual profiling, not a plausible guess."

**Status: VERIFIED.** The "~5 works/s vs. 12 works/s floor" and "real
profiling needed" characterizations match the issue body exactly. The file
scope (`surface.rs`, `engine.rs`) is the plan's own inference about the submit
path's implementation — not stated in the issue, which only names the test file.
That inference is plausible given engine.rs's observed ownership of the submit
pipeline (verified via grep), and is presented as the plan's scope estimate, not
as a claim about what the issue says.

---

## F-A6 · Issue #129 characterization — VERIFIED ✓

**Plan text at issue (§3 table):**

> "#129 | `src/runtime/recovery.rs`, `src/runtime/engine.rs`,
> `tests/m3_execution.rs` | Deterministic extra `OBSERVE` in restart
> reconciliation (3 vs. expected 2), pure engine logic, no OS process
> involved — genuine engine-logic debugging"

**What was measured in-session (`gh issue view 129 --repo miztertea/sergeant-rs`):**

Issue body: "Three OBSERVEs recorded against the survivor's `first_execution`
id, where the test expects exactly two … this is pure engine/recovery logic
(`src/runtime/recovery.rs` / `src/runtime/engine.rs`), not a platform-boundary
fact" and "uses only `FakeBackend` (no real OS process for the backend under
test, no `/proc`/`ps` involved)."

**Status: VERIFIED.** "3 vs. expected 2" matches the issue verbatim ("Three …
expects exactly two"). "Pure engine logic, no OS process involved" matches
verbatim. File scope matches.

---

## F-A7 · Issue #130 characterization — VERIFIED ✓

**Plan text at issue (§3 table):**

> "#130 | `scripts/gate.sh` | `daemon_env_ok()`'s `/proc/<pid>/environ` grep
> has no macOS equivalent; the issue's own body sketches a candidate fix (skip
> the check on the direct-fork/non-systemd path, keep it where a supervisor
> could silently drop env). **[v3] In scope, not a gate dependency**"

**What was measured in-session:**

`gh issue view 130 --repo miztertea/sergeant-rs` confirmed:

- `daemon_env_ok()` is in `scripts/gate.sh`. Actual line range verified via
  `grep -n "daemon_env_ok"` and `Read` of gate.sh lines 85–96: function starts
  at line 89, closes at line 96 — exactly the range the issue's body cites as
  `gate.sh:89-96`.
- Function body uses `grep -qz CARGO_TARGET_DIR "/proc/$pid/environ"` — no
  platform branch.
- Issue's fix sketch: "skip `daemon_env_ok`'s check specifically on the
  direct-fork (non-systemd) path … keep it as a hard requirement only where a
  supervisor (systemd) could silently drop the env." Plan's paraphrase ("skip
  the check on the direct-fork/non-systemd path, keep it where a supervisor
  could silently drop env") is accurate.

**Status: VERIFIED.** File, function name, line range, macOS gap, and fix
sketch all confirmed against primary sources.

---

## F-A8 · PR #126 state — VERIFIED ✓

**Plan text at issue (§2, R1):**

> "PR #126 stays the head PR; no direct push to `main`"

**What was measured in-session (`gh pr view 126 --repo miztertea/sergeant-rs`):**

PR #126 title: "Macbook arrival: measure, flip UNVERIFIED, choose #95's clock"
State: **OPEN**. URL: `https://github.com/miztertea/sergeant-rs/pull/126`.
auto-merge: disabled.

**Status: VERIFIED.** PR #126 exists and is open.

---

## F-A9 · Estate clone tip "471963a as of Wave 0" — PLAUSIBLE

**Plan text at issue (§4):**

> "**[measured]** `sgt repo list` now shows `sergeant-rs` declared, `sgt
> doctor` reports `estate: 1 repositories declared, all present on disk`; the
> estate clone's `git log -1` is `471963a` (pre-plan tip) as of Wave 0 — it
> must be re-synced to this plan's own commit before Wave 1 dispatches"

**What was measured in-session:**

```
cd /Users/thawes/.../repos/sergeant-rs && git log -3 --oneline
0a7e723 docs/gauntlet: MACBOOK-ARRIVAL-1 plan v3 — correct #130's scope and dispatch mechanics
0110551 docs/gauntlet: MACBOOK-ARRIVAL-1 plan and review contract
471963a Fixes #95: choose perl -MTime::HiRes as the macOS perf clock, measured on real hardware
```

The estate clone is now at `0a7e723` — the current tip, which is exactly what
the plan says the re-sync should achieve. Commit `471963a` is the
third-most-recent commit and is the last commit before the plan was authored
and committed, which is consistent with the plan's claim that the estate clone
sat at `471963a` immediately after Wave 0 (Wave 0 ran before the plan
commits `0110551`/`0a7e723` existed).

**Status: PLAUSIBLE.** The claim is consistent with the observable commit
ordering and the current re-synced state of the estate clone. It cannot be
verified retroactively (the clone's HEAD at Wave-0-time is no longer visible),
but no counter-evidence exists.

---

## Summary table

| ID | §§ | Claim | Status | Severity |
|---|---|---|---|---|
| F-A1 | §5 | "disjoint file sets" across WA/WB/WC | CONFIRMED wrong label, plan self-corrects | WARNING |
| F-A2 | §6 | "no Work used more than 2 turns against a well-scoped brief" | CONFIRMED inaccurate for gate dispatches | INFO |
| F-A3 | §4 | Pre-Wave-0 "0 repos declared" / "nothing has ever run" | PLAUSIBLE — not independently verifiable post-Wave-0 | — |
| F-A4 | §6 | `sergeant.toml` has no `[profile.*]` / no `default_backend` | VERIFIED ✓ | — |
| F-A5 | §3 | Issue #128: ~5 works/s vs 12 works/s floor, profiling needed | VERIFIED ✓ | — |
| F-A6 | §3 | Issue #129: 3 vs expected 2 OBSERVEs, pure engine logic | VERIFIED ✓ | — |
| F-A7 | §3 | Issue #130: `daemon_env_ok()` at lines 89-96, fix sketch | VERIFIED ✓ | — |
| F-A8 | §2 | PR #126 exists and is open | VERIFIED ✓ | — |
| F-A9 | §4 | Estate clone at `471963a` as of Wave 0 | PLAUSIBLE — consistent with commit ordering | — |

**Upward path:** F-A1 is the only finding a corrector should act on (wrong
word, easy fix). F-A2 is informational — the envelope decision is sound; the
stated reasoning is imprecise. All other findings are either verified clean or
recorded PLAUSIBLE per the contract.
