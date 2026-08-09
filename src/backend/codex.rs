//! Codex backend — **descoped per deviation D6** (owner decision
//! 2026-08-08), and therefore deliberately empty.
//!
//! D6: Claude is this prototype's only native adapter until an environment
//! exists where Codex can actually be measured. The M4 contract records the
//! same ruling ("`backend/codex.rs` stays a stub (or is removed) with a doc
//! comment citing D6. Any Codex protocol code present in the working tree is
//! out of scope and should be flagged/removed"), and drops the Codex fixture
//! tests from its acceptance list.
//!
//! The rationale is the measure-first doctrine (LESSONS L1) sharpened by L7:
//! adapter code that cannot be validated against the real harness is prose
//! with a compiler. A request builder, a decoder and a normalization table
//! authored from documentation would advertise capabilities nothing has
//! measured — exactly the "documented means supported" inversion the §15
//! contract's honesty rule forbids. So this file holds no protocol layer, no
//! `Backend` implementation, and registers nothing: an unmeasured backend is
//! absent, not unavailable-but-listed.
//!
//! The §15 trait is backend-neutral, so a future Codex adapter is purely
//! additive — it arrives with an installed binary, contract tests against it,
//! and a measured minimum trusted version, the way `backend/claude.rs` did.
