//! Atlas — the daemon-owned world-intelligence store (A1 §1–§8).
//!
//! A second analytical database, in its own file, sitting beside the
//! journal-derived operations projection ([`crate::runtime::analytics`]) and
//! deliberately **not** inside it. This module tree is the whole of Atlas;
//! [`db`] is the whole of its database access.
//!
//! # One owner, per database — two invariants, never one (F2)
//!
//! The operations projection has exactly one owning file
//! ([`crate::runtime::analytics`]) and Atlas has exactly one owning file
//! ([`db`]). Those are two separate invariants over two separate database
//! files, and they are enforced by two separate structural tests
//! (`tests/m5_projections.rs`'s `t2_the_duckdb_file_has_exactly_one_owner`
//! and `tests/x1_atlas_substrate.rs`'s `atlas_database_has_exactly_one_owner`).
//! They are never collapsed into one "these two files may both touch a
//! database driver" union rule: a union rule would let either owner drift
//! into the other's file and still pass.
//!
//! Every sibling module under `runtime/atlas/` is therefore plain Rust —
//! structs, parsing, pure functions over bytes and rows — and hands [`db`]
//! owned values to write. No sibling names the database driver crate, and
//! [`db`] hands no live connection back out.
//!
//! # The two rebuild disciplines (F1) — read this before adding a table
//!
//! Atlas and the operations projection do **not** share a rebuild story, and
//! the most likely way to break Atlas is to assume they do.
//!
//! * **Operations tables (`ops.*`, in the other file)** are disposable.
//!   The daemon deletes that file and re-folds it from the journal on every
//!   start; "delete the file and restart" and "restart" are the same
//!   operation. Nothing may come to depend on state that only lives there.
//!
//! * **Atlas's `source.*`, `git.*` and `meta.coverage` tables PERSIST
//!   across restarts.** They are not a function of the journal — no replay
//!   reproduces them. They are derived from source bytes plus the identity
//!   of the extractor that read those bytes, keyed by SourceGeneration. A
//!   generation is evicted only when the underlying source bytes changed,
//!   and an eviction is reported: it leaves an explicit "generation evicted"
//!   coverage row rather than a silent gap (ruling §4, contract §15–§16).
//!   The journal still carries the authoritative trail as one compact
//!   `source.scanned` summary per completed scan; the unit-level detail
//!   lives here.
//!
//! So: deleting Atlas's file is not free and is not a no-op. It is
//! re-derivable by re-scanning every declared source, which costs what the
//! sources cost, and coverage will say the derived evidence was lost. Do not
//! write code — or a test, or a cleanup path — that treats this file the way
//! the operations projection's file may be treated.
//!
//! # Scope of what exists here today
//!
//! The four namespaces, and the seven tables three walks now write:
//! `source.generations`, `source.files`, `source.units`, `source.symbols`,
//! `source.occurrences`, `source.edges` and `meta.coverage`. Local knowledge,
//! an estate repository at an admission-pinned commit, and a Work surface
//! overlaid on its base all land in the same tables, because they produce the
//! same kind of fact — a resource, its content identity, its extractors, and
//! what those extractors derived — and the source kind is a column, not a
//! schema.
//!
//! # Two extractions of one resource, never one ambiguous extraction (X3b)
//!
//! A path may be claimed by two routing tables at once: [`text`]'s
//! structure-unit extractor and [`syntax`]'s grammar. A Markdown file is
//! claimed by both. That is not a conflict, because F7 keys a derived row on
//! content identity **plus extractor identity** — so one blob read two ways is
//! two extractions with two keys, and `source.units` and
//! `source.occurrences`/`source.edges` are keyed independently. A structure
//! unit's key is unchanged by a grammar bump, so its extraction stays reusable
//! across the re-derivation, which is the whole point of the second key input.
//!
//! **The second key input decides staleness too, not only addressing.** A
//! generation's own `content_key` is content-only by construction, so it cannot
//! notice a parser that changed under unchanged bytes; on its own it would let
//! a re-scan answer "unchanged" after a grammar upgrade and serve the previous
//! parser's rows indefinitely. So [`db::AtlasDb::stage_scan`] compares the
//! standing generation's stored extractor identities against the ones the scan
//! actually ran, and treats a mismatch as a change: a new generation is staged,
//! and ruling §4's eviction takes the old one with its rows — leaving an
//! eviction row that says the extractors changed rather than claiming the
//! source bytes did.
//!
//! [`scan::claims_for`] is the one place the two tables are unioned, and
//! [`scan::extract_resource`] the one place a resource is extracted, for all
//! three walks.
//!
//! `git` and `context` are still empty namespaces, and deliberately so even
//! now that estate-git bytes are indexed: the `git.*` namespace is for
//! *history* facts (commits, authorship, churn), which nothing in this build
//! derives. The empty-table refusal doctrine the operations projection states
//! applies — a table that can only ever answer "zero rows" is a false promise,
//! not completeness — so every table still lands in the wave that lands its
//! writer.
//!
//! ```text
//! deny    ── pure predicate over a path (F10, the acquisition boundary)
//! text    ── pure functions over bytes, and the structure-unit routing table
//! syntax  ── the same, for symbols/imports: one grammar per claimed
//!            language, and the symbol routing table (X3b)
//! scan    ── the walk: filesystem in, plain Rust out; no DB, no journal
//! git     ── the same, from a pinned commit's Git objects (X3a)
//! overlay ── a Work surface's changes, over a base tree (X3a)
//! db      ── the one module that reaches the database driver, in or out
//! record  ── the thin three-step glue F1's crash window is stated over
//! lane    ── the thin F6 glue: an intelligence permit and the blocking pool
//! ```
//!
//! The dependency arrows run strictly downward through that list. `scan`,
//! `git` and `overlay` do not import `db`; `db` does not import the journal;
//! and neither the walks nor `db` import the engine — which is what makes
//! F6's "DB-touching glue kept thin and separately reviewable" a property of
//! the module graph rather than a promise in a comment. `record` and `lane`
//! are the two thin files at the bottom, and they are thin because everything
//! above them is a pure function they merely call.
//!
//! (This file, like every sibling, is forbidden by
//! `tests/x1_atlas_substrate.rs` from even *naming* the driver crate, so it
//! says "the database driver" where a reader might expect the crate's own
//! name. That is the one-owner rule biting its own documentation, which is
//! the correct direction for it to bite.)

pub mod db;
pub mod deny;
pub mod git;
pub mod lane;
pub mod overlay;
pub mod record;
pub mod scan;
pub mod syntax;
pub mod text;
