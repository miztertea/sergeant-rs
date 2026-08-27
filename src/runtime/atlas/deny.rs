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
//! # A denied byte is a counted byte
//!
//! Nothing here silently drops a path. The scanner turns every verdict into a
//! [`Coverage::Excluded`](crate::domain::source::Coverage::Excluded) row
//! naming the pattern that matched — which is what makes "the deny set is
//! working" a checkable claim rather than an absence of evidence.

use globset::{Glob, GlobSet, GlobSetBuilder};

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

/// The dotfile rule's own reported reason — a pattern-shaped string so a
/// coverage row's `detail` reads the same whichever layer refused the path.
pub const DOTFILE_PATTERN: &str = "<dotfile>";

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
            let glob = Glob::new(&pattern).map_err(|source| BadPattern {
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
        // The dotfile rule, first and separately: it is a component rule, not
        // a glob, and it is the one that catches `.git`, `.ssh`, `.env` and
        // every `.env.<anything>` in a single line of ordinary Rust (R6 —
        // and R3, since `str::split` is all it needs).
        if relative
            .split('/')
            .any(|component| component.starts_with('.') && component != "." && component != "..")
        {
            return Verdict::Denied {
                pattern: DOTFILE_PATTERN.to_string(),
            };
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
