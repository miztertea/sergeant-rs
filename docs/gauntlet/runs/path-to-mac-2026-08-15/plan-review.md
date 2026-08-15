# Path-to-Mac sprint plan — review (two-axis code-review pass)

**Fixed point:** `main` (`242abe3`) → `HEAD` (`d86885f`), diff confirmed
non-empty before this review started: one file,
`docs/gauntlet/runs/path-to-mac-2026-08-15/plan.md`, 221 insertions.

**Scope note.** This is the narrower `code-review` pass named in
`docs/gauntlet/contracts/PATH-TO-MAC-1.md` (that contract lives on
`integration/path-to-mac-2026-08-15`, dispatched separately) — "a narrower
`code-review` pass (Work `01M01XGMY8JJ4M4RJDN1RSZXJC`, two axes)... is input
to this unit, not a substitute for it." Standards/Spec, not the contract's
four-axis fidelity/invariants/enactability/assumptions panel. This review does
not read or anticipate that panel's output, and its own findings should not be
treated as a substitute for it.

**Method note, stated rather than left implicit.** The reviewed skill
(`reference/sergeant-upstream/.agents/skills/code-review/SKILL.md`) spawns
Standards and Spec as two parallel, context-isolated sub-agents so neither
axis's framing leaks into the other. This pass was instead run as one direct
investigation: by the time the Spec-axis verification (reading the ADRs,
`src/runtime/surface.rs`, `LESSONS.md`, `GAUNTLET.md`, and grepping the repo
for cited quotes) was complete, spawning a fresh Standards sub-agent would have
either duplicated that reading or worked from a thinner base. The risk this
trades away is real — a single reviewer can let one axis's findings color the
other — and is flagged here rather than smoothed over. Every finding below
still states its axis and is argued on that axis's own terms; the Standards
axis's clean result was checked for, not assumed.

Every finding distinguishes **VERIFIED** (checked in-session against the cited
file/line or a grep run this session) from **BELIEVE** (reasoned argument, not
a document lookup) per L15. The nine owner rulings in plan §2 are not
re-litigated; findings below are about the plan's stated *justification* for a
ruling, or about text outside §2 entirely.

---

## Standards

Checked against `docs/DEVELOPMENT.md` and `LESSONS.md`'s own meta-rules for
how a governing artifact in this repo should be written and cited — not
against Fowler's smell baseline, which targets code and does not transfer
cleanly to sprint-planning prose.

**No violations found.** Specifically checked and passed:

- **L23 (cite the file that holds the content, never the alias).** `docs/DEVELOPMENT.md:8` is the exact citation this rule exists to protect (`CLAUDE.md` is a symlink; `AGENTS.md`/`DEVELOPMENT.md` hold the content). VERIFIED: `grep -n "CLAUDE.md" plan.md` returns nothing — the plan never cites the symlink path. Every `docs/DEVELOPMENT.md` reference (§2 R8, §9) names the real file.
- **File:line citation accuracy.** Plan §5 (line 111) cites `src/runtime/surface.rs:332` and `431-441` for "a work branch is cut from the repository's current HEAD." VERIFIED: line 332 reads verbatim "work branch cut from that repository's current HEAD"; lines 431-441 are the `base_sha`/`add_worktree` sequence that does the cutting. Accurate.
- **Retrospective citation.** Plan §5 (lines 116-118) attributes "anything that survives as a commit is a Work, with no exception for size" to "§4 of the cross-platform retrospective." VERIFIED: the quote is at `docs/gauntlet/runs/cross-platform-2026-08-14/retrospective.md`, section "4. The honest accounting," word for word (that section itself attributes the line onward to `lessons.md` H, but the plan only claims the retrospective's §4 states it, which is true).
- **L15 (evidence vs. belief labeling).** The plan tags session-taken measurements `[measured]` throughout §4 and §6, and explicitly separates what §4 measured (binary size, compile time, present/absent crates) from what it did not ("Not measured, and it is the number that matters for #18: `sysinfo`'s runtime cost..."). This labeling discipline is consistent and did not need correction anywhere in the diff.

One judgment call, not a violation: §4's line 91 stretches L23's "read the artifact, not the view of it" to cover a cache hit misread as a fresh timing (a silently-failed `cargo clean`). That is a fair generalization of L23's own stated shape ("any indirection between you and the artifact"), not a misuse of the citation.

**Summary — Standards: 0 findings.** This is a legitimate result, not a gap in the pass: the checks above were specifically chosen from the artifact classes L23/L15 exist to catch, and the plan held on all of them.

---

## Spec

Checked the plan's content against `NORTH-STAR.md`, ADR 0001/0002/0005,
`docs/DEVELOPMENT.md`, `LESSONS.md`, and `GAUNTLET.md`'s deviation register —
plus, where a claim is about the codebase itself, the cited source directly.

### 1. [error] §4's ADR 0002 (D4) citation does not exist in ADR 0002

**Plan text (lines 93-96):** *"This retires ADR 0002 (D4)'s objection on its
own terms. That decision declined 'adding a libc-binding dependency for one
syscall.'"*

**Governing text it contradicts:** `docs/adr/0002-platform-boundary-shape.md`,
decision D4 ("What is behind the boundary," lines 46-56) is entirely about
Docker/`claude` CLI being out of scope of the platform-fact boundary — it says
nothing about libc, syscalls, or dependency cost. **VERIFIED**: `grep -rn
"libc-binding\|one syscall" docs/ reference/ .` across the whole repository
returns exactly one hit — the plan's own line. No ADR, ledger entry, or note
contains this objection in any form.

The plan's own preamble says every ruling's "provenance... is the interview
itself," which covers §2's nine rulings. This sentence is different: it names
a specific accepted document and decision letter and puts the objection in
quotes, which reads as sourcing the objection to written text that does not
say it. If the objection was raised live and never written into the ADR, that
is a fact about the ADR being incomplete, not license to attribute it to D4 as
if D4 already says it.

**Why this matters:** §4's whole paragraph is the argument for why adding
`fs4`/`sysinfo`/`directories` doesn't reopen a settled objection. If the
objection this paragraph "retires" was never actually registered in ADR 0002,
the paragraph is retiring something that was never an obstacle, and the real
question — whether ADR 0002's actual reasoning (D2/D3: platform is
compile-time, not worth a trait; D4: Docker/claude stay out of scope) has
anything to say about this specific dependency addition — goes unaddressed.

**Correction:** either cite where the objection actually lives (an interview
transcript, a different note) and label it as unwritten/interview-only rather
than "ADR 0002 (D4)," or drop the "retires ADR 0002 (D4)" framing and argue
the dependency addition on its own terms.

### 2. [error] §8 risk 3's "no attach needed" claim reproduces, not sidesteps, FOUNDATION-1's finding

**Plan text (lines 184-188):** *"Review is Captain-serial. ADR 0005's items 3
and 4... remain unbuilt, so a gate Work cannot be bound to another Work's
branch. This sprint sidesteps it: W6 cuts from the integration tip and
therefore already holds the content, never needing `surface::attach`. That
works here and does not generalize."*

**Governing text it contradicts:** `docs/adr/0005-gating-becomes-a-dispatched-work.md`
(Consequences, lines 84-108) built `surface::attach` specifically because
`materialize()` "mints its own `sergeant/<work-id>` branch and cannot bind to
a branch it did not mint" — and `GAUNTLET.md`'s FOUNDATION-1 entry (lines
141-153) names the failure mode this was fixing: *"reviewing a copy breaks
no-mistakes' recovery of auto-fix commits onto the shipped branch, yielding
'a gate Work that passes its own copy while the actual shipping branch never
received the pipeline's fixes.'"*

**VERIFIED** against `src/runtime/surface.rs`: `materialize()` (doc comment,
line 331-332) always cuts "a fresh work branch... from that repository's
current HEAD" — a brand-new `sergeant/<w6-work-id>` branch, distinct from
whatever branch it was cut from. `attach()` (line 561) is the only path that
checks a surface's worktree onto an *existing* branch instead of minting one.
Plan §5's own wave table (line 127) has W6 cut "from: integration tip" —
i.e. via ordinary `materialize()`, not `attach()`.

The plan's §7 explicitly grants W6 auto-fix authority ("The Work may
authorize `auto-fix`"), and §7/R9 name a persistent `integration/…` branch
with an early-opened head PR as the thing "the owner merges." If W6's gate run
authorizes an auto-fix commit, that commit lands on W6's own freshly-minted
branch — not on the integration branch with the open PR — for exactly the
reason FOUNDATION-1 named: content parity at cut time is not branch identity.
"Already holds the content" answers the *review-reads-correct-content*
question `attach` was never needed for; it does not answer the *auto-fix
write-back* question `attach` was built for. The plan states the content
argument as if it covers both.

**Correction:** either W6 must attach to the integration branch itself
(reintroducing the unbuilt-mechanism problem this risk already names), or the
plan needs an explicit reconciliation step — merge/fast-forward W6's branch
onto integration, or make W6's branch the one the owner actually merges —
before "the owner merges" in §7. Left as written, the line the task brief
flagged ("if it is wrong, the sprint's final wave does not work") applies: W6
can pass its own gate while shipping nothing back to the branch under review.

### 3. [warning] W1/W3 file-disjointness claim is contradicted by the plan's own §4 table

**Plan text (lines 129-130):** *"W1 and W3 are the only parallel pair; their
file sets are disjoint (`src/` + `Cargo.toml` vs `tests/` + `scripts/perf/`)."*

**Governing text it contradicts:** the plan's own §4 crate table (line 78 row
for #18, part of W1's scope) names `tests/support/mod.rs` as one of the three
call sites #18 replaces ("direct `/proc` reads in `daemon.rs`,
`backend/claude.rs`, `tests/support/mod.rs`"). `docs/DEVELOPMENT.md:47`
describes `tests/support/mod.rs`'s `DataDir` guard as the thing that "reaps by
`/proc` argv scan on Drop"; `docs/gauntlet/runs/cross-platform-2026-08-14/retrospective.md`
§1.3 (lines 95-99) names extending that exact guard's reaping "to
start-of-run... would close the class" as the fix shape for the SIGKILL/Drop
problem R6 cites for #108 (W3's scope). **VERIFIED**: `grep` confirms no
`tests/` file currently references `platform::`/`fs4`/`sysinfo`/`directories`
(so #18's tests/support/mod.rs edit is real, new work, not already done), and
`tests/support/mod.rs`'s own `DataDir` guard is exactly where a start-of-run
reaper for orphaned daemons already lives per `DEVELOPMENT.md:47`.

Both W1 (#18, replacing `/proc` reads in `tests/support/mod.rs`) and W3
(#108, plausibly extending `tests/support/mod.rs`'s reaping to start-of-run)
have a documented reason to touch the same file. That contradicts "file sets
are asserted disjoint" as the basis for running them in parallel.

**Correction:** name the actual target file(s) for #18's `tests/support/mod.rs`
change and #108's reaper before dispatch; if they collide, resequence W1/W3
rather than running them ‖.

### 4. [warning] R6's stated cause for #108 does not match the source it's describing

**Plan text (line 43, R6):** *"#108 is fixed by a **start-of-run reaper**,
using standard patterns | `Drop` does not survive `SIGKILL`, and SIGKILL is
how these die."*

**Governing text it contradicts:** `LESSONS.md` L21 (lines 70-85) — the entry
that files #108 — attributes the leak to the dead-man test's premise: *"the
dead-man test — whose whole premise is that the release path never appears —
removes nothing... the test that exercises the abnormal path is the least
likely to carry the cleanup."* Nothing in L21 mentions `SIGKILL` or `Drop`.
The "`Drop` does not survive `SIGKILL`" reasoning is `retrospective.md` §1.3
(lines 90-99), which is explicitly about a *different* residue row — the 1.7 GB
`/var/tmp/sgt-rs-tests` test rigs, generalized from #91 — not #108 (§1.2,
lines 77-85).

**BELIEVE, argued from both sources read in-session:** a start-of-run reaper
is still a defensible fix for #108 (sweeping stale markers left by prior
incomplete runs is a reasonable independent justification), but the reason
given is borrowed from the wrong row of the same retrospective.

**Correction:** state #108's actual cause (test premise, not process death),
or keep the SIGKILL/Drop reasoning attached to the residue item it actually
describes and give R6 its own, correct rationale.

### 5. [info] §6/§8's ceiling-size argument overstates what it mitigates

**Plan text (lines 136-140, and §8 line 182):** *"#90's fix cannot protect
its own sprint... Fewer ceiling firings is the only lever available tonight.
The cost is that a genuinely stuck turn burns 90 minutes before it surfaces."*
§8: *"#90 is unprotected tonight. Mitigated only by envelope sizing (§6)."*

**BELIEVE**, reasoned from #90's own documented shape
(`docs/gauntlet/runs/cross-platform-2026-08-14/lessons.md:78-89`, `close-out.md:45`:
"ceiling interrupt wedges a Work in `active` with no verb that reaches it"): a
larger `--ceiling-secs` reduces the *rate* of firings only for turns that would
have finished inside the new window. For a genuinely stuck turn, #90's actual
defect — no exit door once the ceiling does fire — is untouched; the turn
still wedges, later and at higher sunk cost, which is exactly what the plan's
own next sentence admits. Calling this "mitigated" claims more than the
mechanism delivers; "the incident rate is lower, but #90's defect is unchanged
and each remaining incident costs more" is the more accurate framing, and the
plan already has all the facts needed to state it that way.

**Correction:** narrow §8 risk 1 to "incident rate reduced, defect
unmitigated" rather than "mitigated."

### No finding — R8 (ADR 0005 supersedes `DEVELOPMENT.md:71`)

**VERIFIED**: `docs/DEVELOPMENT.md:70-73`'s session-conduct bullet is quoted
verbatim in ADR 0005's own Context section, and ADR 0005's Decision section
states plainly, "The rule dissolves rather than being amended (D1)." R8's
characterization is accurate — this is not an over-read of an owner remark
into a document that says something narrower; the document itself says this.
(What ADR 0005 leaves genuinely open — items 3/4, the gate-Work submission
shape — is a separate matter, addressed under finding 2 above, not a defect
in R8 itself.)

### No finding — #85's sysinfo Disks API claim

**VERIFIED**: §4's crate table is explicitly framed as "researched via
Context7 against the crates' own documentation rather than from training
data" (preceding the table), distinct from the `[measured]` bullets that
follow it, and §10 explicitly defers macOS verification of `fs4`/`sysinfo`/
`directories`'s documented claims to the Mac trip ("the crates make the claim
cheap, they do not make it measured"). Honestly labeled as documentation, not
measurement, at both points where it matters.

### No finding — R4 vs. `AGENTS.md`'s guardrail against destroying preserved state

**VERIFIED**: `AGENTS.md:203-205` protects "a retained branch, a journal, a
Work record" from destruction under standing authorization. R4 ("retain the
dirty **state**, never the **directory**; `target/` is never in scope")
narrows retention to exactly that class and excludes gitignored build
artifacts — the same conclusion `LESSONS.md` L22 itself reaches ("what gets
preserved must be the thing the policy means, not the whole directory it
happens to live in," L22, lines 63-65). No conflict; R4 is L22's own fix.

### No finding — scope vs. `NORTH-STAR.md`'s gating

**VERIFIED**: none of the ten in-scope issues (#18, #81, #82, #85, #90, #94,
#96, #108, #109, #95) appear in `NORTH-STAR.md`'s "Gated" or "Never" lists
(lines 105-115). All ten are post-MVP defects surfaced by the 2026-08-14
cross-platform sprint, after North Star's own wave plan was superseded by the
MVP bucketing (line 62-64) and the MVP shipped (`GAUNTLET.md`, "MVP
CLOSE-OUT"). They postdate the wave plan entirely, so there is no gating
decision here for the sprint's scope to contradict.

**Summary — Spec: 5 findings (2 error, 2 warning, 1 info); 4 lines of attack
closed with no finding.** Worst issue on this axis: finding 2 (the `attach`
claim) — if uncorrected, W6 can pass its own gate without the integration
branch ever receiving the fixes it authorized, which is the exact failure
mode ADR 0005 built `attach` to prevent.
