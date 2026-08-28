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
//!   boundary. [`docx_units`] returns [`OfficeUnit`]/[`OfficeError`] — both
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
//! an unchanged [`docx_units`] signature, and the structural pin is what
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
//! carries a **structural** coordinate instead — `block:<index>`, the
//! section's starting position in anydoc's own top-level block sequence —
//! recoverable and stable across runs of the same extractor version, but
//! explicitly not a byte offset. [`super::worker::WorkerUnit::coordinate`]
//! is the wire field this rides on; the Document unit needs no coordinate
//! more specific than "the whole resource" (`None`), exactly as text.rs's
//! own Document unit carries no `heading_level`/`title` beyond what its first
//! heading lends it.
//!
//! Output is derived, never canonical (A1-12): re-running this extractor
//! against the same bytes and the same anydoc version reproduces the same
//! units, but the units are not the document — the `.docx` file is, and
//! [`docx_units`]' caller is expected to keep citing the *original resource*
//! (its path, its content hash), never a temp file this adapter might read
//! bytes through.
//!
//! # Spreadsheet formats claim no write-back coordinates
//!
//! Not exercised this wave — the corpus and the extension routing below are
//! `.docx`-only, per G3's "docx first" gate order and the sprint plan's "NOT
//! in scope: … a second Office format beyond the one the gate adopts." Named
//! here because the wave brief requires it stated as a design rule, ahead of
//! any code exercising it: **should a spreadsheet format ever route through
//! this contract, its [`OfficeUnit`]s must never claim a coordinate a caller
//! could use to write back to a specific cell.** `MarkerKind`/grid positions
//! anydoc resolves for a spreadsheet are read-only derived evidence — a
//! `block:<index>` coordinate (or no coordinate) is honest; a `Sheet1!A1`-
//! shaped coordinate would assert a two-way binding this adapter does not
//! have and never will, because normalized text has no cell back-reference
//! anydoc's own model preserves.
//!
//! # A known, honest gap: what anydoc's docx frontend does not expose
//!
//! `tests/fixtures/anydoc_corpus/manifest.json`'s hand-verified counts are
//! pinned at the raw OOXML level *specifically* so they do not depend on any
//! extractor's own vocabulary (`MANIFEST.md`'s own words). Two of its fields
//! are, empirically, **not recoverable through anydoc's normalized model**,
//! and this module says so rather than fabricating an answer:
//!
//! * `numId`/`ilvl` per list paragraph — anydoc resolves numbering into an
//!   actual nested [`List`]/[`ListItem`] tree (marker kind, start, nesting by
//!   containment) rather than preserving the OOXML numbering identity that
//!   produced it. That is the abstraction working as designed — "any doc"
//!   means normalizing across formats that do not all *have* an `numId` — and
//!   asserting it back here would put an OOXML-specific concept in our own
//!   vocabulary, which the boundary above forbids anyway.
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

/// Extractor identity + version (F7's second cache-key input, and the
/// provenance "normalizer identity" A1 §6.3 requires) — versioned so a future
/// anydoc bump, or a rewrite behind this same contract, changes the identity
/// and therefore the derived key, exactly as [`super::text::MARKDOWN_EXTRACTOR`]'s
/// own doc explains for its own version tag.
pub const DOCX_EXTRACTOR: &str = "anydoc/0.2.4+docx/v1";

/// Extensions routed to [`docx_units`]. `.docx` only — `.docm` shares
/// anydoc's own parser but is not part of this wave's adopted corpus or
/// footprint measurement, and routing it here would silently widen what G3
/// actually gated.
pub const DOCX_EXTENSIONS: &[&str] = &["docx"];

/// The extractor for a path, by extension, or `None` for anything this
/// adapter does not claim — mirrors [`super::text::extractor_for`]'s own
/// shape (extension-driven, never content-sniffed, for the same reason: an
/// unclaimed extension is honestly `unsupported`, not guessed at).
///
/// Nothing calls this in production yet: wiring the local-knowledge walk to
/// dispatch a claimed path to the supervised worker is daemon-scheduling
/// work Y1 did not ship either (`lane::run_worker_on_lane` has no production
/// caller as of this wave) and is not this wave's own deliverable list — the
/// adapter, the worker route, and the real-parser supervision proof are.
/// Named now so a later wave's routing table has one place to look, exactly
/// as [`crate::domain::source::SourceKind::ExternalGit`]'s own doc names a
/// seam ahead of the wave that fills it.
pub fn extractor_for(relative: &str) -> Option<&'static str> {
    let extension = std::path::Path::new(relative)
        .extension()
        .and_then(|e| e.to_str())?
        .to_ascii_lowercase();
    DOCX_EXTENSIONS
        .contains(&extension.as_str())
        .then_some(DOCX_EXTRACTOR)
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
    /// A structural coordinate into the normalized document (`block:<n>`),
    /// or `None` for the whole-document unit, which needs nothing more
    /// specific. Never a byte offset — see the module doc.
    pub coordinate: Option<String>,
    /// The unit's own rendered text.
    pub text: String,
}

/// Why [`docx_units`] could not produce units — in our own vocabulary. Every
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
    /// The document is encrypted or password-protected.
    #[error("document is encrypted")]
    Encrypted,
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
pub fn docx_units(bytes: &[u8]) -> Result<Vec<OfficeUnit>, OfficeError> {
    install_recovery_watch();
    // `RECOVERY_WATCH` is one process-global flag (module doc): correct for
    // the worker binary, which runs exactly one `docx_units` call per
    // process, and would otherwise race under this crate's own multi-
    // threaded test runner, where several unit tests can call `docx_units`
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

    let document =
        anydoc::to_document(bytes, anydoc::Format::Docx).map_err(classify_convert_error)?;

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
    // so this counts *whether anything logged at WARN or above fired at
    // all* during this call — never matches on wording — which is exactly
    // the signal anydoc's own doc names as the sanctioned one ("Recovery and
    // skipped-content events are reported through the log facade").
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

fn classify_convert_error(error: anydoc::ConvertError) -> OfficeError {
    match error {
        anydoc::ConvertError::ResourceLimit { .. } => OfficeError::ResourceLimit(error.to_string()),
        anydoc::ConvertError::NeedsOcr { .. } => OfficeError::NeedsOcr(error.to_string()),
        anydoc::ConvertError::Encrypted => OfficeError::Encrypted,
        // `ConvertError` is `#[non_exhaustive]`: every variant this build
        // knows about is named above, and anything a future anydoc version
        // adds falls in here — honestly `Malformed` (a document this build
        // cannot make sense of), never a silent panic on an unmatched arm.
        anydoc::ConvertError::Unsupported(_)
        | anydoc::ConvertError::Malformed { .. }
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

fn render_list(list: &anydoc::model::List, out: &mut String) {
    for item in &list.items {
        for child in &item.blocks {
            render_block(child, out);
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

// ------------------------------------------------------- the recovery watch

/// A process-global `log::Log` whose only job is to notice whether *anything*
/// logged at [`log::Level::Warn`] or above during one [`docx_units`] call —
/// see that function's own doc for why. Message text is never inspected
/// (anydoc's own doc: log wording is not a stable API), only whether an
/// event at this severity fired.
struct RecoveryWatch {
    warned: AtomicBool,
}

impl log::Log for RecoveryWatch {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Warn
    }

    fn log(&self, record: &log::Record<'_>) {
        if record.level() <= log::Level::Warn {
            self.warned.store(true, Ordering::SeqCst);
        }
    }

    fn flush(&self) {}
}

static RECOVERY_WATCH: RecoveryWatch = RecoveryWatch {
    warned: AtomicBool::new(false),
};

/// Install [`RECOVERY_WATCH`] as the process's global logger, once.
///
/// Safe to call from every caller of [`docx_units`] in this process,
/// including repeatedly across calls (idempotent via [`Once`]) and including
/// concurrently (`log::set_logger` itself is the synchronization point).
/// Never called from the `sgt` daemon binary — only [`super::super::atlas`]'s
/// worker binary (`src/bin/atlas_worker.rs`) links this path, and that binary
/// installs no other logger (`tracing_subscriber::fmt().init()` is `sgt`'s
/// own `cli.rs`, a different process), so there is nothing here to conflict
/// with.
fn install_recovery_watch() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        // A second `set_logger` call after this one (there is none in this
        // binary, but `set_logger` itself is fallible for exactly that
        // reason) would return `Err` — ignored deliberately: the watch is
        // already installed by the first caller, which is all any caller in
        // this process needs.
        let _ = log::set_logger(&RECOVERY_WATCH);
        log::set_max_level(log::LevelFilter::Warn);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> Vec<u8> {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/anydoc_corpus/docx_fixtures")
            .join(name);
        std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    /// F6, stated exactly as `text.rs`'s own test states it: two calls on
    /// equal bytes are equal.
    #[test]
    fn extraction_is_a_pure_function_of_its_input() {
        let bytes = fixture("01-plain-headings-paragraphs.docx");
        assert_eq!(docx_units(&bytes), docx_units(&bytes));
    }

    #[test]
    fn extractor_for_claims_only_docx() {
        assert_eq!(extractor_for("report.docx"), Some(DOCX_EXTRACTOR));
        assert_eq!(extractor_for("REPORT.DOCX"), Some(DOCX_EXTRACTOR));
        // The second-format boundary (G3, NOT in scope): nothing else routes
        // here yet, even formats anydoc itself can parse.
        for other in [
            "book.xlsx",
            "deck.pptx",
            "notes.odt",
            "report.pdf",
            "plain.txt",
        ] {
            assert_eq!(
                extractor_for(other),
                None,
                "{other} must not be claimed yet"
            );
        }
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
        let units = docx_units(&fixture("01-plain-headings-paragraphs.docx")).expect("parses");
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
    /// `numId`/`ilvl` fields are the one thing this adapter's vocabulary
    /// cannot recover (module doc); item text and count are.
    #[test]
    fn fixture_02_nested_list_items_are_flattened_in_order() {
        let units = docx_units(&fixture("02-nested-list-numbering.docx")).expect("parses");
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
        let units = docx_units(&bytes).expect("parses");
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
        let units = docx_units(&bytes).expect("parses");
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
        let err = docx_units(&fixture("05-malformed-unclosed-element.docx"))
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
        let err = docx_units(&bytes).expect_err("an oversized entry must be refused");
        assert!(
            matches!(err, OfficeError::ResourceLimit(_)),
            "must be classified ResourceLimit: {err:?}"
        );
    }

    // --------------------------------------------------- shared verification helpers
    //
    // These call anydoc directly (allowed here — this is the adapter's own
    // module) to compute the SAME counts `manifest.json` defines, over
    // anydoc's block tree, independently of `docx_units`'s own Document/
    // Section flattening — so a bug in `flatten`/`render_blocks` cannot hide
    // behind a bug in these helpers agreeing with it.

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
