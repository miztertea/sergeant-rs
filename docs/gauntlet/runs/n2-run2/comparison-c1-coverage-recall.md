# Comparison C1 — Source Coverage, Behavioral Recall, Behavioral Precision

N2 run2 (`docs/gauntlet/runs/n2-run2/`) vs the frozen reference corpus
(`reference-corpus/`, `FROZEN.md` v1: 979 units, 34 draft packages). Governed
by `docs/gauntlet/contracts/N2.md` §9.8–§9.9 and
`reference/proposal-next-iteration-icm-workflows.md` §9.9 (dimension
definitions) and §22.2 (success criterion: no reference behavior with
safety/identity/recovery/delivery/human-decision consequence silently
absent).

**Ground rule enforced throughout:** the generated run's own coverage
accounting says it reached 18 (see §0 — actually 16, corrected below) of 136
`decompose`-dispositioned files. Everything below is scoped to that covered
subset unless explicitly labeled "raw coverage," which reports the
uncovered 118/136 honestly as absence-of-attempt, not as a recall failure.

---

## §0. Correction to the run's own coverage claim: 16 files reached, not 18

`90-reconcile/output/measurement-package.md` states "108 units recorded
(`BU-0001`–`BU-0108`), spanning **18** of 136 `decompose` files (`AGENTS.md`,
`README.md`, and all 14 files of the `bin: fleet dispatch & lifecycle`
partition)" and "118 of 136 ... not reached." The named file list is
`AGENTS.md` (1) + `README.md` (1) + the 14 named `bin/sgt-*` files = **16**,
not 18; correspondingly `136 − 16 = 120` files not reached, not 118. This
is not a rounding dispute — it is independently verifiable three ways, all
agreeing on 16:

1. **Direct enumeration of `20-harvest/output/behavior-units.ndjson`** (the
   authoritative extraction artifact, 108 records): exactly 16 distinct
   `source.path` values —
   `AGENTS.md` (23 units), `README.md` (17), `bin/sgt-ack-response` (5),
   `bin/sgt-cleanup` (5), `bin/sgt-dag-dispatch-hook` (3),
   `bin/sgt-dag-run` (5), `bin/sgt-dispatch` (10), `bin/sgt-drain` (7),
   `bin/sgt-drain-force` (2), `bin/sgt-interactive-worker` (5),
   `bin/sgt-notify` (4), `bin/sgt-recover` (4), `bin/sgt-respond` (6),
   `bin/sgt-undrain` (2), `bin/sgt-wake` (5), `bin/sgt-watch` (5); sums to
   108. `30-normalize/output/behavior-units.normalized.ndjson` reproduces
   the identical 16-file/108-unit breakdown record-for-record.
2. **`10-inventory/output/inventory.md`'s own partition listing** (line 113):
   `bin: fleet dispatch & lifecycle (14)` names exactly the 14 files above;
   its "Root files (5)" section (lines 79–89) lists only `AGENTS.md` and
   `README.md` as `decompose` among the 5 root files (`.gitignore` and
   `Dockerfile.test` are `helper-evidence`, `LICENSE` is
   `reference-only`) — so there is no "+2 root" beyond those two.
3. **The measurement package's own arithmetic is self-contradictory**: the
   file list it prints sums to 16, but the headline number and the
   complementary "not reached" count (118) both assume 18. `118 + 18 = 136`
   is internally consistent with itself but not with the 16-file list two
   sentences earlier.

**This report uses the verified figure: 16 files, 108 units, 120 files not
reached.** This is itself a precision-relevant finding about the run's
self-measurement, not a hostile nitpick — see §3.4.

---

## §1. Raw coverage (both scales, unconditioned)

| Metric | Generated (verified) | Reference (frozen) |
|---|---:|---:|
| Files with ≥1 extracted unit | 16 | up to 139 traced (`decompose` set = 136 per N1) |
| Files/136 `decompose` set | 16/136 (11.8%) | 136/136 (100%, by construction) |
| Behavior units | 108 | 979 |
| Units as % of reference scale | 108/979 (11.0%) | — |

Both fractions land near 11–12%, consistent with each other and with the
run's own honest "not reached" framing in
`grammar-pressure.ndjson`'s `20-harvest` engine-gap record (one actor turn
covering 16/136 files, citing the coverage note directly as the engine
pressure evidence). **Everywhere below, "coverage," "recall," and
"precision" are measured only within the 16-file/108-unit scope actually
attempted** — per the task's ground rule, the 120 unreached files are a
coverage gap, not a recall failure, and are not double-counted as misses.

---

## §2. Source coverage within the 16 covered files

For each covered file, reference units citing that exact `source.path`
(prefix `reference/sergeant-upstream/` stripped) vs generated units citing
the same path:

| File | Ref units in file | Gen units in file | Ref matched (≥1 gen unit citing overlapping quote/statement) |
|---|---:|---:|---:|
| `AGENTS.md` | 67 | 23 | 35/67 (52%) |
| `README.md` | 36 | 17 | 18/36 (50%) |
| `bin/sgt-ack-response` | 4 | 5 | 4/4 (100%) |
| `bin/sgt-cleanup` | 8 | 5 | 1/8 (13%) |
| `bin/sgt-dag-dispatch-hook` | 1 | 3 | 1/1 (100%) |
| `bin/sgt-dag-run` | 1 | 5 | 0/1 (0%) |
| `bin/sgt-dispatch` | 6 | 10 | 2/6 (33%) |
| `bin/sgt-drain` | 2 | 7 | 1/2 (50%) |
| `bin/sgt-drain-force` | 3 | 2 | 1/3 (33%) |
| `bin/sgt-interactive-worker` | 10 | 5 | 2/10 (20%) |
| `bin/sgt-notify` | 4 | 4 | 3/4 (75%) |
| `bin/sgt-recover` | 5 | 4 | 2/5 (40%) |
| `bin/sgt-respond` | 6 | 6 | 3/6 (50%) |
| `bin/sgt-undrain` | 1 | 2 | 1/1 (100%) |
| `bin/sgt-wake` | 5 | 5 | 3/5 (60%) |
| `bin/sgt-watch` | 6 | 5 | 1/6 (17%) |
| **Total** | **165** | **108** | **78/165 (47.3%)** |

Every one of the 16 files produced at least one traceable unit on both
sides — no covered file is a zero-unit gap. But the reference decomposes
this same 16-file scope into **165** distinct behaviors while the generated
run produced **108**, and matching them one-for-one shows source coverage
is uneven by file: `bin/sgt-ack-response`, `bin/sgt-dag-dispatch-hook`, and
`bin/sgt-undrain` are fully covered (small files, few reference units), but
`bin/sgt-cleanup` (13%), `bin/sgt-dag-run` (0%), `bin/sgt-watch` (17%), and
`bin/sgt-interactive-worker` (20%) are the weakest — notably the four files
where the reference corpus itself found the most granular, safety-critical
behavior (see §3.2).

---

## §3. Behavioral recall within the 16 covered files

**Method:** every reference unit in the 16-file scope was matched against
every generated unit in the same file by `max(quote-text word-overlap,
statement word-overlap)` (Jaccard over lowercased content words), threshold
0.25, hand-spot-checked against borderline cases by reading the source
passage directly (script and detail retained in this run's evidence; ids
below are exact). This is a source-citation match, not exact-decomposition
match — §9.9 explicitly asks for recall "regardless of exact workflow
grouping," so one generated unit legitimately matching several finer-grained
reference units (e.g. `BU-0017` matching four reference units
`BU-P1-035`–`BU-P1-038`, all citing the same AGENTS.md sentence) counts as
recovered, not partial.

**Result: 78/165 reference units recovered (47.3%); 87/165 missed.**

### 3.1 Matched (78, by file — reference id → generated id)

- `AGENTS.md` (35/67): `BU-P1-001→BU-0001`, `002→002`, `003→003`, `007→002`,
  `011→004`, `013→005`, `017→006`, `020→007`, `021→008`, `022→009`,
  `023→010`, `024→010`, `025→011`, `026→011`, `027→012`, `029→013`,
  `030→014`, `032→015`, `034→016`, `035→017`, `036→017`, `037→017`,
  `038→017`, `039→018`, `040→018`, `041→019`, `042→019`, `043→019`,
  `048→020`, `049→021`, `056→007`, `057→022`, `058→023`, `059→023`,
  `060→023`.
- `README.md` (18/36): `BU-P1-072→BU-0028`, `073→029`, `075→030`, `076→031`,
  `077→032`, `078→033`, `079→033`, `080→034`, `084→035`, `085→036`,
  `087→037`, `088→038`, `089→038`, `090→039`, `091→026`, `092→026`,
  `096→027`, `097→040`.
- `bin/sgt-ack-response` (4/4): `BU-P6-031→085`, `032→088`, `033→086`,
  `034→087`.
- `bin/sgt-cleanup` (1/8): `BU-P6-135→063`.
- `bin/sgt-dag-dispatch-hook` (1/1): `BU-P6-016→052`.
- `bin/sgt-dispatch` (2/6): `BU-P6-123→041`, `125→050`.
- `bin/sgt-drain` (1/2): `BU-P6-064→068`.
- `bin/sgt-drain-force` (1/3): `BU-P6-039→071`.
- `bin/sgt-interactive-worker` (2/10): `BU-P6-107→100`, `111→102`.
- `bin/sgt-notify` (3/4): `BU-P6-027→105`, `028→107`, `029→108`.
- `bin/sgt-recover` (2/5): `BU-P6-071→075`, `072→077`.
- `bin/sgt-respond` (3/6): `BU-P6-076→083`, `077→081`, `078→079`.
- `bin/sgt-undrain` (1/1): `BU-P6-015→073`.
- `bin/sgt-wake` (3/5): `BU-P6-096→095`, `097→099`, `099→097`.
- `bin/sgt-watch` (1/6): `BU-P6-104→093`.

### 3.2 Missed (87) — and which carry §22.2-relevant consequence

The 87 misses are not uniform. A large fraction of the `bin/*` misses are
specifically the safety/identity/recovery/delivery-consequence class §22.2
asks about by name. Citing statement + reference id + generated file's
absence (verified: no generated unit in the same file mentions the
underlying mechanism — checked by keyword grep across all 108 generated
`statement`/`notes` fields, not just the threshold match):

- **PID-reuse verification before signalling a process** — `BU-P6-040`
  (`bin/sgt-drain-force`, L139-157: "force-stop verifies the process's
  recorded start time still matches its actual start time, so a PID that
  has since been reused ... is recognized and treated as already-gone
  rather than being killed") and `BU-P6-138` (`bin/sgt-cleanup`, L153-160,
  same guarantee before cleanup signals a process). No generated unit in
  either file's 5-7 records mentions start-time/PID-reuse verification at
  all; `bin/sgt-cleanup`'s 5 generated units (`BU-0059`–`BU-0063`) cover
  task-id validation and callback-sync ordering, not process-kill safety.
- **Evidence preservation before destructive action** — `BU-P6-139`
  (`bin/sgt-cleanup`, L380-386/787-796: cleanup "refuses to retire fleet
  state at all while an unfinishable response is still outstanding, so
  retiring a task can never silently discard evidence") and `BU-P6-140`
  (same file, L2621-2642: a durable, resumable cleanup-phase record is
  published before removal begins so an interrupted cleanup can be safely
  retried). Neither concept — pre-removal durable phase record, or
  refusing retirement while evidence is outstanding — appears in the
  generated `bin/sgt-cleanup` set.
- **Exactly-once delivery / notification-lease convergence** — `BU-P6-073`,
  `BU-P6-079` (`bin/sgt-recover`, `bin/sgt-respond`: an outstanding
  notification action-lease must converge through the one shared finalizer
  using only the agent's own completion proof before recovery/response
  proceeds) and `BU-P6-113`/`BU-P6-114` (`bin/sgt-interactive-worker`: the
  lease is settled at a single unified exit boundary across every terminal
  status, and the readiness gate for delivery is bounded with a specific
  non-fabrication guarantee on timeout). The generated `bin/sgt-respond`
  set (`BU-0079`–`BU-0084`) covers pane-ack timeouts and pending-consumption
  guards but not the lease-convergence-before-action invariant itself.
- **Orphaned-worker detection on a hazardous transition** — `BU-P6-103`
  (`bin/sgt-watch`, L561-567: a status transitioning to `done` while the
  worktree's result file is empty is refused and reclassified `orphaned`)
  and `BU-P6-115` (`bin/sgt-interactive-worker`, L490-495: exit is
  `orphaned` unless genuinely terminal with substantiating evidence). Not
  present in generated `bin/sgt-watch` (`BU-0090`–`BU-0094`, which cover
  stall detection, dagr reporting, and pane-recycle idempotency) or
  `bin/sgt-interactive-worker` (`BU-0100`–`BU-0104`, harness-registry
  validation and drain handoff, not exit-classification).
- **Model/variant pin honored on resume, not silently substituted** —
  `BU-P6-108` (`bin/sgt-interactive-worker`, L44-49: a worker resumed by
  response/recovery/wake always runs the exact pinned tuple; a tuple the
  resuming harness cannot honor is a terminal failure, not a silent
  fallback to the ambient default). Generated `BU-0100`'s closest content
  is harness-registry validation at *dispatch* time, not the *resume-time*
  re-honoring guarantee — a distinct behavior, genuinely absent.
- **Fail-closed unmet-vs-escalate distinction for wake conditions** —
  `BU-P6-098` (`bin/sgt-wake`: "unmet" — may still resolve later — is
  distinguished from "escalate" — permanently unsatisfiable, so retrying
  would be dishonest). Generated `bin/sgt-wake` set has a deadline-timeout
  unit (`BU-0096`) but not this two-state distinction.
- **AGENTS.md-level guardrail against destructive/risk-accepting shortcuts**
  — `BU-P1-050` ("Standing authorization may remove repetitive dispatch
  confirmation, but never authorizes risk acceptance, gate skipping, force
  operations, secret exposure, or destruction of preserved state.") and
  `BU-P1-016` ("Never use direct mode to edit several repositories in one
  checkout, or to bypass repository instructions, task ownership, review
  independence, or shipping gates."). Neither rule appears anywhere in the
  23 generated `AGENTS.md` units; `BU-0014`'s confirm-only-on-risk unit is
  adjacent but covers *when to ask*, not the standing-authorization
  boundary or the direct-mode scope guardrail.
- **Rollback scoping precision on dispatch failure** — `BU-P6-127`
  (`bin/sgt-dispatch`, L324-335/974-976: a managed coordinator pane the
  invocation itself created is killed on failure, but a pre-existing
  selected pane is never touched, and the rollback trap disarms once every
  repo has dispatched so a later unrelated failure can't retroactively kill
  a now-owned pane). Generated `bin/sgt-dispatch`'s 10 units include
  `BU-0042` (`--adopt-branch`, matched loosely by the heuristic but is
  actually about resuming branches, not rollback) — the rollback-scoping
  guarantee itself is absent.

Also missed but lower-consequence (structural/orientation, not
safety-critical): the entire AGENTS.md "procedural skills" routing table —
6 distinct skill-trigger rules (`BU-P1-132`–`BU-P1-137`: load-project,
cross-repo-work, dispatch, wiki, sergeant-help, sergeant-setup triggers) —
is uncaptured as a set; only the toolbelt-fallback rule from the same
region (`BU-P1-020`) was extracted. And README.md's entire orientation/
genesis section (`BU-P1-062`–`BU-P1-067`: "what Sergeant is," the
firstmate-lineage narrowing, the three-directory mental model) is fully
missed — the generated README.md units all come from the later
operational sections (no-mistakes, review routing, drain locking), meaning
whatever turn budget hit README.md read the back half but not the front.

### 3.3 Extra generated units (45) — precision-relevant, judged in §4

45 of 108 generated units in scope did not match any reference unit by the
above method. Per-file counts: `AGENTS.md` 0, `README.md` 2 (`BU-0024`,
`BU-0025`), `bin/sgt-cleanup` 4, `bin/sgt-dag-dispatch-hook` 2,
`bin/sgt-dag-run` 5, `bin/sgt-dispatch` 8, `bin/sgt-drain` 6,
`bin/sgt-drain-force` 1, `bin/sgt-interactive-worker` 3, `bin/sgt-notify` 1,
`bin/sgt-recover` 2, `bin/sgt-respond` 3, `bin/sgt-undrain` 1,
`bin/sgt-wake` 2, `bin/sgt-watch` 4. Spot-verification (§4) against source
found every sampled "extra" unit to be a genuine, correctly-cited, finer- or
differently-grained behavior the reference corpus simply didn't extract at
that boundary (e.g. `BU-0059`–`BU-0062` are real `sgt-cleanup` guard clauses
— task-id path-traversal rejection, symlink rejection, callback-sync
ordering — that the reference's 8 `sgt-cleanup` units genuinely don't cite).
These read as **legitimate alternate decomposition** (§9.8 category), not
generator invention — see §4 for the evidence.

### 3.4 Recall in aggregate

| Scope | Ref units | Matched | Recall |
|---|---:|---:|---:|
| `AGENTS.md` + `README.md` (2 root files) | 103 | 53 | 51.5% |
| 14-file `bin` fleet-dispatch partition | 62 | 25 | 40.3% |
| **All 16 covered files** | **165** | **78** | **47.3%** |

Root-file recall (52%) and bin-partition recall (40%) are both well under
100%, and the `bin` misses skew toward exactly the consequence classes
§22.2 names — PID-reuse safety, evidence-before-destruction, exactly-once
delivery, orphan detection, pin-honoring on resume, fail-closed
condition-state distinctions. **Within its self-declared covered scope,
this run does not meet the §22.2 bar cleanly**: several reference behaviors
with recovery/delivery/identity consequence are silently absent (not
flagged as `# AMBIGUOUS`, not recorded as a `gold miss` or engine-gap — they
simply don't appear), concentrated in `bin/sgt-cleanup` (7/8 missed),
`bin/sgt-interactive-worker` (8/10 missed), `bin/sgt-watch` (5/6 missed),
and `bin/sgt-recover` (3/5 missed).

---

## §4. Behavioral precision within the 16 covered files

**Method:** the full 108-unit generated set was checked programmatically
(not just sampled) for two things against `reference/sergeant-upstream` at
the pinned SHA (`f430cfd4f90174a98adbd7abebbece6303817929`, present in this
checkout): (a) does `sha256(source.quote) == source.quote_hash`, and (b) is
`source.quote` a literal contiguous substring of the named file. Then ≥15
units were read by hand across every file category to judge whether the
`statement` is actually supported by the `quote` (the qualitative half of
"source-supported... rather than invented from generic software-development
priors," §9.9) — not just whether the quote hash checks out.

### 4.1 Full-set hash/verbatim check (108/108)

- **Verbatim presence: 108/108 (100%).** Every generated unit's `quote`
  field is a literal substring of its cited `source.path` at the pinned
  SHA. Zero units cite a quote that isn't actually in the named file — no
  fabricated citations found anywhere in the 108-unit set.
- **Hash self-consistency: 92/108 (85.2%) — 16/108 (14.8%) FAIL.** 16
  records' stored `quote` field does not hash to their own declared
  `quote_hash`: `BU-0017`, `BU-0023`, `BU-0024`, `BU-0026`, `BU-0027`,
  `BU-0036`, `BU-0037`, `BU-0038`, `BU-0068`, `BU-0070`, `BU-0083`,
  `BU-0086`, `BU-0089`, `BU-0090`, `BU-0092`, `BU-0093`. Every one of these
  16 has a `quote` field of **exactly 500 characters**, truncated mid-word
  (e.g. `BU-0070`'s stored quote ends `"...reserved for drain admission
  lock state'` — the source line actually continues `...lock state)"` per
  `reference/sergeant-upstream/bin/sgt-drain` line 118; `BU-0036`'s stored
  quote ends `"...so the contract and the router ca` — cut off mid-word
  before "cannot," per `README.md` line 314). This is a systematic
  500-character cap applied to the stored `quote` field sometime after
  `quote_hash` was computed against the full (longer) span — a data-
  integrity defect in the `20-harvest`/`30-normalize` record, not a
  fabrication: the truncated text is still a genuine, correctly-located
  verbatim prefix of the real quote in all 16 cases (confirmed above by
  the verbatim-substring check, which passed for all 16).
- **This directly contradicts the run's own self-check.**
  `90-reconcile/output/measurement-package.md`'s "Behavioral precision"
  section reports "29/29 sampled citations verified cleanly (hash matched,
  quote appeared contiguously at the cited locator)." That claim traces to
  `80-adversarial-review/output/review-summary.md`'s "Citation
  re-verification," which explicitly says it recomputed the hash by
  "reading the full quoted span from the file directly, **not the stored
  `quote` field**, to avoid checking a truncated 500-char prefix and
  hashing the complete span, not just the stored prefix." That methodology
  is sound for confirming the *cited span is real* (which it is — 0
  invention findings, correctly), but by design it never checks
  `hash(stored quote) == stored quote_hash`, so it could not and did not
  catch the exact defect found here. The adversarial-review stage's own
  wording shows it was aware records could be truncated at 500 chars, yet
  `findings.ndjson` recorded 0 findings — this specific, verifiable
  self-consistency defect was not raised as a finding by the run's own
  review stage.

### 4.2 Hand-read semantic spot-check (18 units, all passed)

Read `statement` against `quote` directly for units spanning every covered
file category (root files, small bin utilities, the two largest bin files):
`BU-0001`, `BU-0017`, `BU-0023`, `BU-0032`, `BU-0037` (`AGENTS.md`/
`README.md`); `BU-0041`, `BU-0042`, `BU-0051`, `BU-0059`, `BU-0060`,
`BU-0064`, `BU-0073`, `BU-0085`, `BU-0090`, `BU-0095`, `BU-0100`,
`BU-0105`, `BU-0106` (bin files, including 6 of the "extra"/unmatched
units from §3.3). In every case the `statement` is a tight, accurate
paraphrase of what the `quote` actually says — no case of a statement
asserting a stronger, broader, or generic-best-practices claim beyond what
its cited text supports. Example: `BU-0059` ("A cleanup task-id is
rejected if it is empty, a dot/dot-dot, an absolute path, or contains a
path separator, before any fleet state is touched") is a direct restatement
of the quoted `case "$TASK_ID" in ""|"."|".."|/*|*/*) ...` guard clause in
`bin/sgt-cleanup` — no invention.

### 4.3 Randomized verbatim/hash spot-check (20 additional units)

A `random.seed(42)`-selected sample of 20 units across the 108 (disjoint
from most of §4.2) was independently re-verified for hash and verbatim
match, surfacing 18/20 clean and reproducing 2 of the 16 truncation
failures already counted in §4.1 (`BU-0036`, `BU-0070`) — consistent with,
not additional to, the systematic count above.

### 4.4 Precision verdict

**Zero invention findings** across 108/108 verbatim checks and 38 hand- and
random-sampled statement/quote reads (well over the ≥15 required) — every
generated unit in the covered scope is a real, correctly-located citation,
and every read statement is faithful to its quote. Precision on the
"invented from generic priors" axis that §9.9 actually asks about is
**high (108/108 verbatim, 0/38 sampled inventions)**. Separately, and not
conflatable with invention, **14.8% of records (16/108) carry a data-
integrity defect** (truncated `quote` field inconsistent with its own
`quote_hash`) that the run's own adversarial-review stage's methodology
was structurally unable to catch, and that its self-reported "29/29 ...
hash matched" precision claim in `measurement-package.md` therefore
overstates.

---

## §5. Summary

| Dimension | Finding |
|---|---|
| Raw coverage | 16/136 `decompose` files (11.8%), 108/979 units on scale (11.0%) — **not** 18/136 as self-reported (§0) |
| Source coverage (within 16 files) | Every covered file yields ≥1 unit; per-file ref-recovery ranges from 0% (`bin/sgt-dag-run`) to 100% (`bin/sgt-ack-response`, `bin/sgt-dag-dispatch-hook`, `bin/sgt-undrain`) |
| Behavioral recall (within 16 files) | 78/165 reference units recovered (47.3%); 87 missed, concentrated in `bin/sgt-cleanup`, `bin/sgt-interactive-worker`, `bin/sgt-watch`, `bin/sgt-recover` and skewed toward PID-reuse safety, pre-destruction evidence preservation, exactly-once delivery-lease convergence, orphan detection, and pin-honoring-on-resume — the exact consequence classes §22.2 names |
| Behavioral precision (within 16 files) | 108/108 verbatim-cited, 0/38 sampled inventions — high; but 16/108 (14.8%) records fail self-consistency (`hash(quote) ≠ quote_hash`) due to a 500-char truncation defect the run's own review stage's methodology could not detect and did not flag |

**Bottom line for §22.2 within the covered scope:** coverage is honest
(nothing silently skipped — the 120 unreached files are explicitly
recorded as not-reached, not silently dropped), and what was extracted is
well-cited (no invention). But recall inside the attempted 16 files is
well under half, and the misses are not evenly distributed across
low-stakes and high-stakes content — they concentrate in exactly the
safety/identity/recovery/delivery mechanisms §22.2 treats as the bar for
"cannot be silently absent." This run's own internal self-measurement
(`measurement-package.md`) does not detect this because it explicitly
scopes itself to five dimensions computable without the reference corpus
and defers behavioral recall to this comparison — consistent with the
contract, but meaning no one inside the run itself checked this until now.
