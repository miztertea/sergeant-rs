//! F10's secrets posture, enforced at the acquisition boundary.
//!
//! One rule, and the whole module exists to make it structurally true: **a
//! denied path's bytes are never read.** Not read-then-discarded, not
//! read-then-redacted — the verdict is a pure function of the path, so the
//! scanner can ask it *before* it opens anything, and a file it never opens
//! cannot leak through a fold, a cache, a log line, or a crash dump.
//!
//! Everything here is a pure function over a relative path string. No IO, no
//! DB handle, no daemon state (F6's adapter-shape mandate).
//!
//! # Two layers, one direction
//!
//! [`DEFAULT_DENY`] is the built-in minimum and a per-source
//! `[[knowledge]] ignore` list **extends** it. There is deliberately no
//! syntax for narrowing: a source cannot opt back into a pattern the default
//! set protects it from, because the one thing a secrets floor may not have
//! is a per-source override that turns it off.
//!
//! # Dot-ness is not the boundary
//!
//! An earlier version of this module denied every path with *any* dotted
//! directory component — which made the estate's own shipped doctrine tree
//! (`.sergeant/`) unindexable. The owner's ruling
//! (`projection-model-and-false-j0s-2026-08-31.md` #1) names that a defect,
//! not a security posture: "does not indexing `.sergeant` satisfy estate
//! intelligence? You already had your answer." What actually needs denying
//! is three separate, narrower things, matching the standard shape real
//! tools use (gitignore semantics plus a small named machinery list, not a
//! blanket dot rule — semble's own [`_DEFAULT_IGNORED_DIRS`] and ripgrep's
//! hidden/VCS handling read the same way):
//!
//! - a **hidden file** (its own leaf name starts with `.`) —
//!   [`AcquisitionFilter::verdict`] still refuses these, because a dotfile
//!   is exactly the shape most secret/config files come in (`.env`,
//!   `.npmrc`, `.netrc`) even before [`DEFAULT_DENY`]'s named patterns are
//!   consulted;
//! - a **known VCS/machinery or credential-store directory**
//!   ([`DENIED_DIRECTORIES`]) — `.git`, `.hg`, `.svn`, `.ssh`, plus the
//!   conventional per-tool credential stores `.aws`, `.docker`, `.kube`,
//!   `.gnupg`, `.m2`, `.npm` — denied by identity because a scanner must
//!   never open bytes from inside version-control internals or a
//!   directory whose entire purpose is to hold key material or embedded
//!   credentials (`~/.aws/credentials`, `~/.docker/config.json`'s auth
//!   tokens, `~/.kube/config`, `~/.m2/settings.xml`'s server passwords,
//!   `~/.npm/_auth`) — never because the directory happens to be dotted;
//! - the **estate's own mutable runtime state**
//!   (`crate::domain::manifest::DEFAULT_ESTATE_DATA_DIR`,
//!   `.sergeant/data/`) — journal, blobs, worker surfaces — excluded
//!   structurally by that one known path, because it is machinery the
//!   projection is derived *from* the daemon's operation of, not corpus.
//!
//! A dotted *directory* that is none of the above — `.sergeant/common/`,
//! `.sergeant/workflows/` — is walked exactly like an undotted one.
//!
//! # Case is not a way through the floor
//!
//! Every glob here is compiled **case-insensitively**. The reason is an
//! asymmetry that would otherwise be exploitable by accident: extractor
//! routing lowercases the extension before it decides
//! ([`extractor_for`](crate::runtime::atlas::scan)), so `NOTES.MD` is read
//! exactly as `notes.md` is — while a case-sensitive deny set would let
//! `Secrets.md`, `CREDENTIALS.txt` or `ID_RSA` walk straight past the very
//! families [`DEFAULT_DENY`] names, be read, hashed, extracted and persisted.
//! A secrets floor may not be narrower than the reader it guards, so the two
//! agree about case in the one direction that is safe: the floor tolerates
//! everything the reader tolerates.
//!
//! Operator `ignore` globs are compiled the same way, for the same reason and
//! in the same direction — case-insensitivity only ever *widens* an
//! exclusion, and `ignore` is a list that may only widen.
//!
//! # A denied byte is a counted byte
//!
//! Nothing here silently drops a path. The scanner turns every verdict into a
//! [`Coverage::Excluded`](crate::domain::source::Coverage::Excluded) row
//! naming the pattern that matched — which is what makes "the deny set is
//! working" a checkable claim rather than an absence of evidence.

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

/// The built-in deny set (F10's G4 minimum), in addition to the dotfile rule
/// below.
///
/// Deliberately conservative and deliberately *small*. Every entry names a
/// file family whose whole purpose is to hold a credential — private keys,
/// keystores, credential and secret files, environment files. Broad
/// catch-alls that would sweep up ordinary prose (`*token*`, `*password*`,
/// `env.*`) are **not** here: a false positive is unfixable from a manifest,
/// since `ignore` only ever extends this list, so the default set stays
/// narrow and the operator widens it.
pub const DEFAULT_DENY: &[&str] = &[
    // Private keys and key material.
    "**/*.pem",
    "**/*.key",
    "**/*.p8",
    "**/*.p12",
    "**/*.pfx",
    "**/*.jks",
    "**/*.keystore",
    "**/*.kdbx",
    "**/*.ppk",
    "**/*.asc",
    "**/*.gpg",
    "**/id_rsa*",
    "**/id_dsa*",
    "**/id_ecdsa*",
    "**/id_ed25519*",
    // Environment files that are not already dotfiles (`.env`, `.env.local`
    // and friends are caught by the dotfile rule).
    "**/*.env",
    // Credential and secret files by convention.
    "**/credentials",
    "**/credentials.*",
    "**/secrets",
    "**/secrets.*",
    "**/*.secret",
    "**/service-account*.json",
];

/// The hidden-*file* rule's own reported reason (leaf name starts with
/// `.`) — a pattern-shaped string so a coverage row's `detail` reads the
/// same whichever layer refused the path.
pub const DOTFILE_PATTERN: &str = "<dotfile>";

/// Known version-control/machinery and credential-store directory names,
/// denied by identity wherever they appear as a non-leaf path component —
/// never because the name happens to start with `.` (see this module's
/// "Dot-ness is not the boundary"). `.git`/`.hg`/`.svn` are the VCS
/// internals no scanner may open bytes from; `.ssh` is key material by
/// convention, the same reasoning [`DEFAULT_DENY`]'s `id_rsa*` family
/// already applies to individual files. `.aws`, `.docker`, `.kube`,
/// `.gnupg`, `.m2`, `.npm` are the conventional per-tool credential stores:
/// each holds at least one plainly-named, non-dotfile leaf that carries a
/// credential by convention (`config`, `config.json`, `settings.xml`,
/// `_auth`) — a leaf name [`DEFAULT_DENY`]'s globs do not and should not
/// try to guess, so the directory is denied by identity instead, the same
/// pattern `.ssh` already uses.
pub const DENIED_DIRECTORIES: &[&str] = &[
    ".git", ".hg", ".svn", ".ssh", ".aws", ".docker", ".kube", ".gnupg", ".m2", ".npm",
];

/// A known machinery *directory*'s own reported reason — distinct from
/// [`DOTFILE_PATTERN`] so a coverage row can tell "this is a hidden file"
/// from "this is inside version-control internals."
pub const MACHINERY_DIR_PATTERN: &str = "<machinery-dir>";

/// The estate's own mutable runtime state's reported reason — distinct from
/// both of the above, because this exclusion is structural (one known path
/// the manifest scaffolds), not a dot rule or a VCS-name rule.
pub const ESTATE_DATA_PATTERN: &str = "<estate-data>";

/// A pattern the caller supplied that is not a valid glob.
#[derive(Debug, thiserror::Error)]
#[error("{pattern:?} is not a valid ignore glob: {source}")]
pub struct BadPattern {
    /// The offending pattern.
    pub pattern: String,
    /// globset's own diagnostic.
    #[source]
    pub source: globset::Error,
}

/// What the boundary decided about one path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// May be opened.
    Allowed,
    /// Must not be opened; the pattern that refused it, for the coverage row.
    Denied {
        /// The matching pattern, or [`DOTFILE_PATTERN`].
        pattern: String,
    },
}

impl Verdict {
    /// Whether the path may be opened.
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }
}

/// The compiled acquisition boundary for one source.
#[derive(Debug)]
pub struct AcquisitionFilter {
    set: GlobSet,
    /// Patterns in the same index order [`GlobSet::matches`] reports, so a
    /// match can name itself. `GlobSet` gives back indices, not patterns.
    patterns: Vec<String>,
}

impl AcquisitionFilter {
    /// Compile [`DEFAULT_DENY`] plus this source's own `ignore` globs.
    ///
    /// The defaults are compiled first so an index reported by the set maps
    /// back onto the same pattern list every time, and a source pattern can
    /// never displace a default one.
    ///
    /// Every glob is built with `case_insensitive(true)` — see this module's
    /// "Case is not a way through the floor". That is a property of the
    /// compilation, not of the pattern strings, so a default entry and an
    /// operator entry cannot drift apart on it.
    pub fn new(ignore: &[String]) -> Result<Self, BadPattern> {
        let mut builder = GlobSetBuilder::new();
        let mut patterns = Vec::with_capacity(DEFAULT_DENY.len() + ignore.len());
        for pattern in DEFAULT_DENY.iter().map(|p| (*p).to_string()).chain(
            // A bare `foo.log` from an operator means "anywhere", the same
            // thing it means in a `.gitignore`; `**/` makes globset agree.
            // A pattern that already carries a separator is left exactly as
            // written, because then the operator is being specific on
            // purpose.
            ignore.iter().map(|p| {
                if p.contains('/') || p.starts_with("**") {
                    p.clone()
                } else {
                    format!("**/{p}")
                }
            }),
        ) {
            let glob = GlobBuilder::new(&pattern)
                .case_insensitive(true)
                .build()
                .map_err(|source| BadPattern {
                    pattern: pattern.clone(),
                    source,
                })?;
            builder.add(glob);
            patterns.push(pattern);
        }
        let set = builder.build().map_err(|source| BadPattern {
            pattern: "<set>".to_string(),
            source,
        })?;
        Ok(Self { set, patterns })
    }

    /// The verdict for one path, relative to the source root, `/`-separated.
    ///
    /// Answers for directories exactly as for files — a denied directory is
    /// one the walk must not descend into, which is how `.git`, `.ssh` and a
    /// `node_modules` in an `ignore` list cost one verdict rather than one
    /// per contained file.
    ///
    /// The two spellings of "exclude a directory" therefore differ in
    /// *reporting*, not in what is excluded: `build` matches the directory
    /// itself, so the walk refuses it once and never descends; `build/**`
    /// matches its contents, so each excluded file gets its own coverage row
    /// and the directory is merely `discovered`. Both exclude the same bytes,
    /// and both say so out loud.
    pub fn verdict(&self, relative: &str) -> Verdict {
        self.verdict_component(relative, false)
    }

    /// As [`Self::verdict`], but `relative` names a *directory*, not a file
    /// — the shape `directory_coverage` (`git.rs`) calls this with, once
    /// per distinct directory prefix a tree contains, on the directory's
    /// own bare path string.
    ///
    /// A single-component path is ambiguous by itself: `".sergeant"` could
    /// be a hidden file or a directory, and [`Self::verdict`]'s leaf rule
    /// can only guess from the string. This method removes the guess by
    /// telling `verdict_component` the truth its caller already knows: the
    /// leaf component here is a directory name, so it is checked against
    /// [`DENIED_DIRECTORIES`] like any non-leaf component, never against
    /// the hidden-*file* dotfile rule — bug 1
    /// (`brief-search-three-bugs.md`): `.sergeant`/`.github` are dotted
    /// directories, not hidden files, and this module's own "Dot-ness is
    /// not the boundary" already says a dotted directory that is not
    /// machinery is walked like any other.
    pub fn verdict_directory(&self, relative: &str) -> Verdict {
        self.verdict_component(relative, true)
    }

    /// Shared by [`Self::verdict`] and [`Self::verdict_directory`] — the
    /// only difference between a file verdict and a directory verdict is
    /// whether the *leaf* component may be a hidden file (`is_directory:
    /// false`) or must be checked as a directory name instead (`true`);
    /// everything else — the estate-data-dir check, the machinery-directory
    /// check for every non-leaf component, the glob set — is identical.
    fn verdict_component(&self, relative: &str, is_directory: bool) -> Verdict {
        // The estate's own machinery, first and structurally: one known
        // path, not a dot rule — `crate::domain::manifest::DEFAULT_ESTATE_DATA_DIR`
        // is `sgt init`'s own scaffolded literal, so this module and the
        // manifest module agree on it by construction rather than by two
        // hand-copied strings drifting apart.
        if relative == crate::domain::manifest::DEFAULT_ESTATE_DATA_DIR
            || relative.starts_with(&format!(
                "{}/",
                crate::domain::manifest::DEFAULT_ESTATE_DATA_DIR
            ))
        {
            return Verdict::Denied {
                pattern: ESTATE_DATA_PATTERN.to_string(),
            };
        }
        // Component rules, in order: a known VCS/machinery *directory*
        // anywhere in the path, then a hidden *leaf* file — component rules,
        // not globs, so both cost one line of ordinary Rust (R6, R3:
        // `str::split` is all either needs). Neither is "any dotted
        // component", per this module's "Dot-ness is not the boundary": a
        // dotted directory that is not on the machinery list (`.sergeant/`
        // itself, say) is walked like any other.
        //
        // The leaf component is only ever checked against the hidden-file
        // dotfile rule when the caller has told us it names a file
        // (`!is_directory`); a directory's own leaf name is checked against
        // `DENIED_DIRECTORIES` exactly like every non-leaf component is.
        let mut components = relative.split('/').peekable();
        while let Some(component) = components.next() {
            let is_leaf = components.peek().is_none();
            if is_leaf && !is_directory {
                if component.starts_with('.') && component != "." && component != ".." {
                    return Verdict::Denied {
                        pattern: DOTFILE_PATTERN.to_string(),
                    };
                }
            } else if DENIED_DIRECTORIES.contains(&component) {
                return Verdict::Denied {
                    pattern: MACHINERY_DIR_PATTERN.to_string(),
                };
            }
        }
        match self.set.matches(relative).first() {
            Some(&index) => Verdict::Denied {
                pattern: self.patterns[index].clone(),
            },
            None => Verdict::Allowed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filter() -> AcquisitionFilter {
        AcquisitionFilter::new(&[]).expect("compile defaults")
    }

    /// The G4 minimum, checked as behavior rather than as a constant: each
    /// family the default set names is actually refused, at the top level and
    /// nested.
    #[test]
    fn the_default_set_refuses_every_family_it_names() {
        let filter = filter();
        for denied in [
            ".env",
            ".env.production",
            ".git/config",
            ".ssh/known_hosts",
            "notes/.hidden.md",
            "server.pem",
            "deep/nest/server.key",
            "keys/id_rsa",
            "keys/id_ed25519.pub",
            "local.env",
            "credentials",
            "aws/credentials.json",
            "secrets.yaml",
            "deploy/service-account-prod.json",
            "vault.kdbx",
        ] {
            assert!(
                !filter.verdict(denied).is_allowed(),
                "{denied:?} must be refused at the acquisition boundary"
            );
        }
    }

    /// The floor is as case-tolerant as the reader it guards.
    ///
    /// [`extractor_for`](crate::runtime::atlas::scan) lowercases the
    /// extension before routing, so `Secrets.md` and `CREDENTIALS.txt` are
    /// files this build will happily read, hash, extract and persist. A
    /// case-sensitive deny set would therefore let exactly the families
    /// [`DEFAULT_DENY`] names through the acquisition boundary — which is the
    /// leak F10 exists to prevent, arriving by way of a shift key.
    #[test]
    fn the_default_set_refuses_case_variants_of_the_families_it_names() {
        let filter = filter();
        for denied in [
            "Secrets.md",
            "SECRETS.md",
            "Secrets.MD",
            "notes/CREDENTIALS.txt",
            "Credentials.markdown",
            "CREDENTIALS",
            "keys/ID_RSA",
            "keys/Id_Ed25519.txt",
            "Server.PEM",
            "deep/API.Key",
            "Local.ENV",
            "deploy/Service-Account-Prod.json",
            "Vault.KDBX",
        ] {
            assert!(
                !filter.verdict(denied).is_allowed(),
                "{denied:?} must be refused at the acquisition boundary — the \
                 extractor routing that follows is case-insensitive, so the \
                 floor cannot be case-sensitive"
            );
        }
    }

    /// The ruling (`projection-model-and-false-j0s-2026-08-31.md` #1): a
    /// blanket dot-*directory* deny made the estate's own shipped doctrine
    /// tree unindexable, and "does not indexing `.sergeant` satisfy estate
    /// intelligence?" already answers that this is a defect, not a posture.
    /// A doctrine file nested under a dotted directory component must be
    /// admissible — it is not itself secret-shaped and its directory is not
    /// VCS/machinery.
    #[test]
    fn the_shipped_doctrine_tree_is_indexable() {
        let filter = filter();
        for allowed in [
            ".sergeant/common/contexts/pin-fixed-point.md",
            ".sergeant/workflows/implement-change/index.md",
            ".sergeant/AGENTS.md",
        ] {
            assert_eq!(
                filter.verdict(allowed),
                Verdict::Allowed,
                "{allowed:?} is shipped doctrine, not machinery or a secret"
            );
        }
    }

    /// The same ruling, other direction: the estate's own mutable runtime
    /// state under `.sergeant/data/` (journal, blobs, worker surfaces) is
    /// machinery, not corpus, and must stay excluded — but *structurally*,
    /// by the path `sgt init` scaffolds
    /// ([`crate::domain::manifest::DEFAULT_ESTATE_DATA_DIR`]), not by the
    /// dot rule the doctrine-tree test above proves is gone.
    #[test]
    fn the_estates_own_data_dir_stays_excluded_structurally() {
        let filter = filter();
        for denied in [
            ".sergeant/data",
            ".sergeant/data/atlas.duckdb",
            ".sergeant/data/surfaces/01ABCDEF/sergeant-rs/src/main.rs",
        ] {
            assert!(
                !filter.verdict(denied).is_allowed(),
                "{denied:?} is estate machinery and must stay excluded"
            );
        }
    }

    /// Bug 1 (`brief-search-three-bugs.md`): a bare top-level dotted
    /// *directory* path is not a hidden file leaf, and must not be
    /// classified as one just because, as a standalone string with no `/`
    /// in it, it happens to look like one. This is the shape
    /// `directory_coverage` (`git.rs:597`) actually calls `verdict()` with
    /// — the file-leaf tests above never exercise it (`.sergeant/common/...`
    /// has a slash; a bare `".sergeant"` does not).
    #[test]
    fn a_bare_dotted_directory_that_is_not_machinery_is_walkable() {
        let filter = filter();
        for allowed_dir in [".sergeant", ".github", ".config"] {
            assert_eq!(
                filter.verdict_directory(allowed_dir),
                Verdict::Allowed,
                "{allowed_dir:?} is a directory, not a hidden-file leaf, and \
                 is not VCS/machinery"
            );
        }
    }

    /// Same bare-directory shape, other direction: a machinery directory
    /// name must still be refused when it *is* the leaf component, not only
    /// when nested deeper (`verdict_directory(".git")`, not just
    /// `verdict(".git/config")`).
    #[test]
    fn a_bare_machinery_directory_is_still_denied_by_name() {
        let filter = filter();
        for denied_dir in [".git", ".ssh", ".aws"] {
            assert!(
                !filter.verdict_directory(denied_dir).is_allowed(),
                "{denied_dir:?} is a bare machinery directory and must still \
                 be refused"
            );
        }
    }

    /// The file-leaf dotfile rule must be unaffected by the directory fix:
    /// `verdict()` (the file path) still refuses a bare hidden file.
    #[test]
    fn a_bare_hidden_file_is_still_denied_as_a_file() {
        let filter = filter();
        assert!(!filter.verdict(".env").is_allowed());
    }

    /// Known version-control/machinery directories are still refused by
    /// identity — not because their name starts with a dot, but because a
    /// scanner must never open bytes from inside them. This is the seam-2
    /// replacement for the two dotted-directory cases the family test above
    /// already asserts (`.git/config`, `.ssh/known_hosts`): named here on
    /// their own so a regression reads as "the VCS-dir rule broke", not
    /// folded into the family list's generic message.
    #[test]
    fn known_machinery_directories_are_refused_by_identity() {
        let filter = filter();
        for denied in [
            ".git/config",
            ".git/hooks/pre-commit",
            ".hg/hgrc",
            ".svn/entries",
            ".ssh/known_hosts",
            ".ssh/authorized_keys",
        ] {
            assert!(
                !filter.verdict(denied).is_allowed(),
                "{denied:?} is VCS/machinery and must stay excluded"
            );
        }
    }

    /// The planted-secret red case (review brief standing requirement #9):
    /// before this test existed, a credential-shaped leaf nested under a
    /// dotted directory that was *not* VCS/machinery (`.aws`, `.docker`,
    /// `.kube`, `.gnupg`, `.m2`, `.npm`) fell through both the component
    /// rule (only a leaf itself starting with `.` was refused) and every
    /// [`DEFAULT_DENY`] glob (`config`, `config.json`, `settings.xml` and
    /// `_auth` match none of them) and was silently `Verdict::Allowed`.
    /// Each of these is a real, conventional credential file for its tool.
    #[test]
    fn known_credential_store_directories_are_refused_by_identity() {
        let filter = filter();
        for denied in [
            ".aws/credentials",
            ".aws/config",
            ".docker/config.json",
            ".kube/config",
            ".gnupg/pubring.gpg",
            ".gnupg/private-keys-v1.d/deadbeef.key",
            ".m2/settings.xml",
            ".npm/_auth",
        ] {
            assert!(
                !filter.verdict(denied).is_allowed(),
                "{denied:?} is a conventional credential file and must be \
                 refused at the acquisition boundary — this is the \
                 planted-secret red case for a non-machinery dotted \
                 directory"
            );
        }
    }

    /// An operator `ignore` entry is compiled the same way, and case can only
    /// ever widen what it excludes — never re-admit anything.
    #[test]
    fn an_operator_pattern_is_case_insensitive_too() {
        let filter = AcquisitionFilter::new(&["*.log".to_string()]).expect("compile");
        assert_eq!(
            filter.verdict("deep/RUN.LOG"),
            Verdict::Denied {
                pattern: "**/*.log".to_string()
            }
        );
    }

    /// And ordinary prose is not swept up. A deny set that refuses the corpus
    /// it was meant to protect is a deny set nobody keeps switched on.
    #[test]
    fn ordinary_documents_are_allowed() {
        let filter = filter();
        for allowed in [
            "README.md",
            "notes/2026-08-27.md",
            "design/keystore-design.md",
            "runbooks/rotate-credentials-quarterly.md",
            "env-vars.md",
            "monkey.txt",
            // Case-insensitivity widens the floor; it must not widen it onto
            // ordinary prose that merely mentions a sensitive word.
            "Design/Keystore-Design.md",
            "Runbooks/Rotate-Credentials-Quarterly.md",
            "NOTES.MD",
        ] {
            assert_eq!(
                filter.verdict(allowed),
                Verdict::Allowed,
                "{allowed:?} is ordinary evidence"
            );
        }
    }

    /// A verdict names the pattern that produced it — a coverage row that
    /// said only "excluded" would leave the operator unable to tell a
    /// deliberate `ignore` from the built-in floor.
    #[test]
    fn a_denial_names_the_pattern_that_refused_it() {
        let filter = AcquisitionFilter::new(&["*.log".to_string(), "build/**".to_string()])
            .expect("compile");
        assert_eq!(
            filter.verdict("server.pem"),
            Verdict::Denied {
                pattern: "**/*.pem".to_string()
            }
        );
        assert_eq!(
            filter.verdict(".env"),
            Verdict::Denied {
                pattern: DOTFILE_PATTERN.to_string()
            }
        );
        // A bare operator pattern means "anywhere", `.gitignore`-style.
        assert_eq!(
            filter.verdict("deep/nest/run.log"),
            Verdict::Denied {
                pattern: "**/*.log".to_string()
            }
        );
        // One that already carries a separator is honoured exactly as
        // written, and stays anchored at the source root.
        assert_eq!(
            filter.verdict("build/out.md"),
            Verdict::Denied {
                pattern: "build/**".to_string()
            }
        );
        assert_eq!(filter.verdict("src/build/out.md"), Verdict::Allowed);
    }

    /// F10's direction rule: `ignore` extends, never narrows. There is no
    /// spelling of an entry that re-admits a defaulted-denied path — the
    /// closest an operator can get is a negation, and globset treats `!` as
    /// an ordinary character, so it simply fails to match anything rather
    /// than quietly disabling the floor.
    #[test]
    fn a_source_pattern_cannot_reopen_a_defaulted_denial() {
        let filter =
            AcquisitionFilter::new(&["!*.pem".to_string(), "!.env".to_string()]).expect("compile");
        assert!(!filter.verdict("server.pem").is_allowed());
        assert!(!filter.verdict(".env").is_allowed());
    }

    /// A malformed operator pattern is refused by name at compile time, not
    /// silently dropped — a silently dropped `ignore` is a silently indexed
    /// directory.
    #[test]
    fn a_malformed_pattern_is_refused_by_name() {
        let err = AcquisitionFilter::new(&["a[".to_string()]).expect_err("must refuse");
        assert!(err.to_string().contains("a["), "{err}");
    }
}
