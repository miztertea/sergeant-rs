//! Profiles: named launch configuration (proposal §14).
//!
//! §14's boundary is the whole point: "Depot should **not** become an
//! authentication broker. The native harness owns credentials." A profile
//! carries only what is needed to *launch* a backend — which backend, which
//! executable, which config home, environment overrides, a default model,
//! runtime options.
//!
//! "No credentials, ever" is answered *structurally*, by the type below:
//! [`Profile`] has no credential field, and `deny_unknown_fields` means a
//! `sergeant.toml` cannot invent one. What it deliberately does **not** do is
//! inspect the values a user puts in `env` and `options` and guess which of
//! them look secret. That guess has no true positive available to it — a
//! profile is checked-in workspace configuration, so anything in it is
//! already in the user's repository whatever sergeant thinks — and it has
//! real false positives: `GIT_AUTHOR_NAME` is launch configuration for the
//! work branch's commit identity, and any substring rule blunt enough to
//! catch `GH_AUTH_TOKEN` catches it too. Sergeant refuses to become the
//! credential layer by not having one, not by lint.

use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A named execution context (§14): everything needed to launch a backend,
/// and nothing that authenticates one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    /// Profile name, referenced by `--profile`.
    pub name: String,
    /// Backend this profile launches.
    pub backend: String,
    /// Executable to launch instead of the backend's default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executable: Option<PathBuf>,
    /// Config/home location for the harness, when it needs its own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_home: Option<PathBuf>,
    /// Environment overrides applied to the launched harness.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
    /// Model the backend should default to for this profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    /// Backend-specific runtime options.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub options: BTreeMap<String, String>,
}

/// The `permission_mode` option key inside [`Profile::options`] (#47).
pub const PERMISSION_MODE_OPTION: &str = "permission_mode";

/// The claude CLI's own `--permission-mode` vocabulary, as far as this
/// adapter passes it through (§14, #47). Nothing here is invented: each
/// variant is one value the CLI accepts for `--permission-mode`, plus
/// [`PermissionMode::Default`] — an explicit way to ask for the CLI's own
/// unflagged default, distinct from leaving the option unset.
///
/// This is a closed set on purpose. Passing an unrecognized string straight
/// through to the CLI would trade a structured, load-time refusal for
/// whatever the CLI itself does with a bad `--permission-mode` value — a
/// harness-specific and unmeasured failure mode. Refusing here, at profile
/// load, means the CLI's own argument parser never has to be the thing that
/// catches a typo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionMode {
    /// No `--permission-mode` flag at all — the CLI's own default, chosen
    /// explicitly rather than left unspecified.
    Default,
    /// `--permission-mode acceptEdits`.
    AcceptEdits,
    /// `--permission-mode bypassPermissions`. Never the adapter's own
    /// default — a profile must name this explicitly (#47).
    BypassPermissions,
    /// `--permission-mode dontAsk`.
    DontAsk,
    /// `--permission-mode plan`.
    Plan,
}

impl PermissionMode {
    /// Parse one of the five vocabulary strings, case-sensitive (the CLI's
    /// own casing — `acceptEdits`, not `acceptedits` or `accept-edits`).
    pub fn parse(raw: &str) -> Result<Self, UnknownPermissionMode> {
        match raw {
            "default" => Ok(Self::Default),
            "acceptEdits" => Ok(Self::AcceptEdits),
            "bypassPermissions" => Ok(Self::BypassPermissions),
            "dontAsk" => Ok(Self::DontAsk),
            "plan" => Ok(Self::Plan),
            other => Err(UnknownPermissionMode {
                value: other.to_string(),
            }),
        }
    }

    /// The CLI argv this mode contributes: nothing for [`Self::Default`]
    /// (the CLI's own unflagged behavior), `--permission-mode <value>`
    /// otherwise.
    pub fn cli_args(self) -> Vec<String> {
        match self {
            Self::Default => Vec::new(),
            other => vec![
                "--permission-mode".to_string(),
                other.as_cli_value().to_string(),
            ],
        }
    }

    /// The literal value this mode passes to `--permission-mode`, for
    /// display (doctor, error messages) even though [`Self::Default`] never
    /// reaches argv.
    pub fn as_cli_value(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::AcceptEdits => "acceptEdits",
            Self::BypassPermissions => "bypassPermissions",
            Self::DontAsk => "dontAsk",
            Self::Plan => "plan",
        }
    }
}

/// The five values [`PermissionMode::parse`] accepts, for error messages.
pub const PERMISSION_MODE_VALUES: &[&str] = &[
    "default",
    "acceptEdits",
    "bypassPermissions",
    "dontAsk",
    "plan",
];

/// A profile named a `permission_mode` this build does not recognize.
///
/// Deliberately fails closed rather than passing the raw string through to
/// the CLI: an unrecognized `--permission-mode` value is the CLI's problem to
/// diagnose, at token-spending launch time, in a place the operator may not
/// even see (a daemon's own stderr). Catching it at profile load turns that
/// into a same-second, no-cost refusal that names the profile and the value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownPermissionMode {
    /// The value that did not match any of [`PERMISSION_MODE_VALUES`].
    pub value: String,
}

impl fmt::Display for UnknownPermissionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown permission_mode {:?}; expected one of: {}",
            self.value,
            PERMISSION_MODE_VALUES.join(", ")
        )
    }
}

impl std::error::Error for UnknownPermissionMode {}

impl Profile {
    /// This profile's validated `permission_mode` (#47).
    ///
    /// `Ok(None)` means the option is simply absent — unspecified, which the
    /// adapter treats identically to `Ok(Some(PermissionMode::Default))`:
    /// no `--permission-mode` flag, never a silent
    /// `--dangerously-skip-permissions`. The two are kept distinct here
    /// (rather than collapsing "unset" into `Default` at this layer) so a
    /// caller that cares about the difference — doctor, reporting whether a
    /// profile made a choice at all — still can.
    pub fn permission_mode(&self) -> Result<Option<PermissionMode>, UnknownPermissionMode> {
        match self.options.get(PERMISSION_MODE_OPTION) {
            None => Ok(None),
            Some(raw) => PermissionMode::parse(raw).map(Some),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The §14 boundary is structural: the record has launch fields and no
    /// credential field, and `deny_unknown_fields` refuses to grow one.
    #[test]
    fn a_profile_carries_launch_configuration_and_cannot_declare_a_credential() {
        let profile: Profile = toml::from_str(
            "name = \"enterprise\"\nbackend = \"claude\"\n\
             default_model = \"claude-opus-4-7\"\n\
             [env]\nCLAUDE_CONFIG_DIR = \"/home/u/.claude-work\"\n\
             GIT_AUTHOR_NAME = \"sergeant\"\n",
        )
        .expect("launch configuration loads");
        assert_eq!(profile.default_model.as_deref(), Some("claude-opus-4-7"));
        assert_eq!(profile.env["CLAUDE_CONFIG_DIR"], "/home/u/.claude-work");
        assert_eq!(
            profile.env["GIT_AUTHOR_NAME"], "sergeant",
            "commit identity for the work branch is launch configuration (§14)"
        );

        let err = toml::from_str::<Profile>(
            "name = \"leaky\"\nbackend = \"claude\"\napi_key = \"sk-nope\"\n",
        )
        .expect_err("there is no field to put a credential in");
        assert!(
            err.to_string().contains("api_key"),
            "the diagnostic must name the refused field: {err}"
        );
    }

    /// #47: unspecified and each of the five vocabulary strings all resolve
    /// distinctly, and an unrecognized value fails closed rather than riding
    /// through to the CLI unchecked.
    #[test]
    fn permission_mode_is_the_closed_cli_vocabulary_or_a_structured_refusal() {
        let unspecified = Profile {
            name: "n".to_string(),
            backend: "claude".to_string(),
            executable: None,
            config_home: None,
            env: BTreeMap::new(),
            default_model: None,
            options: BTreeMap::new(),
        };
        assert_eq!(
            unspecified
                .permission_mode()
                .expect("unset is not an error"),
            None,
            "unspecified must not silently become a mode"
        );

        for (raw, expected) in [
            ("default", PermissionMode::Default),
            ("acceptEdits", PermissionMode::AcceptEdits),
            ("bypassPermissions", PermissionMode::BypassPermissions),
            ("dontAsk", PermissionMode::DontAsk),
            ("plan", PermissionMode::Plan),
        ] {
            let mut profile = unspecified.clone();
            profile
                .options
                .insert(PERMISSION_MODE_OPTION.to_string(), raw.to_string());
            assert_eq!(
                profile.permission_mode().expect("a listed value parses"),
                Some(expected),
                "{raw:?} must parse as {expected:?}"
            );
        }

        // Default's argv is empty — it is the CLI's own unflagged behavior,
        // chosen explicitly — while every other mode carries the flag.
        assert!(PermissionMode::Default.cli_args().is_empty());
        assert_eq!(
            PermissionMode::BypassPermissions.cli_args(),
            vec![
                "--permission-mode".to_string(),
                "bypassPermissions".to_string()
            ]
        );

        let mut bad = unspecified;
        bad.options
            .insert(PERMISSION_MODE_OPTION.to_string(), "yolo".to_string());
        let err = bad
            .permission_mode()
            .expect_err("an unrecognized value must be refused");
        assert_eq!(err.value, "yolo");
        assert!(
            err.to_string().contains("yolo"),
            "the diagnostic must name the offending value: {err}"
        );
        for known in PERMISSION_MODE_VALUES {
            assert!(
                err.to_string().contains(known),
                "the diagnostic must list the valid vocabulary ({known}): {err}"
            );
        }
    }
}
