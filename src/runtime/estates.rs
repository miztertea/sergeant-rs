//! The admitted-estate registry (H1 §4, sprint-plan D3/D4/D10).
//!
//! One host daemon serves many estates. Nothing about that set is baked into
//! the process at startup: a daemon comes up bound to **zero** estates — the
//! normal state, not a test-rig edge case — and learns each one the first
//! time a request addresses it. That is what "observational" means in H1 §4:
//! the registry records estates that were *validated*, never estates that
//! were inferred, discovered by search, or trusted from the wire.
//!
//! Three rules the whole module exists to keep:
//!
//! 1. **Admission is the validation.** An estate enters this map only by
//!    passing [`Estate::admit`]'s exact-root check (§4.1: the exact
//!    directory, no parent search, no Git inference) *and*
//!    `refuse_if_unreliable` for that root's own filesystem — different
//!    estates on different filesystems are real, so the check that used to
//!    run once for the one data dir now runs per root.
//! 2. **A failed admission is a named, estate-specific refusal, never
//!    process death.** The pre-H1 daemon refused to *start* when its one
//!    estate did not admit ([`crate::daemon::start_with`]'s old step 0a).
//!    With N estates that would let one broken `sergeant.toml` take down
//!    every other estate's Work, so admission failure is answered to the
//!    caller that asked for it and nothing else.
//! 3. **No persistence, and no estate UUID (H1-06).** The registry rebuilds
//!    itself from requests; what makes Work↔estate durable is the journal's
//!    own `workspace_id` coordinate (D1), which is a canonical root, not an
//!    identifier this process mints.
//!
//! **The retired refusal class.** Before H1 the client-side gate was
//! `RuntimeDescriptor::check_estate_root`: strict equality against the one
//! root the descriptor published, refusing in both directions ("this daemon
//! is bound to a different estate", "this daemon is bound to no estate").
//! That class is retired here, deliberately and with the reasoning recorded
//! for the W5 ADR: under H1 a daemon serving an estate it was not started
//! from is the *point*, not the bug. What replaces it is
//! [`EstateAdmissionError`]'s taxonomy — "not an estate", "admission
//! failed", "no estate addressed" — which keeps the property the old check
//! actually protected (a request is never served for an estate nobody
//! validated) without the property H1 removes (one process, one estate).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::domain::estate::{Estate, EstateRoot, EstateRootError};
use crate::domain::event::rfc3339_utc_now;
use crate::platform::fs_locking::{self, Reliability};

/// Why a request addressed at an estate could not be served.
///
/// The taxonomy the client gate and the daemon-side registry share, because
/// they are asking the same question from two sides (sprint-plan D4, brief
/// deliverable 4):
///
/// - (a) the requested root is not a valid estate → [`Self::NotAnEstate`],
///   carrying [`Estate::admit`]'s own §4.4 diagnostic verbatim;
/// - (b) it is a plausible estate but admission failed for a reason of this
///   registry's own → [`Self::UnreliableFilesystem`];
/// - (c) an estate-scoped operation named no estate at all →
///   [`Self::NoEstateAddressed`].
#[derive(Debug, thiserror::Error)]
pub enum EstateAdmissionError {
    /// (c) — H1 §11 criterion 3 preserved: an estate-scoped operation with
    /// no estate to operate on fails closed rather than picking one.
    #[error(
        "{operation} is estate-scoped, but no estate was addressed\n\n\
         The daemon is host-scoped: it serves every estate that has been admitted to it, and \
         therefore never assumes which one a request means.\n\
         Name one:\n  \
         sgt -C <estate-root> <command>\n  \
         or run the command from the estate root itself."
    )]
    NoEstateAddressed {
        /// What was being attempted, for the message.
        operation: String,
    },
    /// (a) — the exact-root check refused. The inner error is §4.4's full
    /// corrective block; it is never summarized, because the whole value of
    /// exact-root admission is that the refusal teaches the operator where
    /// they actually are.
    #[error("cannot admit the estate at {root}\n\n{source}")]
    NotAnEstate {
        /// The directory that was addressed, as given.
        root: PathBuf,
        /// §4.4's own diagnostic.
        #[source]
        source: Box<EstateRootError>,
    },
    /// (b) — #85 / ADR 0003 D6, per estate root rather than once for the one
    /// data dir. An estate on a filesystem where advisory locking is
    /// unreliable cannot have its repository locks trusted, so it is refused
    /// at admission rather than served and quietly raced.
    #[error(
        "the estate at {root} sits on a {filesystem} filesystem, where advisory locking is \
         unreliable; refusing to admit it — {remedy}"
    )]
    UnreliableFilesystem {
        /// The estate root that was refused.
        root: PathBuf,
        /// The offending filesystem type, as the mount table names it.
        filesystem: String,
        /// What to do about it.
        remedy: String,
    },
}

impl EstateAdmissionError {
    /// Stable per-variant code, for the `{"error": {"code", "message"}}`
    /// shape every API refusal answers with.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NoEstateAddressed { .. } => "no_estate",
            Self::NotAnEstate { .. } => "invalid_estate",
            Self::UnreliableFilesystem { .. } => "unreliable_filesystem",
        }
    }
}

/// One admitted estate, as the registry holds it.
///
/// Everything here is read **at admission time** from the manifest that made
/// the root an estate — the same "re-read the manifest, do not cache it from
/// process startup" discipline `Engine::plan` has always had, now applied per
/// estate rather than once per daemon (brief deliverable 8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedEstate {
    /// Canonical estate root — the coordinate (D1, H1-06: no UUID).
    pub root: PathBuf,
    /// `<root>/sergeant.toml`.
    pub manifest_path: PathBuf,
    /// `[estate] name`, display only (D1: never identity).
    pub name: String,
    /// `[estate] retention`, read per estate at admission. W4a owns
    /// partitioning the retention *enforcement*; this wave owns the read.
    pub retention: Option<u32>,
    /// `[estate] surfaces_dir`, resolved absolute. Narrows the daemon-wide
    /// surfaces root per estate (H1-07: Work surfaces stay estate-local).
    pub surfaces_dir: Option<PathBuf>,
    /// RFC3339 UTC time this root was first admitted to this process.
    pub admitted_at: String,
}

/// A registry row as `GET /v1/estates` reports it: the admitted facts plus
/// whether the estate still validated the last time anything touched it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EstateEntry {
    /// The estate as admitted.
    pub estate: AdmittedEstate,
    /// `false` once a re-validation has failed — the estate is still in the
    /// registry (its Work is still journaled against it), but nothing will
    /// mutate through it until it validates again.
    pub available: bool,
    /// Why it is unavailable, when it is.
    pub unavailable_reason: Option<String>,
    /// RFC3339 UTC time of the last admission or re-validation attempt.
    pub last_touched_at: String,
}

/// The daemon's admitted-estate registry: in-memory, keyed by canonical
/// root, rebuilt from requests.
#[derive(Debug, Default)]
pub struct EstateRegistry {
    entries: Mutex<BTreeMap<PathBuf, EstateEntry>>,
}

impl EstateRegistry {
    /// An empty registry — a freshly started host daemon's normal state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Admit `root`, or refresh what is already admitted.
    ///
    /// Lazy by design: this is called on the first estate-addressed request
    /// and on every one after it, so a manifest edited while the daemon runs
    /// is picked up without a restart (the existing `plan()`-re-reads-the-
    /// manifest discipline, now per estate). The canonical root is the map
    /// key, so two spellings of the same directory are one entry, not two.
    pub fn admit(&self, root: &Path) -> Result<AdmittedEstate, EstateAdmissionError> {
        let outcome = admit_root(root);
        let now = rfc3339_utc_now();
        let mut entries = self.entries.lock().expect("estate registry lock");
        match outcome {
            Ok(admitted) => {
                let entry = entries
                    .entry(admitted.root.clone())
                    .or_insert_with(|| EstateEntry {
                        estate: admitted.clone(),
                        available: true,
                        unavailable_reason: None,
                        last_touched_at: now.clone(),
                    });
                // An already-admitted estate keeps its original
                // `admitted_at` — when this process first validated it is a
                // fact, and a later refresh is not a new admission — but
                // every other field is what the manifest says *now*.
                let admitted_at = entry.estate.admitted_at.clone();
                entry.estate = AdmittedEstate {
                    admitted_at,
                    ..admitted
                };
                entry.available = true;
                entry.unavailable_reason = None;
                entry.last_touched_at = now;
                Ok(entry.estate.clone())
            }
            Err(e) => {
                // Only *known* roots are marked unavailable. A request naming
                // a directory that was never an estate must not create a
                // registry row — the registry records what was admitted, and
                // an admission that never happened is not an observation.
                if let Some(entry) = canonical_key(root).and_then(|key| entries.get_mut(&key)) {
                    entry.available = false;
                    entry.unavailable_reason = Some(e.to_string());
                    entry.last_touched_at = now;
                }
                Err(e)
            }
        }
    }

    /// Re-validate an estate before a *resumed* mutation (recovery's path).
    ///
    /// Identical to [`Self::admit`] in mechanism and deliberately so: H1 §4's
    /// "re-validated before resumed mutation" is exactly "admit it again,
    /// now" — a Work resuming against an estate whose manifest has since been
    /// deleted or broken must fail closed with a reason, not act on the
    /// admission a previous process life recorded.
    pub fn revalidate(&self, root: &Path) -> Result<AdmittedEstate, EstateAdmissionError> {
        self.admit(root)
    }

    /// Every row, in canonical-root order.
    pub fn entries(&self) -> Vec<EstateEntry> {
        self.entries
            .lock()
            .expect("estate registry lock")
            .values()
            .cloned()
            .collect()
    }

    /// How many estates this daemon has admitted — `sgt daemon stop`'s blast
    /// radius (D5).
    pub fn len(&self) -> usize {
        self.entries.lock().expect("estate registry lock").len()
    }

    /// Whether nothing has been admitted yet.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// The exact-root admission check, shared by the client gate and the
/// registry: [`Estate::admit`] (§4.1), then this root's own filesystem
/// reliability (#85 / ADR 0003 D6, per root), then the per-estate policy the
/// manifest declares.
pub fn admit_root(root: &Path) -> Result<AdmittedEstate, EstateAdmissionError> {
    let EstateRoot {
        path,
        manifest_path,
    } = Estate::admit(root).map_err(|source| EstateAdmissionError::NotAnEstate {
        root: root.to_path_buf(),
        source: Box::new(source),
    })?;
    if let Reliability::Unreliable { filesystem } = fs_locking::detect_for_path(&path) {
        return Err(EstateAdmissionError::UnreliableFilesystem {
            remedy: fs_locking::remedy(&filesystem),
            root: path,
            filesystem,
        });
    }
    // `Estate::admit` proved the manifest parses structurally; this reads the
    // policy fields out of the same file. A failure here is unreachable in
    // practice (the parse just succeeded) and is treated as "no declared
    // policy" rather than a new refusal, exactly as `start_with`'s old
    // per-daemon retention read did.
    let declared = Estate::from_config_structural(&manifest_path).ok();
    Ok(AdmittedEstate {
        name: declared
            .as_ref()
            .map(|e| e.name.clone())
            .unwrap_or_default(),
        retention: declared.as_ref().and_then(|e| e.retention),
        surfaces_dir: declared.as_ref().and_then(|e| e.surfaces_dir.clone()),
        admitted_at: rfc3339_utc_now(),
        root: path,
        manifest_path,
    })
}

/// The client-side gate, replacing `RuntimeDescriptor::check_estate_root`.
///
/// The old gate compared the client's root against the one root the
/// descriptor published. A v3 descriptor publishes no root at all (D3), so
/// what is left to check is the only thing that was ever load-bearing: is the
/// estate this command addresses actually an estate, admissible right now?
/// `None` — an estate-scoped operation that named none — is refusal (c).
pub fn check_estate_root(
    wanted: Option<&Path>,
    operation: &str,
) -> Result<AdmittedEstate, EstateAdmissionError> {
    let Some(wanted) = wanted else {
        return Err(EstateAdmissionError::NoEstateAddressed {
            operation: operation.to_string(),
        });
    };
    admit_root(wanted)
}

/// Canonicalize for map lookup, matching [`Estate::admit`]'s own fallback
/// (an unresolvable path is used as given rather than erroring here — the
/// admission itself is what refuses it).
fn canonical_key(root: &Path) -> Option<PathBuf> {
    Some(std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scaffold(root: &Path, name: &str) {
        std::fs::create_dir_all(root.join("repos").join("solo")).expect("mount dir");
        std::fs::write(
            root.join("sergeant.toml"),
            format!("[estate]\nname = {name:?}\n"),
        )
        .expect("manifest");
    }

    /// H1 §4: a daemon starts bound to zero estates and learns each one on
    /// first contact; the canonical root is the key, so admitting the same
    /// estate twice is one row, not two, and the first admission's timestamp
    /// survives the refresh.
    #[test]
    fn admission_is_lazy_keyed_by_canonical_root_and_idempotent() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path().join("estate");
        scaffold(&root, "payments");
        let registry = EstateRegistry::new();
        assert!(registry.is_empty(), "a fresh daemon has admitted nothing");

        let first = registry.admit(&root).expect("admit");
        assert_eq!(first.name, "payments");
        assert_eq!(
            first.root,
            std::fs::canonicalize(&root).expect("canonical"),
            "the coordinate is the canonical root (D1), never the spelling asked for"
        );
        assert_eq!(registry.len(), 1);

        // A second spelling of the same directory is the same estate.
        let again = registry
            .admit(&root.join(".").join("..").join("estate"))
            .expect("re-admit");
        assert_eq!(registry.len(), 1, "one canonical root, one row");
        assert_eq!(
            again.admitted_at, first.admitted_at,
            "a refresh is not a new admission"
        );
    }

    /// Refusal (a): a directory that is not an estate never becomes a
    /// registry row, and the refusal carries §4.4's own diagnostic rather
    /// than a summary of it.
    #[test]
    fn a_directory_that_is_not_an_estate_is_refused_and_never_recorded() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let registry = EstateRegistry::new();
        let err = registry
            .admit(dir.path())
            .expect_err("a bare directory is not an estate");
        assert_eq!(err.code(), "invalid_estate");
        assert!(
            err.to_string()
                .contains("does not search parent directories"),
            "§4.4's own block must survive verbatim, got: {err}"
        );
        assert!(
            registry.is_empty(),
            "a refused admission is not an observation"
        );
    }

    /// Refusal (c): an estate-scoped operation that addressed no estate is
    /// refused by name (H1 §11 criterion 3 preserved), and says how to name
    /// one.
    #[test]
    fn an_estate_scoped_operation_with_no_estate_addressed_is_refused() {
        let err = check_estate_root(None, "sgt run").expect_err("no estate addressed");
        assert_eq!(err.code(), "no_estate");
        let message = err.to_string();
        assert!(
            message.contains("sgt run") && message.contains("sgt -C <estate-root>"),
            "the refusal names the operation and the remedy, got: {message}"
        );
    }

    /// H1 §4: one estate going bad is that estate's refusal and nothing
    /// else's — the registry marks the known row unavailable with the reason
    /// and leaves every other admitted estate exactly where it was.
    #[test]
    fn a_broken_estate_goes_unavailable_without_touching_its_neighbour() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let payments = dir.path().join("payments");
        let billing = dir.path().join("billing");
        scaffold(&payments, "payments");
        scaffold(&billing, "billing");
        let registry = EstateRegistry::new();
        registry.admit(&payments).expect("admit payments");
        registry.admit(&billing).expect("admit billing");

        std::fs::remove_file(payments.join("sergeant.toml")).expect("break payments");
        let err = registry
            .admit(&payments)
            .expect_err("a vanished manifest must refuse");
        assert_eq!(err.code(), "invalid_estate");

        let rows = registry.entries();
        assert_eq!(rows.len(), 2, "the broken estate stays in the registry");
        let payments_row = rows
            .iter()
            .find(|r| r.estate.name == "payments")
            .expect("payments row");
        assert!(!payments_row.available);
        assert!(
            payments_row
                .unavailable_reason
                .as_deref()
                .is_some_and(|r| r.contains("no estate found")),
            "the row records why, got: {:?}",
            payments_row.unavailable_reason
        );
        let billing_row = rows
            .iter()
            .find(|r| r.estate.name == "billing")
            .expect("billing row");
        assert!(
            billing_row.available,
            "one estate's failure is not another's"
        );
        registry.admit(&billing).expect("billing still admits");
    }

    /// Brief deliverable 8: per-estate policy is read at admission, from
    /// that estate's own manifest — not once, daemon-wide, at startup.
    #[test]
    fn per_estate_policy_is_read_from_each_manifest_at_admission() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let tight = dir.path().join("tight");
        let loose = dir.path().join("loose");
        std::fs::create_dir_all(&tight).expect("dir");
        std::fs::create_dir_all(&loose).expect("dir");
        std::fs::write(
            tight.join("sergeant.toml"),
            "[estate]\nname = \"tight\"\nretention = 500\nsurfaces_dir = \"work-surfaces\"\n",
        )
        .expect("manifest");
        std::fs::write(loose.join("sergeant.toml"), "[estate]\nname = \"loose\"\n")
            .expect("manifest");

        let registry = EstateRegistry::new();
        let tight = registry.admit(&tight).expect("admit tight");
        let loose = registry.admit(&loose).expect("admit loose");
        assert_eq!(tight.retention, Some(500));
        assert_eq!(
            tight.surfaces_dir,
            Some(
                std::fs::canonicalize(dir.path())
                    .expect("canonical")
                    .join("tight")
                    .join("work-surfaces")
            )
        );
        assert_eq!(loose.retention, None, "each estate answers for itself");
        assert_eq!(loose.surfaces_dir, None);
    }
}
