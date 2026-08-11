# Adjudication log — `90-reconcile`, run 01KZQRGZE32RQ79KT82XTB9MV2

Every finding in `../80-adversarial-review/output/findings.ndjson` (6
findings: 1 high, 4 medium, 1 low), disposed per
`references/reconciliation-method.md` §1. Repairs applied directly to the
affected files are cited by exact edit; nothing is silently rewritten.

## AF-0001 (high, boundary-honesty) — accept

**Finding:** `20-harvest/output/partition-ledger.md`'s "Orchestrator ruling
on the census mismatch" section quoted specific `reference-corpus/` content
(a source-inventory line number, a disposition, and an N1 adjudication id)
to justify excluding `.agents/skills/diagnosing-bugs/scripts/
hitl-loop.template.sh` from harvest. Verified independently: the quoted
text is real and is the only hit across `00-contract` through `70-lint`
that names actual reference-corpus *content* rather than restating the
exclusion policy's own wording.

**Ruling:** Accept. The ledger's own hedge (this stage's actor did not
consult `reference-corpus/` itself) is true but does not cure the defect —
the specific content is now written into this run's own committed
evidence regardless of who looked at the excluded directory first, and it
was used to decide this run's own generative output (excluding a file from
harvest). This is a real blindness-boundary violation in the artifact, not
a false positive.

**Repair applied:** `20-harvest/output/partition-ledger.md`, the
"Orchestrator ruling on the census mismatch" section — the specific
`reference-corpus/source-inventory.md` line number, disposition, and "N1
adjudication A11" citation are redacted from the quoted ruling text and
replaced with an explicit redaction note attributing the change to this
adjudication. The ruling's **operative disposition is unchanged**: the file
remains excluded from harvest, ruled in run 3's favor, per L9 (binding on
this stage) — only the specific reference-corpus content citation is
removed from this run's own committed evidence.

## AF-0002 (medium, boundary-honesty) — accept

**Finding:** `00-contract/output/contract.md` §3 explicitly rules this run
carries no measurement framing and no answer-key directory is present, yet
`20-harvest/output/partition-ledger.md`'s census-mismatch ruling frames the
same run as an actively graded comparison ("the FROZEN reference corpus",
"N1 adjudication", "for the comparison record") — an internal contradiction
no stage in between reconciles.

**Ruling:** Accept. Verified: the contract's own §3 text and the ledger's
framing language are both real and do contradict each other; nothing
downstream of `00-contract` revises or overturns its no-measurement-framing
ruling.

**Repair applied:** `20-harvest/output/partition-ledger.md`, same section as
AF-0001 — the disposition line's "for the comparison record" framing is
removed and the disagreement is re-described as an ordinary
generator-vs-prior-attempt disposition disagreement; a note is added citing
`00-contract/output/contract.md` §3's explicit no-measurement-framing
ruling and flagging that the orchestrator ruling's original wording
contradicted it. The file's harvest exclusion itself is unchanged.

## AF-0003 (medium, structural-self-consistency) — accept

**Finding:** `10-inventory/output/inventory.md` states a `decompose` count
of 83 (checksum 83+80+16+0=179 ✓), but `20-harvest/output/
behavior-units.ndjson` covers only 82 distinct `source.path` values — the
one-file gap is exactly the file AF-0001's ruling excluded, and
`inventory.md`'s own stated count was never annotated to reflect that its
downstream harvest yield is one short.

**Ruling:** Accept. Verified independently (recount of distinct
`source.path` values = 82, matching the review's own recount). `10-inventory`'s
83 is a correct report of that stage's own independent classification and
is not itself wrong, but a reader taking the count at face value has no
signal that harvest only reached 82 for a documented, ruled reason.

**Repair applied:** `10-inventory/output/inventory.md`, directly below the
"Disposition counts" checksum line — added a note cross-referencing the
AF-0001-adjudicated exclusion and pointing to `partition-ledger.md`'s
ruling section. The 83 count itself is unchanged (it remains this stage's
own correct, independent classification); only a cross-reference is added.

## AF-0004 (medium, structural-self-consistency) — accept

**Finding:** `20-harvest/output/consequence-class-sweep.md` has 82 rows (no
blank cells, no duplicates — matches the actual harvest), one short of
`inventory.md`'s 83 `decompose` files. The missing row is the same file
excluded by the AF-0001 ruling — the checklist names a missing row as a
finding in its own right, independent of AF-0003's count mismatch.

**Ruling:** Accept. Verified independently (82 data rows, 82 distinct
first-column paths, 0 blank cells, no `hitl-loop` row). The sweep correctly
covers every file this stage actually harvested; the row is absent because
the file itself was never harvested, per the same ruling AF-0001/AF-0003
already explain.

**Repair applied:** `20-harvest/output/consequence-class-sweep.md`,
appended a note after the table cross-referencing the ruling and stating
plainly that the missing row is a known, ruled exclusion rather than an
uncovered gap in the sweep's own coverage.

## AF-0005 (low, structural-self-consistency) — reject

**Finding:** `partition-ledger.md` uses run 3's P1–P21 partition naming
throughout; `inventory.md` uses this run's own A–S naming for the same
file census. Zero overlap between the two label sets.

**Ruling:** Reject. Verified the naming divergence is real (zero overlap,
confirmed by direct comparison), but it does not hold up as a defect on
inspection: both files' own text discloses and explains it — the ledger's
"Scheme provenance (run 4)" section states plainly that this resumed run
harvests by run 3's own partition scheme, that partitioning is actor
judgment (not deterministic), and that the resume reconciles at the file
level, not the partition-label level. Requiring two independently-run
`10-inventory` partitionings across a resume boundary to use identical
partition names is not this workflow's method — it would in fact require
inventing a correspondence neither this run's own contract nor
`references/*` calls for. No repair applied; the existing disclosure
already carries the explanation a reader needs.

## AF-0006 (medium, invention) — accept

**Finding:** `30-normalize/output/behavior-units.normalized.ndjson`
(BU-0040, BU-0041) cite `source.locator: "AGENTS.md L150-153"`, but their
stored `quote` (hash-verified, genuine and contiguous in the file) actually
spans AGENTS.md L150–157 — the locator recovers less than half of what is
actually quoted, failing evidence-policy.md's "precise enough to re-open
the exact evidence" bar. Not fabrication (quote and hash are both genuine),
but a locator defect.

**Ruling:** Accept. Verified directly against `reference/sergeant-upstream/
AGENTS.md`: lines 150–157 (not 150–153) are the actual contiguous span
matching the stored `quote` field for both records, byte-for-byte.

**Repair applied:** `source.locator` corrected from `"AGENTS.md L150-153"`
to `"AGENTS.md L150-157"` for BU-0040 and BU-0041 in both
`20-harvest/output/behavior-units.ndjson` (where the defect originates) and
`30-normalize/output/behavior-units.normalized.ndjson` (the finding's
named target) — the same wrong locator was carried unchanged from harvest
into normalize, so both are corrected for consistency. `statement`,
`quote`, and `quote_hash` are untouched (already genuine and correct).

## Summary

| Finding | Severity | Axis | Disposition |
|---|---|---|---|
| AF-0001 | high | boundary-honesty | accept — reference-corpus citation redacted, disposition preserved |
| AF-0002 | medium | boundary-honesty | accept — measurement-framing language removed, cross-referenced to contract §3 |
| AF-0003 | medium | structural-self-consistency | accept — cross-reference note added to inventory.md |
| AF-0004 | medium | structural-self-consistency | accept — cross-reference note added to consequence-class-sweep.md |
| AF-0005 | low | structural-self-consistency | reject — disclosed, licensed divergence, not a defect |
| AF-0006 | medium | invention | accept — locator corrected in both harvest and normalize ndjson |

5 accepted (repaired in place), 1 rejected (no repair — reasoned above), 0
parked. No accepted finding was an Axis-3 engine-gap refutation (Axis 3 had
zero records to re-attempt per the review), so no classification record's
`representation` changes as a result of this adjudication.
