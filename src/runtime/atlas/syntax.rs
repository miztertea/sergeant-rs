//! Syntax-derived symbols and imports — **pure functions over bytes** (F6's
//! adapter-shape mandate), one tree-sitter grammar per language this build
//! claims.
//!
//! Nothing here opens a file, touches a database, reads a clock, or knows a
//! source exists. [`extract`] takes a language and bytes and returns owned
//! facts. Same property, same reason, as [`super::text`]: extraction stays
//! reviewable and unit-testable without a daemon, and the DB glue that will
//! carry these rows adds nothing a reader has to take on trust.
//!
//! # Syntax, not semantics (A1-09)
//!
//! Every label here is what the *grammar* called the node — `function`,
//! `struct`, `class`, `heading`. Nothing resolves a name to a definition,
//! follows a re-export, or decides which `count` a call meant. An [`Import`]'s
//! `target` is the text the file wrote, not a resolved module. That is the
//! whole claim, and it is deliberately smaller than the claim a language
//! server makes: these rows are evidence about syntax, and a consumer that
//! treats them as a resolved symbol graph is reading a promise this module
//! never made.
//!
//! # Two routing tables, not one overriding the other
//!
//! [`super::text::extractor_for`] routes bytes to a *structure-unit*
//! extractor (Document/Section spans, for retrieval). [`language_for`] here
//! routes bytes to a *symbol* extractor. A Markdown file is claimed by both,
//! and that is not a conflict: F7 keys derived rows on content identity **plus
//! extractor identity**, so two extractors over one blob are two extractions
//! with two keys, never one extraction that could have been done two ways.
//! The invariant `text` states — identical bytes plus identical extractor
//! identity are one extraction — is preserved exactly.
//!
//! # Provenance is byte offsets into the original
//!
//! Every [`Symbol`] and [`Import`] carries `byte_start`/`byte_end` into the
//! bytes it was extracted from, so A1 §3's provenance question is answered by
//! slicing the original (A1-12), same rule as [`super::text`].
//!
//! # A parse error is an error, never a partial answer
//!
//! tree-sitter is error-tolerant by design: given malformed input it returns a
//! tree with `ERROR` nodes and whatever it could still recognize. This module
//! refuses that middle ground — [`extract`] returns [`SyntaxError::Parse`] for
//! any tree containing an error or a missing node, and returns **no** partial
//! symbol list alongside it. A file that parsed badly is coverage-reported as
//! `error` (F8); it is never reported as indexed-with-fewer-symbols, because
//! "fewer symbols" and "all the symbols" are indistinguishable to every
//! downstream consumer. F5's corpus gate is the standing proof that this path
//! is reachable rather than vacuous.
//!
//! # G2's revisit trigger, answered (S4 Y5)
//!
//! G2 kept this module in-process as a **named accepted risk** — tree-sitter
//! is generated C grammars fed arbitrary bytes, squarely §12's
//! malformed-input class, held in-process anyway on the stated predicate
//! "inputs are local/estate-owned rather than attacker-chosen" — with a
//! revisit trigger: *the first syntax-lane crash, or the first external-git
//! source feeding it remote bytes (Y5), whichever comes first.* Y5 is
//! exactly the second condition, and this is that decision, made explicitly
//! rather than left to pass silently.
//!
//! **Decided: stays in-process for this wave.** Not because the predicate
//! still holds — it does not, and pretending otherwise would be the exact
//! erosion the trigger exists to catch — but because every mitigating fact
//! the predicate's replacement rests on is independently true and already
//! enforced, upstream of this module, on the identical code path external-git
//! bytes now share with every other source kind:
//!
//! 1. **The bytes that reach [`extract`] are never raw remote bytes.**
//!    [`super::git::extract_blobs`] — the *one* function this module's caller
//!    shares across `estate_git` and `external_git` alike (S4 Y5's reuse: an
//!    external source reads through the identical `list_tree`/`extract_blobs`
//!    an estate mount always has) — refuses anything over
//!    [`super::scan::MAX_RESOURCE_BYTES`] and anything that fails
//!    [`super::text::as_text`]'s UTF-8 check *before* a byte reaches an
//!    extractor of any kind. What lands here is therefore always
//!    size-bounded, valid-UTF-8 text — the same envelope every `estate_git`
//!    and `local_knowledge` byte already had, not a wider one an external
//!    source gets to skip.
//! 2. **tree-sitter's own design goal is robustness to adversarial-looking
//!    input, not merely well-formed input** — it is built to re-read
//!    syntactically-invalid source on every keystroke of a live editing
//!    session, which is a stronger adversarial-input posture than a
//!    general-purpose format parser (the ttf-parser/docx class §12's own
//!    rationale names) ever claims. That is a property of the parser, not of
//!    who supplied the bytes, so it does not weaken when the byte's origin
//!    changes.
//! 3. **The crash record is over the identical code path, not a proxy for
//!    it.** `estate_git` bytes have run through this exact module since X3b
//!    with zero crashes across S3's and S4's own corpora — external-git
//!    reuses that path byte-for-byte rather than adding a new one, so there
//!    is no new code here for a new risk to hide in.
//! 4. **The git subprocess that acquires external bytes is itself supervised**
//!    (G2's own amendment, [`crate::runtime::git::git_fetch_restricted`]) —
//!    a defense that does not reach this module's own risk class but does
//!    mean the *acquisition* half of "attacker-chosen bytes" already sits
//!    behind a bounded, killable process boundary before extraction ever
//!    starts.
//!
//! **What was NOT done, and why not this wave.** Moving syntax extraction
//! worker-side for external-git bytes specifically — the trigger's other
//! named option — was considered and set aside rather than attempted: it
//! would mean splitting [`super::scan::extract_resource`]'s single call into
//! two different execution shapes keyed on source kind (in-process for
//! `estate_git`/`local_knowledge`, worker-routed for `external_git`), a real
//! architecture change to the one place every source kind currently shares,
//! attempted at the tail of an already-large wave (locator allowlist, host
//! cache, provenance, package identity, the scan trigger, this doctrine
//! amendment). A change to a shared extraction path deserves its own
//! reviewed wave, not a rider on this one's remaining budget.
//!
//! **The trigger is re-armed, not spent.** G2's original OR had two legs;
//! the external-git leg is answered here, not struck. What remains standing
//! — verbatim — is *the first syntax-lane crash*, and it is now armed with
//! genuinely attacker-influenced bytes flowing through it for the first
//! time, which is a strictly sharper test of the same trigger than S3 ever
//! ran. The worker-side move stays the obvious next step the day that
//! trigger fires, or the day a consumer of external-git syntax data
//! (S5+) makes the cost of moving it worth paying up front.

use tree_sitter::{Node, Parser};

/// Version component of every extractor identity in this module (F7's second
/// cache-key input).
///
/// Bump this when what [`extract`] produces changes — a new node kind in a
/// symbol table, a different name rule, a changed label. Bumping it changes
/// every derived key, which is what makes a re-extraction happen instead of a
/// stale reuse.
pub const SYNTAX_EXTRACTOR_VERSION: &str = "v1";

/// A language this build claims for symbol extraction.
///
/// Exactly the six families F5 names, and no more. `.tsx` is deliberately
/// absent: it needs the TSX grammar rather than the TypeScript one, and a
/// language with no fixture in the corpus would be a claim nothing checks —
/// so a `.tsx` file routes to `None` and is honestly coverage-reported
/// `unsupported` (F8) rather than parsed by an almost-right grammar.
///
/// `.jsx`/`.mjs`/`.cjs` sit on the other side of that line: they ARE
/// claimed by the JavaScript family, so a file whose syntax the JS grammar
/// cannot parse (real JSX above all) reports `error` coverage with zero
/// symbols — a claimed-but-failed parse, not an unclaimed extension. That
/// is F8's honest-error rule doing its job, but it makes JavaScript's
/// claimed surface broader than what the grammar parses cleanly; narrowing
/// `.jsx` out (or adopting the TSX grammar) is a later wave's call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyntaxLanguage {
    Rust,
    Toml,
    Markdown,
    Python,
    JavaScript,
    TypeScript,
    Bash,
}

impl SyntaxLanguage {
    /// Every claimed language, in a stable order.
    pub const ALL: &'static [SyntaxLanguage] = &[
        SyntaxLanguage::Rust,
        SyntaxLanguage::Toml,
        SyntaxLanguage::Markdown,
        SyntaxLanguage::Python,
        SyntaxLanguage::JavaScript,
        SyntaxLanguage::TypeScript,
        SyntaxLanguage::Bash,
    ];

    /// The language's manifest/coverage name.
    pub fn name(self) -> &'static str {
        match self {
            SyntaxLanguage::Rust => "rust",
            SyntaxLanguage::Toml => "toml",
            SyntaxLanguage::Markdown => "markdown",
            SyntaxLanguage::Python => "python",
            SyntaxLanguage::JavaScript => "javascript",
            SyntaxLanguage::TypeScript => "typescript",
            SyntaxLanguage::Bash => "bash",
        }
    }

    /// This language's extractor identity — F7's second cache-key input.
    ///
    /// Per language, not one identity for the module: changing the Rust symbol
    /// table must not invalidate every extracted Python file.
    pub fn extractor_identity(self) -> String {
        format!("syntax-{}/{SYNTAX_EXTRACTOR_VERSION}", self.name())
    }

    /// Extensions routed to this language, lowercase, without the dot.
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            SyntaxLanguage::Rust => &["rs"],
            SyntaxLanguage::Toml => &["toml"],
            SyntaxLanguage::Markdown => &["md", "markdown"],
            SyntaxLanguage::Python => &["py"],
            SyntaxLanguage::JavaScript => &["js", "mjs", "cjs", "jsx"],
            SyntaxLanguage::TypeScript => &["ts", "mts", "cts"],
            SyntaxLanguage::Bash => &["sh", "bash"],
        }
    }

    /// Every label this language's grammar can produce, in table order.
    ///
    /// The *complete* set of what [`extract`] may put on a [`Symbol`] for this
    /// language — which is what makes A1-09 checkable downstream rather than
    /// merely asserted here: a consumer can compare the labels it stored
    /// against this and see that nothing was classified, inferred or resolved
    /// on the way. Duplicates are kept (several Rust node kinds are all
    /// `function`), because this answers "what may a label be?" and not "how
    /// many kinds are there?".
    pub fn labels(self) -> impl Iterator<Item = &'static str> {
        self.symbol_kinds().iter().map(|(_, label)| *label)
    }

    fn grammar(self) -> tree_sitter::Language {
        match self {
            SyntaxLanguage::Rust => tree_sitter_rust::LANGUAGE.into(),
            SyntaxLanguage::Toml => tree_sitter_toml_ng::LANGUAGE.into(),
            SyntaxLanguage::Markdown => tree_sitter_md::LANGUAGE.into(),
            SyntaxLanguage::Python => tree_sitter_python::LANGUAGE.into(),
            SyntaxLanguage::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            SyntaxLanguage::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            SyntaxLanguage::Bash => tree_sitter_bash::LANGUAGE.into(),
        }
    }

    /// Node kinds that are symbols, paired with the label they carry.
    ///
    /// Read this as the definition of "symbol" for the language — the corpus
    /// manifest's hand-verified counts are counts of exactly these kinds, and
    /// nothing outside this table is a symbol however much it looks like one
    /// (a `const` bound to an arrow function in JavaScript is a lexical
    /// binding; an `impl` block in Rust has no name to be a symbol of).
    fn symbol_kinds(self) -> &'static [(&'static str, &'static str)] {
        match self {
            SyntaxLanguage::Rust => &[
                ("function_item", "function"),
                ("function_signature_item", "function"),
                ("struct_item", "struct"),
                ("enum_item", "enum"),
                ("union_item", "union"),
                ("trait_item", "trait"),
                ("mod_item", "module"),
                ("const_item", "const"),
                ("static_item", "static"),
                ("type_item", "type"),
                ("macro_definition", "macro"),
            ],
            SyntaxLanguage::Toml => &[
                ("table", "table"),
                ("table_array_element", "table_array"),
                ("pair", "key"),
            ],
            SyntaxLanguage::Markdown => {
                &[("atx_heading", "heading"), ("setext_heading", "heading")]
            }
            SyntaxLanguage::Python => &[
                ("function_definition", "function"),
                ("class_definition", "class"),
            ],
            SyntaxLanguage::JavaScript => &[
                ("function_declaration", "function"),
                ("generator_function_declaration", "function"),
                ("class_declaration", "class"),
                ("method_definition", "method"),
            ],
            SyntaxLanguage::TypeScript => &[
                ("function_declaration", "function"),
                ("generator_function_declaration", "function"),
                ("class_declaration", "class"),
                ("abstract_class_declaration", "class"),
                ("method_definition", "method"),
                ("method_signature", "method"),
                ("interface_declaration", "interface"),
                ("type_alias_declaration", "type"),
                ("enum_declaration", "enum"),
            ],
            SyntaxLanguage::Bash => &[("function_definition", "function")],
        }
    }

    /// Node kinds inspected for an import.
    ///
    /// A kind listed here is a *candidate*: [`imports_in`] returns an empty
    /// list for a node that turns out not to be one (every Bash `command` is a
    /// candidate; only `source` and `.` are imports). It may also return
    /// *several* — one Python `import_statement` can name a whole comma list.
    fn import_kinds(self) -> &'static [&'static str] {
        match self {
            SyntaxLanguage::Rust => &["use_declaration", "extern_crate_declaration"],
            // Neither format has an import construct. Zero is the honest
            // answer, not an unimplemented one.
            SyntaxLanguage::Toml | SyntaxLanguage::Markdown => &[],
            SyntaxLanguage::Python => &["import_statement", "import_from_statement"],
            SyntaxLanguage::JavaScript | SyntaxLanguage::TypeScript => &["import_statement"],
            SyntaxLanguage::Bash => &["command"],
        }
    }
}

/// The language claimed for a path, or `None` for a family this build does
/// not claim (coverage: `unsupported`).
///
/// Extension-driven and nothing else, for the same reason
/// [`super::text::extractor_for`] is: content sniffing would mean reading
/// bytes to decide whether to read bytes, and routing has to be one function
/// per question or the same blob could be extracted differently depending on
/// where it was acquired.
pub fn language_for(relative: &str) -> Option<SyntaxLanguage> {
    let extension = std::path::Path::new(relative)
        .extension()
        .and_then(|e| e.to_str())?
        .to_ascii_lowercase();
    SyntaxLanguage::ALL
        .iter()
        .copied()
        .find(|language| language.extensions().contains(&extension.as_str()))
}

/// One syntax-derived symbol, positioned in the original bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    /// What the grammar called it — `function`, `struct`, `heading`, … Never
    /// a semantic classification (A1-09).
    pub label: &'static str,
    /// The name as written. Not resolved, not qualified, not deduplicated:
    /// two `count` methods in one file are two symbols named `count`.
    pub name: String,
    /// Inclusive start offset into the original bytes.
    pub byte_start: usize,
    /// Exclusive end offset into the original bytes.
    pub byte_end: usize,
}

/// One syntax-derived import, positioned in the original bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    /// The import target exactly as the file wrote it — `node:fs`,
    /// `std::collections::HashMap`, `./lib/common.sh`. Unresolved by
    /// construction.
    pub target: String,
    /// Inclusive start offset into the original bytes.
    pub byte_start: usize,
    /// Exclusive end offset into the original bytes.
    pub byte_end: usize,
}

/// Everything one extraction produced, in document order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyntaxFacts {
    pub symbols: Vec<Symbol>,
    pub imports: Vec<Import>,
}

/// Why an extraction produced nothing rather than something partial.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SyntaxError {
    /// Bytes were not valid UTF-8. Strict, like [`super::text::as_text`]: a
    /// lossy decode would shift every byte offset this module promises.
    #[error("not valid UTF-8")]
    NotUtf8,
    /// The compiled grammar was rejected by the runtime — an ABI mismatch
    /// between the `tree-sitter` crate and a grammar crate. Not reachable
    /// through a bad input; reachable through a bad dependency bump, which is
    /// exactly why it is a named variant rather than a panic.
    #[error("grammar for {language} rejected by the tree-sitter runtime: {detail}")]
    Grammar {
        language: &'static str,
        detail: String,
    },
    /// The parse produced a tree containing an error or missing node. No
    /// partial results accompany this: see the module docs.
    #[error("{language}: parse failed at byte {byte_start}")]
    Parse {
        language: &'static str,
        byte_start: usize,
    },
    /// The parser returned no tree at all (cancellation or timeout — neither
    /// is set by this call, so this is defensive).
    #[error("{language}: parser returned no tree")]
    NoTree { language: &'static str },
}

/// Extract symbols and imports from `bytes` as `language`.
///
/// Pure: no I/O, no state, no clock. Deterministic for a given
/// (language, bytes) pair — which is what lets F7 key the result on content
/// identity plus [`SyntaxLanguage::extractor_identity`] and reuse it.
pub fn extract(language: SyntaxLanguage, bytes: &[u8]) -> Result<SyntaxFacts, SyntaxError> {
    let text = std::str::from_utf8(bytes).map_err(|_| SyntaxError::NotUtf8)?;

    let mut parser = Parser::new();
    parser
        .set_language(&language.grammar())
        .map_err(|error| SyntaxError::Grammar {
            language: language.name(),
            detail: error.to_string(),
        })?;
    let tree = parser.parse(text, None).ok_or(SyntaxError::NoTree {
        language: language.name(),
    })?;

    let root = tree.root_node();
    if root.has_error() {
        return Err(SyntaxError::Parse {
            language: language.name(),
            byte_start: first_error_byte(root),
        });
    }

    let mut facts = SyntaxFacts::default();
    let symbol_kinds = language.symbol_kinds();
    let import_kinds = language.import_kinds();

    let mut cursor = root.walk();
    'walk: loop {
        let node = cursor.node();
        if node.is_named() {
            let kind = node.kind();
            if let Some((_, label)) = symbol_kinds.iter().find(|(k, _)| *k == kind) {
                if let Some(name) = symbol_name(language, node, text) {
                    facts.symbols.push(Symbol {
                        label,
                        name,
                        byte_start: node.start_byte(),
                        byte_end: node.end_byte(),
                    });
                }
            } else if import_kinds.contains(&kind) {
                // `extend`, not `push`: one candidate node can name more than
                // one import (`import os, sys`), and a signature that could
                // only answer once would drop the rest silently — the partial
                // answer this module's own doctrine refuses.
                facts.imports.extend(imports_in(language, node, text));
            }
        }

        if cursor.goto_first_child() {
            continue;
        }
        loop {
            if cursor.goto_next_sibling() {
                continue 'walk;
            }
            if !cursor.goto_parent() {
                break 'walk;
            }
        }
    }

    Ok(facts)
}

/// The first error-or-missing node's start byte, for the refusal's message.
fn first_error_byte(root: Node<'_>) -> usize {
    let mut cursor = root.walk();
    'walk: loop {
        let node = cursor.node();
        if node.is_error() || node.is_missing() {
            return node.start_byte();
        }
        if node.has_error() && cursor.goto_first_child() {
            continue;
        }
        loop {
            if cursor.goto_next_sibling() {
                continue 'walk;
            }
            if !cursor.goto_parent() {
                break 'walk;
            }
        }
    }
    root.start_byte()
}

fn node_text(node: Node<'_>, text: &str) -> Option<String> {
    text.get(node.start_byte()..node.end_byte())
        .map(str::to_owned)
}

fn named_field(node: Node<'_>, field: &str, text: &str) -> Option<String> {
    node_text(node.child_by_field_name(field)?, text)
}

/// The name a symbol node carries.
///
/// A `None` here drops the node from the output entirely — deliberate for the
/// two cases where a grammar produces a symbol-kind node with no name (an
/// anonymous TOML `pair` cannot occur; a nameless Rust item cannot either),
/// and the corpus manifest's exact counts are the check that this is not
/// silently dropping real symbols.
fn symbol_name(language: SyntaxLanguage, node: Node<'_>, text: &str) -> Option<String> {
    match language {
        SyntaxLanguage::Markdown => {
            let content = node.child_by_field_name("heading_content")?;
            Some(node_text(content, text)?.trim().to_owned())
        }
        SyntaxLanguage::Toml => {
            // `table`, `table_array_element` and `pair` all name themselves
            // with their first key child rather than a `name` field.
            let mut child_cursor = node.walk();
            let key = node
                .named_children(&mut child_cursor)
                .find(|child| matches!(child.kind(), "bare_key" | "dotted_key" | "quoted_key"))?;
            Some(unquote(node_text(key, text)?))
        }
        _ => named_field(node, "name", text),
    }
}

/// Every import one candidate node names, in source order.
///
/// **A list, not an `Option`.** A single node can name several imports —
/// Python's `import os, sys` is one `import_statement` carrying two `name`
/// fields — and a signature that can answer only once turns "two imports" into
/// "one import" with nothing downstream able to tell. That is a silent partial
/// extraction of a claimed node, which is the same thing the module docs refuse
/// for a partial parse, so it gets the same treatment: every name the node
/// wrote becomes a row, or the extraction is wrong.
///
/// Empty is the honest answer for a candidate that is not an import after all
/// (every Bash `command` is a candidate; only `source` and `.` are imports).
/// Empty is never the answer for a node that *is* one.
fn imports_in(language: SyntaxLanguage, node: Node<'_>, text: &str) -> Vec<Import> {
    match (language, node.kind()) {
        (SyntaxLanguage::Rust, "use_declaration") => one(node, named_field(node, "argument", text)),
        (SyntaxLanguage::Rust, "extern_crate_declaration") => {
            one(node, named_field(node, "name", text))
        }
        (SyntaxLanguage::Python, "import_from_statement") => {
            one(node, named_field(node, "module_name", text))
        }
        (SyntaxLanguage::Python, "import_statement") => {
            // tree-sitter-python attaches the `name` field to EVERY entry of
            // the comma list, so this iterates the field rather than asking for
            // it once (`child_by_field_name` answers with the first only).
            // Each entry carries its own span: two names in one statement are
            // two facts written at two places, and giving both the statement's
            // span would make them indistinguishable by position.
            let mut cursor = node.walk();
            node.children_by_field_name("name", &mut cursor)
                .filter_map(|entry| {
                    let target = if entry.kind() == "aliased_import" {
                        named_field(entry, "name", text)?
                    } else {
                        node_text(entry, text)?
                    };
                    Some(at(entry, target))
                })
                .collect()
        }
        (SyntaxLanguage::JavaScript | SyntaxLanguage::TypeScript, "import_statement") => {
            // The ordinary form puts the module string on the statement's own
            // `source` field. TypeScript's legacy CommonJS-interop form —
            // `import foo = require("./x")` — is the same node kind with no
            // `source` field of its own: the string sits inside an
            // `import_require_clause` child. Both are unambiguously imports, so
            // both produce an edge; only a node matching neither shape (which
            // this grammar does not produce) answers empty.
            let source = named_field(node, "source", text).or_else(|| {
                let mut cursor = node.walk();
                let clause = node
                    .named_children(&mut cursor)
                    .find(|child| child.kind() == "import_require_clause")?;
                named_field(clause, "source", text)
            });
            one(node, source.map(unquote))
        }
        (SyntaxLanguage::Bash, "command") => {
            // Only `source X` and `. X` are imports; every other command is a
            // candidate that answers empty.
            let target = named_field(node, "name", text)
                .filter(|name| name == "source" || name == ".")
                .and_then(|_| named_field(node, "argument", text))
                .map(unquote);
            one(node, target)
        }
        _ => Vec::new(),
    }
}

/// The zero-or-one case of [`imports_in`], spanning the whole candidate node.
fn one(node: Node<'_>, target: Option<String>) -> Vec<Import> {
    target.map(|target| at(node, target)).into_iter().collect()
}

/// One import, positioned at `node`.
fn at(node: Node<'_>, target: String) -> Import {
    Import {
        target,
        byte_start: node.start_byte(),
        byte_end: node.end_byte(),
    }
}

/// Strip one layer of matching ASCII quotes, if present.
fn unquote(value: String) -> String {
    let bytes = value.as_bytes();
    if bytes.len() >= 2
        && (bytes[0] == b'"' || bytes[0] == b'\'')
        && bytes[bytes.len() - 1] == bytes[0]
    {
        return value[1..value.len() - 1].to_owned();
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routing_claims_exactly_the_six_families_and_refuses_tsx() {
        assert_eq!(language_for("src/main.rs"), Some(SyntaxLanguage::Rust));
        assert_eq!(language_for("Cargo.toml"), Some(SyntaxLanguage::Toml));
        assert_eq!(language_for("README.MD"), Some(SyntaxLanguage::Markdown));
        assert_eq!(language_for("a/b.py"), Some(SyntaxLanguage::Python));
        assert_eq!(language_for("a/b.mjs"), Some(SyntaxLanguage::JavaScript));
        assert_eq!(language_for("a/b.ts"), Some(SyntaxLanguage::TypeScript));
        assert_eq!(language_for("a/b.bash"), Some(SyntaxLanguage::Bash));
        assert_eq!(language_for("a/b.tsx"), None);
        assert_eq!(language_for("LICENSE"), None);
    }

    #[test]
    fn extractor_identity_is_per_language_and_versioned() {
        assert_eq!(SyntaxLanguage::Rust.extractor_identity(), "syntax-rust/v1");
        assert_eq!(
            SyntaxLanguage::TypeScript.extractor_identity(),
            "syntax-typescript/v1"
        );
        let mut identities: Vec<String> = SyntaxLanguage::ALL
            .iter()
            .map(|l| l.extractor_identity())
            .collect();
        identities.sort();
        identities.dedup();
        assert_eq!(identities.len(), SyntaxLanguage::ALL.len());
    }

    #[test]
    fn a_symbol_slices_back_out_of_the_original_bytes() {
        let source = b"pub fn only() {}\n";
        let facts = extract(SyntaxLanguage::Rust, source).expect("parses");
        assert_eq!(facts.symbols.len(), 1);
        let symbol = &facts.symbols[0];
        assert_eq!(symbol.name, "only");
        assert_eq!(
            &source[symbol.byte_start..symbol.byte_end],
            b"pub fn only() {}"
        );
    }

    #[test]
    fn malformed_input_is_an_error_and_carries_no_partial_symbols() {
        let error = extract(SyntaxLanguage::Rust, b"pub fn ok() {}\npub fn broken( {\n")
            .expect_err("must not parse");
        assert!(
            matches!(
                error,
                SyntaxError::Parse {
                    language: "rust",
                    ..
                }
            ),
            "unexpected error: {error:?}"
        );
    }

    #[test]
    fn every_name_in_a_python_comma_import_becomes_its_own_edge() {
        let source = b"import os, sys\nimport a.b as ab, c\n";
        let facts = extract(SyntaxLanguage::Python, source).expect("parses");
        let targets: Vec<&str> = facts.imports.iter().map(|i| i.target.as_str()).collect();
        assert_eq!(
            targets,
            vec!["os", "sys", "a.b", "c"],
            "a comma list must not lose every name after the first"
        );
        // And each one is positioned at the name it was written as, not at the
        // statement, so two names in one statement are two distinguishable
        // facts.
        for import in &facts.imports {
            let slice = std::str::from_utf8(&source[import.byte_start..import.byte_end])
                .expect("span is UTF-8");
            assert!(slice.contains(&import.target), "{slice:?}");
        }
        assert_ne!(
            (facts.imports[0].byte_start, facts.imports[0].byte_end),
            (facts.imports[1].byte_start, facts.imports[1].byte_end)
        );
    }

    #[test]
    fn a_typescript_import_require_is_an_edge_not_a_silent_drop() {
        let source = b"import foo = require(\"./x\");\nimport bar from \"node:fs\";\n";
        let facts = extract(SyntaxLanguage::TypeScript, source).expect("parses");
        let targets: Vec<&str> = facts.imports.iter().map(|i| i.target.as_str()).collect();
        assert_eq!(
            targets,
            vec!["./x", "node:fs"],
            "the CommonJS-interop form is claimed as an import_statement and must yield an edge"
        );
    }

    #[test]
    fn a_command_that_is_not_source_is_not_an_import() {
        let facts = extract(
            SyntaxLanguage::Bash,
            b"set -euo pipefail\nsource ./lib.sh\n",
        )
        .expect("parses");
        let targets: Vec<&str> = facts.imports.iter().map(|i| i.target.as_str()).collect();
        assert_eq!(targets, vec!["./lib.sh"]);
    }

    #[test]
    fn multi_byte_text_before_a_name_does_not_shift_its_span() {
        let source = "// 日本語のコメント — em dash too\npub fn café_fn() {}\n".as_bytes();
        let facts = extract(SyntaxLanguage::Rust, source).expect("parses");
        assert_eq!(facts.symbols.len(), 1);
        let symbol = &facts.symbols[0];
        assert_eq!(symbol.name, "café_fn");
        assert_eq!(
            std::str::from_utf8(&source[symbol.byte_start..symbol.byte_end]).expect("UTF-8"),
            "pub fn café_fn() {}"
        );
    }

    #[test]
    fn non_utf8_bytes_are_refused_rather_than_lossily_decoded() {
        assert_eq!(
            extract(SyntaxLanguage::Rust, &[0xff, 0xfe]).unwrap_err(),
            SyntaxError::NotUtf8
        );
    }
}
