//! External Git locator validation (A1 §9, A1-24, S4 Y5 G6) — **the primary
//! control**, checked before Git ever sees the string.
//!
//! # The threat, researched rather than recalled
//!
//! Git's own documentation (`git-clone`, `gitprotocol-pack`, `git-remote-ext`,
//! `git-config`'s `protocol.allow`) names a wider transport surface than "a
//! URL": `http[s]://<host>[:<port>]/<path>`, `ssh://[<user>@]<host>[:<port>]/<path>`,
//! a scp-like `[<user>@]<host>:<path>` with no scheme at all, `git://`,
//! `ftp[s]://`, `file:///<path>` (and a bare `/path` implying `--local`), and
//! — the named threat (A1-24) — `<transport>::<address>` remote-helper syntax,
//! of which `ext::<command>[ <args>]` is the sharpest edge: Git's own
//! `git-remote-ext` manual says plainly that this form runs `<command>` as a
//! subprocess to speak the "transport". Any string this build hands to Git
//! unexamined is a command line, not a URL.
//!
//! **This module allowlists exactly two forms — HTTPS and SSH (including
//! scp-like) — and refuses everything else, including `file://`, `git://`,
//! `ftp[s]://`, a bare local path, and any `<transport>::` form** (A1-24's
//! "minimal allowlist" resolution, R7 in the contract's own rung column: no
//! lower rung expresses "safe Git transport policy", so a hand-written
//! allowlist is the minimum that actually closes it). This is checked
//! entirely in Rust, against the raw string, before a single byte of it
//! reaches a `git` argv — refusing here costs nothing and leaks nothing to a
//! subprocess that might interpret it differently than this parser did.
//!
//! # Two controls, not one — [`super::external_git`] carries the second
//!
//! An application-level allowlist is not the only thing standing between an
//! operator-typed string and Git's own transport dispatch: the *installed*
//! Git may have its own `url.<base>.insteadOf` rewrite rule (in the
//! operator's `~/.gitconfig`, entirely outside this build's control) that
//! turns an innocent-looking `https://` locator into something else by the
//! time Git actually opens a connection — the class `protocol.allow`/
//! `GIT_ALLOW_PROTOCOL` exists to close, per Git's own config documentation:
//! "known-dangerous protocols (ext) have a default policy of never", overridable
//! only by explicit configuration, and `GIT_ALLOW_PROTOCOL` "behave[s] as if
//! `protocol.allow` is set to `never`... **overriding any existing
//! configuration**" for exactly the protocols it names. So every acquisition
//! call in [`super::external_git`] *also* sets `GIT_ALLOW_PROTOCOL=https:ssh`
//! on the subprocess itself — a second, independent gate that holds even if
//! this module's own parser had a bug, or if an `insteadOf` rewrite tried to
//! retarget a validated locator to `ext::` underneath it. **Not independently
//! verified**: whether Git's protocol-policy check runs *after* `insteadOf`
//! resolution (so a rewritten `ext::` target is still caught) is this
//! module's working assumption, grounded in `GIT_ALLOW_PROTOCOL`'s documented
//! purpose (protecting commands, like recursive submodule init, that run
//! without direct user input) rather than in a line of Git's source read for
//! this wave — stated rather than silently relied on.
//!
//! # What is refused, explicitly, and why
//!
//! * **Any `::`** — the remote-helper marker (`ext::`, or any other
//!   `<transport>::<address>` this build has never heard of) is refused
//!   outright, before any scheme check. This is a strict superset refusal:
//!   neither an accepted HTTPS/SSH URL nor a scp-like locator ever legitimately
//!   contains a literal `::`.
//! * **Any scheme other than `https://` or `ssh://`** — `http://` (plaintext,
//!   and a credential typed into it travels in the clear), `git://`
//!   (unauthenticated, unencrypted daemon protocol), `ftp[s]://`,
//!   `file://` and a bare path (A1-23's own host-cache design already reads
//!   local Git objects through `ls-tree`/`cat-file`, so a *locator* naming a
//!   local path is never a legitimate acquisition request — it is either an
//!   estate mount, which has its own admission path, or an attempt to read
//!   an arbitrary local path this process can see).
//! * **Embedded credentials** (`user:password@host` — a colon before the
//!   first `@`) — never accepted, on either scheme. The locator's `origin`
//!   is stored verbatim as provenance (A1 §9), and a credential embedded in
//!   it would be a secret landing in the journal and in a coverage-adjacent
//!   row — exactly what "credentials never enter config, the journal, or a
//!   coverage row" forbids. The operator's ambient git/ssh agent is the only
//!   accepted auth path (G6), so a locator that tries to carry its own
//!   credential is refused rather than silently stripped — stripping would
//!   accept the request while quietly discarding what the operator typed,
//!   which is worse than refusing it outright.
//! * **A userinfo/host component starting with `-`** — the argument-injection
//!   class SSH's own `-oProxyCommand=...`-style flags exploit when a "host"
//!   string reaches a shell as an unescaped argument. `git`/`ssh` invocations
//!   in this codebase already pass everything after a literal `--`
//!   ([`super::super::git::git_clone`]'s own precedent), which closes this at
//!   the subprocess boundary too — this check closes it earlier, at the
//!   string itself, so a locator that *looks* like a flag is refused with a
//!   named reason rather than silently defused.
//! * **Control characters, embedded whitespace, or an empty string** —
//!   nothing here is ever typed by a human at an interactive prompt; a
//!   locator arrives as one CLI argument or one JSON field, and any of these
//!   is either impossible to have meant or a sign the string was built by
//!   concatenation somewhere upstream.
//! * **A bare `host:path` with no `user@`** — Git itself accepts this scp-like
//!   form with no explicit remote user (defaulting to the local login name on
//!   the far end), but this build requires an explicit `user@` prefix,
//!   stricter than Git needs to be. This is the allowlist's own "minimal,
//!   not exhaustive" posture (A1-24, R7): the scp-like and remote-helper forms
//!   are the closest to each other in this grammar (`ext::foo` differs from
//!   `foo::bar` only in which word is not a scheme this build recognizes), and
//!   requiring `user@` removes any locator whose validity depends on there
//!   being no `::` *and* the segment before the first `:` not resembling a
//!   scheme.

use std::fmt;

/// The two accepted transport shapes, matched at parse time so a caller can
/// branch without re-deriving it from the raw string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocatorForm {
    /// `https://<host>[:<port>]/<path>`
    Https,
    /// `ssh://[<user>@]<host>[:<port>]/<path>`
    Ssh,
    /// `[<user>@]<host>:<path>` — Git's own scp-like shorthand for SSH, with
    /// `user@` required by this build (see module doc).
    ScpLike,
}

/// A locator that passed [`validate`] — the raw string, and which accepted
/// form it took.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalGitLocator {
    /// Exactly what the operator typed. Stored verbatim as provenance's
    /// `origin` (A1 §9) — never normalized, never re-derived, so the
    /// provenance row says what was actually asked for.
    pub raw: String,
    /// Which accepted grammar it matched.
    pub form: LocatorForm,
}

/// Why a locator was refused. Every variant names the reason a coverage row
/// or a CLI error can quote directly — never a bare "invalid".
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LocatorError {
    /// Nothing was given.
    #[error("an external Git locator may not be empty")]
    Empty,
    /// A control character (or a NUL) anywhere in the string.
    #[error("external Git locator {raw:?} contains a control character")]
    ControlCharacter {
        /// The offending string, for the error message.
        raw: String,
    },
    /// Contains a literal `::` — the remote-helper/transport-helper marker
    /// (`ext::`, or any `<transport>::<address>` this build does not know),
    /// A1-24's named threat.
    #[error(
        "external Git locator {raw:?} contains \"::\", the remote-helper transport marker \
         (git-remote-ext and friends) — refused before Git ever sees it"
    )]
    RemoteHelperSyntax {
        /// The offending string.
        raw: String,
    },
    /// A `scheme://` this build does not allowlist.
    #[error(
        "external Git locator {raw:?} uses the \"{scheme}\" transport, which is not \
         allowlisted — only https:// and ssh:// (or user@host:path) are accepted"
    )]
    UnsupportedScheme {
        /// The offending string.
        raw: String,
        /// The scheme that was refused, without its `://`.
        scheme: String,
    },
    /// Neither a recognized `scheme://` nor a valid scp-like `user@host:path`.
    #[error(
        "external Git locator {raw:?} is not a recognized https://, ssh://, or user@host:path form"
    )]
    Unrecognized {
        /// The offending string.
        raw: String,
    },
    /// A `user:password@` (or equivalent) credential embedded in the
    /// locator itself.
    #[error(
        "external Git locator {raw:?} appears to embed a credential (a \":\" before the first \
         \"@\") — credentials never enter config, the journal, or a coverage row; use the \
         operator's ambient git/ssh agent instead"
    )]
    EmbeddedCredential {
        /// The offending string.
        raw: String,
    },
    /// The host (or, for scp-like, the user) component is empty, or starts
    /// with `-` (an argument-injection shape).
    #[error("external Git locator {raw:?} has an empty or unsafe host/user component: {detail}")]
    UnsafeComponent {
        /// The offending string.
        raw: String,
        /// Which component, and why.
        detail: String,
    },
    /// The path component (after the host) is empty.
    #[error("external Git locator {raw:?} names no path")]
    EmptyPath {
        /// The offending string.
        raw: String,
    },
}

impl fmt::Display for LocatorForm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Https => "https",
            Self::Ssh => "ssh",
            Self::ScpLike => "scp-like",
        })
    }
}

/// Validate `raw` against the allowlist, refusing everything the module doc
/// names. Pure — no I/O, no Git invoked, nothing but string inspection.
pub fn validate(raw: &str) -> Result<ExternalGitLocator, LocatorError> {
    if raw.is_empty() {
        return Err(LocatorError::Empty);
    }
    if raw.chars().any(|c| c.is_control()) {
        return Err(LocatorError::ControlCharacter {
            raw: raw.to_string(),
        });
    }
    // The remote-helper marker, checked before the general whitespace
    // refusal below: `ext::sh -c '...'`-shaped input fails BOTH checks, and
    // the more specific, more useful diagnosis (this is the named A1-24
    // threat, not merely a malformed string) is the one worth surfacing.
    // Neither an accepted HTTPS/SSH URL nor a valid scp-like locator ever
    // legitimately contains a literal `::`.
    if raw.contains("::") {
        return Err(LocatorError::RemoteHelperSyntax {
            raw: raw.to_string(),
        });
    }
    if raw.chars().any(|c| c.is_whitespace()) {
        return Err(LocatorError::ControlCharacter {
            raw: raw.to_string(),
        });
    }

    if let Some(rest) = raw.strip_prefix("https://") {
        validate_url_authority(raw, rest, LocatorForm::Https)
    } else if let Some(rest) = raw.strip_prefix("ssh://") {
        validate_url_authority(raw, rest, LocatorForm::Ssh)
    } else if let Some(colon) = raw.find("://") {
        Err(LocatorError::UnsupportedScheme {
            raw: raw.to_string(),
            scheme: raw[..colon].to_string(),
        })
    } else {
        validate_scp_like(raw)
    }
}

/// `https://` or `ssh://`'s shared shape: `[user[:pass]@]host[:port]/path`.
/// `user:pass@` is refused outright (see module doc); a plain `user@` is
/// accepted for `ssh://` (identifies the remote login) and refused for
/// `https://` (an https locator has no legitimate use for an embedded
/// username either — the ambient credential helper is the only auth path).
fn validate_url_authority(
    raw: &str,
    rest: &str,
    form: LocatorForm,
) -> Result<ExternalGitLocator, LocatorError> {
    let Some(slash) = rest.find('/') else {
        return Err(LocatorError::EmptyPath {
            raw: raw.to_string(),
        });
    };
    let authority = &rest[..slash];
    let path = &rest[slash..];
    if path.len() <= 1 {
        // Just "/" — no repository path.
        return Err(LocatorError::EmptyPath {
            raw: raw.to_string(),
        });
    }
    let host_port = if let Some(at) = authority.find('@') {
        let userinfo = &authority[..at];
        if userinfo.contains(':') {
            return Err(LocatorError::EmbeddedCredential {
                raw: raw.to_string(),
            });
        }
        if form == LocatorForm::Https {
            return Err(LocatorError::UnsafeComponent {
                raw: raw.to_string(),
                detail: "https:// does not accept an embedded user; use the ambient credential \
                         helper"
                    .to_string(),
            });
        }
        if userinfo.is_empty() || userinfo.starts_with('-') {
            return Err(LocatorError::UnsafeComponent {
                raw: raw.to_string(),
                detail: format!("user component {userinfo:?} is empty or unsafe"),
            });
        }
        &authority[at + 1..]
    } else {
        authority
    };
    let host = host_port.split(':').next().unwrap_or("");
    if host.is_empty() || host.starts_with('-') {
        return Err(LocatorError::UnsafeComponent {
            raw: raw.to_string(),
            detail: format!("host component {host:?} is empty or unsafe"),
        });
    }
    Ok(ExternalGitLocator {
        raw: raw.to_string(),
        form,
    })
}

/// Git's scp-like shorthand: `user@host:path`, `user@` required (module
/// doc's own narrowing).
fn validate_scp_like(raw: &str) -> Result<ExternalGitLocator, LocatorError> {
    let Some(at) = raw.find('@') else {
        return Err(LocatorError::Unrecognized {
            raw: raw.to_string(),
        });
    };
    let user = &raw[..at];
    let rest = &raw[at + 1..];
    if user.is_empty() || user.starts_with('-') || user.contains(':') {
        return Err(LocatorError::UnsafeComponent {
            raw: raw.to_string(),
            detail: format!("user component {user:?} is empty, unsafe, or embeds a credential"),
        });
    }
    let Some(colon) = rest.find(':') else {
        return Err(LocatorError::Unrecognized {
            raw: raw.to_string(),
        });
    };
    let host = &rest[..colon];
    let path = &rest[colon + 1..];
    if host.is_empty() || host.starts_with('-') {
        return Err(LocatorError::UnsafeComponent {
            raw: raw.to_string(),
            detail: format!("host component {host:?} is empty or unsafe"),
        });
    }
    if path.is_empty() {
        return Err(LocatorError::EmptyPath {
            raw: raw.to_string(),
        });
    }
    Ok(ExternalGitLocator {
        raw: raw.to_string(),
        form: LocatorForm::ScpLike,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // --------------------------------------------------------- accepted forms

    #[test]
    fn accepted_https_forms() {
        for raw in [
            "https://github.com/example/repo.git",
            "https://gitlab.example.com:8443/group/sub/repo.git",
            "https://example.com/repo",
        ] {
            let locator = validate(raw).unwrap_or_else(|e| panic!("{raw} refused: {e}"));
            assert_eq!(locator.form, LocatorForm::Https);
            assert_eq!(locator.raw, raw);
        }
    }

    #[test]
    fn accepted_ssh_forms() {
        for raw in [
            "ssh://git@github.com/example/repo.git",
            "ssh://example.com/repo.git",
            "ssh://git@example.com:2222/repo.git",
        ] {
            let locator = validate(raw).unwrap_or_else(|e| panic!("{raw} refused: {e}"));
            assert_eq!(locator.form, LocatorForm::Ssh);
        }
    }

    #[test]
    fn accepted_scp_like_forms() {
        for raw in [
            "git@github.com:example/repo.git",
            "user@example.com:path/to/repo.git",
        ] {
            let locator = validate(raw).unwrap_or_else(|e| panic!("{raw} refused: {e}"));
            assert_eq!(locator.form, LocatorForm::ScpLike);
        }
    }

    // ---------------------------------------------------------- refused forms

    #[test]
    fn ext_remote_helper_is_refused() {
        let err = validate("ext::sh -c 'touch /tmp/pwned'").expect_err("must refuse");
        assert!(
            matches!(err, LocatorError::RemoteHelperSyntax { .. }),
            "{err}"
        );
    }

    #[test]
    fn any_double_colon_is_refused_even_disguised() {
        for raw in ["fd::/proc/self/exe", "foo::bar@baz:qux", "https://x::y"] {
            let err = validate(raw).expect_err("must refuse");
            assert!(
                matches!(err, LocatorError::RemoteHelperSyntax { .. }),
                "{raw}: {err}"
            );
        }
    }

    #[test]
    fn file_scheme_is_refused() {
        let err = validate("file:///etc/passwd").expect_err("must refuse");
        assert!(
            matches!(err, LocatorError::UnsupportedScheme { .. }),
            "{err}"
        );
    }

    #[test]
    fn bare_local_path_is_refused() {
        let err = validate("/etc/passwd").expect_err("must refuse");
        assert!(matches!(err, LocatorError::Unrecognized { .. }), "{err}");
        let err = validate("../escape/repo.git").expect_err("must refuse");
        assert!(matches!(err, LocatorError::Unrecognized { .. }), "{err}");
    }

    #[test]
    fn plaintext_and_daemon_and_ftp_schemes_are_refused() {
        for raw in [
            "http://example.com/repo.git",
            "git://example.com/repo.git",
            "ftp://example.com/repo.git",
            "ftps://example.com/repo.git",
        ] {
            let err = validate(raw).expect_err(raw);
            assert!(
                matches!(err, LocatorError::UnsupportedScheme { .. }),
                "{raw}: {err}"
            );
        }
    }

    #[test]
    fn embedded_credentials_are_refused_on_both_schemes() {
        let err = validate("https://user:hunter2@example.com/repo.git").expect_err("must refuse");
        assert!(
            matches!(err, LocatorError::EmbeddedCredential { .. }),
            "{err}"
        );
        let err = validate("ssh://user:hunter2@example.com/repo.git").expect_err("must refuse");
        assert!(
            matches!(err, LocatorError::EmbeddedCredential { .. }),
            "{err}"
        );
        let err = validate("user:hunter2@example.com:repo.git").expect_err("must refuse");
        assert!(matches!(err, LocatorError::UnsafeComponent { .. }), "{err}");
    }

    #[test]
    fn https_with_a_bare_username_is_refused() {
        // Not a credential (no colon), but https:// has no legitimate use for
        // an embedded user at all.
        let err = validate("https://user@example.com/repo.git").expect_err("must refuse");
        assert!(matches!(err, LocatorError::UnsafeComponent { .. }), "{err}");
    }

    #[test]
    fn a_host_starting_with_dash_is_refused_argument_injection() {
        let err = validate("ssh://-oProxyCommand=evil/repo.git").expect_err("must refuse");
        assert!(matches!(err, LocatorError::UnsafeComponent { .. }), "{err}");
        let err = validate("git@-oProxyCommand=evil:repo.git").expect_err("must refuse");
        assert!(matches!(err, LocatorError::UnsafeComponent { .. }), "{err}");
    }

    #[test]
    fn a_bare_host_path_scp_like_with_no_user_is_refused() {
        // Git itself accepts this; this build requires an explicit user@
        // (module doc's own narrowing).
        let err = validate("example.com:repo.git").expect_err("must refuse");
        assert!(matches!(err, LocatorError::Unrecognized { .. }), "{err}");
    }

    #[test]
    fn empty_and_whitespace_and_control_characters_are_refused() {
        assert!(matches!(validate(""), Err(LocatorError::Empty)));
        assert!(matches!(
            validate("https://example.com/re po.git"),
            Err(LocatorError::ControlCharacter { .. })
        ));
        assert!(matches!(
            validate("https://example.com/repo.git\n"),
            Err(LocatorError::ControlCharacter { .. })
        ));
        assert!(matches!(
            validate("https://example.com/re\0po.git"),
            Err(LocatorError::ControlCharacter { .. })
        ));
    }

    #[test]
    fn an_empty_path_is_refused() {
        let err = validate("https://example.com/").expect_err("must refuse");
        assert!(matches!(err, LocatorError::EmptyPath { .. }), "{err}");
        let err = validate("https://example.com").expect_err("must refuse");
        assert!(matches!(err, LocatorError::EmptyPath { .. }), "{err}");
    }

    #[test]
    fn raw_is_preserved_verbatim_for_provenance() {
        let raw = "https://github.com/example/repo.git";
        let locator = validate(raw).expect("valid");
        assert_eq!(locator.raw, raw, "provenance stores exactly what was typed");
    }
}
