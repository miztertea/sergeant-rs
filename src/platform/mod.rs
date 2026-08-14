//! The platform boundary (ADR 0002): a standard API over `#[cfg]`-selected
//! modules, not a trait. Platform is a compile-time fact — nobody selects one
//! at runtime the way `--backend` selects a [`crate::backend::Backend`] impl
//! — so this module buys none of the dispatch a `Platform` trait would, and
//! none of its category errors (a `WindowsPlatform` instantiated on a Linux
//! host) either. See the ADR for the full argument and its precedents
//! (`Reducer` as a plain fn pointer; `TtyWatch::watching` in `src/tui.rs`).
//!
//! Each fact below follows the same shape (ADR 0002 D3): the decision logic
//! is a plain function, unconditionally compiled and unit-tested from
//! whatever host happens to be building this crate, and only the raw
//! syscall/shell-out underneath it is `#[cfg]`-gated per platform. That is
//! what makes a macOS arm's *logic* reviewable — and its fail-closed path
//! exercisable — from a Linux box, even though the macOS arm's actual `df`/
//! `ps`/`kill` invocation cannot be.
//!
//! The measured targets are Linux and macOS (ADR 0001 D1; WSL2 counts as
//! Linux underneath, so it needs no third arm). **Every macOS-specific arm in
//! this module is marked UNVERIFIED**: there is no macOS host in this estate
//! yet (`docs/environments/`), and `docs/DEVELOPMENT.md`'s rule is
//! measured-not-assumed. Those arms close their tracking issue (#81, #82,
//! #18) only once someone runs the suite on a real macOS host and records it
//! there — not when this module lands.

pub mod data_dir;
pub mod disk;
pub mod process;
