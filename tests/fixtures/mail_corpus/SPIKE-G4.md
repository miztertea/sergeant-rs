# G4 spike record — Mail (`.eml`) adoption (S4 Y4)

Sprint plan `sprint-plan-2026-08-27.md`, decision **G4** (J2, ruling 6; same
gate order as G3). Wave brief `brief-y4-mail.md`. The gates run **strictly
in order**, each fully before the next; any failure stops the spike and
escalates — no vendoring, forking, or allowlisting around it. This file is
the durable record of what was actually run and what it printed. Sibling
record: `MANIFEST.md` (gate b's corpus documentation) and `manifest.json`
(the exact-match counts).

Lane: worktree `/var/tmp/hats4/y4`, branch `hats4/y4-mail`, base `92a51e5e`
(integration after Y3/#330 merged — the container/child-key machinery this
wave's attachment recursion would ride, per the brief). Host: 20 cores,
30 GiB RAM, `/var/tmp` on a quota-clear volume (508G avail at spike end).
Toolchain `rustc 1.98.0 (88d9e12ae 2026-08-18)` / `cargo 1.98.0
(797e8a9bc 2026-08-05)` (pinned by `rust-toolchain.toml`). `cargo-deny
0.20.2`. Every Cargo command ran with `TMPDIR=/var/tmp/sgt-test-tmp`. Run
2026-08-28, ~09:35–10:20Z.

## (a) Deny gate — RUN FIRST, BEFORE ANY EXTRACTION CODE

**Result: PASS — no new advisory/license/ban/source failure.**

### The candidate crate, evaluated (not assumed)

`mail-parser` — the brief's and sprint plan's named candidate for `.eml`
parsing (brief-y4-mail.md; G4), re-verified today rather than trusted from
memory:

| Check | Result | Source |
|---|---|---|
| Version on crates.io today | 0.11.8 | `cargo info mail-parser` |
| License | `Apache-2.0 OR MIT` | `cargo info mail-parser`; confirmed by `cargo deny check licenses` passing and by direct read of the registry-cached `Cargo.toml` |
| `rust-version` | unreported by the registry (`cargo info` prints "unknown") — resolves and builds cleanly under this crate's pinned 1.98.0, which is the only floor that matters here | `cargo info mail-parser` |
| Maintainer / provenance | `stalwartlabs` — the same organization that ships the Stalwart mail server, i.e. a production consumer of its own parser, not an unmaintained side project | `cargo info mail-parser` (`repository`/`homepage`), ctx7 |
| Rust API shape | `MessageParser::parse(&[u8]) -> Option<Message>`; `Message::{subject, from, to, cc, message_id, references, date, text_body, html_body, attachments, headers}`; nested `message/rfc822` reached via `attachment.message() -> Option<Message>`; `MimeHeaders::attachment_name()` for filenames | ctx7 `/websites/rs_mail-parser_mail_parser` (1021 snippets, "High" source reputation), cross-checked empirically below |
| Documented parsing posture | "Follows the Robustness Principle, making a best-effort to parse non-conformant e-mail messages... `parse()` never panics; if no headers are found, `None` is returned." This is a **load-bearing finding**, not incidental — see gate (b)'s caveat below | ctx7, `docs.rs/mail-parser/latest/mail_parser/index.html` and `struct.MessageParser.html` |
| Own feature flags | `default = []`; `encoding_rs`/`full_encoding` (legacy charset decode — declared with `full_encoding` here, see Cargo.toml comment); `rkyv`; `serde` — none of these change the dependency SET, only what's compiled inside `mail-parser` itself | `cargo info mail-parser` |

A real alternative was checked, not skipped: `mailparse` (staktrace),
0.16.1, license `0BSD`. It has **no Rust-side documentation indexed in
ctx7** (`ctx7 library "mailparse"` returns only Node/Python/PHP libraries
of the same common name — none of them this crate), so its behavior could
not be verified the way the brief requires ("VERIFY crate behaviour... your
training knowledge... is likely stale"). Its own maintenance signal (one
maintainer, a `master`-branch-only homepage link, notably terser API
surface than `mail-parser`'s per its crates.io page) did not justify
building a from-memory case for it against a candidate that already has
1021 ctx7-indexed snippets and a production consumer maintaining it. Not
pursued further; recorded so the choice is auditable rather than silent.

Added to `Cargo.toml`'s `[dependencies]` as
`mail-parser = { version = "0.11.8", features = ["full_encoding"] }` —
`full_encoding` pulls in `encoding_rs` (already resolved elsewhere in this
graph, so it added zero new packages) so the encoding-zoo fixture's
non-UTF-8 charset decodes rather than lossy-replacing; without it,
`mail-parser`'s own feature table shows no charset fallback beyond
UTF-8/ASCII. Comment records this was provisional pending gates (b)/(c),
before any extraction code was written, matching the gate-order
requirement.

**A build-breaking authoring mistake, caught and fixed before it touched
gate (c)'s numbers**: the first edit to `Cargo.toml` accidentally deleted
the pre-existing `log = "0.4"` direct-dependency line (Y2's own addition,
needed by `runtime/atlas/office.rs`'s recovery watch) while inserting the
`mail-parser` line in its place. `cargo deny check` and `cargo metadata`
do not compile code, so this did not surface until gate (c)'s first
`cargo build --tests`, which failed with `E0433: use of unresolved
module... log`. Fixed by restoring the `log = "0.4"` line; gate (a) was
then **rerun in full against the corrected `Cargo.toml`** so the verbatim
evidence below matches exactly what gate (c) actually built (the
Cargo.lock diff after the fix is purely additive — see below — so the
correction did not change gate (a)'s dependency-graph conclusion, only
made it accurate to what ships).

### The crate set the addition actually locks

`cargo metadata` (no `--locked`, letting the resolver add the new subtree)
added **2 new packages** to `Cargo.lock`, zero existing entries removed or
version-bumped (diffed by `(name, version)` pair, not by counting diff `+`
lines):

```
hashify 0.2.9       (mail-parser's own const-lookup-table macro dependency)
mail-parser 0.11.8
```

`encoding_rs` was **already resolved** elsewhere in the graph (via
`reqwest`), so `full_encoding` added zero further packages. Package count:
**484 → 486** (verified by counting `name = ` lines in `Cargo.lock`, and
independently by set-diffing every `(name, version)` pair between the
pre- and post-change lockfiles).

### Verbatim results

**Baseline**, `Cargo.toml`/`Cargo.lock` unmodified at `92a51e5e`:

```
$ TMPDIR=/var/tmp/sgt-test-tmp cargo deny check
[... 1341 lines of duplicate-version warnings and dependency trees ...]
error[yanked]: detected yanked crate (try `cargo update -p chacha20`)
   ┌─ Cargo.lock:50:1
   │
50 │ chacha20 0.10.1 registry+https://github.com/rust-lang/crates.io-index
   │ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ yanked version
   │
   ├ chacha20 v0.10.1
     └── rand v0.10.2
         ├── lopdf v0.42.0
         │   └── pdf-inspector v1.17.0
         │       └── anydoc v0.2.4
         │           └── sergeant-rs v0.3.0
         └── ulid v3.0.0
             └── sergeant-rs v0.3.0 (*)

advisories FAILED, bans ok, licenses ok, sources ok
(exit 1)
```

**This is inherited, not new, and stated plainly as the brief requires**:
the baseline this spike started from already fails the full
`cargo deny check` on `main`'s own pre-existing graph, via `ulid` → `rand`
0.10.2 → a yanked `chacha20` 0.10.1 (sergeant-rs#328). This predates S4
entirely and has nothing to do with mail parsing. It is not this spike's
failure to fix, suppress, or explain away — it is recorded only so the
diff below is honest about what changed and what did not. (Full output:
`deny-baseline-full.txt`, beside this file.)

**With `mail-parser` added** (corrected `Cargo.toml`, `Cargo.lock` at 486
packages):

```
$ TMPDIR=/var/tmp/sgt-test-tmp cargo deny check
[... 1350 lines of duplicate-version warnings and dependency trees,
     now including hashify/mail-parser's position in those same
     pre-existing duplicate-version warnings' dependency trees ...]
error[yanked]: detected yanked crate (try `cargo update -p chacha20`)
   ┌─ Cargo.lock:50:1
   │
50 │ chacha20 0.10.1 registry+https://github.com/rust-lang/crates.io-index
   │ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ yanked version
   │
   ├ chacha20 v0.10.1
     └── rand v0.10.2
         ├── lopdf v0.42.0
         │   └── pdf-inspector v1.17.0
         │       └── anydoc v0.2.4
         │           └── sergeant-rs v0.3.0
         └── ulid v3.0.0
             └── sergeant-rs v0.3.0 (*)

advisories FAILED, bans ok, licenses ok, sources ok
(exit 1)
```

(Full output: `deny-with-mail-parser-full.txt`, beside this file.)

**The diff, computed, not eyeballed**: extracting every `^error[` and
`^warning[` line from both runs and diffing the two sorted sets produces
**zero lines of difference** — same one `error[yanked]` (chacha20, verbatim
identical span/tree), same 16 `warning[duplicate]` lines (all pre-existing
multi-version crates already in the S3/Y2/Y3 graph — `mail-parser` and
`hashify` appear only as new leaves *inside* a few of those pre-existing
trees' printed context, e.g. under the `indexmap`/`syn` duplicate-version
warnings that `anydoc`'s subtree already triggered, never as the SOURCE of
a new warning). `licenses ok`, `bans ok`, `sources ok` in both runs.
**`mail-parser` adds no new error and no new warning class.** Gate (a)
passes.

## (b) Hand-verified `.eml` fixture corpus

**Result: PASS**, against `manifest.json`'s CURRENT (corrected) values, not
against every value this file originally reported here at spike time — see
step 3 below and `MANIFEST.md`'s own "One field corrected in Y4" section for
the one field that did not match at spike time and was fixed downstream.
Full methodology, per-fixture coverage table, and the exact-match counts
live in `MANIFEST.md` and `manifest.json` beside this file — not restated
here so there is one place that can drift. Summary of what was done, in
order:

1. Six `.eml` fixtures hand-authored as raw bytes (`build_mail_fixtures.py`)
   covering: plain text; `multipart/alternative` (text+HTML); an attachment;
   a nested `message/rfc822`; an encoding zoo (quoted-printable body in a
   non-UTF-8 charset, a base64 attachment, RFC 2047 UTF-8 B-encoded Subject,
   RFC 2047 ISO-8859-1 Q-encoded From display-name); and a deliberately
   malformed message with zero RFC 5322 header lines.
2. Expected counts written to `manifest.json`, with a `counting_rules`
   block stating plainly what each field counts.
3. Independently cross-checked with Python's stdlib `email` package
   (`verify_with_stdlib_email.py`; transcript `stdlib-email-crosscheck.txt`)
   — a different implementation from both the authoring script and from
   `mail-parser`. Header counts, address counts, decoded Subject/From (RFC
   2047), the windows-1252→Unicode round-trip through quoted-printable, and
   attachment byte lengths all matched exactly. **One field did not**:
   fixture `04-nested-rfc822.eml`'s `nested_message.body_text_decoded`, as
   originally hand-typed into `manifest.json` at spike time, read with no
   trailing `\r\n`; `stdlib-email-crosscheck.txt` (this same commit,
   `a6e21c3d`) already shows a trailing newline on this exact field
   (`'...now nested one level deep.\n'`), and the diagnostic `mail-parser`
   run below (step 4, same commit) shows `\r\n` — both independent sources
   disagreed with the hand-authored manifest value at the moment this gate
   was recorded, which this file did not say at the time. **This is a
   correction to what this gate actually found, not a new re-run**: this
   file originally claimed an exact three-way match here, and that claim
   was false against artifacts committed in this very commit. The field was
   fixed downstream, in the adoption wave, after a direct re-read of the
   fixture's own raw bytes settled which trailing terminator RFC 2046
   §5.1.1 actually assigns to the boundary versus the body (`MANIFEST.md`'s
   own "One field corrected in Y4" section, `manifest.json`'s own
   `body_text_decoded_correction_note`) — the corrected value is
   independently re-verified (a direct hex read of the fixture plus stdlib
   `email`) and is very likely correct, so this is a correction to this
   record's own claim, not a reason to revisit the adopt decision. No other
   field, on this fixture or any other, is affected — see the full
   per-field comparison in `stdlib-email-crosscheck.txt` and
   `mail-parser-diagnostic-run.txt`, both committed alongside this file.
4. **Only then**, `mail-parser` was run once, diagnostically, from a
   throwaway scratch crate **outside this repository**
   (`/var/tmp/hats4-mailspike-scratch/`, never part of `hats4/y4`'s git
   tree) — never as committed extraction code, and never as the source of
   any manifest number. Transcript: `mail-parser-diagnostic-run.txt`; probe
   source kept for the record: `mail-parser-diagnostic-probe.rs`,
   `mail-parser-diagnostic-probe.Cargo.toml.txt`.

### The diagnostic run — four of five valid fixtures matched exactly; one field on the fifth did not, at the time

For fixtures `01`–`03` and `05`, `mail-parser`'s header counts, decoded
Subject/From, decoded body text, and attachment name/length matched
`manifest.json`'s independently-verified values **exactly**, including the
full encoding-zoo round-trip (`Café update ☕` recovered from the B-encoded
Subject, `René Dupont` from the Q-encoded From, and `Prix unitaire: 12€ le
café.\nTotal: 24€.` recovered byte-correct from windows-1252
quoted-printable — the same three values `verify_with_stdlib_email.py`
independently produced). Fixture `04`'s nested `message/rfc822` was reached
via `attachment.message()` exactly as ctx7's docs described, with the inner
message's own subject correct — **but its decoded body text, as this file
originally reported here, was NOT an exact match against the hand-authored
manifest value at spike time**: `mail-parser` produced a trailing `\r\n`
`manifest.json` did not have (see gate (b) step 3, above, for the full
correction and why the corrected value — not the original manifest
value — is the one independently re-verified as correct).

### Fixture 06 passed its own requirement

`06-malformed-no-headers.eml` → `parser.parse()` returned `None`, matching
`manifest.json`'s `required_outcome` exactly and confirming `mail-parser`'s
own documented contract ("if no headers are found, `None` is returned").

### A caveat found empirically — load-bearing for downstream adoption, not a gate-b failure

`06`'s pass is real but narrow: it proves `mail-parser` refuses input that
is *not email-shaped at all*. It does **not** prove the crate refuses input
that *looks like* a message but is broken inside — and the diagnostic run
shows it does not. `diagnostic-not-manifest-broken-mime.eml` (not part of
the exact-match corpus; see `MANIFEST.md`) is a **well-formed RFC 5322
envelope** (valid From/To/Subject/Date/Message-ID/MIME-Version/Content-Type)
whose declared `multipart/mixed` boundary is opened but never closed.
`mail-parser` did **not** return `None`:

```
=== 07-broken-mime-diagnostic-only.eml ===
  parse() -> Some(Message)
  header count: 7
  subject: Some("Unterminated boundary probe")
  body_text(0): None
  body_html(0): None
  text_body count: 0
  html_body count: 0
  attachment count: 1
    attachment[0] name=None len=48
```

The one body part — intended as `text/plain` — silently became a nameless,
untyped attachment with zero recognized text/html bodies, and no error,
warning, or `Result::Err` of any kind was raised. This is the direct mail
analogue of Y2's anydoc finding (`runtime/atlas/office.rs`'s own module doc:
anydoc's "lenient default... returning `Ok` with partially-repaired
content"): **`mail-parser` has no strict mode; its documented Robustness
Principle means "malformed but message-shaped" degrades silently rather
than failing.** A future adapter cannot rely on `parse()` returning `None`
as its coverage-honesty signal for anything but the narrowest case (zero
headers) — it will need its own detection layer, structurally the same
shape as `office.rs`'s log-based recovery watch, to turn a
silently-downgraded parse (fewer body/attachment units recovered than the
MIME structure declares) into an honest `error` coverage row rather than a
quiet, incomplete `ok`. **Named here as downstream wave work, not built in
this spike** — the brief's gate (b) requires fixture `06` to fail, which it
does; it does not require (and this spike does not claim) that `mail-parser`
fails on every malformed input unaided.

### A second caveat — synthesized bodies

The diagnostic run also showed `mail-parser` **auto-generating** an
`html_body` from a `text/plain`-only source (fixtures `01` and `03`: no
`text/html` part exists in the wire bytes, yet `html_body count: 1` and
`body_html(0)` returns a synthesized `<html>...</html>` wrapper around the
plain text). This matches the crate's documented behavior ("Automatic
conversion between inline HTML and plain text body parts is handled when
an alternative version is missing") but means a future adapter preserving
"message shape" (A1 §6.5: "text and HTML bodies") must distinguish a
genuinely-present HTML body from one `mail-parser` synthesized, or
provenance will silently claim the original message had HTML content it
did not. `manifest.json`'s `body_html_present` field is deliberately
defined against the **wire bytes**, not `mail-parser`'s output, for exactly
this reason.

## (c) Footprint — build-time and binary-size delta

**Method**: `tslp-footprint-delta-2026-08-27.md`'s discipline, including its
linked-vs-naive binary correction. Both legs solo (no concurrent build on
this host during either leg), fresh `CARGO_TARGET_DIR` per leg,
`TMPDIR=/var/tmp/sgt-test-tmp`, `rustc 1.98.0`. BEFORE = `92a51e5e`
(`Cargo.toml`/`Cargo.lock` restored to their committed state — the accidental
`log` deletion above was never present in this leg); AFTER = +1 crates.io
line (`mail-parser 0.11.8`, `full_encoding` feature) referenced from
`src/main.rs`.

Commands (per leg): `rm -rf $CARGO_TARGET_DIR; time cargo build --locked
--tests; time cargo build --locked; du -sb $CARGO_TARGET_DIR; stat -c %s
$CARGO_TARGET_DIR/debug/sgt`. Full logs: `footprint-before-tests.log`,
`footprint-before-plain.log`, `footprint-after-tests.log`,
`footprint-after-plain.log`.

| Metric | Before | After | Delta |
|---|---|---|---|
| Cargo.lock packages | 484 | 486 | +2 (`mail-parser`, `hashify`; `encoding_rs` already resolved) |
| cold `build --tests` | 161.154 s | 160.072 s | −1.08 s (−0.7%, noise-level) |
| cold total (`--tests` then plain) | 178.541 s | 177.483 s | −1.06 s (−0.6%, noise-level) |
| `target/` | 17,693,997,445 B | 17,687,644,427 B | **−6,353,018 B (−0.036%)** — smaller, not larger; see note below |
| debug `sgt` (naive) | 274,808,424 B | 274,657,168 B | −151,256 B — **MISLEADING**, see correction |
| **debug `sgt` (linked)** | 274,808,424 B | 275,742,536 B | **+934,112 B (+0.89 MiB, +0.34%)** |

**The naive number is a linker artifact, exactly as tslp's own spike found**:
nothing reachable from `sgt`'s `main` called into `mail_parser` at spike
time (no extraction code exists), so the linker dropped every `mail-parser`
symbol — `nm target/debug/sgt | grep -ci mail_parser` → **0** on the naive
binary. The linked number was measured by temporarily forcing a call from
`src/main.rs` (behind a runtime-`false` env-var check that never fires, so
behavior is unaffected), rebuilding, measuring, then reverting — confirmed
byte-for-byte back to 274,657,168 B after the revert (`git diff --stat
src/main.rs` empty) with `mail_parser` symbol count back to 0. On the
forced-reference build, `nm` found **1,272** `mail_parser` symbols.
**X5's F4 combined-delta rule carries +934,112 B (+0.89 MiB), not
−151,256 B.**

**Named gap, not glossed, matching the tslp precedent's own honesty
requirement**: the `target/` total went *down* by 6.35 MB (0.036% of a
17.7 GB tree) rather than up. This is reported as **measurement noise**,
not a causal shrink from adding two crates — `mail-parser`'s and
`hashify`'s own build artifacts total 8.85 MB (`find ... -iname
'*mail_parser*' -o -iname '*hashify*' | du -cb`), smaller than the
observed swing, and both legs are independent cold builds of a 486-crate
graph on the same host with no concurrent contention controlled for beyond
"solo" — filesystem cache state, incremental-fingerprint bookkeeping, and
build-order jitter are all more than sufficient to explain a swing this
small in either direction. The two build-time deltas (−0.6% to −0.7%) are
the same story: below the noise floor this measurement method can resolve,
not evidence that adding a dependency made the build faster. The only
metric with a clean causal explanation in either direction is the linked
binary size, because that one was isolated with a forced-reference/revert
pair specifically to remove linker nondeterminism from the answer.
**Release-profile size was not measured** (both legs are dev-profile, same
scope tslp's own spike named as an open gap) — provisional, not a validated
release-build number.

## Full suite — one decisive check, run once, PIPESTATUS checked

`TMPDIR=/var/tmp/sgt-test-tmp cargo nextest run --no-fail-fast` (worktree's
own `target/`, not either footprint leg): `fmt --check` clean, `clippy
--locked --all-targets -- -D warnings` clean, then nextest: **2153 tests
run, 2152 passed, 1 failed, 38 skipped** (`${PIPESTATUS[0]}` = 100, checked
directly, not read off a piped `tail`). The one failure,
`sergeant-rs::codex_backend
appserver_a_stray_notification_for_the_displaced_turn_never_taints_the_new_one`,
**passed cleanly re-run in isolation**
(`cargo nextest run -E 'test(appserver_a_stray_notification...)'` →
1 passed, `${PIPESTATUS[0]}` = 0) — a full-suite-load timing flake in an
unrelated codex-appserver backend test, not caused by this spike: nothing
in `src/` calls `mail_parser` yet (confirmed by gate (c)'s own `nm` symbol
count, 0, on the unforced binary), so an inert `Cargo.toml` line cannot be
the cause. Recorded per this repo's flakiness discipline rather than
silently re-run away; not this spike's failure to fix, same footing as the
pre-existing `chacha20` advisory above.

## (d) Adopt or escalate

**All three gates pass. ADOPT `mail-parser` 0.11.8 (`full_encoding`
feature) for `.eml` parsing (J2 — this decision is the gate's own outcome
rule, "ADOPT OR ESCALATE... all three pass → adopt," not a separate
judgment call; ruling 6 governs the gate order itself).**

- (a) deny gate: no new advisory/license/ban/source failure — zero-line
  diff against baseline's error/warning set.
- (b) fixture corpus: all six required categories covered, every count
  independently cross-checked two ways before `mail-parser` ran once, and
  the required-to-fail fixture failed exactly as required.
- (c) footprint: +2 packages, build-time delta at noise level, +0.89 MiB
  linked binary growth (isolated from linker nondeterminism by the
  forced-reference correction) — no ceiling in this wave's brief or the
  combined-delta rule this crosses.

**Two named caveats carry forward as downstream wave work, not spike
failures** (both recorded in gate (b) above, not new here): (1)
`mail-parser` has no strict mode — a message-shaped-but-broken input
degrades silently rather than erroring, so a real adapter needs its own
recovery-detection layer (structurally the same shape as
`runtime/atlas/office.rs`'s log-based watch for anydoc) to satisfy the
brief's coverage-honesty rule for anything beyond the zero-headers case;
(2) `mail-parser` synthesizes an HTML body from plain text when none
exists in the source, so "message shape preserved" claims in a real
adapter must be checked against the wire bytes, not against `mail-parser`'s
own output, for the `html_body_present` field specifically.

**`Cargo.toml`/`Cargo.lock` are left with `mail-parser` added** (adoption
outcome, not reverted) — `full_encoding` feature, comment recording the
gate-a/b/c evidence trail per this file. No extraction/adapter code was
written in this spike (out of scope per the brief's own gate order: gates
(a)–(c) precede any extraction code, and the adapter module, replaceability
boundary, `child_key` composition for attachment recursion, register row 8,
and the CHANGELOG entry are the wave's own downstream implementation work,
not the spike). `src/main.rs` is unmodified (the footprint probe was
reverted, confirmed byte-identical). The scratch verification crate at
`/var/tmp/hats4-mailspike-scratch/` and the two fresh `CARGO_TARGET_DIR`s
used for gate (c) (`/var/tmp/hats4-y4-footprint-before`,
`/var/tmp/hats4-y4-footprint-after`) live outside this repository and are
not part of this commit.
