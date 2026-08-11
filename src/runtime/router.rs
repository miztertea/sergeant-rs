//! Backend routing (proposal §13): origin affinity with an explicit
//! precedence chain, and no silent substitution.
//!
//! ```text
//! explicit --backend
//!         ↓
//! origin/client affinity
//!         ↓
//! workspace default
//!         ↓
//! global default
//!         ↓
//! fail with available options
//! ```
//!
//! The rule that gives the chain its meaning is §13's last constraint:
//! "sergeant never silently moves work to a different commercial provider
//! merely because one backend is unavailable." So a tier that *names* a
//! backend is decisive — if that backend is unknown or unavailable, routing
//! fails with the available options rather than falling through to the next
//! tier. Falling through would be the silent substitution.
//!
//! Origin affinity has one deliberate subtlety. §13's table maps
//! `Terminal → configured default`: an origin client that does not name a
//! backend (a terminal, an editor, a script) contributes no candidate and the
//! chain continues. An origin client that *does* name a registered backend —
//! a Claude front end with a Claude backend installed — is decisive like any
//! other tier. The distinction is "did the client ask for a provider", not
//! "is a provider available".

use serde::{Deserialize, Serialize};

use crate::backend::BackendRegistry;

/// Which precedence tier decided the route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteSource {
    /// An explicit `--backend` / request field.
    Explicit,
    /// The origin client's affinity (§13).
    OriginAffinity,
    /// The workspace's `default_backend` in `sergeant.toml`.
    WorkspaceDefault,
    /// The daemon's global default.
    GlobalDefault,
    /// A `[stage."<id>"] harness = …` declaration (§12.2). The highest tier
    /// there is: the workflow author said this checkpoint needs this harness.
    StageHarness,
    /// The Work-level actor default, inherited by a stage that named no
    /// harness of its own (§12.5's second tier). The Work-level chain above
    /// already decided *which* backend that is and recorded its own tier;
    /// this records that the stage took it rather than choosing.
    WorkActorDefault,
}

impl RouteSource {
    /// The tier's canonical snake_case name.
    pub fn as_str(self) -> &'static str {
        match self {
            RouteSource::Explicit => "explicit",
            RouteSource::OriginAffinity => "origin_affinity",
            RouteSource::WorkspaceDefault => "workspace_default",
            RouteSource::GlobalDefault => "global_default",
            RouteSource::StageHarness => "stage_harness",
            RouteSource::WorkActorDefault => "work_actor_default",
        }
    }
}

/// A resolved route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Route {
    /// Backend the work will execute on.
    pub backend: String,
    /// Tier that decided it.
    pub source: RouteSource,
}

/// The four tiers' inputs.
#[derive(Debug, Clone, Copy, Default)]
pub struct RouteInputs<'a> {
    /// Explicitly requested backend.
    pub explicit: Option<&'a str>,
    /// Origin client name from the request (§13's `origin.client`).
    pub origin_client: Option<&'a str>,
    /// Workspace default from `sergeant.toml`.
    pub workspace_default: Option<&'a str>,
    /// The daemon's global default.
    pub global_default: Option<&'a str>,
}

/// Why routing failed. Every variant carries the available options, because
/// §13's terminal state is "fail **with available options**".
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RouteError {
    /// A tier named a backend that is not registered.
    #[error(
        "no backend named {requested:?} ({tier} selection); available: {}",
        options(available)
    )]
    NotFound {
        /// The name that was asked for.
        requested: String,
        /// Tier that asked for it.
        tier: RouteSource,
        /// Registered backend names.
        available: Vec<String>,
    },
    /// A tier named a registered backend that cannot operate here. Routing
    /// stops: substituting another provider is exactly what §13 forbids.
    #[error(
        "backend {requested:?} ({tier} selection) is unavailable: {detail}; \
         sergeant will not substitute another backend. available: {}",
        options(available)
    )]
    Unavailable {
        /// The unavailable backend.
        requested: String,
        /// Tier that selected it.
        tier: RouteSource,
        /// The probe's explanation.
        detail: String,
        /// Registered backend names.
        available: Vec<String>,
    },
    /// No tier named anything.
    #[error(
        "no backend selected and no default configured; available: {}",
        options(available)
    )]
    NoSelection {
        /// Registered backend names.
        available: Vec<String>,
    },
}

impl RouteError {
    /// The structured error code the API answers with.
    pub fn code(&self) -> &'static str {
        match self {
            RouteError::NotFound { .. } => "backend_not_found",
            RouteError::Unavailable { .. } => "backend_unavailable",
            RouteError::NoSelection { .. } => "no_backend_selected",
        }
    }

    /// Backends the caller could have asked for.
    pub fn available(&self) -> &[String] {
        match self {
            RouteError::NotFound { available, .. }
            | RouteError::Unavailable { available, .. }
            | RouteError::NoSelection { available } => available,
        }
    }
}

fn options(available: &[String]) -> String {
    if available.is_empty() {
        "(none)".to_string()
    } else {
        available.join(", ")
    }
}

impl std::fmt::Display for RouteSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Resolve a backend for one submission (§13).
pub fn route(inputs: &RouteInputs<'_>, registry: &BackendRegistry) -> Result<Route, RouteError> {
    // Origin affinity contributes a candidate only when the client names a
    // backend sergeant actually knows about (§13's `Terminal → configured
    // default` row). Every other tier is decisive as written.
    let affinity = inputs
        .origin_client
        .filter(|client| registry.get(client).is_some());

    resolve_tiers(
        &[
            (RouteSource::Explicit, inputs.explicit),
            (RouteSource::OriginAffinity, affinity),
            (RouteSource::WorkspaceDefault, inputs.workspace_default),
            (RouteSource::GlobalDefault, inputs.global_default),
        ],
        registry,
    )
}

/// Resolve the harness for **one actor stage** (§12.5), given the Work-level
/// actor default the §13 chain already produced.
///
/// ```text
/// stage.harness explicitly names a harness
///         ↓
/// otherwise the Work actor default
///         ↓
/// fail with available harnesses
/// ```
///
/// This is [`route`]'s rule applied stage by stage, and it is the same rule
/// for the same reason: a tier that *names* a harness is decisive. §12.5 says
/// it in as many words — "if a workflow explicitly requires Claude for review
/// and Claude is unavailable, Sergeant does not silently use Codex" — so an
/// explicit stage harness that is unknown or unavailable fails here rather
/// than falling through to the Work default. Falling through would be exactly
/// the substitution §13 forbids, wearing a stage-shaped hat.
pub fn route_stage(
    stage_harness: Option<&str>,
    work_actor_default: Option<&str>,
    registry: &BackendRegistry,
) -> Result<Route, RouteError> {
    resolve_tiers(
        &[
            (RouteSource::StageHarness, stage_harness),
            (RouteSource::WorkActorDefault, work_actor_default),
        ],
        registry,
    )
}

/// The one tier walk both chains use: first tier that names something decides,
/// and a named backend that is unknown or unavailable fails the whole chain.
fn resolve_tiers(
    tiers: &[(RouteSource, Option<&str>)],
    registry: &BackendRegistry,
) -> Result<Route, RouteError> {
    let available = registry.names();
    for (source, requested) in tiers {
        let Some(requested) = requested else { continue };
        let Some(backend) = registry.get(requested) else {
            return Err(RouteError::NotFound {
                requested: (*requested).to_string(),
                tier: *source,
                available,
            });
        };
        let probe = backend.probe();
        if !probe.available {
            return Err(RouteError::Unavailable {
                requested: (*requested).to_string(),
                tier: *source,
                detail: probe
                    .detail
                    .unwrap_or_else(|| "backend probe reported unavailable".to_string()),
                available,
            });
        }
        return Ok(Route {
            backend: (*requested).to_string(),
            source: *source,
        });
    }
    Err(RouteError::NoSelection { available })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::backend::fake::FakeBackend;

    fn registry() -> BackendRegistry {
        BackendRegistry::new()
            .with(Arc::new(FakeBackend::new("claude")))
            .with(Arc::new(FakeBackend::new("codex")))
            .with(Arc::new(FakeBackend::new("fake")))
    }

    #[test]
    fn precedence_is_explicit_then_affinity_then_workspace_then_global() {
        let registry = registry();
        let all = RouteInputs {
            explicit: Some("claude"),
            origin_client: Some("codex"),
            workspace_default: Some("fake"),
            global_default: Some("fake"),
        };
        assert_eq!(
            route(&all, &registry).expect("explicit wins"),
            Route {
                backend: "claude".to_string(),
                source: RouteSource::Explicit
            }
        );

        let no_explicit = RouteInputs {
            explicit: None,
            ..all
        };
        assert_eq!(
            route(&no_explicit, &registry).expect("affinity wins"),
            Route {
                backend: "codex".to_string(),
                source: RouteSource::OriginAffinity
            }
        );

        let workspace = RouteInputs {
            explicit: None,
            origin_client: Some("terminal"),
            workspace_default: Some("claude"),
            global_default: Some("fake"),
        };
        assert_eq!(
            route(&workspace, &registry).expect("workspace wins"),
            Route {
                backend: "claude".to_string(),
                source: RouteSource::WorkspaceDefault
            }
        );

        let global = RouteInputs {
            explicit: None,
            origin_client: Some("terminal"),
            workspace_default: None,
            global_default: Some("fake"),
        };
        assert_eq!(
            route(&global, &registry).expect("global default"),
            Route {
                backend: "fake".to_string(),
                source: RouteSource::GlobalDefault
            }
        );
    }

    #[test]
    fn an_origin_client_that_names_no_backend_falls_through_to_the_default() {
        let registry = registry();
        let terminal = RouteInputs {
            origin_client: Some("terminal"),
            global_default: Some("fake"),
            ..RouteInputs::default()
        };
        assert_eq!(
            route(&terminal, &registry).expect("route").source,
            RouteSource::GlobalDefault
        );
    }

    #[test]
    fn an_unavailable_selection_never_substitutes_another_provider() {
        let claude = FakeBackend::new("claude");
        claude.set_available(false, "claude is not logged in");
        let registry = BackendRegistry::new()
            .with(Arc::new(claude))
            .with(Arc::new(FakeBackend::new("codex")));

        // Explicit, affinity and defaults all fail closed rather than
        // silently landing on the other provider.
        for inputs in [
            RouteInputs {
                explicit: Some("claude"),
                global_default: Some("codex"),
                ..RouteInputs::default()
            },
            RouteInputs {
                origin_client: Some("claude"),
                global_default: Some("codex"),
                ..RouteInputs::default()
            },
            RouteInputs {
                workspace_default: Some("claude"),
                global_default: Some("codex"),
                ..RouteInputs::default()
            },
        ] {
            let err = route(&inputs, &registry).expect_err("must not substitute");
            assert_eq!(err.code(), "backend_unavailable");
            assert!(err.to_string().contains("claude is not logged in"));
            assert_eq!(err.available(), ["claude".to_string(), "codex".to_string()]);
        }
    }

    /// §12.5's chain, stage by stage: an explicit stage harness decides, a
    /// stage that names none inherits the Work actor default, and a stage on
    /// a harness that is unknown or unavailable fails **without** falling
    /// through to the default. That last row is the whole point — falling
    /// through would run a checkpoint the author reserved for Claude on
    /// whatever else happened to be configured.
    #[test]
    fn a_stage_harness_decides_and_never_falls_through_to_the_work_default() {
        let claude = FakeBackend::new("claude");
        claude.set_available(false, "claude is not logged in");
        let registry = BackendRegistry::new()
            .with(Arc::new(claude))
            .with(Arc::new(FakeBackend::new("codex")))
            .with(Arc::new(FakeBackend::new("fake")));

        assert_eq!(
            route_stage(Some("codex"), Some("fake"), &registry).expect("stage wins"),
            Route {
                backend: "codex".to_string(),
                source: RouteSource::StageHarness
            }
        );
        assert_eq!(
            route_stage(None, Some("fake"), &registry).expect("inherits the default"),
            Route {
                backend: "fake".to_string(),
                source: RouteSource::WorkActorDefault
            }
        );

        let unavailable = route_stage(Some("claude"), Some("codex"), &registry)
            .expect_err("must not substitute the default for a named stage harness");
        assert_eq!(unavailable.code(), "backend_unavailable");
        assert!(unavailable.to_string().contains("claude is not logged in"));

        let unknown = route_stage(Some("opencode"), Some("fake"), &registry)
            .expect_err("an unregistered stage harness is not a fallback");
        assert_eq!(unknown.code(), "backend_not_found");
        assert_eq!(
            unknown.available(),
            [
                "claude".to_string(),
                "codex".to_string(),
                "fake".to_string()
            ]
        );

        // Neither tier named anything: §13's terminal state, unchanged.
        assert_eq!(
            route_stage(None, None, &registry)
                .expect_err("nothing selected")
                .code(),
            "no_backend_selected"
        );
    }

    #[test]
    fn unknown_and_absent_selections_fail_with_the_options() {
        let registry = registry();
        let unknown = RouteInputs {
            explicit: Some("opencode"),
            global_default: Some("fake"),
            ..RouteInputs::default()
        };
        let err = route(&unknown, &registry).expect_err("unknown backend");
        assert_eq!(err.code(), "backend_not_found");
        assert_eq!(
            err.available(),
            [
                "claude".to_string(),
                "codex".to_string(),
                "fake".to_string()
            ]
        );

        let err = route(&RouteInputs::default(), &registry).expect_err("nothing selected");
        assert_eq!(err.code(), "no_backend_selected");
        assert!(err.to_string().contains("claude, codex, fake"));
    }
}
