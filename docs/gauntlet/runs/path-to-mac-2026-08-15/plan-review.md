# Path-to-Mac sprint plan — review (two-axis code-review pass)

**Fixed point:** `main` (`242abe3`) → `d86885f`, one file:
`docs/gauntlet/runs/path-to-mac-2026-08-15/plan.md`, 221 insertions.

**This supersedes commit `3de1266`'s review of the same file, and says so
rather than silently overwriting it.** That review's own text was correct in
structure (Standards/Spec, evidence-vs-belief labeling, four lines of attack
closed) but **finding 1's verification claim was itself false**: it asserted
`grep -rn "libc-binding\|one syscall" docs/ reference/ .` returns "exactly one
hit — the plan's own line," and concluded the objection "does not exist... in
any form." Re-running that exact command this session returns two more hits —
`src/platform/disk.rs:5` and `src/cli.rs:1531` — because `.` as a grep path
argument covers the whole repo, `src/` included, and the prior review's own
narrative ("No ADR, ledger entry, or note contains this objection") did not
match its own command's actual output. The objection is real; it just isn't
where the plan says it is. Full analysis under Spec finding 1 below. This is
the same class of failure LESSONS L19 exists to catch, applied one level up —
a review is itself executable through the program that trusts it, so a review
that ships an unverified verification is exactly as dangerous as an unreviewed
plan.

**Scope note.** This is the narrower `code-review` pass named in
`docs/gauntlet/contracts/PATH-TO-MAC-1.md` (that contract lives on
`integration/path-to-mac-2026-08-15`, dispatched separately) — input to that
unit's own four-axis panel, not a substitute for it.

**Method.** Standards and Spec were run as two genuinely parallel,
context-isolated sub-agents (neither given sight of the other, or of the prior
`3de1266` review — both were explicitly instructed not to open it), per the
reviewed skill's own design
(`reference/sergeant-upstream/.agents/skills/code-review/SKILL.md`). Findings
below are the sub-agents' verified output, cross-checked directly against the
cited files by the orchestrating pass before being written up here (the
`sergeant.toml`, ADR 0005 quote-wording, and #108 retrospective-section checks
were re-run independently rather than taken on the sub-agent's word). Every
finding distinguishes **VERIFIED** (checked in-session against the cited
file/line or a grep run this session) from **BELIEVE** (reasoned, not
directly checked), per L15. The nine owner rulings in plan §2 are not
re-litigated; findings are about the plan's stated *justification* for a
ruling, or about text outside §2.

---

## Standards

Checked against `docs/DEVELOPMENT.md` and `LESSONS.md`'s own meta-rules for
how a governing artifact in this repo cites evidence — not Fowler's smell
baseline, which targets code and doesn't transfer to sprint-planning prose.

**No hard violations of the symlink-citation rule.** VERIFIED: `grep -n
"CLAUDE.md" plan.md` returns nothing — the plan never cites the symlink path;
every `docs/DEVELOPMENT.md` reference names the real file.

### 1. [warning] The `[measured]` tag on §5's `surface.rs` citation is mislabeled by the plan's own stated convention

**Plan text (line 111):** *"**[measured]** `src/runtime/surface.rs:332` and
`431-441`: a work branch is cut from the repository's **current HEAD**."*

**Plan's own rule (line 10-12, the preamble):** *"Where a claim is a
measurement taken this session, it is marked **[measured]**; where it is read
from a repo artifact, the artifact is cited."*

Reading `surface.rs:332`'s doc comment and the `431-441` code range is exactly
"read from a repo artifact" — no runtime behavior was observed or timed this
session for this claim; the plan's own paragraph immediately after (line
112-113) is the actual observation ("`repos/sergeant-rs` is a separate clone
... clean on `main` at `242abe3`"), and that part legitimately could carry the
tag. Tagging the source-read half `[measured]` blurs the plan's own
declared distinction between "I observed this" and "I read this," which is
the exact distinction L15 exists to preserve elsewhere in the same document
(§4's careful separation of measured binary-size deltas from the unmeasured
`sysinfo` runtime cost is the standard this line falls short of).

**Correction:** drop the `[measured]` tag from the `surface.rs` citation
itself, or split the sentence so the tag attaches only to the HEAD-of-clone
observation.

### 2. [info] R7's `sergeant.toml:4-7` citation is imprecise

**Plan text (line 44):** *"R7 | Sonnet for every dispatched Work |
`--profile sonnet`, declared at `sergeant.toml:4-7`."*

VERIFIED: `sergeant.toml` lives at the estate root
(`/home/miztertea/sergeant-rs/sergeant.toml`), not inside this repo checkout.
Lines 4-7 are a comment explaining why `default_backend = "claude"` was set,
which mentions the phrase "`--profile sonnet`" in passing. The actual
declaration — `[[profile]] name = "sonnet"` — is at lines 10-13. The cited
range contains the phrase but not the declaration it's attributed to.

**Correction:** cite `sergeant.toml:10-13` (the `[[profile]]` block) as where
the profile is declared; lines 4-7 can still be cited separately as the
rationale for `default_backend`.

### 3. [info] §8's `"this Work reviews that Work"` is a paraphrase presented as a quotation

**Plan text (line 185):** *"a submission shape for `"this Work reviews that
Work"` — remain unbuilt..."*

VERIFIED: ADR 0005 (`docs/adr/0005-gating-becomes-a-dispatched-work.md:110`)
reads *"a submission shape for 'this Work reviews `<target-work-id>`'"* —
different wording, inside quotation marks in both documents. The plan's
substitution of "that Work" for the literal `<target-work-id>` placeholder
changes a citation from a quote into a paraphrase without dropping the quote
marks.

**Correction:** either quote ADR 0005 verbatim or drop the quotation marks
around the paraphrase.

**Summary — Standards: 3 findings (0 error, 1 warning, 2 info).** All three
are citation-precision issues, none change what a Work would do differently;
contrast with Spec finding 1 below, which is the same *class* of defect
(misattributing a claim to a source that doesn't say it) but with real
downstream consequence because it assigns a Work to correct the wrong
document.

---

## Spec

Checked against `NORTH-STAR.md`, ADR 0001/0002/0005, `docs/DEVELOPMENT.md`,
`LESSONS.md`, `GAUNTLET.md`'s deviation register and FOUNDATION-1 entry, and
`AGENTS.md` — plus, where a claim is about the codebase itself, the cited
source directly (`src/runtime/surface.rs`, `src/platform/disk.rs`,
`src/backend/docker.rs`'s history, `tests/support/mod.rs`).

### 1. [error] §4's "ADR 0002 (D4)" citation names the wrong document — the objection is real but lives in code, not the ADR

**Plan text (lines 93-95):** *"This retires ADR 0002 (D4)'s objection on its
own terms. That decision declined 'adding a libc-binding dependency for one
syscall.'"*

**Governing text it contradicts:** `docs/adr/0002-platform-boundary-shape.md`,
decision D4 ("What is behind the boundary," lines 46-56), is entirely about
scoping Docker/`claude` CLI behavior out of the platform-fact boundary. It
says nothing about libc, syscalls, or dependency cost.

**VERIFIED**, corrected from the prior review's version of this finding: `grep
-rn "libc-binding\|one syscall"` across the *whole* repository (not just
`docs/` and `reference/`) surfaces the real source —
`src/platform/disk.rs:1-8`'s module doc comment: *"`df` remains the
mechanism — not a `libc`/`statvfs` binding. The module this fact used to live
in (`src/backend/docker.rs`) explicitly declined that binding 'for one
syscall' in favor of the same shell-out posture the rest of this crate already
takes."* This is a real, specific, previously-made engineering decision about
`#81` — it is simply not in ADR 0002, and never was.

**Why this matters:** the paragraph uses this citation to justify "An ADR
refresh is owed... Assigned to **W1**" (lines 96-98). If W1 goes looking in
ADR 0002 for the objection it's meant to retire, it won't find it there — the
actual decision to revisit lives in `src/platform/disk.rs`'s doc comment
(originally `src/backend/docker.rs`), which is source code, not an ADR at all.
"Retiring ADR 0002 (D4)" is a category error: D4 was never an objection to a
libc-binding dependency, so nothing about D4 is retired by adding
`fs4`/`rustix`. The actual question — does the `disk.rs`-documented tradeoff
still hold given `#81`'s measured GNU-`--output` portability failure — goes
unaddressed by a paragraph that thinks it already answered it.

**Correction:** cite `src/platform/disk.rs`'s doc comment (and, if traceable,
whatever decision record — if any — the `src/backend/docker.rs` predecessor
comment pointed to) as the source of the "one syscall" objection, not "ADR
0002 (D4)." If no ADR ever recorded this, say so explicitly rather than
implying one did.

### 2. [error] §8 risk 3's "no attach needed" claim reproduces, not sidesteps, FOUNDATION-1's finding

**Plan text (lines 184-188):** *"Review is Captain-serial. ADR 0005's items 3
and 4... remain unbuilt, so a gate Work cannot be bound to another Work's
branch. This sprint sidesteps it: W6 cuts from the integration tip and
therefore already holds the content, never needing `surface::attach`. That
works here and does not generalize."*

**Governing text it contradicts:** `docs/adr/0005-gating-becomes-a-dispatched-work.md`'s
Consequences section built `surface::attach` specifically so *"a fix commit
made in the resulting worktree lands on the branch that will actually
ship."* `GAUNTLET.md`'s FOUNDATION-1 entry names the exact failure mode this
fixes: reviewing a copy breaks no-mistakes' recovery of auto-fix commits onto
the shipped branch, yielding "a gate Work that passes its own copy while the
actual shipping branch never received the pipeline's fixes."

**VERIFIED** against `src/runtime/surface.rs`: `materialize()` (doc comment,
line 332) always cuts a *fresh* `sergeant/<work-id>` branch from current
HEAD — including when W6 cuts from the integration tip. `attach()` (line 561)
is the only path that checks a surface's worktree onto an *existing* branch
instead of minting one. Plan §5's wave table has W6 cut "from: integration
tip" via ordinary `materialize()`, not `attach()`.

Plan §7 explicitly grants W6 auto-fix authority ("The Work may authorize
`auto-fix`"), and §7/R9 name the persistent `integration/…` branch with an
early-opened head PR as what "the owner merges." If W6's gate run authorizes
an auto-fix commit, that commit lands on W6's own freshly-minted branch — not
on the integration branch with the open PR. "Already holds the content"
answers the *review-reads-correct-content* question `attach` was never needed
for; it does not answer the *auto-fix write-back* question `attach` was built
for. Cutting from the tip is a fix for staleness, not for branch identity —
the two are different axes, and the plan conflates them.

**Correction:** either W6 attaches to the integration branch itself
(reintroducing the unbuilt-mechanism problem this risk already names), or the
plan needs an explicit reconciliation step — fast-forward/merge W6's branch
onto integration before "the owner merges" — or W6 is restricted to
`no-op`/`ask-user` findings only, with `auto-fix` withheld until attach is
wired up for this shape. Left as written, W6 can pass its own gate while
shipping nothing back to the branch under review — the sprint's final wave
does not work as specified.

### 3. [error] W1/W3 file-disjointness claim is contradicted by the plan's own §4 table

**Plan text (lines 129-130):** *"W1 and W3 are the only parallel pair; their
file sets are disjoint (`src/` + `Cargo.toml` vs `tests/` + `scripts/perf/`)."*

**Governing text it contradicts:** the plan's own §4 crate table (line 77)
names `tests/support/mod.rs` as one of three call sites `#18` (W1's scope)
replaces. `docs/DEVELOPMENT.md:47` describes that file's `DataDir` guard as
reaping "by `/proc` argv scan on Drop." The cross-platform retrospective's own
recommendation for `#108` (W3's scope), §1.3/§3.3.3, is explicit: *"A
start-of-run reaper, not another `Drop` guard... Extending `DataDir`'s
`/proc`-scan reaping to run at suite start... closes the class."* **VERIFIED**:
that is the same file W1 is already documented as editing.

Both W1 (`#18`, replacing `/proc` reads in `tests/support/mod.rs`) and W3
(`#108`, plausibly extending that same file's reaping to start-of-run) have a
plan-documented reason to touch the same file — directly contradicting the
"disjoint" claim used to justify running them in parallel. This is
load-bearing, not cosmetic: the plan's stated reason waves exist at all is
"file ownership, not logic... Three Works editing `Cargo.toml` would conflict
by construction" (§5 opening) — W1/W3 risk exactly the class of conflict the
wave structure exists to prevent. (Rated error rather than warning: unlike
finding 4 below, this isn't a citation mismatch with no operational
consequence — a real concurrent edit to the same file is the specific outcome
the plan says waves were designed to avoid. It is a less severe error than
finding 2, though: a same-file conflict is a recoverable merge/rebase, not a
silently stranded commit.)

**Correction:** name the actual target regions of `tests/support/mod.rs` for
`#18`'s edit and `#108`'s reaper before dispatch; if they collide, resequence
W1/W3 rather than running them in parallel.

### 4. [warning] R6's stated cause for #108 is borrowed from a different retrospective row

**Plan text (line 43, R6):** *"#108 is fixed by a **start-of-run reaper**,
using standard patterns | `Drop` does not survive `SIGKILL`, and SIGKILL is
how these die."*

**Governing text it contradicts:** `docs/gauntlet/runs/cross-platform-2026-08-14/retrospective.md`
§1.2 (lines 77-86) — the section that names `#108` and generalizes it as
`LESSONS.md` L21 — attributes the leak entirely to the dead-man test's
premise: *"The dead-man test — whose premise is that the release path never
appears — removes nothing... the abnormal path is the least likely to carry
cleanup."* **VERIFIED**: neither that section nor L21 itself mentions SIGKILL
or `Drop` anywhere. The "`Drop` does not survive `SIGKILL`" reasoning is
§1.3/§3.3.3 (lines 90-100, 319-322) — explicitly about a *different* row, the
1.7 GB `/var/tmp/sgt-rs-tests` rig leak generalized from `#91`, not `#108`.

**VERIFIED, not merely believed** (upgraded from the prior review's BELIEVE
label after re-reading both sections directly): a start-of-run reaper is
still a defensible fix for `#108` on independent grounds (it would catch the
dead-man test's leaked marker regardless of cause), but R6's given rationale
is the wrong row's reasoning attached to the right row's fix.

**Correction:** state `#108`'s actual cause (test premise, not process
death), or keep the SIGKILL/Drop reasoning attached to the `#91`-derived
residue item it actually describes and give R6 its own correct rationale.

### No finding — §6/§8's ceiling-size ("mitigated only by envelope sizing") framing

Reconsidered against the prior review's info-level critique of this same
text. **VERIFIED**: plan lines 136-140 state plainly, in the same breath as
the claim, that "a genuinely stuck turn burns 90 minutes before it surfaces"
and that fewer firings is "the only lever available tonight"; §8 risk 1 says
"Mitigated **only** by envelope sizing" — the word "only" is doing honest
work, not overclaiming. The plan does not claim `#90`'s defect is fixed, and
discloses the limitation adjacent to the claim rather than separately from
it. No finding.

### No finding — R8 (ADR 0005 supersedes `DEVELOPMENT.md:71`)

**VERIFIED**: `docs/DEVELOPMENT.md:69-72`'s session-conduct bullet is quoted
near-verbatim (identical `BU-0041, BU-0122, BU-1196` citations) in ADR 0005's
own Context section, and ADR 0005's Decision states plainly: "The rule
dissolves rather than being amended (D1)." Direct, on-point supersession, not
an over-read of a narrower document.

### No finding — #85's sysinfo Disks API claim

**VERIFIED**: §4's crate table is explicitly framed as "researched via
Context7 against the crates' own documentation," distinct from the
`[measured]` bullets that follow, and §10 defers macOS verification to the
Mac trip ("the crates make the claim cheap, they do not make it measured").
Honestly labeled at both points where it matters.

### No finding — R4 vs. `AGENTS.md`'s guardrail against destroying preserved state

**VERIFIED**: `AGENTS.md:203-205` protects "a retained branch, a journal, a
Work record" from destruction under standing authorization. R4 ("retain the
dirty **state**, never the **directory**; `target/` is never in scope")
narrows retention to exactly that class and excludes only gitignored build
artifacts — the retrospective's own #109 analysis reaches the same
conclusion. No conflict.

### No finding — scope vs. `NORTH-STAR.md`'s gating

**VERIFIED**: `grep` for all ten in-scope issue numbers (`#18, #81, #82, #85,
#90, #94, #96, #108, #109, #95`) against `NORTH-STAR.md` returns zero matches.
**BELIEVE** (this checkout's `sergeant-rs` origin has no GitHub remote, so
issue titles can't be independently cross-checked here): by category, none of
the ten — platform crates, engine honesty, operator surfaces, harness
hygiene — obviously fall under NORTH-STAR's "Gated" or "Never" lists. They
postdate the wave plan the North Star superseded with MVP bucketing.

**Summary — Spec: 4 findings (3 error, 1 warning); 4 lines of attack closed
with no finding.** Worst issue: finding 2 (the `attach` claim) — if
uncorrected, W6 can pass its own gate without the integration branch ever
receiving the fixes it authorized, the exact failure mode ADR 0005 built
`attach` to prevent. Finding 1 is a correction to the *prior* review of this
plan, not a new discovery this session invented — the underlying citation
defect was already flagged in `3de1266`, but that review's own verification
step was wrong about where the true source lives, which would have sent W1
looking in the wrong document.
