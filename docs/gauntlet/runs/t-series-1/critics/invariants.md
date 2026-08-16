# T-SERIES-1 — blind critic: invariants

Axis: does any section of `reference/proposal-tui-t-series.md` §1–§25 violate
`NORTH-STAR.md`'s ownership boundaries or its "Never" list, the R-NS-*
rulings (R-NS-6 especially — grading §5.6's own restatement rather than
accepting it), the architecture invariants in `docs/DEVELOPMENT.md` (journal
is the only truth, one owner, work state ≠ process state, clients are equal
— reach state only through the API, ambiguity fails closed), or the
equal-client boundary `tests/m6_surfaces.rs` t5 enforces today? Includes the
Ponytail Minimality Ladder (`reference/notes/ideaos-agent-contract.md`)
against §22's decision register, per `docs/gauntlet/contracts/T-SERIES-1.md`.

Context read: `reference/proposal-tui-t-series.md` (full, §1–§25),
`docs/gauntlet/contracts/T-SERIES-1.md`, `NORTH-STAR.md`,
`docs/DEVELOPMENT.md`, `reference/notes/ideaos-agent-contract.md`,
`tests/m6_surfaces.rs` (t5 and t5b, the structural-scan tests), and the
FOUNDATION-1 precedent (`docs/gauntlet/runs/foundation-1/critics/invariants.md`)
for report shape.

Three findings. The first lands on the equal-client API boundary that
`tests/m6_surfaces.rs` t5 enforces today, against §5.3/§16.2/§16.3's Estate
and Doctor sharing decision. The second and third are Ponytail-register and
R-NS-6-restatement findings, per the contract's explicit instruction to grade
the register and the restatement rather than accept them.

---

## Finding 1: `inv-estate-doctor-bypasses-client-boundary`

**Severity:** error

**Section:** §5.3 ("Estate-local behavior is shared, not tunneled through
the daemon"), Decision T2-14; §16.2 ("Estate operations"); §16.3 ("Doctor
report"); §23.3 ("New daemon APIs for repo/group/Doctor" — Rejected)

**Claim at issue:** §5.3, Decision T2-14, verbatim: "The TUI consumes
repo/group and Doctor behavior through narrow typed local operations
extracted from the current CLI implementation. The CLI and TUI format the
same outcomes differently." Immediately after, the proposal itself names
what this changes: "This is an explicit refinement of the old 'TUI imports
only ApiClient' source-scan rule." §23.3 rejects the alternative that would
avoid this: "New daemon APIs for repo/group/Doctor — Rejected. They would
distort local/no-daemon semantics. Share the existing local implementation
instead."

**What I checked:** `docs/DEVELOPMENT.md`'s actual "Clients are equal"
invariant: "The CLI and TUI (`src/tui.rs`) reach state only through the
loopback HTTP/SSE API (`src/api.rs`) via `ApiClient`. This is enforced by
tests, not convention: `tests/m6_surfaces.rs` t5 scans `tui.rs` for
internals imports — widening that reach fails the test by design. If a
client needs something the API lacks, extend the API." Then I read t5
itself (`tests/m6_surfaces.rs:2308`): it asserts `crate_paths(&source) ==
vec!["api"]` over all of `tui.rs`, and separately bans naming
`ApiState`/`registry`/`Journal`/`Analytics`/`Engine`/`blocking_lock` as
tokens. t5b (line 2348) exists specifically to prove the scan cannot be
fooled by an unusual `use` spelling — this is a hardened structural gate,
not an incidental accident of today's minimal TUI. I then checked T2-16
(§5.5): `sgt tui` "continues to refuse without a running daemon" (ADR 0009
no-auto-spawn) — so `tui.rs` code only ever executes while a daemon is
already live and reachable.

**What I found:** T2-14 proposes that `tui.rs` call "narrow typed local
operations extracted from the current CLI implementation" directly — i.e.
in-process Rust function calls into a shared module that is not
`crate::api` — for repo/group and Doctor behavior. That is a second
crate-internal import path beyond `api`, which t5 rejects today by
construction; the moment `tui.rs` gains a `use crate::estate::...` (or
wherever this shared extraction lands), t5 fails. The proposal is aware the
rule it is changing exists (it names it, "the old ... source-scan rule"),
but never once names `tests/m6_surfaces.rs`, `t5`, or "this test needs
revision" anywhere in the document — not in §16 where the extraction is
specified, not in §19 (Testing and Validation), and not in §21 (Acceptance
Contract), where item 40 states "Estate/Doctor behavior is shared through
small typed extractions, not duplicated" as an achieved fact rather than a
named test-boundary change. §23.3's stated reason for rejecting the
API-preserving alternative — "would distort local/no-daemon semantics" — is
a cost the *CLI* bears, since only the CLI needs to run Doctor/repo/group
without a daemon. It is not a cost the TUI's own decision actually forces:
per T2-16, the TUI never runs without a live, reachable daemon, so nothing
requires *tui.rs specifically* to reach estate/Doctor facts without going
through the API. The document applies one client's no-daemon requirement to
justify weakening the other client's dedicated, currently-enforced
architecture test, without ever separating the two clients' actual
constraints.

**Does the section survive the correction?** Not as worded. Either (a)
expose repo/group/Doctor through new authenticated daemon routes reached via
`ApiClient` like everything else — satisfying DEVELOPMENT.md's own
prescribed remedy ("if a client needs something the API lacks, extend the
API"), leaving t5 untouched, and leaving the CLI's existing no-daemon local
implementation alone as a separately-scoped decision — or (b) explicitly
name `tests/m6_surfaces.rs` t5 as a boundary this proposal revises, with its
own stated justification for why estate-local, non-daemon operations should
be a second sanctioned reach path for `tui.rs`, added to the T0/T3 program
as an adjudicated step rather than an unstated background assumption. As
written, T2-14 cannot be enacted without silently breaking or rewriting a
named, currently-green invariant test that this contract specifically asked
the panel to check the proposal against.

---

## Finding 2: `inv-t2-40-unjustified-r7`

**Severity:** warning

**Section:** §11.2 ("Read-only catalog route"), §22 (T2-40)

**Claim at issue:** §22's register: "T2-40 | R2/R6/R7 | One minimum
workflow-catalog endpoint." §11.2 body: "**Decision T2-40 (R2/R6/R7):** Add
one workflow catalog projection because the TUI must not privately
reinterpret executable procedure." The supporting text: "The route reuses:
current estate/workspace discovery; current workflow loader and validation;
root publication boundary; current embedded fallback; existing Axum and
`ApiClient` patterns."

**What I checked:** `reference/notes/ideaos-agent-contract.md`'s rung
convention: "An `R7` entry must name which lower rungs were checked and why
they failed." I then checked the register's one other R7 entry, T2-31
(§8.7, the `ratatui-textarea` dependency decision), which does this
correctly and explicitly: "R1 fails: multiline deliberate input is settled.
R2 fails: the current one-line `String` buffer cannot satisfy it. R3/R4
fail: standard Rust and terminal events do not provide editor behavior. R5
fails: the currently installed dependency set has no editor. R6 fails:
correct wrapping, visual-row cursor movement, paste, delete, and scrolling
are not a tiny composition."

**What I found:** T2-40's own supporting prose argues almost entirely for
R2 (reuse of existing discovery/loader/validation/fallback/Axum patterns)
and R6 (a thin new route as "tiny composition"). Nowhere in §11 does the
proposal name what specifically fails at R6 and requires the escalation to
R7 — no lower rung is stated as checked-and-failed for this decision, unlike
T2-31 a few decisions earlier in the same register. Tagging the entry
"R2/R6/R7" without that reasoning is the unjustified-R7 failure mode the
contract names explicitly, and it is inconsistent with how the document
itself demonstrates the convention should be applied one section prior.

**Does the section survive the correction?** Yes — either drop the R7 tag
(the entry reads consistently as R2/R6 given its own stated reuse, and nothing
about the route's design changes) or add the missing failed-lower-rung
narrative for whichever part of the route is judged to exceed "tiny
composition." Neither fix changes the endpoint itself.

---

## Finding 3: `inv-r-ns-6-restatement-incomplete`

**Severity:** info

**Section:** §5.6 ("Execution is not dialogue"), Decision T2-17;
`NORTH-STAR.md` R-NS-6

**Claim at issue:** §5.6, verbatim: "R-NS-6 distinguishes execution
mechanics from the harness-owned conversation. `respond` answers a parked
request. It is not a generic message operation." **Decision T2-17
(R1/R2):** "T-Series adds no arbitrary active-turn guidance, continuous
chat, embedded harness session, or PTY supervision."

**What I checked:** `NORTH-STAR.md`'s actual R-NS-6 text in full: "sgt owns
message *mechanics* to a running execution (`needs_input`/`respond`,
journaled); the harness owns the *conversation*. Nothing conversational is
ever engine work; whether a transport's actor can ask mid-run is a measured
per-transport capability with runtime withdrawal, never new hold machinery.
Consequence: the WORKFLOW-IF-E3 category is empty — grilling-class packages
are operator skills."

**What I found:** Per the contract's instruction to grade this restatement
rather than accept it: §5.6 keeps the load-bearing half of R-NS-6 (engine
owns mechanics, not conversation) but drops both its explicit conditional
dimension — that a transport's actor asking mid-run is a *measured,
per-transport, runtime-withdrawable* capability, not a flat ban — and its
named consequence (WORKFLOW-IF-E3 dissolved into operator skills). As
written, §5.6 reads as though R-NS-6 forecloses any mid-run actor question
outright, when the actual ruling is that such capability may already exist
per-transport and is governed by measurement, not by whether a hold exists.
Nothing in T-Series' own scope is wrong as a result — §6.2 excludes any
*new* active-turn channel regardless of transport capability, which is
compatible with either reading — but a reader relying on §5.6 as a
restatement of R-NS-6 would not learn that this is a conditional, measured
ruling elsewhere in the architecture rather than an absolute one.

**Does the section survive the correction?** Yes, trivially — restoring the
per-transport clause and the WORKFLOW-IF-E3 consequence costs nothing and
changes no T-Series decision, since T-Series adds no such channel under
either reading of R-NS-6.

---

## What I did not find

Checked and clean, no finding filed:

- **Fleet as a domain object** (`NORTH-STAR.md`'s Never list). §10 describes
  Fleet as "the complete Work browser" — a filtered/sorted view over
  existing Work records reached through `ApiClient`, with no new persisted
  entity, store, or backend type introduced anywhere in §10 or §16. No
  finding.
- **PM semantics, upstream author-specific integrations, the re-hash intent
  ceremony, the settled D7/B1/#131-class machinery** (Never list). Nothing
  in the proposal's ticketing-free Fleet/Work model, submission fields
  (§9.1), or dispositions (§23) approaches any of these. Not implicated.
- **Reconstructed tmux-era supervision / embedded harness PTY** (Never
  list; ADR 0006's exec-not-supervise boundary). §23.3 explicitly rejects
  "Embedded harness/PTTY" citing ADR 0006; T2-17 excludes PTY supervision
  and continuous harness sessions; T2-33 excludes mouse/PTY-adjacent
  interaction entirely. Consistent, no drift toward supervision language
  found.
- **"One owner" / the daemon exclusively owns the data dir.** No section
  proposes a client-side cache, second store, or persisted client-local
  state for daemon-owned facts. T2-23 is explicit that the Attention
  drawer "stores no notification state." Estate/Doctor's local, non-daemon
  domain (flagged in Finding 1 for a different reason — the client
  boundary, not data-dir ownership) does not touch the data dir either way.
- **"Work state ≠ process state."** §5.7/T2-18 and §8.5 explicitly derive
  spinners and labels from journal-projected state ("Silence, elapsed time,
  or a process table never creates a Work transition"), consistent with
  `docs/DEVELOPMENT.md`'s restart-reconciliation invariant and the
  fail-closed pattern it extends elsewhere in this repository.
- **"Ambiguity fails closed."** The proposal's only "fail closed" language
  (§19.4, workflow-catalog "missing/malformed/disagreeing records fail
  closed") is correctly scoped to its own domain and does not stretch the
  citation to cover an unrelated mechanism the way a prior gauntlet unit
  flagged elsewhere. No finding.
- **§22 register completeness.** All 64 `T2-01`..`T2-64` decisions named in
  the body have exactly one matching row in §22's table, and vice versa —
  no missing entries, no orphaned register rows, no decision left
  unlogged. The contract's "§22's decision register claims a rung for every
  normative decision" check holds numerically.
