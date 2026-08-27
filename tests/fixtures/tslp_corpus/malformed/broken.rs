// Deliberately unparseable Rust, for the corpus gate's red side.
//
// F5's criterion is "a parse error on any fixture is a FAIL, not a skip". A
// suite that only ever asserts "no fixture errored" is vacuous unless
// something proves the extractor can still say "this errored" — that is what
// this file, and its sibling broken.py, are for.
pub fn broken( {
    let = ;
}
