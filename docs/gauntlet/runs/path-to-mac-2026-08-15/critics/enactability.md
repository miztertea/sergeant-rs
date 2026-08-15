# PATH-TO-MAC-1 — enactability critic

## Method note

Read, in order: `docs/gauntlet/contracts/PATH-TO-MAC-1.md`; the plan artifact
at `docs/gauntlet/runs/path-to-mac-2026-08-15/plan.md` pinned to commit
`d86885f` (checked out via `git show d86885f:...` into `/var/tmp/`, not read
off the working tree, so line numbers below are the graded commit's, not
whatever HEAD is now); `reference/notes/gauntlet-pattern.md`; `docs/DEVELOPMENT.md`;
`GAUNTLET.md`'s deviation register and backlog (no path-to-mac items
pre-registered — nothing here re-litigates a settled deviation); ADR 0001,
0002, 0005; `docs/gauntlet/runs/foundation-1/critics/enactability.md` (this
axis's own precedent, for format and severity calibration); the
cross-platform-2026-08-14 `plan.md`/`close-out.md`/`lessons.md`/`retrospective.md`
and `LESSONS.md` L20/L21/L23 for the prior-sprint context the plan's rulings
draw on.

I did **not** read `plan-review.md` or Work `01M01XGMY8JJ4M4RJDN1RSZXJC`'s
output, per the blind-panel instruction, and did not edit the plan or any
other file under review.

**What I ran**, all read-only, no mutation, nothing in a worktree needed
(no code changes are proposed by this axis, so no disposable-worktree probe
applied):

- `git remote -v` / `git config --get remote.origin.url` in this checkout —
  confirms `origin` is the local path `/home/miztertea/sergeant-rs`, not a
  GitHub host.
- `gh issue view 90` (no `--repo`) — reproduces the exact failure a
  dispatched Work would hit.
- `gh auth status` — confirms `gh` **is** authenticated (`miztertea`
  account), so the failure above is a repo-resolution gap, not an auth gap.
- `gh issue view <N> --repo miztertea/sergeant-rs [--comments]` for #90,
  #94, #95, #96, #108, #109, #12, #10 — the real issue bodies, used to check
  whether each Work's scope line in the plan is actually buildable against
  what its cited issue says.
- `gh issue list --repo miztertea/sergeant-rs --state all --limit 200` —
  confirms every issue number the plan cites exists and is `OPEN` where the
  plan implies it is.
- `grep -n` against `src/runtime/surface.rs` to check the §5 mechanism
  citation (`surface_root` / `materialize`'s branch-cut-from-HEAD claim).
- `find` / `sed` to confirm `sergeant.toml` is gitignored, out-of-tree-per-worktree
  config, not a repo source file (relevant only as background — the R7
  citation's line accuracy is a fidelity-axis question, not graded here).

**What I could not check**: whether the live `grilling` interview that
produced this plan carries context beyond what is committed anywhere in this
repo (e.g. a fuller description of #96 or #90's intended fix that only the
owner and the orchestrating session hold). Where a finding below turns on
"nothing in the accessible corpus says X," that is a claim about the
repository's *written* record, verified by direct search, not a claim about
what the owner or orchestrator privately knows.

Severity key, carried from FOUNDATION-1's enactability axis: **error** = a
dispatched Work would stall or produce an unreviewable guess; **warning** =
a Work could proceed but would have to invent something the plan should have
supplied; **info** = worth recording, doesn't block dispatch.

---

## enactability-F1 — §5's #95 scope line contradicts §3/§10's own statement of what the Mac exclusively owns

**Severity argued:** error — this is not an underspecified detail, it is the
same document asserting a decision is both made-here and reserved-elsewhere.

**The exact plan text at issue:**
- §3 (line 59): `**In:** #18, #81, #82, #85 (platform crates) · #90, #94
  (engine honesty) · #96, #109 (operator surfaces) · #95, #108 (harness
  hygiene).`
- §3 (lines 62–66): `**Out, and deliberately so:** nothing from the
  portability set is deferred to the Mac any more. What the Mac still owns
  is **verification**... plus `#95`'s clock choice, which needs timing on
  real hardware to pick between `perl -MTime::HiRes`, `python3
  time.time_ns()`, and accepting millisecond resolution.`
- §5 (line 123): `| 1 ‖ | **W3 · harness hygiene** | #108 (start-of-run
  reaper), #95 | `main` |`
- §10 (line 219): `- Choosing #95's clock on real hardware.` (listed under
  "What the Mac still owns")

**The governing text it contradicts:** the plan contradicts itself — §3's
own "In" list and §5's Work-scope table place #95 inside W3's Cerberus-wave-1
deliverable, while §3's own "Out" paragraph and §10 both state, in the same
document, that the clock choice — #95's entire substance — is something only
the Mac can settle. I also read #95's own issue body
(`gh issue view 95 --repo miztertea/sergeant-rs`): its "Scope" section
frames the clock pick and its implementation as one task — "A
nanosecond-resolution clock that works on macOS bash 3.2... Whichever path,
it should fail loudly... Verify on the MacBook before closing" — with no
internal split between a Cerberus-buildable part and a Mac-only part.

**What I found:** a Work dispatched to W3 with the scope line "#108
(start-of-run reaper), #95" has two contradictory readings available in the
same plan and no way to choose between them: (a) build and ship the actual
clock choice now, which directly contradicts §3/§10's explicit statement
that this decision needs real-hardware timing only the Mac can produce, or
(b) do nothing substantive under "#95," which contradicts #95 being listed
in §3's "In" set and in §5's scope table at all. Nothing in §5, §6, or the
issue body itself narrows "#95" to a Cerberus-buildable subset (e.g. "wire
the fail-loudly guard and a placeholder, leave the actual pick open").

**What a correction would be:** name the split explicitly — e.g. "W3 builds
the fail-loudly guard around whichever clock is chosen and a millisecond-resolution
fallback path; the clock pick itself is an Unknown, resolved on the Mac
per §10" — or remove #95 from §5's W3 scope line entirely and leave it
solely under §10, which is where §3's own "Out" language already puts it.

**Verified vs. believed:** verified — all four quoted passages read
directly from the pinned commit; #95's issue body read directly via `gh`.
The contradiction is between the plan's own sentences, not a claim requiring
outside context.

---

## enactability-F2 — W2's #90 half has no stated target shape; its #94 half does

**Severity argued:** warning — a Work could still proceed (the issue names
the symptom and the invariant it must restore), but it has to invent the
actual state-machine fix the plan never names, on a topic the plan itself
flags as the sprint's own named risk.

**The exact plan text at issue:** §5 (line 124): `| 2 | **W2 · engine
honesty** | #90, #94 | wave-1 tip |`. §8 (line 182): `1. **#90 is unprotected
tonight.** Mitigated only by envelope sizing (§6).`

**The governing text it contradicts:** not a contradiction — a scope gap,
checked against the two issues' own bodies (`gh issue view 90` and
`gh issue view 94 --repo miztertea/sergeant-rs --comments`). #94 carries a
"Suggested shape" a Work can build directly against: "A closing stage that
declares a commit as its outcome should have that checked: if
`finalize_commit` equals the base sha and the worktree is dirty, the Work
has not met its contract and should not land in plain `completed`... surface
it where operators actually look." #90 has no equivalent section. Its body
ends on the corrected diagnosis: "after a ceiling interrupt the Work sits in
`active`, `retry` refuses it with a 409, `extend` is inert without a retry
to gate, and the only reachable verb is `cancel`. The Work never lands
anywhere an operator can resume it... and remains the thing to fix" — a
restated symptom and invariant, not a target mechanism. This also touches
`docs/DEVELOPMENT.md:40`: "ambiguity fails closed into `blocked` with a
reason, never a guess" — the invariant #90's fix must restore, but not a
specification of how.

**What I found:** the plan gives R4 an explicit ruling for #109 and R6 an
explicit ruling for #108, but supplies no equivalent ruling for #90 despite
naming it, in the very next section, as the sprint's own unprotected risk.
A Work executing W2 has to design a new landing state (or repurpose
`blocked`) and decide what `retry`/`extend` do differently for a
ceiling-interrupted turn — a decision that touches the same fail-closed
state-machine invariant `docs/DEVELOPMENT.md:40` states for the whole
engine, with no proposed shape and no pointer to the one plausible precedent
in the repo (`R-MVP1-10`'s `extend_turn_envelope` exit door for the
*envelope-exhausted* `blocked` landing — a different trigger than a ceiling
interrupt, but the nearest existing analogue) which the plan doesn't cite
either.

**What a correction would be:** either state #90's target landing behavior
explicitly (e.g., "a ceiling-interrupted turn lands in `blocked` with a
named reason, by analogy to R-MVP1-10's exit door" — if that is the intended
shape) or name it explicitly as an Unknown W2 must propose and flag rather
than resolve silently under a single Sonnet turn.

**Verified vs. believed:** verified — both issue bodies read in full via
`gh`; the DEVELOPMENT.md citation read directly; the R-MVP1-10 precedent
confirmed via `grep` against `GAUNTLET.md`/`docs/gauntlet/runs/mvp4-soak*/run-manifest.md`
existing and describing an exit door for a different (envelope-exhausted,
not ceiling-interrupted) trigger.

---

## enactability-F3 — §4's sysinfo-latency acceptance criterion names an action but not a trigger threshold

**Severity argued:** warning — the plan does supply a directional rule
("does **not** license a Linux/macOS split — it licenses raising the
finding"), so a Work is not stalled; but nothing ties "raise a finding" to
a number, despite two numbered issues (#12, #10) sitting right next to the
claim with numbers of their own.

**The exact plan text at issue:** §4 (lines 100–104): "**Not measured, and
it is the number that matters for #18:** `sysinfo`'s *runtime* cost for a
single-pid refresh versus the current `/proc` read. Binary size is not the
risk; #12 (doctor's ~450 ms floor) and #10 (cold-call latency scaling) are.
W1's acceptance requires measuring it. Under R2, a slower result does
**not** license a Linux/macOS split — it licenses raising the finding."

**The governing text it contradicts:** this is exactly the pattern the
contract's own axis brief names as the target: "'Measure X and raise a
finding if it is slow' — is there a threshold, or is it a judgment call the
plan has not made?" (`docs/gauntlet/contracts/PATH-TO-MAC-1.md`, enactability
axis section). I read the two cited issues directly: #12's own title is "`sgt
doctor` has a fixed ~450 ms floor... load-independent"; #10's is
"`blocked_time_per_work` cold-call latency scales with journal size
(153→792 ms at 10k→50k events)". Both carry numbers the new measurement
could be compared against, and neither is referenced as the comparison
basis.

**What I found:** "a slower result... licenses raising the finding" is
readable two ways — either any measurable slowdown is meant to be raised
regardless of magnitude (a defensible zero-threshold policy, given #12/#10
are already flagged as fragile budgets), or some materiality bar is implied
and left to the Work's judgment. The plan doesn't say which, so a Work has
no way to know whether a 2% single-pid refresh slowdown and a 40% one get
the same response.

**What a correction would be:** either state explicitly that any measured
slowdown is to be raised regardless of size (making the zero-threshold
reading the ruled one), or name a comparison basis (e.g. "raise a finding if
the added latency measurably narrows #12's ~450 ms floor or #10's growth
curve").

**Verified vs. believed:** verified — plan text, contract text, and both
issue titles read directly.

---

## enactability-F4 — #109's plan scope (R4 only) may be narrower than what the issue itself now defines as the top-level deliverable

**Severity argued:** warning — R4 is a legitimate, in-scope owner ruling on
one sub-question; the gap is whether the plan silently narrows #109 to that
sub-question without saying so.

**The exact plan text at issue:** §2 (line 41): `| R4 | #109: retain the
dirty **state**, never the **directory**; `target/` is never in scope |
*"The journal is the real artifact. Let's keep the disk clean."* Turns 30 GB
into megabytes |`. §5 (line 125): `| 3 | **W4 · operator surfaces** | #109
(R4), #96 | wave-2 tip |`.

**The governing text it contradicts:** #109's own latest comment (read via
`gh issue view 109 --repo miztertea/sergeant-rs --comments`), an explicit
"Owner ruling — this is a contract gap, not hygiene" reframe: "the top-level
item [is] **write the disk-footprint contract**, with the verbs in the
original description as its enforcement surface rather than as the fix...
[it covers] what a Work may write, and where... who owns each artifact at
each Work state... what teardown retains, at what granularity, and for how
long... who may reclaim it, through which verb... Retention scope
[R4's question]... remains the decision that determines whether this is a
hygiene feature or a design correction; **under the contract framing it is
the contract's central clause**" (emphasis mine). The issue's own text
subordinates R4's question to a larger, still-unwritten deliverable.

**What I found:** §5 reads as though R4 settles #109 for W4's purposes. But
#109's own current, owner-authored text says retention scope is one clause
of a larger contract-writing task the plan never mentions (an inspection
verb, a fail-closed disposal verb, and the written contract itself). This is
not a re-litigation of R4 — R4's answer to the retention-scope question
stands. It is a question of whether W4's scope, as stated, covers what #109
now asks for, or deliberately narrows it, and the plan doesn't say which.

**What a correction would be:** state explicitly whether W4's #109 scope is
"R4's retention-scope ruling only" (a legitimate narrowing, but say so, and
note #109 will not fully close on W4's output) or "R4 plus the
disk-footprint contract #109 now asks for" (in which case W4's brief needs
that scope named, not just R4's clause).

**Verified vs. believed:** verified — #109's full comment thread read
directly via `gh`; the "In" claim R4 answers versus the broader ask the
issue's own later comment states are both quoted above from primary text,
not summarized.

---

## enactability-F5 — every issue-numbered scope item is unreachable via the natural first move (`gh issue view <N>`) in this checkout; §6's per-brief checklist doesn't carry the fix forward

**Severity argued:** warning — recoverable within a Work's own turn budget
once discovered, but the failure mode is actively misleading, and every one
of W1–W5's scope lines depends on resolving at least one bare issue number.

**The exact plan text at issue:** §3 states scope entirely as bare issue
numbers (#18, #81, #82, #85, #90, #94, #96, #109, #95, #108 — ten numbers,
zero inline descriptions). §6 (lines 150–161) states what "every brief
carries": prior art named as settled, the commit-trailer choice, the *why*,
a pointer at the deviation register, evidence-vs-hypothesis labels, the
`PATH="$HOME/.cargo/bin:$PATH"` fact, and the foreground-execution rule —
six specific environment/process facts, but not this one.

**The governing text it contradicts:** ADR 0001's own open questions,
written the day before this plan: "This ADR could not independently confirm
issue #81's tracked title or body against GitHub (this checkout's `origin`
remote is a local path, not a GitHub host, so `gh issue view` cannot resolve
it here)." The same fact holds today — I reproduced it directly: `git
remote -v` shows `origin` as `/home/miztertea/sergeant-rs`; `gh issue view
90` (no flag) fails with "none of the git remotes configured for this
repository point to a known GitHub host. To tell gh about a new GitHub
host, please use `gh auth login`" — a misdirecting remedy, since `gh auth
status` confirms the account (`miztertea`) is already authenticated; the
actual fix is `gh issue view 90 --repo miztertea/sergeant-rs`, which I
confirmed succeeds and returns the real issue body.

**What I found:** the plan already knows to carry forward several
checkout-specific gotchas into every brief (PATH, `/tmp` tmpfs, foreground
execution) but not this one, even though it is the same class of fact (a
known, previously-documented checkout quirk that produces a misleading
error) and it blocks something every single Work needs to do at least once
— read what its own scope numbers mean.

**What a correction would be:** add `gh issue view <N> --repo
miztertea/sergeant-rs` (or `export GH_REPO=miztertea/sergeant-rs` once, in
the brief) to §6's per-brief checklist, next to the PATH line it already
carries.

**Verified vs. believed:** verified — every command above run directly in
this session; the ADR 0001 citation read directly at the quoted line.

---

## enactability-F6 — §7 names one Work-level failure mode (ceiling wedge) and mitigates it, but is silent on a wave-1 Work landing in `failed`

**Severity argued:** warning — this is an orchestrator-duty gap rather than
a specific dispatched Work being unable to act, so it doesn't block any one
Work's own turn, but it leaves the sprint's own sequencing mechanism
undefined on a non-happy path.

**The exact plan text at issue:** §7 (lines 163–178) lists four orchestrator
duties — watcher arming, `finalize_commit`-based completion verification,
the gate-finding authority split, and no-merge-to-`main` — with no mention
of a Work reaching state `failed`. §5's mechanism paragraph (lines
111–118) states only the success case: "Advancing *its* HEAD to the
integration tip between waves means every later Work cuts from the previous
wave's result."

**The governing text it contradicts:** not a contradiction with a specific
governing document — a gap the axis brief asks me to check directly ("Are
the failure paths named? ... Is there a stated response, or does the sprint
assume the happy path?"). I checked `GAUNTLET.md` and the
cross-platform-2026-08-14 run's own retrospective/lessons/close-out for a
standing convention on wave-Work failure this plan could be assumed to
inherit silently; none of them state one.

**What I found:** §8 names and mitigates exactly one failure mode — the
ceiling-interrupt wedge (#90) — because that is the mode the sprint's own
generous envelope sizing (§6) is a direct response to. It says nothing about
the more mundane case: a wave-1 Work (W1 or W3) genuinely finishing its
turns and landing in `failed` (gates red, or the actor stops short) without
a commit. Since W2 is stated to cut from "wave-1 tip," and wave-1 is W1‖W3
run in parallel, a `failed` wave-1 Work leaves at least three live
readings unaddressed: does the wave block and wait for a retry, does the
orchestrator re-dispatch against the same base, or does the next wave
proceed cutting from stale `main`, silently dropping that Work's scope from
the sprint? All three are operationally different and the plan chooses
none.

**What a correction would be:** add one sentence to §7 stating the response
to a wave-Work landing in `failed` (e.g. "retry once against the same base;
if still `failed`, the wave blocks and Captain escalates to the owner") —
or name it explicitly as an Unknown left to the orchestrator's live
judgment, the way §4 already does for the sysinfo-latency question (F3
above notwithstanding).

**Verified vs. believed:** verified that no explicit statement exists in
the plan (direct read, twice) and that no standing convention is recorded
elsewhere in the accessible ledger/retrospective corpus (targeted `grep`,
reported above — absence-of-evidence, stated as such rather than as proof
of a genuine gap in the owner's own head, which I cannot verify).

---

## What I checked and found nothing on

- **W6 (gate) dispatchability.** Confirmed `.sergeant/workflows/validate-and-ship/`
  exists, published, with the stages (`40-drive-gates`, `50-reconcile-custody`)
  the plan and ADR 0005 both cite — matching FOUNDATION-1's own enactability
  finding 4, which already validated this mechanism is real and dispatchable.
  §8's own risk 3 (W6 sidesteps `surface::attach` by cutting fresh from the
  integration tip rather than attaching to another Work's branch) checks out
  mechanically against `src/runtime/surface.rs:332` and `:431-441`
  (`materialize`'s ordinary path cuts from the repository's current HEAD,
  confirmed by direct read) — an ordinary `materialize` against an
  already-advanced integration tip does produce a branch that "already holds
  the content" as claimed. Not a finding.
- **§5's wave-cutting mechanism itself** (the `src/runtime/surface.rs:332`
  and `:431-441` citations). Read both spans directly; the claimed behavior
  (branch cut from current HEAD, `base_sha` captured via `git rev-parse
  HEAD`, worktree added from that SHA) matches. Not a finding for this axis
  (a citation-accuracy question belongs to the fidelity axis, and this one
  is accurate).
- **§6's ceiling-sizing rationale** (5400s, because #90 has no exit door
  yet). Internally consistent with §8.1's own risk statement — the plan
  doesn't hide this one, it names it as a real, accepted cost. Not a
  finding.
- **R6's "#108 is fixed by a start-of-run reaper" mechanism**, checked
  against #108's own issue body and its `LESSONS.md` L21 entry, both of
  which propose RAII/in-test cleanup rather than a start-of-run sweep. On
  reflection this is not a contradiction: a start-of-run reaper that sweeps
  the known `sgt-watch-test-hold-*.ready` glob is a coherent, arguably more
  robust alternative precisely because (per R6's own stated reasoning) an
  RAII guard's `Drop` would not survive a `SIGKILL`'d test process either.
  The plan doesn't name the exact glob or where the sweep lives, but that is
  an ordinary implementation-judgment gap, not an undecided product
  question — a Sonnet Work can reasonably resolve it without inventing a
  decision the plan should have made. Not a finding.
