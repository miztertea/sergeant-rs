# Path-to-Mac sprint plan — review (two-axis code-review pass)

**Fixed point:** `main` (`242abe3`) → `d86885f`, one file:
`docs/gauntlet/runs/path-to-mac-2026-08-15/plan.md`, 221 insertions. Confirmed
via `git diff main..d86885f --stat` and `git diff d86885f..HEAD -- .../plan.md`
(empty — the file is unchanged since that commit).

**Scope note.** This is a fresh-context review, per `LESSONS.md` L19: the plan
directs Work dispatch and is therefore executable through the sessions that
obey it, so it takes the review loop before it governs. Per L3, `GAUNTLET.md`'s
deviation register and ledger rulings were read before filing; no finding
below re-litigates a registered deviation without arguing the ruling itself is
wrong. Per L15, every claim below is marked **VERIFIED** (checked in-session
against the cited file/line) or **BELIEVE** (inference not independently
re-run). No script or gate was run; no crate measurement was re-derived; the
artifact itself was not edited.

Spec sources consulted: `NORTH-STAR.md`, ADR 0001, ADR 0002, ADR 0005,
`docs/DEVELOPMENT.md`, `LESSONS.md`, `GAUNTLET.md` (deviation register,
backlog, FOUNDATION-1 entry).

---

## Standards

Findings about this repo's own documentation and process conventions — the
plan's stated citation/evidence discipline (its own preamble: "Where a claim
is a measurement taken this session, it is marked **[measured]**; where it is
read from a repo artifact, the artifact is cited; where it is a belief, it
says so (**L15**)") — as distinct from whether its factual claims are true.

### S1 — Medium: the plan's central wave-sequencing claim is asserted with none of the plan's own evidence tags, and is the one claim in the document that turns out to be checkable and wrong

**Plan text at issue** (`plan.md:129-130`):
> W1 and W3 are the only parallel pair; their file sets are disjoint (`src/` +
> `Cargo.toml` vs `tests/` + `scripts/perf/`).

This sentence carries no `[measured]` tag, no artifact citation, and no belief
label — it reads as settled fact, in a document whose own preamble commits to
labeling every claim one of those three ways. It is exactly the kind of
claim the plan's own discipline exists to catch: a two-bucket file-ownership
assertion, checkable in seconds against the plan's own §4 table, that does not
hold (see Spec finding P2 — the same file, `tests/support/mod.rs`, is named in
§4 as part of W1's edit and is the natural landing site for W3's #108 fix).
The lapse is procedural, independent of which direction the fact turns out to
run: a claim load-bearing enough to justify skipping conflict resolution for
the sprint's only parallel wave pair should have carried the same evidentiary
weight the document applies everywhere else.

**Governing text:** `plan.md:9-12` (the plan's own stated discipline, itself
grounded in `LESSONS.md` L15, `LESSONS.md:186`).

**Correction:** Tag the disjointness claim `[measured]` with the check shown
(`grep -rn 'tests/support/mod.rs' plan.md`, or diff the two Works' expected
file lists) — or drop the "conflicts do not arise" framing to "expected to
be low-conflict, unverified" until that check is actually run.

**VERIFIED** — read `plan.md:1-14` for the stated discipline and `plan.md:129-130` for the untagged claim.

### S2 — Low: DEVELOPMENT.md citation cites a single line for a rule spanning four

**Plan text at issue** (`plan.md:48-49`):
> `docs/DEVELOPMENT.md` line 71 states the rule; **ADR 0005 (Accepted,
> 2026-08-14)** dissolved it

`docs/DEVELOPMENT.md`'s "an actor never invokes the gate" rule spans lines
70-73 (`- A workflow stage or actor executing inside a worktree never invokes` /
`` `scripts/gate.sh`/no-mistakes itself — only the top-level orchestrating `` /
`session owns a shipping-gate run, matching the single-owner posture the` /
`engine itself enforces on the data dir.`). Line 71 alone is a sentence
fragment ("`scripts/gate.sh`/no-mistakes itself — only the top-level
orchestrating"); the clause the plan is citing only completes across all four
lines. This repo's own convention elsewhere (`docs/DEVELOPMENT.md:85`'s D-number
citation rule: "a citation crossing between the two always pairs...") treats
precise citation as something worth getting right; a single-line pointer into
a four-line rule sends a reader to an incomplete sentence.

**Governing text:** `docs/DEVELOPMENT.md:70-73`.

**Correction:** Cite the range, "lines 70-73," not a single line.

**VERIFIED** — `grep -n` against `docs/DEVELOPMENT.md` confirms the rule's exact line span.

**Standards summary: 2 findings (0 high, 1 medium, 1 low). Worst: S1 (medium) — the plan's own evidence-labeling discipline lapses on exactly the claim that turns out to be false.**

---

## Spec

Findings about whether the plan's claims are actually true against the cited
governing documents and repository ground truth.

### P1 — High: the plan attributes a quoted objection to ADR 0002 (D4) that does not appear anywhere in ADR 0002, and skips the document that actually makes it

**Plan text at issue** (`plan.md:93-98`):
> **This retires ADR 0002 (D4)'s objection on its own terms.** That decision
> declined "adding a libc-binding dependency for one syscall." `libc` and
> `rustix` are already dependency edges this crate carries, and the count is
> now four facts rather than one. **An ADR refresh is owed** [...] Assigned to
> **W1**.

I read ADR 0002 in full. D4 ("What is behind the boundary") is about which
facts belong to the platform boundary versus remaining `Backend` capabilities
(Docker/claude CLI semantics) — it says nothing about a libc-binding
dependency, one syscall, or `#81`/disk-space measurement at all. I then
searched the repository for the exact quoted phrase:

```
grep -rn "libc-binding\|one syscall" docs/ src/ reference/ *.md
```

The only real match is `src/platform/disk.rs:1-20`'s module doc:

> Free disk space (#81).
>
> `df` remains the mechanism — not a `libc`/`statvfs` binding. The module
> this fact used to live in (`src/backend/docker.rs`) explicitly declined
> that binding "for one syscall" [...] #81 asks whether that tradeoff still
> holds now that the shell-out has a *measured* portability cost [...] **It
> still holds**: the fix below is a second, POSIX-portable invocation shape
> (`df -k <path>`, no `--output`) [...]

(`src/cli.rs:1528-1531` contains the phrase "for one syscall" too, but about
`kill(1)` vs. a signals crate for process termination — an unrelated
objection, not the disk/libc one the plan is citing.)

Two problems follow from this:

1. **The citation is wrong.** ADR 0002 (D4) has no objection to retire; the
   plan is quoting `src/platform/disk.rs`'s own module doc and mislabeling it
   as an ADR ruling. If W1's brief instructs it to update "ADR 0002 (D4)"
   based on this text, the Work will not find the passage there.
2. **The actual objection is current and still affirmed, not stale.** The
   real source — `src/platform/disk.rs`'s module doc, which is live code
   documentation, not a frozen ADR — explicitly re-examined this exact
   question and concluded "It still holds," with a specific, dated argument
   (BSD/macOS `df --output` failure is a *measured* cost now, but the
   POSIX-portable `df -k` reparse is judged the better trade over adding a
   dependency). The plan's crate-adoption case for `fs4`/`statvfs` (§4) does
   not engage with that argument at all — it treats the objection as an ADR
   artifact awaiting a "refresh," when it is instead a currently-standing,
   reasoned decision in the code the Work is about to reverse.

The plan's own point — that `libc`/`rustix` being pre-existing dependency
edges changes the calculus — may well be a sufficient answer to
`disk.rs`'s argument. But that argument has to be made against the text
that actually holds the objection, not against an ADR clause that does not
exist.

**Governing text:** `docs/adr/0002-platform-boundary-shape.md` (full text, no
"one syscall" passage anywhere) vs. `src/platform/disk.rs:1-20` (the actual,
currently-affirmed objection).

**Correction:** Retarget the citation from "ADR 0002 (D4)" to
`src/platform/disk.rs`'s module doc; have W1's brief require directly
answering "It still holds" rather than treating the change as an ADR
refresh. If an ADR update is still warranted once that argument is made, it
is a new decision, not the "retirement" of an existing one.

**VERIFIED** — read `docs/adr/0002-platform-boundary-shape.md` in full;
`grep -rn "libc-binding\|one syscall" docs/ src/ reference/ *.md` run this
session; read `src/platform/disk.rs:1-20` and `src/cli.rs:1520-1536`.

### P2 — High: W1 and W3's file sets are not disjoint — both touch `tests/support/mod.rs`, contradicting the plan's own no-conflict argument for its only parallel wave pair

**Plan text at issue** (`plan.md:106-118, 129-130`):
> Waves exist because of **file ownership**, not logic. [...] Advancing *its*
> HEAD to the integration tip between waves means every later Work cuts from
> the previous wave's result — so conflicts do not arise rather than being
> resolved. [...] W1 and W3 are the only parallel pair; their file sets are
> disjoint (`src/` + `Cargo.toml` vs `tests/` + `scripts/perf/`).

The plan's own §4 lists W1's #18 scope as replacing "direct `/proc` reads in
`daemon.rs`, `backend/claude.rs`, **`tests/support/mod.rs`**" (`plan.md:77`).
That is a `tests/` file, not a `src/`+`Cargo.toml` one — it falls inside the
bucket §5 assigns exclusively to W3.

Separately, W3 owns #108 (R6: "fixed by a start-of-run reaper, using standard
patterns"). I checked where that fix actually lands. `tests/support/mod.rs`
*is* the file: its own module doc says "Kept deliberately dependency-free
(`kill(1)` and `/proc`" and it already implements `DataDir`'s `Drop`-based
`/proc` argv-scan reaper (`reap_daemons`, `tests/support/mod.rs:158,193-244`).
The cross-platform retrospective's own recommendation for #108, which the
plan cites elsewhere, names this exact mechanism: "Extending `DataDir`'s
`/proc`-scan reaping to run at suite start [...] closes the class"
(`docs/gauntlet/runs/cross-platform-2026-08-14/retrospective.md:319-322`).
There is no other plausible home for a start-of-run reaper than the file that
already defines `DataDir`.

So both W1 (per the plan's own §4) and W3 (per the plan's own R6 and the
retrospective it cites) edit `tests/support/mod.rs`. The two are dispatched
in the same wave, "‖" (parallel), both cut from `main` (§5's table). If both
touch that file — which, per §4, W1 explicitly intends to rewrite the `/proc`
reads inside — advancing the estate clone's HEAD between waves does not make
the conflict "not arise"; it just defers a real content conflict in that file
to whichever of the two lands second, requiring the hand-merge the plan's own
reasoning ("buys zero hand-authored merges") says this scheme avoids.

I could not fully verify from the plan text alone whether W1's edit and W3's
edit would touch overlapping *lines* within `tests/support/mod.rs` (that
depends on how each Work is actually briefed) — but the plan's claim is
narrower and stronger than "low risk of overlap": it asserts the file *sets*
are disjoint, which is false on the plan's own evidence, independent of where
in the file each edit lands.

**Governing text:** `plan.md:77` (W1's stated file scope) vs. `plan.md:129-130`
(the disjointness claim) — an internal contradiction — corroborated against
`tests/support/mod.rs:1-244` (VERIFIED) and
`docs/gauntlet/runs/cross-platform-2026-08-14/retrospective.md:319-322`
(VERIFIED, the #108 fix's named mechanism) and `docs/DEVELOPMENT.md:47`
(VERIFIED, `tests/support/mod.rs`'s role as the shared `DataDir` guard every
daemon-spawning suite must go through).

**Correction:** Either serialize W1 and W3 (breaking the "only parallel pair"
claim and its wall-clock justification), or scope the two Works' briefs to
touch non-overlapping regions of `tests/support/mod.rs` explicitly and say so
in the wave table — the current table's bucket labels (`src/` + `Cargo.toml`
vs `tests/` + `scripts/perf/`) are not accurate as written.

**VERIFIED** — read `plan.md` §4 and §5 in full; read
`tests/support/mod.rs:1-244`; read
`docs/gauntlet/runs/cross-platform-2026-08-14/retrospective.md:90-105,
319-322`; read `docs/DEVELOPMENT.md:47`.

### Lines of attack checked with no finding

Per the review brief, a clean result on a targeted line of attack is reported
as such rather than omitted.

- **Ruling R8 / ADR 0005 (`plan.md:45,48-55`).** ADR 0005's Decision section
  quotes `docs/DEVELOPMENT.md`'s rule near-verbatim and states "The rule
  dissolves rather than being amended (D1)," and its Consequences section
  ("Captain's job changes shape... instead of driving `axi respond` and
  `axi sync --recover` plumbing by hand for every gate") confirms the
  orchestrator no longer invokes the gate directly. R8's claim that ADR 0005
  supersedes `docs/DEVELOPMENT.md`'s line-70-73 rule holds up; it is not an
  over-reading of a narrower ADR. (**VERIFIED** — read
  `docs/adr/0005-gating-becomes-a-dispatched-work.md` in full.) See Standards
  S2 for a citation-precision nit on the line number only.
- **The attach claim (`plan.md:186-188`, risk 3).** `surface::materialize`
  (`src/runtime/surface.rs:353-456`) cuts a Work's branch from
  `git rev-parse HEAD` on the source repository path at dispatch time
  (`surface.rs:431-441`) — an ordinary fresh cut, not `attach`
  (`surface.rs:561-`, the branch-takeover mechanism ADR 0005 describes for a
  gate Work reviewing *another Work's own* branch). If W6 is dispatched after
  the orchestrator's local clone HEAD has been advanced to the integration
  tip, its fresh-cut branch already contains everything merged so far, and
  `attach` genuinely is not needed for that specific case. The plan's own
  caveat ("That works here and does not generalize") is accurate — this
  reasoning does not extend to a gate Work reviewing an arbitrary other
  Work's un-merged branch, which is exactly the gap ADR 0005's open questions
  describe. (**VERIFIED** — read `src/runtime/surface.rs:327-460, 561-`.)
- **#85 / sysinfo Disks API labeling (`plan.md:78, 100-105, 216-218`).** The
  plan frames the whole crate table as "Researched via Context7 against the
  crates' own documentation rather than from training data" (`plan.md:70-71`)
  and separately states in §10 that macOS behavior is unmeasured: "the crates
  make the claim cheap, they do not make it measured." That is an honest
  label, not an overclaim. I went further than the plan's own citation and
  checked the vendored crate source
  (`~/.cargo/registry/src/.../sysinfo-0.37.2`): `Disk::file_system()`
  (`src/common/disk.rs:63`) is backed by a real, non-stub macOS
  implementation reading `libc::statfs.f_fstypename`
  (`src/unix/apple/disk.rs:55, 450-455`) — the capability the plan is relying
  on genuinely exists in the crate, beyond what "documentation says so" would
  guarantee on its own. (**VERIFIED**, beyond what the plan itself claims to
  have checked.)
- **R4 vs. AGENTS.md's preserved-state guardrail (`plan.md:41`).** The
  cross-platform retrospective — which the plan draws R4 from — already
  worked through this exact tension: "Teardown preserves uncommitted work and
  never deletes a branch, and `AGENTS.md` puts preserved state outside
  anything standing authorization may destroy. **The gap is the other
  half:**... Essentially the whole 30 GB is `target/`: build artifacts,
  gitignored, not 'uncommitted work' under any reading"
  (`docs/gauntlet/runs/cross-platform-2026-08-14/retrospective.md:44-56`).
  AGENTS.md's guardrail (`AGENTS.md:202-205`) names three protected
  categories — "a retained branch, a journal, a Work record" — none of which
  is a gitignored build directory. R4 ("retain the dirty state, never the
  directory") is also an explicit owner ruling from a live interview, not
  autonomous action the "standing authorization" clause is scoped to govern.
  I found no conflict. (**VERIFIED** — read `AGENTS.md:189-207` and the
  retrospective section cited above.)
- **Envelope reasoning / `--ceiling-secs 5400` (`plan.md:134-141`).** The
  plan's own text already separates the two effects a longer ceiling has —
  fewer false-positive kills of turns that would have finished ("Fewer
  ceiling firings is the only lever available tonight") against a larger
  blast radius on a genuinely stuck turn ("a genuinely stuck turn burns 90
  minutes before it surfaces") — and states the tradeoff plainly rather than
  claiming the ceiling change fixes #90. That is a defensible
  frequency-vs-severity trade, honestly captioned as a mitigation, not a fix.
  I found no unstated logical gap. (**VERIFIED**, read against #90's
  description in `docs/gauntlet/runs/cross-platform-2026-08-14/close-out.md:45`
  and `plan.md` §8 risk 1, which itself says "Mitigated only by envelope
  sizing.")
- **Scope vs. NORTH-STAR gating (`plan.md:57-66`).** None of the plan's ten
  in-scope issues (#18, #81, #82, #85, #90, #94, #95, #96, #108, #109) match
  anything on `NORTH-STAR.md`'s Gated ("not yet") or Never lists (stranger
  onboarding, T-series full spec, H1 contract-v2 remainder, N4 Docker, G3
  callbacks, G1 scheduler, estate graph, clean-distro extraction, fleet as a
  domain object, PM semantics, etc. — `NORTH-STAR.md:105-115`). All ten read
  as bug/hygiene fixes against an already-shipped MVP surface (engine
  honesty, platform facts, harness cleanup), not features the North Star
  explicitly sequences later. I found no gating violation. (**VERIFIED** —
  read `NORTH-STAR.md` in full and cross-checked each issue's description
  against the Gated/Never lists.)
- **FOUNDATION-1 method-note citation (`plan.md:205-210`).** The plan's claim
  — "the three axes whose refuter was given a *specific line of attack*
  produced the unit's only refutation and all three severity downgrades" —
  matches `GAUNTLET.md:171-173` nearly verbatim ("Method note carried
  forward: the three axes whose refuter was given a **specific line of
  attack** produced the unit's only refutation and all three downgrades").
  Accurate citation. (**VERIFIED** — read `GAUNTLET.md:121-179`.)

**Spec summary: 2 findings (2 high, 0 medium, 0 low). Worst: P1 and P2 (tied,
high) — one misattributes a currently-affirmed code-level objection to an ADR
that never made it, the other falsifies the file-disjointness claim the
sprint's only parallel-wave pair depends on. Six additional lines of attack
(R8/ADR 0005, the attach claim, #85's crate labeling, R4 vs. the AGENTS.md
guardrail, the envelope reasoning, and scope vs. NORTH-STAR) were checked and
returned no finding.**

---

## Closing summary

**Standards: 2 findings (worst: S1, medium).** **Spec: 2 findings (worst: P1
and P2, tied at high).** The two axes are not merged or reranked against each
other; no overall winner is picked. Both Spec findings should be resolved
before W1 and W6's briefs are written — P1 changes what W1 is actually being
asked to answer, and P2 changes whether W1/W3 can run in parallel at all.
