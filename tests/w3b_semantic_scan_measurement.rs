//! S5 W3b — **what A2-07's exact cosine scan actually costs.**
//! Read-only with respect to the estate, `#[ignore]`d — run explicitly,
//! never in CI.
//!
//! A2-07 (**R1**): *"Use exact cosine first; defer ANN/vector DB"*, because
//! *"No measured corpus/latency yet proves approximate indexing is
//! required."* A2 §16 lists *"vector database/ANN engine before
//! measurement"* as an explicit non-goal, and A2 §6 says *"Do not add
//! ANN/vector DB machinery until actual corpus size/latency measurements
//! prove exact scanning inadequate."*
//!
//! **This file is that measurement, and it is the only thing that could ever
//! justify an index.** It does not decide anything: it prints
//! units-per-second and per-query wall time at three corpus sizes so a
//! later decision has a figure to point at instead of an intuition. The
//! figures it produced are recorded — dated, with this file named as the
//! method — in the estate's
//! `knowledge/evidence/perf/model2vec-footprint-and-scan-2026-08-30.md`.
//!
//! Shape copied from `tests/w1d_overlay_scan_measurement.rs` (R2): resolve
//! what it needs, skip loudly rather than fail when it is absent, print the
//! figures, write nothing into the estate.
//!
//! Honesty note, in the same spirit: this measures a synthetic corpus of
//! short one-sentence units on one host, in a **debug build**. Release
//! figures are not measured here and would be better; the shape — cost
//! linear in admissible units, dominated by embedding rather than by SQL —
//! is what the numbers demonstrate, and nothing here says anything about a
//! corpus two orders of magnitude larger than the largest run below.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::Instant;

use sergeant_rs::domain::event::rfc3339_utc_now;
use sergeant_rs::domain::source::{AuthorityClass, SourceKind, UnitKind};
use sergeant_rs::runtime::atlas::db::{Admissibility, AtlasDb, LexicalQuery, SourceSelector};
use sergeant_rs::runtime::atlas::record::record_scan;
use sergeant_rs::runtime::atlas::scan::{ScannedFile, ScannedUnit, SourceScan};
use sergeant_rs::runtime::atlas::semantic::{MODEL_DIR_ENV, SemanticRequest, SemanticStatus};
use sergeant_rs::runtime::atlas::tabular::ContextFields;
use sergeant_rs::runtime::atlas::text::MARKDOWN_EXTRACTOR;
use sergeant_rs::runtime::journal::Journal;

/// One sentence per unit, varied so no two units embed identically.
fn body(index: usize) -> String {
    const SUBJECTS: [&str; 8] = [
        "the settlement ledger",
        "the retry policy",
        "the configuration loader",
        "the disk-pressure alert",
        "the credential rule",
        "the scheduler",
        "the archive pruner",
        "the journal writer",
    ];
    format!(
        "Unit {index}: {} is described here, together with the reason it behaves the way \
         it does in generation {}.",
        SUBJECTS[index % SUBJECTS.len()],
        index / SUBJECTS.len()
    )
}

fn scan(units: usize) -> SourceScan {
    let files = (0..units)
        .map(|i| ScannedFile {
            relative_path: format!("docs/unit-{i:06}.md"),
            content_hash: format!("hash/{i}"),
            extractor: MARKDOWN_EXTRACTOR.to_string(),
            local_key: format!("key/{i}"),
            byte_len: 128,
            mtime_millis: None,
            units: vec![ScannedUnit {
                ordinal: 0,
                kind: UnitKind::Document,
                heading_level: None,
                title: None,
                byte_start: 0,
                byte_end: body(i).len() as u64,
                coordinate: None,
                text: body(i),
            }],
            syntax: None,
            parent: None,
        })
        .collect();
    let mut extractors = BTreeSet::new();
    extractors.insert(MARKDOWN_EXTRACTOR.to_string());
    SourceScan {
        source_name: "knowledge".to_string(),
        kind: SourceKind::LocalKnowledge,
        authority: AuthorityClass::EstateReadonly,
        content_key: format!("knowledge@{units}"),
        revision: None,
        observed_at: rfc3339_utc_now(),
        files,
        coverage: Vec::new(),
        extractors,
        datasets: Vec::new(),
        root: None,
        context_fields: ContextFields::none(),
    }
}

/// Point at the committed assets, or skip loudly. A measurement that
/// silently measured a `not_installed` no-op would be worse than none.
fn require_assets() -> bool {
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/semantic-model");
    if !assets.join("model.safetensors").is_file() {
        eprintln!(
            "SKIPPED: no semantic assets at {} — this measurement needs the real model",
            assets.display()
        );
        return false;
    }
    unsafe { std::env::set_var(MODEL_DIR_ENV, &assets) };
    true
}

/// Print per-query wall time and units/second at three corpus sizes.
///
/// **What a reader should take from the output.** The scan embeds every
/// admissible unit on every query — there is no cache and no index (A2-07)
/// — so the interesting number is units/second and whether the per-query
/// time at a realistic corpus size is tolerable. If a future corpus makes it
/// intolerable, *that* run is the evidence A2 §16 asks for before ANN
/// machinery may be considered, and this file is how to produce it.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn exact_cosine_scan_cost_by_corpus_size() {
    if !require_assets() {
        return;
    }
    println!(
        "{:>8}  {:>12}  {:>12}  {:>14}",
        "units", "first (ms)", "warm (ms)", "units/s (warm)"
    );
    for units in [100usize, 1_000, 5_000] {
        let data = tempfile::tempdir().expect("data dir");
        let mut journal = Journal::open(data.path()).expect("journal");
        let mut db = AtlasDb::open(data.path()).expect("atlas");
        record_scan(&mut db, &mut journal, &scan(units), None).expect("record");

        let filter = Admissibility {
            source: SourceSelector::Named("knowledge".to_string()),
            kind: None,
            authority: None,
        };
        let query = LexicalQuery {
            text: "why does the settlement ledger behave this way",
            filter: &filter,
            family: None,
            limit: 20,
            semantic: SemanticRequest::Requested,
        };

        // The first search pays the one-time model load (32 MB of weights
        // plus a 1 MB tokenizer); every later one on the same handle does
        // not. Both are reported, because a caller that opens a handle per
        // query would pay the first number every time.
        let started = Instant::now();
        let answer = db.semantic_search(&query).expect("semantic search");
        let first = started.elapsed();
        assert_eq!(
            answer.semantic,
            SemanticStatus::Applied,
            "the measurement must have measured a real scan"
        );

        let mut warm = std::time::Duration::ZERO;
        const RUNS: u32 = 5;
        for _ in 0..RUNS {
            let started = Instant::now();
            db.semantic_search(&query).expect("semantic search");
            warm += started.elapsed();
        }
        let warm = warm / RUNS;
        println!(
            "{units:>8}  {:>12.1}  {:>12.1}  {:>14.0}",
            first.as_secs_f64() * 1000.0,
            warm.as_secs_f64() * 1000.0,
            units as f64 / warm.as_secs_f64(),
        );
    }
}
