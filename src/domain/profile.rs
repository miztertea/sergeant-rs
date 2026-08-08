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
}
