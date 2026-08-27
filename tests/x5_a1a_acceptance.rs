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
//!    with a destination sprint, never as "met". Item 4 is such a row, and
//!    [`a1a_item_4_gap_cloud_placeholder_detection_is_not_shipped`] is its
//!    tripwire: it fails the day someone ships the heuristic without updating
//!    the walk, so the gap cannot quietly close or quietly widen.
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
//! | 2 | estate Git indexing pins exact repo/base generations and Work overlays without changing Work authority | met | `x3a_git_plumbing::a_scan_stays_on_its_pinned_sha_while_head_advances` |
//! | 3 | a declared read-only local knowledge Source indexes a cloud-synced ordinary directory without becoming `[[repo]]` or receiving a worktree | met | `x5_a1a_acceptance::a1a_item_3_a_knowledge_source_is_indexed_without_becoming_a_repo_or_getting_a_worktree` |
//! | 4 | online-only/unreadable local resources are reported as coverage gaps, not silently indexed as empty | gap | `x5_a1a_acceptance::a1a_item_4_gap_cloud_placeholder_detection_is_not_shipped` |
//! | 5 | Markdown/text and at least one Office format normalize into document units with provenance | deferred-s4 | — |
//! | 6 | CSV/JSON/Parquet stay relational, with a deterministic aggregate and selected text-field context units sharing row identity | met | `x4_tabular_map::datasets_are_registered_and_read_in_place_as_derived_evidence` |
//! | 7 | a bounded ZIP exposes child resources while rejecting unsafe paths and enforcing ceilings | deferred-s4 | — |
//! | 8 | `.eml` or the chosen first mail format produces structured message evidence | deferred-s4 | — |
//! | 9 | image/scanned evidence enters the OCR fallback with page/region/engine provenance | deferred-s4 | — |
//! | 10 | external Git acquisition resolves an exact commit in a no-Work-checkout cache | deferred-s4 | — |
//! | 11 | source parsing uses content/extractor identity so unchanged resources reuse cached facts | met | `x2_knowledge_sources::a_scan_records_once_reuses_an_unchanged_generation_and_evicts_a_changed_one` |
//! | 12 | daemon remains sole Atlas writer and worker failure cannot corrupt journal authority | met-with-deviation | `x5_a1a_acceptance::a1a_item_12_no_atlas_write_path_is_reachable_from_the_cli` |
//! | 13 | `sgt map`/status surfaces expose source/generation/coverage rather than arbitrary SQL | met | `x5_a1a_acceptance::a1a_item_13_no_client_sql_reaches_the_store` |
//! | 14 | all existing exact-root, Work-surface, distro-route/edition and split-hardening output-contract tests remain green | met | `x5_a1a_acceptance::a1a_item_14_the_inherited_contract_pins_still_exist` |
//!
//! The `Decisive check` column names **one** check per row so the table stays
//! readable; every row's full check list — several rows carry four — is in
//! [`WALK`] below, and each one is verified to exist.
//!
//! ## One cross-cutting gap, named once
//!
//! Every capability item above is met **as a capability**: the writers exist,
//! they are called through their real entry points, and the read surfaces
//! answer over the wire and through the verb. What S3 did not ship is an
//! operator-reachable *trigger*: no CLI verb, no route, and no daemon job
//! calls a scan, so on a real installation Atlas stays empty until something
//! invokes it, and `sgt doctor` correctly says so. That is not a defect in any
//! one item — it is the seam the intelligence consumer will land on — but it
//! is exactly the sort of thing an acceptance walk exists to say out loud, so
//! [`a1a_cross_cutting_gap_no_shipped_surface_triggers_a_scan`] pins it and
//! items 3, 6 and 13 name it. Destination: the wave that owns the first
//! consumer of derived evidence (S5's retrieval work at the earliest).

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
}

impl Verdict {
    fn as_str(self) -> &'static str {
        match self {
            Self::Met => "met",
            Self::MetWithDeviation => "met-with-deviation",
            Self::Gap => "gap",
            Self::DeferredS4 => "deferred-s4",
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
        verdict: Verdict::Met,
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
        ],
        note: "'Without changing Work authority' is the read-only half: the scan runs only \
               read-only git plumbing, touches no branch, no index and no worktree, and an \
               overlay's lifetime is its Work's.",
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
        verdict: Verdict::Gap,
        checks: &[
            at(
                "tests/x5_a1a_acceptance.rs",
                "a1a_item_4_gap_cloud_placeholder_detection_is_not_shipped",
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
        note: "THE UNREADABLE HALF IS MET: unreadable roots, unreadable datasets, unparseable \
               files and excluded paths all leave a named coverage row, never a silent absence. \
               THE ONLINE-ONLY HALF IS A GAP: F8's best-effort cloud-placeholder heuristic is \
               NOT shipped. A synced-but-not-materialized file that the filesystem answers as a \
               readable zero-byte file is indexed today with zero units — exactly the \
               'silently indexed as empty' case this item forbids. DESTINATION: S4, beside the \
               Office/Anydoc work that gives the heuristic its first real corpus. Not deferred \
               by a ruling; named here as unfinished A1a scope.",
    },
    Item {
        number: 5,
        verdict: Verdict::DeferredS4,
        checks: &[],
        note: "S4's, per the ratified Host+Atlas re-cut ('Anydoc office/docs spike+adoption \
               under S4') and the S3 sprint plan's panel adjudication finding 1, which corrected \
               a commissioning line that had mis-claimed this item for S3. The Markdown/text \
               half of the item does ship (see item 3's checks); the Office half — and with it \
               the item — is S4's.",
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
        verdict: Verdict::DeferredS4,
        checks: &[],
        note: "Bounded ZIP / container adapters are S4's (adapters and external Git), per the \
               S3 sprint plan's commissioning line and panel finding 1.",
    },
    Item {
        number: 8,
        verdict: Verdict::DeferredS4,
        checks: &[],
        note: "Mail (`.eml`) evidence is S4's, same citation as item 7 — and gated behind its \
               own candidate spike in the contract's own wording.",
    },
    Item {
        number: 9,
        verdict: Verdict::DeferredS4,
        checks: &[],
        note: "OCR fallback is S4's, same citation as item 7, and likewise gated behind the OCR \
               candidate spike the item itself names.",
    },
    Item {
        number: 10,
        verdict: Verdict::DeferredS4,
        checks: &[],
        note: "External Git acquisition is S4's, same citation as item 7. The seam it will land \
               on exists and is named today (`SourceKind`'s external-git variant, `domain/\
               source.rs`), which is why S3 could pin generation identity without it.",
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
        verdict: Verdict::MetWithDeviation,
        checks: &[
            at(
                "tests/x5_a1a_acceptance.rs",
                "a1a_item_12_no_atlas_write_path_is_reachable_from_the_cli",
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
        ],
        note: "RESIDUAL, named rather than glossed: `sgt doctor`'s atlas coverage row opens the \
               store from the CLI process when no daemon holds the lock. It writes no fact — the \
               open runs idempotent `IF NOT EXISTS` DDL and then reads — but it is a read-write \
               open by a non-daemon process, so 'sole writer' is exact for facts and approximate \
               for the file handle. DESTINATION: S4, as a read-only open. Everything else holds: \
               no fact-writing path is reachable from the CLI at all, and a worker that dies \
               mid-scan leaves the journal authoritative in both crash windows.",
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
            Verdict::DeferredS4 => assert!(
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
    let deferred: BTreeSet<u8> = WALK
        .iter()
        .filter(|item| item.verdict == Verdict::DeferredS4)
        .map(|item| item.number)
        .collect();
    assert_eq!(
        deferred,
        BTreeSet::from([5, 7, 8, 9, 10]),
        "exactly items 5 and 7-10 are S4's, per the ratified re-cut"
    );

    // And every deferral has to say so in words a reader can check, not just
    // by its enum.
    for item in WALK.iter().filter(|i| i.verdict == Verdict::DeferredS4) {
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
            Verdict::DeferredS4 => assert_eq!(
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

// ------------------------------------------------------- item 4: the tripwire

/// §17.4's **named gap**, held open so it cannot close silently.
///
/// F8 promised best-effort cloud-placeholder detection, honestly labeled. S3
/// did not ship it: nothing in the Atlas tree reasons about a file that is
/// present-but-not-materialized, so such a file is read as the empty file the
/// filesystem presents and reported `indexed` with no units.
///
/// This test asserts the gap is exactly that shape — no detection, and no
/// coverage vocabulary pretending to describe one. It is deliberately a
/// tripwire, not a proof of absence: the day S4 lands the heuristic this
/// fails, and whoever lands it must come here, change the verdict, and name
/// the real check. That is the intended cost.
#[test]
fn a1a_item_4_gap_cloud_placeholder_detection_is_not_shipped() {
    let atlas = repo_root().join("src/runtime/atlas");
    let mut claims = Vec::new();
    for entry in fs::read_dir(&atlas).expect("read atlas dir") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text = fs::read_to_string(&path)
            .expect("read module")
            .to_lowercase();
        // Phrases specific to the unshipped feature. Deliberately not the bare
        // word "placeholder", which the store already uses in an unrelated
        // sense (the coverage row a walk leaves for a path a later step
        // replaces).
        for needle in [
            "cloud placeholder",
            "cloud-placeholder",
            "online-only",
            "online_only",
            "onedrive",
            "dehydrat",
            "reparse",
        ] {
            if text.contains(needle) {
                claims.push(format!("{} names {needle}", path.display()));
            }
        }
    }
    assert!(
        claims.is_empty(),
        "Atlas now appears to reason about cloud placeholders ({claims:?}). §17 item 4's \
         online-only half was walked at S3 close as an OPEN GAP with destination S4. If it \
         has been closed, update this file's register row 4 — verdict, note and decisive \
         check — instead of deleting this tripwire."
    );

    // The half that IS met, stated here too so the row is not purely negative:
    // the coverage vocabulary has a word for every way a resource can fail to
    // be indexed, and no word for "skipped".
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
            "generation_evicted",
        ]),
        "F8's coverage vocabulary is what makes an unreadable resource a reported gap"
    );
}

// ------------------------------------------- the cross-cutting gap, pinned

/// S3 shipped Atlas's writers and Atlas's readers and **no trigger between
/// them**.
///
/// Stated as a test rather than as a sentence in a report, because a sentence
/// cannot notice when it stops being true. Nothing an operator can run — no
/// `sgt` verb, no route, no daemon job — starts a scan today; the only callers
/// of the record entry points outside the Atlas tree are tests. On a real
/// installation that means the store stays empty, `sgt intelligence status`
/// reports nothing indexed, and `sgt doctor`'s atlas row says so plainly,
/// which is honest but is not the same as being wired.
///
/// When the trigger lands, this fails. Whoever lands it should delete this
/// test and update the register's cross-cutting note — that is the intended
/// handshake, not a chore to route around.
#[test]
fn a1a_cross_cutting_gap_no_shipped_surface_triggers_a_scan() {
    // Neither client-facing surface offers one.
    for (file, what) in [("src/cli.rs", "the CLI"), ("src/daemon.rs", "the daemon")] {
        let text = read(file);
        assert!(
            !text.contains("scan_and_record"),
            "{what} now triggers an Atlas scan; §17's cross-cutting gap has closed — update \
             the register instead of deleting this test"
        );
    }

    // The API references the writer only from its own test module.
    let api = read("src/api.rs");
    let first_test_module = api
        .find("#[cfg(test)]")
        .expect("api.rs must have a test module for this check to mean anything");
    for (index, _) in api.match_indices("scan_and_record") {
        assert!(
            index > first_test_module,
            "src/api.rs calls the Atlas writer from production code; the cross-cutting gap \
             has closed — update the register"
        );
    }

    // And `sgt intelligence` still offers reading only — the verb set itself,
    // not the prose around it, which legitimately says "indexed" all over.
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
        BTreeSet::from(["status"]),
        "`sgt intelligence` ships exactly one, read-only verb: {text}"
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
        vec!["let sources = match AtlasDb::open(data_dir).and_then(|db| db.indexed_sources()) {"],
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

    // 1. No public API takes SQL. `query_identity` takes the SQL that ran, to
    //    hash it into provenance — it executes nothing — so it is named as the
    //    single allowed exception rather than matched around.
    for line in db.lines().filter(|line| line.contains("pub fn ")) {
        if line.contains("pub fn query_identity(") {
            continue;
        }
        assert!(
            !line.contains("sql:"),
            "Atlas exposes an SQL-taking entry point, which item 13 forbids: {line}"
        );
    }

    // 2. Every statement is a literal. The only interpolation any SQL-building
    //    format string performs is `reader_call(format)`, a compile-time
    //    constant chosen by a three-variant enum — the operator's own path is
    //    bound as a `?` parameter, never pasted in.
    let mut interpolations = Vec::new();
    for (index, line) in db.lines().enumerate() {
        let Some(open) = line.find('{') else { continue };
        if !line.contains("SELECT")
            && !line.contains("INSERT")
            && !line.contains("DELETE")
            && !line.contains("UPDATE")
            && !line.contains("FROM")
        {
            continue;
        }
        // `{}` in an SQL literal is a hole something fills. The filler is the
        // first line after the literal ends — so skip the literal's own
        // continuation lines, which end in a backslash or close the string.
        if line[open..].starts_with("{}") {
            let filler = db
                .lines()
                .skip(index + 1)
                .map(str::trim)
                .find(|l| !l.is_empty() && !l.ends_with('\\') && !l.ends_with("\","))
                .unwrap_or("");
            interpolations.push(filler.to_string());
        }
    }
    assert!(
        !interpolations.is_empty(),
        "this check must actually find the canned readers' format strings"
    );
    for filler in &interpolations {
        assert!(
            filler.starts_with("reader_call(format)"),
            "an SQL literal is interpolated with `{filler}`; the only fragment that may vary \
             is the enum-chosen reader call"
        );
    }

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
