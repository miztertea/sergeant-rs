# Cold-segment compression research — Rule C ladder rung 1 (2026-08-11)

Status: research findings, informing the amended Rule C design ladder in
[retention-design-ruling-draft-2026-08-11.md](retention-design-ruling-draft-2026-08-11.md)
("compress cold segments in place (R2)" fires before any snapshot machinery).
This note answers three questions: **which codec**, **which Rust crate**, and
**what decompress-on-rebuild costs against the measured 54.6k events/s rate**.

Placement note: the invoking workflow declares its artifact in an
`output/README.md` that is not materialized on this surface, so this file is
placed per the repository's evident note-keeping convention —
`docs/gauntlet/notes/<topic>-<date>.md`, alongside the ruling it serves.

Method: claims are traced to (a) first-party measurements run for this note on
real Sergeant journal data (marked **[M]**, method in §1), (b) repo source and
measurement docs (cited by path), or (c) primary external sources (RFC, project
READMEs/benchmarks, crates.io registry API, NVD), footnoted at the bottom.

## 0. Grounding facts (repo-sourced)

| Fact | Value | Source |
|---|---|---|
| Segment rotation threshold | 8 MiB (`DEFAULT_SEGMENT_MAX_BYTES = 8 * 1024 * 1024`) | `src/runtime/journal.rs:52` |
| Journal format | append-only segmented NDJSON, one complete event per line, replay in seq order across segments | `src/runtime/journal.rs:1-16` |
| Crash-tail handling | torn trailing line quarantined to `<segment>.partial` on open; truncation applies to the segment being opened for append | `src/runtime/journal.rs:14-16` |
| Measured rebuild rate | 54.6k events/s at 50k events (container baseline 29.2k) | `docs/perf/baseline-cerberus-2026-08-11.md` §S5 |
| Steady event size | ~571 B/event (container ~549 B/event) | `docs/perf/baseline-cerberus-2026-08-11.md` §S5 |
| Rebuild trigger | measured rebuild-on-start > 30 s ≈ ~1.6M events at 54.6k ev/s | ruling, Rule C amendment |
| 50k-event rebuild footprint | 1.7 s / 178 MB RSS | ruling, Rule C original draft text |
| Ruling's ratio estimate | "text compresses ~10×" | ruling, Rule C amendment — **measured below at 6.5–7.7× on real data; see §1** |

## 1. First-party measurements on real journal data [M]

Host: 12th Gen Intel Core i7-12800H, Linux; `zstd` CLI v1.5.7, `xz` (XZ Utils)
5.8.3, `gzip` 1.14 (versions from `--version` output). Corpus A is the largest
real captured Sergeant journal in the repo:
`docs/gauntlet/runs/runB/attempt-2/journal_full.ndjson`, 185,575 B, 71 events —
the "186 kB largest run ever" the ruling itself cites. Throughputs for zstd are
from `zstd -b` (in-memory benchmark, no process/pipe overhead).

### 1a. Corpus A (real journal, 186 kB) — ratio and speed

| Codec / level | Compressed size | Ratio | Compress | Decompress |
|---|---|---|---|---|
| zstd -1 | 28,630 B | 6.48× | 919 MB/s | 3,439 MB/s |
| zstd -3 (default¹) | 27,544 B | 6.74× | 629 MB/s | 2,993 MB/s |
| zstd -9 | 25,059 B | 7.41× | 133 MB/s | 3,410 MB/s |
| zstd -19 | 24,235 B | 7.66× | 7.5 MB/s | 3,020 MB/s |
| gzip -6 | 47,074 B | 3.94× | — | — |
| gzip -9 | 46,941 B | 3.95× | — | — |
| xz -6 | 23,532 B | 7.89× | — | — |
| xz -9e | 23,492 B | 7.90× | — | — |

Readings: (1) zstd decompression is ~3 GB/s on this data *at every level* —
level choice buys ratio at compression-time cost only. (2) Ratio flattens hard
after level ~9 (7.41× → 7.66× for a 17× compression-speed drop). (3) xz's best
ratio beats zstd -19 by only 3% (7.89× vs 7.66×). (4) gzip manages barely half
the ratio of either — see §1b for why. (5) The ruling's "~10×" estimate is
optimistic at this corpus size: **measured 6.5–7.7× on real data**; the ratio
should improve at full 8.4 MB segment scale (more window history to match
against), but trigger arithmetic below uses the measured figures.

### 1b. Corpus B (synthetic 8.4 MB segment) — window-scale effects, upper bound

To probe full-segment scale, 8.4 MB of events were synthesized from Corpus A by
resampling its 71 real events with freshly randomized ULIDs, hex hashes,
timestamps, and renumbered `seq` (2,880 events, seeded PRNG). **Caveat: drawing
8.4 MB from a 71-event pool exaggerates cross-event redundancy, so absolute
ratios here are upper bounds, not forecasts.** What the corpus isolates is
*window reach*:

| Codec / level | Compressed size | Ratio |
|---|---|---|
| zstd -3 | 383,917 B | 21.9× |
| zstd -19 | 337,482 B | 24.9× |
| gzip -9 | 2,267,152 B | 3.7× |
| xz -6 | 334,480 B | 25.1× |

gzip's ratio *does not move* between 186 kB and 8.4 MB (3.95× → 3.7×) while
zstd and xz triple theirs: DEFLATE's 32 KiB window² cannot reach the
cross-event redundancy (repeated schema keys, source blocks, workflow/stage
names) that recurs across a whole segment, which zstd (8 MB recommended
decoder window³) and xz exploit. Journal segments are structurally matched to
a large-window codec and structurally mismatched to gzip.

CLI decompression wall-time on this 8.4 MB corpus (20 iterations, 168 MB raw
emitted): zstd 1.57 GB/s, gzip 414 MB/s, xz 424 MB/s (xz flattered here by the
corpus's inflated redundancy; published realistic xz decompression is ~98–127
MB/s⁴ — use those for planning).

## 2. Codec comparison — published primary-source numbers

For scale-independent reference (Silesia corpus):

| Codec | Ratio | Compress | Decompress | Source |
|---|---|---|---|---|
| zstd 1.5.7 -1 | 2.90 | 510 MB/s | 1,550 MB/s | zstd README benchmark, i7-9700K⁵ |
| zlib 1.3.1 -1 | 2.74 | 105 MB/s | 390 MB/s | zstd README benchmark⁵ |
| brotli 1.1.0 -1 | 2.88 | 290 MB/s | 425 MB/s | zstd README benchmark⁵ |
| lz4 1.10.0 | 2.10 | 675 MB/s | 3,850 MB/s | zstd README benchmark⁵ |
| xz 5.6.3 -6 | 4.31 (23.21%) | 3.0 MB/s | 127 MB/s | lzbench 2.0.1, EPYC 9554⁴ |
| zstd 1.5.6 -1 | 2.89 (34.64%) | 422 MB/s | 1,347 MB/s | lzbench 2.0.1⁴ |
| zlib 1.3.1 -6 | 3.11 (32.19%) | 25 MB/s | 344 MB/s | lzbench 2.0.1⁴ |

Format-property fit for a replayable append-only journal (zstd, from RFC 8878³):

- **Streaming decompression** with a priori bounded memory, content size not
  required up front — matches line-by-line replay through a reader stack.
- **Frame concatenation**: "decompressed content of multiple concatenated
  frames is the concatenation of each frame's decompressed content" — a
  compressed segment stays a valid single file even if written frame-at-a-time.
- **Decoder memory ≈ Window_Size, recommended ≤ 8 MB** — bounded and small
  against the measured 178 MB rebuild RSS at 50k events (§0).
- **Skippable frames** exist if segment metadata ever needs embedding.
- Format is **stable, standardized in RFC 8878**⁵ ³ — replayability of archived
  segments doesn't depend on one library's longevity; an independent pure-Rust
  decoder (ruzstd) exists as a second implementation (§3).
- Seekable/random-access variants (zstd contrib) are **not needed**: replay is
  strictly sequential from seq 1 across segments (`src/runtime/journal.rs`).

Supply-chain note on xz: liblzma upstream tarballs 5.6.0/5.6.1 shipped embedded
malicious code (CVE-2024-3094, CVSS 10.0 Critical)⁶. Recoverable, but a
relevant liability weight for a codec whose ratio edge over zstd -19 measured
3% (§1a).

## 3. Rust crate landscape

Registry facts from the crates.io API on 2026-08-11⁷; in-tree facts from this
repo's `Cargo.lock`.

| Crate | Version | Last publish | Downloads | License | Nature |
|---|---|---|---|---|---|
| `zstd` | 0.13.3 | 2025-02-20 | 352M | MIT | Bindings to bundled libzstd; `stream::read::Decoder` (impl `Read`), `stream::write::Encoder`; `DEFAULT_COMPRESSION_LEVEL = 3`¹ |
| `zstd-sys` | 2.0.16+zstd.1.5.7 | 2025-09-04 | — | MIT/Apache-2.0 | Bundles zstd C 1.5.7 (version embedded in crate version string) |
| `ruzstd` | 0.9.0 | 2026-07-26 | 58M | MIT | Pure-Rust decoder (complete) + encoder "usable, but does not yet reach the speed, ratio or configurability of the original zstd library" — only fastest level implemented⁸ |
| `lz4_flex` | 0.14.0 | 2026-07-14 | 122M | MIT | Pure Rust, "no unsafe by default" (safe-encode/safe-decode), block + frame formats; claimed 2.3–5.5 GiB/s decompression⁹ |
| `flate2` | 1.1.9 | 2026-02-03 | 609M | MIT OR Apache-2.0 | DEFLATE/gzip/zlib as Read/Write streams; **already in this repo's tree** via `libduckdb-sys` and `zip` (`Cargo.lock`) |
| `liblzma` | 0.4.8 | 2026-08-09 | 18M | MIT OR Apache-2.0 | liblzma (xz) bindings, fork of `xz2` |

Fit notes:

- **`zstd`** is the direct path: `Decoder` wraps the segment file reader and
  the existing line-by-line replay reads through it unchanged. Cost: one new
  native-code dependency (zstd C via `zstd-sys`); the crate's last publish is
  18 months old but it tracks a stable C library (1.5.7, current upstream
  release⁵) and is the ecosystem default at 352M downloads.
- **`ruzstd`** cannot be the writer (encoder immature⁸) but is a credible
  *second decoder* for the archived format — useful as a compile-time fallback
  if a pure-Rust-only build of a reader tool is ever wanted, and as evidence
  the format outlives any one implementation.
- **`flate2`** is the only zero-new-dependency option (R2's strongest reading)
  — but §1b shows gzip is structurally mismatched: ~4× ratio ceiling
  regardless of segment size, and 323–414 MB/s decompression⁴ [M].
- **`lz4_flex`** is the pure-Rust performance option: decompression above
  zstd's, but ratio ~2.1× (Silesia⁵) to ~4.4× (small JSON⁹) — it halves or
  worse the disk win, and disk is the entire point of this rung.
- **`liblzma`**: ~3% ratio gain over zstd -19 (§1a) for ~10× slower
  decompression⁴ plus the CVE-2024-3094 history⁶. No case.

## 4. Decompress-on-rebuild cost at the trigger point

Arithmetic anchored to measured figures (§0): rebuild consumes journal text at
**54,600 ev/s × 571 B/ev ≈ 31.2 MB/s**. The 30 s trigger therefore fires at
~1.64M events ≈ **935 MB of raw journal ≈ 111 cold segments** of 8.4 MB.

Cost of decompressing all 935 MB during one rebuild-on-start, single-threaded,
using the *conservative* end of each codec's decompression range (published
figures⁴ ⁵; zstd's local measurements run 2–3× faster than the figure used):

| Codec | Decompress rate used | Added wall time | Share of 30 s budget | Disk at trigger (ratio from §1a) |
|---|---|---|---|---|
| none (status quo) | — | 0 s | 0% | 935 MB |
| zstd -3 | 1,347 MB/s⁴ | **~0.7 s** | **~2.3%** | 139 MB (6.74×) |
| zstd -19 | 1,332 MB/s⁴ | ~0.7 s | ~2.3% | 122 MB (7.66×) |
| lz4 | 3,716 MB/s⁴ | ~0.25 s | ~0.8% | ~213–445 MB (4.4×⁹–2.1×⁵) |
| gzip (flate2) | 323 MB/s⁴ | ~2.9 s | ~10% | 237 MB (3.95×) |
| xz -6 | 127 MB/s⁴ | ~7.4 s | ~25% | 119 MB (7.89×) |

Three corollaries:

1. **zstd decompression is ~43× faster than the rebuild's own consumption rate**
   (1,347 vs 31.2 MB/s), so streamed decode-behind-parse adds low-single-digit
   percent to rebuild wall time; the deserialization/apply path stays the
   bottleneck. The trigger's *event-count* location (~1.6M) barely moves.
2. On a cold page cache the sign can flip: rebuild reads 139 MB from disk
   instead of 935 MB. Any storage slower than ~1.2 GB/s sequential makes the
   compressed path *faster* end-to-end, not slower.
3. xz is the only ratio winner over zstd and it spends a quarter of the entire
   30 s budget to beat zstd -19 by 3 MB at trigger scale. It moves the trigger
   materially; zstd does not.

Compression-side cost (background, once per segment as it goes cold, measured
rates §1a): zstd -3 ≈ **13 ms per 8.4 MB segment**; zstd -19 ≈ 1.1 s. Either is
negligible for an archiver that runs at rotation time, but -19's +0.9
percentage-point ratio gain does not justify 84× the CPU; levels 3–9 are the
sensible band.

## 5. Conclusion for the ladder rung

**zstd, via the `zstd` crate (bundled libzstd 1.5.7), at default level 3
(band 3–9), streaming `Decoder` inserted under the existing replay reader.**
On measured real journal data it delivers 6.7–7.7× disk reduction (939 MB →
~130 MB at trigger scale) for ~2% added rebuild wall time — likely net-negative
added time on non-NVMe storage — with an RFC-standardized format³, bounded
decoder memory, an independent second decoder implementation in pure Rust⁸,
and no interaction with the crash-tail quarantine machinery (only the *active*
segment can have a torn tail, `src/runtime/journal.rs:14-16`; cold segments
are complete and immutable by construction before they are candidates).

Ranked alternatives, should constraints change:
- *Pure-Rust-only build*: `lz4_flex` (frame format) — accept ~2× worse disk.
- *Zero new dependencies*: `flate2` (already in-tree) — accept ~4× ratio
  ceiling (structural, §1b) and ~10% of the rebuild budget.
- *Rejected*: xz/liblzma — 3% ratio gain, 25% of the rebuild budget, CVE-2024-3094 history⁶.

One measurement this note licenses but does not perform (it needs the N4
1M-event journal the ruling already gates): re-run §1a on a real ≥8.4 MB
segment to replace the Corpus B upper bound with a measured full-window ratio.

## Sources

1. `zstd` crate API docs — `DEFAULT_COMPRESSION_LEVEL = 3`; `stream::read::Decoder` / `stream::write::Encoder`. https://docs.rs/zstd/latest/zstd/ (fetched 2026-08-11)
2. RFC 1951 (DEFLATE) — 32 KiB back-reference window ("distances up to 32K bytes"). https://www.rfc-editor.org/rfc/rfc1951 *(window figure is the format's defining constant; all other DEFLATE claims here are measured [M])*
3. RFC 8878 — "Zstandard Compression and the 'application/zstd' Media Type" (Feb 2021): frame concatenation, streaming with bounded memory, decoder window ≤ 8 MB recommendation, skippable frames. https://www.rfc-editor.org/rfc/rfc8878.html (fetched 2026-08-11)
4. lzbench 2.0.1 published results — AMD EPYC 9554, single thread, Silesia corpus: xz 5.6.3 -6 = 2.97 MB/s c / 127 MB/s d / 23.21%; xz -0 = 98.2 MB/s d; zstd 1.5.6 -1 = 422/1347 MB/s / 34.64%; zlib 1.3.1 = 323–344 MB/s d; lz4 1.10.0 = 3716 MB/s d. https://github.com/inikep/lzbench (README, fetched 2026-08-11)
5. facebook/zstd README — Silesia benchmark (i7-9700K, Ubuntu 24.04): zstd 1.5.7 -1 = 510/1550 MB/s, zlib 1.3.1 -1 = 105/390 MB/s, lz4 1.10.0 = 675/3850 MB/s, ratio column; "Zstandard's format is stable and documented in RFC8878"; dual BSD OR GPLv2; current release 1.5.7. https://github.com/facebook/zstd (fetched 2026-08-11)
6. NVD, CVE-2024-3094 — malicious code in xz/liblzma upstream tarballs 5.6.0–5.6.1; CVSS v3.1 10.0 Critical. https://nvd.nist.gov/vuln/detail/CVE-2024-3094 (fetched 2026-08-11)
7. crates.io registry API — versions, publish dates, download counts, licenses for `zstd`, `zstd-sys`, `ruzstd`, `lz4_flex`, `flate2`, `liblzma`. https://crates.io/api/v1/crates/{name} (fetched 2026-08-11)
8. KillingSpark/zstd-rs (ruzstd) README — decoder complete; encoder "usable, but it does not yet reach the speed, ratio or configurability of the original zstd library"; only fastest compression level implemented; no dictionary support. https://github.com/KillingSpark/zstd-rs (fetched 2026-08-11)
9. pseitz/lz4_flex README — block + frame formats; "no unsafe via the default feature flags"; benchmarks (Ryzen 7 5900HX): 66 kB JSON 4,540 MiB/s decompression at 0.2284 ratio (≈4.4×); 10 MB Dickens 2,338 MiB/s at 0.6372 (≈1.6×). https://github.com/pseitz/lz4_flex (fetched 2026-08-11)

**[M]** First-party measurements, this note, 2026-08-11: host 12th Gen Intel
Core i7-12800H; `zstd` v1.5.7 (`zstd -b1 -e19` in-memory benchmark), `xz`
5.8.3, `gzip` 1.14; Corpus A = `docs/gauntlet/runs/runB/attempt-2/journal_full.ndjson`
(real, 185,575 B, 71 events); Corpus B = synthetic 8.4 MB resample of Corpus A
(seeded PRNG, randomized ULIDs/hashes/timestamps, renumbered seq) — upper
bound only. CLI wall-times measured over 20–200 pipe iterations; small-file
loops are spawn-dominated, which is why in-memory `zstd -b` figures are quoted
for throughput.
