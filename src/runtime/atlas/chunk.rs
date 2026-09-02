//! Code/config chunking — **pure functions over bytes** (F6's adapter-shape
//! mandate), ported from semble's `chunking/` package: the exact algorithm a
//! sibling embedding pipeline already uses to turn one file into several
//! retrieval-sized units, applied here to the code/config family this
//! build's own extraction has no answer for yet (`super::text::plain_units`
//! puts a whole file — however large — into one [`UnitKind::Document`] unit;
//! see `knowledge/evidence/resources/host-atlas-s6-series/brief-chunker-port.md`
//! for the diagnosed gap this module exists to close).
//!
//! # Scope: the algorithm only, not the wiring (Captain-stated, this wave)
//!
//! [`chunk_source`] is `(text, language) -> Vec<Chunk>` and nothing calls it
//! yet. No change lands in [`super::scan`]'s routing or [`super::db`]'s
//! `source.units` table this wave — that split is the wave brief's own
//! stated boundary, ratified J5/J4 in this wave's `00-orient` stage, and a
//! later reviewer of this diff must not read the absence of a caller as an
//! oversight.
//!
//! # The oracle is semble's real chunker, not a re-derived spec
//!
//! Every constant and every algorithm step here is ported from a specific
//! file:line in the installed `semble` package (cited on each item below),
//! and every fixture's expected output in `tests/fixtures/chunk_corpus/` is
//! semble's *actual* chunker's output on that exact fixture, captured by
//! `tests/fixtures/chunk_corpus/build_chunk_goldens.py` — never a hand-typed
//! expectation. Drift from the reference is measurable by re-running that
//! script against a newer `semble` install and diffing the goldens, forever.
//!
//! # Byte offsets throughout, not semble's char offsets (a stated, tested divergence)
//!
//! semble's Python chunker works in **char** offsets end-to-end for its
//! line-fallback path (`chunking/core.py::chunk_lines`, native `str`
//! indexing) but in **byte** offsets for its AST-merge path
//! (`chunking/core.py::chunk`, `tree_sitter::Node::start_byte`/`end_byte`),
//! converting the latter back to char offsets only at its own return
//! boundary (`chunking/core.py:150-152`) because its caller
//! (`chunking/chunking.py::chunk_source`) slices a Python `str`.
//!
//! This port works in **byte offsets throughout, on both paths** — no
//! conversion step exists, because Rust `&str` slicing is already
//! byte-indexed and [`super::text::StructureUnit`]/[`super::syntax::Symbol`]
//! already promise `byte_start`/`byte_end` provenance (A1-12), not char
//! offsets, so byte-native is the sibling-consistent choice (R2), not merely
//! the convenient one. The two are observably identical for ASCII input
//! (every char is one byte), which is why every fixture in
//! `tests/fixtures/chunk_corpus/` is ASCII-only and
//! `build_chunk_goldens.py` asserts that ASCII property itself — a
//! non-ASCII fixture would need its own documented tolerance and none is
//! claimed here because none is needed yet.
//!
//! # Line-fallback line splitting: `\n` only, a stated tolerance
//!
//! `chunk_lines`'s Python original splits on every line boundary
//! `str.splitlines` recognizes (`\r\n`, lone `\r`, `\v`, `\f`, and several
//! Unicode line/paragraph separators). This port splits on `\n` only
//! (`str::split_inclusive('\n')`) — the one line-boundary convention every
//! fixture and every source file this build's own corpus (`syntax.rs`,
//! `deny.toml`, `c2-suites.sh`, per the wave brief) actually uses. A file
//! using CR-only or Unicode line separators would chunk differently under
//! this port than under semble's; no such file exists in this build's own
//! corpus today, so the tolerance is named rather than chased.
//!
//! # Grammar selection is duplicated from [`super::syntax`], not reused directly
//!
//! [`super::syntax::SyntaxLanguage`] is reused here for language identity
//! (R2) — the same six-family enum [`super::syntax::language_for`] already
//! routes a path to. Its private `grammar()` method is not: that method has
//! module-private visibility in `syntax.rs`, and widening it (or otherwise
//! editing `syntax.rs`) is exactly the change `00-orient`'s boundary rules
//! out for this wave (*"Any change to `text.rs`, `syntax.rs`, `scan.rs`, or
//! `db.rs`"* is explicitly out of scope). [`grammar_for`] below is the
//! minimal, faithful fallback once that reuse path is closed: the same
//! seven-arm match over the same tree-sitter grammar crates already in
//! `Cargo.toml` (R5), duplicated rather than exposed, at the rung R2
//! genuinely could not reach without violating a settled boundary.
//!
//! # A parse error is not refused here, unlike [`super::syntax::extract`]
//!
//! [`super::syntax::extract`] refuses any tree containing an `ERROR` or
//! missing node and returns no partial symbols. This module does not: it
//! chunks whatever tree `tree-sitter` returns, error nodes included, because
//! that is what semble's own `chunk()` does (`chunking/core.py:137-154` has
//! no `has_error` check at all) — a malformed file still has retrievable
//! text, and chunking is not making a claim about syntactic correctness the
//! way symbol extraction is. This is a deliberate divergence from the
//! sibling module's policy, not an oversight.

use super::syntax::SyntaxLanguage;
use tree_sitter::{Node, Parser};

/// The target chunk length, in characters in semble's original and in bytes
/// here (see the module-level "byte offsets throughout" note).
///
/// Ported from `chunking/chunking.py:10`,
/// `_DESIRED_CHUNK_LENGTH_CHARS = 750`.
pub const DESIRED_CHUNK_LENGTH: usize = 750;

/// A node shorter than this is never split further — its whole span becomes
/// one chunk boundary rather than being recursed into.
///
/// Ported from `chunking/core.py:13`, `_MIN_CHUNK_SIZE = 50`. The wave
/// brief's own prose does not name this constant; `00-orient` located it
/// directly in the oracle source because the AST-merge algorithm depends on
/// it (§5 item 4 of that stage's boundary).
const MIN_CHUNK_SIZE: usize = 50;

/// Recursion depth ceiling for [`merge_node_inner`] — a defensive bound, not
/// one any fixture in this corpus is expected to reach.
///
/// Ported from `chunking/core.py:12`, `_RECURSION_DEPTH = 500`.
const RECURSION_DEPTH: usize = 500;

/// One chunk of a larger file: its text, its byte span, and its line span.
///
/// Mirrors `semble.types.Chunk` minus `file_path`/`language` — this module
/// is pure over bytes (F6) and never sees a path; a caller that wires this
/// in (next wave) attaches those itself, the same split
/// [`super::syntax::Symbol`] and [`super::text::StructureUnit`] already
/// make.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    /// The chunk's own text, sliced from the original.
    pub content: String,
    /// Inclusive start offset into the original bytes.
    pub byte_start: usize,
    /// Exclusive end offset into the original bytes.
    pub byte_end: usize,
    /// 1-based line the chunk starts on.
    pub line_start: usize,
    /// 1-based line the chunk ends on (inclusive).
    pub line_end: usize,
}

/// A candidate span before it becomes a [`Chunk`] — byte offsets only, no
/// text yet. Mirrors `chunking/core.py`'s `ChunkBoundary` dataclass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ChunkBoundary {
    start: usize,
    end: usize,
}

/// Chunk `text` into retrieval-sized spans.
///
/// `language` selects the AST-merge path over `text`'s parse tree when
/// `Some` (matching semble's `chunk_source(source, file_path, language)`
/// called with a non-`None` language, `chunking/chunking.py:13-23`); `None`
/// takes the line-fallback path directly, the same branch semble's original
/// takes both when its caller passes no language and when
/// `_cached_get_parser` cannot find one — a distinction this port does not
/// need to represent, because every [`SyntaxLanguage`] variant always has a
/// bundled grammar (`Cargo.toml`'s pinned grammar set), so the AST path
/// never fails once selected.
///
/// Whitespace-only or empty `text` produces no chunks at all, matching
/// `chunking/chunking.py:15-16`'s `if not source.strip(): return []`.
///
/// Pure: no I/O, no state, no clock, same property [`super::text`] and
/// [`super::syntax`] state and keep for the identical reason (F6).
pub fn chunk_source(text: &str, language: Option<SyntaxLanguage>) -> Vec<Chunk> {
    if text.trim().is_empty() {
        return Vec::new();
    }
    let boundaries = match language {
        Some(language) => ast_chunk_boundaries(text, language),
        None => chunk_lines(text),
    };
    boundaries
        .into_iter()
        .map(|boundary| finalize_chunk(text, boundary))
        .collect()
}

/// Turn one [`ChunkBoundary`] into a [`Chunk`], slicing `text` and counting
/// lines.
///
/// Ported from `chunking/chunking.py:26-38`. semble's own `end_index =
/// max(boundary.end - 1, boundary.start)` clamp exists so a Python slice on
/// a degenerate (zero-length, or past-end-of-text) boundary still returns
/// *something* rather than an off-by-one empty string; Python's own slicing
/// additionally clamps any out-of-range index silently, which Rust's `&str`
/// indexing does not, so `byte_end` is clamped to `text.len()` explicitly
/// here — the one place this port adds an explicit bound semble's dynamic
/// slicing gave it for free.
///
/// # Char-boundary snapping (chunker-utf8 wave)
///
/// A [`ChunkBoundary`] here is always in bytes (see the module-level "byte
/// offsets throughout" note), but a byte offset a caller hands in is not
/// guaranteed to land on a UTF-8 char boundary — every fixture in this
/// corpus was ASCII until this wave, where every byte offset trivially is
/// one, which is exactly why this went unnoticed
/// (`knowledge/evidence/resources/host-atlas-s6-series/brief-chunker-utf8.md`:
/// production panicked at `text[byte_start..byte_end]` on real multi-byte
/// TOML/shell comments). `00-orient`'s own investigation traced the panic
/// to this slice and confirmed there is no separate codepoint-index value
/// being reused as a byte offset here — the boundary value itself is
/// already byte-native; it just is not always *valid*. So the fix is not a
/// unit conversion, it is validation: both `byte_start` and `byte_end` snap
/// to the nearest valid char boundary via [`ceil_char_boundary`] — rounding
/// *up*, never down.
///
/// # Why both ends round up (fixed: F-IN-01)
///
/// Every [`ChunkBoundary`] this module produces is contiguous with its
/// neighbor: [`chunk_lines`] and [`merge_adjacent_chunks`] always set the
/// next boundary's `start` to the previous boundary's raw `end` (see their
/// own bodies), so a straddling codepoint's raw split offset is shared,
/// byte for byte, between chunk N's `end` and chunk N+1's `start`. An
/// earlier version of this fix rounded `byte_end` up
/// ([`ceil_char_boundary`]) but `byte_start` *down*
/// ([`floor_char_boundary`]) — each chunk independently decided to
/// *include* the straddling codepoint, so the same codepoint's bytes were
/// sliced into both chunks' `content`: not mere metadata overlap but
/// duplicated retrieval content (F-IN-01). Rounding both ends up instead
/// means chunk N's `ceil(end)` and chunk N+1's `ceil(start)` resolve to the
/// *same* snapped offset whenever they share a raw split point, so the
/// straddling codepoint lands in exactly one chunk (whichever chunk's raw
/// `end` first reached or passed it) — no duplication, no loss. Both
/// snapping functions are no-ops on any boundary that was already valid
/// (every existing ASCII fixture's output is therefore unchanged, byte for
/// byte).
fn finalize_chunk(text: &str, boundary: ChunkBoundary) -> Chunk {
    let end_index = boundary.end.saturating_sub(1).max(boundary.start);
    let byte_end = ceil_char_boundary(text, (end_index + 1).min(text.len()));
    let byte_start = ceil_char_boundary(text, boundary.start).min(byte_end);
    let content = text[byte_start..byte_end].to_string();
    let line_start = 1 + count_newlines(&text[..byte_start]);
    let line_end = if byte_end > byte_start {
        // `byte_end` is now a valid char boundary, but `byte_end - 1` need
        // not be one (it can sit mid-codepoint, one byte inside a
        // multi-byte char whose first byte is what `floor_char_boundary`
        // below lands on). No '\n' byte ever occurs as a non-leading byte
        // of a multi-byte UTF-8 codepoint, so flooring this index down to
        // its own char boundary cannot change the newline count between
        // here and `byte_end - 1` — the floor is exact, not approximate.
        1 + count_newlines(&text[..floor_char_boundary(text, byte_end - 1)])
    } else {
        line_start
    };
    Chunk {
        content,
        byte_start,
        byte_end,
        line_start,
        line_end,
    }
}

fn count_newlines(text: &str) -> usize {
    text.bytes().filter(|&byte| byte == b'\n').count()
}

/// Snap `index` down to the nearest valid UTF-8 char boundary in `text`
/// (never below 0). Standard library has `str::floor_char_boundary`, but it
/// is nightly-only (`round_char_boundary`, unstable as of this wave's
/// toolchain — R3 checked and unavailable); this is the minimal stable
/// equivalent (R7).
fn floor_char_boundary(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while index > 0 && !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

/// Snap `index` up to the nearest valid UTF-8 char boundary in `text`
/// (never past `text.len()`). See [`floor_char_boundary`] on why this is
/// hand-written rather than `str::ceil_char_boundary`.
fn ceil_char_boundary(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while index < text.len() && !text.is_char_boundary(index) {
        index += 1;
    }
    index
}

/// The AST-merge path: parse `text` as `language`, then merge/split the
/// parse tree into boundaries. Ported dispatch from `chunking/core.py:137-154`.
fn ast_chunk_boundaries(text: &str, language: SyntaxLanguage) -> Vec<ChunkBoundary> {
    let mut parser = Parser::new();
    parser
        .set_language(&grammar_for(language))
        .expect("every SyntaxLanguage grammar is a fixed, already-linked crate");
    let tree = parser
        .parse(text, None)
        .expect("tree-sitter always returns a tree for a text input with no cancellation flag set");
    merge_node(tree.root_node(), DESIRED_CHUNK_LENGTH)
}

/// The tree-sitter grammar for `language`. See the module-level note on why
/// this duplicates rather than calls [`super::syntax::SyntaxLanguage`]'s own
/// private `grammar()`.
fn grammar_for(language: SyntaxLanguage) -> tree_sitter::Language {
    match language {
        SyntaxLanguage::Rust => tree_sitter_rust::LANGUAGE.into(),
        SyntaxLanguage::Toml => tree_sitter_toml_ng::LANGUAGE.into(),
        SyntaxLanguage::Markdown => tree_sitter_md::LANGUAGE.into(),
        SyntaxLanguage::Python => tree_sitter_python::LANGUAGE.into(),
        SyntaxLanguage::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
        SyntaxLanguage::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        SyntaxLanguage::Bash => tree_sitter_bash::LANGUAGE.into(),
    }
}

/// Recursively turn `node` into chunk boundaries, then merge adjacent ones.
///
/// Ported from `chunking/core.py:118-121`, `_merge_node`.
fn merge_node(node: Node, desired_length: usize) -> Vec<ChunkBoundary> {
    let raw = merge_node_inner(node, desired_length, 0);
    merge_adjacent_chunks(&raw)
}

/// Recursively merge and split `node`'s children into boundaries.
///
/// Direct port of `chunking/core.py:68-115`, `_merge_node_inner`, including
/// its three early-return guards (leaf node, recursion depth, node shorter
/// than [`MIN_CHUNK_SIZE`]) and its greedy left-to-right grouping of
/// siblings up to `desired_length`. `node.children` in the Python original
/// (tree-sitter's `Node.children` property) is *every* child, named and
/// anonymous alike; this port's `node.child(index)` walk is the Rust
/// binding's equivalent all-children indexing, not
/// [`Node::named_children`].
fn merge_node_inner(node: Node, desired_length: usize, depth: usize) -> Vec<ChunkBoundary> {
    let child_count = node.child_count();
    if child_count == 0 {
        return vec![ChunkBoundary {
            start: node.start_byte(),
            end: node.end_byte(),
        }];
    }
    let span_length = node.end_byte() - node.start_byte();
    // Prevent recursion issues. A depth of > 500 is unlikely
    // (core.py:75-77).
    if depth > RECURSION_DEPTH {
        return vec![ChunkBoundary {
            start: node.start_byte(),
            end: node.end_byte(),
        }];
    }
    // Prevent recursing into short chunks (core.py:79-81).
    if span_length < MIN_CHUNK_SIZE {
        return vec![ChunkBoundary {
            start: node.start_byte(),
            end: node.end_byte(),
        }];
    }

    let mut groups = Vec::new();
    let mut index = 0usize;
    while index < child_count {
        let child = node
            .child(index as u32)
            .expect("index bounded by child_count on every iteration");
        let start = child.start_byte();
        let mut end = child.end_byte();
        let mut length = end - start;
        index += 1;

        if length > desired_length {
            groups.extend(merge_node_inner(child, desired_length, depth + 1));
            continue;
        }

        while index < child_count {
            let next = node
                .child(index as u32)
                .expect("index bounded by child_count on every iteration");
            let next_length = next.end_byte() - next.start_byte();
            if length + next_length > desired_length {
                break;
            }
            end = next.end_byte();
            length += next_length;
            index += 1;
        }
        groups.push(ChunkBoundary { start, end });
    }
    groups
}

/// Merge a sequence of adjacent boundaries up to `desired_length`, greedily
/// left to right.
///
/// Direct port of `chunking/core.py:38-65`, `_merge_adjacent_chunks`.
/// `chunks` must be non-empty — every caller in this module only ever
/// passes a non-empty list, matching semble's own unchecked `chunks[0]`
/// indexing at `core.py:45-46`.
fn merge_adjacent_chunks(chunks: &[ChunkBoundary]) -> Vec<ChunkBoundary> {
    debug_assert!(
        !chunks.is_empty(),
        "merge_adjacent_chunks called with no boundaries, like semble's chunks[0] would panic on"
    );
    let mut merged = Vec::new();
    let mut current_start = chunks[0].start;
    let mut current_end = chunks[0].end;
    let mut current_length = current_end - current_start;

    for group in &chunks[1..] {
        let length = group.end - group.start;
        if current_length + length > DESIRED_CHUNK_LENGTH {
            merged.push(ChunkBoundary {
                start: current_start,
                end: current_end,
            });
            current_start = group.start;
            current_end = group.end;
            current_length = length;
            continue;
        }
        current_end = group.end;
        current_length += length;
    }
    merged.push(ChunkBoundary {
        start: current_start,
        end: current_end,
    });
    merged
}

/// Ported from `chunking/core.py:124-134`, `chunk_lines`. See the
/// module-level note on the `\n`-only line-splitting tolerance.
fn chunk_lines(text: &str) -> Vec<ChunkBoundary> {
    if text.trim().is_empty() {
        return Vec::new();
    }
    let mut lines_as_groups = Vec::new();
    let mut index = 0usize;
    for line in text.split_inclusive('\n') {
        let length = line.len();
        lines_as_groups.push(ChunkBoundary {
            start: index,
            end: index + length,
        });
        index += length;
    }
    merge_adjacent_chunks(&lines_as_groups)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One fixture's golden data, as captured by
    /// `tests/fixtures/chunk_corpus/build_chunk_goldens.py` from semble's
    /// real chunker — never hand-typed.
    #[derive(serde::Deserialize)]
    struct GoldenChunk {
        byte_start: usize,
        byte_end: usize,
        line_start: usize,
        line_end: usize,
        content: String,
    }

    fn assert_matches_golden(text: &str, language: Option<SyntaxLanguage>, golden_json: &str) {
        let golden: Vec<GoldenChunk> =
            serde_json::from_str(golden_json).expect("golden fixture is valid JSON");
        let chunks = chunk_source(text, language);
        assert_eq!(
            chunks.len(),
            golden.len(),
            "chunk count must match semble's oracle output exactly (no tolerance claimed on count)"
        );
        for (index, (chunk, golden)) in chunks.iter().zip(golden.iter()).enumerate() {
            assert_eq!(
                chunk.byte_start, golden.byte_start,
                "chunk {index} byte_start"
            );
            assert_eq!(chunk.byte_end, golden.byte_end, "chunk {index} byte_end");
            assert_eq!(
                chunk.line_start, golden.line_start,
                "chunk {index} line_start"
            );
            assert_eq!(chunk.line_end, golden.line_end, "chunk {index} line_end");
            assert_eq!(chunk.content, golden.content, "chunk {index} content");
        }
    }

    #[test]
    fn empty_and_whitespace_only_text_produce_no_chunks() {
        assert_eq!(chunk_source("", None), Vec::new());
        assert_eq!(chunk_source("   \n\t  \n", None), Vec::new());
        assert_eq!(
            chunk_source("   \n\t  \n", Some(SyntaxLanguage::Rust)),
            Vec::new()
        );
    }

    #[test]
    fn rust_fixture_matches_semble_oracle() {
        assert_matches_golden(
            include_str!("../../../tests/fixtures/chunk_corpus/sample.rs"),
            Some(SyntaxLanguage::Rust),
            include_str!("../../../tests/fixtures/chunk_corpus/sample.rs.golden.json"),
        );
    }

    #[test]
    fn toml_fixture_matches_semble_oracle() {
        assert_matches_golden(
            include_str!("../../../tests/fixtures/chunk_corpus/sample.toml"),
            Some(SyntaxLanguage::Toml),
            include_str!("../../../tests/fixtures/chunk_corpus/sample.toml.golden.json"),
        );
    }

    /// `sample_multibyte.toml` is the chunker-utf8 wave's own density
    /// fixture (accented words, CJK, emoji, real em-dash comments) — the
    /// first non-ASCII fixture in this corpus. Same golden-oracle
    /// discipline as every other fixture here: `build_chunk_goldens.py`
    /// captures semble's real chunker output, never a hand-typed
    /// expectation.
    ///
    /// # What this does — and does not — prove (F-TH-02)
    ///
    /// This proves non-ASCII round-trip fidelity through the real
    /// AST-merge path: every golden offset lands on a valid char boundary
    /// and every recorded chunk matches semble's own oracle output
    /// byte-for-byte. It does **not** exercise the mid-codepoint
    /// boundary-splitting bug [`finalize_chunk`] was patched for — no
    /// chunk seam in this fixture's committed golden lands mid-codepoint
    /// (tree-sitter node boundaries are always valid char boundaries, so
    /// the AST-merge path cannot naturally produce one; see the corrected
    /// comment in `sample_multibyte.toml` itself, F-IN-02). That specific
    /// invariant is proven directly by the hand-constructed boundaries in
    /// `finalize_chunk_never_panics_on_a_boundary_that_splits_a_multibyte_char`
    /// and `finalize_chunk_never_duplicates_content_across_adjacent_boundaries_
    /// sharing_a_split_point` below, not by this golden comparison.
    #[test]
    fn toml_multibyte_fixture_matches_semble_oracle() {
        assert_matches_golden(
            include_str!("../../../tests/fixtures/chunk_corpus/sample_multibyte.toml"),
            Some(SyntaxLanguage::Toml),
            include_str!("../../../tests/fixtures/chunk_corpus/sample_multibyte.toml.golden.json"),
        );
    }

    #[test]
    fn bash_fixture_matches_semble_oracle() {
        assert_matches_golden(
            include_str!("../../../tests/fixtures/chunk_corpus/sample.sh"),
            Some(SyntaxLanguage::Bash),
            include_str!("../../../tests/fixtures/chunk_corpus/sample.sh.golden.json"),
        );
    }

    #[test]
    fn python_fixture_matches_semble_oracle() {
        assert_matches_golden(
            include_str!("../../../tests/fixtures/chunk_corpus/sample.py"),
            Some(SyntaxLanguage::Python),
            include_str!("../../../tests/fixtures/chunk_corpus/sample.py.golden.json"),
        );
    }

    #[test]
    fn plain_text_fixture_matches_semble_oracle_via_line_fallback() {
        assert_matches_golden(
            include_str!("../../../tests/fixtures/chunk_corpus/sample.txt"),
            None,
            include_str!("../../../tests/fixtures/chunk_corpus/sample.txt.golden.json"),
        );
    }

    /// `finalize_chunk`'s clamp (ported from semble's `end_index = max(end -
    /// 1, start)`, see that function's own docs) must never panic on a
    /// degenerate boundary, even though no fixture in this corpus produces
    /// one from real tree-sitter output. Directed test, not a golden
    /// comparison — there is no oracle output for a boundary that never
    /// occurs in the reference's own real runs either.
    #[test]
    fn finalize_chunk_clamps_a_zero_length_boundary_at_end_of_text() {
        let text = "abc";
        let chunk = finalize_chunk(text, ChunkBoundary { start: 3, end: 3 });
        assert_eq!(chunk.byte_start, 3);
        assert_eq!(chunk.byte_end, 3);
        assert_eq!(chunk.content, "");
        assert_eq!(chunk.line_start, chunk.line_end);
    }

    /// The reproduction behind `knowledge/evidence/resources/host-atlas-s6-series/
    /// brief-chunker-utf8.md`: a real production run of `sgt intelligence scan`
    /// panicked at this exact site (`chunk.rs:191:33` at the time of that
    /// measurement, `"end byte index 3592 is not a char boundary; it is
    /// inside '—'"`) because a `ChunkBoundary` produced by the AST-merge path
    /// can land its end (or, symmetrically, its start) inside a multi-byte
    /// UTF-8 codepoint rather than on one of its byte edges. This directed
    /// test reproduces that shape without depending on which grammar/input
    /// combination causes tree-sitter to emit such a boundary (00-orient's
    /// own investigation could not fully bottom out the mechanism) — it
    /// constructs the degenerate boundary directly, the same pattern the
    /// existing zero-length-boundary test above already uses.
    #[test]
    fn finalize_chunk_never_panics_on_a_boundary_that_splits_a_multibyte_char() {
        // "a—b": a(1 byte) + em-dash U+2014(3 bytes, offsets 1..4) + b(1 byte).
        let text = "a\u{2014}b";
        assert_eq!(text.len(), 5);
        assert!(
            !text.is_char_boundary(2),
            "fixture must land mid-codepoint at 2"
        );
        assert!(
            !text.is_char_boundary(3),
            "fixture must land mid-codepoint at 3"
        );

        // A boundary whose END lands inside the em-dash.
        let end_split = finalize_chunk(text, ChunkBoundary { start: 0, end: 3 });
        assert!(
            text.is_char_boundary(end_split.byte_start)
                && text.is_char_boundary(end_split.byte_end),
            "byte_start/byte_end must always be valid char boundaries: {end_split:?}"
        );
        assert_eq!(
            end_split.content,
            &text[end_split.byte_start..end_split.byte_end],
            "content must match what byte_start/byte_end actually slice"
        );
        assert!(
            end_split.content.contains('\u{2014}'),
            "the em-dash must not be silently truncated: {end_split:?}"
        );

        // A boundary whose START lands inside the em-dash.
        let start_split = finalize_chunk(text, ChunkBoundary { start: 2, end: 5 });
        assert!(
            text.is_char_boundary(start_split.byte_start)
                && text.is_char_boundary(start_split.byte_end),
            "byte_start/byte_end must always be valid char boundaries: {start_split:?}"
        );
        assert_eq!(
            start_split.content,
            &text[start_split.byte_start..start_split.byte_end]
        );
    }

    /// F-IN-01: two *adjacent* boundaries that share a raw split point
    /// landing mid-codepoint (exactly what [`chunk_lines`] and
    /// [`merge_adjacent_chunks`] always produce — chunk N+1's `start`
    /// equals chunk N's raw `end`) must not slice the straddling
    /// codepoint's bytes into both chunks' `content`. An earlier version of
    /// [`finalize_chunk`] rounded `byte_end` up and `byte_start` down
    /// independently, so each side of the pair separately chose to
    /// *include* the em-dash, duplicating it across both chunks — this is
    /// the negative case the single-boundary tests above never exercised.
    #[test]
    fn finalize_chunk_never_duplicates_content_across_adjacent_boundaries_sharing_a_split_point() {
        // "a—b": a(1 byte) + em-dash U+2014(3 bytes, offsets 1..4) + b(1 byte).
        let text = "a\u{2014}b";
        assert!(
            !text.is_char_boundary(3),
            "fixture must share a mid-codepoint raw split point at 3"
        );

        // Chunk N ends, and chunk N+1 starts, at the same raw offset (3) —
        // exactly how this module's own boundary producers emit adjacent
        // chunks.
        let chunk_n = finalize_chunk(text, ChunkBoundary { start: 0, end: 3 });
        let chunk_n_plus_1 = finalize_chunk(text, ChunkBoundary { start: 3, end: 5 });

        assert_eq!(
            chunk_n.byte_end, chunk_n_plus_1.byte_start,
            "adjacent chunks must snap to the SAME char boundary at a shared raw split point, \
             not diverge (one rounding up, the other down)"
        );
        let combined_dash_count = chunk_n.content.matches('\u{2014}').count()
            + chunk_n_plus_1.content.matches('\u{2014}').count();
        assert_eq!(
            combined_dash_count, 1,
            "the straddling em-dash must appear in exactly one of the two chunks, not zero \
             (lost) or two (duplicated): chunk_n={:?} chunk_n_plus_1={:?}",
            chunk_n, chunk_n_plus_1
        );
        assert_eq!(
            format!("{}{}", chunk_n.content, chunk_n_plus_1.content),
            text,
            "concatenating the two chunks must reproduce the source exactly, with no \
             duplicated or dropped bytes at the shared seam"
        );
    }
}
