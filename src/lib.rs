//! sergeant-rs library surface.
//!
//! The binary (`sgt`) is a thin shell over this crate; keeping the modules in
//! a library target lets integration tests exercise the event core directly.

pub mod api;
pub mod backend;
pub mod cli;
pub mod daemon;
pub mod domain;
pub mod runtime;
pub mod telemetry;
pub mod tui;
pub mod web;
