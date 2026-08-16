# PATH-TO-MAC-1 — adjudication

Orchestrator adjudication of the panel and refuter output, 2026-08-15.
Contract: `docs/gauntlet/contracts/PATH-TO-MAC-1.md`. Artifact:
`docs/gauntlet/runs/path-to-mac-2026-08-15/plan.md` at `d86885f`.

## Outcome: **sent back on §4/§5; validated with findings elsewhere**

This is the first unit in this ledger to reach **sent back** on any section,
and FOUNDATION-1's own framing is why: its outcome was "validated with
findings" precisely because *"every finding's own verdict holds that the
affected section survives a local correction."* That is not true here. §4 and
§5 rest on a factual premise the panel disproved — that #18, #81, and #82 need
building. They are already built. No local correction repairs a section whose
subject does not exist.

Everything outside §4/§5 survives local correction and is corrected below.

## Verdicts

| Axis | Findings | Refuted | Confirmed | Severity moves |
|---|---|---|---|---|
| fidelity | 4 | 0 outright (1 partially) | 3 + 1 PLAUSIBLE | 1 × downgrade-in-reasoning |
| invariants | 4 | 0 outright (1 sub-claim) | 2 + 2 PLAUSIBLE | none |
| enactability | 6 | 0 | 6 | **1 × upgrade** (a non-finding became a finding) |
| assumptions | 8 | 1 central claim | 7 + 3 PLAUSIBLE | 1 × downgrade, 2 × upgrade |
| **total** | **22** | **1 central + 2 sub-claims** | **18 + 6 PLAUSIBLE** | **5 moves** |

## The finding that sends §4/§5 back

No single seat stated it. It assembles from `assumptions-F3`, `assumptions-F7`,
the assumptions refuter's assigned second line of attack, and `invariants-F1` —
which is what an adjudicator is for.

| Issue | Plan assumed | Established |
|---|---|---|
| **#18** | direct `/proc` reads need `sysinfo` | **Shipped**, both platforms, unit-tested; macOS arm UNVERIFIED |
| **#81** | GNU-only `df` needs `fs4` | **Shipped**; the POSIX arm (`df -k <path>` + positional parsing) already fixes the filed defect |
| **#82** | hand-rolled XDG tail | **Shipped**, both conventions, unit-tested; macOS arm UNVERIFIED |
| **#85** | replaces `/proc/mounts` parsing | **Unbuilt** — no mount-parsing code exists anywhere in `src/` |

Three of W1's four items are complete, dual-platform, tested implementations
**shipping in the binary today**, each blocked on exactly one thing: a macOS
host to flip its `UNVERIFIED` marker. `src/platform/process.rs` says so in its
own module doc — *"They close #18 when someone measures them there, not when
this lands."* `src/platform/data_dir.rs` says it too.

So W1 as written is not build work. It is a **replace-working-code-with-a-
dependency** decision, and `src/platform/disk.rs:1-21` — written 2026-08-14,
*one day before this plan* — already argues the other way and says the
shell-out posture **"still holds."**

**Root cause is L18**, and it names itself: *"R1's 'already exists' includes
the product you are building… the comparison list is a moving target that
every future pass re-derives from the product, never from memory."* The plan
was derived from the issue tracker and the authoring session's memory of it,
never from the current `src/platform/`.

## Carried to the owner — rulings, not corrections

FOUNDATION-1 left §6 open on exactly this principle: a correction the
orchestrator may apply, a ruling it may not.

1. **Crates, or verify?** Adopting `fs4`/`sysinfo`/`directories` for
   #18/#81/#82 means replacing tested working code and overturning
   `disk.rs:1-21`'s recorded, one-day-old reasoning. The alternative is to take
   all three to the Mac and close them by measurement, which is ADR 0001's
   posture and satisfies the parity ruling already — the existing code is
   hand-rolled on *both* platforms, not split. **Orchestrator recommendation:
   verify, and build only #85.**
2. **#108's scope.** The reaper ruling was made on a mis-framed question — see
   below. #108 needs guard-shaped cleanup; the reaper closes a different,
   unnumbered finding. **Recommendation: W3 does both, labeled separately, and
   the reaper's own finding gets filed as an issue.**
3. **#109's width.** #109's own later comment reframes it as *"write the
   disk-footprint contract,"* with the verbs as enforcement surface and
   retention scope as its central clause. R4 answers the clause, not the
   contract. **Recommendation: state the narrowing explicitly and accept #109
   will not fully close on W4's output.**

## Corrections applied to the plan

| Source | Section | Change |
|---|---|---|
| fidelity-F1 + assumptions-F1, as refuted | §4 | Citation moves from "ADR 0002 (D4)" to `src/platform/disk.rs:4-6` (originally a comment on `src/backend/docker.rs::free_space`, moved by `4eadc50`). **The quote is kept** — it is verbatim and real. "An ADR refresh is owed" is struck: there is no ADR to refresh |
| assumptions-F2 + the enactability refuter's upgrade | §2 R6, §5 | R6's stated justification is false for #108. Corrected to name the two mechanisms separately |
| enactability-F1 + invariants-F4 | §3, §5, §10 | #95's split named explicitly: the fail-loudly guard is Cerberus-buildable, the clock choice is the Mac's |
| enactability-F2 | §5 | W2's #90 half gains a target shape — `R-MVP1-10`'s existing exit door as the nearest precedent — or an explicit Unknown |
| enactability-F3 | §4 | The sysinfo-latency criterion states a zero threshold: any measured slowdown is raised |
| enactability-F5 | §6 | Every brief carries `gh … --repo miztertea/sergeant-rs`. **Already fixed structurally** — a `github` remote was added to the estate clone, so the failure no longer reproduces |
| enactability-F6 | §7 | The response to a wave-Work landing `failed` is stated rather than assumed |
| invariants-F1, as refuted | §4 | If crates are ruled in, `fs4` stays and the plan states why raw `rustix` was rejected. Correction (a) is **struck**: `rustix` is transitive-only and resolved *without* the `fs` feature, so "no new dependency edge" is false |
| invariants-F2 | §4 | Every proposed dependency records a Ponytail rung |
| assumptions-F4, as refuted | §4 | The 5 KiB gap is stated as an unexplained reconciliation gap with the command that would settle it. **Not** re-summed to 166 — 161 is a real combined measurement, 166 a derived sum of three noisier ones |
| assumptions-F5, F6 | §2 | `sergeant.toml:11-14`; `docs/DEVELOPMENT.md:70-73` |
| assumptions-F8 | §8 | The pre-upgrade `gh` claim is labelled |

## Method notes for the ledger

**A specific line of attack changed every outcome, again.** All four refuters
were given one; all four moved something. Two produced the unit's only
refutations, one produced its only upgrade, and one refuted the orchestrator's
own hypothesis. FOUNDATION-1 recorded this pattern from three axes; two units
now show it.

**The panel produced a false absence, and the refuter caught it.** Two seats
independently reported the "one syscall" quote as invented, both from greps
scoped to `docs/`, `reference/`, `GAUNTLET.md`, `LESSONS.md` — excluding
`src/`, where it lives. The fidelity refuter re-ran the critic's own stated
command and got three hits where the critic reported one, then traced the
phrase verbatim into `src/backend/docker.rs`'s git history. **Had the critic's
proposed correction been applied, a true and sourced claim would have been
deleted from the plan.** This is L23 occurring inside two reports that both
cite L23 — the third recorded instance of a pattern surviving being named.

**The panel disagreed with itself and the disagreement was the signal.**
`enactability` explicitly declined to find on R6; `assumptions` graded it an
error. The enactability refuter settled it by reading `src/watch.rs` rather
than reasoning: `test_hold_wait` returns an ordinary `Err` and the test
completes normally, so no `SIGKILL` is involved and `Drop` would run. A
"nothing found" from one seat is not evidence of nothing.

**An all-Sonnet panel held** (contract Unknown 2). 8 seats, all Sonnet, all 1
turn of 25, 2,299 lines of findings and verdicts. It refuted a critic's central
claim, upgraded a declined non-finding, downgraded two severities, refuted the
orchestrator's own hypothesis, and disproved the premise of two plan sections.
Second data point after FOUNDATION-1; the model-assignment note's reservation
of Opus for "blind adversarial review" has now not been needed twice.

**`research` was a serviceable critic seat** (contract Unknown 3). Its durable
outcome — primary sources only, every claim traced, one cited Markdown file —
is a critic's contract nearly verbatim, and every seat honored it. Friction
worth recording: the workflow has no notion of a *verdict*, so refuters
invented their own CONFIRMED/REFUTED/DOWNGRADED vocabulary from the brief. A
dedicated `gauntlet-seat` workflow would carry that vocabulary and the
probe-hygiene rules structurally instead of by brief.

**One product defect surfaced by running the loop:** a dispatched Work cannot
reach the GitHub tracker. The estate clone's only remote was a local path, so
`gh` failed with a misdirecting auth remedy while `gh auth status` showed a
valid account. Every Work whose scope is stated as bare issue numbers depends
on this. Fixed in the estate by adding a `github` remote; the general problem —
that estate clones get no tracker reachability by construction — is worth its
own issue.
