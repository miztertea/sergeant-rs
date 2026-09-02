//! S5 W5 — A2 §13's search trace, A2 §14's two verbs, and the three A2 §17
//! items the mid-sprint acceptance walk found unassigned (items 3, 6, 8).
//!
//! # What each test here is the evidence for
//!
//! | Claim | Test |
//! |---|---|
//! | §17 item 8 — external evidence is visibly external **from the answer alone** | [`an_external_hit_is_identifiable_as_external_from_the_answer_alone`] |
//! | §17 item 8 — and the same is true of a fused answer, not only a lexical one | [`a_fused_answer_carries_the_source_kind_and_authority_class_of_every_hit`] |
//! | §17 item 3 — a relational aggregate **joins to retrieved row evidence** | [`a_relational_aggregate_and_a_retrieved_row_join_on_one_shared_row_identity`] |
//! | §17 item 3 — and that relational read is **reachable outside the test binary** | [`item_3s_relational_read_is_reachable_from_outside_the_process`] |
//! | §17 item 6 — one query **spans** a normalized Office document and a Markdown one | [`one_local_knowledge_query_spans_a_normalized_docx_and_a_markdown_file`] |
//! | §13 — all nine trace fields are recorded | [`the_trace_records_every_one_of_a2_section_13s_nine_fields`] |
//! | §13 — the tokenizer version is not a promise to remember | [`the_lexical_tokenizer_version_is_pinned_to_the_tokenizers_actual_output`] |
//! | §13 — nor is the RRF/rerank policy version | [`the_retrieval_policy_version_is_pinned_to_the_actual_rrf_and_rerank_policy`] |
//! | H4 — `semantic:` is on the trace and is never absent | [`the_trace_states_the_semantic_status_even_when_the_caller_suppressed_it`] |
//! | §14 — the printed coordinate is the coordinate `related` accepts | [`a_printed_coordinate_parses_back_to_itself`] |
//! | §14 — `related` returns neighbours, never its own anchor | [`related_returns_real_neighbours_and_never_its_own_anchor`] |
//! | A2 §8 — `related` cannot be used to walk out of the filter | [`related_refuses_an_anchor_the_admissibility_filter_excludes`] |
//! | §14 — no retrieval weight is exposed on the CLI | [`the_search_cli_exposes_a2_section_14s_selectors_and_no_weight_knob`] |
//! | H13.2 — `sgt search` is a pure reader | [`the_search_and_related_routes_reach_atlas_through_the_read_only_handle`] |
//! | H13.1 — `--content config` is refused with its reason, never half-answered | [`content_config_is_refused_by_name_rather_than_answered_partially`] |
//!
//! # The live-estate acceptance is not in this file, and could not be
//!
//! The wave's own acceptance — `sgt search` finding a real symbol in this
//! estate's mounted `sergeant-rs`, and `sgt related` returning real
//! neighbours for a real coordinate — is a run against a real daemon over a
//! real estate, recorded in the wave's report. Three capabilities in this
//! program shipped green-in-tests and unreachable in practice; a suite is
//! what proves the mechanism, and a live run is what proves it is reachable.
//! Neither substitutes for the other and this file does not pretend to be
//! both.

/// **S6 D1 — A2 §2 stage 1's estate coordinate.** This suite is
/// single-estate: every generation it records is bound to this one root and
/// every filter it builds is admitted from it. The cross-estate case — two
/// estates on one host daemon, which is where the axis actually earns its
/// keep — is `tests/d1_estate_isolation.rs`, deliberately not folded in
/// here, because a suite that never crosses estates cannot notice an estate
/// filter that does nothing (that is exactly how the leak survived: this
/// file's ancestors all passed).
#[allow(dead_code)]
const D1_ESTATE: &str = "/estates/w5_search_surface";

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tempfile::TempDir;

use sergeant_rs::domain::event::rfc3339_utc_now;
use sergeant_rs::domain::source::{AuthorityClass, SourceKind, UnitKind};
use sergeant_rs::runtime::atlas::db::{
    Admissibility, AtlasDb, DATASET_COLUMN_PROFILE, FusedAnswer, LexicalQuery, RelatedRequest,
    SourceSelector,
};
use sergeant_rs::runtime::atlas::fusion::{
    BOOST_EXACT_MATCH, FILE_COHERENCE_BOOST_FRAC, FILE_SATURATION_DECAY, PENALTY_NON_CANONICAL,
    RRF_K, RerankSignals, STEM_BOOST_MULTIPLIER, STEM_MATCH_MIN_RATIO,
};
use sergeant_rs::runtime::atlas::lexical::{LexicalFamily, UnitAddress, UnitCoordinate, tokenize};
use sergeant_rs::runtime::atlas::record::record_scan;
use sergeant_rs::runtime::atlas::scan::{
    KnowledgeSource, ScannedFile, ScannedUnit, SourceScan, scan_local_knowledge,
    scan_local_knowledge_with_worker,
};
use sergeant_rs::runtime::atlas::semantic::{SemanticRequest, SemanticStatus};
use sergeant_rs::runtime::atlas::tabular::{ContextFields, DatasetFormat};
use sergeant_rs::runtime::atlas::text::MARKDOWN_EXTRACTOR;
use sergeant_rs::runtime::atlas::trace::{
    Attribution, LEXICAL_TOKENIZER_VERSION, RETRIEVAL_POLICY_VERSION, SearchTrace,
};
use sergeant_rs::runtime::atlas::worker::WorkerRuntime;
use sergeant_rs::runtime::journal::Journal;

/// The real worker binary Cargo built alongside this test binary — the same
/// one `tests/y2_office_adapter.rs` and `tests/w7_container_children.rs`
/// drive. Item 6 is about a **normalized Office document**, so the `.docx`
/// below goes through the real adapter in a real subprocess; a hand-built
/// unit claiming the office extractor's identity would prove the search half
/// and quietly assume the half item 6 is actually about.
const SGT_ATLAS_WORKER: &str = env!("CARGO_BIN_EXE_sgt-atlas-worker");

// ---------------------------------------------------------------- fixtures

/// The shared term item 6's one query is run on: it appears in the `.docx`
/// (through the office adapter's normalization) and in the Markdown file.
const SPANNING_TERM: &str = "Heading";

/// A Markdown document under the same knowledge source as the `.docx`.
const NOTES_MD: &str = "\
# Heading conventions

This note is about Heading levels and how a Heading becomes a section.
";

/// A CSV whose allowlisted columns become `context.row_units` — item 3's
/// retrieved row evidence, and the file item 3's relational aggregate is
/// computed over.
const TICKETS_CSV: &str = "\
number,short_description
INC0012345,PaymentRetryPolicy timed out during settlement
INC0099999,unrelated printer jam
";

fn write(root: &Path, relative: &str, body: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create fixture directory");
    }
    std::fs::write(&path, body).expect("write fixture");
}

fn worker() -> WorkerRuntime {
    WorkerRuntime {
        program: PathBuf::from(SGT_ATLAS_WORKER),
        deadline: Duration::from_secs(30),
    }
}

/// One of the repository's real `.docx` fixtures — a genuine Office file,
/// not a renamed text file.
fn docx_fixture(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/anydoc_corpus/docx_fixtures")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()))
}

fn document_unit(title: Option<&str>, text: &str) -> ScannedUnit {
    ScannedUnit {
        ordinal: 0,
        kind: UnitKind::Document,
        heading_level: None,
        title: title.map(str::to_string),
        byte_start: 0,
        byte_end: text.len() as u64,
        coordinate: None,
        text: text.to_string(),
    }
}

/// A hand-built scan, for the one source whose *kind* is the point rather
/// than its bytes — the external one item 8 is about.
fn hand_built_scan(
    source_name: &str,
    kind: SourceKind,
    authority: AuthorityClass,
    relative_path: &str,
    body: &str,
) -> SourceScan {
    let file = ScannedFile {
        relative_path: relative_path.to_string(),
        content_hash: format!("hash/{relative_path}"),
        // Real BLAKE3 of the real bytes, not a fake string — this is the
        // bug-2 identity-merge key (F-IN-01's own territory), and a fixture
        // whose digest does not actually match its bytes would silently
        // stop exercising the merge it claims to.
        content_digest: sergeant_rs::domain::source::content_hash(body.as_bytes()),
        extractor: MARKDOWN_EXTRACTOR.to_string(),
        local_key: format!("key/{relative_path}"),
        byte_len: body.len() as u64,
        mtime_millis: None,
        units: vec![document_unit(None, body)],
        syntax: None,
        parent: None,
    };
    let mut extractors = BTreeSet::new();
    extractors.insert(MARKDOWN_EXTRACTOR.to_string());
    SourceScan {
        source_name: source_name.to_string(),
        kind,
        authority,
        content_key: format!("{source_name}@key-1"),
        revision: None,
        observed_at: rfc3339_utc_now(),
        files: vec![file],
        coverage: Vec::new(),
        extractors,
        datasets: Vec::new(),
        root: None,
        identity_root: None,
        context_fields: ContextFields::none(),
    }
}

/// The estate every test below queries.
///
/// * `library` — a **real** walk over a real directory through the **real**
///   Office worker: `notes.md` (Markdown), `report.docx` (a genuine `.docx`
///   normalized by the adapter), and `tickets.csv` with `number`/
///   `short_description` allowlisted (F10a).
/// * `vendor-lib` — `external_git`/`external`, carrying the same words, so
///   item 8's question ("is this hit external?") has two candidate answers
///   and the test is not vacuous.
struct Estate {
    _data: TempDir,
    root: TempDir,
    db: AtlasDb,
}

impl Estate {
    fn search(
        &self,
        text: &str,
        filter: &Admissibility,
        family: Option<LexicalFamily>,
    ) -> FusedAnswer {
        self.db
            .fused_search(&LexicalQuery {
                text,
                filter,
                family,
                limit: 50,
                semantic: SemanticRequest::Requested,
            })
            .expect("fused search")
    }

    fn traced(
        &self,
        text: &str,
        filter: &Admissibility,
        semantic: SemanticRequest,
    ) -> (FusedAnswer, SearchTrace) {
        self.db
            .traced_search(
                &LexicalQuery {
                    text,
                    filter,
                    family: None,
                    limit: 50,
                    semantic,
                },
                Attribution::Unmanaged,
            )
            .expect("traced search")
    }
}

fn estate() -> Estate {
    let data = tempfile::tempdir().expect("data dir");
    let root = tempfile::tempdir().expect("knowledge root");
    write(root.path(), "notes.md", NOTES_MD);
    write(root.path(), "tickets.csv", TICKETS_CSV);
    std::fs::write(
        root.path().join("report.docx"),
        docx_fixture("01-plain-headings-paragraphs.docx"),
    )
    .expect("write docx fixture");

    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");

    let library = KnowledgeSource {
        name: "library".to_string(),
        root: root.path().to_path_buf(),
        ignore: Vec::new(),
        context_fields: ContextFields::declared(&[
            "number".to_string(),
            "short_description".to_string(),
        ]),
    };
    // The real adapter, in a real subprocess — `scan_local_knowledge` alone
    // would report the `.docx` `unsupported` and item 6 would be testing
    // nothing.
    let scan = scan_local_knowledge_with_worker(&library, &worker()).expect("scan library");
    record_scan(
        &mut db,
        &mut journal,
        &scan,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record library");

    let vendor = hand_built_scan(
        "vendor-lib",
        SourceKind::ExternalGit,
        AuthorityClass::External,
        "docs/vendor.md",
        "Heading conventions in the vendor's own manual: Heading, Heading, Heading.",
    );
    record_scan(
        &mut db,
        &mut journal,
        &vendor,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record vendor-lib");

    Estate {
        _data: data,
        root,
        db,
    }
}

fn any() -> Admissibility {
    Admissibility::within_estate(D1_ESTATE)
}

fn only_library() -> Admissibility {
    Admissibility {
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: SourceSelector::Named("library".to_string()),
        kind: None,
        authority: None,
    }
}

// ============================================================ §17 item 8

/// **A2 §17 item 8**: *"external evidence remains **visibly** external."*
///
/// Visibility is a property of the ANSWER. Before this wave a hit carried
/// `source_name` and nothing else about its world, so a caller holding an
/// answer had a *name* and no taxonomy — it could not tell an `external_git`
/// source from an `estate_git` one without a second lookup it has no bounded
/// surface for.
///
/// This is the pin, and it is deliberately shaped so a second store read
/// cannot rescue it: the answer is destructured into plain values, the
/// database handle is **dropped**, and the external hit is identified from
/// those values alone.
#[test]
fn an_external_hit_is_identifiable_as_external_from_the_answer_alone() {
    let estate = estate();
    let answer = estate.search(SPANNING_TERM, &any(), None);

    // Both worlds are in the answer, or the question is vacuous.
    let sources: BTreeSet<&str> = answer.hits.iter().map(|h| h.source_name.as_str()).collect();
    assert!(
        sources.contains("library") && sources.contains("vendor-lib"),
        "both an estate source and an external one must be in this answer, \
         or 'can you tell them apart' is not being asked: {sources:?}"
    );

    // Everything a consumer receives, and nothing else.
    let received: Vec<(String, SourceKind, AuthorityClass)> = answer
        .hits
        .iter()
        .map(|hit| {
            (
                hit.source_name.clone(),
                hit.source_kind,
                hit.authority_class,
            )
        })
        .collect();
    drop(estate);

    let external: Vec<&(String, SourceKind, AuthorityClass)> = received
        .iter()
        .filter(|(_, kind, authority)| {
            *kind == SourceKind::ExternalGit && *authority == AuthorityClass::External
        })
        .collect();
    assert!(
        !external.is_empty(),
        "an external hit must be identifiable as external from the answer alone: {received:?}"
    );
    assert!(
        external.iter().all(|(name, _, _)| name == "vendor-lib"),
        "and only the external source's hits may say so: {external:?}"
    );
    assert!(
        received
            .iter()
            .any(|(name, kind, authority)| name == "library"
                && *kind == SourceKind::LocalKnowledge
                && *authority == AuthorityClass::EstateReadonly),
        "the estate-side hits must carry their own kind, not a default: {received:?}"
    );
}

/// **F-SF-01 / `brief-search-three-bugs.md` Bug 3** — *"the same (source,
/// path) twice in one top-5"*, through the **real** scan → index → query
/// pipeline (Standing Requirement 5), not a hand-built `FusedHit` list.
///
/// `crowded.md` has six sections that each match the query term heavily;
/// `sparse.md` mentions it once. Non-vacuous the same way `tests/
/// w4_rrf_fusion.rs::
/// a_second_chunk_of_the_same_file_is_decayed_so_one_file_cannot_take_every_slot`
/// is: without `fusion::dedup_file_rows`, `crowded.md`'s six units all
/// outscore `sparse.md`'s one, and several of them individually survive
/// `FILE_SATURATION_DECAY` — decay alone thins them, it does not collapse
/// them to a single file-level row. This asserts the row count directly:
/// `crowded.md` may appear **at most once** in the final answer.
#[test]
fn a_file_with_many_matching_units_appears_once_in_file_level_results() {
    let data = tempfile::tempdir().expect("data dir");
    let root = tempfile::tempdir().expect("knowledge root");
    let mut crowded = String::new();
    for i in 0..6 {
        crowded.push_str(&format!(
            "## Section {i}\n\nwidgetcraft widgetcraft widgetcraft, discussion {i} of widgetcraft.\n\n"
        ));
    }
    write(root.path(), "crowded.md", &crowded);
    write(
        root.path(),
        "sparse.md",
        "# Sparse\n\nwidgetcraft is mentioned here exactly once.\n",
    );

    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");
    let source = KnowledgeSource {
        name: "library".to_string(),
        root: root.path().to_path_buf(),
        ignore: Vec::new(),
        context_fields: ContextFields::none(),
    };
    let scan = scan_local_knowledge(&source).expect("scan library");
    record_scan(
        &mut db,
        &mut journal,
        &scan,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record library");

    let answer = db
        .fused_search(&LexicalQuery {
            text: "widgetcraft",
            filter: &Admissibility::within_estate(D1_ESTATE),
            family: None,
            limit: 10,
            semantic: SemanticRequest::Suppressed,
        })
        .expect("fused search");

    let paths: Vec<&str> = answer
        .hits
        .iter()
        .map(|h| h.coordinate.relative_path())
        .collect();
    let crowded_rows = paths.iter().filter(|p| **p == "crowded.md").count();
    assert_eq!(
        crowded_rows, 1,
        "crowded.md's many matching units surfaced as {crowded_rows} file-level rows, not one: {paths:?}"
    );
    assert!(
        paths.contains(&"sparse.md"),
        "sparse.md was crowded out of the answer entirely: {paths:?}"
    );
}

/// Item 8 again, through **the fused answer specifically**.
///
/// `FusedHit` is built by `fusion::fuse` from two input lists, so item 8's
/// two fields could have been correct on `LexicalHit`, correct on
/// `SemanticHit`, and dropped in the join. They are not: every fused hit's
/// pair equals the pair its own source generation was admitted with.
#[test]
fn a_fused_answer_carries_the_source_kind_and_authority_class_of_every_hit() {
    let estate = estate();
    let answer = estate.search(SPANNING_TERM, &any(), None);
    assert!(!answer.hits.is_empty(), "the query must find something");
    let admitted = estate
        .db
        .admissible_generations(&any(), 100)
        .expect("admissible generations");
    for hit in &answer.hits {
        let generation = admitted
            .hits
            .iter()
            .find(|g| g.id == hit.generation_id)
            .unwrap_or_else(|| panic!("hit cites a generation the filter did not admit: {hit:?}"));
        assert_eq!(
            (hit.source_kind, hit.authority_class),
            (generation.kind, generation.authority),
            "a fused hit's item-8 pair must be its generation's own, not a default"
        );
    }
}

// ============================================================ §17 item 3

/// **A2 §17 item 3**: *"relational dataset queries remain available
/// independently of text retrieval **and can join to retrieved row
/// evidence**."*
///
/// The *independent* half was already met (A1's tabular lane). The **join**
/// half had no test, and "possible" is what the last four register overclaims
/// all said — so this performs the join rather than asserting it could be
/// performed.
///
/// The join key is A2 §4's own: *"Those context units carry the same row
/// identity as the relational table. This lets a deterministic aggregate and
/// retrieved example tickets be joined/cited together instead of becoming two
/// unrelated representations."* So:
///
/// 1. the **relational** side — `dataset_probe`'s canned column profile over
///    the CSV, computed with no retrieval involved at all;
/// 2. the **retrieval** side — one `row-text` search returning one row;
/// 3. the **join** — the retrieved row's `dataset_key` is the dataset the
///    aggregate answered about, its `relative_path` is the file the aggregate
///    read, its `fields` are columns that aggregate profiled, and its
///    `row_key` is a value of the key column the aggregate counted. Every
///    step is an equality between two values neither side invented.
#[test]
fn a_relational_aggregate_and_a_retrieved_row_join_on_one_shared_row_identity() {
    let estate = estate();

    // (1) The relational lane, independently of text retrieval.
    let csv = estate.root.path().join("tickets.csv");
    let aggregate = estate
        .db
        .dataset_probe(
            DatasetFormat::Csv,
            csv.to_str().expect("utf-8 fixture path"),
            &DATASET_COLUMN_PROFILE,
        )
        .expect("relational aggregate");
    let profiled: BTreeSet<String> = aggregate
        .rows
        .iter()
        .filter_map(|row| row.first().cloned().flatten())
        .collect();
    assert!(
        profiled.contains("number") && profiled.contains("short_description"),
        "the aggregate must profile the dataset's columns: {profiled:?}"
    );

    // (2) The retrieval lane.
    let answer = estate.search("INC0012345", &only_library(), Some(LexicalFamily::RowText));
    let hit = answer
        .hits
        .iter()
        .find(|hit| hit.coordinate.relative_path() == "tickets.csv")
        .expect("a row-text hit for the ticket");
    let UnitCoordinate::RowText {
        relative_path,
        dataset_key,
        row_key,
        fields,
        ..
    } = &hit.coordinate
    else {
        panic!("expected a row-text coordinate, got {:?}", hit.coordinate);
    };

    // (3) The join, in both directions.
    let datasets = estate
        .db
        .admissible_datasets(&only_library(), 100)
        .expect("admissible datasets");
    let stored = datasets
        .hits
        .iter()
        .find(|d| &d.dataset.dataset_key == dataset_key)
        .unwrap_or_else(|| {
            panic!("the retrieved row's dataset key must name an admissible dataset: {dataset_key}")
        });
    assert_eq!(
        &stored.dataset.relative_path, relative_path,
        "the row the search returned and the dataset the aggregate read are one file"
    );
    // `dataset_probe` answers about a **path** the caller already holds and
    // deliberately carries no stored dataset key of its own (its
    // `ScannedDataset` is a probe envelope, not a recorded generation), so
    // the file itself is the join on this side — and it is the same file the
    // retrieved row's dataset row names.
    assert!(
        aggregate
            .relative_path
            .ends_with(&stored.dataset.relative_path),
        "the aggregate read the dataset the retrieved row belongs to: {} vs {}",
        aggregate.relative_path,
        stored.dataset.relative_path
    );
    for field in fields {
        assert!(
            profiled.contains(field),
            "every exposed field of the retrieved row is a column the aggregate profiled: \
             {field} not in {profiled:?}"
        );
    }
    // **The join that matters**: the retrieved row resolves back to one
    // actual row of the dataset the aggregate answered about. `row_key` is a
    // content-derived digest (`RowKeyBasis`), not a column value, so the
    // positional half of A2 §3's `dataset/row-id/field-set` coordinate — the
    // `ordinal`, in the dataset reader's own order — is what addresses the
    // row inside the relational table.
    assert_eq!(
        stored.dataset.row_count, 2,
        "the relational row count is the dataset's, not the retrieval index's"
    );
    assert!(
        hit.coordinate.ordinal() < stored.dataset.row_count,
        "the retrieved row's ordinal addresses a row the aggregate counted: {} of {}",
        hit.coordinate.ordinal(),
        stored.dataset.row_count
    );
    let text = std::fs::read_to_string(&csv).expect("read the dataset back");
    let rows: Vec<&str> = text.lines().skip(1).collect();
    assert!(
        rows[hit.coordinate.ordinal() as usize].contains("INC0012345"),
        "and it addresses the RIGHT row: the aggregate's row {} is {:?}",
        hit.coordinate.ordinal(),
        rows[hit.coordinate.ordinal() as usize]
    );
    assert!(
        !row_key.is_empty(),
        "the row also carries its own stable identity, so two answers about it join"
    );
}

/// **A2 §17 item 3, the reachability half** (S5 closeout F-AC-03).
///
/// [`a_relational_aggregate_and_a_retrieved_row_join_on_one_shared_row_identity`]
/// really performs the join — but it performed it entirely through `pub`
/// library functions that, at the close of S5, had no caller anywhere in
/// `src/`: `dataset_probe`, `dataset_facts` and `admissible_datasets`. An
/// acceptance item met only inside the test binary is this program's
/// signature defect, and it is what this test exists to make impossible to
/// repeat. It pins two things:
///
/// 1. the aggregate half of the join is served by the read the daemon route
///    actually calls — `dataset_facts`, which reads only rows the store
///    already holds — and that read joins to a retrieved row on
///    `dataset_key`, A2 §4's shared row identity;
/// 2. that read has a daemon route AND a CLI verb, in the shape
///    `GET /v1/map/children` / `sgt map children` set in W7.
///
/// `dataset_probe` is deliberately *not* reachable and is asserted so: its
/// own doc says `path` is a file path this process will open, *"so this is
/// not, and must not become, an HTTP-reachable surface"*. The distinction is
/// the point — one of these two reads is safe to expose and one is not, and
/// "neither is exposed" was not the same as "the unsafe one is not".
#[test]
fn item_3s_relational_read_is_reachable_from_outside_the_process() {
    let estate = estate();

    // (1) The aggregate, through the read the route calls — no probe, no
    // path, no client-named file: rows the store already holds.
    let facts = estate
        .db
        .dataset_facts("library", 100)
        .expect("stored relational evidence");
    assert!(
        !facts.is_empty(),
        "a scanned dataset lands its canned queries' answers"
    );

    // The retrieval half, and the join key.
    let answer = estate.search("INC0012345", &only_library(), Some(LexicalFamily::RowText));
    let hit = answer
        .hits
        .iter()
        .find(|hit| hit.coordinate.relative_path() == "tickets.csv")
        .expect("a row-text hit for the ticket");
    let UnitCoordinate::RowText {
        relative_path,
        dataset_key,
        ..
    } = &hit.coordinate
    else {
        panic!("expected a row-text coordinate, got {:?}", hit.coordinate);
    };
    let fact = facts
        .iter()
        .find(|f| &f.dataset_key == dataset_key)
        .unwrap_or_else(|| {
            panic!("the retrieved row's dataset key must name stored evidence: {dataset_key}")
        });
    assert_eq!(
        &fact.relative_path, relative_path,
        "the aggregate and the retrieved row cite one file"
    );
    assert!(
        !fact.query_identity.is_empty() && !fact.output_hash.is_empty(),
        "and the aggregate is checkable, not merely present: {fact:?}"
    );

    // (2) The surface. Without this, (1) is another green test for a
    // capability nothing can call.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let api = std::fs::read_to_string(manifest.join("src/api.rs")).expect("read src/api.rs");
    let cli = std::fs::read_to_string(manifest.join("src/cli.rs")).expect("read src/cli.rs");
    assert!(
        api.contains("atlas.dataset_facts("),
        "item 3's relational read must have a daemon route that calls it — \
         `dataset_facts` reached no production caller at the close of S5"
    );
    assert!(
        api.contains(r#".route("/map/facts", get(map_facts))"#),
        "and that route must be mounted, not merely written"
    );
    assert!(
        cli.contains("/v1/map/facts?source="),
        "and a CLI verb must reach it, in the shape `sgt map children` set"
    );
    // The negative half: the producer that opens a file the caller names
    // stays off the wire, exactly as its own doc requires. A *call*, not a
    // mention — the route's own doc names it to say why it is absent.
    assert!(
        !api.contains(".dataset_probe("),
        "`dataset_probe` opens a path the caller supplies and must never \
         become an HTTP-reachable surface (its own doc)"
    );
    assert!(
        !cli.contains("dataset_probe"),
        "and no CLI verb may reach it either"
    );
}

// ============================================================ §17 item 6

/// **A2 §17 item 6**: *"local knowledge Source searches can span normalized
/// Office/docs without losing original path/source coordinates."*
///
/// The *"without losing original path"* half already held —
/// `UnitCoordinate::Document` carries `relative_path`, and Y2 proved a
/// `.docx` worker returns document/section units with provenance. Unproven
/// was the **span**: ONE query over ONE `local_knowledge` source matching
/// BOTH a normalized Office document and an ordinary Markdown one, with both
/// coordinates intact in the same answer.
///
/// The `.docx` here is a real Office file read by the real adapter in a real
/// subprocess (see [`estate`]); the Markdown is a real file on the same root.
///
/// # Which coordinates carry a real byte span, and which honestly do not
///
/// *"Coordinates intact"* is not *"every hit cites a byte range"*, and this
/// test asserts the true version rather than the convenient one:
///
/// * **Markdown** is its own bytes, so every document-family unit of
///   `notes.md` carries a real span into the original file and no native
///   coordinate — the span **is** the address. (The same file also produces
///   a `Code`-family hit, its tree-sitter heading; item 6 is about the
///   document family, so the span claims below skip it.)
/// * **A `.docx`** can promise that for exactly one unit. The whole-document
///   unit truthfully spans the whole input, in any container format. Every
///   **section** unit carries `0`/`0` **by design** — `sgt-atlas-worker`'s
///   own `normal_batch` calls that "not applicable" honestly, because the
///   original bytes are a compressed ZIP/XML container and no offset into
///   them corresponds to a position in the normalized text
///   (`office::OfficeUnit`'s own doc). What a section cites instead is
///   [`UnitCoordinate::Document`]'s `native` — the normalizer's own address,
///   `block:<n>` — and that is what this test holds an Office section to.
///
/// # "Office", in this build, means eleven formats (S6)
///
/// A2 §9's own example output shows a `.pptx` slide and a `.docx` section
/// side by side, and §9's prose names Word, PowerPoint, Excel, PDF and mail
/// as things a local knowledge Source *may contain*. This build claims all
/// of them and more: `office::OFFICE_EXTENSIONS` routes `.doc`, `.docx`,
/// `.epub`, `.odp`, `.ods`, `.odt`, `.pdf`, `.ppt`, `.pptx`, `.rtf` and
/// `.xlsx` (case-insensitively), pinned by
/// `office::tests::extractor_for_claims_eleven_formats_by_canonical_extension`.
///
/// This paragraph used to say the opposite — *"this build claims one of
/// them: `.docx`"* — and cited the Y2 wave brief's own scope boundary as
/// justification. The owner ruled that boundary a narrowing rather than a
/// scope decision once its spike had ended
/// (`twelve-formats-is-0.3.0-criteria-2026-08-30`): *"1/12 is a failure of
/// 0.3.0 completion criteria for estate intelligence."* S6 routed the rest.
///
/// The twelfth format the normalizer parses, `csv`, is deliberately still
/// unclaimed here and belongs to the relational lane instead (A1 §6.4,
/// A1-13) — see `office::CSV_IS_NOT_A_DOCUMENT`. A path no table claims
/// still gets an honest `unsupported` coverage row rather than a unit.
///
/// Item 6's "span" is proven below for `.docx`, the format whose corpus
/// carries hand-verified per-field counts; the section-coordinate claim it
/// rests on (`0`/`0` byte span, `block:<n>` native coordinate) is a
/// property of the adapter, not of the format, and `office.rs`'s own
/// `coordinates_hold_their_contract_for_every_routed_format` holds every
/// other routed format to it.
///
/// An earlier version of this test asserted `byte_end > byte_start` for "a
/// document coordinate" of *both* files and passed only because the one hit
/// it happened to pick per path was the whole-document one. That prose
/// claimed a guarantee the adapter does not make; this one states the
/// guarantee the adapter actually makes, and checks both halves of it.
#[test]
fn one_local_knowledge_query_spans_a_normalized_docx_and_a_markdown_file() {
    let estate = estate();
    let answer = estate.search(SPANNING_TERM, &only_library(), None);

    let paths: BTreeSet<&str> = answer
        .hits
        .iter()
        .map(|hit| hit.coordinate.relative_path())
        .collect();
    assert!(
        paths.contains("report.docx"),
        "the normalized Office document must be in the answer: {paths:?}"
    );
    assert!(
        paths.contains("notes.md"),
        "and the Markdown one, from the SAME query: {paths:?}"
    );

    // "without losing original path/source coordinates": every hit from
    // either file still names the original resource — the `.docx` path, not
    // a normalized intermediate — and cites the one source it came from.
    let hits_for = |path: &str| {
        answer
            .hits
            .iter()
            .filter(|hit| hit.coordinate.relative_path() == path)
            .collect::<Vec<_>>()
    };
    for path in ["report.docx", "notes.md"] {
        for hit in hits_for(path) {
            assert_eq!(hit.source_name, "library");
            assert_eq!(hit.source_kind, SourceKind::LocalKnowledge);
        }
    }

    // The Markdown half: the span IS the address, for every document unit.
    let mut markdown_documents = 0;
    for hit in hits_for("notes.md") {
        let UnitCoordinate::Document {
            byte_start,
            byte_end,
            native,
            ..
        } = &hit.coordinate
        else {
            // The same file's tree-sitter heading is a `Code` coordinate;
            // the document-family claim is what item 6 is about.
            continue;
        };
        assert!(
            native.is_none(),
            "a Markdown unit needs no native coordinate — its span is its \
             address: {:?}",
            hit.coordinate
        );
        assert!(
            byte_end > byte_start,
            "and that span is a real one: {:?}",
            hit.coordinate
        );
        markdown_documents += 1;
    }
    assert!(
        markdown_documents > 0,
        "the Markdown half of the span must not be vacuous"
    );

    // The Office half: one byte-recoverable whole-document unit, and
    // sections that cite `block:<n>` rather than inventing a span.
    let office = hits_for("report.docx");
    let (mut whole_document, mut sections) = (0, 0);
    for hit in &office {
        let UnitCoordinate::Document {
            byte_start,
            byte_end,
            native,
            ..
        } = &hit.coordinate
        else {
            panic!(
                "an Office hit is a document-family unit: {:?}",
                hit.coordinate
            )
        };
        match native {
            None => {
                assert!(
                    byte_end > byte_start,
                    "the whole-document unit spans the whole `.docx`: {:?}",
                    hit.coordinate
                );
                whole_document += 1;
            }
            Some(native) => {
                assert_eq!(
                    (*byte_start, *byte_end),
                    (0, 0),
                    "an Office section is not byte-recoverable and must not \
                     claim a span: {:?}",
                    hit.coordinate
                );
                assert!(
                    !native.is_empty(),
                    "so its native coordinate is the only address it has: {:?}",
                    hit.coordinate
                );
                sections += 1;
            }
        }
    }
    assert_eq!(
        whole_document, 1,
        "exactly one `.docx` unit carries a real span — the whole-document \
         one: {office:?}"
    );
    assert!(
        sections >= 1,
        "and the honest-`0`/`0` half is not vacuous either: {office:?}"
    );
}

// ============================================================ §13 the trace

/// **A2 §13**: *"Record at minimum"* — nine lines. This asserts all nine are
/// present in the rendered trace, **by the contract's own field list**, and
/// that each carries a real value rather than an empty placeholder.
#[test]
fn the_trace_records_every_one_of_a2_section_13s_nine_fields() {
    let estate = estate();
    let (answer, trace) = estate.traced("Heading", &only_library(), SemanticRequest::Requested);
    assert!(!answer.hits.is_empty(), "the query must find something");
    let json = trace.json();
    let object = json.as_object().expect("a trace is an object");

    for field in SearchTrace::FIELDS {
        assert!(
            object.contains_key(field),
            "A2 §13 field `{field}` is missing from the trace: {:?}",
            object.keys().collect::<Vec<_>>()
        );
    }

    // 1. query text/hash
    assert_eq!(json["query"]["text"], "Heading");
    assert_eq!(
        json["query"]["hash"],
        blake3::hash(b"Heading").to_hex().to_string(),
        "the recorded hash is of the recorded text"
    );
    // 2. execution/work attribution when managed — stated, not absent.
    assert_eq!(json["attribution"]["managed"], false);
    // 3. source-generation filter
    assert_eq!(json["source_generation_filter"]["selector"], "named");
    assert_eq!(json["source_generation_filter"]["source"], "library");
    assert_eq!(
        json["source_generation_filter"]["work_scope"],
        "not_work_scoped"
    );
    // 4. content/authority filters
    assert!(json["content_authority_filter"]["content"].is_null());
    assert!(json["content_authority_filter"]["kind"].is_null());
    // 5. retrieval generation — the exact worlds the answer was computed in.
    let generations = json["retrieval_generation"]["generations"]
        .as_array()
        .expect("a generation list");
    assert_eq!(
        generations.len(),
        1,
        "one named source admits one generation: {generations:?}"
    );
    assert_eq!(generations[0]["source"], "library");
    assert!(
        generations[0]["generation_id"]
            .as_str()
            .is_some_and(|id| !id.is_empty()),
        "the generation is identified, not merely counted"
    );
    // 6. lexical tokenizer/version
    assert_eq!(json["lexical"]["version"], LEXICAL_TOKENIZER_VERSION);
    assert!(
        json["lexical"]["tokenizer"]
            .as_str()
            .is_some_and(|t| t.contains("tokenize")),
        "the tokenizer is named by something a reader can go read"
    );
    // 7. semantic model identity/hash IF USED — plus H4's required status.
    assert_eq!(json["semantic"], answer.semantic.as_str());
    match answer.semantic {
        SemanticStatus::Applied => assert!(!json["semantic_model"].is_null()),
        _ => assert!(
            json["semantic_model"].is_null(),
            "§13 says 'if used'; an unused model must not be claimed"
        ),
    }
    // 8. RRF/rerank policy version
    assert_eq!(json["policy"]["version"], RETRIEVAL_POLICY_VERSION);
    assert_eq!(json["policy"]["rrf_k"], RRF_K);
    // 9. result evidence IDs + ranks
    let results = json["results"].as_array().expect("a result list");
    assert_eq!(
        results.len(),
        answer.hits.len(),
        "every answered hit has a trace row"
    );
    for (index, row) in results.iter().enumerate() {
        assert_eq!(row["rank"], index + 1);
        assert_eq!(row["unit_key"], answer.hits[index].unit_key.as_str());
        assert_eq!(
            row["generation_id"],
            answer.hits[index].generation_id.as_str()
        );
        assert_eq!(
            row["signals"].as_array().map(Vec::len),
            Some(9),
            "A2 §8's nine signals — including the three that cannot reorder \
             anything today, which is exactly what the trace carries them for"
        );
    }
}

/// **Decision H4, on the trace.** A2 §15 requires a degraded answer to report
/// itself honestly, and H4 makes that a *required, non-omittable* value. So a
/// suppressed semantic half is reported as `disabled` — a different fact from
/// `not_installed`, and not reported as one.
#[test]
fn the_trace_states_the_semantic_status_even_when_the_caller_suppressed_it() {
    let estate = estate();
    let (_, trace) = estate.traced("Heading", &only_library(), SemanticRequest::Suppressed);
    assert_eq!(trace.semantic, SemanticStatus::Disabled);
    assert_eq!(trace.json()["semantic"], "disabled");
    assert!(
        trace.json()["semantic_model"].is_null(),
        "a suppressed half used no model, so none may be claimed"
    );
}

/// **A2 §13 field 6 is a claim, and this is what makes it checkable.**
///
/// A version constant that is only a promise to remember is worth nothing: a
/// change to `tokenize` that nobody bumped the version for would produce two
/// incomparable corpora under one recorded identity. This pins the version to
/// a golden tokenization of A2 §5's own six literal forms — change what the
/// tokenizer emits without bumping [`LEXICAL_TOKENIZER_VERSION`] and this
/// goes red.
#[test]
fn the_lexical_tokenizer_version_is_pinned_to_the_tokenizers_actual_output() {
    // A2 §5's six forms, in the contract's order, tokenized by version `1`.
    let golden: Vec<(&str, Vec<&str>)> = vec![
        (
            "PaymentRetryPolicy",
            vec!["paymentretrypolicy", "payment", "retry", "policy"],
        ),
        (
            "payment_retry_policy",
            vec!["payment_retry_policy", "payment", "retry", "policy"],
        ),
        (
            "payment-retry-policy",
            vec!["payment-retry-policy", "payment", "retry", "policy"],
        ),
        ("Foo::bar", vec!["foo::bar", "foo", "bar"]),
        ("POST /payments", vec!["post", "post /payments", "payments"]),
        ("INC0012345", vec!["inc0012345", "inc", "0012345"]),
    ];
    assert_eq!(
        LEXICAL_TOKENIZER_VERSION, "1",
        "this golden set describes tokenizer version 1; a new version needs its own"
    );
    for (input, expected) in golden {
        let actual: Vec<String> = tokenize(input);
        assert_eq!(
            actual,
            expected.iter().map(|t| t.to_string()).collect::<Vec<_>>(),
            "tokenizer version {LEXICAL_TOKENIZER_VERSION} changed its output for {input:?} \
             without bumping the version A2 §13 records"
        );
    }
}

/// **A2 §13 field 8, pinned the same way.** The recorded policy version names
/// RRF at `k = 60`, the α blend the semble-parity port introduced, and A2
/// §8's nine signals. Change any of them without bumping the string and this
/// goes red — which is what it did when the α blend landed, because a trace
/// that still said `/1` would have been a false provenance record, not a
/// saved test.
#[test]
fn the_retrieval_policy_version_is_pinned_to_the_actual_rrf_and_rerank_policy() {
    assert_eq!(
        RETRIEVAL_POLICY_VERSION,
        "rrf-k60+a2s8-score-adjust+semble-boosts/5"
    );
    assert_eq!(
        RRF_K, 60.0,
        "the recorded version says k=60; the constant must agree"
    );
    assert_eq!(
        (
            BOOST_EXACT_MATCH,
            PENALTY_NON_CANONICAL,
            FILE_SATURATION_DECAY
        ),
        (3.0, 0.3, 0.5),
        "the recorded version says score-adjust; the multipliers must agree"
    );
    assert_eq!(
        (
            FILE_COHERENCE_BOOST_FRAC,
            STEM_BOOST_MULTIPLIER,
            STEM_MATCH_MIN_RATIO
        ),
        (0.2, 1.0, 0.10),
        "the recorded version says semble-boosts; those constants must agree"
    );
    // Nine signals, and the array the rerank key is built from is nine long.
    let all = RerankSignals {
        exact_match: true,
        definition_over_reference: true,
        caller_selected_source: true,
        work_changed_unit: true,
        same_section_as_anchor: true,
        structural_relationship: true,
        canonical_path: true,
        knowledge_source_requested: true,
        current_generation: true,
    };
    assert_eq!(
        all.priority().len(),
        9,
        "the recorded version says nine signals"
    );
    assert_eq!(all.fired(), 9);
}

// ============================================================ §14 `related`

/// **A2 §14's round trip.** `sgt related <coordinate>` takes the coordinate
/// `sgt search` prints; if the two spellings drifted, the second verb would
/// be unusable from the first's output and nobody would notice until a human
/// tried it.
#[test]
fn a_printed_coordinate_parses_back_to_itself() {
    let estate = estate();
    let answer = estate.search(SPANNING_TERM, &any(), None);
    assert!(!answer.hits.is_empty());
    for hit in &answer.hits {
        let printed = UnitAddress::render(&hit.source_name, &hit.unit_key);
        let parsed = UnitAddress::parse(&printed)
            .unwrap_or_else(|| panic!("a printed coordinate must parse: {printed}"));
        assert_eq!(parsed.source_name, hit.source_name);
        assert_eq!(parsed.unit_key, hit.unit_key);
    }
    // And an overlay-shaped source name, whose own `/` is exactly what a
    // naive first-`/` split would break on.
    let overlay = UnitAddress::parse("work:01ABC/repo-a/code:src/lib.rs#3")
        .expect("an overlay coordinate must parse");
    assert_eq!(overlay.source_name, "work:01ABC/repo-a");
    assert_eq!(overlay.unit_key, "code:src/lib.rs#3");
    // "Never approximate" applies to addressing too.
    assert!(UnitAddress::parse("library/not-a-unit-key").is_none());
    assert!(UnitAddress::parse("no-separator").is_none());
}

/// **A2 §14's second verb, doing its job.** Real neighbours for a real
/// coordinate — and never the anchor itself, which is trivially its own best
/// match under both halves and is not a neighbour.
#[test]
fn related_returns_real_neighbours_and_never_its_own_anchor() {
    let estate = estate();
    let answer = estate.search(SPANNING_TERM, &only_library(), None);
    let anchor = answer.hits.first().expect("an anchor hit").clone();

    let filter = only_library();
    let related = estate
        .db
        .related(&RelatedRequest {
            source_name: &anchor.source_name,
            unit_key: &anchor.unit_key,
            filter: &filter,
            family: None,
            limit: 10,
            semantic: SemanticRequest::Requested,
            attribution: Attribution::Unmanaged,
        })
        .expect("related")
        .expect("the anchor resolves");

    assert_eq!(related.anchor.unit_key, anchor.unit_key);
    assert_eq!(related.anchor.source_name, anchor.source_name);
    assert_eq!(related.anchor.source_kind, anchor.source_kind);
    assert!(
        !related.answer.hits.is_empty(),
        "an anchor with siblings in its own source must have neighbours"
    );
    assert!(
        related
            .answer
            .hits
            .iter()
            .all(|hit| hit.unit_key != anchor.unit_key
                || hit.generation_id != anchor.generation_id),
        "the anchor is never its own neighbour: {:?}",
        related
            .answer
            .hits
            .iter()
            .map(|h| &h.unit_key)
            .collect::<Vec<_>>()
    );
    // The trace describes what was actually retrieved on — the anchor's own
    // text — not the coordinate string the caller typed.
    assert!(
        !related.trace.query.text.is_empty(),
        "the trace records the text the neighbours were ranked against"
    );
    assert_eq!(
        related.trace.results.len(),
        related.answer.hits.len(),
        "the trace's result rows are the answer's, after the anchor was removed"
    );
}

/// **A2 §8's prohibition, applied to the second verb.** *"The reranker must
/// never silently cross an authority/source filter merely because a candidate
/// scores well."* An anchor lookup is exactly the shape that could have
/// become a hole in it: address a unit in a source the filter excludes, and
/// get its neighbours anyway.
///
/// It cannot. The anchor is resolved through `admissible_generations`, so a
/// coordinate outside the filter resolves to *nothing* — and the same
/// coordinate resolves fine once the filter admits it, which is what makes
/// this negative non-vacuous.
#[test]
fn related_refuses_an_anchor_the_admissibility_filter_excludes() {
    let estate = estate();
    let external = estate.search(SPANNING_TERM, &any(), None);
    let hit = external
        .hits
        .iter()
        .find(|hit| hit.source_name == "vendor-lib")
        .expect("an external hit to anchor on");

    let open = any();
    let reachable = estate
        .db
        .related(&RelatedRequest {
            source_name: &hit.source_name,
            unit_key: &hit.unit_key,
            filter: &open,
            family: None,
            limit: 5,
            semantic: SemanticRequest::Requested,
            attribution: Attribution::Unmanaged,
        })
        .expect("related, filter open");
    assert!(
        reachable.is_some(),
        "with the filter open the anchor must resolve, or the negative below is vacuous"
    );

    let closed = only_library();
    let refused = estate
        .db
        .related(&RelatedRequest {
            source_name: &hit.source_name,
            unit_key: &hit.unit_key,
            filter: &closed,
            family: None,
            limit: 5,
            semantic: SemanticRequest::Requested,
            attribution: Attribution::Unmanaged,
        })
        .expect("related, filter closed");
    assert!(
        refused.is_none(),
        "a coordinate the filter excludes must resolve to nothing, not to its neighbours"
    );
}

/// **S6 pidless-and-related, seam 2 — the CLI path stays working.**
/// `related_query`'s HTTP handler (`src/api.rs`) parses the coordinate
/// `sgt search` prints via `UnitAddress::parse`, which recovers the
/// path-based key (`UnitCoordinate::path_key`), never `db::unit_key`'s
/// digest-or-canonical-path dedup identity. `5fc63f56` fixed exactly this
/// path; the fix for the two tests above (matching on `db::unit_key` too,
/// for direct Rust-API callers like [`RelatedRequest`]'s other two tests
/// here) must not regress it — pinned directly so a future change to
/// `related`'s anchor match sees this break, not just a CLI smoke test.
#[test]
fn related_still_resolves_the_printed_path_coordinate_the_cli_parses() {
    let estate = estate();
    let answer = estate.search(SPANNING_TERM, &only_library(), None);
    let anchor = answer.hits.first().expect("an anchor hit").clone();
    let path_key = anchor.coordinate.path_key();

    let filter = only_library();
    let related = estate
        .db
        .related(&RelatedRequest {
            source_name: &anchor.source_name,
            unit_key: &path_key,
            filter: &filter,
            family: None,
            limit: 10,
            semantic: SemanticRequest::Requested,
            attribution: Attribution::Unmanaged,
        })
        .expect("related")
        .expect("the path-keyed coordinate the CLI round-trips through must still resolve");

    assert_eq!(related.anchor.unit_key, anchor.unit_key);
    assert_eq!(related.anchor.source_name, anchor.source_name);
}

// ============================================================ the surface

/// **A2 §14's prohibition, structurally**: *"Do not expose raw retrieval
/// weight tuning in workflow files. Workflows declare semantic context
/// profiles in C1."*
///
/// This pins the exact flag set `sgt search`/`sgt related` accept against A2
/// §14's own selector list. Adding a weight knob — `--k1`, `--boost`,
/// `--rrf-k`, a per-signal weight — goes red here, and so does adding any
/// other flag without stating it.
///
/// A source scan rather than a `--help` parse, because it is the *declaration*
/// that must stay narrow: a flag with a `hide` attribute would not appear in
/// help at all.
#[test]
fn the_search_cli_exposes_a2_section_14s_selectors_and_no_weight_knob() {
    let cli = std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/cli.rs"))
        .expect("read src/cli.rs");
    let start = cli
        .find("pub struct SearchSelectors {")
        .expect("SearchSelectors must exist");
    let body = &cli[start..];
    let end = body.find("\n}\n").expect("SearchSelectors must end");
    let body = &body[..end];

    let declared: BTreeSet<&str> = body
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            // Field declarations only — the `pub struct` header line is not
            // one, and a helper method's `pub fn` would not be either.
            let line = line.strip_prefix("pub ")?.strip_suffix(',')?;
            let name = line.split(':').next()?;
            Some(name.trim())
        })
        .collect();
    let expected: BTreeSet<&str> = [
        "source",      // §14 --source <name>
        "work",        // §14 --work <id>
        "repo",        // resolves which repository a multi-repo --work means
        "content",     // §14 --content
        "source_type", // §14 --type
        "external",    // §14 --external
        "top",         // §14 --top <n>
        "no_semantic", // decision H4's request side
    ]
    .into_iter()
    .collect();
    assert_eq!(
        declared, expected,
        "the selector set is A2 §14's list; a new flag must be argued for, and a \
         retrieval weight may never be one"
    );
}

/// **H13.2: `sgt search` is a pure reader.**
///
/// H13.2 rejected query-time scanning specifically to keep it one, and
/// `tests/w1b_overlay_lifecycle_trigger.rs::
/// the_admissibility_filter_cannot_write_and_neither_can_anything_it_calls`
/// pins the store side. This pins the *route* side: both handlers reach Atlas
/// through the read-only `with_atlas`, never `with_atlas_write` or
/// `with_existing_atlas_write`, so no index-on-demand, scan-if-stale or
/// cache-warming side effect can be introduced without this going red.
#[test]
fn the_search_and_related_routes_reach_atlas_through_the_read_only_handle() {
    let api = std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/api.rs"))
        .expect("read src/api.rs");
    for handler in ["async fn search_query", "async fn related_query"] {
        let start = api
            .find(handler)
            .unwrap_or_else(|| panic!("{handler} must exist"));
        let body = &api[start..];
        let end = body.find("\n}\n").expect("the handler must end");
        let body = &body[..end];
        assert!(
            body.contains("with_atlas(&state"),
            "{handler} must read through the read-only handle"
        );
        assert!(
            !body.contains("with_atlas_write") && !body.contains("with_existing_atlas_write"),
            "{handler} must not reach a write handle: `sgt search` is a pure reader (H13.2)"
        );
        assert!(
            !body.contains("reindex_lexical") && !body.contains("record_scan"),
            "{handler} must not index or scan on demand (H13.2)"
        );
    }
}

/// **H13.1's `--content config` gap, carried forward honestly.**
///
/// A2 §14 lists `config` among `--content`'s values. Atlas stores no value
/// that separates a `.toml` read through the `text/v1` fallback from an
/// ordinary text document, so a `config` lane would return *some* config
/// files as though it had returned all of them. The rule is: *either the
/// value resolves to a lane and SAYS so, or it is not offered — never a value
/// that returns partial results as though complete.*
///
/// So the value is refused **by name, with its reason** — not silently
/// answered, and not reported as an unknown value a reader would take for a
/// typo.
#[test]
fn content_config_is_refused_by_name_rather_than_answered_partially() {
    let gap = sergeant_rs::api::CONTENT_CONFIG_GAP;
    assert!(
        gap.contains("config"),
        "the refusal names the value that was refused"
    );
    assert!(
        gap.contains("text/v1"),
        "and the actual reason — the fallback extractor Atlas cannot distinguish"
    );
    // `config` is not a retrieval family, which is the fact the refusal is
    // about: there is no lane for it to resolve to.
    assert!(LexicalFamily::parse("config").is_none());
    assert_eq!(
        LexicalFamily::ALL.len(),
        4,
        "A2 §17 item 2's four families, and no fifth invented to satisfy a flag"
    );

    // The static properties above hold regardless of whether `--content
    // config` is actually refused — this drives the real decision
    // (`sergeant_rs::api::content_family`, what `SearchSelectors::family`
    // resolves to) so a deleted `Some("config") => Err(...)` arm falling
    // through to the generic `unknown_content` branch would be caught here,
    // not waved through.
    let Err(response) = sergeant_rs::api::content_family(Some("config")) else {
        panic!("--content config must be refused, not resolved to a family");
    };
    let (status, code, message) = response_body_json(*response);
    assert_eq!(
        status,
        axum::http::StatusCode::BAD_REQUEST,
        "the refusal is a 400, same as any other bad selector"
    );
    assert_eq!(
        code, "content_config_unavailable",
        "the specific code this arm returns — `unknown_content` is a different, generic gap"
    );
    assert_eq!(
        message, gap,
        "the wire body carries CONTENT_CONFIG_GAP verbatim, not a paraphrase"
    );

    // A neighbouring value that IS a real lane still resolves normally, so
    // the refusal is `config`-specific rather than a blanket content-family
    // failure.
    assert_eq!(
        sergeant_rs::api::content_family(Some("code")).expect("`code` is a real lane"),
        Some(LexicalFamily::Code)
    );
}

/// Synchronously decode an axum `Response`'s status and its JSON error
/// body's `code`/`message` fields — the same buffered-`Json`-body technique
/// `tests/i9_floor_pinning.rs` uses for `below_floor_refusal`, reused here
/// for `content_family`'s refusal.
fn response_body_json(
    response: axum::response::Response,
) -> (axum::http::StatusCode, String, String) {
    use http_body_util::BodyExt;
    let status = response.status();
    let mut collect = std::pin::pin!(response.into_body().collect());
    let waker = std::task::Waker::noop();
    let mut cx = std::task::Context::from_waker(waker);
    let collected = match collect.as_mut().poll(&mut cx) {
        std::task::Poll::Ready(result) => {
            result.expect("content_family's body must not itself error")
        }
        std::task::Poll::Pending => panic!(
            "content_family's response body was not immediately ready — it must be a fully \
             buffered Json body, never a stream"
        ),
    };
    let bytes = collected.to_bytes();
    let json: serde_json::Value =
        serde_json::from_slice(&bytes).expect("content_family's body must be JSON");
    (
        status,
        json["error"]["code"]
            .as_str()
            .expect("body has an `error.code` field")
            .to_string(),
        json["error"]["message"]
            .as_str()
            .expect("body has an `error.message` field")
            .to_string(),
    )
}
