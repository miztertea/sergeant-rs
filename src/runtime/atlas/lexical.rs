//! S5 W2 — A2 §5's lexical retrieval: tokenization and BM25 scoring.
//!
//! A2 §5, verbatim: *"Start with a small local BM25 implementation tuned for
//! identifier/document tokens rather than adding a full search server."*
//! Decision **A2-05 (R7)** already checked the lower rungs and this module
//! does not re-derive them: *"R1 yes; R2 no current lexical index exists;
//! R3–R5 stdlib/platform/installed deps do not provide BM25; R6 cannot
//! express an index; one small local implementation is less machinery than a
//! search service."*
//!
//! This module is **pure Rust over strings and numbers**. It holds no
//! database connection and names no database driver — Atlas's one-owner
//! invariant ([`super::db`], pinned by
//! `tests/x1_atlas_substrate.rs::atlas_database_has_exactly_one_owner`) means
//! the SQL that stores postings and applies A2 §2's admissibility filter
//! lives in [`super::db`], and only the tokenizer, the scoring expression and
//! the result vocabulary live here.
//!
//! # What A2 §5 asks tokenization to preserve
//!
//! *"Tokenization should preserve useful forms such as"* — six literal forms,
//! and each one is a test case in `tests/w2_lexical_retrieval.rs`:
//!
//! ```text
//! PaymentRetryPolicy
//! payment_retry_policy
//! payment-retry-policy
//! Foo::bar
//! POST /payments
//! INC0012345
//! ```
//!
//! [`tokenize`] emits, for every compound it finds, **the whole compound as
//! written** *and* its parts. Both halves matter and the first is the one
//! easy to lose: a tokenizer that only splits `PaymentRetryPolicy` into
//! `payment`/`retry`/`policy` has destroyed the exact-identifier advantage
//! that is BM25's entire reason for being in this design (A2 §5's own
//! "tuned for identifier/document tokens", and [EXT-SEMBLE]'s "particularly
//! for identifiers/API names"). So `PaymentRetryPolicy` and `payment` both
//! find the unit, and `tests/w2_lexical_retrieval.rs::
//! splitting_a_camel_identifier_never_costs_the_whole_identifier` fails if
//! either half stops being true.
//!
//! # *"Document/mail retrieval additionally retains ordinary
//! natural-language tokens"*
//!
//! That sentence is true of this design through **what each family indexes**,
//! not through a second tokenizer (R1: a second token rule does not need to
//! exist). [`LexicalFamily::Code`]'s indexed text is the symbol name a
//! grammar claimed — identifiers, and nothing else. The document, mail and
//! selected-row-text families index the unit's own title and body, so their
//! postings additionally carry every ordinary word of the prose. One
//! tokenizer, applied to different text, which is the minimum that makes the
//! contract sentence true;
//! `tests/w2_lexical_retrieval.rs::code_units_index_identifiers_while_
//! document_units_additionally_index_prose` is what pins it.
//!
//! # Scoring
//!
//! [`bm25_contribution`] is the textbook Okapi BM25 term contribution, one
//! expression, summed over the query's distinct terms by [`Bm25Corpus`].
//! Nothing is trained and nothing self-tunes (A2 §16 forbids both).

use std::collections::BTreeMap;

/// Which of A2 §17 item 2's four unit families a lexical hit belongs to —
/// *"lexical search returns code/document/mail/selected-row-text units with
/// exact A1 provenance"*.
///
/// The family is a property of the A1 rows the posting was derived from, not
/// a guess about content: [`Self::Code`] is a `source.occurrences` row,
/// [`Self::Document`] and [`Self::Mail`] are `source.units` rows split by the
/// owning file's extractor identity, and [`Self::RowText`] is a
/// `context.row_units` row (A1's F10a-gated selected-row text).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LexicalFamily {
    /// A grammar-claimed symbol site (`source.occurrences`).
    Code,
    /// A document structure unit (`source.units`, a document extractor).
    Document,
    /// A mail structure unit (`source.units`, the mail extractor).
    Mail,
    /// One selected-row-text unit (`context.row_units`).
    RowText,
}

impl LexicalFamily {
    /// The stable DB spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Document => "document",
            Self::Mail => "mail",
            Self::RowText => "row-text",
        }
    }

    /// The inverse of [`Self::as_str`].
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "code" => Some(Self::Code),
            "document" => Some(Self::Document),
            "mail" => Some(Self::Mail),
            "row-text" => Some(Self::RowText),
            _ => None,
        }
    }
}

impl std::fmt::Display for LexicalFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The characters a compound is made of: every alphanumeric character, plus
/// the three separators the six contract forms are spelled with (`_`, `-`,
/// `:`). `char::is_alphanumeric` rather than `is_ascii_alphanumeric` so a
/// non-ASCII word (`café`) is one compound rather than a truncated `caf`.
fn is_compound(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-' || c == ':'
}

/// The separators a compound is split on into words.
fn is_separator(c: char) -> bool {
    c == '_' || c == '-' || c == ':'
}

/// A2 §5's tokenizer: every token `text` contributes, **in order, with
/// repeats** (the caller counts term frequencies).
///
/// For each maximal run of [`is_compound`] characters holding at least one
/// alphanumeric:
///
/// 1. the whole compound, lowercased — `payment_retry_policy`, `foo::bar`,
///    `inc0012345`, `paymentretrypolicy`;
/// 2. its separator-split words, when there is more than one — `payment`,
///    `retry`, `policy` from `payment-retry-policy`; `foo`, `bar` from
///    `Foo::bar`;
/// 3. each word's camel/digit-split parts, when there is more than one —
///    `payment`, `retry`, `policy` from `PaymentRetryPolicy`; `inc`,
///    `0012345` from `INC0012345`; `http`, `server` from `HTTPServer`.
///
/// Plus one joint form, because `POST /payments` is the one contract form
/// that spans a space: an all-uppercase compound of 2–8 letters followed by
/// exactly `" /"` and a path run emits `post /payments` in addition to
/// `post` and `payments`. The bound is a shape, not a method allowlist —
/// this tokenizer has no business knowing HTTP's verb set, and an allowlist
/// would silently drop tomorrow's verb.
///
/// Everything is lowercased, so retrieval is case-insensitive in both
/// directions; the exact-case spelling survives in the A1 unit the hit cites,
/// which is where an answer's exactness actually lives.
pub fn tokenize(text: &str) -> Vec<String> {
    let compounds = compounds_of(text);
    let mut out = Vec::new();
    for (index, (start, end)) in compounds.iter().enumerate() {
        let compound = &text[*start..*end];
        if !compound.chars().any(char::is_alphanumeric) {
            continue;
        }
        out.push(compound.to_lowercase());
        let words: Vec<&str> = compound
            .split(is_separator)
            .filter(|w| !w.is_empty())
            .collect();
        for word in &words {
            if words.len() > 1 {
                out.push(word.to_lowercase());
            }
            let parts = case_split(word);
            if parts.len() > 1 {
                for part in parts {
                    out.push(part.to_lowercase());
                }
            }
        }
        if let Some(joint) = method_path_token(text, &compounds, index) {
            out.push(joint);
        }
    }
    out
}

/// The byte spans of every maximal [`is_compound`] run in `text`.
fn compounds_of(text: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut open: Option<usize> = None;
    for (offset, c) in text.char_indices() {
        match (is_compound(c), open) {
            (true, None) => open = Some(offset),
            (false, Some(start)) => {
                spans.push((start, offset));
                open = None;
            }
            _ => {}
        }
    }
    if let Some(start) = open {
        spans.push((start, text.len()));
    }
    spans
}

/// `POST /payments`'s joint token, when the compound at `index` is an
/// uppercase method-shaped run immediately followed by `" /"` and a path.
fn method_path_token(text: &str, compounds: &[(usize, usize)], index: usize) -> Option<String> {
    let (start, end) = compounds[index];
    let method = &text[start..end];
    if method.len() < 2 || method.len() > 8 || !method.chars().all(|c| c.is_ascii_uppercase()) {
        return None;
    }
    let rest = text.get(end..)?;
    let path = rest.strip_prefix(" /")?;
    let taken: String = path
        .chars()
        .take_while(|c| is_compound(*c) || *c == '/')
        .collect();
    if taken.is_empty() {
        return None;
    }
    Some(format!(
        "{} /{}",
        method.to_lowercase(),
        taken.to_lowercase()
    ))
}

/// Split one separator-free word at camel-case and letter/digit boundaries.
fn case_split(word: &str) -> Vec<&str> {
    let chars: Vec<(usize, char)> = word.char_indices().collect();
    let mut parts = Vec::new();
    let mut start = 0usize;
    for position in 1..chars.len() {
        let (offset, current) = chars[position];
        let (_, previous) = chars[position - 1];
        let next = chars.get(position + 1).map(|(_, c)| *c);
        let boundary =
            // lower/digit -> upper: `paymentRetry`
            (!previous.is_uppercase() && current.is_uppercase())
            // acronym end: `HTTPServer` breaks before `Server`
            || (previous.is_uppercase()
                && current.is_uppercase()
                && next.is_some_and(|c| c.is_lowercase()))
            // letter <-> digit: `INC0012345`, `v1Alpha`
            || (previous.is_alphabetic() && current.is_numeric())
            || (previous.is_numeric() && current.is_alphabetic());
        if boundary {
            parts.push(&word[start..offset]);
            start = offset;
        }
    }
    parts.push(&word[start..]);
    parts.retain(|p| !p.is_empty());
    parts
}

/// Distinct query terms, in a deterministic order — [`tokenize`] with
/// repeats folded away, because a term repeated in the *query* multiplies a
/// document's score without saying anything more about the document.
pub fn query_terms(text: &str) -> Vec<String> {
    let mut terms: Vec<String> = tokenize(text);
    terms.sort();
    terms.dedup();
    terms
}

/// [`tokenize`]'s output as `(term, frequency)` pairs, plus the document
/// length the scorer needs — one pass, deterministic order (`BTreeMap`).
pub fn term_frequencies(text: &str) -> (BTreeMap<String, u64>, u64) {
    let tokens = tokenize(text);
    let length = tokens.len() as u64;
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for token in tokens {
        *counts.entry(token).or_insert(0) += 1;
    }
    (counts, length)
}

/// Okapi BM25's `k1` — the term-frequency saturation constant.
///
/// **Provenance: this is the textbook default, not a measurement of this
/// corpus.** `k1 = 1.2` and `b = 0.75` are the values Robertson et al.'s
/// Okapi BM25 has carried since the TREC-3 experiments, and they are used
/// here because no corpus of Sergeant evidence units has been measured. When
/// one is, this constant is the thing a measurement replaces; until then it
/// is honest to say it was inherited rather than derived.
pub const BM25_K1: f64 = 1.2;

/// Okapi BM25's `b` — the document-length normalization constant. See
/// [`BM25_K1`] for its provenance, which is the same.
pub const BM25_B: f64 = 0.75;

/// The corpus-level facts BM25 needs, measured over the **admissible** set
/// and nothing wider (A2 §2: the filter decides the world; ranking happens
/// inside it). A term's IDF and a unit's length normalization are therefore
/// both relative to what the caller was allowed to see — an inadmissible
/// generation does not merely fail to appear in the results, it does not
/// influence them either.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bm25Corpus {
    /// How many admissible units are in the corpus.
    pub units: u64,
    /// Their mean token count.
    pub average_length: f64,
}

/// One term's BM25 contribution to one unit's score.
///
/// ```text
/// idf(t)  = ln(1 + (N - df + 0.5) / (df + 0.5))
/// score  += idf(t) * tf * (k1 + 1) / (tf + k1 * (1 - b + b * dl / avgdl))
/// ```
///
/// The `ln(1 + ...)` form is the standard non-negative variant: the raw
/// Robertson/Sparck-Jones IDF goes negative for a term appearing in more than
/// half the corpus, and a negative contribution would let a *matching* term
/// push a unit down the list, which is not a ranking anyone can explain.
pub fn bm25_contribution(
    corpus: Bm25Corpus,
    document_frequency: u64,
    term_frequency: u64,
    unit_length: u64,
) -> f64 {
    if corpus.units == 0 || document_frequency == 0 || term_frequency == 0 {
        return 0.0;
    }
    let n = corpus.units as f64;
    let df = document_frequency as f64;
    let tf = term_frequency as f64;
    let idf = (1.0 + (n - df + 0.5) / (df + 0.5)).ln();
    let average = if corpus.average_length > 0.0 {
        corpus.average_length
    } else {
        1.0
    };
    let normalized = BM25_K1 * (1.0 - BM25_B + BM25_B * (unit_length as f64) / average);
    idf * (tf * (BM25_K1 + 1.0)) / (tf + normalized)
}

/// Where one lexical hit came from, in A1's own coordinates — A2 §3: *"A
/// retrievable unit resolves back to an A1 SourceGeneration/Resource/native
/// coordinate"*, and *"the result's evidence coordinate remains A1-owned"*.
///
/// **The coordinate is family-shaped, because A2 §3 says it is.** §3 lists a
/// different coordinate per family, and only three of the four are spans:
/// code is `source/revision/path/symbol/span`, document is
/// `source/generation/path/heading-or-slide/section`, email is
/// `source/generation/path/message-id/body-or-attachment coordinate`, and
/// **structured text is `source/generation/dataset/row-id/field-set` — no
/// byte range at all**, because a selected-row-text unit is assembled from
/// allowlisted columns of a row that is read in place and never copied into
/// Atlas. The W2 brief's summary line ("every hit cites source + generation +
/// unit + byte range") is the common case, not the contract; where the two
/// disagree the contract wins (J5), so [`Self::RowText`] carries the row
/// identity and the exposed field set instead of a span it could only invent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitCoordinate {
    /// A2 §3's code coordinate: path, symbol, and the span of the definition
    /// site in the original file bytes.
    Code {
        /// Path relative to the source root.
        relative_path: String,
        /// The grammar's language name.
        language: String,
        /// The grammar's own label for the site (`function`, `struct`, ...).
        label: String,
        /// The symbol name as written.
        symbol: String,
        /// Position of the site within its file, in extraction order.
        ordinal: u64,
        /// Start offset into the original file bytes.
        byte_start: u64,
        /// End offset into the original file bytes, exclusive.
        byte_end: u64,
    },
    /// A2 §3's document coordinate: path, the heading/section this unit is,
    /// and its span of the original file bytes.
    Document {
        /// Path relative to the source root.
        relative_path: String,
        /// Position of the unit within its file.
        ordinal: u64,
        /// The heading text, when the unit is a section.
        title: Option<String>,
        /// Start offset into the original file bytes.
        byte_start: u64,
        /// End offset into the original file bytes, exclusive.
        byte_end: u64,
    },
    /// A2 §3's email coordinate, as far as A1's stored rows carry it: the
    /// `.eml` resource's path, the unit within it, and its byte span.
    Mail {
        /// Path relative to the source root.
        relative_path: String,
        /// Position of the unit within the message.
        ordinal: u64,
        /// The unit's title — the subject, for a message's own body unit.
        title: Option<String>,
        /// Start offset into the original file bytes.
        byte_start: u64,
        /// End offset into the original file bytes, exclusive.
        byte_end: u64,
    },
    /// A2 §3's structured-text coordinate:
    /// `source/generation/dataset/row-id/field-set`.
    RowText {
        /// Path relative to the source root.
        relative_path: String,
        /// F7's key for the dataset the row came from.
        dataset_key: String,
        /// Position in the dataset, in reader order.
        ordinal: u64,
        /// The row's name.
        row_key: String,
        /// The allowlisted columns that produced this unit (F10a's audit
        /// trail of what was exposed).
        fields: Vec<String>,
    },
}

impl UnitCoordinate {
    /// The family this coordinate belongs to.
    pub fn family(&self) -> LexicalFamily {
        match self {
            Self::Code { .. } => LexicalFamily::Code,
            Self::Document { .. } => LexicalFamily::Document,
            Self::Mail { .. } => LexicalFamily::Mail,
            Self::RowText { .. } => LexicalFamily::RowText,
        }
    }

    /// Path relative to the source root — the one field every coordinate
    /// shape has, and half of the deterministic tie-break key.
    pub fn relative_path(&self) -> &str {
        match self {
            Self::Code { relative_path, .. }
            | Self::Document { relative_path, .. }
            | Self::Mail { relative_path, .. }
            | Self::RowText { relative_path, .. } => relative_path,
        }
    }

    /// Position within the resource — the other half of the tie-break key.
    pub fn ordinal(&self) -> u64 {
        match self {
            Self::Code { ordinal, .. }
            | Self::Document { ordinal, .. }
            | Self::Mail { ordinal, .. }
            | Self::RowText { ordinal, .. } => *ordinal,
        }
    }
}

/// One ranked lexical hit: a BM25 score and the A1 coordinate that makes it
/// traceable to exact evidence.
///
/// *"A hit that cannot be traced back to exact bytes is not a hit, it is a
/// claim"* (the W2 brief) — so `source_name`, `generation_id`, `content_key`
/// and `coordinate` are not decoration and are not optional. A caller holding
/// one of these can name the source, the exact generation of it, the resource
/// inside that generation, and — for every family whose evidence is a
/// resource's bytes — the byte range within it.
#[derive(Debug, Clone, PartialEq)]
pub struct LexicalHit {
    /// The summed BM25 score over the query's distinct terms.
    pub score: f64,
    /// The declared source.
    pub source_name: String,
    /// The exact SourceGeneration this unit belongs to.
    pub generation_id: String,
    /// That generation's content identity.
    pub content_key: String,
    /// The index's own per-generation unit identity — `<family>:<path>#<ordinal>`.
    pub unit_key: String,
    /// A1's coordinate for the unit itself.
    pub coordinate: UnitCoordinate,
}

impl LexicalHit {
    /// The stated tie-break key: `(source_name, relative_path, ordinal,
    /// unit_key)`, applied in that order to hits whose scores are equal.
    ///
    /// Every component is a stored value, so the order is a function of the
    /// evidence and nothing else — never `HashMap` iteration order, never the
    /// order rows happened to arrive in. `tests/w2_lexical_retrieval.rs::
    /// equal_scores_are_broken_by_the_stated_key_not_by_row_arrival_order`
    /// seeds the tying units in the reverse of this order and fails if the
    /// answer follows arrival instead.
    pub fn tie_break_key(&self) -> (&str, &str, u64, &str) {
        (
            &self.source_name,
            self.coordinate.relative_path(),
            self.coordinate.ordinal(),
            &self.unit_key,
        )
    }
}

/// Order two hits by A2 §8's deterministic rule: **score descending, then
/// [`LexicalHit::tie_break_key`] ascending.**
///
/// `f64::total_cmp` rather than `partial_cmp`: a total order over the score
/// leaves no pair of hits whose relative order is undefined, which is what
/// "same query + same generations ⇒ same ordered result" actually requires.
pub fn rank_order(left: &LexicalHit, right: &LexicalHit) -> std::cmp::Ordering {
    right
        .score
        .total_cmp(&left.score)
        .then_with(|| left.tie_break_key().cmp(&right.tie_break_key()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(text: &str) -> Vec<String> {
        tokenize(text)
    }

    #[test]
    fn a_camel_identifier_keeps_the_whole_form_and_its_parts() {
        let out = tokens("PaymentRetryPolicy");
        assert!(
            out.contains(&"paymentretrypolicy".to_string()),
            "the whole identifier must survive splitting: {out:?}"
        );
        for part in ["payment", "retry", "policy"] {
            assert!(out.contains(&part.to_string()), "missing {part}: {out:?}");
        }
    }

    #[test]
    fn snake_and_kebab_identifiers_keep_the_whole_form_and_its_words() {
        for whole in ["payment_retry_policy", "payment-retry-policy"] {
            let out = tokens(whole);
            assert!(out.contains(&whole.to_string()), "{whole}: {out:?}");
            for part in ["payment", "retry", "policy"] {
                assert!(
                    out.contains(&part.to_string()),
                    "{whole} missing {part}: {out:?}"
                );
            }
        }
    }

    #[test]
    fn a_path_scoped_identifier_keeps_its_whole_form() {
        let out = tokens("Foo::bar");
        assert!(out.contains(&"foo::bar".to_string()), "{out:?}");
        assert!(out.contains(&"foo".to_string()), "{out:?}");
        assert!(out.contains(&"bar".to_string()), "{out:?}");
    }

    #[test]
    fn a_method_and_path_keeps_the_joint_form() {
        let out = tokens("POST /payments");
        assert!(out.contains(&"post /payments".to_string()), "{out:?}");
        assert!(out.contains(&"post".to_string()), "{out:?}");
        assert!(out.contains(&"payments".to_string()), "{out:?}");
    }

    #[test]
    fn a_ticket_identifier_keeps_the_whole_form_and_its_halves() {
        let out = tokens("INC0012345");
        assert!(out.contains(&"inc0012345".to_string()), "{out:?}");
        assert!(out.contains(&"inc".to_string()), "{out:?}");
        assert!(out.contains(&"0012345".to_string()), "{out:?}");
    }

    #[test]
    fn an_acronym_prefix_breaks_before_the_word_that_follows_it() {
        let out = tokens("HTTPServer");
        assert!(out.contains(&"http".to_string()), "{out:?}");
        assert!(out.contains(&"server".to_string()), "{out:?}");
    }

    #[test]
    fn a_non_ascii_word_is_one_token_not_a_truncated_one() {
        let out = tokens("café");
        assert_eq!(out, vec!["café".to_string()], "{out:?}");
    }

    #[test]
    fn query_terms_are_distinct_and_ordered() {
        assert_eq!(
            query_terms("retry retry policy"),
            vec!["policy".to_string(), "retry".to_string()]
        );
    }

    #[test]
    fn a_term_in_every_unit_never_scores_below_zero() {
        let corpus = Bm25Corpus {
            units: 10,
            average_length: 5.0,
        };
        assert!(bm25_contribution(corpus, 10, 3, 5) >= 0.0);
    }

    #[test]
    fn a_rarer_term_outscores_a_common_one_at_equal_frequency() {
        let corpus = Bm25Corpus {
            units: 100,
            average_length: 10.0,
        };
        assert!(bm25_contribution(corpus, 1, 2, 10) > bm25_contribution(corpus, 50, 2, 10));
    }
}
