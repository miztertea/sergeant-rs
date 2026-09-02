//! Hand-counted Rust fixture for the F5 corpus gate.
//!
//! Not compiled by anything: `tests/fixtures/` is not a cargo target root, so
//! this file is bytes the corpus suite reads, never a module.

use std::collections::HashMap;
use std::fmt::{self, Display};

pub const LIMIT: usize = 8;
static NAME: &str = "fixture";

pub type Pairs = HashMap<String, usize>;

pub struct Counter {
    hits: usize,
}

pub enum Outcome {
    Hit,
    Miss,
}

pub trait Countable {
    fn count(&self) -> usize;
}

impl Counter {
    pub fn new() -> Self {
        Counter { hits: 0 }
    }

    pub fn bump(&mut self) {
        self.hits += 1;
    }
}

impl Countable for Counter {
    fn count(&self) -> usize {
        self.hits
    }
}

impl Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::Hit => write!(f, "hit"),
            Outcome::Miss => write!(f, "miss"),
        }
    }
}

pub mod nested {
    pub fn helper() -> u8 {
        1
    }

    pub struct Inner;
}

macro_rules! twice {
    ($e:expr) => {
        $e + $e
    };
}

pub fn main_like() -> usize {
    let mut c = Counter::new();
    c.bump();
    let _ = NAME;
    let _ = LIMIT;
    twice!(c.count())
}
