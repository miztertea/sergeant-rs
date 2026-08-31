//! S5 W2 — A2 §5's lexical retrieval (BM25), and A2 §17 item 2.
//!
//! A2 §5, verbatim: *"Start with a small local BM25 implementation tuned for
//! identifier/document tokens rather than adding a full search server."*
//! A2 §17 item 2, verbatim: *"lexical search returns code/document/mail/
//! selected-row-text units with exact A1 provenance."* All four families are
//! enumerated below, each with a test that finds a unit of that kind and
//! resolves its provenance — a family present in the machinery and absent
//! from the tests is the defect this program has recorded three times.
//!
//! **The most important test in this file is a negative.** A2 §8: *"The
//! reranker must never silently cross an authority/source filter merely
//! because a candidate scores well."* So
//! [`an_inadmissible_unit_with_a_perfect_lexical_match_is_never_returned`]
//! plants a unit that is a *better* lexical match than anything admissible
//! and proves it does not come back — after first proving it is reachable at
//! all, because a decoy that was never in the index proves nothing.
//!
//! # What is real here and what is a fixture
//!
//! The code, document and selected-row-text families come from a **real**
//! `scan_local_knowledge` walk over a real directory, recorded through the
//! real three-step `record_scan` path — so their byte ranges are the real
//! offsets into real files, and the tests slice the files with them.
//!
//! The mail family comes from a **real** `.eml` fixture walked through the
//! real supervised worker subprocess (`scan_local_knowledge_with_worker`, the
//! shape `scan.rs`'s `worker_extractor_for` routes `.eml` into) and recorded
//! through the same `record_scan`. It was hand-built until the S5 closeout
//! (F-AC-02) and that was the defect: the hand-built row set `title` and a
//! real byte span by hand, which the worker landing path cannot produce, so
//! the test pinned values production never emits and would have passed with
//! the whole mail adapter deleted. A family's provenance test has to land the
//! family the way production lands it.

/// **S6 D1 — A2 §2 stage 1's estate coordinate.** This suite is
/// single-estate: every generation it records is bound to this one root and
/// every filter it builds is admitted from it. The cross-estate case — two
/// estates on one host daemon, which is where the axis actually earns its
/// keep — is `tests/d1_estate_isolation.rs`, deliberately not folded in
/// here, because a suite that never crosses estates cannot notice an estate
/// filter that does nothing (that is exactly how the leak survived: this
/// file's ancestors all passed).
#[allow(dead_code)]
const D1_ESTATE: &str = "/estates/w2_lexical_retrieval";

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tempfile::TempDir;

use sergeant_rs::domain::event::rfc3339_utc_now;
use sergeant_rs::domain::source::{AuthorityClass, SourceKind, UnitKind};
use sergeant_rs::runtime::atlas::db::{
    Admissibility, AtlasDb, LexicalAnswer, LexicalQuery, MAX_ROWS, SourceSelector,
};
use sergeant_rs::runtime::atlas::lexical::{LexicalFamily, LexicalHit, UnitCoordinate};
use sergeant_rs::runtime::atlas::mail::MAIL_EXTRACTOR;
use sergeant_rs::runtime::atlas::record::{ScanRecord, record_scan, scan_and_record};
use sergeant_rs::runtime::atlas::scan::{
    KnowledgeSource, ScannedFile, ScannedSymbol, ScannedSyntax, ScannedUnit, SourceScan,
    scan_local_knowledge_with_worker,
};
use sergeant_rs::runtime::atlas::semantic::SemanticRequest;
use sergeant_rs::runtime::atlas::tabular::ContextFields;
use sergeant_rs::runtime::atlas::text::MARKDOWN_EXTRACTOR;
use sergeant_rs::runtime::atlas::worker::WorkerRuntime;
use sergeant_rs::runtime::journal::Journal;

// ---------------------------------------------------------------- fixtures

/// A2 §5's six literal token forms, in the contract's own order.
const FORMS: [&str; 6] = [
    "PaymentRetryPolicy",
    "payment_retry_policy",
    "payment-retry-policy",
    "Foo::bar",
    "POST /payments",
    "INC0012345",
];

/// The Markdown document every one of the six forms is written into.
const FORMS_MD: &str = "\
# Retry forms

The PaymentRetryPolicy is spelled payment_retry_policy in Rust and
payment-retry-policy in configuration. The helper is Foo::bar. The
endpoint is POST /payments. The originating ticket is INC0012345.
";

/// A Rust file: one grammar-claimed symbol, and prose that only its
/// plain-text fallback unit can carry.
const LIB_RS: &str = "\
// Ordinary narrative prose about reconciliation of settlement batches.
pub fn payment_retry_policy() {}
";

/// A CSV whose allowlisted columns become `context.row_units`.
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

/// A whole-resource [`ScannedUnit`], the shape every document extractor's
/// first unit takes.
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

fn scanned_file(relative_path: &str, extractor: &str, units: Vec<ScannedUnit>) -> ScannedFile {
    ScannedFile {
        relative_path: relative_path.to_string(),
        content_hash: format!("hash/{relative_path}"),
        extractor: extractor.to_string(),
        local_key: format!("key/{relative_path}"),
        byte_len: 64,
        mtime_millis: None,
        units,
        syntax: None,
        // Not a container child: this helper hand-builds a top-level resource,
        // which is exactly the case `ScannedFile::parent`'s own doc calls
        // `None` — "every resource acquired directly from a source root".
        parent: None,
    }
}

fn hand_built_scan(
    source_name: &str,
    kind: SourceKind,
    authority: AuthorityClass,
    content_key: &str,
    files: Vec<ScannedFile>,
) -> SourceScan {
    let mut extractors = BTreeSet::new();
    for f in &files {
        extractors.insert(f.extractor.clone());
        if let Some(s) = &f.syntax {
            extractors.insert(s.extractor.clone());
        }
    }
    SourceScan {
        source_name: source_name.to_string(),
        kind,
        authority,
        content_key: content_key.to_string(),
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

/// A one-symbol Rust syntax extraction, for the overlay fixtures.
fn rust_symbol(name: &str) -> ScannedSyntax {
    ScannedSyntax {
        language: "rust",
        extractor: "syntax-rust/v1".to_string(),
        syntax_key: format!("syntax-key/rust/{name}"),
        symbols: vec![ScannedSymbol {
            ordinal: 0,
            label: "function",
            name: name.to_string(),
            byte_start: 0,
            byte_end: name.len() as u64,
        }],
        edges: Vec::new(),
    }
}

/// The estate every test below queries.
///
/// * `knowledge` (local-knowledge, estate-readonly) — a REAL walk over a real
///   directory: `docs/forms.md`, `src/lib.rs`, `tickets.csv` with
///   `number`/`short_description` allowlisted (F10a).
/// * `mailbox` (local-knowledge, estate-readonly) — a REAL walk over a
///   directory holding this repo's own `02-multipart-alternative.eml`
///   fixture, dispatched through the real supervised worker subprocess.
/// * `vendor-lib` (external-git, external) — the decoy: a document whose body
///   is a *better* lexical match for the queries below than anything
///   admissible, planted so the negative-admission test has something real to
///   fail on.
struct Estate {
    _data: TempDir,
    _mailbox: TempDir,
    root: TempDir,
    journal: Journal,
    db: AtlasDb,
}

impl Estate {
    /// Bytes of one file in the real knowledge root — what a byte range from
    /// a hit is sliced out of.
    fn bytes(&self, relative: &str) -> Vec<u8> {
        std::fs::read(self.root.path().join(relative)).expect("read fixture bytes")
    }

    fn search(
        &self,
        text: &str,
        filter: &Admissibility,
        family: Option<LexicalFamily>,
    ) -> LexicalAnswer {
        self.db
            .lexical_search(&LexicalQuery {
                text,
                filter,
                family,
                limit: 50,
                semantic: SemanticRequest::Requested,
            })
            .expect("lexical search")
    }
}

/// The decoy body: every one of the six forms, repeated, so BM25 cannot
/// prefer anything else on any of the queries this suite runs.
fn decoy_body() -> String {
    let once = FORMS.join(" ");
    format!("{once} {once} {once} {once}")
}

fn estate() -> Estate {
    let data = tempfile::tempdir().expect("data dir");
    let root = tempfile::tempdir().expect("knowledge root");
    write(root.path(), "docs/forms.md", FORMS_MD);
    write(root.path(), "src/lib.rs", LIB_RS);
    write(root.path(), "tickets.csv", TICKETS_CSV);

    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");

    let knowledge = KnowledgeSource {
        name: "knowledge".to_string(),
        root: root.path().to_path_buf(),
        ignore: Vec::new(),
        context_fields: ContextFields::declared(&[
            "number".to_string(),
            "short_description".to_string(),
        ]),
    };
    scan_and_record(
        &mut db,
        &mut journal,
        &knowledge,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record knowledge");

    // The mail source is REAL: this repo's own `.eml` fixture, walked
    // through the real supervised worker subprocess the production `.eml`
    // route uses. See the module doc for why it is not hand-built.
    let mailbox_root = tempfile::tempdir().expect("mailbox root");
    std::fs::create_dir_all(mailbox_root.path().join("inbox")).expect("inbox");
    std::fs::write(
        mailbox_root.path().join("inbox/message.eml"),
        std::fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/mail_corpus/02-multipart-alternative.eml"),
        )
        .expect("read the mail fixture"),
    )
    .expect("write the mail fixture");
    let mailbox = scan_local_knowledge_with_worker(
        &KnowledgeSource {
            name: "mailbox".to_string(),
            root: mailbox_root.path().to_path_buf(),
            ignore: Vec::new(),
            context_fields: ContextFields::none(),
        },
        &WorkerRuntime {
            program: PathBuf::from(env!("CARGO_BIN_EXE_sgt-atlas-worker")),
            deadline: Duration::from_secs(20),
        },
    )
    .expect("scan the mailbox through the real worker");
    assert!(
        mailbox.extractors.contains(MAIL_EXTRACTOR),
        "the mail fixture must have routed through the real mail adapter: {:?}",
        mailbox.extractors
    );
    record_scan(
        &mut db,
        &mut journal,
        &mailbox,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record mailbox");

    let decoy = decoy_body();
    let vendor = hand_built_scan(
        "vendor-lib",
        SourceKind::ExternalGit,
        AuthorityClass::External,
        "vendor-lib@key-1",
        vec![scanned_file(
            "docs/leak.md",
            MARKDOWN_EXTRACTOR,
            vec![document_unit(None, &decoy)],
        )],
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
        _mailbox: mailbox_root,
        root,
        journal,
        db,
    }
}

/// Only the `knowledge` source: the world the six-form tests run in.
fn only_knowledge() -> Admissibility {
    Admissibility {
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: SourceSelector::Named("knowledge".to_string()),
        kind: None,
        authority: None,
    }
}

fn paths(answer: &LexicalAnswer) -> Vec<String> {
    answer
        .hits
        .iter()
        .map(|hit| format!("{}:{}", hit.source_name, hit.coordinate.relative_path()))
        .collect()
}

fn finds(answer: &LexicalAnswer, relative_path: &str) -> bool {
    answer
        .hits
        .iter()
        .any(|hit| hit.coordinate.relative_path() == relative_path)
}

fn hit_at<'a>(answer: &'a LexicalAnswer, relative_path: &str) -> &'a LexicalHit {
    answer
        .hits
        .iter()
        .find(|hit| hit.coordinate.relative_path() == relative_path)
        .unwrap_or_else(|| panic!("no hit at {relative_path}; got {:?}", paths(answer)))
}

// ------------------------------------------- A2 §5's six literal forms

/// A2 §5 names six forms tokenization *"should preserve"*. Each is a query
/// here, and each must find the document that spells it — the contract naming
/// its own minimum, tested as six cases rather than one.
#[test]
fn each_of_the_six_contract_token_forms_finds_the_document_that_spells_it() {
    let estate = estate();
    let filter = only_knowledge();
    for form in FORMS {
        let answer = estate.search(form, &filter, None);
        assert!(
            finds(&answer, "docs/forms.md"),
            "A2 §5's form {form:?} must find the document that spells it; got {:?}",
            paths(&answer)
        );
    }
}

/// **The trap A2 §5 sets.** Camel-case splitting must not cost the whole
/// identifier: searching `PaymentRetryPolicy` and searching `payment` must
/// BOTH find the unit. A tokenizer that only splits passes the second half
/// and fails the first, and has destroyed the exact-identifier advantage that
/// is BM25's entire reason for being in this design.
#[test]
fn splitting_a_camel_identifier_never_costs_the_whole_identifier() {
    let estate = estate();
    let filter = only_knowledge();
    for query in ["PaymentRetryPolicy", "payment"] {
        let answer = estate.search(query, &filter, None);
        assert!(
            finds(&answer, "docs/forms.md"),
            "{query:?} must find the unit spelling PaymentRetryPolicy; got {:?}",
            paths(&answer)
        );
    }
}

/// A2 §5's second sentence: *"Document/mail retrieval additionally retains
/// ordinary natural-language tokens."* The code family's indexed text is the
/// symbol name a grammar claimed and nothing else, so a prose word from the
/// same file's comments is unreachable through it — and reachable through the
/// document family, which indexes the file's own body.
#[test]
fn code_units_index_identifiers_while_document_units_additionally_index_prose() {
    let estate = estate();
    let filter = only_knowledge();

    let code = estate.search("reconciliation", &filter, Some(LexicalFamily::Code));
    assert!(
        code.hits.is_empty(),
        "the code family indexes identifiers, not prose; got {:?}",
        paths(&code)
    );

    let document = estate.search("reconciliation", &filter, Some(LexicalFamily::Document));
    assert!(
        finds(&document, "src/lib.rs"),
        "the document family must retain the ordinary prose word; got {:?}",
        paths(&document)
    );

    let identifier = estate.search("payment_retry_policy", &filter, Some(LexicalFamily::Code));
    assert!(
        finds(&identifier, "src/lib.rs"),
        "the code family must still find the identifier; got {:?}",
        paths(&identifier)
    );
}

// ------------------------------------ A2 §17 item 2: all four families

/// Family 1 of 4 — **code**, with A2 §3's code coordinate
/// (`source/revision/path/symbol/span`) resolving to the exact bytes of the
/// real file the real walk read.
#[test]
fn lexical_search_returns_a_code_unit_with_exact_a1_provenance() {
    let estate = estate();
    let answer = estate.search(
        "payment_retry_policy",
        &only_knowledge(),
        Some(LexicalFamily::Code),
    );
    let hit = hit_at(&answer, "src/lib.rs");
    assert_eq!(hit.source_name, "knowledge");
    assert!(!hit.generation_id.is_empty(), "a hit names its generation");
    assert!(
        !hit.content_key.is_empty(),
        "a hit names the generation's content identity"
    );
    let UnitCoordinate::Code {
        symbol,
        language,
        byte_start,
        byte_end,
        ..
    } = &hit.coordinate
    else {
        panic!("expected a code coordinate, got {:?}", hit.coordinate);
    };
    assert_eq!(symbol, "payment_retry_policy");
    assert_eq!(language, "rust");
    let bytes = estate.bytes("src/lib.rs");
    let span = String::from_utf8(bytes[*byte_start as usize..*byte_end as usize].to_vec())
        .expect("the span is utf-8");
    assert!(
        span.contains("payment_retry_policy"),
        "the byte range must resolve to the bytes the hit claims: {span:?}"
    );
}

/// Family 2 of 4 — **document**, with A2 §3's document coordinate resolving
/// to the exact bytes of the real Markdown file.
#[test]
fn lexical_search_returns_a_document_unit_with_exact_a1_provenance() {
    let estate = estate();
    let answer = estate.search(
        "payment-retry-policy",
        &only_knowledge(),
        Some(LexicalFamily::Document),
    );
    let hit = hit_at(&answer, "docs/forms.md");
    assert_eq!(hit.source_name, "knowledge");
    let UnitCoordinate::Document {
        byte_start,
        byte_end,
        ..
    } = &hit.coordinate
    else {
        panic!("expected a document coordinate, got {:?}", hit.coordinate);
    };
    let bytes = estate.bytes("docs/forms.md");
    let span = String::from_utf8(bytes[*byte_start as usize..*byte_end as usize].to_vec())
        .expect("the span is utf-8");
    assert!(
        span.contains("payment-retry-policy"),
        "the byte range must resolve to the bytes the hit claims: {span:?}"
    );
}

/// Family 3 of 4 — **mail**, landed the way production lands it: this
/// repo's own `.eml` fixture through the real worker subprocess.
///
/// **What a mail hit actually carries, and why.** The message has two
/// bodies (A1 §6.5's `text/html body`), so it lands two `Document`-kind
/// units at one path. Neither is byte-recoverable into the original wire
/// bytes — a decoded Content-Transfer-Encoding is a transform — so both
/// carry `0`/`0`, the honest "not applicable" the worker's own comment
/// names, and the *only* thing that tells them apart is the normalizer's
/// native coordinate (`text-body`/`html-body`). The subject is the unit
/// title. Those three facts are what A2 §17 item 2's "exact A1 provenance"
/// and §9's "still cite the original source path/native coordinate" mean
/// for this family; asserting a byte span here would be asserting a value
/// production cannot produce, which is precisely what the pre-F-AC-02
/// version of this test did.
#[test]
fn lexical_search_returns_mail_units_with_exact_a1_provenance() {
    let estate = estate();
    let filter = Admissibility {
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: SourceSelector::Named("mailbox".to_string()),
        kind: None,
        authority: None,
    };
    let answer = estate.search("alternative", &filter, Some(LexicalFamily::Mail));

    let mut seen: Vec<String> = Vec::new();
    for hit in &answer.hits {
        assert_eq!(hit.source_name, "mailbox");
        let UnitCoordinate::Mail {
            relative_path,
            title,
            byte_start,
            byte_end,
            native,
            ordinal: _,
        } = &hit.coordinate
        else {
            panic!("expected a mail coordinate, got {:?}", hit.coordinate);
        };
        assert_eq!(relative_path, "inbox/message.eml");
        assert_eq!(
            title.as_deref(),
            Some("Alternative body demo"),
            "the message subject is the unit's title (A1 §6.5), and the worker landing path \
             must carry it: {:?}",
            hit.coordinate
        );
        assert_eq!(
            (*byte_start, *byte_end),
            (0, 0),
            "a mail body is not byte-recoverable into the original wire bytes; 0/0 is the \
             honest not-applicable, and no other value is truthful: {:?}",
            hit.coordinate
        );
        seen.push(native.clone().unwrap_or_else(|| {
            panic!(
                "a mail unit must carry its native coordinate — it is \
                     the only thing telling the two bodies apart: {:?}",
                hit.coordinate
            )
        }));
    }
    seen.sort();
    assert_eq!(
        seen,
        vec!["html-body".to_string(), "text-body".to_string()],
        "the fixture has a text body and an html body; both are indexed units, and the native \
         coordinate is what distinguishes them: {:?}",
        answer.hits
    );
}

/// Family 4 of 4 — **selected-row-text**, with A2 §3's structured-text
/// coordinate `source/generation/dataset/row-id/field-set`.
///
/// **This coordinate carries no byte range, and that is the contract, not an
/// omission.** A2 §3 gives structured text `source/generation/dataset/row-id/
/// field-set`; the row is read in place and never copied into Atlas, so there
/// are no Atlas-owned bytes to point at. The W2 brief's summary line ("every
/// hit cites ... byte range") is the common case; where it and A2 §3 disagree
/// the contract wins (J5).
#[test]
fn lexical_search_returns_a_selected_row_text_unit_with_exact_a1_provenance() {
    let estate = estate();
    let answer = estate.search(
        "INC0012345",
        &only_knowledge(),
        Some(LexicalFamily::RowText),
    );
    let hit = hit_at(&answer, "tickets.csv");
    assert_eq!(hit.source_name, "knowledge");
    let UnitCoordinate::RowText {
        dataset_key,
        row_key,
        fields,
        ..
    } = &hit.coordinate
    else {
        panic!("expected a row-text coordinate, got {:?}", hit.coordinate);
    };
    assert!(!dataset_key.is_empty(), "a row hit names its dataset");
    assert!(!row_key.is_empty(), "a row hit names its row");
    assert!(
        fields.contains(&"short_description".to_string()),
        "F10a's exposed field set rides with the hit: {fields:?}"
    );
    // The index is per ROW, not per file: the sibling ticket shares the `inc`
    // part-token, so it is a legitimate weaker match — and the row that
    // spells the whole identifier outranks it, which is the exact-identifier
    // advantage A2 §5 buys.
    assert_eq!(
        hit.coordinate.ordinal(),
        0,
        "the matching row is the first one"
    );
    let sibling = answer
        .hits
        .iter()
        .find(|other| other.coordinate.ordinal() != 0)
        .expect("the sibling row matches on the shared `inc` part-token");
    assert!(
        hit.score > sibling.score,
        "the row spelling the whole identifier must outrank the one sharing only a part:          {} vs {}",
        hit.score,
        sibling.score
    );
    let UnitCoordinate::RowText {
        row_key: sibling_key,
        ..
    } = &sibling.coordinate
    else {
        panic!(
            "expected a row-text coordinate, got {:?}",
            sibling.coordinate
        );
    };
    assert_ne!(
        row_key, sibling_key,
        "two rows of one dataset are two units with two row identities"
    );
}

// ------------------------------------------- A2 §8: the filter is never crossed

/// **The wave's most important test.** A2 §8: *"The reranker must never
/// silently cross an authority/source filter merely because a candidate
/// scores well."*
///
/// `vendor-lib`'s planted document is the best lexical match in the store for
/// every one of A2 §5's six forms — the first half of this test proves that,
/// because a decoy that never ranked first would make the second half
/// vacuous. The second half applies the authority filter and the decoy is
/// gone: not demoted, not present-but-last — absent, because BM25 ranks
/// inside the admissible set and cannot widen it.
#[test]
fn an_inadmissible_unit_with_a_perfect_lexical_match_is_never_returned() {
    let estate = estate();
    let unfiltered = Admissibility::within_estate(D1_ESTATE);
    let wide = estate.search("PaymentRetryPolicy", &unfiltered, None);
    assert_eq!(
        wide.hits.first().map(|hit| hit.source_name.as_str()),
        Some("vendor-lib"),
        "the decoy must actually be the best-scoring candidate, or this test proves nothing: \
         {:?}",
        paths(&wide)
    );

    for filter in [
        Admissibility {
            estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
            source: SourceSelector::Any,
            kind: None,
            authority: Some(AuthorityClass::EstateReadonly),
        },
        only_knowledge(),
        Admissibility {
            estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
            source: SourceSelector::Any,
            kind: Some(SourceKind::LocalKnowledge),
            authority: None,
        },
    ] {
        let answer = estate.search("PaymentRetryPolicy", &filter, None);
        assert!(
            !answer.hits.is_empty(),
            "the filter must still admit the legitimate matches: {filter:?}"
        );
        assert!(
            answer
                .hits
                .iter()
                .all(|hit| hit.source_name != "vendor-lib"),
            "a perfect lexical match outside the admissible set must never be returned \
             ({filter:?}); got {:?}",
            paths(&answer)
        );
    }
}

/// The same prohibition on the Work-overlay axis (S5 W1b): a lexical query
/// scoped to one Work sees that Work's own overlay and never another's, and a
/// plain `--source` query sees no overlay at all — even when the other Work's
/// overlay is the better lexical match.
#[test]
fn another_works_overlay_unit_never_surfaces_through_a_lexical_query() {
    let data = tempfile::tempdir().expect("data dir");
    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");

    const MINE: &str = "01MINE0000000000000000000A";
    const OTHER: &str = "01OTHER000000000000000000B";

    let base = hand_built_scan(
        "repo-a",
        SourceKind::EstateGit,
        AuthorityClass::EstateMutable,
        "repo-a@key-1",
        vec![ScannedFile {
            syntax: Some(rust_symbol("widget_base")),
            ..scanned_file(
                "src/main.rs",
                sergeant_rs::runtime::atlas::text::TEXT_EXTRACTOR,
                vec![document_unit(None, "fn widget_base() {}\n")],
            )
        }],
    );
    record_scan(
        &mut db,
        &mut journal,
        &base,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record base");

    for (work_id, symbol) in [(MINE, "widget_mine"), (OTHER, "widget_theirs")] {
        let overlay = hand_built_scan(
            &sergeant_rs::runtime::atlas::overlay::overlay_source_name(work_id, "repo-a"),
            SourceKind::EstateGit,
            AuthorityClass::EstateMutable,
            &format!("repo-a@base+{work_id}"),
            vec![ScannedFile {
                syntax: Some(rust_symbol(symbol)),
                ..scanned_file(
                    "src/main.rs",
                    sergeant_rs::runtime::atlas::text::TEXT_EXTRACTOR,
                    vec![document_unit(None, &format!("fn {symbol}() {{}}\n"))],
                )
            }],
        );
        let recorded = record_scan(
            &mut db,
            &mut journal,
            &overlay,
            None,
            &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
        )
        .expect("record overlay");
        assert!(
            matches!(recorded, ScanRecord::Recorded { .. }),
            "the overlay fixture must be journaled and confirmed: {recorded:?}"
        );
    }

    let mine = Admissibility {
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: SourceSelector::WorkBase {
            work_id: MINE.to_string(),
            repository: "repo-a".to_string(),
        },
        kind: None,
        authority: None,
    };
    let answer = db
        .lexical_search(&LexicalQuery {
            text: "widget_theirs widget_mine widget_base",
            filter: &mine,
            family: None,
            limit: 50,
            semantic: SemanticRequest::Requested,
        })
        .expect("lexical search");
    let sources: BTreeSet<String> = answer
        .hits
        .iter()
        .map(|hit| hit.source_name.clone())
        .collect();
    assert!(
        sources.contains(&sergeant_rs::runtime::atlas::overlay::overlay_source_name(
            MINE, "repo-a"
        )),
        "--work must see its own overlay: {sources:?}"
    );
    assert!(
        !sources.contains(&sergeant_rs::runtime::atlas::overlay::overlay_source_name(
            OTHER, "repo-a"
        )),
        "--work must never see another Work's overlay, however well it scores: {sources:?}"
    );

    let named = Admissibility {
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: SourceSelector::Named("repo-a".to_string()),
        kind: None,
        authority: None,
    };
    let plain = db
        .lexical_search(&LexicalQuery {
            text: "widget_theirs widget_mine widget_base",
            filter: &named,
            family: None,
            limit: 50,
            semantic: SemanticRequest::Requested,
        })
        .expect("lexical search");
    assert!(
        plain.hits.iter().all(|hit| hit.source_name == "repo-a"),
        "a plain --source query reaches no overlay at all: {:?}",
        paths(&plain)
    );

    // And retiring the Work takes its overlay's postings with it (H2).
    db.evict_work_overlays(MINE).expect("evict overlays");
    let after = db
        .lexical_search(&LexicalQuery {
            text: "widget_mine",
            filter: &mine,
            family: None,
            limit: 50,
            semantic: SemanticRequest::Requested,
        })
        .expect("lexical search");
    assert!(
        after
            .hits
            .iter()
            .all(|hit| !hit.source_name.starts_with("work:")),
        "a retired Work's overlay postings are evicted with it: {:?}",
        paths(&after)
    );
}

/// The cross-check that keeps W2's own composed predicate honest: every
/// generation a lexical hit cites is one [`AtlasDb::admissible_generations`]
/// admits under the same filter. Compared as answers, not as SQL strings —
/// two identically wrong queries would agree on the string.
#[test]
fn every_generation_a_lexical_hit_cites_is_one_the_admissibility_filter_admits() {
    let estate = estate();
    for filter in [
        Admissibility::within_estate(D1_ESTATE),
        only_knowledge(),
        Admissibility {
            estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
            source: SourceSelector::Any,
            kind: None,
            authority: Some(AuthorityClass::EstateReadonly),
        },
    ] {
        let admitted: BTreeSet<String> = estate
            .db
            .admissible_generations(&filter, 1_000)
            .expect("admissible generations")
            .hits
            .into_iter()
            .map(|generation| generation.id)
            .collect();
        let answer = estate.search("PaymentRetryPolicy INC0012345", &filter, None);
        assert!(!answer.hits.is_empty(), "no hits to check for {filter:?}");
        for hit in &answer.hits {
            assert!(
                admitted.contains(&hit.generation_id),
                "a hit cited generation {} which the filter {filter:?} does not admit",
                hit.generation_id
            );
        }
    }
}

/// An empty query returns nothing rather than everything — the degenerate
/// case where "rank the admissible set" could quietly become "dump it".
#[test]
fn a_query_with_no_terms_returns_no_hits_rather_than_the_whole_corpus() {
    let estate = estate();
    for text in ["", "   ", "!!! ---"] {
        let answer = estate.search(text, &only_knowledge(), None);
        assert!(
            answer.hits.is_empty(),
            "{text:?} must not return the corpus; got {:?}",
            paths(&answer)
        );
    }
}

// ---------------------------------------------------------- determinism

/// Same query + same generations ⇒ same ordered result, scores included.
#[test]
fn the_same_query_over_the_same_generations_returns_the_same_ordered_result() {
    let estate = estate();
    let filter = Admissibility::within_estate(D1_ESTATE);
    let first = estate.search("PaymentRetryPolicy INC0012345 payments", &filter, None);
    for _ in 0..5 {
        let again = estate.search("PaymentRetryPolicy INC0012345 payments", &filter, None);
        assert_eq!(
            first, again,
            "the same query over the same generations must return the same ordered result"
        );
    }
}

/// **The stated tie-break rule, pinned.** Two units with byte-identical text
/// score identically, so their order is decided entirely by
/// `LexicalHit::tie_break_key` — `(source_name, relative_path, ordinal,
/// unit_key)` ascending.
///
/// The fixture records the sources in the REVERSE of that order (`zeta`
/// first, `alpha` second), so an implementation accumulating into a hash map
/// would put them in whatever order that map iterated and fail here.
///
/// That score outranks arrival order at all is proved next door rather than
/// here: in
/// [`an_inadmissible_unit_with_a_perfect_lexical_match_is_never_returned`],
/// `vendor-lib` is last in every arrival ordering the store produces
/// (`knowledge` < `mailbox` < `vendor-lib`) and is asserted to come back
/// first, which only the score-descending sort can do.
#[test]
fn equal_scores_are_broken_by_the_stated_key_not_by_row_arrival_order() {
    let data = tempfile::tempdir().expect("data dir");
    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");

    for name in ["zeta", "alpha"] {
        let scan = hand_built_scan(
            name,
            SourceKind::LocalKnowledge,
            AuthorityClass::EstateReadonly,
            &format!("{name}@key-1"),
            vec![scanned_file(
                "note.md",
                MARKDOWN_EXTRACTOR,
                vec![document_unit(None, "PaymentRetryPolicy")],
            )],
        );
        record_scan(
            &mut db,
            &mut journal,
            &scan,
            None,
            &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
        )
        .expect("record tie fixture");
    }

    let filter = Admissibility::within_estate(D1_ESTATE);
    let answer = db
        .lexical_search(&LexicalQuery {
            text: "PaymentRetryPolicy",
            filter: &filter,
            family: None,
            limit: 50,
            semantic: SemanticRequest::Requested,
        })
        .expect("lexical search");
    let order: Vec<&str> = answer
        .hits
        .iter()
        .map(|hit| hit.source_name.as_str())
        .collect();
    assert_eq!(
        order,
        vec!["alpha", "zeta"],
        "tied hits order by the stated key, not by the order the rows were written"
    );
    assert_eq!(
        answer.hits[0].score, answer.hits[1].score,
        "the fixture must actually tie, or the tie-break is untested"
    );
}

// ------------------------------------------------------ index lifecycle

/// H2: the index is keyed by SourceGeneration, so a superseded generation's
/// postings are evicted with it — no stale hit survives a re-scan, and every
/// surviving hit cites the new generation.
#[test]
fn a_superseded_generations_postings_are_evicted_with_it() {
    let mut estate = estate();
    let before = estate.search(
        "INC0012345",
        &only_knowledge(),
        Some(LexicalFamily::Document),
    );
    let stale_generation = hit_at(&before, "docs/forms.md").generation_id.clone();

    // Re-scan the same source with different bytes: the ticket is gone.
    write(
        estate.root.path(),
        "docs/forms.md",
        "# Retry forms\n\nThe ticket reference was withdrawn.\n",
    );
    let knowledge = KnowledgeSource {
        name: "knowledge".to_string(),
        root: estate.root.path().to_path_buf(),
        ignore: Vec::new(),
        context_fields: ContextFields::declared(&[
            "number".to_string(),
            "short_description".to_string(),
        ]),
    };
    let recorded = scan_and_record(
        &mut estate.db,
        &mut estate.journal,
        &knowledge,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("re-record knowledge");
    assert!(
        matches!(recorded, ScanRecord::Recorded { .. }),
        "the re-scan must actually supersede: {recorded:?}"
    );

    let after = estate.search(
        "INC0012345",
        &only_knowledge(),
        Some(LexicalFamily::Document),
    );
    assert!(
        !finds(&after, "docs/forms.md"),
        "the superseded generation's postings must be gone: {:?}",
        paths(&after)
    );
    for hit in &after.hits {
        assert_ne!(
            hit.generation_id, stale_generation,
            "no surviving hit may cite the evicted generation"
        );
    }
    // The CSV row unit is unchanged bytes in the new generation, so it is
    // still findable — proving the eviction took the superseded generation
    // and not the index.
    let rows = estate.search(
        "INC0012345",
        &only_knowledge(),
        Some(LexicalFamily::RowText),
    );
    assert!(
        finds(&rows, "tickets.csv"),
        "the new generation's own postings must be there: {:?}",
        paths(&rows)
    );
}

/// The index is derived evidence (A1-01: the journal, Git and the original
/// bytes remain authority), so it must be rebuildable from the A1 rows alone
/// — and the rebuild must produce the same answer as the build.
#[test]
fn the_lexical_index_rebuilds_from_the_a1_rows_it_derives_from() {
    let mut estate = estate();
    let filter = Admissibility::within_estate(D1_ESTATE);
    let before = estate.search("PaymentRetryPolicy INC0012345", &filter, None);
    let outcome = estate.db.reindex_lexical().expect("reindex");
    assert!(outcome.indexed > 0, "the rebuild must actually index units");
    assert!(
        !outcome.truncated,
        "this fixture is far under MAX_ROWS generations; a true flag here means the cap logic is wrong"
    );
    let after = estate.search("PaymentRetryPolicy INC0012345", &filter, None);
    assert_eq!(
        before, after,
        "a rebuilt index must answer identically to the one built at staging time"
    );
}

/// A bounded answer says it is bounded. Nothing in this fixture reaches
/// `MAX_ROWS`, so the flag is false — which is the assertion that keeps it
/// from being a field nobody ever reads.
#[test]
fn a_bounded_answer_states_whether_the_posting_scan_was_capped() {
    let estate = estate();
    let answer = estate.search(
        "PaymentRetryPolicy",
        &Admissibility::within_estate(D1_ESTATE),
        None,
    );
    assert!(
        !answer.truncated,
        "this corpus is far under the cap; a true flag here means the cap logic is wrong"
    );
}

/// F-TH-01: the sibling of the test above, exercising the branch that one
/// never reaches — `MAX_ROWS + 1` postings for a single term, all admissible,
/// so the posting scan (`db.rs`'s `'terms` loop) must hit the cap and report
/// it. Without this test, `truncated = true;` could be deleted or made a
/// permanent no-op and nothing in the suite would notice.
#[test]
fn a_bounded_answer_reports_true_when_the_posting_scan_is_actually_capped() {
    let mut db = AtlasDb::open_in_memory().expect("atlas");
    let data = tempfile::tempdir().expect("data dir");
    let mut journal = Journal::open(data.path()).expect("journal");

    let unit_count = MAX_ROWS + 1;
    let units: Vec<ScannedUnit> = (0..unit_count as u64)
        .map(|ordinal| ScannedUnit {
            ordinal,
            kind: UnitKind::Document,
            heading_level: None,
            title: None,
            byte_start: 0,
            byte_end: 8,
            coordinate: None,
            text: "needleterm".to_string(),
        })
        .collect();
    let bulk = hand_built_scan(
        "bulk",
        SourceKind::LocalKnowledge,
        AuthorityClass::EstateReadonly,
        "bulk@key-1",
        vec![scanned_file("bulk.md", MARKDOWN_EXTRACTOR, units)],
    );
    record_scan(
        &mut db,
        &mut journal,
        &bulk,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record bulk");

    let answer = db
        .lexical_search(&LexicalQuery {
            text: "needleterm",
            filter: &Admissibility::within_estate(D1_ESTATE),
            family: None,
            limit: MAX_ROWS,
            semantic: SemanticRequest::Requested,
        })
        .expect("lexical search");
    assert!(
        answer.truncated,
        "{unit_count} admissible postings for one term must trip the {MAX_ROWS}-row cap"
    );
}
