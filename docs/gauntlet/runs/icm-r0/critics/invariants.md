# ICM-R0 — invariants critic

Axis: **invariants** (`docs/gauntlet/contracts/ICM-R0.md`, axis 2).
Artifact under review: `reference/proposal-icm-r-procedure-authority.md`
(full text, §1–§20 plus the Source-to-Decision Map and Owner Decisions).

This critic is blind: written without reading any other critic's output or
anything under `docs/gauntlet/runs/icm-r0/`.

## Method

Read the proposal in full against current `main` (`ad20ec7`, not the
proposal's own `3a46b87` pin, per the contract's audit-pin-drift
instruction). Checked, in order:

1. `NORTH-STAR.md`'s ownership boundaries — Core/OS/Estate/Surfaces and
   R-NS-1 through R-NS-6 (full text pulled from
   `docs/gauntlet/notes/north-star-draft-2026-08-11.md`, since
   `NORTH-STAR.md` itself only restates R-NS-6 verbatim and references the
   other five as "drafted").
2. `AGENTS.md`'s routing table and standard workflow loop, as currently
   written on `main`.
3. Every ADR in `docs/adr/` (directory listing read fresh, not assumed) —
   0001 through 0012, with 0012 (2026-08-16, Estate/Doctor as daemon API
   surface) given the deliberate scrutiny the contract calls for since it
   postdates the proposal's audit pin.
4. The Ponytail Minimality Ladder's actual seven rungs
   (`reference/notes/ideaos-agent-contract.md`) against every citation in
   §17's Ponytail Decision Register (the register that uses that literal
   R1–R7 vocabulary — distinct from the unrelated inline `R1`…`R25`-style
   superscripts scattered through §1–§16, which are citation markers into
   §18's Source-to-Decision Map, not Ponytail rungs, and were not in this
   axis's scope).
5. The proposal's own stated hard boundary (Executive Summary: "No changes
   to src/, API routes, journal schema, Work state, backend traits, TUI
   behavior, or workflow.toml grammar in the initial workstream," restated
   in §11.2 and Acceptance Contract item 33) against what §7 and §10
   (not just §10.1–10.2; §10.3–10.4 are where the pilot and full corpus are
   actually scoped) propose doing — including checking file paths on disk
   for every package the proposal names as reconciliation subject.

## Findings

### F1 — severity: error — §10.4 (ICM-R3 full library reconciliation) vs. the proposal's own hard boundary

**Claim.** The proposal's Executive Summary states a hard boundary: "No
changes to src/, API routes, journal schema, Work state, backend traits,
TUI behavior, or workflow.toml grammar in the initial workstream." §11.2
repeats `src/**` as "Explicitly out of scope initially." Acceptance
Contract item 33 makes this a completion gate: "No file under src/... changes
during ICM-R0 through ICM-R4."

§10.4 (ICM-R3 — Full library reconciliation) names as an explicit subject
of the full-corpus reconciliation wave: "the built-in software-change
workflow as a separate embedded package." The reconciliation outcome for
every subject in that wave, per §10.4 and the required instruction shapes
in §7.2/§7.3, is that "every surviving package carries an authority
envelope" — i.e., every workflow's Layer-1 `CONTEXT.md` gains an `##
Authority envelope` section and every actor stage's `CONTEXT.md` gains a
`## Bounded judgment` section (§7.2, §7.3), applied corpus-wide per §15
Acceptance items 7 and 12.

**What I checked.** Where the built-in `software-change` workflow actually
lives on disk and how it reaches the running product.

```
$ grep -rn "software-change" src/domain/workflow.rs
593: /// The built-in `software-change` workflow, embedded at build time from
598: const EMBEDDED_WORKFLOW_TOML: &str = include_str!("../workflows/software-change/workflow.toml");
603:     include_str!("../workflows/software-change/00-prepare/CONTEXT.md"),
607:     include_str!("../workflows/software-change/10-implement/CONTEXT.md"),
611:     include_str!("../workflows/software-change/20-review/CONTEXT.md"),
```

`find . -iname "*software-change*"` resolves to exactly one directory:
`src/workflows/software-change/` — containing `workflow.toml` and each
stage's `CONTEXT.md`. There is no separate copy outside `src/`; `src/tui/
workflows.rs` and `src/domain/mod.rs` both treat it as `source: "embedded"`,
compiled into the binary via `include_str!` at build time. AGENTS.md's
"Choose a workflow" step confirms this is the actual default: "let `sgt
run` fall back to the workspace's own `software-change` workflow, then the
built-in default" — the built-in one is this embedded copy, and it is the
one the proposal names explicitly ("as a separate embedded package," §10.4)
as distinct from any workspace-local override.

**What I found.** Reconciling this package the way §7.2/§7.3 require —
adding an Authority envelope section to `src/workflows/software-change/
workflow.toml`-adjacent `CONTEXT.md` files and a Bounded judgment section
to each of its three stage `CONTEXT.md` files — is editing files whose
path is literally under `src/`. That is exactly the change class the
proposal's own Executive Summary, §11.2, and Acceptance item 33 name as
excluded through ICM-R4. §10.4 falls inside ICM-R0–ICM-R4 (it is ICM-R3).
The proposal does not carve out an exception for this one package, does
not note that its own hard boundary and its own reconciliation subject
list are in tension, and does not propose an alternative (e.g., editing a
non-`src/` staging copy and deferring the `include_str!` sync to a later,
explicitly-code milestone). This is not a hypothetical "quietly requires
engine cooperation" case — it is a direct, unqualified inclusion of a
`src/`-resident file set inside a workstream whose own acceptance contract
forbids touching `src/`.

**Verdict.** Does not survive as written. §10.4's software-change item
needs either (a) an explicit exception carved into the hard boundary and
Acceptance item 33, naming the embedded package and why touching its
`CONTEXT.md`/`workflow.toml` content (not its Rust plumbing) doesn't count
as a "src/ change" for this workstream's purposes, or (b) removal from the
ICM-R0–ICM-R4 corpus with reconciliation deferred to whatever milestone is
allowed to touch `src/`. Everything else in §10.4's subject list
(`.sergeant/workflows/*`, `skills/*`) is outside `src/` and unaffected by
this finding.

### F2 — severity: warning — §17 Ponytail Decision Register: two rung citations don't match the rung's own question

**Claim.** §17 states the register "uses the repository's existing R1–R7
Ponytail vocabulary" for its 25 `ICMR-*` decisions, and the source
document for that vocabulary (`reference/notes/ideaos-agent-contract.md`)
requires: "An `R7` entry must name which lower rungs were checked and why
they failed," as part of the "rung logging convention (this repo)."

**What I checked.** Read each of the 7 Ponytail rungs' literal question
(R1 "does this need to exist?", R2 "already in this codebase?", R3
"stdlib?", R4 "native platform feature?", R5 "installed dependency?", R6
"one line?", R7 "only then — the minimum that works") against every one of
the 25 `ICMR-*` rows' resolution text in §17.

**What I found.**

- **ICMR-01** cites R1 ("Pin `3a46b87`; do not design against moving
  main"). R1's actual question is "does this need to exist? No → skip it."
  Pinning an audit revision isn't a creation-vs-skip decision; the
  proposal's own §2.2 justifies the practice as precedent reuse — "The
  T-series supplies the closest house-format precedent. It pins an audit
  revision... This proposal uses the same form" — which is R2's question
  ("already in this codebase? reuse it, don't rewrite"), not R1's.
- **ICMR-22** cites R7 ("Admit runtime work only through a separately
  accepted PL-7 engine-gap contract"). This decision is not itself new
  authorship reaching the "only then, minimum that works" rung — it is a
  restatement of a discipline the proposal itself says already exists:
  §11.4 closes with "This is the existing engine-gap discipline, applied
  rather than merely cited" (citing source #11, the current ICM ladder's
  engine-gap template). Applying an existing discipline is R2's question,
  not R7's. And even read charitably as a genuine R7 entry, it doesn't
  satisfy the ladder's own logging convention — it never names which of
  R1–R6 were checked and why they failed for *this specific decision*; it
  points at a template (§5.9) that will require that of future engine-gap
  records, which is a different claim.
- Separately, and not itself a defect: none of the 25 rows ever cite R3,
  R4, R5, or R6. Given this is a content-and-procedure proposal with no
  dependency, stdlib, or platform-feature decisions in it, those rungs are
  largely inapplicable by construction, so their total absence is
  explainable rather than suspicious on its own. It does mean the register
  functions as a three-way R1/R2/R7 classifier in practice, which makes
  the two mislabeled rows above easier to miss on a skim — worth fixing
  alongside them rather than as a separate ask.

**Verdict.** Survives with a correction. Neither mismatch changes what the
proposal actually decided (pin the revision; require a PL-7 record before
runtime work) — both decisions are sound and consistent with the rest of
the document. The citations themselves are wrong and should be relabeled
R2 (reuse an established convention / reuse an existing discipline) before
ICM-R1 treats §17 as the canonical worked example other packages' decision
registers will imitate — a template with mislabeled rungs will propagate
the mislabeling corpus-wide once ICM-R3 starts citing it as precedent.

## Checked, no violation found

- **NORTH-STAR.md ownership boundaries (Core/OS/Estate/Surfaces).** The
  proposal's invariants (§4, especially 4.1 "the journal remains the only
  durable runtime truth," 4.2 Work-state distinctness, 4.5 "procedure
  remains data") track R-NS-1 (durability test: judgment content stays
  out of core) cleanly. Nothing in the proposal asks core to gain an
  opinion or asks OS content to gain a durability guarantee.
- **R-NS-2 (regeneration test).** §11.1's expected-changes list includes
  `.sergeant/index.md` (generated, disposable) and `AGENTS.md` ("routing/
  doctrine references" only) — consistent with AGENTS.md being canonical
  and referencing the catalog rather than being regenerated from it.
- **R-NS-3 / R-NS-5 (estate opacity, no second home).** Not implicated;
  the proposal never touches `repos/`, the manifest, or estate-repo
  inference.
- **R-NS-4 (surface statelessness) and ADR 0012.** The proposal explicitly
  excludes TUI behavior (Executive Summary, §11.2) and never proposes a
  second reach path into daemon-owned state. ADR 0012's estate/doctor API
  routes are a "later engine capability" per the contract's framing, but
  they don't intersect this proposal's content-only scope — no package
  hypothesis in §12 depends on the CLI-vs-TUI transport for `sgt doctor`/
  `sgt repo`, only on what those commands *do*, which ADR 0012 leaves
  unchanged (same underlying `crate::domain::manifest`/`mod doctor`
  functions, per ADR 0012's own "never a second computation" clause).
- **R-NS-6 (execution ≠ dialogue).** This is the proposal's central
  organizing principle (§1's driver/admission discriminator, §4.4, PL-2's
  discriminator, ICMR-07) and is stated and applied consistently
  throughout — including correctly citing that grilling-class packages
  already moved to skills under this exact rule.
- **§7 and §10.1–10.2 specifically** (the contract's named sections)
  against the hard boundary. §10.1 (ICM-R0) states "No code changes."
  §10.2 (ICM-R1) explicitly repeats "No src/, API, journal, TUI, backend,
  or workflow grammar changes." §7.1's canonical bounded-judgment source
  (`.sergeant/common/contexts/bounded-judgment.md`) reuses the existing
  `@@name` shared-context resolution rule already in AGENTS.md's routing
  table, requiring no new engine grammar. §7.2/§7.3's authority-envelope
  and bounded-judgment sections are prose added to `CONTEXT.md` files,
  which the engine already passes through to the actor as unstructured
  context (§3.2, ICMR-F2) rather than parsing — adding sections there is
  not a workflow.toml grammar change. §7.8 explicitly disclaims engine
  enforcement of completion boundaries. §14.9's `requires_ask` reference is
  to an existing `workflow.toml` field
  (`src/domain/workflow.rs:247,578,1033,1108`, present on `main` today,
  not new grammar). None of §7's required shapes, read on their own,
  requires an engine change to be enforceable — the one place the
  boundary actually breaks is §10.4's software-change item (F1), which is
  outside §7/§10.1–10.2 but squarely inside the same ICM-R0–ICM-R4 window
  the boundary governs.
- **ADRs 0001–0011.** None constrains procedural/content placement in a
  way this proposal's ladders touch. 0001–0004 (platform/durability/
  cross-platform) and 0006–0007 (harness passthrough, actor execution
  model) are Rust-runtime and environment-contract decisions the proposal
  doesn't propose changing. 0005 (gating becomes a dispatched Work) and
  0009 (auto-spawn never on observation) describe current behavior the
  proposal's findings are consistent with (e.g., ICMR-F1's fresh-
  execution-per-stage claim matches 0007's actor model) but neither ADR is
  contradicted by anything in §3–§10. 0010/0011 (bare `sgt` homepage,
  dashboard deletion) are unrelated to procedural placement.
- **PL/J ladder internal consistency against the current ICM ladder**
  (`.sergeant/workflows/repo-to-icm/_config/icm-ladder.md`). The current
  ladder is a first-match, stop-at-first-yes classifier over three
  representations (`agents-invariant`, `workflow`, `stage`, plus helper/
  shared-helper below what I read). The proposal's PL-0…PL-7 explicitly
  extends rather than replaces this shape (ICMR-04, "extend the existing
  decomposition method; do not create an unrelated classifier"), preserves
  the same first-matching-rung discipline (§5.1: "Ask the rungs in order
  and stop at the first one that holds"), and preserves the existing
  ladder's reimplementation test for the stage rung (§5.7 restates it
  near-verbatim). No conflict found between the two documents' rules.

## Summary

One error-severity finding: **F1**, the proposal's own full-corpus
reconciliation scope (§10.4) names the embedded `software-change` workflow
— physically resident under `src/workflows/software-change/` and compiled
in via `include_str!` — as a reconciliation subject, which contradicts the
proposal's own hard boundary and Acceptance Contract item 33 forbidding
`src/` changes through ICM-R4. This needs an explicit resolution (carve an
exception, or defer the package) before ICM-R3 can execute as scoped.

One warning-severity finding: **F2**, two of §17's 25 Ponytail rung
citations (ICMR-01, ICMR-22) don't match the rung they cite, though the
underlying decisions are sound — a labeling fix, not a substantive
reversal, but one worth making before other packages' decision registers
copy the pattern.

Every other invariants surface I checked — NORTH-STAR's R-NS-1 through
R-NS-6, AGENTS.md's routing doctrine, all twelve current ADRs including
0012, and the placement ladder's consistency with the existing ICM
ladder — holds without a confirmed violation.
