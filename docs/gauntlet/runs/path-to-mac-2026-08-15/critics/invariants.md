# PATH-TO-MAC-1 — blind critic: invariants

Axis: the plan against `NORTH-STAR.md`'s ownership boundaries (Core/OS/Estate/
Surfaces, R-NS-1..6, the Never list), `docs/DEVELOPMENT.md`'s architecture
invariants (journal-is-truth, one owner, work state ≠ process state,
adjacent-append crash windows, fail-closed ambiguity), and the Ponytail
Minimality Ladder (`reference/notes/ideaos-agent-contract.md`) for every
addition the plan proposes.

## Method note

Read, in full, in this order: `docs/gauntlet/contracts/PATH-TO-MAC-1.md`;
`docs/gauntlet/runs/path-to-mac-2026-08-15/plan.md` at `d86885f`;
`reference/notes/gauntlet-pattern.md`; `docs/DEVELOPMENT.md`; `NORTH-STAR.md`;
`reference/notes/ideaos-agent-contract.md`; `LESSONS.md` L3, L5, L6, L13–L23
(the entries dated during or citing the cross-platform sprint this plan
continues); `GAUNTLET.md`'s deviation register and backlog (rows D1–D10,
B1–B8) in full; `docs/adr/0005-gating-becomes-a-dispatched-work.md` in full
(cited by the plan's R8); `docs/gauntlet/runs/cross-platform-2026-08-14/
retrospective.md` and `lessons.md` for the prior-sprint context the plan's
issue numbers (#90, #94, #95, #108, #109) come from, since this sandboxed
clone has no reachable GitHub remote (`gh issue list` fails — "none of the
git remotes... point to a known GitHub host") and these in-repo documents are
the best available primary source for what those issues actually are.

Ran: two `Context7` documentation queries against the upstream crate docs for
`rustix` (`/bytecodealliance/rustix`) and `fs4` (`/al8n/fs4`) to check a
Ponytail-ladder question about §4's crate table (below) — this is source
verification of the crates' own published API surface, not a re-derivation of
§4's *measurements* (binary size, compile time), which the contract places
out of scope and which I did not touch. No mutation probe was needed for this
axis, so no disposable worktree was created.

Could not check: the plan's own citations to issue numbers against a live
tracker (no GitHub remote reachable, as above) — I cross-checked the plan's
issue descriptions against the in-repo retrospective/lessons/close-out
documents instead, which is weaker than a live tracker and is flagged
per-finding where it matters. I did not re-derive §4's measured binary-size
or compile-time figures (explicit non-goal). I did not evaluate `#96`
specifically for the R-NS-6 (execution ≠ dialogue) boundary because no
in-repo document available to me describes what #96 actually is beyond its
number appearing in a list of "findings nobody was hunting" — recording this
as an explicit gap rather than guessing at the issue's content.

Checked and found clean, no finding: `NORTH-STAR.md`'s Never list (fleet as
domain object, PM semantics, tmux-era supervision, etc. — none proposed here);
R-NS-1/2/3/5 (durability, regeneration, no-second-home, estate opacity — the
plan touches none of estate/manifest/AGENTS.md-generation territory); the
Core/OS/Estate/Surface ownership split in NORTH-STAR's "Ownership" section
against every wave's scope; §6's dispatch parameters against the spend-envelope
ownership rule (an orchestrator setting `--turns`/`--ceiling-secs` at submit
time is existing, ratified engine surface, not a new one — GAUNTLET.md B6/ADR
lineage); R6's start-of-run reaper (#108) against "one owner" — it operates on
test-harness-created rigs, not the production daemon's data dir, so no
conflict; R4's "target/ never in scope" against `DEVELOPMENT.md`'s build-dir
placement guidance — aligned, not in tension.

---

## Finding invariants-F1: `fs4` is proposed as new machinery (R7) without the plan checking whether the already-present `rustix` (R5) already does the job

**Severity argued: warning.** Nothing breaks if this ships as written — the
measured binary-size cost is trivial (§4: "+4.7 KiB"). The finding is that the
plan skips a ladder rung this repo's own convention requires it to check, and
in this specific case the skipped rung is very plausibly sufficient, which is
exactly the shape of finding this axis exists to catch ("An unjustified R7 is
a finding... machinery proposed where a lower rung would do").

**The exact plan text at issue** (`plan.md` lines 75, 78, 82–85):

> `#81 | fs4::available_space / statvfs — Unix path is rustix v1 → statvfs(2) | df -k --output=avail shell-out (GNU-only)`
> `#85 | fs4 for portable flock(2); sysinfo Disks API for per-mount filesystem type | /proc/mounts parsing (Linux-only)`
> "Of the 10 crates in the candidate tree, **5 are already present** —
> `bitflags`, `libc`, `linux-raw-sys`, `memchr`, and **`rustix`**. Genuinely
> new: `fs4`, `sysinfo`, `directories`, `dirs-sys`, `option-ext`."

**The governing text it contradicts** —
`reference/notes/ideaos-agent-contract.md` lines 34–54:

> "| R5 | Installed dependency? | Use it |" ... "**Rung logging convention
> (this repo):** every design decision in a ledger entry, every
> deviation-register row, and every new dependency, file, trait, or store
> records the rung it resolved at (`R1`–`R7`). An `R7` entry must name which
> lower rungs were checked and why they failed."

§4 never asks whether `rustix` — already present, per the plan's own
measurement two lines above the crate table — already exposes what #81/#85
need directly, before reaching for a new crate.

**Verified vs believed.** VERIFIED in-session via Context7 against `rustix`'s
own docs (`/bytecodealliance/rustix`): `rustix::fs::statvfs` is a public
function ("Public function in `rustix::fs::statvfs` that takes a path and
returns `io::Result<StatVfs>`. On macOS, `StatVfs.f_bavail` and
`StatVfs.f_frsize` are available for calculating available bytes"), and
rustix's own platform-support notes state "full support for Linux across
multiple architectures and... broad support for various Unix-like operating
systems including macOS." VERIFIED via `fs4`'s own docs (`/al8n/fs4`): "On
Unix-like systems (Linux, macOS, BSD), `fs4` utilizes `rustix` v1 with the
`fs` feature, employing `flock(2)` for locking and `statvfs(2)` for
filesystem statistics" — `fs4`'s own locking code is shown calling
`rustix::fs::FlockOperation::NonBlockingLockExclusive` directly. So on the
plan's own two target platforms (Linux, macOS — Windows is not in scope
anywhere in this plan or in R2's parity ruling), `fs4`'s entire contribution
for #81/#85 is a thin wrapper over a crate this codebase's dependency tree
already resolves. BELIEVED, not verified: that wiring `rustix::fs::statvfs`/
`flock` directly into `src/platform/` (the ADR 0002 `#[cfg]`-selected-module
boundary this repo already uses for exactly this kind of OS fact) would be a
clean fit in practice — I did not run a build or `cargo tree` against this
checkout to confirm feature-flag/version compatibility, since that would mean
touching `Cargo.toml`/`Cargo.lock`, and this axis does not warrant a mutation
probe for a documentation-level ladder question.

**What a correction would be.** Either (a) drop `fs4` from §4's crate list
and have W1 call `rustix::fs::statvfs`/`flock` directly through the existing
`src/platform/` boundary — this also directly answers ADR 0002 (D4)'s original
objection ("adding a libc-binding dependency for one syscall") more completely
than adding a new crate does, since no new dependency edge is added at all; or
(b) keep `fs4` but add one sentence to §4 naming why the R5 rustix-direct path
was rejected (e.g., a stable ergonomic `FsStats`/`TryLockError`/`FileExt` API
that isn't rustix's job to provide, or insulation from a future rustix
major-version bump) — satisfying the rung-logging convention either way.

---

## Finding invariants-F2: none of §4's five new dependencies records a Ponytail rung, contrary to this repo's own logging convention

**Severity argued: info/warning.** Process-conformance gap, not a
correctness defect.

**The exact plan text at issue:** the §4 crate table (`plan.md` lines 73–78)
and "Genuinely new: `fs4`, `sysinfo`, `directories`, `dirs-sys`,
`option-ext`" (line 85). No `R1`–`R7` annotation appears anywhere in §4 for
any of the five.

**The governing text it contradicts** — `GAUNTLET.md` lines 14–15:

> "Design decisions and deviations log their **Ponytail rung** (`R1`–`R7`;
> ladder in `reference/notes/ideaos-agent-contract.md`): the rung the
> decision resolved at. `R7` (new machinery) entries name which lower rungs
> failed and why."

— and `ideaos-agent-contract.md`'s rung-logging convention quoted in F1
above, which extends the same requirement to "every new dependency."

For `sysinfo` and `directories`, the rung is inferable and defensible from
§2's R2 parity ruling (no already-present crate does cross-platform process
introspection or macOS-vs-XDG path resolution, and R2 forbids a Linux-crate/
macOS-hand-roll split that might otherwise let a lower rung apply
asymmetrically) — but the plan never states this reasoning as a rung
resolution, it has to be reconstructed from a different section. `fs4`'s case
is F1 above. `dirs-sys` and `option-ext` are transitive dependencies of
`directories` and were not evaluated independently anywhere.

**Verified vs believed.** VERIFIED: read §4 in full; no rung notation for any
of the five crates. BELIEVED, not proven: that this convention is meant to
bind a *plan* document and not only ledger/deviation-register entries after
the fact — the convention's own wording ("every new dependency... records the
rung it resolved at") does not scope itself to post-hoc records only, and a
plan that already table-izes measured costs per crate is the natural place to
also record the rung, but I cannot point to an enforcement mechanism that
would reject this plan for the gap.

**What a correction would be.** Add a rung column or inline clause to §4's
table, e.g. "`sysinfo` — R7, no already-present crate crosses the
Linux/macOS process-inspection gap under R2" / "`directories` — R7, same gap
for path resolution" / "`fs4` — see the rung question raised as
invariants-F1."

---

## Finding invariants-F3 (PLAUSIBLE): the plan's per-brief checklist never routes the adjacent-append crash-window check to the two Works most likely to need it

**Severity argued: warning**, recorded PLAUSIBLE because it depends on an
implementation shape (W2's and W4's actual diffs) that does not exist yet —
this is a plan, and the contract's non-goals bar me from writing or deriving
that code.

**The exact plan text at issue** — §6's full per-brief-content list
(`plan.md` lines 150–161), quoted in full because the absence is the finding:

> "Every brief carries, per retrospective §2.4's *confirmed* approaches: the
> prior art named as **settled**... the commit trailer stated explicitly...
> **why**, not only what... a pointer at `GAUNTLET.md`'s deviation register
> (**L3**); evidence-vs-hypothesis labels on every factual claim (**L15**);
> `PATH="$HOME/.cargo/bin:$PATH"`... **run long commands in the
> foreground.**"

Seven items, none of them a pointer to `DEVELOPMENT.md`'s architecture
invariants or to `LESSONS.md` L6 specifically. The wave table (line 124)
assigns #90/#94 — described by the plan itself as "where an interrupted turn
should land, and what `completed` must guarantee" (this exact framing is
quoted from the cross-platform plan at
`docs/gauntlet/runs/cross-platform-2026-08-14/plan.md:114`, which this plan's
§2 R1 table cites as the source of #90/#94) — to W2, and #109's dirty-state
retention (R4) to W4.

**The governing text it contradicts** — `docs/DEVELOPMENT.md` line 41:

> "**Adjacent-append crash windows** are this architecture's recurring hazard
> (LESSONS L6): any path appending two causally-linked events must tolerate
> the second one missing or write one compound event. Check for this class in
> review of any journal-touching change."

— and `LESSONS.md` L6 (lines 311–317): "M2: exact-once broke in the window
between submit's two journal appends. M3: a daemon crash in the same window
stranded work in a state nothing would pick up... Check for this class
explicitly in every milestone that adds a multi-append sequence."

Both #90 ("a ceiling interrupt wedges a Work in `active` with no verb that
reaches it," per the plan's own §6) and #109's retention recording are
exactly the shape of change — a new terminal-state or retention-recording
path with more than one causally linked fact to persist — that L6 names as
this architecture's *recurring* hazard, not a hypothetical one.

**Verified vs believed.** VERIFIED: §6's checklist, read in full, does not
mention L6, "adjacent-append," or `DEVELOPMENT.md`'s architecture-invariants
section; its only pointer into governing text is the deviation register
(a different document section, addressing a different failure mode — L3's
relitigation risk, not L6's crash-window risk). BELIEVED, not verified: that
W2's or W4's actual code will in fact add a new multi-append sequence — that
diff does not exist yet, and per the contract's non-goals this unit does not
write or derive it. I infer the risk from the *problem descriptions* the plan
itself gives for #90 and #109, not from any implementation I've seen.

**What a correction would be.** Add an eighth item to §6's per-brief list (or
a targeted addendum on W2's and W4's briefs specifically): "if this fix adds
a new sequence of causally-linked journal appends, check it against
`DEVELOPMENT.md`'s adjacent-append crash-window hazard (L6) — tolerate the
second append missing, or write one compound event."

---

## Finding invariants-F4 (PLAUSIBLE): §3/§10 reserve #95's clock choice for the Mac, but §5 dispatches all of #95 to a Cerberus Work in Wave 1, with no stated boundary between the two

**Severity argued: info**, recorded PLAUSIBLE — this reads as an internal
tension on its face, but a benign resolution (W3 builds everything except the
final selection) is equally consistent with the text as written, and I can't
adjudicate which the plan intends from the text alone.

**The exact plan text at issue.** §3 lists #95 as in-scope, undifferentiated,
alongside #108 (`plan.md` line 60): "`#95, #108 (harness hygiene)`." The very
next paragraph (lines 65–66) narrows it: "What the Mac still owns is
verification... plus **#95's clock choice**, which needs timing on real
hardware to pick between `perl -MTime::HiRes`, `python3 time.time_ns()`, and
accepting millisecond resolution." §10 repeats the reservation on its own
line (219): "Choosing #95's clock on real hardware." §5's wave table (line
123) assigns "`#108 (start-of-run reaper), #95`" to **W3**, Wave 1, cut from
`main` — i.e., dispatched on Cerberus, before any Mac session in this plan's
sequence.

**The governing text it engages** — `docs/DEVELOPMENT.md` line 40: "ambiguity
fails closed into `blocked` with a reason, never a guess" (stated there for
recovery reconciliation, but the plan's own §7 orchestrator-duties section
already imports the same doctrine for gate findings — "An `ask-user` finding
waits for the owner; it is never resolved autonomously" — so the plan treats
fail-closed as a general operating rule, not one scoped only to
`recovery.rs`).

The risk: if W3 is expected to fully "do" #95 on Cerberus, and the actual
clock-source decision requires real Mac hardware timing per §3/§10, a Work
executing that brief has nothing to do but guess at (or hardcode a default
for) the one measurement the plan says only the Mac can make — which is
exactly the behavior fail-closed doctrine forbids. The benign reading — W3
builds a pluggable clock abstraction and leaves the selection as an open
parameter — is not ruled out by the text, but it is also not stated by it.

**Verified vs believed.** VERIFIED: both passages exist as quoted, in the
same document, same commit, and are in tension as read literally. Not
CONFIRMED as a defect: I did not find any place in the plan that says which
of the two readings W3's brief should carry, so I cannot tell whether this is
a real gap or just terse phrasing that a competent brief-writer would resolve
correctly without the plan spelling it out.

**What a correction would be.** One sentence in §3 or §5 stating what W3
concretely delivers for #95 given the clock-choice reservation — e.g., "W3
implements a pluggable clock-source trait with the concrete selection left as
an explicit, documented open parameter; the Mac session's job is to pick and
pin it, not to build the abstraction."
