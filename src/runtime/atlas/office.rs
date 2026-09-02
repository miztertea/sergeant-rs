//! Office document adapter (S4 Y2, G3) — **the one module in this tree that
//! may name `anydoc`.**
//!
//! ```text
//! bytes  ──►  anydoc::to_document  ──►  OfficeUnit (our own vocabulary)
//! ```
//!
//! # The replaceability boundary (owner ruling, 2026-08-27)
//!
//! The owner's sanction to adopt `anydoc` past a RUSTSEC advisory
//! (`knowledge/rulings/owner-rulings/anydoc-adoption-2026-08-27.md`, J4) is
//! conditioned on a narrow contract in **our own vocabulary** — bytes in,
//! document units out — with anydoc as one implementation behind it. So:
//!
//! * No `anydoc::` type, `anydoc::ConvertError` variant, or anydoc concept
//!   (a `Block`, a `Format`, a `model::Document`) may cross this module's own
//!   boundary. [`office_units`] returns [`OfficeUnit`]/[`OfficeError`] — both
//!   defined here, in this crate's own terms — never an anydoc type.
//! * Every anydoc-touching test lives in this module's own `#[cfg(test)]`
//!   block, not in `tests/`. `tests/y2_office_adapter.rs` proves the real
//!   parser end to end, but does so entirely through
//!   [`super::worker::WorkerBatch`]/[`super::worker::WorkerUnit`] — the wire
//!   vocabulary, which already never names anydoc — so it can exercise the
//!   real subprocess without becoming a second place anydoc is named.
//! * [`tests/y2_office_boundary.rs`](../../../../tests/y2_office_boundary.rs)
//!   pins this structurally, in the shape of `tests/x1_atlas_substrate.rs`'s
//!   existing one-owner test for the database driver: a token scan of every
//!   `.rs` file in the crate (`src/` and `tests/`) that fails if this
//!   module's crate is named anywhere but here (and that one test file,
//!   which necessarily writes the name once, as data, to check for it).
//!
//! That test — not this comment — is what makes the accepted RUSTSEC risk
//! reversible: ripping out anydoc later means rewriting this one file behind
//! an unchanged [`office_units`] signature, and the structural pin is what
//! would catch a caller that quietly started depending on anydoc's own
//! shapes instead of ours.
//!
//! # The UnitKind decision (schema decision, not an implementation detail)
//!
//! **Document/Section already fit Office units; no new [`UnitKind`] variant is
//! added.** [`domain::source::UnitKind`]'s own doc carries the decision and
//! the reasoning; this module is what exercises it. In short: a `.docx`
//! normalizes (via anydoc's block tree) into the same shape Markdown already
//! has — a whole-resource unit, plus flat, heading-delimited sections — so
//! reusing Document/Section avoids growing a second enum that would just
//! restate the same two shapes under format-specific names.
//!
//! # Why a *coordinate*, not a byte range, for Office sections
//!
//! [`super::text`]'s Markdown/plain-text units carry `byte_start`/`byte_end`
//! into the *original* bytes, because plain text IS the bytes: slicing the
//! input at those offsets reproduces the unit exactly (A1-12's provenance
//! rule, made checkable). A `.docx` cannot make that promise: the original
//! bytes are a compressed ZIP/XML container, anydoc's `model::Document` has
//! already unpacked, resolved and normalized them, and no position in that
//! model maps back to a byte offset in the ZIP's compressed stream — the
//! normalization is exactly what "any doc" buys, and losing byte-exact
//! back-reference is the honest cost of it. So [`OfficeUnit::coordinate`]
//! carries a **structural** coordinate instead.
//!
//! **The contract, in our own terms — not anydoc's.** A coordinate is
//! nothing more than: a stable, opaque, per-unit address that round-trips to
//! the same unit for the same bytes under the same extractor identity
//! ([`DOCX_EXTRACTOR`]). Concretely, any adapter behind this contract —
//! anydoc today, a replacement tomorrow — must produce a coordinate such
//! that:
//!
//! 1. it is `Some` for every unit that is not the whole-document unit (the
//!    whole-document unit needs nothing more specific than "the whole
//!    resource", so it stays `None`, exactly as text.rs's own Document unit
//!    carries no `heading_level`/`title` beyond what its first heading
//!    lends it);
//! 2. it is unique among the units of one document — two different
//!    sections of the same parse never share a coordinate;
//! 3. it is deterministic — re-running the same extractor identity against
//!    the same bytes reproduces the identical coordinate for the
//!    corresponding unit ([`office_units`]'s own purity guarantee, F6, makes
//!    this automatic rather than a separate promise to keep);
//! 4. it makes no write-back claim — a coordinate identifies a position for
//!    *citation*, never a position a caller could use to mutate the
//!    original resource (A1-12, "derived, not canonical" — see "Spreadsheet
//!    formats claim no write-back coordinates" below).
//!
//! Nothing in this contract requires any particular string shape. This
//! adapter happens to spell its coordinates `block:<index>` — the section's
//! starting position in anydoc's own top-level block sequence — because that
//! is the cheapest opaque address anydoc's own model hands back satisfying
//! the four properties above, not because `block:<n>` is part of the
//! contract itself. A tree-shaped replacement adapter is free to spell its
//! own coordinates however its own model makes cheapest (a dotted path, a
//! node id, anything opaque) as long as the four properties hold; nothing
//! downstream may assume the `block:` prefix, and
//! `tests/y2_office_adapter.rs` asserts the four properties, not the prefix.
//! [`super::worker::WorkerUnit::coordinate`] is the wire field this rides
//! on, and states the same contract in its own doc.
//!
//! Output is derived, never canonical (A1-12): re-running this extractor
//! against the same bytes and the same anydoc version reproduces the same
//! units, but the units are not the document — the `.docx` file is, and
//! [`office_units`]' caller is expected to keep citing the *original resource*
//! (its path, its content hash), never a temp file this adapter might read
//! bytes through.
//!
//! # Which lane owns which format (S6) — decided from the contract
//!
//! The owner ruled that routing one of the twelve formats this normalizer
//! parses is *"a failure of 0.3.0 completion criteria for estate
//! intelligence"*
//! (`knowledge/rulings/owner-rulings/twelve-formats-is-0.3.0-criteria-2026-08-30.md`,
//! **J4**). Eleven now route here; the twelfth deliberately does not. The
//! three decisions that were not mere routing:
//!
//! **`csv` stays relational — J5, a governing constraint.** A1 §6.4, in its
//! own words: *"A 100k-ticket ServiceNow export must **not** be normalized
//! into 100k Markdown documents just to make it searchable."* A1-13 keeps
//! CSV/JSON/Parquet corpora in DuckDB, read in place, with A1-14's textual
//! lane over operator-declared columns sharing the same row identity. This
//! normalizer *can* parse a CSV; routing it here would convert data into
//! prose, which §6.4 forbids by name. [`CSV_IS_NOT_A_DOCUMENT`] carries the
//! rule at the routing table, and two tests enforce it from both sides —
//! this module's own `csv_is_never_claimed_by_the_document_lane` and
//! `tests/x4_tabular_map.rs`'s routing-disjointness pin.
//!
//! **`xlsx`/`ods` take the document lane — J3, a settled authoritative
//! record.** This was the genuinely open one, and A1 §6.3 settles it
//! directly rather than by extension from §6.4: *"For Office spreadsheets,
//! readable Markdown/table normalization is **sufficient for Sprint 1
//! knowledge/search**, but it is not evidence that exact workbook sheet/cell
//! write-back coordinates were preserved."* [OWNER-02]. Three further facts
//! agree and none conflict, so no rung above J3 is engaged:
//!
//! * §6.4's relational lane is cited to DuckDB's own file readers, which
//!   A1 names exhaustively as *"CSV, JSON and Parquet"* — not workbooks.
//!   A1-13 names the same three. The relational lane's own evidence does
//!   not reach `.xlsx`.
//! * Reaching it would need DuckDB's `excel` extension, i.e. an install-time
//!   fetch, which A1 §14/§15 and A2-12 forbid as a class: *"No surprise
//!   grammar/model downloads during a running stage"*. Option (a) is not
//!   merely unevidenced, it is blocked.
//! * A2 §9 already describes the retrieval shape this produces: *"A result
//!   from a spreadsheet may identify sheet/row/table coordinates when the
//!   adapter can prove them; otherwise it must use the strongest coordinate
//!   actually produced, not invent cell precision."*
//!
//! Not decided here, and deliberately: whether a LARGE workbook should also
//! get a relational lane (§6.4's two-lane shape). That would need a size
//! threshold, and a numeric bound with no dated measurement behind it is a
//! fabricated rationale — there is no measurement of workbook sizes in this
//! estate to derive one from. What ships is the lane §6.3 authorizes for
//! *knowledge/search*; a second lane is a later, evidence-backed decision,
//! not an assumption baked in now.
//!
//! **`pdf` splits on content.** A text-bearing PDF extracts natively (and
//! through a different call than every other format — see [`office_units`],
//! since this normalizer has no document model for PDFs at all). A scanned
//! or image-only PDF is [`OfficeError::NeedsOcr`], a NAMED coverage gap
//! pointing at the OCR epic — never silence and never a false empty
//! extraction, per A1 §15: *"Missing capability is never represented as
//! successful empty evidence."* OCR itself is the one thing deliberately
//! outside 0.3.0.
//!
//! # Spreadsheet formats claim no write-back coordinates
//!
//! Stated as a design rule before any code exercised it (S4 Y2), and now
//! exercised: **a spreadsheet routed through this contract must never claim
//! a coordinate a caller could use to write back to a specific cell.** Grid
//! positions the normalizer resolves for a spreadsheet are read-only derived
//! evidence — a `block:<index>` coordinate (or no coordinate) is honest; a
//! `Sheet1!A1`-shaped coordinate would assert a two-way binding this adapter
//! does not have and never will, because normalized text has no cell
//! back-reference the underlying model preserves. That is A1 §6.3's own
//! caveat and A2 §9's *"not invent cell precision"*, and
//! `xlsx_fixture_yields_table_text_and_claims_no_cell_coordinate` is what
//! holds it.
//!
//! # A known, honest gap: what anydoc's docx frontend does not expose
//!
//! `tests/fixtures/anydoc_corpus/manifest.json`'s hand-verified counts are
//! pinned at the raw OOXML level *specifically* so they do not depend on any
//! extractor's own vocabulary (`MANIFEST.md`'s own words). Two of its fields
//! are, empirically, **not recoverable through anydoc's normalized model**,
//! and this module says so rather than fabricating an answer:
//!
//! * `numId`/`ilvl` per list paragraph, specifically — anydoc resolves
//!   numbering into an actual nested [`List`]/[`ListItem`] tree (marker
//!   kind, start, nesting by containment) rather than preserving the OOXML
//!   numbering identity that produced it. That is the abstraction working
//!   as designed — "any doc" means normalizing across formats that do not
//!   all *have* a `numId` — and asserting the raw id back here would put an
//!   OOXML-specific concept in our own vocabulary, which the boundary above
//!   forbids anyway. What that tree *does* carry — nesting depth, and
//!   ordered-vs-bulleted marker kind — is not part of this gap:
//!   [`render_list`] renders it back out as an indent-and-marker textual
//!   proxy (two spaces per level, [`MarkerKind::label`](anydoc::model::MarkerKind::label)
//!   per item), so a numbered top-level item and a bulleted second-level
//!   item stay visually and texturally distinguishable in [`OfficeUnit::text`],
//!   even though the source `numId` that drove them is gone.
//! * `header_parts`/`footer_parts` — anydoc's `model::Document` carries body
//!   content and notes only; page headers/footers are DOCX package parts
//!   outside the main reading flow, and anydoc's docx frontend does not parse
//!   them (verified by reading `formats/docx/` — no `header`/`footer` part is
//!   read anywhere in it). A Markdown-shaped output has nowhere to put a
//!   running header anyway, so this is not a bug this adapter can fix.
//!
//! Every other field in the manifest — `body_top_level_paragraphs` (defined
//! recursively over paragraph-equivalent blocks, so it holds regardless of
//! how a list nests), `heading_paragraphs`, `tables`/`table_cell_paragraphs`,
//! and both footnote counts — **is** independently verified against this
//! adapter's real output, in this module's own tests.

use std::sync::Once;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::domain::source::UnitKind;

/// Extractor identity + version, per format (F7's second cache-key input, and
/// the provenance "normalizer identity" A1 §6.3 requires) — versioned so a
/// future anydoc bump, or a rewrite behind this same contract, changes the
/// identity and therefore the derived key, exactly as
/// [`super::text::MARKDOWN_EXTRACTOR`]'s own doc explains for its own version
/// tag.
///
/// **One identity per format, not one for "office" (S6).** Each of anydoc's
/// frontends is a separate parser over a separate container specification
/// (`formats/docx/`, `formats/odf/`, `formats/pptx/`, `formats/sheet/`,
/// `formats/rtf/`, `formats/epub/`, `formats/doc/`, `formats/ppt/`, and
/// pdf-inspector behind `formats/pdf.rs`), with independent fidelity and
/// independent failure modes; a single shared identity would make a
/// frontend-specific fix invalidate every other format's cached facts, and
/// would flatten a coverage surface that is more useful naming which
/// normalizer read a resource.
pub const DOCX_EXTRACTOR: &str = "anydoc/0.2.4+docx/v1";
/// Binary Word 97–2003, and the RTF-wearing-a-`.doc`-extension case anydoc's
/// own dispatcher routes to its RTF frontend — see [`OFFICE_EXTENSIONS`].
pub const DOC_EXTRACTOR: &str = "anydoc/0.2.4+doc/v1";
/// OpenDocument Text.
pub const ODT_EXTRACTOR: &str = "anydoc/0.2.4+odt/v1";
/// Binary PowerPoint 97–2003.
pub const PPT_EXTRACTOR: &str = "anydoc/0.2.4+ppt/v1";
/// PresentationML.
pub const PPTX_EXTRACTOR: &str = "anydoc/0.2.4+pptx/v1";
/// OpenDocument Presentation.
pub const ODP_EXTRACTOR: &str = "anydoc/0.2.4+odp/v1";
/// Rich Text Format.
pub const RTF_EXTRACTOR: &str = "anydoc/0.2.4+rtf/v1";
/// EPUB 2 and 3.
pub const EPUB_EXTRACTOR: &str = "anydoc/0.2.4+epub/v1";
/// SpreadsheetML workbooks — see "Spreadsheet formats claim no write-back
/// coordinates" above for why these produce table text and never a cell
/// coordinate.
pub const XLSX_EXTRACTOR: &str = "anydoc/0.2.4+xlsx/v1";
/// OpenDocument Spreadsheet — same rule as [`XLSX_EXTRACTOR`].
pub const ODS_EXTRACTOR: &str = "anydoc/0.2.4+ods/v1";
/// Text-bearing PDF. A scanned/image-only PDF is [`OfficeError::NeedsOcr`],
/// a named coverage gap pointing at the OCR epic — never a silent empty
/// extraction (A1 §15: "Missing capability is never represented as
/// successful empty evidence").
pub const PDF_EXTRACTOR: &str = "anydoc/0.2.4+pdf/v1";

/// Every identity this module can write to `source.files`, as one code-owned
/// `LIKE` pattern (S6) — the document-family twin of
/// [`super::db::CODE_EXTRACTOR_LIKE`], and F12-safe for the same reason: a
/// fixed constant this crate owns, never a client-supplied pattern.
///
/// Exhaustive by construction, and that is the point. Before S6 the document
/// filter enumerated one office identity by name, so an office format that
/// landed a NEW identity fell silently outside `--content document` until
/// someone remembered to widen a list in another module. Every identity in
/// [`OFFICE_EXTENSIONS`] begins with this prefix (asserted in this module's
/// own tests over the whole table, so a twelfth format cannot land outside
/// it by accident), which makes the filter cover a format the day it is
/// routed rather than the day someone notices.
///
/// The prefix is the vendor identity every one of this module's own
/// `..._EXTRACTOR` constants already carries; nothing outside this module
/// spells it, which is what keeps the replaceability boundary
/// (`tests/y2_office_boundary.rs`) intact — a replacement adapter changes
/// this constant here, in the one file allowed to name its implementation.
pub const OFFICE_EXTRACTOR_LIKE: &str = "anydoc/%";

/// The ONE routing table: extension → extractor identity (S6).
///
/// Eleven of the twelve formats anydoc parses. The twelfth, `csv`, is
/// **deliberately absent** and must stay absent — see
/// [`CSV_IS_NOT_A_DOCUMENT`] for the contract sentence that decides it.
///
/// Extension-driven and nothing else, exactly as [`super::text::extractor_for`]
/// is, and for the same reason: an unclaimed extension is honestly
/// `unsupported`, not guessed at. Only the canonical extension the owner
/// ruling names for each format is claimed — not anydoc's own wider alias
/// set (`.docm`, `.xlsm`, `.xlsb`, `.xls`, `.pptm`, `.ppsx`, `.ppsm`,
/// `.pps`, `.pot`), which shares a parser but is not part of this corpus or
/// its footprint measurement; routing an alias here would silently widen
/// what this wave's fixtures actually cover (R1 — the aliases do not need to
/// exist yet).
pub const OFFICE_EXTENSIONS: &[(&str, &str)] = &[
    ("doc", DOC_EXTRACTOR),
    ("docx", DOCX_EXTRACTOR),
    ("epub", EPUB_EXTRACTOR),
    ("odp", ODP_EXTRACTOR),
    ("ods", ODS_EXTRACTOR),
    ("odt", ODT_EXTRACTOR),
    ("pdf", PDF_EXTRACTOR),
    ("ppt", PPT_EXTRACTOR),
    ("pptx", PPTX_EXTRACTOR),
    ("rtf", RTF_EXTRACTOR),
    ("xlsx", XLSX_EXTRACTOR),
];

/// Why `.csv` is not in [`OFFICE_EXTENSIONS`], in the contract's own words —
/// a `const` rather than a comment so the rule is quotable from the test
/// that enforces it, and so deleting the rule and deleting its justification
/// are the same edit.
///
/// A1 §6.4, verbatim: *"A 100k-ticket ServiceNow export must **not** be
/// normalized into 100k Markdown documents just to make it searchable."*
/// A1-13 keeps CSV/JSON/Parquet corpora relational in DuckDB. `.csv` already
/// has that lane — [`super::tabular::format_for`] → `Walk::dataset` → DuckDB
/// reads it in place, with A1-14's textual lane over operator-declared
/// columns sharing the same row identity. anydoc *can* parse a CSV into a
/// table block; routing it here would convert data into prose, which is the
/// one thing §6.4 forbids by name. J5 — a governing constraint, not a
/// preference.
pub const CSV_IS_NOT_A_DOCUMENT: &str = "csv stays relational (A1 §6.4, A1-13): a dataset is read in place by DuckDB, never \
     normalized into prose documents";

/// The extractor for a path, by extension, or `None` for anything this
/// adapter does not claim.
///
/// Called from a real scan since S4 Y8: [`super::scan::worker_extractor_for`]
/// unions this with [`super::archive::extractor_for`]/
/// [`super::mail::extractor_for`], and [`super::scan::Walk::file`]/
/// [`super::git::extract_blobs`] dispatch a path this claims to the real
/// supervised worker (`super::worker::run_worker`, called directly from
/// [`super::scan::dispatch_worker_resource`] under the whole-scan
/// intelligence-lane permit [`super::lane::scan_local_knowledge_on_lane`]/
/// [`super::lane::scan_estate_git_on_lane`] already hold — not through
/// `lane::run_worker_on_lane`, which remains test-only, see its own doc)
/// rather than reporting it unsupported.
pub fn extractor_for(relative: &str) -> Option<&'static str> {
    let extension = std::path::Path::new(relative)
        .extension()
        .and_then(|e| e.to_str())?
        .to_ascii_lowercase();
    OFFICE_EXTENSIONS
        .iter()
        .find(|(candidate, _)| *candidate == extension)
        .map(|(_, extractor)| *extractor)
}

/// Whether `extractor` is one this module runs — the predicate
/// `src/bin/atlas_worker.rs` dispatches on, so that binary never has to
/// spell eleven identities (and so a twelfth would reach it automatically).
pub fn is_office_extractor(extractor: &str) -> bool {
    OFFICE_EXTENSIONS
        .iter()
        .any(|(_, identity)| *identity == extractor)
}

/// One Office structure unit, in our own vocabulary — see the module doc's
/// "Why a coordinate, not a byte range" for why this is not
/// [`super::text::StructureUnit`] reused verbatim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfficeUnit {
    /// Whole resource, or a heading-delimited span.
    pub kind: UnitKind,
    /// Heading depth for a section (1..=6, anydoc's own outline-level
    /// clamp — see anydoc's `Block::Heading` doc); `None` for the document
    /// unit and for a preamble section that precedes the first heading.
    pub heading_level: Option<u8>,
    /// Heading text, trimmed; `None` when there is none.
    pub title: Option<String>,
    /// A stable, opaque, per-unit address into the normalized document —
    /// see the module doc's "The contract, in our own terms" for the four
    /// properties this must hold (present, unique per document, deterministic,
    /// no write-back claim). `None` for the whole-document unit, which needs
    /// nothing more specific. This adapter spells it `block:<n>`; that
    /// spelling is not itself the contract — never a byte offset either
    /// way.
    pub coordinate: Option<String>,
    /// The unit's own rendered text.
    pub text: String,
}

/// Why [`office_units`] could not produce units — in our own vocabulary. Every
/// variant carries anydoc's own message text (via `Display`, not the typed
/// value) so the coverage row a caller builds from this stays informative
/// without anydoc's `ConvertError` ever crossing the boundary.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OfficeError {
    /// The document is structurally unusable, or its container could not be
    /// read at all — includes the recovery-watch refusal below: a document
    /// anydoc's own lenient parser had to *repair* rather than read cleanly
    /// is treated as malformed by this adapter's stricter policy, not as a
    /// silent partial success.
    #[error("malformed document: {0}")]
    Malformed(String),
    /// A fixed safety limit was crossed (decompression, nesting depth, node
    /// count, repeat expansion, retained asset bytes) — a hostile or
    /// pathological input, refused before it could exhaust memory.
    #[error("resource limit exceeded: {0}")]
    ResourceLimit(String),
    /// Scanned or image-only content needing OCR, which this build does not
    /// do (OCR is out of scope post-S4 by owner ruling 3).
    #[error("needs OCR, unsupported in this build: {0}")]
    NeedsOcr(String),
    /// The document is encrypted or password-protected — a NAMED coverage
    /// gap with its own detail text, distinct from [`Self::Malformed`]:
    /// anydoc's own API distinguishes an encrypted document from an
    /// unreadable one, and collapsing the two would tell an operator their
    /// file is broken when in fact it is locked and they hold the key.
    #[error("document is encrypted or password-protected")]
    Encrypted,
    /// The format, or this build's ability to convert it, is not supported —
    /// anydoc's own `Unsupported`, kept distinct from [`Self::Malformed`]
    /// for the same reason [`Self::Encrypted`] is: "we cannot read this
    /// kind of file" and "this file is damaged" are different coverage
    /// answers and an operator acts on them differently.
    #[error("unsupported document: {0}")]
    Unsupported(String),
}

/// Extract Office structure units from `.docx` bytes — the whole of this
/// wave's adapter, and the only function this module exports for actually
/// running anydoc.
///
/// Pure (F6's adapter-shape mandate, unchanged across the process boundary
/// worker.rs carries it over): no file is opened beyond the bytes already
/// handed in, no database or journal is touched, no clock is read. Two calls
/// on equal bytes are equal — proven directly in this module's own tests,
/// the same way `text.rs`'s `extraction_is_a_pure_function_of_its_input`
/// proves it for Markdown.
pub fn office_units(bytes: &[u8], extractor: &str) -> Result<Vec<OfficeUnit>, OfficeError> {
    install_recovery_watch();
    // `RECOVERY_WATCH` is one process-global flag (module doc): correct for
    // the worker binary, which runs exactly one `office_units` call per
    // process, and would otherwise race under this crate's own multi-
    // threaded test runner, where several unit tests can call `office_units`
    // concurrently in one process. Serializing the reset-call-check sequence
    // behind this lock costs nothing in production (there is never a second
    // concurrent call to race) and makes the watch's answer belong to
    // exactly one call in every context, including tests. A poisoned lock
    // (a prior call panicked mid-check) is recovered rather than propagated
    // — the flag itself is reset immediately below either way.
    static CALL_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = CALL_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    RECOVERY_WATCH.warned.store(false, Ordering::SeqCst);

    // PDF is the one format with no document-model form: anydoc's own
    // `to_document` refuses it by name ("PDF converts directly to Markdown;
    // use to_markdown or to_markdown_bytes" — `formats/mod.rs`), because
    // pdf-inspector emits Markdown rather than a block tree. So the PDF arm
    // takes the Markdown and sections it with `text.rs`'s OWN Markdown
    // sectioner (R2 — that walker already exists, is already the shape this
    // module's `flatten` mirrors, and reusing it means a PDF's sections and
    // a `.md` file's sections are cut the same way). No second Markdown
    // parser, and no anydoc type crosses the boundary: what leaves anydoc
    // here is a `String`.
    if extractor == PDF_EXTRACTOR {
        let markdown = anydoc::to_markdown_bytes(bytes, anydoc::Format::Pdf)
            .map_err(classify_convert_error)?;
        return Ok(markdown_to_office_units(&markdown));
    }

    let document = anydoc::to_document(bytes, format_for_extractor(extractor)?)
        .map_err(classify_convert_error)?;

    // Our own, stricter policy on top of anydoc's lenient default (module
    // doc, "The replaceability boundary" + fixture 05): anydoc's XML layer
    // *repairs* an unclosed/mismatched element and returns `Ok` with
    // whatever it could recover, logging a warning rather than erroring
    // (verified directly against this crate's own `package::xml::parse_xml`
    // — `check_end_names = false`, and a recovery is `log::warn!`, never
    // returned as an `Err`). A coverage-honest adapter cannot accept that:
    // "recovered from a well-formedness problem" and "read cleanly" are not
    // the same claim, and F8 promises the honest one. `log`'s own message
    // text is explicitly *not* a stable API (anydoc's `lib.rs` doc says so),
    // so this counts *whether the specific xml-recovery event fired* —
    // never matches on wording. It does NOT count every WARN in the call
    // tree: anydoc's own doc names two different event classes sharing this
    // one facade ("Recovery and skipped-content events are reported through
    // the log facade"), and only the first is a well-formedness problem.
    // Verified directly against anydoc 0.2.4's source: a *benign*,
    // by-design skip — a dangling relationship target
    // (`package/relationships.rs`), a corrupt optional part
    // (`package/archive.rs`), an unresolvable related-part path
    // (`formats/docx/mod.rs`), a numbering instance referencing an unknown
    // abstract id (`formats/docx/numbering.rs`), or a corrupt chart/diagram
    // part (`formats/docx/content.rs`) — also logs at WARN, on a document
    // anydoc still returns fully well-formed. Only `package::xml`'s own
    // `log::warn!("recovered malformed xml …")` (`package/xml.rs`) is the
    // well-formedness-recovery signal; [`RecoveryWatch::log`] below matches
    // on that module's own log target (`module_path!()`, the default target
    // when a macro call names none — confirmed against the `log` crate's
    // own docs), not on level alone.
    if RECOVERY_WATCH.warned.swap(false, Ordering::SeqCst) {
        return Err(OfficeError::Malformed(
            "anydoc recovered from a well-formedness problem while parsing this document \
             (a mismatched or unclosed element); refused rather than trusted, because a \
             repaired document is not a cleanly read one"
                .to_string(),
        ));
    }

    Ok(flatten(&document.blocks))
}

/// This module's own extractor identity → the anydoc format that reads it.
/// The mapping lives here and nowhere else: an `anydoc::Format` value is an
/// anydoc concept, and the replaceability boundary (module doc) forbids one
/// crossing out of this file, so every caller names an identity string from
/// [`OFFICE_EXTENSIONS`] instead.
///
/// An identity this build does not know is [`OfficeError::Unsupported`] —
/// the honest answer for a worker invoked with an extractor from a newer or
/// older build, never a panic and never a silent fall-through to some
/// default format.
fn format_for_extractor(extractor: &str) -> Result<anydoc::Format, OfficeError> {
    Ok(match extractor {
        _ if extractor == DOCX_EXTRACTOR => anydoc::Format::Docx,
        _ if extractor == DOC_EXTRACTOR => anydoc::Format::Doc,
        _ if extractor == ODT_EXTRACTOR => anydoc::Format::Odt,
        _ if extractor == PPT_EXTRACTOR => anydoc::Format::Ppt,
        _ if extractor == PPTX_EXTRACTOR => anydoc::Format::Pptx,
        _ if extractor == ODP_EXTRACTOR => anydoc::Format::Odp,
        _ if extractor == RTF_EXTRACTOR => anydoc::Format::Rtf,
        _ if extractor == EPUB_EXTRACTOR => anydoc::Format::Epub,
        _ if extractor == XLSX_EXTRACTOR => anydoc::Format::Excel,
        _ if extractor == ODS_EXTRACTOR => anydoc::Format::Ods,
        // `PDF_EXTRACTOR` never reaches here: `office_units` takes the
        // Markdown arm above before calling this, because there is no
        // document model to ask for.
        _ => {
            return Err(OfficeError::Unsupported(format!(
                "no office format is routed to extractor identity {extractor:?} in this build"
            )));
        }
    })
}

/// Section anydoc's PDF Markdown with [`super::text::markdown_units`] and
/// re-address it in this module's own vocabulary.
///
/// The coordinate spelling here is `md-offset:<n>` rather than `block:<n>`,
/// because the addressable thing is a position in the DERIVED Markdown, not
/// a position in a block tree — and deliberately not a byte range field,
/// which would read as an offset into the original PDF bytes it emphatically
/// is not. Both spellings satisfy the module doc's four coordinate
/// properties (present on every non-document unit, unique within one parse,
/// deterministic, no write-back claim); nothing downstream may assume
/// either prefix, which is exactly why the module doc says the spelling is
/// not the contract.
fn markdown_to_office_units(markdown: &str) -> Vec<OfficeUnit> {
    super::text::markdown_units(markdown)
        .into_iter()
        .map(|unit| OfficeUnit {
            kind: unit.kind,
            heading_level: unit.heading_level,
            title: unit.title,
            coordinate: match unit.kind {
                UnitKind::Document => None,
                UnitKind::Section => Some(format!("md-offset:{}", unit.byte_start)),
            },
            text: markdown
                .get(unit.byte_start..unit.byte_end)
                .unwrap_or_default()
                .trim()
                .to_string(),
        })
        .collect()
}

fn classify_convert_error(error: anydoc::ConvertError) -> OfficeError {
    match error {
        anydoc::ConvertError::ResourceLimit { .. } => OfficeError::ResourceLimit(error.to_string()),
        anydoc::ConvertError::NeedsOcr { .. } => OfficeError::NeedsOcr(error.to_string()),
        anydoc::ConvertError::Encrypted => OfficeError::Encrypted,
        // `ConvertError` is `#[non_exhaustive]`: every variant this build
        // knows about is named above, and anything a future anydoc version
        // adds falls in here — honestly `Malformed` (a document this build
        // cannot make sense of), never a silent panic on an unmatched arm.
        anydoc::ConvertError::Unsupported(_) => OfficeError::Unsupported(error.to_string()),
        anydoc::ConvertError::Malformed { .. }
        | anydoc::ConvertError::MissingPart { .. }
        | anydoc::ConvertError::Io(_)
        | _ => OfficeError::Malformed(error.to_string()),
    }
}

/// Whole-document unit, plus flat, heading-delimited sections — the same
/// shape [`super::text::markdown_units`] builds over ATX headings, walked
/// here over anydoc's own `Block::Heading` transitions instead.
fn flatten(blocks: &[anydoc::model::Block]) -> Vec<OfficeUnit> {
    let mut units = vec![OfficeUnit {
        kind: UnitKind::Document,
        heading_level: None,
        title: None,
        coordinate: None,
        text: render_blocks(blocks),
    }];

    let heading_indices: Vec<usize> = blocks
        .iter()
        .enumerate()
        .filter_map(|(index, block)| {
            matches!(block, anydoc::model::Block::Heading { .. }).then_some(index)
        })
        .collect();
    if heading_indices.is_empty() {
        return units;
    }
    units[0].title = heading_title(&blocks[heading_indices[0]]);

    // Anything before the first heading is its own preamble section, unless
    // it renders to nothing — the same "blank preamble is not evidence" rule
    // `markdown_units` applies.
    let first = heading_indices[0];
    if first > 0 {
        let preamble = render_blocks(&blocks[..first]);
        if !preamble.trim().is_empty() {
            units.push(OfficeUnit {
                kind: UnitKind::Section,
                heading_level: None,
                title: None,
                coordinate: Some(format!("block:{first}")),
                text: preamble,
            });
        }
    }
    for (position, &start) in heading_indices.iter().enumerate() {
        let end = heading_indices
            .get(position + 1)
            .copied()
            .unwrap_or(blocks.len());
        let (level, title) = match &blocks[start] {
            anydoc::model::Block::Heading { level, content, .. } => (
                Some(*level),
                (!content.is_empty()).then(|| anydoc::model::inlines_to_plain_text(content)),
            ),
            _ => (None, None),
        };
        units.push(OfficeUnit {
            kind: UnitKind::Section,
            heading_level: level,
            title,
            coordinate: Some(format!("block:{start}")),
            text: render_blocks(&blocks[start..end]),
        });
    }
    units
}

fn heading_title(block: &anydoc::model::Block) -> Option<String> {
    match block {
        anydoc::model::Block::Heading { content, .. } => {
            (!content.is_empty()).then(|| anydoc::model::inlines_to_plain_text(content))
        }
        _ => None,
    }
}

/// Render a block range to plain, legible text — this adapter's own
/// projection, independent of anydoc's private Markdown renderer
/// (`render::markdown` is not `pub`; `to_markdown_bytes` renders a *whole*
/// document, not an addressable sub-range, so it cannot produce one
/// section's own text). Good enough for search/retrieval evidence; not a
/// faithful Markdown re-serialization and not trying to be one.
fn render_blocks(blocks: &[anydoc::model::Block]) -> String {
    let mut out = String::new();
    for block in blocks {
        render_block(block, &mut out);
    }
    out
}

fn render_block(block: &anydoc::model::Block, out: &mut String) {
    use anydoc::model::Block;
    match block {
        Block::Heading { content, .. } => {
            push_line(out, &anydoc::model::inlines_to_plain_text(content));
        }
        Block::Paragraph(inlines) => {
            push_line(out, &anydoc::model::inlines_to_plain_text(inlines));
        }
        Block::List(list) => render_list(list, out),
        Block::Table(table) => render_table(table, out),
        Block::BlockQuote(inner) => {
            for child in inner {
                render_block(child, out);
            }
        }
        Block::CodeBlock { text, .. } => push_line(out, text),
        Block::Rule => push_line(out, "---"),
        Block::Math(tex) => push_line(out, tex),
    }
}

/// Renders one list level's items, in order. Each item gets a rendered
/// marker — the list's own [`MarkerKind::label`](anydoc::model::MarkerKind::label)
/// at its resolved ordinal, or the item's literal `marker_label` override
/// for composite source numbering anydoc can't reproduce from marker+
/// position alone — and a nested list's own item lines are indented two
/// spaces past their parent item's marker, one "  " added at each level as
/// the recursion unwinds. This recovers, as a textual proxy, the
/// containment hierarchy and the ordered/bulleted marker family anydoc's
/// own model retains (module doc, "known, honest gap": what it still can't
/// recover is the *raw* `numId`/`ilvl` identity that produced this
/// hierarchy, an OOXML-specific concept the boundary above forbids keeping
/// anyway — the hierarchy and marker kind those numbers produced are not
/// the same claim as the numbers themselves, and only the latter is lost).
fn render_list(list: &anydoc::model::List, out: &mut String) {
    for (position, item) in list.items.iter().enumerate() {
        let ordinal = list.start + position as u64;
        let marker = item
            .marker_label
            .clone()
            .unwrap_or_else(|| list.marker.label(ordinal));

        let mut item_text = String::new();
        for child in &item.blocks {
            render_block(child, &mut item_text);
        }
        if item_text.is_empty() {
            push_line(out, &marker);
            continue;
        }
        let mut lines = item_text.lines();
        let first = lines.next().unwrap_or_default();
        push_line(out, &format!("{marker} {first}"));
        for line in lines {
            // `line` may itself already be a nested list's own marker line
            // (`render_block` above dispatches a `Block::List` child right
            // back into this function) — either way it gets exactly one
            // more "  " than it arrived with, which is what accumulates
            // into a full per-depth indent as the recursion returns.
            // `push_line` trims both ends, which would strip this
            // indentation right back off, so this uses the raw pusher
            // instead.
            push_indented_line(out, &format!("  {line}"));
        }
    }
}

fn render_table(table: &anydoc::model::Table, out: &mut String) {
    for row in &table.grid {
        let mut cells = Vec::with_capacity(row.len());
        for slot in row {
            if let anydoc::model::CellSlot::Origin(cell) = slot {
                let mut cell_text = String::new();
                for block in &cell.blocks {
                    render_block(block, &mut cell_text);
                }
                cells.push(cell_text.trim().replace('\n', " "));
            }
        }
        if !cells.is_empty() {
            push_line(out, &cells.join(" | "));
        }
    }
}

fn push_line(out: &mut String, text: &str) {
    if text.trim().is_empty() {
        return;
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(text.trim());
}

/// [`push_line`] without the trim — for a line whose leading whitespace is
/// deliberate (a [`render_list`] indent step), not incidental source
/// formatting to be discarded.
fn push_indented_line(out: &mut String, text: &str) {
    if text.is_empty() {
        return;
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(text);
}

// ------------------------------------------------------- the recovery watch

/// A process-global `log::Log` whose only job is to notice whether the one
/// well-formedness-recovery event fired during one [`office_units`] call — see
/// that function's own doc for why, and for why this is narrower than "any
/// WARN": anydoc's docx pipeline logs an unrelated, benign class of
/// "skipped content" WARN too (a dangling relationship target, a corrupt
/// optional part, an unresolvable numbering instance, …), and those must
/// never trip this adapter's stricter refusal. Message text is never
/// inspected (anydoc's own doc: log wording is not a stable API); only the
/// event's *target* — `anydoc::package::xml`, the one module that logs the
/// actual recovery — is.
struct RecoveryWatch {
    warned: AtomicBool,
}

impl log::Log for RecoveryWatch {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Warn
    }

    fn log(&self, record: &log::Record<'_>) {
        // `enabled` above is a level-only gate (the cheap filter every
        // caller of the `log` macros consults before even building a
        // `Record`); the target check that actually distinguishes
        // well-formedness recovery from a benign skipped-content WARN has
        // to happen here, once a `Record` exists to inspect.
        if record.level() <= log::Level::Warn && record.target() == XML_RECOVERY_LOG_TARGET {
            self.warned.store(true, Ordering::SeqCst);
        }
    }

    fn flush(&self) {}
}

/// The one module whose `log::warn!` call is the well-formedness-recovery
/// signal (`anydoc::package::xml`'s own `parse_xml`, verified against
/// anydoc 0.2.4's source — see [`office_units`]'s doc). `log`'s default
/// target is the call site's `module_path!()` (confirmed against the `log`
/// crate's own docs), so this is exactly what a bare, target-less
/// `log::warn!` inside that module produces.
const XML_RECOVERY_LOG_TARGET: &str = "anydoc::package::xml";

static RECOVERY_WATCH: RecoveryWatch = RecoveryWatch {
    warned: AtomicBool::new(false),
};

/// Install [`RECOVERY_WATCH`] as the process's global logger, once.
///
/// Safe to call from every caller of [`office_units`] in this process,
/// including repeatedly across calls (idempotent via [`Once`]) and including
/// concurrently (`log::set_logger` itself is the synchronization point).
/// Intended to be called only from [`super::super::atlas`]'s worker binary
/// (`src/bin/atlas_worker.rs`), which installs no other `log::Log` — never
/// from the `sgt` daemon binary, which owns the process-global logger slot
/// for its own `tracing_subscriber::fmt().init()` (`cli.rs`).
///
/// That "never in-process in the daemon" half used to be prose only — a
/// promise nothing checked. It is checkable now: `log::set_logger` itself
/// fails if a *different* logger already occupies the process-global slot,
/// and `office.rs` is `pub mod`, reachable from the daemon binary today even
/// though nothing currently calls [`office_units`] there. Previously that
/// failure was silently swallowed (`let _ = log::set_logger(..)`), which
/// would have let a future in-process call from the daemon either silently
/// lose this module's own recovery-watch signal (if the daemon's logger won
/// the race) or silently hijack the daemon's own logging (if this module's
/// `Once` ran first) — a race, either way, never an error. Now the first
/// caller in a process that already has a foreign logger installed panics
/// loudly instead: whoever removes the subprocess indirection this
/// invariant depends on trips this the moment `office_units` first runs
/// in-process next to another logger, rather than racing it unnoticed. A
/// second call from *this* module's own `Once` — the expected, safe,
/// repeated-call case documented above — never reaches `set_logger` twice in
/// the first place, so it cannot trip this.
fn install_recovery_watch() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        log::set_logger(&RECOVERY_WATCH).unwrap_or_else(|e| {
            panic!(
                "office_units tried to install its recovery-watch logger, but a different \
                 `log::Log` is already installed in this process ({e}). This function must \
                 only ever run inside the sgt-atlas-worker subprocess, never in-process \
                 alongside another logger (e.g. the sgt daemon's own tracing_subscriber) — \
                 see this function's own doc. If this fired, something now calls office_units \
                 in-process next to another logger; that invariant no longer holds and must be \
                 fixed before this call is safe."
            )
        });
        log::set_max_level(log::LevelFilter::Warn);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The docx corpus's own entry point — `office_units` under the one
    /// extractor identity these S4 fixtures were built for.
    fn docx(bytes: &[u8]) -> Result<Vec<OfficeUnit>, OfficeError> {
        office_units(bytes, DOCX_EXTRACTOR)
    }

    fn fixture(name: &str) -> Vec<u8> {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/anydoc_corpus/docx_fixtures")
            .join(name);
        std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    /// Regression pin for the recovery-watch narrowing: a benign
    /// skipped-content WARN from elsewhere in anydoc's docx pipeline must
    /// not trip the watch, but the actual `anydoc::package::xml`
    /// well-formedness-recovery event still must. Exercised directly
    /// against [`RecoveryWatch::log`] with hand-built [`log::Record`]s
    /// (rather than a corpus fixture) because the distinguishing signal is
    /// the log target, not anything a parsed document's own shape can
    /// assert on.
    #[test]
    fn recovery_watch_ignores_benign_warn_events_outside_xml_recovery() {
        use log::Log as _;

        let watch = RecoveryWatch {
            warned: AtomicBool::new(false),
        };

        let benign = log::Record::builder()
            .level(log::Level::Warn)
            .target("anydoc::formats::docx::numbering")
            .args(format_args!(
                "numbering instance 3 references unknown abstract 7"
            ))
            .build();
        watch.log(&benign);
        assert!(
            !watch.warned.load(Ordering::SeqCst),
            "a benign skipped-content WARN outside {XML_RECOVERY_LOG_TARGET} must not trip the \
             recovery watch"
        );

        let recovery = log::Record::builder()
            .level(log::Level::Warn)
            .target(XML_RECOVERY_LOG_TARGET)
            .args(format_args!(
                "recovered malformed xml (unclosed or mismatched elements)"
            ))
            .build();
        watch.log(&recovery);
        assert!(
            watch.warned.load(Ordering::SeqCst),
            "the actual xml well-formedness recovery signal must still trip the watch"
        );
    }

    /// F6, stated exactly as `text.rs`'s own test states it: two calls on
    /// equal bytes are equal.
    #[test]
    fn extraction_is_a_pure_function_of_its_input() {
        let bytes = fixture("01-plain-headings-paragraphs.docx");
        assert_eq!(docx(&bytes), docx(&bytes));
    }

    /// S6: the ONE routing table claims eleven of the twelve formats anydoc
    /// parses, by their canonical extension, case-insensitively.
    #[test]
    fn extractor_for_claims_eleven_formats_by_canonical_extension() {
        for (extension, extractor) in OFFICE_EXTENSIONS {
            assert_eq!(
                extractor_for(&format!("report.{extension}")),
                Some(*extractor),
                ".{extension} must route to {extractor}"
            );
            assert_eq!(
                extractor_for(&format!("REPORT.{}", extension.to_uppercase())),
                Some(*extractor),
                ".{extension} must route case-insensitively"
            );
        }
        assert_eq!(
            OFFICE_EXTENSIONS.len(),
            11,
            "eleven of anydoc's twelve formats route here; csv is the twelfth and \
             stays relational — see CSV_IS_NOT_A_DOCUMENT"
        );
    }

    /// The one format of the twelve that must NOT route here (A1 §6.4,
    /// A1-13, J5) — and not merely "not claimed by this table": not claimed
    /// by the WORKER routing table at all, which is the union this adapter
    /// actually contributes to. `tests/x4_tabular_map.rs` asserts the other
    /// half — that `.csv` IS claimed by the tabular table — so between the
    /// two, a future edit that moved CSV into the document lane fails
    /// somewhere rather than silently converting data into prose.
    #[test]
    fn csv_is_never_claimed_by_the_document_lane() {
        for path in ["tickets.csv", "TICKETS.CSV", "export/tickets.csv"] {
            assert_eq!(extractor_for(path), None, "{path}: {CSV_IS_NOT_A_DOCUMENT}");
            assert_eq!(
                super::super::scan::worker_extractor_for(path),
                None,
                "{path}: {CSV_IS_NOT_A_DOCUMENT}"
            );
        }
        assert!(
            !OFFICE_EXTENSIONS.iter().any(|(ext, _)| *ext == "csv"),
            "{CSV_IS_NOT_A_DOCUMENT}"
        );
    }

    /// Nothing outside the eleven, including anydoc's own wider alias set
    /// (`OFFICE_EXTENSIONS`' own doc: routing an alias would silently widen
    /// what this corpus covers) and formats belonging to other adapters.
    #[test]
    fn extractor_for_claims_nothing_outside_the_eleven() {
        for other in [
            "plain.txt",
            "notes.md",
            "message.eml",
            "bundle.zip",
            "rows.parquet",
            // anydoc parses these too, behind the same frontends, but they
            // are deliberately not routed (R1): no fixture covers them.
            "macro.docm",
            "macro.xlsm",
            "binary.xlsb",
            "legacy.xls",
            "macro.pptm",
            "show.ppsx",
            "show.ppsm",
            "slides.pps",
            "template.pot",
        ] {
            assert_eq!(extractor_for(other), None, "{other} must not be claimed");
        }
        assert_eq!(extractor_for("no-extension"), None);
    }

    /// [`OFFICE_EXTRACTOR_LIKE`] is exhaustive over the whole table by
    /// construction — the property `db.rs`'s document-family filter now
    /// depends on, so a twelfth routed format lands inside `--content
    /// document` on the day it is routed rather than the day someone
    /// remembers to widen a list in another module.
    #[test]
    fn every_routed_identity_matches_the_document_family_pattern() {
        let prefix = OFFICE_EXTRACTOR_LIKE
            .strip_suffix('%')
            .expect("the pattern is a prefix match");
        for (extension, extractor) in OFFICE_EXTENSIONS {
            assert!(
                extractor.starts_with(prefix),
                "{extractor} (.{extension}) falls outside {OFFICE_EXTRACTOR_LIKE}"
            );
        }
        // Distinct identities, or two formats would share an F7 cache key.
        let unique: std::collections::BTreeSet<_> =
            OFFICE_EXTENSIONS.iter().map(|(_, e)| *e).collect();
        assert_eq!(unique.len(), OFFICE_EXTENSIONS.len());
        assert!(!super::super::text::MARKDOWN_EXTRACTOR.starts_with(prefix));
        assert!(!super::super::mail::MAIL_EXTRACTOR.starts_with(prefix));
    }

    /// An extractor identity this build does not route is honestly
    /// [`OfficeError::Unsupported`] — never a panic, and never a silent
    /// fall-through to some default format (the shape a worker invoked from
    /// a newer or older daemon would hit).
    #[test]
    fn an_unrouted_extractor_identity_is_unsupported_not_a_panic() {
        let error = office_units(&fixture("01-plain-headings-paragraphs.docx"), "mystery/v1")
            .expect_err("an unknown extractor identity must be refused");
        assert!(matches!(error, OfficeError::Unsupported(_)), "{error:?}");
        assert!(error.to_string().contains("mystery/v1"), "{error}");
    }

    // ----------------------------------------------------- gate (b), fixture by fixture
    //
    // `manifest.json`'s counts are ground truth, hand-verified at the OOXML
    // level before this extractor existed (`MANIFEST.md`). What follows
    // checks every one of its fields this adapter's own vocabulary can
    // recover — see the module doc's "known, honest gap" section for the two
    // it cannot (`numId`/`ilvl`, header/footer parts).

    /// Fixture 01: two headings, three plain paragraphs, five paragraphs
    /// total — no lists, no tables, no notes.
    #[test]
    fn fixture_01_plain_headings_and_paragraphs() {
        let units = docx(&fixture("01-plain-headings-paragraphs.docx")).expect("parses");
        let doc = &units[0];
        assert_eq!(doc.kind, UnitKind::Document);
        assert_eq!(doc.coordinate, None);

        let sections: Vec<_> = units
            .iter()
            .filter(|u| u.kind == UnitKind::Section)
            .collect();
        assert_eq!(sections.len(), 2, "one section per heading, no preamble");
        assert_eq!(sections[0].heading_level, Some(1));
        assert_eq!(sections[0].title.as_deref(), Some("Introduction"));
        assert!(sections[0].text.contains("first body paragraph"));
        assert!(sections[0].text.contains("second body paragraph"));
        assert_eq!(sections[1].heading_level, Some(2));
        assert_eq!(sections[1].title.as_deref(), Some("Background"));
        assert!(sections[1].text.contains("A single body paragraph"));

        assert_eq!(
            count_body_paragraphs(&raw_document(&fixture("01-plain-headings-paragraphs.docx"))),
            5,
            "manifest body_top_level_paragraphs"
        );
    }

    /// Fixture 02: a two-level nested list under one heading — the manifest's
    /// raw `numId`/`ilvl` identity is the one thing this adapter's
    /// vocabulary cannot recover (module doc); item text, count, nesting
    /// depth, and marker kind all are.
    #[test]
    fn fixture_02_nested_list_preserves_marker_kind_and_nesting_depth() {
        let units = docx(&fixture("02-nested-list-numbering.docx")).expect("parses");
        let sections: Vec<_> = units
            .iter()
            .filter(|u| u.kind == UnitKind::Section)
            .collect();
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].title.as_deref(), Some("Shopping list"));
        for item in ["Produce", "Apples", "Bananas", "Dairy", "Milk"] {
            assert!(
                sections[0].text.contains(item),
                "{item:?} missing from {:?}",
                sections[0].text
            );
        }
        // The manifest's own numId/ilvl gap (module doc) is about the raw
        // OOXML identity, not the nesting/marker *shape* that identity
        // produced — the fixture's numbering.xml pins level 0 as decimal
        // and level 1 as lowerLetter (`build_docx_fixtures.py`), and per
        // OOXML's default level-restart behavior each new level-0 item
        // restarts its own level-1 counter, so "Apples"/"Bananas" are
        // `a.`/`b.` under "Produce" and "Milk" is `a.` again under "Dairy",
        // not `c.` This is the exact shape [`render_list`] must reproduce
        // as a textual proxy, not just the bag of item words checked above.
        assert_eq!(
            sections[0].text,
            "Shopping list\n1. Produce\n  a. Apples\n  b. Bananas\n2. Dairy\n  a. Milk",
            "nesting depth and marker kind must survive rendering, not just item text"
        );
        assert_eq!(
            count_body_paragraphs(&raw_document(&fixture("02-nested-list-numbering.docx"))),
            6,
            "manifest body_top_level_paragraphs (1 heading + 5 list items)"
        );
    }

    /// Fixture 03: a 3x2 table with paragraphs before and after it —
    /// verifies table shape and cell text directly against anydoc's own
    /// grid, and the paragraph/cell-paragraph split the manifest defines.
    #[test]
    fn fixture_03_table_shape_and_cell_text() {
        let bytes = fixture("03-table.docx");
        let units = docx(&bytes).expect("parses");
        let sections: Vec<_> = units
            .iter()
            .filter(|u| u.kind == UnitKind::Section)
            .collect();
        assert_eq!(sections.len(), 1);
        assert!(sections[0].text.contains("Widget"));
        assert!(sections[0].text.contains("Gadget"));
        assert!(sections[0].text.contains("table below lists current stock"));
        assert!(sections[0].text.contains("End of inventory report"));

        let document = raw_document(&bytes);
        assert_eq!(
            count_body_paragraphs(&document),
            3,
            "manifest body_top_level_paragraphs"
        );
        let tables = table_blocks(&document);
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].grid.len(), 3, "manifest tables[0].rows");
        assert_eq!(tables[0].grid[0].len(), 2, "manifest tables[0].cols");
        assert_eq!(
            count_table_cell_paragraphs(tables[0]),
            6,
            "manifest table_cell_paragraphs"
        );
        let expected = [["Name", "Quantity"], ["Widget", "12"], ["Gadget", "7"]];
        for (row, expected_row) in tables[0].grid.iter().zip(expected.iter()) {
            for (slot, expected_cell) in row.iter().zip(expected_row.iter()) {
                let anydoc::model::CellSlot::Origin(cell) = slot else {
                    panic!("this fixture's table has no spans");
                };
                let mut text = String::new();
                for block in &cell.blocks {
                    render_block(block, &mut text);
                }
                assert_eq!(text.trim(), *expected_cell, "manifest cell_text_row_major");
            }
        }
    }

    /// Fixture 04: two footnotes, referenced twice in body flow. Header/
    /// footer parts are the module doc's other named, honest gap.
    #[test]
    fn fixture_04_footnotes_are_recovered_body_and_content() {
        let bytes = fixture("04-footnotes-headers-footers.docx");
        let units = docx(&bytes).expect("parses");
        let sections: Vec<_> = units
            .iter()
            .filter(|u| u.kind == UnitKind::Section)
            .collect();
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].title.as_deref(), Some("Findings"));

        let document = raw_document(&bytes);
        assert_eq!(
            count_body_paragraphs(&document),
            3,
            "manifest body_top_level_paragraphs"
        );
        assert_eq!(
            document.notes.len(),
            2,
            "manifest footnote_content_entries (structural placeholders excluded by anydoc itself)"
        );
        let note_texts: Vec<String> = document
            .notes
            .iter()
            .map(|note| render_blocks(&note.blocks))
            .collect();
        assert!(note_texts.iter().any(|t| t.contains("First footnote text")));
        assert!(
            note_texts
                .iter()
                .any(|t| t.contains("Second footnote text"))
        );
        assert!(
            document
                .notes
                .iter()
                .all(|n| n.kind == anydoc::model::NoteKind::Footnote)
        );
        assert_eq!(
            count_note_refs(&document.blocks),
            2,
            "manifest footnote_references_in_body"
        );
    }

    /// **Gate (b)'s own pass criterion, fixture 05**: a document whose
    /// `word/document.xml` is not well-formed XML must fail this adapter
    /// outright — zero output, never a partial or silently-repaired one.
    /// Without the recovery watch this would pass with one merged paragraph
    /// (verified against this exact fixture during development); this is the
    /// regression pin for that.
    #[test]
    fn fixture_05_malformed_is_refused_not_repaired() {
        let err = docx(&fixture("05-malformed-unclosed-element.docx"))
            .expect_err("a malformed document must be refused, never silently repaired");
        assert!(
            matches!(err, OfficeError::Malformed(_)),
            "must be classified Malformed: {err:?}"
        );
    }

    /// The hostile case (brief item 4's "consider also…"): a `.docx` whose
    /// `word/document.xml` entry decompresses past anydoc's own
    /// `max_entry_bytes` ceiling (128 MiB), reached through a tiny on-disk
    /// file via a high compression ratio — a genuine, real-parser stress on
    /// the memory axis, refused by anydoc's *own* internal limit (comfortably
    /// below the worker's outer 512 MiB `RLIMIT_AS`, so this is deterministic
    /// and never races the worker-level cap or its deadline).
    #[test]
    fn hostile_entry_expansion_trips_anydocs_own_resource_limit() {
        let bytes = fixture("06-hostile-entry-expansion.docx");
        let err = docx(&bytes).expect_err("an oversized entry must be refused");
        assert!(
            matches!(err, OfficeError::ResourceLimit(_)),
            "must be classified ResourceLimit: {err:?}"
        );
    }

    // --------------------------------------------------- shared verification helpers
    //
    // These call anydoc directly (allowed here — this is the adapter's own
    // module) to compute the SAME counts `manifest.json` defines, over
    // anydoc's block tree, independently of `office_units`'s own Document/
    // Section flattening — so a bug in `flatten`/`render_blocks` cannot hide
    // behind a bug in these helpers agreeing with it.

    // ------------------------------------------- S6: the ten newly routed formats
    //
    // `MANIFEST.md`'s "Office fixture corpus (S6)" section records the
    // hand-known expected extraction of every fixture below, produced the
    // same way the docx corpus's was: `build_office_fixtures.py` writes each
    // fixture's one text-bearing part as a literal string, so the author
    // knew the answer before any parser ran. Each test asserts the WHOLE
    // rendered text of the document unit, not a substring — a reviewer can
    // `unzip -p <fixture> content.xml` (or `cat` the RTF/PDF) and check the
    // assertion line by line, which is what F5 gate 2's "hand-verified"
    // means and what "it compiled" is not.

    fn office_fixture(name: &str) -> Vec<u8> {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/anydoc_corpus/office_fixtures")
            .join(name);
        std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    /// Run a fixture through the routing table by its own filename, so each
    /// test proves the EXTENSION reaches the right frontend, not just that
    /// some hardcoded identity parses some bytes.
    fn units_of(name: &str) -> Result<Vec<OfficeUnit>, OfficeError> {
        let extractor = extractor_for(name).unwrap_or_else(|| panic!("{name} is not routed"));
        office_units(&office_fixture(name), extractor)
    }

    fn sections(units: &[OfficeUnit]) -> Vec<&OfficeUnit> {
        units
            .iter()
            .filter(|u| u.kind == UnitKind::Section)
            .collect()
    }

    /// `.rtf` — a plain-text format, so the fixture IS its own source
    /// listing: two `\pard ... \par` paragraphs and nothing else. No
    /// headings, therefore one whole-document unit and no sections, exactly
    /// as `text.rs` treats a heading-less Markdown file.
    #[test]
    fn rtf_fixture_yields_both_paragraphs_and_no_sections() {
        let units = units_of("07-rtf-plain.rtf").expect("parses");
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].kind, UnitKind::Document);
        assert_eq!(units[0].coordinate, None);
        assert_eq!(units[0].text, "First rtf paragraph.\nSecond rtf paragraph.");
    }

    /// `.doc` — the one legacy-Word path this corpus can hand-author, and a
    /// case anydoc's own dispatcher names ("RTF files wearing a .doc
    /// extension are common in the wild"): byte-identical to the `.rtf`
    /// fixture, routed by extension to `DOC_EXTRACTOR`, and yielding the
    /// identical extraction. The BINARY Word 97 path behind the same
    /// extension has no fixture — see MANIFEST.md's named corpus gap.
    #[test]
    fn doc_fixture_is_rtf_in_disguise_and_extracts_identically() {
        assert_eq!(
            extractor_for("08-doc-rtf-in-disguise.doc"),
            Some(DOC_EXTRACTOR)
        );
        assert_eq!(
            office_fixture("08-doc-rtf-in-disguise.doc"),
            office_fixture("07-rtf-plain.rtf"),
            "the fixture pair is deliberately byte-identical: only the extension differs"
        );
        let units = units_of("08-doc-rtf-in-disguise.doc").expect("parses");
        assert_eq!(units, units_of("07-rtf-plain.rtf").expect("parses"));
    }

    /// Adversarial, `.rtf`: 400 nested groups before any text.
    ///
    /// **A recorded finding, not a refusal.** anydoc's fixed limits
    /// (`package::limits`) bound archives, XML depth/nodes, spreadsheet grid
    /// expansion and binary record depth — there is NO RTF-specific nesting
    /// or size cap, and this fixture parses clean. What that means is
    /// stated rather than papered over: for RTF (and `.doc` bytes that are
    /// RTF), the bound is not anydoc's, it is the supervised worker's own —
    /// `scan::MAX_RESOURCE_BYTES` daemon-side and
    /// `worker::WORKER_ADDRESS_SPACE_LIMIT_BYTES` (RLIMIT_AS) around the
    /// process. The property this fixture DOES prove is the one that
    /// matters at this layer: deep nesting is handled iteratively, so it
    /// neither overflows the stack nor hangs.
    #[test]
    fn deeply_nested_rtf_does_not_overflow_the_parser_and_has_no_anydoc_limit() {
        let units = units_of("09-rtf-deep-nesting.rtf")
            .expect("anydoc applies no RTF-specific depth limit — see this test's doc");
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].text, "deep");
    }

    /// `.odt` — two ODF headings (`text:h` with `text:outline-level`) and a
    /// paragraph under each.
    #[test]
    fn odt_fixture_yields_two_heading_sections() {
        let units = units_of("10-odt-headings.odt").expect("parses");
        assert_eq!(
            units[0].text,
            "Odt Introduction\nFirst odt paragraph.\nOdt Background\nSecond odt paragraph."
        );
        assert_eq!(units[0].title.as_deref(), Some("Odt Introduction"));
        let sections = sections(&units);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].heading_level, Some(1));
        assert_eq!(sections[0].title.as_deref(), Some("Odt Introduction"));
        assert_eq!(sections[0].text, "Odt Introduction\nFirst odt paragraph.");
        assert_eq!(sections[1].heading_level, Some(2));
        assert_eq!(sections[1].text, "Odt Background\nSecond odt paragraph.");
    }

    /// `.odp` — two `draw:page` slides, each a title frame plus one outline
    /// frame. anydoc renders a slide title as a level-2 heading, which is
    /// what makes each slide its own addressable section.
    #[test]
    fn odp_fixture_yields_one_section_per_slide() {
        let units = units_of("12-odp-slides.odp").expect("parses");
        assert_eq!(
            units[0].text,
            "Odp Slide One\nOdp first bullet.\nOdp Slide Two\nOdp second bullet."
        );
        let sections = sections(&units);
        assert_eq!(sections.len(), 2, "one section per slide");
        assert_eq!(sections[0].heading_level, Some(2));
        assert_eq!(sections[0].text, "Odp Slide One\nOdp first bullet.");
        assert_eq!(sections[1].text, "Odp Slide Two\nOdp second bullet.");
    }

    /// `.pptx` — the OOXML twin of the `.odp` fixture: two slides reached
    /// through the real OPC relationship chain (`_rels/.rels` →
    /// `ppt/presentation.xml` → `sldIdLst` → `ppt/slides/slideN.xml`), so
    /// this proves package traversal, not just XML text extraction.
    #[test]
    fn pptx_fixture_yields_one_section_per_slide() {
        let units = units_of("15-pptx-slides.pptx").expect("parses");
        assert_eq!(
            units[0].text,
            "Pptx Slide One\nPptx first bullet.\nPptx Slide Two\nPptx second bullet."
        );
        let sections = sections(&units);
        assert_eq!(sections.len(), 2, "one section per slide");
        assert_eq!(sections[0].title.as_deref(), Some("Pptx Slide One"));
        assert_eq!(sections[1].text, "Pptx Slide Two\nPptx second bullet.");
    }

    /// `.epub` — the OPF title becomes the book's own heading, then each
    /// spine chapter's `<h1>` becomes a section, in spine order.
    #[test]
    fn epub_fixture_yields_the_title_and_one_section_per_spine_chapter() {
        let units = units_of("20-epub-chapters.epub").expect("parses");
        assert_eq!(
            units[0].text,
            "Epub Fixture Book\nEpub Chapter One\nFirst epub paragraph.\n\
             Epub Chapter Two\nSecond epub paragraph."
        );
        assert_eq!(units[0].title.as_deref(), Some("Epub Fixture Book"));
        let sections = sections(&units);
        assert_eq!(sections.len(), 3, "the OPF title, then two spine chapters");
        assert_eq!(sections[0].text, "Epub Fixture Book");
        assert_eq!(sections[1].text, "Epub Chapter One\nFirst epub paragraph.");
        assert_eq!(sections[2].text, "Epub Chapter Two\nSecond epub paragraph.");
    }

    // ------------------------------------------------------ the spreadsheet pair
    //
    // A1 §6.3, OWNER-02, decides which lane owns these and this pair is what
    // exercises it: "For Office spreadsheets, readable Markdown/table
    // normalization is sufficient for Sprint 1 knowledge/search, but it is
    // **not** evidence that exact workbook sheet/cell write-back coordinates
    // were preserved." Both halves are asserted below — the table text IS
    // produced, and NO unit claims a cell.

    /// `.xlsx` — a three-row, two-column worksheet reached through the OPC
    /// chain (`_rels/.rels` → `xl/workbook.xml` → `worksheets/sheet1.xml`),
    /// rendered as table text.
    #[test]
    fn xlsx_fixture_yields_table_text_and_claims_no_cell_coordinate() {
        let units = units_of("18-xlsx-sheet.xlsx").expect("parses");
        assert_eq!(units[0].text, "Item | Cost\nWidget | 10\nGadget | 20");
        assert!(
            units.iter().all(|u| u.coordinate.is_none()
                || u.coordinate
                    .as_deref()
                    .is_some_and(|c| c.starts_with("block:"))),
            "no unit may carry a `Sheet1!A1`-shaped coordinate — A1 §6.3's own \
             write-back caveat, and this module's own design rule: {units:?}"
        );
    }

    /// `.ods` — the OpenDocument half of the same decision, over the same
    /// three rows. The two extractions being IDENTICAL is the point: one
    /// workbook saved in either container reaches retrieval the same way,
    /// which is what makes "keep the document lane" a coherent answer for
    /// both rather than a per-format accident.
    #[test]
    fn ods_fixture_matches_the_xlsx_extraction_exactly() {
        let ods = units_of("11-ods-sheet.ods").expect("parses");
        let xlsx = units_of("18-xlsx-sheet.xlsx").expect("parses");
        assert_eq!(ods[0].text, "Item | Cost\nWidget | 10\nGadget | 20");
        assert_eq!(ods, xlsx);
    }

    // ---------------------------------------------------------------- the PDF split
    //
    // "pdf splits on content" (owner ruling, decided item 3). Both halves
    // are fixtures, because only one of them is a parse.

    /// A text-bearing PDF extracts natively — through
    /// `to_markdown_bytes`/`markdown_to_office_units`, since anydoc has no
    /// document model for PDFs at all. The coordinate spelling differs from
    /// every other format here (`md-offset:`, not `block:`), which is
    /// exactly what the module doc means by "the spelling is not the
    /// contract".
    #[test]
    fn text_bearing_pdf_extracts_natively_with_a_markdown_coordinate() {
        let units = units_of("22-pdf-text.pdf").expect("a text-bearing PDF parses");
        assert_eq!(units[0].kind, UnitKind::Document);
        assert_eq!(units[0].coordinate, None);
        assert_eq!(
            units[0].text,
            "# Pdf Fixture Heading\n\nFirst pdf paragraph. Second pdf paragraph. \
             Third pdf paragraph."
        );
        let sections = sections(&units);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].heading_level, Some(1));
        assert_eq!(sections[0].title.as_deref(), Some("Pdf Fixture Heading"));
        assert_eq!(sections[0].coordinate.as_deref(), Some("md-offset:0"));
    }

    /// A scanned/image-only PDF is a NAMED coverage gap pointing at the OCR
    /// epic — **never** silence, and never a false empty extraction that
    /// would read downstream as a document containing no text (A1 §15:
    /// "Missing capability is never represented as successful empty
    /// evidence"). OCR is the one thing deliberately outside 0.3.0.
    #[test]
    fn scanned_pdf_is_a_named_ocr_coverage_gap_not_an_empty_extraction() {
        let error = units_of("23-pdf-scanned-needs-ocr.pdf")
            .expect_err("an image-only PDF must not extract as an empty document");
        assert!(matches!(error, OfficeError::NeedsOcr(_)), "{error:?}");
        let text = error.to_string();
        assert!(text.contains("OCR"), "{text}");
        assert!(
            text.contains("page 1 of 1"),
            "the gap must name WHICH pages need OCR, not merely that some do: {text}"
        );
    }

    // ------------------------------------------------------- adversarial fixtures
    //
    // The `.docx` corpus's own pattern (05-malformed-unclosed-element,
    // 06-hostile-entry-expansion), repeated across the container families:
    // every Office file here is a ZIP, so W7's container bounds, the shared
    // depth counter and the whole-tree byte budget already apply, and these
    // show them applying.

    /// Malformed XML inside an otherwise well-formed package is REFUSED, not
    /// repaired — one fixture per container family, so the stricter policy
    /// (`office_units`' own recovery watch) is shown to hold for the ODF,
    /// PresentationML, SpreadsheetML and EPUB frontends and not only the
    /// WordprocessingML one it was written against.
    #[test]
    fn a_malformed_part_is_refused_in_every_container_family() {
        for name in [
            "13-odt-malformed-unclosed-element.odt",
            "16-pptx-malformed-unclosed-element.pptx",
            "19-xlsx-malformed-unclosed-element.xlsx",
            "21-epub-malformed-unclosed-element.epub",
        ] {
            let error = match units_of(name) {
                Err(error) => error,
                Ok(units) => panic!("{name} must be refused, not repaired; got {units:?}"),
            };
            assert!(
                matches!(error, OfficeError::Malformed(_)),
                "{name}: {error:?}"
            );
        }
    }

    /// A password-protected document is a NAMED gap with its own detail
    /// text, distinct from a parse failure — anydoc's API distinguishes
    /// `Encrypted` from every other error and so must the coverage row an
    /// operator reads, because "locked" and "damaged" are different
    /// problems with different remedies.
    #[test]
    fn an_encrypted_document_is_its_own_named_gap_not_a_parse_failure() {
        let error =
            units_of("14-odt-encrypted.odt").expect_err("an encrypted package must not parse");
        assert!(matches!(error, OfficeError::Encrypted), "{error:?}");
        assert_eq!(
            error.to_string(),
            "document is encrypted or password-protected"
        );
        // Distinct detail text, not merely a distinct variant: the malformed
        // fixture's row must not read the same way.
        let malformed = units_of("13-odt-malformed-unclosed-element.odt")
            .expect_err("the malformed fixture must also fail");
        assert_ne!(error.to_string(), malformed.to_string());
    }

    /// An oversized entry in a NON-docx OOXML package trips anydoc's own
    /// `max_entry_bytes` on the real parser path, one layer inside the
    /// supervised worker's RLIMIT_AS — the same claim
    /// `06-hostile-entry-expansion.docx` makes for WordprocessingML, shown
    /// here for PresentationML.
    ///
    /// The entry is `ppt/slides/slide1.xml`, a part the frontend actually
    /// reads. An earlier draft of this fixture parked the bomb in
    /// `ppt/media/`, which the frontend never opens; it parsed clean and
    /// proved nothing. Recorded here because "the fixture is hostile" is a
    /// claim about what the PARSER touches, not about what the zip contains.
    #[test]
    fn a_hostile_entry_in_a_pptx_trips_anydocs_own_resource_limit() {
        let error = units_of("17-pptx-hostile-entry-expansion.pptx")
            .expect_err("an oversized entry must be refused");
        assert!(matches!(error, OfficeError::ResourceLimit(_)), "{error:?}");
        assert!(
            error.to_string().contains("ppt/slides/slide1.xml"),
            "the refusal must name the entry: {error}"
        );
    }

    /// F6 for every newly routed format, not only `.docx`: two calls on
    /// equal bytes are equal — the purity the F7 cache key depends on.
    #[test]
    fn extraction_is_pure_for_every_routed_format() {
        for name in [
            "07-rtf-plain.rtf",
            "08-doc-rtf-in-disguise.doc",
            "10-odt-headings.odt",
            "11-ods-sheet.ods",
            "12-odp-slides.odp",
            "15-pptx-slides.pptx",
            "18-xlsx-sheet.xlsx",
            "20-epub-chapters.epub",
            "22-pdf-text.pdf",
        ] {
            assert_eq!(units_of(name), units_of(name), "{name} is not pure");
        }
    }

    /// The module doc's coordinate contract, over every routed format that
    /// produces sections: present on every non-document unit, absent on the
    /// document unit, and unique within one parse.
    #[test]
    fn coordinates_hold_their_contract_for_every_routed_format() {
        for name in [
            "10-odt-headings.odt",
            "12-odp-slides.odp",
            "15-pptx-slides.pptx",
            "20-epub-chapters.epub",
            "22-pdf-text.pdf",
        ] {
            let units = units_of(name).expect("parses");
            assert_eq!(units[0].coordinate, None, "{name}: document unit");
            let mut seen = std::collections::BTreeSet::new();
            for unit in units.iter().filter(|u| u.kind == UnitKind::Section) {
                let coordinate = unit
                    .coordinate
                    .as_deref()
                    .unwrap_or_else(|| panic!("{name}: a section with no coordinate"));
                assert!(seen.insert(coordinate), "{name}: duplicate {coordinate}");
            }
            assert!(!seen.is_empty(), "{name} produced no sections");
        }
    }

    fn raw_document(bytes: &[u8]) -> anydoc::model::Document {
        anydoc::to_document(bytes, anydoc::Format::Docx).expect("valid fixture parses")
    }

    /// `manifest.json`'s `body_top_level_paragraphs`: headings and plain
    /// paragraphs count 1 each; a list's own paragraph-equivalents are its
    /// items, recursed (an item's own text is itself a `Block::Paragraph` in
    /// `item.blocks`, so this never double-counts); tables are excluded
    /// (their cell paragraphs are a separate count, below).
    fn count_body_paragraphs(document: &anydoc::model::Document) -> usize {
        count_paragraph_equivalents(&document.blocks)
    }

    fn count_paragraph_equivalents(blocks: &[anydoc::model::Block]) -> usize {
        use anydoc::model::Block;
        blocks
            .iter()
            .map(|block| match block {
                Block::Heading { .. } | Block::Paragraph(_) => 1,
                Block::List(list) => list
                    .items
                    .iter()
                    .map(|item| count_paragraph_equivalents(&item.blocks))
                    .sum(),
                Block::BlockQuote(inner) => count_paragraph_equivalents(inner),
                Block::Table(_) => 0,
                // Not exercised by this corpus; a preformatted block, rule or
                // formula is still one body-flow paragraph-equivalent.
                Block::CodeBlock { .. } | Block::Rule | Block::Math(_) => 1,
            })
            .sum()
    }

    fn table_blocks(document: &anydoc::model::Document) -> Vec<&anydoc::model::Table> {
        document
            .blocks
            .iter()
            .filter_map(|b| match b {
                anydoc::model::Block::Table(table) => Some(table),
                _ => None,
            })
            .collect()
    }

    /// `manifest.json`'s `table_cell_paragraphs`: every `<w:p>` nested inside
    /// any `<w:tc>`, i.e. every origin cell's own paragraph-equivalent count.
    fn count_table_cell_paragraphs(table: &anydoc::model::Table) -> usize {
        table
            .grid
            .iter()
            .flatten()
            .map(|slot| match slot {
                anydoc::model::CellSlot::Origin(cell) => count_paragraph_equivalents(&cell.blocks),
                anydoc::model::CellSlot::Covered { .. } => 0,
            })
            .sum()
    }

    /// `manifest.json`'s `footnote_references_in_body`: every
    /// [`anydoc::model::Inline::NoteRef`] in the body flow's own inline runs.
    fn count_note_refs(blocks: &[anydoc::model::Block]) -> usize {
        use anydoc::model::{Block, Inline};
        fn count_inlines(inlines: &[Inline]) -> usize {
            inlines
                .iter()
                .map(|inline| match inline {
                    Inline::NoteRef(_) => 1,
                    Inline::Link { content, .. } => count_inlines(content),
                    _ => 0,
                })
                .sum()
        }
        blocks
            .iter()
            .map(|block| match block {
                Block::Heading { content, .. } => count_inlines(content),
                Block::Paragraph(inlines) => count_inlines(inlines),
                Block::List(list) => list
                    .items
                    .iter()
                    .map(|item| count_note_refs(&item.blocks))
                    .sum(),
                Block::BlockQuote(inner) => count_note_refs(inner),
                Block::Table(table) => table
                    .grid
                    .iter()
                    .flatten()
                    .map(|slot| match slot {
                        anydoc::model::CellSlot::Origin(cell) => count_note_refs(&cell.blocks),
                        anydoc::model::CellSlot::Covered { .. } => 0,
                    })
                    .sum(),
                Block::CodeBlock { .. } | Block::Rule | Block::Math(_) => 0,
            })
            .sum()
    }
}
