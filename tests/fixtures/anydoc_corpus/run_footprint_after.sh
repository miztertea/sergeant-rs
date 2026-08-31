#!/usr/bin/env bash
set -e
export TMPDIR=/var/tmp/sgt-test-tmp
export CARGO_TARGET_DIR=/var/tmp/sgt-footprint-after
cd /var/tmp/hats4/y2
rm -rf "$CARGO_TARGET_DIR"
{ time cargo build --locked --tests ; } 2> /var/tmp/sgt-footprint-after-tests-time.txt
{ time cargo build --locked ; } 2> /var/tmp/sgt-footprint-after-build-time.txt
du -sb "$CARGO_TARGET_DIR" > /var/tmp/sgt-footprint-after-du.txt
stat -c %s "$CARGO_TARGET_DIR/debug/sgt" > /var/tmp/sgt-footprint-after-binsize-naive.txt
nm "$CARGO_TARGET_DIR/debug/sgt" 2>/dev/null | grep -ci "anydoc\|pdf_inspector\|lopdf" > /var/tmp/sgt-footprint-after-nm-anydoc-count.txt || true
echo DONE > /var/tmp/sgt-footprint-after-DONE.marker
