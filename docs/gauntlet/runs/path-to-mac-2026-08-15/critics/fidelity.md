# Fidelity critic — PATH-TO-MAC-1

Blind seat, axis: fidelity. Grades the plan (`docs/gauntlet/runs/path-to-mac-2026-08-15/plan.md`
at `d86885f`) against every document, ADR, code location, and issue it cites.
Did not read the prior `code-review` Work's output or any `plan-review.md`.

## Method note

Read in full: the contract (`docs/gauntlet/contracts/PATH-TO-MAC-1.md`), the
plan (`plan.md`), `reference/notes/gauntlet-pattern.md`, `docs/DEVELOPMENT.md`,
ADR 0001, ADR 0002, ADR 0005, `docs/gauntlet/runs/cross-platform-2026-08-14/retrospective.md`,
`LESSONS.md` entries L3/L15/L19/L20/L22/L23, `GAUNTLET.md`'s deviation-register
header. Read the estate's `sergeant.toml` (found at `/home/miztertea/sergeant-rs/sergeant.toml`,
the estate root — one level above `repos/sergeant-rs`, the git checkout under
review). Read `src/runtime/surface.rs` around lines 320-442 and
`src/platform/disk.rs` around lines 1-105. Ran `git remote -v` / `git log`
in both `repos/sergeant-rs` and this surface's own checkout to check the §5
HEAD/clone claims. Fetched every named GitHub issue (`gh issue view <n>`) from
`/home/miztertea/sergeant-rs` (the estate root has a real GitHub origin;
`repos/sergeant-rs`'s origin is a local path back to the estate root and
`gh` cannot resolve it from there — noted as an environment fact, not a plan
defect) for #10, #12, #18, #81, #82, #85, #90, #94, #95, #96, #108, #109,
including #109's and #90's/#94's comment threads. Grepped the full repo for
the exact quoted string the plan attributes to ADR 0002 (D4) — see fidelity-F1
— and found it nowhere outside the plan itself, including ADR 0002's sole git
revision.

Not checked: §5's `directories`/`fs4`/`sysinfo` binary-size and compile-time
figures (contract explicitly excludes re-deriving §4's measurements); the
`gh` version-upgrade claim in §8 item 4 (no way to check a past `gh --version`
from this session); the live interview's actual content behind the nine
rulings in §2 (unrecorded anywhere in the repo — evidence for these is
definitionally unavailable to this session, so I did not attempt to "verify"
owner-ruling provenance and did not file findings against it per the
contract's non-goals).

## Findings

### fidelity-F1 — ADR 0002 (D4) is quoted for an objection it does not contain

**Severity: error.** The plan uses this misattribution to justify a concrete
deliverable (an ADR refresh assigned to a Work), and the quoted text does not
exist anywhere in the cited source or anywhere else in the repository.

**Plan text at issue** (plan.md §4, "The crate adoption, with measurements"):

> "**This retires ADR 0002 (D4)'s objection on its own terms.** That decision
> declined 'adding a libc-binding dependency for one syscall.' `libc` and
> `rustix` are already dependency edges this crate carries, and the count is
> now four facts rather than one. **An ADR refresh is owed**... Assigned to
> **W1**."

**The governing text it contradicts** — `docs/adr/0002-platform-boundary-shape.md`,
Decision D4 in full (lines 46-64):

> "**What is behind the boundary (D4).** Platform facts are in scope for this
> boundary. Docker's and the `claude` CLI's behavior are explicitly out of
> scope — those remain `Backend` capabilities with runtime withdrawal, which
> is existing machinery and not something this ADR changes. Docker Desktop on
> macOS has different bind-mount and uid semantics than Docker on Linux; the
> worktree-ownership fix already in this codebase... is itself a Unix concept
> — it does not obviously generalize to a boundary about platform facts.
> Whether `claude -p` behaves identically across hosts is a measure-first
> question the adapter owns... not something a platform module should assume
> on the adapter's behalf."

D4 is about *what falls inside the platform-facts boundary* (Docker and the
claude CLI are excluded; raw platform facts are included) — it says nothing
about declining a dependency, libc-binding or otherwise, for any syscall.
Verified in-session: `grep -rn "libc-binding dependency\|one syscall" .`
across the whole repo returns exactly one hit — the plan itself
(`plan.md:94`). `grep -n "libc\|syscall\|declin"` against ADR 0002's full text
returns only the unrelated phrase "the raw syscall itself is `cfg`-gated" in
D3. ADR 0002 has exactly one git revision
(`git log --oneline -- docs/adr/0002-platform-boundary-shape.md` → `8ee7b2b`
only), so there is no earlier wording this could be citing residually. ADR
0001's "Alternatives considered" section is the only other place `libc`
appears in the ADR corpus, and that occurrence is `libc::kill` inside a grep
pattern for signal-handling references — unrelated to a dependency-count
objection.

This is the exact failure mode the contract names by precedent (FOUNDATION-1's
refuted finding was a critic that invented a contract quote) — here the
artifact under review is the one doing the inventing, and it borrows ADR
0002's authority for an objection ADR 0002 never raised.

**What a correction would be:** either drop the "retires ADR 0002 (D4)'s
objection" framing and the invented quote entirely, or — if the intended point
is genuinely "this repo already carries `libc`/`rustix` as dependency edges,
so a new syscall-binding crate is a smaller marginal cost than it would have
been" — restate it as the plan's own argument rather than as something ADR
0002 said, and drop the quotation marks. Either way, "An ADR refresh is owed"
(assigned to W1) needs a correct citation before a Work can act on it; as
written, a Work opening ADR 0002 to find the objection it is meant to retire
will not find one.

**Verified vs believed:** Verified in-session (file reads + repo-wide grep +
git log on the ADR's own history).

---

### fidelity-F2 — R7's crate/config citation points to the wrong lines

**Severity: warning.** The fact cited is true; the line span is not where it
lives, so a Work resolving R7 by opening the cited lines finds unrelated
prose.

**Plan text at issue** (plan.md §2, ruling R7):

> "R7 | Sonnet for every dispatched Work | `--profile sonnet`, declared at
> `sergeant.toml:4-7`"

**What is actually at those lines** — `sergeant.toml` (estate root,
`/home/miztertea/sergeant-rs/sergeant.toml`), lines 4-7:

```
# §13's third precedence tier (WorkspaceDefault), which outranks the daemon's
# own hardcoded GlobalDefault of "fake" (src/daemon.rs:251). Set 2026-08-15:
# every Work in this estate is real work on a real harness, and `--profile
# sonnet` launches "claude", so leaving the default at "fake" made every
```

That is a comment explaining `default_backend`'s precedence, not the
`sonnet` profile declaration. The actual declaration is four lines later, at
11-14:

```
[[profile]]
name = "sonnet"
backend = "claude"
default_model = "sonnet"
```

Verified in-session: `find / -iname sergeant.toml` locates exactly one file
in this filesystem, read directly, line numbers confirmed by `Read`. Worth
noting for whoever picks this up: `sergeant.toml` is estate configuration
one level above `repos/sergeant-rs` (the git checkout `plan.md` otherwise
cites) — it is not part of the repository under review's own tree, though
that placement is not itself a defect, only relevant context for why a
Work materialized inside `repos/sergeant-rs` cannot `cat` this path without
knowing to look outside its own checkout.

**What a correction would be:** `sergeant.toml:11-14`.

**Verified vs believed:** Verified in-session (file read with line numbers).

---

### fidelity-F3 — R8's DEVELOPMENT.md line citation lands mid-sentence

**Severity: info.** The rule is real and the plan's characterization of it is
accurate; only the exact line pointer is imprecise enough that opening line 71
alone shows a sentence fragment, not the rule's start.

**Plan text at issue** (plan.md §2, ruling R8):

> "`docs/DEVELOPMENT.md` line 71 states the rule; **ADR 0005 (Accepted,
> 2026-08-14)** dissolved it..."

**The governing text** — `docs/DEVELOPMENT.md` lines 70-73:

> "70: - A workflow stage or actor executing inside a worktree never invokes
> 71:   `scripts/gate.sh`/no-mistakes itself — only the top-level orchestrating
> 72:   session owns a shipping-gate run, matching the single-owner posture the
> 73:   engine itself enforces on the data dir. <!-- BU-0041, BU-0122, BU-1196 -->"

Line 71 is the middle clause of a sentence that begins on line 70 ("A
workflow stage or actor executing inside a worktree never invokes...") and
ends on line 73. Line 71 alone does contain the load-bearing words ("never
invokes... itself"), so a reader who opens exactly that line is not misled
about content — but the contract's citation format calls for the line a
claim lives at, and the rule's sentence does not start there.

Separately, and this part is fully confirmed rather than a nit: the
substantive claim — that ADR 0005 dissolves this rule — is accurate. ADR
0005 D1 states verbatim: "The rule dissolves rather than being amended (D1)"
and closes with the exact quote the plan uses, "Gating becomes a dispatched
Work: Captain adjudicates findings; Sgt executes" (`docs/adr/0005-gating-becomes-a-dispatched-work.md:26,38-39`,
verified by direct read). The plan is also right that `docs/DEVELOPMENT.md`
itself is unchanged and still carries the dissolved rule's prose — this is
L20's shape, and L20's text ("a document says what we knew when it was
written; if it is wrong, supersede it and move on... Supersession fires on
contradiction") supports the plan's framing exactly.

**What a correction would be:** cite `docs/DEVELOPMENT.md:70-73` (the full
sentence), not `:71` alone.

**Verified vs believed:** Verified in-session (file read with line numbers,
cross-checked against ADR 0005's full text).

---

### fidelity-F4 — the L23 attribution for the compile-timing incident is a stretch, not a fabrication

**Severity: PLAUSIBLE (info if confirmed).** Cannot be confirmed or refuted
from artifacts available in this session; recorded per the "ambiguity fails
closed" rule rather than dropped.

**Plan text at issue** (plan.md §4, measurement paragraph):

> "Clean single-threaded compile of all five new crates: **3.90 s** (verified
> against a cached-rerun control at 0.02 s, because the first timing was a
> cache hit from a silently-failed `cargo clean` — **L23**)."

**The governing text** — `LESSONS.md` L23's own frame (lines 9-13, its table
of six/seven instances, and its stated generalization at line ~35): "Every
reduction of output lied at least once: read the artifact, not the view of
it... a view was narrowed and the narrowing, not the subject, produced the
answer." Every instance in L23's own table is a *filtered or truncated view*
of a command's output (an anchoring `grep`, a `tail -1`, a `head -3 && echo
OK`, a guessed JSON path, a resolved-vs-symlinked path) producing a false
conclusion from real output. L23 does also say, more broadly, "the two pipe
rows are the dangerous class, because the reduction ran on a *mutating*
command and discarded the channel that reported failure" — which is close in
spirit to "a silently-failed `cargo clean` produced a false timing," if (and
only if) the clean's own failure signal was itself discarded by a pipe or
`&&` rather than simply not checked. The plan does not say which.

I cannot confirm this because I was not present for the measurement session
and no artifact in the repo records the actual `cargo clean` invocation or
its exit code — this is a claim about what happened in a working session,
not about a document's content, so there is no file for me to open that
settles it either way. It is plausible on L23's broader "mutating command,
discarded failure channel" reading; it is not one of L23's own listed
instances, which are specifically about *narrowed views of output* rather
than *trusting an unchecked mutating command*.

**What a correction would be:** if the `cargo clean` failure really was
discarded via a pipe/`&&` (matching L23's "dangerous class" exactly), say so
explicitly ("cargo clean's own exit status was piped away") rather than
leaving the mechanism unstated; if it was simply not checked at all (no pipe
involved), this is arguably a fresh instance worth its own LESSONS row rather
than filed under L23, since L23's own text is specifically about *reductions
of output*, not about unchecked mutating commands in general.

**Verified vs believed:** Believed / unverifiable from available artifacts —
recorded as PLAUSIBLE per the method doc.

---

## What checked out (recorded per the contract: a truthful "this is sound" is a successful result)

The plan's other major citations were checked and confirmed accurate,
listed here so the adjudicator does not have to re-derive that a clean check
happened:

- **ADR 0001 (D8)**, plan §1's "measured" bar (probe-env.sh + `docs/environments/macbook.md`
  + full suite with published skip count) — verified verbatim against
  `docs/adr/0001-platform-targets-and-measurement-posture.md`'s D8 section.
- **`src/runtime/surface.rs:332` and `:431-441`**, plan §5's "a work branch is
  cut from the repository's current HEAD" — line 332 reads, verbatim, "work
  branch cut from that repository's current HEAD"; lines 431-441 show
  `base_sha` computed via `git rev-parse HEAD` and passed to `add_worktree`,
  supporting the claim exactly. Verified by direct read with line numbers.
- **ADR 0005**'s "Gating becomes a dispatched Work" quote and the "auto-fix /
  no-op / ask-user... unchanged" claim in plan §7 — both verified verbatim
  against `docs/adr/0005-gating-becomes-a-dispatched-work.md`.
- **The cross-platform retrospective**'s §2.2, §2.3, §2.4, and §4 citations
  in plan §6-7 (monitor-armed-late framing, ten-minute backgrounding rule,
  the two-Works-overrode-their-briefs claim, the backtick/command-substitution
  incident, and "anything that survives as a commit is a Work, with no
  exception for size") — all verified against
  `docs/gauntlet/runs/cross-platform-2026-08-14/retrospective.md`, matching
  in substance and mostly near-verbatim.
- **LESSONS L19 and L20** — plan's framing of both ("a governing document...
  takes the loop before it governs"; "stale-but-true... supersession fires on
  contradiction") verified against `LESSONS.md`'s actual entries.
- **The `code-review` workflow's two-axis shape** (plan §9: "Standards and
  Spec, reported side by side and never merged or reranked") — verified
  verbatim against `.sergeant/workflows/code-review/CONTEXT.md` and
  `40-aggregate/CONTEXT.md`.
- **`src/platform/disk.rs:71`**, plan §4's "`df -k --output=avail` shell-out
  (GNU-only)" — verified verbatim (`.args(["-k", "--output=avail"])`).
- **Every named GitHub issue** (#10, #12, #18, #81, #82, #85, #90, #94, #95,
  #96, #108, #109) exists, is open, and its title/body matches the plan's
  characterization of what it covers. #85's "macOS detection is unmeasured...
  `/proc/mounts` is Linux-only" is verified verbatim in the issue body,
  supporting plan §2's R5 ruling and §4's "Replaces: `/proc/mounts` parsing
  (Linux-only)" row. #90 and #94's issue bodies verified verbatim against
  plan §6's and §7's characterizations of both.
- **ADR 0005's "Partially implemented" consequences section** (branch-takeover
  mechanism, items 3/4 still open) was read in full but the plan does not cite
  it beyond the D1 quote already checked above — no plan claim rests on it,
  so no finding either way.
