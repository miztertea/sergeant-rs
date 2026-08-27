//! Text and Markdown structure extraction — **pure functions over bytes**
//! (F6's adapter-shape mandate).
//!
//! Nothing in this module opens a file, touches a database, reads a clock, or
//! knows a source exists. Every entry point takes bytes (or the `&str` that
//! decoding them produced) and returns owned structure. That is not stylistic
//! tidiness: it is the one property that lets extraction be reviewed,
//! fuzzed and unit-tested without a daemon, and the reason the DB glue in
//! [`super::record`] can be read on its own and seen to add nothing.
//!
//! # Provenance is byte offsets into the original
//!
//! Every [`StructureUnit`] carries `byte_start`/`byte_end` into the bytes it
//! was extracted from — not into a normalized, re-wrapped or re-encoded copy.
//! A1 §3's provenance question ("can the original resource still be
//! identified?") is answerable by slicing the original file, which is exactly
//! what makes the original bytes canonical and this output derived (A1-12).
//! Decoding is UTF-8 and lossless, so a `&str` offset *is* a byte offset.
//!
//! # What this build understands
//!
//! ATX headings (`# ` .. `###### `), outside fenced code blocks. Setext
//! headings (`===`/`---` underlines) are **not** recognized: they interact
//! with lazy continuation lines and list interruption in ways that cost far
//! more than they buy for a first content family (R1). A document that uses
//! them is still fully indexed — it simply has one Document unit and no
//! sections, which coverage reports as `indexed` because it is.

use crate::domain::source::UnitKind;

/// Extractor identity for Markdown (F7's second cache-key input).
///
/// Versioned in the string on purpose: changing what [`markdown_units`]
/// produces means bumping this, which changes every derived key, which is
/// what makes a re-extraction happen instead of a stale reuse.
pub const MARKDOWN_EXTRACTOR: &str = "markdown/v1";

/// Extractor identity for plain text.
pub const TEXT_EXTRACTOR: &str = "text/v1";

/// One extracted structure unit, positioned in the original bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureUnit {
    /// Whole resource, or a heading-delimited span.
    pub kind: UnitKind,
    /// Heading depth for a section (1..=6); `None` for a document and for a
    /// preamble section that precedes the first heading.
    pub heading_level: Option<u8>,
    /// Heading text, trimmed of its `#` markers; `None` when there is none.
    pub title: Option<String>,
    /// Inclusive start offset into the original bytes.
    pub byte_start: usize,
    /// Exclusive end offset into the original bytes.
    pub byte_end: usize,
}

impl StructureUnit {
    /// The unit's own bytes, sliced back out of the original.
    ///
    /// Total by construction for any `original` this unit was extracted from;
    /// returns `None` for a mismatched buffer rather than panicking, because
    /// the one caller that could get that wrong is DB glue reading a row back
    /// against a re-read file.
    pub fn slice<'a>(&self, original: &'a [u8]) -> Option<&'a [u8]> {
        original.get(self.byte_start..self.byte_end)
    }
}

/// Decode bytes as UTF-8, or refuse.
///
/// The whole of this build's "is it text?" test, and deliberately strict:
/// a lossy decode would invent replacement characters that appear in a unit's
/// text but not in the original bytes, silently breaking the provenance rule
/// above. Bytes this refuses are reported `unsupported`, not indexed badly.
pub fn as_text(bytes: &[u8]) -> Option<&str> {
    std::str::from_utf8(bytes).ok()
}

/// Structure units for a Markdown document: one [`UnitKind::Document`] unit
/// spanning the whole input, then one [`UnitKind::Section`] per span.
///
/// Sections are flat and exhaustive — every byte of the document belongs to
/// exactly one section (a preamble section covers anything before the first
/// heading) — and nesting is carried by `heading_level` rather than by
/// containment. Flat-and-exhaustive is the shape retrieval wants: a section
/// can be returned whole without reconstructing a tree, and no byte is
/// unreachable through any unit.
pub fn markdown_units(text: &str) -> Vec<StructureUnit> {
    let mut units = vec![StructureUnit {
        kind: UnitKind::Document,
        heading_level: None,
        title: None,
        byte_start: 0,
        byte_end: text.len(),
    }];

    let headings = atx_headings(text);
    if headings.is_empty() {
        return units;
    }
    // The Document unit borrows the first heading's text as a title: a
    // document's own name is the most useful single label retrieval can show,
    // and inventing one from the filename would be provenance this module
    // does not have (it never sees a path).
    units[0].title = headings[0].title.clone();

    // Anything before the first heading is its own section, unless it is
    // only whitespace — a blank preamble is not evidence.
    let first = headings[0].byte_start;
    if !text[..first].trim().is_empty() {
        units.push(StructureUnit {
            kind: UnitKind::Section,
            heading_level: None,
            title: None,
            byte_start: 0,
            byte_end: first,
        });
    }
    for (index, heading) in headings.iter().enumerate() {
        let end = headings
            .get(index + 1)
            .map_or(text.len(), |next| next.byte_start);
        units.push(StructureUnit {
            kind: UnitKind::Section,
            heading_level: Some(heading.level),
            title: heading.title.clone(),
            byte_start: heading.byte_start,
            byte_end: end,
        });
    }
    units
}

/// Structure units for plain text: exactly one [`UnitKind::Document`] unit.
///
/// Plain text has no structure this build can claim to see. Inventing
/// paragraph units would assert a document model the format does not carry
/// (R1); one honest whole-document unit is what it is.
pub fn plain_units(text: &str) -> Vec<StructureUnit> {
    vec![StructureUnit {
        kind: UnitKind::Document,
        heading_level: None,
        title: None,
        byte_start: 0,
        byte_end: text.len(),
    }]
}

/// One ATX heading found in the source.
struct Heading {
    level: u8,
    title: Option<String>,
    byte_start: usize,
}

/// Every ATX heading outside a fenced code block, in document order.
///
/// Fence tracking is the part that has to be right: a `# ` inside a fenced
/// block is code being shown, not a heading, and treating it as one would cut
/// a section boundary through the middle of an example.
fn atx_headings(text: &str) -> Vec<Heading> {
    let mut headings = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    let mut offset = 0usize;
    for line in text.split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        let trimmed = line.trim_end_matches(['\n', '\r']);
        let indent = trimmed.len() - trimmed.trim_start_matches(' ').len();
        // More than three leading spaces is an indented code block, which no
        // heading can start inside.
        if indent > 3 {
            continue;
        }
        let body = &trimmed[indent..];
        if let Some((fence_char, fence_len)) = fence {
            // A closing fence is the same character, at least as long, and
            // followed by nothing but whitespace.
            let run = body.chars().take_while(|c| *c == fence_char).count();
            if run >= fence_len && body[run..].trim().is_empty() {
                fence = None;
            }
            continue;
        }
        for fence_char in ['`', '~'] {
            let run = body.chars().take_while(|c| *c == fence_char).count();
            if run >= 3 {
                fence = Some((fence_char, run));
                break;
            }
        }
        if fence.is_some() {
            continue;
        }
        let hashes = body.chars().take_while(|c| *c == '#').count();
        if hashes == 0 || hashes > 6 {
            continue;
        }
        let rest = &body[hashes..];
        // `#hashtag` is not a heading: CommonMark requires the run of `#` to
        // be followed by a space or end of line.
        if !rest.is_empty() && !rest.starts_with(' ') && !rest.starts_with('\t') {
            continue;
        }
        let title = rest.trim().trim_end_matches('#').trim();
        headings.push(Heading {
            level: hashes as u8,
            title: (!title.is_empty()).then(|| title.to_string()),
            byte_start: start,
        });
    }
    headings
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The provenance rule, as an executable claim: every unit slices back
    /// out of the exact bytes it was extracted from, and the sections tile
    /// the document with no gap and no overlap.
    fn assert_tiles(text: &str, units: &[StructureUnit]) {
        let bytes = text.as_bytes();
        for unit in units {
            assert!(
                unit.slice(bytes).is_some(),
                "unit {unit:?} does not slice its own source"
            );
        }
        let mut cursor = 0usize;
        for section in units.iter().filter(|u| u.kind == UnitKind::Section) {
            assert_eq!(section.byte_start, cursor, "gap or overlap at {cursor}");
            cursor = section.byte_end;
        }
        if units.iter().any(|u| u.kind == UnitKind::Section) {
            assert_eq!(cursor, text.len(), "sections do not reach the end");
        }
    }

    #[test]
    fn a_document_with_headings_becomes_a_document_and_flat_sections() {
        let text = "intro line\n\n# Title\n\nbody\n\n## Sub\n\nmore\n";
        let units = markdown_units(text);
        assert_tiles(text, &units);

        assert_eq!(units[0].kind, UnitKind::Document);
        assert_eq!(units[0].byte_start, 0);
        assert_eq!(units[0].byte_end, text.len());
        assert_eq!(units[0].title.as_deref(), Some("Title"));

        let sections: Vec<_> = units
            .iter()
            .filter(|u| u.kind == UnitKind::Section)
            .collect();
        assert_eq!(sections.len(), 3, "preamble + two headings");
        assert_eq!(sections[0].heading_level, None);
        assert_eq!(
            sections[0].slice(text.as_bytes()),
            Some(b"intro line\n\n".as_slice())
        );
        assert_eq!(sections[1].heading_level, Some(1));
        assert_eq!(sections[1].title.as_deref(), Some("Title"));
        assert_eq!(
            sections[1].slice(text.as_bytes()),
            Some(b"# Title\n\nbody\n\n".as_slice())
        );
        assert_eq!(sections[2].heading_level, Some(2));
        assert_eq!(sections[2].title.as_deref(), Some("Sub"));
    }

    /// No headings is not a failure — it is one honest document unit.
    #[test]
    fn a_document_without_headings_is_one_unit_and_an_empty_one_still_exists() {
        let units = markdown_units("just prose\n");
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].kind, UnitKind::Document);

        let empty = markdown_units("");
        assert_eq!(empty.len(), 1);
        assert_eq!((empty[0].byte_start, empty[0].byte_end), (0, 0));
    }

    /// The fence rule. A `#` inside a fenced block is code being shown; a
    /// section boundary there would cut an example in half.
    #[test]
    fn headings_inside_fenced_code_are_not_headings() {
        let text = "\
# Real

```sh
# not a heading
```

~~~
## also not one
~~~

## Second real
";
        let units = markdown_units(text);
        assert_tiles(text, &units);
        let titles: Vec<_> = units
            .iter()
            .filter(|u| u.kind == UnitKind::Section)
            .map(|u| u.title.clone())
            .collect();
        assert_eq!(
            titles,
            vec![Some("Real".to_string()), Some("Second real".to_string())]
        );
    }

    /// A fence closes only on its own character at its own length or longer —
    /// so a shorter run, or the other fence character, stays inside the block.
    #[test]
    fn a_fence_closes_only_on_a_matching_longer_or_equal_run() {
        let text = "````\n``\n~~~\n# still code\n````\n# out\n";
        let units = markdown_units(text);
        assert_tiles(text, &units);
        let titles: Vec<_> = units
            .into_iter()
            .filter(|u| u.kind == UnitKind::Section)
            .map(|u| u.title)
            .collect();
        // The fenced block itself is the (untitled) preamble section: it is
        // content, so it belongs to a unit. Only `# out` is a heading.
        assert_eq!(titles, vec![None, Some("out".to_string())]);
    }

    /// CommonMark's own edge cases, kept because each one is a silent
    /// mis-section if it regresses.
    #[test]
    fn atx_edge_cases_match_commonmark_where_this_build_claims_to() {
        // `#hashtag` needs a space to be a heading.
        assert!(atx_headings("#hashtag\n").is_empty());
        // Seven hashes is not a heading.
        assert!(atx_headings("####### deep\n").is_empty());
        // Up to three spaces of indent is fine; four is an indented code
        // block.
        assert_eq!(atx_headings("   # ok\n").len(), 1);
        assert!(atx_headings("    # code\n").is_empty());
        // Closing hashes are decoration, not title text.
        assert_eq!(
            atx_headings("## Title ##\n")[0].title.as_deref(),
            Some("Title")
        );
        // A bare `#` is a heading with no title, not a heading named "#".
        let bare = atx_headings("#\n");
        assert_eq!(bare.len(), 1);
        assert_eq!(bare[0].title, None);
        // CRLF input keeps its offsets and loses no bytes.
        let crlf = "# A\r\nbody\r\n";
        assert_tiles(crlf, &markdown_units(crlf));
    }

    /// Multi-byte input: offsets stay byte offsets, so a slice is still
    /// valid UTF-8 at a unit boundary and no character is split.
    #[test]
    fn offsets_are_byte_offsets_over_multibyte_text() {
        let text = "# Café ☕\n\nnaïve — em—dash\n\n## Ünter\n\nbody\n";
        let units = markdown_units(text);
        assert_tiles(text, &units);
        for unit in &units {
            let slice = unit.slice(text.as_bytes()).expect("slice");
            assert!(
                std::str::from_utf8(slice).is_ok(),
                "unit boundary split a character: {unit:?}"
            );
        }
        assert_eq!(units[0].title.as_deref(), Some("Café ☕"));
    }

    #[test]
    fn plain_text_is_exactly_one_document_unit_and_non_utf8_is_refused() {
        let units = plain_units("a\nb\n");
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].kind, UnitKind::Document);
        assert_eq!(units[0].byte_end, 4);

        assert!(as_text(b"plain").is_some());
        // A lone continuation byte is not UTF-8, and a lossy decode would
        // invent a character the original bytes do not contain.
        assert!(as_text(&[0x66, 0x80, 0x66]).is_none());
        assert!(as_text(&[0x00, 0x01, 0x02]).is_some_and(|t| t.len() == 3));
    }

    /// F6, stated where it can be checked: these are functions of their
    /// input and nothing else. Two calls on equal bytes are equal, and the
    /// module names no path, clock, connection or environment.
    #[test]
    fn extraction_is_a_pure_function_of_its_input() {
        let text = "# One\n\ntext\n";
        assert_eq!(markdown_units(text), markdown_units(text));
        // Equal *bytes*, not the same buffer: the result depends on content,
        // never on where the content lives.
        let copied = String::from_utf8(text.as_bytes().to_vec()).expect("utf8");
        assert_eq!(markdown_units(text), markdown_units(copied.as_str()));

        let source = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/runtime/atlas/text.rs"
        ))
        .expect("read own source");
        let body = source
            .split("#[cfg(test)]")
            .next()
            .expect("non-test half of this module");
        // The database driver is deliberately absent from this list:
        // `tests/x1_atlas_substrate.rs` already asserts, for every file in
        // this tree, that only `db.rs` names it — and asserting it a second
        // time here would mean this file names the crate, which that test
        // then (correctly) refuses.
        for forbidden in ["std::fs", "SystemTime", "std::env", "Connection"] {
            assert!(
                !body.contains(forbidden),
                "the extractor names {forbidden}, so it is not a pure function over bytes (F6)"
            );
        }
    }
}
