//! Package/dependency identity from `Cargo.lock` (A1 §10, A1-26, S4 Y5) —
//! the minimum honest model the contract itself draws:
//!
//! ```text
//! package identity             a strong lockfile fact
//! package -> source project    mapping evidence (NOT modelled here)
//! source project -> exact SHA  exact only when independently resolved (NOT modelled here)
//! ```
//!
//! This module is the first line only: parse `Cargo.lock`'s `[[package]]`
//! entries into `(name, version, source)` and derive a
//! [purl](https://github.com/package-url/purl-spec) **when applicable**. It
//! deliberately does not attempt the second or third line — "do not
//! collapse a project mapping into 'this exact source commit built my
//! package'" (§10's own words) — because doing so honestly needs
//! independently-resolved evidence (a deps.dev lookup, an attested build)
//! that no wave has built yet, and a package-to-project *guess* dressed up
//! as a purl would be exactly the false confidence A1-26 refuses.
//!
//! # Why not every package gets a purl
//!
//! A crates.io package (`source = "registry+https://…crates.io-index"`) has
//! an unambiguous purl: `pkg:cargo/<name>@<version>` — verified against the
//! `package-url` project's own reference implementation and examples
//! (`pkg:cargo/rand@0.8.5`, resolving to `https://crates.io/crates/rand/…`),
//! not recalled. A git-sourced dependency (`source = "git+https://…"`) and a
//! path dependency (no `source` field at all — Cargo omits it for anything
//! outside a registry) have **no** purl-spec type this build's read of the
//! specification found that cleanly and unambiguously names "this exact
//! revision, from this exact remote, of a crate with no registry entry" —
//! informal `vcs_url` qualifiers exist in the wild but are not part of the
//! base spec's cargo type, and guessing one would be exactly the "PURL only
//! ... §10" contract's own R1 (`R2 no package identity model exists`) rung
//! naming a real absence rather than papering over it. So
//! [`LockedPackage::purl`] is `None` for anything that is not a plain
//! registry dependency — a real, named gap, not a silent one.

use std::collections::BTreeMap;

/// One `[[package]]` entry from a `Cargo.lock`, as resolved (never as
/// declared in `Cargo.toml` — a lockfile is the exact, resolved fact §10
/// asks for).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockedPackage {
    /// Crate name.
    pub name: String,
    /// Resolved version (already exact — a lockfile has no ranges).
    pub version: String,
    /// How Cargo resolved it, verbatim from the `source` field. `None` for a
    /// path dependency (Cargo emits no `source` line for one) and for the
    /// workspace's own root package(s), which likewise carry none.
    pub source: Option<String>,
}

impl LockedPackage {
    /// A1 §10's identity: `pkg:cargo/<name>@<version>` for a plain registry
    /// dependency, `None` for anything else (module doc explains why).
    ///
    /// **Registry, not merely present.** `source.starts_with("registry+")` is
    /// the one shape Cargo emits for `crates.io` and any other cargo
    /// registry (an alternate registry's index URL still uses the
    /// `registry+` prefix) — the shape a purl can name unambiguously. A
    /// `git+`/`sparse+` source, or none at all, answers `None`.
    pub fn purl(&self) -> Option<String> {
        let source = self.source.as_deref()?;
        if source.starts_with("registry+") {
            Some(format!(
                "pkg:cargo/{}@{}",
                purl_encode(&self.name),
                purl_encode(&self.version)
            ))
        } else {
            None
        }
    }
}

/// Failures reading a `Cargo.lock`.
#[derive(Debug, thiserror::Error)]
pub enum LockfileError {
    /// Not valid TOML at all.
    #[error("Cargo.lock is not valid TOML: {0}")]
    Toml(#[from] toml::de::Error),
}

/// Parse every `[[package]]` entry out of a `Cargo.lock`'s bytes, in file
/// order. Bytes, not a path: this is a pure function over what a caller
/// already read (F6's own adapter-shape mandate, applied to a lockfile the
/// same way it applies to every other resource this build extracts).
pub fn parse_cargo_lock(bytes: &str) -> Result<Vec<LockedPackage>, LockfileError> {
    #[derive(serde::Deserialize)]
    struct Raw {
        #[serde(default, rename = "package")]
        packages: Vec<RawPackage>,
    }
    #[derive(serde::Deserialize)]
    struct RawPackage {
        name: String,
        version: String,
        #[serde(default)]
        source: Option<String>,
    }
    let raw: Raw = toml::from_str(bytes)?;
    Ok(raw
        .packages
        .into_iter()
        .map(|p| LockedPackage {
            name: p.name,
            version: p.version,
            source: p.source,
        })
        .collect())
}

/// Every locked package's purl, keyed by `(name, version)` so a caller can
/// join it back onto whichever row named the package — the one thing this
/// module hands a consumer: identity, not a mapping.
pub fn purls_by_package(packages: &[LockedPackage]) -> BTreeMap<(String, String), Option<String>> {
    packages
        .iter()
        .map(|p| ((p.name.clone(), p.version.clone()), p.purl()))
        .collect()
}

/// Percent-encode the handful of characters a purl component must not carry
/// literally (the spec's own minimal set for a name/version segment: `/`,
/// `#`, `?`, `@`, and a literal space). A crate name or a semver string in
/// practice never contains any of these, so this is defense against a
/// pathological input, not a real-world case — but "pure function over
/// bytes" means it must be correct for bytes it does not expect to see, not
/// merely for the bytes it usually gets.
fn purl_encode(segment: &str) -> String {
    let mut out = String::with_capacity(segment.len());
    for byte in segment.bytes() {
        match byte {
            b'/' | b'#' | b'?' | b'@' | b' ' => {
                out.push('%');
                out.push_str(&format!("{byte:02X}"));
            }
            _ => out.push(byte as char),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_LOCK: &str = r#"
# This file is automatically @generated by Cargo.
version = 4

[[package]]
name = "sergeant-rs"
version = "0.3.0"

[[package]]
name = "serde"
version = "1.0.193"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "abc123"

[[package]]
name = "some-fork"
version = "0.1.0"
source = "git+https://github.com/example/some-fork?rev=deadbeef#deadbeef"

[[package]]
name = "local-crate"
version = "0.0.1"
"#;

    #[test]
    fn a_registry_package_gets_the_verified_cargo_purl_shape() {
        let packages = parse_cargo_lock(SAMPLE_LOCK).expect("parse");
        let serde = packages
            .iter()
            .find(|p| p.name == "serde")
            .expect("serde present");
        assert_eq!(serde.purl().as_deref(), Some("pkg:cargo/serde@1.0.193"));
    }

    #[test]
    fn a_git_sourced_package_has_no_purl_named_rather_than_guessed() {
        let packages = parse_cargo_lock(SAMPLE_LOCK).expect("parse");
        let fork = packages
            .iter()
            .find(|p| p.name == "some-fork")
            .expect("present");
        assert_eq!(
            fork.purl(),
            None,
            "a git source is not a registry — no purl is invented"
        );
    }

    #[test]
    fn a_path_or_workspace_package_with_no_source_has_no_purl() {
        let packages = parse_cargo_lock(SAMPLE_LOCK).expect("parse");
        let root = packages
            .iter()
            .find(|p| p.name == "sergeant-rs")
            .expect("present");
        assert_eq!(root.source, None);
        assert_eq!(root.purl(), None);
        let local = packages
            .iter()
            .find(|p| p.name == "local-crate")
            .expect("present");
        assert_eq!(local.purl(), None);
    }

    #[test]
    fn every_locked_package_is_parsed_in_file_order() {
        let packages = parse_cargo_lock(SAMPLE_LOCK).expect("parse");
        let names: Vec<&str> = packages.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["sergeant-rs", "serde", "some-fork", "local-crate"]
        );
    }

    #[test]
    fn purls_by_package_joins_on_name_and_version() {
        let packages = parse_cargo_lock(SAMPLE_LOCK).expect("parse");
        let map = purls_by_package(&packages);
        assert_eq!(
            map.get(&("serde".to_string(), "1.0.193".to_string())),
            Some(&Some("pkg:cargo/serde@1.0.193".to_string()))
        );
        assert_eq!(
            map.get(&("some-fork".to_string(), "0.1.0".to_string())),
            Some(&None)
        );
    }

    #[test]
    fn not_toml_at_all_is_a_named_error_not_an_empty_list() {
        let err = parse_cargo_lock("not { valid toml").expect_err("must refuse");
        assert!(matches!(err, LockfileError::Toml(_)), "{err}");
    }

    #[test]
    fn an_empty_lockfile_is_a_real_empty_answer() {
        let packages = parse_cargo_lock("version = 4\n").expect("parse");
        assert!(packages.is_empty());
    }

    /// The verified reference shape, restated as a literal pin so a future
    /// change to [`LockedPackage::purl`]'s format is caught even if every
    /// other test happens to still agree with it by coincidence.
    #[test]
    fn the_purl_shape_matches_the_verified_reference_examples() {
        let rand = LockedPackage {
            name: "rand".to_string(),
            version: "0.8.5".to_string(),
            source: Some("registry+https://github.com/rust-lang/crates.io-index".to_string()),
        };
        assert_eq!(rand.purl().as_deref(), Some("pkg:cargo/rand@0.8.5"));
    }
}
