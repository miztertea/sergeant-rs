//! Atlas — the daemon-owned world-intelligence store (A1 §1–§8).
//!
//! The estate's **one** analytical database, in its own file, carrying A1
//! §5's five logical schemas. This module tree is the whole of Atlas; [`db`]
//! is the whole of its database access — and, since S5 W1c, the whole of the
//! crate's database access.
//!
//! # One database, one owner, one invariant (F2, A1 §5)
//!
//! Until S5 W1c there were two physical files: the operations projection's
//! own file under `projections/` (`ops.*`) and Atlas's file under `atlas/`
//! (`meta`/`source`/`git`/`context`). Each had one
//! owning file and each was pinned by its own structural test, and those two
//! tests were deliberately never collapsed into a "these two files may both
//! touch a database driver" union rule, because a union rule lets either
//! owner drift into the other's file and still pass.
//!
//! A1 §5 declares one physical file — Atlas's, named by [`db::ATLAS_DB_FILE`]
//! — with schemas `meta`,
//! `ops`, `source`, `git`, `context`, and decision A1-02's rationale is
//! "schemas provide separation without more databases". The second database
//! was a deviation the implementing wave ratified for itself; the owner
//! correction of 2026-08-29 settled that the code converges to the contract.
//! So the second file is gone, `ops` is a schema in this database, and the
//! pair of one-owner tests has become the single assertion the contract
//! implies: `tests/x1_atlas_substrate.rs`'s
//! `atlas_database_has_exactly_one_owner`, scanning the whole of `src/` with
//! no exempted tree. That is the *stronger* rule, not a merged weaker one —
//! the union rule remains forbidden and is now unrepresentable.
//!
//! Every sibling module under `runtime/atlas/` is therefore plain Rust —
//! structs, parsing, pure functions over bytes and rows — and hands [`db`]
//! owned values to write. No sibling names the database driver crate, and
//! [`db`] hands no live connection back out.
//!
//! # The two rebuild disciplines (F1) — read this before adding a table
//!
//! One file, two disciplines. They did not merge when the files did, and the
//! most likely way to break Atlas is to assume they did.
//!
//! * **Operations tables (`ops.*`) are disposable.** The daemon drops the
//!   `ops` schema and re-folds it from the journal on every start
//!   ([`db::Analytics::begin_rebuild`]), so "restart" is still the only
//!   population path and nothing may come to depend on state that only lives
//!   there. It drops a *schema*, never the file.
//!
//! * **`source.*`, `git.*` and `meta.coverage` PERSIST across restarts.**
//!   They are not a function of the journal — no replay reproduces them. They
//!   are derived from source bytes plus the identity of the extractor that
//!   read those bytes, keyed by SourceGeneration. A generation is evicted only
//!   when the underlying source bytes changed, and an eviction is reported: it
//!   leaves an explicit "generation evicted" coverage row rather than a silent
//!   gap (ruling §4, contract §15–§16). The journal still carries the
//!   authoritative trail as one compact `source.scanned` summary per completed
//!   scan; the unit-level detail lives here.
//!
//! # What deleting Atlas's database file costs now
//!
//! It used to be true, of the operations projection's own file, that
//! deleting it lost
//! nothing. That sentence does not transfer, and this is the one place it is
//! stated rather than assumed.
//!
//! Deleting Atlas's database file and restarting **rebuilds `ops` exactly** —
//! every
//! row of it comes back from the journal — and **discards every persisted
//! source generation**, which must be re-scanned at whatever the sources
//! cost. That is acceptable under ruling §4 only because it is reported and
//! not silent: a store with no confirmed generation says so
//! (`sgt doctor`'s atlas row, `sgt intelligence status`), and re-scanning
//! writes fresh coverage. It is not a maintenance step, it is data loss with
//! a recovery path. Do not write code — or a test, or a cleanup path — that
//! deletes this file to "fix" the operations tables; the supported operation
//! for those is a restart.
//!
//! # Scope of what exists here today
//!
//! Of A1 §5's five namespaces, `ops` is folded from the journal (see [`db`])
//! and `context` is still empty. Atlas's own derived tables are the seven
//! three walks now write:
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
//! office  ── the same, for Office documents (S4 Y2, G3): the one module
//!            that may name the third-party document-conversion crate it
//!            adopts, pinned structurally by `tests/y2_office_boundary.rs`
//!            — bytes in, our own Document/Section vocabulary out, no type
//!            of that crate crossing the boundary
//! syntax  ── the same, for symbols/imports: one grammar per claimed
//!            language, and the symbol routing table (X3b)
//! tabular ── the dataset routing table, F10a's column allowlist, and row
//!            identity — all pure; the *reading* is db's (X4)
//! scan    ── the walk: filesystem in, plain Rust out; no DB, no journal
//! git     ── the same, from a pinned commit's Git objects (X3a)
//! overlay ── a Work surface's changes, over a base tree (X3a)
//! db      ── the one module that reaches the database driver, in or out
//! record  ── the thin three-step glue F1's crash window is stated over
//! lane    ── the thin F6 glue: an intelligence permit and the blocking pool
//! worker  ── the supervised parse-worker transport (S4 Y1, G2): spawn,
//!            kill+reap, and the daemon-side AUTHORITY over what a worker
//!            returns — no DB, no journal, independently testable
//! ```
//!
//! # A dataset is read in place, and that inverts one thing (X4)
//!
//! Every other extractor in this tree is Rust over bytes the walk already
//! read. A tabular dataset is not: the walk registers it (path, streamed
//! content hash, size) and never opens a row, and the reading happens later,
//! **in place** — DuckDB opens the operator's own CSV/JSON/Parquet file
//! through a canned parameterized query, bounded by a LIMIT and a row cap
//! (F12), and no copy of those bytes lands in Atlas.
//!
//! That means the extractor for a dataset lives inside [`db`], because the
//! reader *is* the database, and the one-owner rule says only [`db`] may name
//! it. [`tabular`] holds everything about datasets that is not the read
//! itself: which paths are datasets, F10a's column allowlist, and how a row is
//! named. The result of a read is stored as derived evidence carrying its
//! input generation, the identity of the query that produced it, and a hash of
//! its own output (A1 §6.4) — so an answer can be checked rather than
//! trusted.
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

pub mod archive;
pub mod db;
pub mod deny;
pub mod external_git;
pub mod git;
pub mod lane;
pub mod lexical;
pub mod locator;
pub mod mail;
pub mod office;
pub mod overlay;
pub mod record;
pub mod scan;
pub mod semantic;
pub mod syntax;
pub mod tabular;
pub mod text;
pub mod worker;
