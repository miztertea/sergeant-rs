//! Tabular datasets: the routing table, the F10a column allowlist, and row
//! identity — all of it plain Rust (X4).
//!
//! This module names no database driver and opens no file. It answers three
//! questions and nothing else:
//!
//! ```text
//! format_for      is this path a tabular dataset, and in which format?
//! ContextFields   which of its columns may become text, if any? (F10a)
//! row_keys        what is a row called, and is that name honest?
//! ```
//!
//! The reading itself is [`super::db`]'s, because a dataset is read **in
//! place** — DuckDB opens the file the estate already has, through a canned
//! parameterized query, and no copy of those bytes lands in Atlas. That is the
//! whole reason this file is so small: for a Markdown file the extractor is
//! Rust code over bytes ([`super::text`]), and for a dataset the extractor is a
//! SQL reader inside the one module allowed to name it.
//!
//! # F10a — the column allowlist, default NONE
//!
//! F10's path/file deny set governs *acquisition*: which bytes are read at
//! all. It cannot govern a tabular source, because the interesting unit is a
//! column, not a file — a CSV of support tickets is a perfectly ordinary
//! knowledge source whose `email` column is not. So exposure has its own
//! control, at its own granularity, and it fails closed: **a dataset with no
//! declared `context_fields` produces no text units at all.** Not "all columns
//! by default", not "the columns that look safe" — none, until an operator
//! writes them down in the manifest.
//!
//! Registration is not exposure. A dataset with no allowlist is still
//! discovered, still registered, still counted, still profiled in aggregate
//! (counts, distinct values). What it does not do is turn a row's text into a
//! retrievable unit, because that is the thing that would carry a value out of
//! the file and into a context window.
//!
//! **The allowlist rides in the extractor identity** ([`reader_identity`]),
//! which is not decoration. F7 keys a derived row on content identity plus
//! extractor identity, and [`super::db::AtlasDb::stage_scan`] already treats a
//! changed extractor identity as a changed world: a new generation is staged
//! and the old one evicted with its rows. Folding the allowlist in there means
//! *narrowing* an allowlist deletes the units the wider one produced, on the
//! next scan, through machinery that already existed — rather than through a
//! bespoke retraction path that would have to be remembered.
//!
//! # Row identity: stable where the data permits, honestly re-keyed where not
//!
//! A row has no OID and usually no primary key this build could trust. So a
//! row's name is derived from what the row actually *is*: BLAKE3 over the
//! allowlisted `(column, value)` pairs, in allowlist order
//! ([`content_row_key`]). Two scans of an unedited file agree; a row that moves
//! from line 40 to line 12 because something above it was deleted keeps its
//! name, which is exactly the stability an S5 consumer needs.
//!
//! Where the data does *not* permit it, that is said out loud rather than
//! papered over. If two rows project to identical allowlisted values they have
//! identical content and cannot be told apart by it — so **every** row in that
//! colliding group is re-keyed with its ordinal and marked
//! [`RowKeyBasis::ContentAndOrdinal`]. Not just the second one: re-keying only
//! the duplicate would make the first row's identity depend on which of them
//! the walk happened to reach first, which is a stability claim the file does
//! not support. A consumer reading `key_basis` knows whether a row's name will
//! survive an insertion above it.
//!
//! The key is computed over the **allowlisted projection only**, never the
//! whole row. A hash of a denied column's value is still derived from bytes
//! F10a said do not leave the file, and identity is not a loophole in an
//! exposure rule.

use std::collections::BTreeMap;
use std::path::Path;

/// The tabular formats this build reads in place (F4).
///
/// One variant per DuckDB reader, because that is the actual boundary: the
/// three readers are three different table functions with three different
/// canned queries, and nothing here pretends they are interchangeable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DatasetFormat {
    /// Delimiter-separated text, read by DuckDB's core CSV reader (no feature
    /// flag: it has always been in core).
    Csv,
    /// JSON — an array of objects, or one object per line. F4's `json`
    /// feature.
    Json,
    /// Apache Parquet. F4's `parquet` feature.
    Parquet,
}

impl DatasetFormat {
    /// The stable wire/DB spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Json => "json",
            Self::Parquet => "parquet",
        }
    }

    /// The inverse of [`Self::as_str`]; `None` for a spelling this build does
    /// not know — a row written by a newer version is reported unreadable,
    /// never guessed at (the rule every other Atlas vocabulary follows).
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "csv" => Some(Self::Csv),
            "json" => Some(Self::Json),
            "parquet" => Some(Self::Parquet),
            _ => None,
        }
    }

    /// Extensions routed to this reader.
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Self::Csv => &["csv", "tsv"],
            Self::Json => &["json", "jsonl", "ndjson"],
            Self::Parquet => &["parquet"],
        }
    }

    /// Every format, in declaration order.
    pub const ALL: &'static [DatasetFormat] = &[
        DatasetFormat::Csv,
        DatasetFormat::Json,
        DatasetFormat::Parquet,
    ];

    /// The reader's base identity, before F10a's allowlist is composed in.
    ///
    /// Versioned in the string for the same reason
    /// [`super::text::MARKDOWN_EXTRACTOR`] is: changing what the reader
    /// derives means bumping this, which moves every key derived from it,
    /// which is what makes a re-extraction happen instead of a stale reuse.
    pub fn reader_version(self) -> &'static str {
        match self {
            Self::Csv => "csv/v1",
            Self::Json => "json/v1",
            Self::Parquet => "parquet/v1",
        }
    }
}

impl std::fmt::Display for DatasetFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The format claimed for a path, or `None` for one no reader claims.
///
/// Extension-driven and nothing else, exactly like
/// [`super::text::extractor_for`] and [`super::syntax::language_for`] — and
/// disjoint from both of them, which
/// [`tests/x4_tabular_map.rs`](../../../../tests/x4_tabular_map.rs) pins: a
/// path claimed by two routing tables that produce *different kinds of row*
/// would make `source.files` and `source.datasets` disagree about what a
/// resource is.
pub fn format_for(relative: &str) -> Option<DatasetFormat> {
    let extension = Path::new(relative)
        .extension()
        .and_then(|e| e.to_str())?
        .to_ascii_lowercase();
    DatasetFormat::ALL
        .iter()
        .copied()
        .find(|format| format.extensions().contains(&extension.as_str()))
}

/// **F10a's column allowlist**: the operator-declared columns of one source
/// whose text may become context units. Empty means none, and empty is the
/// default.
///
/// A newtype rather than a bare `Vec<String>` on purpose. The defaulting rule
/// is the whole control, and a bare vector's `Default` is indistinguishable
/// from "not filled in yet" at every call site that takes one; this type's
/// only constructor names what it is holding, and [`Self::exposes_nothing`]
/// reads as the refusal it implements.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContextFields {
    columns: Vec<String>,
}

impl ContextFields {
    /// The columns an operator declared, in declaration order.
    ///
    /// Order is preserved rather than sorted: it is the order a rendered unit
    /// puts its fields in, and an operator who wrote `["title", "body"]` meant
    /// that order. Duplicates and empty names are dropped — they name no
    /// second column and would only make the identity string longer.
    pub fn declared(columns: &[String]) -> Self {
        let mut seen = std::collections::BTreeSet::new();
        Self {
            columns: columns
                .iter()
                .map(|column| column.trim().to_string())
                .filter(|column| !column.is_empty() && seen.insert(column.clone()))
                .collect(),
        }
    }

    /// F10a's default: no column is exposed.
    pub fn none() -> Self {
        Self::default()
    }

    /// The declared columns, in declaration order.
    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    /// **The refusal itself**: true when this source exposes no tabular text
    /// at all, which is what an undeclared `context_fields` means.
    pub fn exposes_nothing(&self) -> bool {
        self.columns.is_empty()
    }

    /// Whether one column name is allowed through. Exact match, case
    /// sensitive: a CSV header is data, and folding case here would let
    /// `Email` through an allowlist that named `email`.
    pub fn allows(&self, column: &str) -> bool {
        self.columns.iter().any(|allowed| allowed == column)
    }
}

/// The reader identity for a format under one allowlist — F7's second key
/// input for a tabular extraction.
///
/// With no allowlist this is just the reader version, because nothing about
/// the extraction depends on a list that is empty. With one, the columns are
/// part of the identity, so a changed allowlist is a changed extraction and
/// [`super::db::AtlasDb::stage_scan`]'s existing staleness test supersedes the
/// generation (see this module's doc).
pub fn reader_identity(format: DatasetFormat, fields: &ContextFields) -> String {
    if fields.exposes_nothing() {
        return format.reader_version().to_string();
    }
    format!(
        "{}+context({})",
        format.reader_version(),
        fields.columns().join(",")
    )
}

/// One registered tabular dataset, as the walk found it. Plain data — the
/// bytes were never read into this struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedDataset {
    /// Path relative to the source root, `/`-separated.
    pub relative_path: String,
    /// The reader that claims it.
    pub format: DatasetFormat,
    /// BLAKE3 hex of the file's bytes (F7's content half), streamed rather
    /// than held: a dataset is read in place and never loaded whole.
    pub content_hash: String,
    /// The reader identity under this source's allowlist
    /// ([`reader_identity`]).
    pub reader: String,
    /// F7's reusable extraction key for this dataset.
    pub dataset_key: String,
    /// Size in bytes, as the filesystem reported it.
    pub byte_len: u64,
    /// Modification time in Unix milliseconds — **a change hint only**, part
    /// of no key, exactly as [`super::scan::ScannedFile::mtime_millis`] is.
    pub mtime_millis: Option<i64>,
}

/// How a row's identity was arrived at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowKeyBasis {
    /// Derived from the row's allowlisted values alone. Survives insertions,
    /// deletions and reordering above it.
    Content,
    /// The allowlisted values were not unique in this dataset, so the row's
    /// position was folded in to tell it from its twins. Stable only while
    /// nothing above it moves — said plainly, because a consumer that assumed
    /// otherwise would silently re-associate two rows.
    ContentAndOrdinal,
}

impl RowKeyBasis {
    /// The stable wire/DB spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Content => "content",
            Self::ContentAndOrdinal => "content+ordinal",
        }
    }

    /// The inverse of [`Self::as_str`].
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "content" => Some(Self::Content),
            "content+ordinal" => Some(Self::ContentAndOrdinal),
            _ => None,
        }
    }
}

/// One row's context unit: its name, how honest that name is, and the text the
/// allowlist let through.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowUnit {
    /// Position in the dataset, zero-based, in the reader's own row order.
    pub ordinal: u64,
    /// The row's name (see [`content_row_key`]).
    pub row_key: String,
    /// Whether that name is content-derived or had to include the ordinal.
    pub basis: RowKeyBasis,
    /// The allowlisted columns that actually appeared in this dataset, in
    /// allowlist order — the audit trail of what was exposed.
    pub fields: Vec<String>,
    /// The rendered text: one `column: value` line per allowlisted field.
    pub text: String,
}

/// BLAKE3 over one row's allowlisted `(column, value)` pairs.
///
/// Domain-separated, like every other key in
/// [`crate::domain::source`], so a row key can never collide with a content
/// hash or a generation key that happened to hash similar strings.
pub fn content_row_key(
    source_name: &str,
    relative_path: &str,
    fields: &[(String, String)],
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"sergeant.atlas.row-key/v1\n");
    hasher.update(source_name.as_bytes());
    hasher.update(b"\0");
    hasher.update(relative_path.as_bytes());
    hasher.update(b"\n");
    for (column, value) in fields {
        hasher.update(column.as_bytes());
        hasher.update(b"\0");
        hasher.update(value.as_bytes());
        hasher.update(b"\n");
    }
    hasher.finalize().to_hex().to_string()
}

/// Fold a row's ordinal into its content key, for a row whose allowlisted
/// values are not unique in its dataset.
pub fn ordinal_row_key(content_key: &str, ordinal: u64) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"sergeant.atlas.row-key-ordinal/v1\n");
    hasher.update(content_key.as_bytes());
    hasher.update(b"\n");
    hasher.update(ordinal.to_string().as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// Turn bounded rows read in place into context units — **the F10a gate, and
/// the only function that produces tabular text.**
///
/// `columns` are the dataset's own column names in reader order; `rows` are
/// values already stringified by the reader, aligned with `columns`.
///
/// Returns an empty vector when the allowlist is empty. That is the refusal,
/// and it is checked first, before anything is rendered or hashed: a caller
/// that forgot to pass an allowlist gets no units rather than all of them.
/// A declared column that this dataset does not have is simply absent from
/// [`RowUnit::fields`] — an allowlist is permission, not a schema assertion,
/// and a source whose CSV lost a column should stop exposing it rather than
/// fail to scan.
pub fn row_units(
    source_name: &str,
    relative_path: &str,
    fields: &ContextFields,
    columns: &[String],
    rows: &[Vec<String>],
) -> Vec<RowUnit> {
    if fields.exposes_nothing() {
        return Vec::new();
    }
    // Allowlist order, restricted to columns this dataset actually has. The
    // index is resolved once rather than per row.
    let projected: Vec<(usize, String)> = fields
        .columns()
        .iter()
        .filter_map(|wanted| {
            columns
                .iter()
                .position(|column| column == wanted)
                .map(|index| (index, wanted.clone()))
        })
        .collect();
    if projected.is_empty() {
        return Vec::new();
    }
    let exposed: Vec<String> = projected.iter().map(|(_, name)| name.clone()).collect();

    let mut units: Vec<RowUnit> = Vec::with_capacity(rows.len());
    let mut occurrences: BTreeMap<String, u64> = BTreeMap::new();
    for (ordinal, row) in rows.iter().enumerate() {
        let pairs: Vec<(String, String)> = projected
            .iter()
            .map(|(index, name)| (name.clone(), row.get(*index).cloned().unwrap_or_default()))
            .collect();
        let key = content_row_key(source_name, relative_path, &pairs);
        *occurrences.entry(key.clone()).or_insert(0) += 1;
        units.push(RowUnit {
            ordinal: ordinal as u64,
            row_key: key,
            basis: RowKeyBasis::Content,
            fields: exposed.clone(),
            text: pairs
                .iter()
                .map(|(column, value)| format!("{column}: {value}"))
                .collect::<Vec<_>>()
                .join("\n"),
        });
    }
    // The honest re-key. Every member of a colliding group moves, not just the
    // later ones — see this module's doc for why the first row's identity may
    // not depend on walk order.
    for unit in &mut units {
        if occurrences.get(&unit.row_key).copied().unwrap_or(0) > 1 {
            unit.row_key = ordinal_row_key(&unit.row_key, unit.ordinal);
            unit.basis = RowKeyBasis::ContentAndOrdinal;
        }
    }
    units
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| v.to_string()).collect()
    }

    /// The routing table is by extension and claims exactly the three
    /// families, case-insensitively.
    #[test]
    fn the_routing_table_claims_three_families_and_nothing_else() {
        assert_eq!(format_for("data/rows.csv"), Some(DatasetFormat::Csv));
        assert_eq!(format_for("data/rows.TSV"), Some(DatasetFormat::Csv));
        assert_eq!(format_for("events.ndjson"), Some(DatasetFormat::Json));
        assert_eq!(format_for("events.jsonl"), Some(DatasetFormat::Json));
        assert_eq!(format_for("facts.parquet"), Some(DatasetFormat::Parquet));
        assert_eq!(format_for("README.md"), None);
        assert_eq!(format_for("main.rs"), None);
        assert_eq!(format_for("no-extension"), None);
        for format in DatasetFormat::ALL {
            assert_eq!(DatasetFormat::parse(format.as_str()), Some(*format));
        }
        assert_eq!(DatasetFormat::parse("orc"), None);
    }

    /// F10a's default, at the level the type expresses it: nothing declared is
    /// nothing exposed.
    #[test]
    fn an_undeclared_allowlist_exposes_nothing() {
        let none = ContextFields::none();
        assert!(none.exposes_nothing());
        assert!(!none.allows("title"));
        assert_eq!(ContextFields::declared(&[]), none);
        assert_eq!(
            row_units(
                "notes",
                "rows.csv",
                &none,
                &strings(&["title"]),
                &[strings(&["hello"])]
            ),
            Vec::new()
        );
    }

    /// An allowlist is exact and ordered, and it is part of the extraction's
    /// identity — so narrowing it is a different extraction, which is what
    /// makes the units it produced evictable by machinery that already exists.
    #[test]
    fn the_allowlist_is_exact_ordered_and_part_of_the_identity() {
        let wide = ContextFields::declared(&strings(&["title", "body"]));
        let narrow = ContextFields::declared(&strings(&["title"]));
        assert!(wide.allows("title"));
        assert!(!wide.allows("Title"));
        assert_eq!(wide.columns(), strings(&["title", "body"]).as_slice());
        // Duplicates and blanks name no second column.
        assert_eq!(
            ContextFields::declared(&strings(&["title", " title ", ""])).columns(),
            strings(&["title"]).as_slice()
        );

        let base = reader_identity(DatasetFormat::Csv, &ContextFields::none());
        assert_eq!(base, "csv/v1");
        assert_ne!(reader_identity(DatasetFormat::Csv, &wide), base);
        assert_ne!(
            reader_identity(DatasetFormat::Csv, &wide),
            reader_identity(DatasetFormat::Csv, &narrow)
        );
        // Order is part of it, because it is part of the rendering.
        assert_ne!(
            reader_identity(DatasetFormat::Csv, &wide),
            reader_identity(
                DatasetFormat::Csv,
                &ContextFields::declared(&strings(&["body", "title"]))
            )
        );
    }

    /// Row identity: content-derived and position-independent where the values
    /// are unique, and honestly re-keyed — for every member of the group —
    /// where they are not.
    #[test]
    fn a_row_key_is_content_derived_until_the_data_stops_permitting_it() {
        let fields = ContextFields::declared(&strings(&["title"]));
        let columns = strings(&["title", "secret"]);
        let first = row_units(
            "notes",
            "rows.csv",
            &fields,
            &columns,
            &[
                strings(&["alpha", "a"]),
                strings(&["beta", "b"]),
                strings(&["gamma", "c"]),
            ],
        );
        assert_eq!(first.len(), 3);
        assert!(first.iter().all(|u| u.basis == RowKeyBasis::Content));
        // Only the allowlisted column is rendered; the denied one is nowhere.
        assert_eq!(first[0].text, "title: alpha");
        assert!(
            first.iter().all(|u| !u.text.contains("secret")),
            "a denied column may appear in no rendered unit"
        );
        assert_eq!(first[0].fields, strings(&["title"]));

        // Delete the first row: the survivors keep their names.
        let shifted = row_units(
            "notes",
            "rows.csv",
            &fields,
            &columns,
            &[strings(&["beta", "b"]), strings(&["gamma", "c"])],
        );
        assert_eq!(shifted[0].row_key, first[1].row_key);
        assert_eq!(shifted[1].row_key, first[2].row_key);

        // Two rows the allowlist cannot tell apart: BOTH are re-keyed, and
        // both say so.
        let twins = row_units(
            "notes",
            "rows.csv",
            &fields,
            &columns,
            &[
                strings(&["alpha", "a"]),
                strings(&["alpha", "different"]),
                strings(&["beta", "b"]),
            ],
        );
        assert_eq!(twins[0].basis, RowKeyBasis::ContentAndOrdinal);
        assert_eq!(twins[1].basis, RowKeyBasis::ContentAndOrdinal);
        assert_ne!(twins[0].row_key, twins[1].row_key);
        assert_eq!(twins[2].basis, RowKeyBasis::Content);
        assert_eq!(twins[2].row_key, first[1].row_key);
        for basis in [RowKeyBasis::Content, RowKeyBasis::ContentAndOrdinal] {
            assert_eq!(RowKeyBasis::parse(basis.as_str()), Some(basis));
        }
    }

    /// An allowlist naming a column the dataset does not have exposes the ones
    /// it does, and one naming none of them exposes nothing.
    #[test]
    fn an_allowlist_is_permission_not_a_schema_assertion() {
        let columns = strings(&["title"]);
        let rows = vec![strings(&["alpha"])];
        let partial = row_units(
            "notes",
            "rows.csv",
            &ContextFields::declared(&strings(&["title", "absent"])),
            &columns,
            &rows,
        );
        assert_eq!(partial.len(), 1);
        assert_eq!(partial[0].fields, strings(&["title"]));

        assert!(
            row_units(
                "notes",
                "rows.csv",
                &ContextFields::declared(&strings(&["absent"])),
                &columns,
                &rows,
            )
            .is_empty()
        );
    }
}
