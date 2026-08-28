//! S3 X5 — the **A1a acceptance battery**: contract §17 walked literally.
//!
//! `A1-ATLAS-WORLD-INTELLIGENCE.md` §17 lists fourteen acceptance items. This
//! file is the walk: one register row per item, each carrying a verdict and
//! the **named decisive check** that produces it — an earlier wave's test
//! where one already pins the exact claim, a test written here where none did,
//! or an explicit deferral with the citation that moved it.
//!
//! Three rules this file exists to enforce, none of which a prose checklist
//! could:
//!
//! 1. **No silent pass.** An item that cannot be proven is recorded as a gap
//!    with a destination sprint, never as "met". Item 4 was such a row from
//!    S3 close through S4 Y5: its tripwire,
//!    `a1a_item_4_gap_cloud_placeholder_detection_is_not_shipped`, failed
//!    the day someone shipped the heuristic without updating the walk, so
//!    the gap could not quietly close or quietly widen. S4 Y6 shipped it —
//!    see [`a1a_item_4_the_coverage_vocabulary_now_names_online_only`] for
//!    the update that landed in the tripwire's place.
//! 2. **No dangling citation.** [`every_named_check_exists_in_the_suite_it_names`]
//!    reads every referenced file and fails if a named test was renamed or
//!    deleted. A green suite that no longer contains the test the walk cites
//!    proves nothing, and this is what notices.
//! 3. **No drifting table.** The register below is the source of truth and
//!    [`the_documented_walk_table_matches_the_register`] holds this doc
//!    comment to it, so the PR-ready table and the executable one cannot
//!    disagree.
//!
//! Running this file alone does not re-prove the claims it cites — that is
//! what the wave's own full `cargo nextest run` is for, and item 14's decisive
//! check *is* that run. What this file proves is that every one of the
//! fourteen items has a named, existing, non-fictional answer.
//!
//! ## The walk
//!
//! | # | Contract claim (§17) | Verdict | Decisive check |
//! |---|---|---|---|
//! | 1 | one daemon-owned `atlas.duckdb` holds independently rebuildable `ops/source/git/context/meta` families | met-with-deviation | `x1_atlas_substrate::opening_atlas_declares_the_four_schema_namespaces` |
//! | 2 | estate Git indexing pins exact repo/base generations and Work overlays without changing Work authority | met-with-deviation | `x3a_git_plumbing::a_scan_stays_on_its_pinned_sha_while_head_advances` |
//! | 3 | a declared read-only local knowledge Source indexes a cloud-synced ordinary directory without becoming `[[repo]]` or receiving a worktree | met | `x5_a1a_acceptance::a1a_item_3_a_knowledge_source_is_indexed_without_becoming_a_repo_or_getting_a_worktree` |
//! | 4 | online-only/unreadable local resources are reported as coverage gaps, not silently indexed as empty | met | `y6b_online_only::an_online_only_placeholder_is_a_named_gap_row_through_the_real_scan_trigger` |
//! | 5 | Markdown/text and at least one Office format normalize into document units with provenance | met | `y2_office_adapter::a_docx_worker_returns_document_and_section_units_with_provenance` |
//! | 6 | CSV/JSON/Parquet stay relational, with a deterministic aggregate and selected text-field context units sharing row identity | met | `x4_tabular_map::datasets_are_registered_and_read_in_place_as_derived_evidence` |
//! | 7 | a bounded ZIP exposes child resources while rejecting unsafe paths and enforcing ceilings | met | `y3_zip_adapter::a_zip_worker_declares_admitted_children_through_the_real_subprocess` |
//! | 8 | `.eml` or the chosen first mail format produces structured message evidence | met | `y4_mail_adapter::a_mail_worker_returns_message_shape_and_attachment_with_provenance` |
//! | 9 | image/scanned evidence enters the OCR fallback with page/region/engine provenance | deferred-post-s4 | — |
//! | 10 | external Git acquisition resolves an exact commit in a no-Work-checkout cache | met | `src/runtime/atlas/external_git::a_refresh_resolves_the_origins_new_tip_over_the_same_cache` |
//! | 11 | source parsing uses content/extractor identity so unchanged resources reuse cached facts | met | `x2_knowledge_sources::a_scan_records_once_reuses_an_unchanged_generation_and_evicts_a_changed_one` |
//! | 12 | daemon remains sole Atlas writer and worker failure cannot corrupt journal authority | met | `x5_a1a_acceptance::a1a_item_12_no_atlas_write_path_is_reachable_from_the_cli` |
//! | 13 | `sgt map`/status surfaces expose source/generation/coverage rather than arbitrary SQL | met | `x5_a1a_acceptance::a1a_item_13_no_client_sql_reaches_the_store` |
//! | 14 | all existing exact-root, Work-surface, distro-route/edition and split-hardening output-contract tests remain green | met | `x5_a1a_acceptance::a1a_item_14_the_inherited_contract_pins_still_exist` |
//!
//! The `Decisive check` column names **one** check per row so the table stays
//! readable; every row's full check list — several rows carry four — is in
//! [`WALK`] below, and each one is verified to exist.
//!
//! ## One cross-cutting gap, closed (S4 Y5, G8)
//!
//! S3 shipped Atlas's writers and Atlas's readers with no operator-reachable
//! *trigger* between them: no CLI verb, no route, and no daemon job called a
//! scan, so on a real installation Atlas stayed empty until a test invoked
//! it — a fact this file used to pin with a tripwire
//! (`a1a_cross_cutting_gap_no_shipped_surface_triggers_a_scan`) precisely so
//! it could not close silently. **It has closed.** `sgt intelligence scan`
//! (`POST /v1/intelligence/scan`) drives a full scan of an estate's declared
//! `[[knowledge]]` sources through the daemon, on the intelligence lane,
//! reporting from each source's own coverage counts; `sgt intelligence
//! add`/`list` (`POST`/`GET /v1/intelligence/sources`) is item 10's own
//! acquisition surface. This supersedes the settled J3 record the tripwire
//! encoded and the concept page's "no way to start a scan" sentence — both
//! edited in this same wave, per the authority chain S4's plan (G8) states
//! rather than merely asserts: the owner's explicit ordering delegation plus
//! S4's own acceptance being unprovable without a trigger. `sgt intelligence`
//! is no longer a read-only verb set — see
//! [`the_intelligence_verb_set_now_includes_the_trigger_and_the_acquisition_surface`]
//! for the replacement pin. Scheduling and cadence remain deliberately
//! unbuilt (G10): this is one call, one scan, one report — a recurring
//! trigger is S5+'s, when retrieval needs one.
//!
//! **S4 Y6 widened the trigger to the whole estate** (the owner correction
//! `estate-intelligence-is-the-feature-2026-08-28.md`, carried as G8's own
//! completion rather than new scope): the identical endpoint now also
//! scans every declared `[[repo]]` repository through the Git path at its
//! pinned SHA, and refreshes every external-Git source already recorded on
//! this host — see [`intelligence_scan`](../src/api.rs)'s own doc for the
//! full per-kind shape, and `tests/y6a_estate_scoped_scan.rs` for the
//! end-to-end proof this file's own item-2/item-4 checks do not attempt.
//! `sgt intelligence scan` is the verb's primary spelling now (argued in
//! [`the_intelligence_verb_set_now_includes_the_trigger_and_the_acquisition_surface`]'s
//! own update); `sgt knowledge scan` still runs the same widened scan
//! rather than being narrowed to match its own name.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

use sergeant_rs::domain::source::Coverage;
use sergeant_rs::runtime::atlas::db::AtlasDb;
use sergeant_rs::runtime::atlas::record::{ScanRecord, scan_and_record};
use sergeant_rs::runtime::atlas::scan::KnowledgeSource;
use sergeant_rs::runtime::journal::Journal;

const SGT: &str = env!("CARGO_BIN_EXE_sgt");

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

// ------------------------------------------------------------------ register

/// What the walk concluded about one §17 item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    /// Shipped and proven by the named checks.
    Met,
    /// Shipped and proven, but not in the shape §17's wording assumes — the
    /// deviation is named in the row's `note` with the decision that took it.
    MetWithDeviation,
    /// Not fully provable today. The `note` names what is missing and the
    /// sprint that owns it. Never a pass.
    Gap,
    /// Out of A1a's scope by a ratified re-cut, cited in the `note`.
    DeferredS4,
    /// Out of A1a's scope AND out of S4's own scope, cited in the `note` —
    /// distinct from [`Self::DeferredS4`] because "lands in S4" and "lands
    /// after S4" are different destinations and a reader must not have to
    /// infer which one a row means (S4 Y5's register correction, item 9:
    /// the register's earlier "S4's" was a mis-citation — owner ruling 3 and
    /// the ratified re-cut place OCR after S4).
    DeferredPostS4,
}

impl Verdict {
    fn as_str(self) -> &'static str {
        match self {
            Self::Met => "met",
            Self::MetWithDeviation => "met-with-deviation",
            Self::Gap => "gap",
            Self::DeferredS4 => "deferred-s4",
            Self::DeferredPostS4 => "deferred-post-s4",
        }
    }
}

/// One named check: the suite file it lives in, and the test function's name.
///
/// A `(file, test)` pair rather than a free-text citation precisely so
/// [`every_named_check_exists_in_the_suite_it_names`] can go and look.
struct Check {
    file: &'static str,
    test: &'static str,
}

const fn at(file: &'static str, test: &'static str) -> Check {
    Check { file, test }
}

/// One row of the §17 walk.
struct Item {
    number: u8,
    verdict: Verdict,
    checks: &'static [Check],
    /// Why this verdict, in the words a reviewer needs — the deviation taken,
    /// the gap left, or the citation that deferred the item.
    note: &'static str,
}

/// This file's whole claim, in one place.
const WALK: &[Item] = &[
    Item {
        number: 1,
        verdict: Verdict::MetWithDeviation,
        checks: &[
            at(
                "tests/x1_atlas_substrate.rs",
                "opening_atlas_declares_the_four_schema_namespaces",
            ),
            at(
                "tests/x1_atlas_substrate.rs",
                "atlas_database_has_exactly_one_owner",
            ),
            at(
                "tests/m5_projections.rs",
                "t2_the_duckdb_file_has_exactly_one_owner",
            ),
            at(
                "tests/x2_knowledge_sources.rs",
                "source_facts_survive_a_real_daemon_restart",
            ),
        ],
        note: "DEVIATION, ratified: the families live in TWO daemon-owned databases, not one \
               file. `atlas.duckdb` declares meta/source/git/context; `ops.*` is a schema inside \
               the operations projection (`sergeant.duckdb`), because F2/F3 kept two databases \
               with two independent one-owner invariants rather than collapsing them. \
               `git.*` carries no table of its own: repository-derived facts land in `source.*` \
               with the source kind as a column (see `runtime/atlas/mod.rs`). 'Independently \
               rebuildable' is proven as the two rebuild disciplines F1 names — ops refolds from \
               the journal on every start while source facts persist — by the restart test.",
    },
    Item {
        number: 2,
        verdict: Verdict::MetWithDeviation,
        checks: &[
            at(
                "tests/x3a_git_plumbing.rs",
                "a_scan_stays_on_its_pinned_sha_while_head_advances",
            ),
            at(
                "tests/x3a_git_plumbing.rs",
                "a_new_commit_with_an_identical_tree_is_the_same_generation",
            ),
            at(
                "tests/x3a_git_plumbing.rs",
                "a_work_overlay_is_scoped_to_its_work_and_evicted_with_it",
            ),
            at(
                "tests/x3a_scan_uses_only_local_reads.rs",
                "an_atlas_scan_runs_only_read_only_git_and_changes_nothing",
            ),
            at(
                "tests/y6a_estate_scoped_scan.rs",
                "a_registered_repository_is_scanned_through_the_git_path_and_map_symbol_resolves_a_real_function",
            ),
            at(
                "tests/x5_a1a_acceptance.rs",
                "a1a_item_2_gap_work_overlay_scan_has_no_production_trigger",
            ),
        ],
        note: "'Without changing Work authority' is the read-only half: the scan runs only \
               read-only git plumbing, touches no branch, no index and no worktree, and an \
               overlay's lifetime is its Work's. THE BASE-REPO HALF IS FULLY MET (S4 Y6, G8 \
               correction): `scan_estate_git_on_lane` now has a real production caller — \
               `POST /v1/intelligence/scan`/`sgt intelligence scan` — proven end to end in \
               `tests/y6a_estate_scoped_scan.rs`, closing exactly the 'built but unreachable' \
               defect S4 Y5 left. DEVIATION, review panel finding: the sibling half of this \
               claim — 'Work overlays' — is NOT met the same way. `scan_work_overlay`, \
               `scan_work_overlay_on_lane` and `scan_and_record_overlay` \
               (`src/runtime/atlas/overlay.rs`, `lane.rs`, `record.rs`) exist, are correct at \
               the unit level (the three `x3a_git_plumbing.rs` checks above), and are proven \
               read-only at the plumbing level — but have ZERO production callers: no HTTP \
               route, no `sgt` verb, and no Work-lifecycle hook (admission or retirement) ever \
               invokes them outside a test. A real Work's overlay evidence — files it changed, \
               hashed live over its base tree — cannot actually be produced or recorded on a \
               real installation today; the capability exists only at the unit-test level, \
               exactly the shape the base-repo half was in before this same wave closed it. \
               TRIPWIRE, not deleted when closed: \
               `a1a_item_2_gap_work_overlay_scan_has_no_production_trigger` fails the day a \
               production caller is wired, so this note's own claim is checked rather than \
               merely asserted — update this row's verdict back to `met` and its note when \
               that caller lands, per this file's own no-silent-pass rule (see item 4's own \
               precedent for the shape of that update).",
    },
    Item {
        number: 3,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/x5_a1a_acceptance.rs",
                "a1a_item_3_a_knowledge_source_is_indexed_without_becoming_a_repo_or_getting_a_worktree",
            ),
            at(
                "tests/x2_knowledge_sources.rs",
                "knowledge_add_declares_a_source_and_list_reads_it_back",
            ),
            at(
                "tests/x2_knowledge_sources.rs",
                "a_declared_source_scans_exactly_what_the_manifest_says",
            ),
            at(
                "tests/x2_knowledge_sources.rs",
                "knowledge_add_refuses_a_path_inside_the_estates_own_territory",
            ),
        ],
        note: "X2 proved the positive half (declared, scanned, read back). The NEGATIVE half — \
               not a repository, no worktree, not one byte written into the source tree — had no \
               decisive check, so this wave wrote one. 'Cloud-synced' is exercised as what it \
               is: an ordinary directory, because A1-05 leaves transport and auth to the sync \
               client and Sergeant reads the local bytes; no provider integration is claimed. \
               CROSS-CUTTING GAP applies: 'can index' is met as a capability, invoked here \
               through the real writer, but no shipped surface triggers that scan for an \
               operator yet.",
    },
    Item {
        number: 4,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/y6b_online_only.rs",
                "an_online_only_placeholder_is_a_named_gap_row_through_the_real_scan_trigger",
            ),
            at(
                "tests/y6b_online_only.rs",
                "a_genuinely_empty_file_is_not_misreported_as_a_placeholder_through_the_real_scan_trigger",
            ),
            at(
                "tests/x5_a1a_acceptance.rs",
                "a1a_item_4_the_coverage_vocabulary_now_names_online_only",
            ),
            at(
                "src/runtime/atlas/scan.rs",
                "a_suspected_placeholder_is_never_indexed_as_empty",
            ),
            at(
                "src/runtime/atlas/scan.rs",
                "a_suspected_placeholder_dataset_is_never_registered",
            ),
            at(
                "tests/x2_knowledge_sources.rs",
                "an_unreachable_root_keeps_the_generation_it_cannot_rescan",
            ),
            at(
                "tests/x4_tabular_map.rs",
                "an_unreadable_dataset_is_a_coverage_fact_not_a_scan_failure",
            ),
            at(
                "tests/x3b_syntax_wiring.rs",
                "a_file_a_grammar_cannot_parse_is_an_error_row_with_no_partial_symbols",
            ),
        ],
        note: "UPDATED S4 Y6 (G7/A1-06): this row was a named GAP at S3/S4-Y5 close (see this \
               file's own git history for the retired verdict and note) — the tripwire that \
               pinned it, `a1a_item_4_gap_cloud_placeholder_detection_is_not_shipped`, is \
               retired per its own assertion message's instruction ('update ... rather than \
               deleting'): the register row below is the update, and the vocabulary/behavior it \
               used to forbid now has its own positive checks, cited above. \
               THE UNREADABLE HALF stays met, unchanged: unreadable roots, unreadable datasets, \
               unparseable files and excluded paths all leave a named coverage row. \
               THE ONLINE-ONLY HALF NOW SHIPS, as a best-effort heuristic honestly labelled — \
               which is the acceptance item's own literal wording, not a stronger claim than it \
               makes. SIGNAL: `st_blocks == 0` with `st_size > 0`, read from the \
               `symlink_metadata` call the walker already makes per entry (no added syscall). \
               PERMITTED SYSCALL SET: exactly `lstat`/`stat` — `open()`/`read()` are never \
               attempted on a path this check flags, checked strictly before the byte-read \
               boundary in both `Walk::file` and `Walk::dataset` \
               (`src/runtime/atlas/scan.rs`), because A1 §7 forbids auto-hydrating a library and \
               `open()` is the documented hydration trigger on several cloud filesystems. \
               `listxattr`/`getxattr` were investigated and NOT adopted: no single \
               verifiable-via-documentation xattr convention for a cloud placeholder covers this \
               build's Linux/macOS targets, and guessing one would repeat the `enclosed_name` \
               mistake S4's own record already made once. HONEST LIMITS, named in the coverage \
               row's own `detail` text every time, not just here: a legitimate sparse file (a \
               disk image, a punched-out log) reads identically and is a FALSE POSITIVE; a \
               placeholder a sync client reports with full block allocation before the byte is \
               fetched is not caught at all and is a FALSE NEGATIVE. A genuinely empty file \
               (`st_size == 0`) is never flagged — flagging it would be the opposite dishonesty. \
               PROVEN END TO END through the real trigger \
               (`tests/y6b_online_only.rs`), not only at the pure-function level.",
    },
    Item {
        number: 5,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/y2_office_adapter.rs",
                "a_docx_worker_returns_document_and_section_units_with_provenance",
            ),
            at(
                "tests/y2_office_adapter.rs",
                "a_real_parser_failure_leaves_the_daemon_up_the_permit_freed_and_a_named_coverage_row",
            ),
            at(
                "tests/y2_office_boundary.rs",
                "anydoc_is_named_nowhere_but_the_office_adapter",
            ),
        ],
        note: "S4 Y2, closing the deferral the S3 sprint plan's panel adjudication finding 1 \
               recorded here. The Markdown/text half shipped in S3 (see item 3's checks). The \
               Office half ships now: `.docx` via a third-party document-conversion crate \
               (owner ruling J4, dated 2026-08-27 — see the owner-rulings knowledge library), \
               run inside Y1's supervised worker, proven through the real subprocess and the \
               real parser — not a synthetic fixture. Provenance is A1 §6.3's own shape: \
               normalizer identity + version (`office::DOCX_EXTRACTOR`), citation of the \
               ORIGINAL resource (never a temp path), and a unit coordinate where recoverable \
               — a structural `block:<n>` coordinate for an Office section rather than a byte \
               offset, because the original bytes are a compressed container the normalizer \
               has already unpacked by the time a unit is visible (see `runtime/atlas/office.rs`'s \
               own module doc and `domain::source::UnitKind`'s doc for the full argument). Output is \
               derived, never canonical (A1-12). NARROWING, not a deviation from what §17 asks: \
               `.docx` is this wave's one adopted format (G3's gate order), so `office::extractor_for` \
               claims nothing else yet — a second Office format is explicitly out of this \
               sprint's scope. CROSS-CUTTING GAP applies, same as items 3/6/13: the adapter is \
               invoked here through its own writer (the worker binary) and by tests, not yet by \
               a shipped scan trigger — that daemon-side scheduling is Y5's (G8), same as it was \
               for Y1's own worker transport.",
    },
    Item {
        number: 6,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/x4_tabular_map.rs",
                "datasets_are_registered_and_read_in_place_as_derived_evidence",
            ),
            at(
                "tests/x4_tabular_map.rs",
                "f12_a_stored_fact_describes_the_query_that_actually_ran",
            ),
            at(
                "tests/x4_tabular_map.rs",
                "f10a_a_declared_allowlist_exposes_only_its_columns_with_stable_row_identity",
            ),
            at(
                "tests/x4_tabular_map.rs",
                "f10a_rows_the_allowlist_cannot_distinguish_are_honestly_re_keyed",
            ),
        ],
        note: "NARROWING, ratified: 'selected text-field context units' are gated by F10a's \
               operator-declared column allowlist, whose default is NONE. A dataset with no \
               declared allowlist is still registered, counted and profiled; it publishes no \
               row text at all. That is a deliberate narrowing of the item toward the secrets \
               posture, decided at X4 on panel finding 7 — not an omission. CROSS-CUTTING GAP \
               applies: the dataset walk is invoked by its own writer and by tests, not by a \
               shipped trigger.",
    },
    Item {
        number: 7,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/y3_zip_adapter.rs",
                "a_zip_worker_declares_admitted_children_through_the_real_subprocess",
            ),
            at(
                "tests/y3_zip_adapter.rs",
                "an_archive_level_refusal_fails_its_own_worker_alone",
            ),
            at(
                "src/runtime/atlas/archive.rs",
                "overlapping_files_refuse_the_whole_archive_before_any_entry_opens",
            ),
            at(
                "src/domain/source.rs",
                "a_grandchild_key_chains_through_its_own_parent_not_the_root",
            ),
        ],
        note: "S4 Y3 (G5 as AMENDED 2026-08-28). `enclosed_name` is a path-STRING validator only \
               (research note beside the sprint plan, VERIFIED against zip2's source); this wave \
               adds explicit checks on top with their own named coverage rows: entry TYPE \
               (symlink refused via `is_symlink`, checked first and unconditionally of the \
               entry's name; every other non-regular type — FIFO, char/block device, socket — \
               refused by masking the entry's own Unix mode bits (`S_IFMT`) directly rather than \
               trusting `zip`'s `is_file()`, which is `!is_dir() && !is_symlink()` and does not \
               check `S_IFREG` at all, VERIFIED against `zip` 8.6.0's own `src/read.rs`), \
               non-empty name, name uniqueness, and a Unicode \
               NFC-then-case-fold normalisation rule (not a bare `to_lowercase()`) for \
               case-insensitive/NFC-NFD-folding collisions. Two size bounds — per-entry \
               uncompressed size (reused from `scan::MAX_RESOURCE_BYTES`, R2) and total expanded \
               size — are ENFORCED BY COUNTING STREAMED BYTES (`Read::take`), never by trusting \
               the attacker-controlled `size()`/`compressed_size()` header fields. The other two \
               named bounds are honestly different, not folded into that same claim: entry count \
               is checked against the central directory's own real record count, not an \
               attacker-inflatable declared integer; the compression-ratio bound is an ADVISORY \
               pre-filter computed from those same declared header fields BEFORE any byte \
               streams — cheap triage in front of the streamed per-entry check that actually \
               holds under a header that lies about the ratio too. Nesting depth caps recursion, \
               not bytes. Every ceiling but the reused per-entry one is named PROVISIONAL and \
               named PROVISIONAL and unmeasured (Y1's memory-cap precedent, #325). CLOSES THE \
               RESEARCH'S OPEN ITEM: `zip` 8.6.0 does NOT reject overlapping/self-referential \
               (quine-shaped) constructions on its own — its own `has_overlapping_files` doc says \
               so verbatim ('this doesn't make the archive invalid') — so this wave calls it \
               itself, before opening any entry, and refuses the whole archive when it fires \
               (VERIFIED against a hand-crafted overlapping fixture, sanity-checked against the \
               crate's own diagnostic before asserting anything about this wave's own defence). \
               A SECOND correction to the research note, found while building this wave's own \
               fixtures rather than merely read from source: two central-directory records with \
               BYTE-IDENTICAL raw names do not both survive to be visited at all — the crate's \
               own `IndexMap`-keyed construction collapses them to one entry, silently, \
               last-write-wins, before `len()`/`by_index` are ever called, which is a stronger \
               (and different) claim than the research's 'hidden from by_name, visible by index'. \
               Child resources keep parent provenance (G9): every admitted child carries a \
               content hash and a COMPOSED F7 key (`domain::source::child_key`) built from its \
               IMMEDIATE parent's own key — chained, not resolved to the root archive, so a \
               grandchild's key encodes its whole ancestry without a stored chain. No entry is \
               ever executed and nothing is written to a real path (deliverable d): the adapter \
               is pure bytes-in/structs-out, never touches a filesystem. NAMED SEAM, not a \
               silent gap: `worker::DeclaredChild`/`WorkerBatch` do not yet carry a child's \
               content bytes, hash, or composed key on the wire — only `name`/`relative_path`, \
               Y1's own original shape — so per-entry coverage rows and F7 provenance are proven \
               exhaustively against `archive::expand`'s own return value (in-process) but do not \
               yet reach the daemon; widening that shared wire type is left to the wave that \
               wires real daemon-side persistence (rides G8's trigger, Y5), stated explicitly in \
               `archive.rs`'s own module doc rather than silently deferred. CROSS-CUTTING GAP \
               applies, same as items 3/5/6/13: the adapter is invoked here through the real \
               worker binary and by tests, not yet by a shipped scan trigger.",
    },
    Item {
        number: 8,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/y4_mail_adapter.rs",
                "a_mail_worker_returns_message_shape_and_attachment_with_provenance",
            ),
            at(
                "tests/y4_mail_adapter.rs",
                "a_real_parser_failure_leaves_the_daemon_up_the_permit_freed_and_a_named_coverage_row",
            ),
            at(
                "tests/y4_mail_adapter.rs",
                "a_genuine_html_body_reaches_the_wire_and_a_synthesized_one_does_not",
            ),
            at(
                "src/domain/source.rs",
                "a_grandchild_key_chains_through_its_own_parent_not_the_root",
            ),
        ],
        note: "S4 Y4 (G4 — ADOPT, tests/fixtures/mail_corpus/SPIKE-G4.md). `mail-parser` 0.11.8 \
               (`full_encoding` feature) via the same gate order G3/G5 used: deny gate FIRST \
               (zero new advisory/license/ban/source failure, diffed against the pre-existing \
               yanked-chacha20 baseline, #328, untouched), a hand-verified 6-fixture corpus \
               cross-checked independently with Python's stdlib `email` package BEFORE \
               mail-parser ever ran, footprint (+2 packages, +0.89 MiB linked binary once linker \
               nondeterminism is corrected out). Message shape is A1 §6.5 verbatim: from/to/cc, \
               sent timestamp (RFC3339), subject, text AND html bodies, message id, thread \
               (References + In-Reply-To folded, deduplicated), attachments — provenance carries \
               parser identity + version (`mail::MAIL_EXTRACTOR`). TWO caveats the spike found \
               empirically are CLOSED, not merely named as downstream work: (1) a synthesized \
               text or HTML body (`mail-parser` aliases the SAME MessagePartId into both \
               `text_body` and `html_body` when no genuine alternative exists — VERIFIED against \
               `mail-parser`'s own source, `parsers/message.rs`, not merely observed) is detected \
               by inspecting the aliased part's own physical PartType (not index equality alone \
               — an index-only version shipped a direction bug, caught in review, that discarded \
               a genuinely HTML-only message's real HTML and reported mail-parser's own \
               html_to_text-converted rendering as a genuine text body) and reports whichever \
               side is the synthesized alias absent, matching `manifest.json`'s own \
               wire-bytes-only definition of `body_html_present`/`body_text_present`; (2) a \
               message-shaped-but-broken \
               input (an unterminated MIME boundary) that `mail-parser` silently downgrades into \
               a nameless attachment (SPIKE-G4.md's own diagnostic finding) is detected — any \
               leaf, non-message attachment with no recoverable name — and refuses the WHOLE \
               message, no partial units (F8). A structurally encrypted/S-MIME message \
               (`multipart/encrypted`, `application/pkcs7-mime`) gets its own honest \
               `MailError::Sealed` status, never a garbage-body silent success — `mail-parser` \
               has zero PKCS#7/S-MIME awareness of its own (VERIFIED: no match anywhere in its \
               source for pkcs7/smime/encrypted), so this is detected structurally from the \
               message's own declared Content-Type, never by decrypting or verifying anything. \
               Attachments recurse through Y3's container machinery exactly as the brief \
               requires: a `message/rfc822` attachment recurses via THIS module's own \
               `build_mail_message` using mail-parser's OWN already-parsed embedded value (a \
               design correction made while building this wave's own tests: re-serializing to \
               raw bytes and re-parsing picked up a boundary-adjacent CRLF a second parse read \
               as body content, disagreeing with the hand-verified, stdlib-cross-checked \
               manifest — corrected in manifest.json with the full argument, not silently \
               patched around); an attachment that is itself a ZIP recurses via \
               `archive::expand_at_depth` — the SAME function `archive.rs` calls on itself, \
               widened to `pub(crate)` (R2) so mail and archive nesting share ONE depth counter \
               and ONE whole-tree cumulative-bytes budget, never two independently-sized ones. \
               THE REVERSE DIRECTION CHAINS TOO (a review finding, closed here, not merely \
               documented): `archive::classify` now routes a `.eml`-named ZIP entry back into \
               THIS module's own `parse_at_depth`, through the same shared depth/budget — a \
               `.eml` entry inside a ZIP previously fell through to `unsupported` despite this \
               module's own doc already claiming the chain worked whichever container kind each \
               level happens to be. THE SAME ADMISSION DISCIPLINE AS Y3 (brief item 3): empty-name refusal, \
               path-safety per `/`-component via `domain::is_plain_name`, name uniqueness, and \
               the identical Unicode NFC-then-case-fold collision rule via \
               `archive::collision_key` — reused outright (R2), not a second copy. BOUNDS \
               reused wholesale from `archive.rs` (R2) rather than three new independently-tuned \
               numbers: `MAX_ENTRY_UNCOMPRESSED_BYTES` per attachment, \
               `MAX_TOTAL_EXPANDED_BYTES` cumulative, `MAX_ZIP_ENTRIES` reused as an \
               attachment-count ceiling — all PROVISIONAL, same footing as Y1's memory cap \
               (#325). HONEST GAP, STATED not glossed: unlike ZIP's `Read::take` streaming, \
               `mail-parser` decodes every part eagerly before this adapter's own bounds ever \
               run, so these are POST-decode admission checks, not pre-allocation stream bounds \
               — the worker's own `RLIMIT_AS` (Y1) is the real backstop against a single \
               oversized decode, named plainly rather than implied as equivalent to the ZIP \
               case; separately, MIME transfer encodings have no ZIP-class decompression-bomb \
               ratio (base64 ~4:3, quoted-printable at most ~3:1), so the gap is materially \
               smaller than it would be for a compressed container. NO REPLACEABILITY BOUNDARY \
               for `mail_parser` (J1, stated per the brief's own instruction): Office's boundary \
               exists to discharge a specific owner ruling over a RUSTSEC advisory G4's own deny \
               gate did not find; Y3's `archive.rs` (a second real container adapter) already set \
               the precedent of no dedicated one-owner test, which this wave follows rather than \
               diverging from unprompted. CROSS-CUTTING GAP applies, same as items 3/5/6/7/13: \
               the adapter is invoked here through the real worker binary and by tests, not yet \
               by a shipped scan trigger.",
    },
    Item {
        number: 9,
        verdict: Verdict::DeferredPostS4,
        checks: &[],
        // S4 Y5's correction: the previous note here read "S4's, same
        // citation as item 7" — a mis-citation. Owner ruling 3 and the
        // ratified S4/S5+ re-cut both place OCR/layout evaluation AFTER S4
        // (feature-gate vs. worker-binary, ONNX excluded), not inside it; S4
        // ships items 4/5/7/8/10 + §10 package identity and item 9 is not
        // among them (G1). Corrected in-sprint per J5 (governing ruling)
        // over J3 (an unmoved register note) rather than left standing.
        note: "OCR/layout evidence enters post-S4, not S4 — owner ruling 3 plus the ratified S4 \
               re-cut (G1) name the destination explicitly; the earlier \"S4's, same citation as \
               item 7\" note was a mis-citation, corrected here (S4 Y5).",
    },
    Item {
        number: 10,
        verdict: Verdict::Met,
        checks: &[
            at(
                "src/runtime/atlas/external_git.rs",
                "a_refresh_resolves_the_origins_new_tip_over_the_same_cache",
            ),
            at(
                "src/runtime/atlas/external_git.rs",
                "the_cache_is_bare_and_never_grows_a_working_tree",
            ),
            at(
                "src/runtime/atlas/external_git.rs",
                "an_external_agents_md_becomes_ordinary_indexed_text_never_instructions",
            ),
            at(
                "src/runtime/atlas/locator.rs",
                "ext_remote_helper_is_refused",
            ),
            at(
                "tests/y5_external_git_triggers.rs",
                "an_unallowlisted_locator_is_refused_by_the_api_before_git_runs",
            ),
        ],
        note: "S4 Y5 (G6): locator allowlist BEFORE Git sees the string, bare no-working-tree \
               host cache outside every estate, exact-commit resolution over the identical \
               ls-tree/cat-file plumbing X3a already reads estate-git through, full A1 §9 \
               provenance (origin/requested_ref/resolved_commit/retrieved_at) in a new \
               git.provenance table, `sgt intelligence add/list` as the CLI surface. A live \
               `https://`/`ssh://` acquisition end to end needs network access this suite's \
               sandbox does not have; the acquisition mechanism itself is proven against a \
               local origin with the protocol allowlist deliberately widened for the test only \
               (never in production code — a separate test proves the production entry point \
               never does this), and the allowlist decision itself is proven exhaustively, \
               separately, in `locator.rs`.",
    },
    Item {
        number: 11,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/x2_knowledge_sources.rs",
                "a_scan_records_once_reuses_an_unchanged_generation_and_evicts_a_changed_one",
            ),
            at(
                "tests/x3a_git_plumbing.rs",
                "stored_estate_git_keys_are_blob_oids_and_a_rescan_evicts_nothing",
            ),
            at(
                "tests/x3b_syntax_wiring.rs",
                "the_syntax_extraction_is_keyed_on_the_blob_oid_and_the_grammar",
            ),
            at(
                "tests/x4_tabular_map.rs",
                "f10a_narrowing_an_allowlist_retracts_the_units_it_exposed",
            ),
        ],
        note: "Both halves of F7's key are pinned: content identity (blob OID for repository \
               bytes, BLAKE3 for local knowledge) AND extractor identity — the fourth check is \
               the staleness direction, where unchanged bytes read by a changed extractor \
               supersede rather than serve the previous parser's rows.",
    },
    Item {
        number: 12,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/x5_a1a_acceptance.rs",
                "a1a_item_12_no_atlas_write_path_is_reachable_from_the_cli",
            ),
            at(
                "src/runtime/atlas/db.rs",
                "open_read_only_refuses_a_store_that_does_not_exist_and_creates_nothing",
            ),
            at(
                "src/runtime/atlas/db.rs",
                "open_read_only_reads_confirmed_rows_and_cannot_write",
            ),
            at(
                "tests/x1_atlas_substrate.rs",
                "atlas_database_has_exactly_one_owner",
            ),
            at(
                "tests/x3a_git_plumbing.rs",
                "a_panicking_intelligence_job_is_reported_and_frees_its_permit",
            ),
            at(
                "tests/x2_knowledge_sources.rs",
                "a_crash_between_the_rows_and_the_summary_reports_neither",
            ),
            at(
                "tests/x2_knowledge_sources.rs",
                "a_crash_after_the_summary_but_before_confirmation_completes_the_scan",
            ),
            at(
                "tests/y1_worker_transport.rs",
                "a_fault_worker_leaves_the_daemon_up_the_permit_freed_and_a_named_coverage_row",
            ),
        ],
        note: "RESIDUAL CLOSED (S4 Y1, G2): the row-1 residual read 'DESTINATION: S4, as a \
               read-only open' — `sgt doctor`'s atlas coverage row now opens the store through \
               `AtlasDb::open_read_only`, which asks DuckDB itself for `AccessMode::ReadOnly` and \
               runs no `CREATE SCHEMA`/`CREATE TABLE` DDL at all, idempotent or not. The two new \
               db.rs checks pin both halves: a store that does not exist is refused rather than \
               materialized, and a store that does exist is read but genuinely cannot be written \
               through this connection (DuckDB refuses the write, not merely this crate declining \
               to attempt one). G2 also widens what 'daemon remains sole Atlas writer' has to \
               survive: a worker's returned batch is now itself untrusted input, validated \
               daemon-side (identity, `enclosed_name` path safety, F10 deny-set membership on \
               declared child names) before anything is written — the fault-injection check cited \
               here is the SUPERVISION proof for that (Y2 carries the real-parser malformed-input \
               proof); nothing here claims a third-party parser exists yet.",
    },
    Item {
        number: 13,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/x5_a1a_acceptance.rs",
                "a1a_item_13_no_client_sql_reaches_the_store",
            ),
            at(
                "tests/x4_tabular_map.rs",
                "f11_map_ships_five_verbs_and_defers_neighbors_and_changed",
            ),
            at(
                "tests/x4_tabular_map.rs",
                "f11_the_map_surface_answers_over_http_and_through_the_verb",
            ),
            at(
                "tests/x4_tabular_map.rs",
                "f10_a_dataset_path_a_reader_would_glob_is_refused",
            ),
        ],
        note: "X4 pinned the verbs and the wire shape. What had no check was the negative claim \
               — that no surface accepts SQL and the store exposes no way to run any — so this \
               wave wrote one. `map neighbors`/`changed` are absent by declared deferral (F11), \
               which the verb-set check holds them to. CROSS-CUTTING GAP applies from the other \
               side: these surfaces expose exactly what a scan wrote, and nothing shipped \
               triggers a scan, so on a fresh installation they honestly answer empty.",
    },
    Item {
        number: 14,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/x5_a1a_acceptance.rs",
                "a1a_item_14_the_inherited_contract_pins_still_exist",
            ),
            at(
                "tests/m8_estate_cli.rs",
                "a_descendant_of_the_estate_root_finds_no_estate_and_spawns_nothing",
            ),
            at(
                "tests/m12_child_work.rs",
                "a_bare_sgt_run_from_a_work_surface_still_refuses_while_dash_c_works",
            ),
            at(
                "tests/m11_nested_workflow.rs",
                "a_nested_leafs_own_output_contract_is_enforced_exactly_as_a_flat_ones_is",
            ),
        ],
        note: "The decisive check for 'remain green' is the wave's own full suite run, recorded \
               in the closeout commit — a test cannot assert its siblings passed. What IS \
               assertable, and what the first check does, is that those inherited pins still \
               EXIST: a suite stays trivially green if the tests are deleted, and that is the \
               failure mode this item is really about.",
    },
];

// ------------------------------------------------- the register's own guards

#[test]
fn every_contract_item_is_accounted_for() {
    let numbers: Vec<u8> = WALK.iter().map(|item| item.number).collect();
    assert_eq!(
        numbers,
        (1..=14).collect::<Vec<u8>>(),
        "§17 has fourteen items; the walk must carry each exactly once, in order"
    );

    for item in WALK {
        assert!(
            !item.note.is_empty(),
            "item {} has no note: a verdict without a reason is not a walk",
            item.number
        );
        match item.verdict {
            Verdict::DeferredS4 | Verdict::DeferredPostS4 => assert!(
                item.checks.is_empty(),
                "item {} is deferred; it must not claim a check",
                item.number
            ),
            _ => assert!(
                !item.checks.is_empty(),
                "item {} claims a verdict with no decisive check — the one thing this \
                 battery exists to prevent",
                item.number
            ),
        }
    }

    // The re-cut's own boundary, pinned so a later edit cannot quietly pull an
    // S4 item back into A1a's "accepted" column (or push an S3 one out).
    // Item 9 (OCR) moved from `DeferredS4` to `DeferredPostS4` in S4 Y5's own
    // register correction (the earlier "S4's" note was a mis-citation —
    // owner ruling 3 places it after S4); item 10 (external Git) closed in
    // Y5 itself, exactly as items 5/7/8 closed in Y2/Y3/Y4.
    let deferred_s4: BTreeSet<u8> = WALK
        .iter()
        .filter(|item| item.verdict == Verdict::DeferredS4)
        .map(|item| item.number)
        .collect();
    assert_eq!(
        deferred_s4,
        BTreeSet::new(),
        "S4 has no remaining `deferred-s4` items: item 5 closed in Y2, item 7 in Y3, item 8 in \
         Y4, item 10 in Y5 (register row edits each time); item 9 moved to deferred-post-s4 \
         (S4 Y5's own correction), never simply dropped"
    );
    let deferred_post_s4: BTreeSet<u8> = WALK
        .iter()
        .filter(|item| item.verdict == Verdict::DeferredPostS4)
        .map(|item| item.number)
        .collect();
    assert_eq!(
        deferred_post_s4,
        BTreeSet::from([9]),
        "item 9 (OCR) is the sole post-S4 deferral, corrected here from its earlier mis-citation"
    );

    // And every deferral has to say so in words a reader can check, not just
    // by its enum.
    for item in WALK
        .iter()
        .filter(|i| i.verdict == Verdict::DeferredS4 || i.verdict == Verdict::DeferredPostS4)
    {
        assert!(
            item.note.contains("S4"),
            "item {}'s deferral must cite where it went",
            item.number
        );
    }

    // A gap must name where it is going, or it is a silent pass wearing a
    // different label.
    for item in WALK.iter().filter(|i| i.verdict == Verdict::Gap) {
        assert!(
            item.note.contains("DESTINATION"),
            "item {}'s gap must name a destination sprint",
            item.number
        );
    }
}

#[test]
fn every_named_check_exists_in_the_suite_it_names() {
    for item in WALK {
        for check in item.checks {
            let text = read(check.file);
            assert!(
                text.contains(&format!("fn {}(", check.test)),
                "item {}: {} names no `{}` — a citation that does not resolve proves nothing",
                item.number,
                check.file,
                check.test
            );
        }
    }
}

/// The doc table at the top of this file and [`WALK`] must agree.
///
/// The table is what a reviewer reads in the pull request; the register is
/// what runs. Two copies of one claim drift, so one is checked against the
/// other rather than trusted.
#[test]
fn the_documented_walk_table_matches_the_register() {
    let text = read("tests/x5_a1a_acceptance.rs");
    let rows: Vec<&str> = text
        .lines()
        // The separator row (`//! |---|`) has no space after the bar, so this
        // filter already excludes it; only the header needs skipping.
        .filter(|line| line.starts_with("//! | "))
        .skip(1)
        .collect();
    assert_eq!(
        rows.len(),
        WALK.len(),
        "the doc table must carry every item"
    );

    for (row, item) in rows.iter().zip(WALK) {
        let cells: Vec<&str> = row
            .trim_start_matches("//! |")
            .split('|')
            .map(str::trim)
            .collect();
        assert_eq!(
            cells[0],
            item.number.to_string(),
            "doc table row order must match the register"
        );
        assert_eq!(
            cells[2],
            item.verdict.as_str(),
            "doc table verdict for item {} disagrees with the register",
            item.number
        );
        match item.verdict {
            Verdict::DeferredS4 | Verdict::DeferredPostS4 => assert_eq!(
                cells[3], "—",
                "a deferred item names no check in the table either"
            ),
            _ => {
                let first = &item.checks[0];
                let suite = first
                    .file
                    .trim_start_matches("tests/")
                    .trim_end_matches(".rs");
                assert_eq!(
                    cells[3],
                    format!("`{suite}::{}`", first.test),
                    "doc table check for item {} must be the register's first check",
                    item.number
                );
            }
        }
    }
}

// ------------------------------------------------- item 2: the overlay tripwire

/// §17.2's tripwire for the half of the claim that is NOT met: "Work
/// overlays" (a Work surface's changes, hashed live over its base tree) has
/// no production caller anywhere `sgt` actually runs.
///
/// [`sergeant_rs::runtime::atlas::overlay::scan_work_overlay`],
/// [`sergeant_rs::runtime::atlas::lane::scan_work_overlay_on_lane`] and
/// [`sergeant_rs::runtime::atlas::record::scan_and_record_overlay`] are
/// exercised only from test files (`tests/x3a_git_plumbing.rs` and this
/// suite's own `checks` list) — never from `src/api.rs`'s router or
/// `src/cli.rs`'s command dispatch, which is where every other shipped
/// scan path (`scan_estate_git_on_lane`, `scan_local_knowledge_on_lane`,
/// `acquire_external_git_on_lane`) is actually called from. This mirrors
/// exactly the "function with no production caller" shape S4 Y5 left for
/// `scan_estate_git_on_lane`, which this same wave (Y6a) closed — this
/// tripwire is what keeps the overlay half from being reported closed
/// alongside it by accident.
///
/// **If this test fails**, someone wired a production trigger for
/// Work-overlay scanning (an API parameter, a CLI verb, or a Work-lifecycle
/// hook). That is good news — update this row's verdict from
/// `met-with-deviation` back to `met`, rewrite the note, and either delete
/// this tripwire or repoint it at the new caller, per this file's own
/// no-silent-pass rule (`item 4`'s history is the precedent for exactly
/// this kind of update, not a deletion).
#[test]
fn a1a_item_2_gap_work_overlay_scan_has_no_production_trigger() {
    let api = read("src/api.rs");
    let cli = read("src/cli.rs");
    for needle in [
        "scan_work_overlay_on_lane",
        "scan_and_record_overlay",
        "scan_work_overlay(",
    ] {
        assert!(
            !api.contains(needle) && !cli.contains(needle),
            "src/api.rs or src/cli.rs now calls `{needle}` — a production trigger for \
             Work-overlay scanning appears to have landed. §17 item 2's register row 2 must be \
             updated (verdict + note) to reflect this instead of leaving it as a \
             met-with-deviation gap; see this test's own doc comment for what to do."
        );
    }
}

// --------------------------------------------------- item 3: the negative half

/// §17.3 — a declared knowledge Source is indexed **as evidence**: it never
/// becomes a repository, never receives a worktree, and never has a byte
/// written into it.
///
/// X2 proved every positive half of this item. The negative half is what
/// distinguishes a Source from a `[[repo]]` at all (A1-03), and it had no
/// decisive check, so this is it: declare through the real CLI, scan through
/// the real writer, then look for the three things that must not have
/// happened.
#[test]
fn a1a_item_3_a_knowledge_source_is_indexed_without_becoming_a_repo_or_getting_a_worktree() {
    let estate = TempDir::new().expect("estate");
    let data = TempDir::new().expect("data dir");
    let synced = TempDir::new().expect("synced dir");

    // An ordinary directory of the shape a sync client leaves behind — which
    // is the whole of what A1-05 asks Sergeant to read.
    fs::create_dir_all(synced.path().join("Team Notes")).expect("mkdir");
    fs::write(
        synced.path().join("Team Notes/onboarding.md"),
        "# Onboarding\n\nStep one.\n\n## Access\n\nAsk the estate owner.\n",
    )
    .expect("write note");
    let before = tree_snapshot(synced.path());

    fs::write(
        estate.path().join("sergeant.toml"),
        "[estate]\nname = \"x5-acceptance\"\n",
    )
    .expect("write manifest");

    let declared = Command::new(SGT)
        .current_dir(estate.path())
        .args(["--data-dir", &data.path().display().to_string()])
        .args(["knowledge", "add", "notes"])
        .arg(synced.path())
        .output()
        .expect("sgt knowledge add");
    assert!(
        declared.status.success(),
        "knowledge add must succeed: {}",
        String::from_utf8_lossy(&declared.stderr)
    );

    // 1. It is not a repository. `sgt repo list` is the surface that would say
    //    so, and the manifest is the record it reads.
    let repos = Command::new(SGT)
        .current_dir(estate.path())
        .args(["--data-dir", &data.path().display().to_string()])
        .args(["--json", "repo", "list"])
        .output()
        .expect("sgt repo list");
    assert!(repos.status.success(), "repo list must succeed");
    let listed = String::from_utf8_lossy(&repos.stdout);
    assert!(
        !listed.contains("notes"),
        "a knowledge source must not appear as a repository: {listed}"
    );
    let manifest = fs::read_to_string(estate.path().join("sergeant.toml")).expect("manifest");
    assert!(
        manifest.contains("[[knowledge]]") && !manifest.contains("[[repo]]"),
        "the declaration must land as `[[knowledge]]`, never as `[[repo]]`: {manifest}"
    );

    // 2. Indexing it produces real evidence.
    let mut db = AtlasDb::open(data.path()).expect("open atlas");
    let mut journal = Journal::open(data.path()).expect("open journal");
    let source = KnowledgeSource {
        name: "notes".to_string(),
        root: synced.path().to_path_buf(),
        ignore: Vec::new(),
        context_fields: Default::default(),
    };
    let record = scan_and_record(&mut db, &mut journal, &source, None).expect("scan");
    let generation = match record {
        ScanRecord::Recorded { generation_id, .. } => generation_id,
        other => panic!("a readable source must record a generation, got {other:?}"),
    };
    let counts = db.coverage_counts("notes").expect("coverage");
    assert_eq!(
        counts.get(Coverage::Indexed.as_str()).copied(),
        Some(1),
        "the one note must be indexed: {counts:?}"
    );
    let units = db.units("notes", 50).expect("units");
    assert!(
        !units.is_empty(),
        "an indexed Markdown file must produce units (generation {generation})"
    );

    // 3. Nothing was cut from it and nothing was written into it. A worktree
    //    would leave a `.git` file or directory at the root; a mount would
    //    leave a copy under the estate; a writer would change the bytes.
    assert!(
        !synced.path().join(".git").exists(),
        "no worktree may be cut from a knowledge source"
    );
    assert_eq!(
        tree_snapshot(synced.path()),
        before,
        "a knowledge source is read-only evidence: the scan must not change one byte"
    );
    assert!(
        !estate.path().join("repos").exists(),
        "declaring a knowledge source must not create a repository mount"
    );
    let surfaces = data.path().join("surfaces");
    let cut = fs::read_dir(&surfaces).map(Iterator::count).unwrap_or(0);
    assert_eq!(
        cut, 0,
        "declaring and indexing a knowledge source must cut no Work surface"
    );
}

/// Every file under `root`, as `(relative path, bytes)`, so "unchanged" means
/// unchanged content and unchanged membership — not merely an unchanged count.
fn tree_snapshot(root: &Path) -> BTreeSet<(String, Vec<u8>)> {
    let mut out = BTreeSet::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).expect("read dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .expect("under root")
                    .display()
                    .to_string();
                out.insert((relative, fs::read(&path).expect("read file")));
            }
        }
    }
    out
}

// ------------------------------------ item 4: the online-only heuristic, now shipped

/// §17.4's **named gap, now closed** (S4 Y6, G7/A1-06).
///
/// This replaces `a1a_item_4_gap_cloud_placeholder_detection_is_not_shipped`,
/// which pinned the gap as a negative tripwire: it failed the day Atlas
/// started reasoning about cloud placeholders at all, by name, so its own
/// assertion message said what to do next — "update this file's register
/// row 4 — verdict, note and decisive check — instead of deleting this
/// tripwire". This is that update, following the identical pattern the
/// cross-cutting-gap tripwire below it already set: the old function is
/// retired rather than left to assert a claim that stopped being true, and
/// its replacement proves the positive the old one only ever forbade the
/// negative of.
///
/// Proves the vocabulary grew the named state (not a re-proof of the
/// heuristic's own behavior — [`src/runtime/atlas/scan.rs`]'s own unit tests
/// and `tests/y6b_online_only.rs`'s end-to-end trigger test own that; this
/// is the acceptance-register-level pin that the wire spelling exists and
/// stays exactly what F8 always promised, no more and no fewer states).
#[test]
fn a1a_item_4_the_coverage_vocabulary_now_names_online_only() {
    let vocabulary: BTreeSet<&str> = Coverage::ALL.iter().map(|c| c.as_str()).collect();
    assert_eq!(
        vocabulary,
        BTreeSet::from([
            "discovered",
            "indexed",
            "excluded",
            "unavailable",
            "unsupported",
            "error",
            "online_only",
            "generation_evicted",
        ]),
        "F8's coverage vocabulary is what makes an unreadable OR unmaterialized resource a \
         reported gap — `online_only` is S4 Y6's own addition, and nothing else in the \
         vocabulary should have moved"
    );
}

// -------------------------------------- the cross-cutting gap, now closed

/// S4 Y5 (G8): the trigger this file's former tripwire
/// (`a1a_cross_cutting_gap_no_shipped_surface_triggers_a_scan`) said did not
/// exist now does — the retired test's exact handshake, completed rather
/// than routed around.
///
/// Proves the positive this time: `record_scan`/`record_external_git_scan`
/// (the Atlas writer entry points) really are reachable from production
/// code in `src/api.rs` — a scan trigger that only ever called them from a
/// test module would be the same false claim the old tripwire existed to
/// catch, inverted. Item 12's own boundary (the CLI process itself never
/// writes; only the daemon does) is unweakened — see
/// [`a1a_item_12_no_atlas_write_path_is_reachable_from_the_cli`], which this
/// test does not relax.
#[test]
fn the_trigger_is_reachable_from_production_code_not_only_a_test_module() {
    let api = read("src/api.rs");
    // The real test module boundary — not the first `#[cfg(test)]` in the
    // file, which (as of this wave) also gates a standalone test-only helper
    // function well before this module and would wrongly mark production
    // code between the two as "test-only".
    let test_module = api
        .find("\nmod tests {")
        .expect("api.rs must have a test module for this check to mean anything");
    let mut production_callers = 0;
    for name in ["record_scan(", "record_external_git_scan("] {
        for (index, _) in api.match_indices(name) {
            if index < test_module {
                production_callers += 1;
            }
        }
    }
    assert!(
        production_callers >= 2,
        "src/api.rs must call the Atlas writer entry points from production code (one for \
         `sgt knowledge scan`, one for `sgt intelligence add`), not only from its own test \
         module"
    );
}

/// `sgt intelligence` now offers `status`, `add` and `list` — the read
/// surface S3 shipped plus item 10's acquisition surface (G6, G8). The
/// retired tripwire asserted the verb set was exactly `{status}`; this is
/// its direct successor, asserting the earned superset rather than merely
/// "more than one verb".
#[test]
fn the_intelligence_verb_set_now_includes_the_trigger_and_the_acquisition_surface() {
    let help = Command::new(SGT)
        .args(["intelligence", "--help"])
        .output()
        .expect("sgt intelligence --help");
    let text = String::from_utf8_lossy(&help.stdout);
    let verbs: BTreeSet<&str> = text
        .split("Commands:")
        .nth(1)
        .unwrap_or("")
        .split("Options:")
        .next()
        .unwrap_or("")
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter(|verb| *verb != "help")
        .collect();
    assert_eq!(
        verbs,
        BTreeSet::from(["status", "add", "list", "scan"]),
        "`sgt intelligence` must ship exactly status/add/list/scan: {text}"
    );

    // And `sgt knowledge` kept its own scan spelling working (S4 Y6, G8
    // correction: the trigger is now estate-scoped, not
    // declared-local-sources-only, so `sgt intelligence scan` — beside
    // `status`/`add`/`list`, the verb group that already covers every
    // source kind — is the primary name; `sgt knowledge scan` still runs
    // the identical scan rather than being narrowed or removed).
    let help = Command::new(SGT)
        .args(["knowledge", "--help"])
        .output()
        .expect("sgt knowledge --help");
    let text = String::from_utf8_lossy(&help.stdout);
    let verbs: BTreeSet<&str> = text
        .split("Commands:")
        .nth(1)
        .unwrap_or("")
        .split("Options:")
        .next()
        .unwrap_or("")
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter(|verb| *verb != "help")
        .collect();
    assert_eq!(
        verbs,
        BTreeSet::from(["add", "list", "scan"]),
        "`sgt knowledge` must ship exactly add/list/scan: {text}"
    );
}

// ------------------------------------------------ item 12: the writer boundary

/// §17.12 — no Atlas **write** path is reachable from the CLI process.
///
/// The one-owner test already holds the database driver to a single file. This
/// is the other half: that file's mutating API — staging a scan, confirming
/// one, evicting a generation or a Work's overlays — must be called only from
/// the daemon and its API surface, never from a `sgt` subcommand running in
/// the user's own process.
///
/// Structural rather than behavioural on purpose. The behaviour of a CLI that
/// *did* write is unbounded, so the assertion worth making is that the call
/// does not exist to be made.
#[test]
fn a1a_item_12_no_atlas_write_path_is_reachable_from_the_cli() {
    const WRITE_PATHS: &[&str] = &[
        "stage_scan",
        "confirm_scan",
        "evict_provisional",
        "evict_work_overlays",
    ];

    let db = read("src/runtime/atlas/db.rs");
    for path in WRITE_PATHS {
        assert!(
            db.contains(&format!("pub fn {path}(")),
            "`{path}` must still be Atlas's write API for this check to mean anything"
        );
    }

    let cli = read("src/cli.rs");
    for path in WRITE_PATHS {
        assert!(
            !cli.contains(&format!(".{path}(")),
            "src/cli.rs calls Atlas's `{path}` — the daemon is the sole Atlas writer"
        );
    }

    // The read the CLI *is* allowed is `sgt doctor`'s coverage row, and it is
    // deliberately the only Atlas call in the binary's own surface. Pinning
    // the exhaustive list here is what makes a second one visible in review.
    let opens: Vec<&str> = cli
        .lines()
        .filter(|line| line.contains("AtlasDb::"))
        .map(str::trim)
        .collect();
    assert_eq!(
        opens,
        vec![
            "let sources = match AtlasDb::open_read_only(data_dir).and_then(|db| db.indexed_sources()) {"
        ],
        "the CLI's only Atlas call is doctor's coverage read; a new one is a writer-boundary \
         decision, not a refactor"
    );

    // And the daemon really is the writer, so the boundary is a boundary and
    // not merely an absence.
    let daemon = read("src/daemon.rs");
    let api = read("src/api.rs");
    assert!(
        daemon.contains("AtlasDb::open") || api.contains("AtlasDb::open"),
        "the daemon side must hold the store it is sole writer of"
    );
}

// ------------------------------------------------------- item 13: no client SQL

/// One string literal lifted out of a Rust source file, with the lines it
/// spans — the unit [`sql_literal_holes`] reasons over.
struct Literal {
    start_line: usize,
    end_line: usize,
    text: String,
}

/// Every string literal in `source`, in file order, line comments skipped.
///
/// A character scan rather than a line scan, and that is the point: a Rust
/// string literal may span lines (rustfmt's trailing `\` continuation), so a
/// per-line reader cannot tell a literal's interior from the code around it.
/// Line comments are skipped because db.rs's own doc comments quote SQL in
/// prose, and prose is not a statement.
///
/// `src/runtime/atlas/db.rs` carries no raw strings, no block comments and no
/// character literal holding a quote; the two `assert!`s in
/// [`sql_literal_holes`] fail loudly if that ever stops being true, rather
/// than letting the scan silently see nothing.
fn string_literals(source: &str) -> Vec<Literal> {
    let chars: Vec<char> = source.chars().collect();
    let mut out = Vec::new();
    let mut line = 1usize;
    let mut i = 0usize;
    while i < chars.len() {
        match chars[i] {
            '\n' => {
                line += 1;
                i += 1;
            }
            '/' if chars.get(i + 1) == Some(&'/') => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            '"' => {
                let start_line = line;
                let mut text = String::new();
                i += 1;
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' {
                        if chars.get(i + 1) == Some(&'\n') {
                            line += 1;
                        }
                        i += 2;
                        continue;
                    }
                    if chars[i] == '\n' {
                        line += 1;
                    }
                    text.push(chars[i]);
                    i += 1;
                }
                i += 1;
                out.push(Literal {
                    start_line,
                    end_line: line,
                    text,
                });
            }
            _ => i += 1,
        }
    }
    out
}

/// Every interpolation hole in every SQL-carrying string literal in `source`,
/// rendered as `<enclosing fn>: <hole> <- <filler>` in file order.
///
/// A hole is any `{` that is not the `{{` escape, and it is reported with the
/// text up to its `}` — so a named hole (`{table}`) is reported as `{table}`
/// and shows up as a new, unpinned entry rather than being skipped the way a
/// "leading `{}` only" test would skip it.
fn sql_literal_holes(source: &str, lines: &[&str]) -> Vec<String> {
    const KEYWORDS: [&str; 5] = ["SELECT", "INSERT", "UPDATE", "DELETE", "FROM"];

    let literals = string_literals(source);
    assert!(
        literals.len() > 40,
        "the literal scanner must actually be reading db.rs, saw {} literals",
        literals.len()
    );
    let sql: Vec<&Literal> = literals
        .iter()
        .filter(|literal| KEYWORDS.iter().any(|word| literal.text.contains(word)))
        .collect();
    assert!(
        sql.len() > 10,
        "the scanner must actually be finding Atlas's SQL, saw {} statements",
        sql.len()
    );

    let mut out = Vec::new();
    for literal in sql {
        // The enclosing function: the nearest `fn` declaration at or above the
        // literal's first line.
        let function = lines[..literal.start_line]
            .iter()
            .rev()
            .find_map(|line| {
                let trimmed = line.trim_start();
                let rest = trimmed
                    .strip_prefix("pub fn ")
                    .or_else(|| trimmed.strip_prefix("fn "))?;
                Some(rest.split(['(', '<', ' ']).next().unwrap_or(rest))
            })
            .unwrap_or("<no enclosing fn>");
        // The filler: the first non-blank line after the literal closes.
        let filler = lines[literal.end_line..]
            .iter()
            .map(|line| line.trim())
            .find(|line| !line.is_empty())
            .unwrap_or("<nothing>");

        let text: Vec<char> = literal.text.chars().collect();
        let mut i = 0usize;
        while i < text.len() {
            if text[i] != '{' {
                i += 1;
                continue;
            }
            if text.get(i + 1) == Some(&'{') {
                i += 2; // `{{` is an escaped brace, not a hole.
                continue;
            }
            let close = text[i..]
                .iter()
                .position(|c| *c == '}')
                .map_or(text.len() - 1, |offset| i + offset);
            let hole: String = text[i..=close].iter().collect();
            out.push(format!("{function}: {hole} <- {filler}"));
            i = close + 1;
        }
    }
    out
}

/// §17.13 — the map and status surfaces answer in source/generation/coverage
/// terms, and there is no way to hand the store a query.
///
/// Three things have to be true at once, and each is checked here: the store
/// exposes no "run this SQL" entry point; the SQL that does exist is canned,
/// with the only varying fragment chosen by an enum; and the shipped verb set
/// is closed.
#[test]
fn a1a_item_13_no_client_sql_reaches_the_store() {
    let db = read("src/runtime/atlas/db.rs");
    let lines: Vec<&str> = db.lines().collect();

    // 1. No public API takes SQL. The scan walks whole `pub fn` *signatures* —
    //    accumulating lines until the parameter list's parentheses close —
    //    rather than single lines, because rustfmt splits a long signature
    //    across lines and a per-line scan would wave through an `sql: &str`
    //    parameter that landed on its own line. `query_identity` takes the SQL
    //    that ran, to hash it into provenance — it executes nothing — so it is
    //    named as the single allowed exception rather than matched around.
    assert!(
        db.contains("pub fn query_identity("),
        "the one named exception must still exist, or this check is excusing nothing"
    );
    let mut signatures = 0usize;
    for (index, line) in lines.iter().enumerate() {
        if !line.trim_start().starts_with("pub fn ") {
            continue;
        }
        let mut signature = String::new();
        let mut depth = 0i32;
        for candidate in &lines[index..] {
            signature.push_str(candidate.trim());
            signature.push(' ');
            depth += candidate.matches('(').count() as i32;
            depth -= candidate.matches(')').count() as i32;
            if depth <= 0 {
                break;
            }
        }
        signatures += 1;
        if signature.contains("pub fn query_identity(") {
            continue;
        }
        assert!(
            !signature.contains("sql:"),
            "Atlas exposes an SQL-taking entry point, which item 13 forbids: {signature}"
        );
    }
    assert!(
        signatures > 20,
        "the signature scan must actually be walking Atlas's public API, saw {signatures}"
    );

    // 2. Every statement is a literal. The only interpolation any SQL-building
    //    format string performs is `reader_call(format)`, a compile-time
    //    constant chosen by a three-variant enum — the operator's own path is
    //    bound as a `?` parameter, never pasted in.
    //
    //    This walks db.rs's string literals character by character, because a
    //    line-based scan is evadable three ways: a literal rustfmt split
    //    across lines, a *named* hole (`{table}`) rather than a positional
    //    one, and a hole sitting on a continuation line that carries no SQL
    //    keyword of its own. Every hole in every SQL literal is collected
    //    wherever it sits, and the resulting list is pinned exhaustively — the
    //    same discipline item 12 applies to the CLI's `AtlasDb::` call list.
    let holes = sql_literal_holes(&db, &lines);
    assert_eq!(
        holes.iter().map(String::as_str).collect::<Vec<_>>(),
        vec![
            "rows_sql: {} <- reader_call(format)",
            "row_count_sql: {} <- reader_call(format)",
            "column_profile_sql: {} <- reader_call(format)",
        ],
        "the exhaustive list of (function, hole, filler) triples for every interpolation in \
         every SQL literal in db.rs. A new hole — positional or named, on any line — is a \
         query-surface decision, not a refactor"
    );

    // 3. The verb set is closed, and the two deferred verbs stay deferred. The
    //    verb list itself, not the prose around it — `map --help`'s own text
    //    explains why `neighbors` and `changed` are absent, and naming them is
    //    the honest thing for it to do.
    let help = Command::new(SGT)
        .args(["map", "--help"])
        .output()
        .expect("sgt map --help");
    let text = String::from_utf8_lossy(&help.stdout);
    let verbs: BTreeSet<&str> = text
        .split("Commands:")
        .nth(1)
        .unwrap_or("")
        .split("Options:")
        .next()
        .unwrap_or("")
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter(|verb| *verb != "help")
        .collect();
    assert_eq!(
        verbs,
        BTreeSet::from(["repos", "stats", "outline", "symbol", "references"]),
        "`sgt map` ships F11's five verbs and no others — `neighbors` and `changed` land \
         with the waves whose consumers need them: {text}"
    );
    for forbidden in ["--sql", "--query", "--where"] {
        assert!(
            !text.contains(forbidden),
            "`sgt map` must expose no query surface, found {forbidden}: {text}"
        );
    }
}

// --------------------------------------------------- item 14: the inherited pins

/// §17.14 — the inherited contract tests are still *there*.
///
/// "Remain green" is proven by running them, and this wave's closeout commit
/// records that run. What a test can add is the half a green run cannot show:
/// that the pins still exist. A deleted test never fails.
#[test]
fn a1a_item_14_the_inherited_contract_pins_still_exist() {
    // One representative, load-bearing pin from each family §17.14 names.
    const PINS: &[(&str, &str, &str)] = &[
        (
            "exact-root",
            "tests/m8_estate_cli.rs",
            "a_descendant_of_the_estate_root_finds_no_estate_and_spawns_nothing",
        ),
        (
            "Work-surface",
            "tests/m12_child_work.rs",
            "a_bare_sgt_run_from_a_work_surface_still_refuses_while_dash_c_works",
        ),
        (
            "distro-route/edition",
            "tests/m8_estate_cli.rs",
            "init_written_templates_carry_the_binary_edition",
        ),
        (
            "distro-route/edition",
            "tests/m8_estate_cli.rs",
            "init_writes_the_embedded_distro",
        ),
        (
            "split-hardening output contract",
            "tests/m11_nested_workflow.rs",
            "a_nested_leafs_own_output_contract_is_enforced_exactly_as_a_flat_ones_is",
        ),
        (
            "split-hardening output contract",
            "tests/m3_execution.rs",
            "t10_a_stage_completed_without_its_declared_output_is_reprompted_then_needs_input",
        ),
    ];

    for (family, file, test) in PINS {
        let text = read(file);
        assert!(
            text.contains(&format!("fn {test}(")),
            "the {family} pin `{test}` is gone from {file}; §17.14 is about these surviving"
        );
    }
}
