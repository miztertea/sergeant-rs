#!/usr/bin/env python3
"""Captures golden chunk boundaries from semble's ACTUAL chunker
(semble.chunking.chunking.chunk_source / semble.chunking.core) run against
this corpus's own fixtures, and writes one <fixture>.golden.json per
fixture beside it.

This does not reimplement or approximate the reference algorithm -- it
calls it, live, per the wave brief's "semble's own output is the oracle"
test discipline (knowledge/evidence/resources/host-atlas-s6-series/
brief-chunker-port.md, "Test discipline" section). The Rust port's tests
assert against the committed *.golden.json files this script produced,
not against a re-derived expectation.

Most fixtures in this corpus are ASCII-only (see each fixture's own header
comment), where char offsets (what semble's Python chunker natively
produces) and byte offsets (what the Rust port natively produces)
coincide exactly. `sample_multibyte.toml` (chunker-utf8 wave) is not --
this script always derives byte offsets by UTF-8-encoding the source up
to each chunk boundary rather than assuming char offset == byte offset,
so the goldens it writes are correct for both ASCII and multi-byte input
alike.

Usage: python3 build_chunk_goldens.py
"""
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))

# semble's chunking module is installed as a uv tool, not on sys.path by
# default (~/.claude/CLAUDE.md's own documented install shape).
SEMBLE_SITE_PACKAGES = os.path.expanduser(
    "~/.local/share/uv/tools/semble/lib/python3.14/site-packages"
)
sys.path.insert(0, SEMBLE_SITE_PACKAGES)

from semble.chunking.chunking import (  # noqa: E402
    _DESIRED_CHUNK_LENGTH_CHARS,
    chunk_source,
)

FIXTURES = [
    ("sample.rs", "rust"),
    ("sample.toml", "toml"),
    ("sample.sh", "bash"),
    ("sample.py", "python"),
    ("sample.txt", None),  # no grammar -- exercises the line-fallback path
    ("sample_multibyte.toml", "toml"),  # chunker-utf8 wave: non-ASCII density
]


def main() -> None:
    assert _DESIRED_CHUNK_LENGTH_CHARS == 750, (
        "brief-chunker-port.md cites chunking.py::_DESIRED_CHUNK_LENGTH_CHARS "
        f"= 750; installed semble reports {_DESIRED_CHUNK_LENGTH_CHARS} -- "
        "the port's ported constant would silently drift from the oracle"
    )

    for filename, language in FIXTURES:
        path = os.path.join(HERE, filename)
        with open(path, encoding="utf-8") as f:
            source = f.read()

        chunks = chunk_source(source, filename, language)
        # Chunk (from semble.types) does not carry raw byte offsets, only
        # content + line numbers -- recompute byte offsets from content by
        # re-finding each chunk's text in the source. `source.index` and
        # `len(c.content)` are Python **char** offsets/lengths; for
        # multi-byte content those diverge from the byte offsets the Rust
        # port actually produces (chunker-utf8 wave: an earlier version of
        # this script used the char offset directly as the byte offset,
        # which is only correct for ASCII-only fixtures and silently wrong
        # otherwise -- caught by `toml_multibyte_fixture_matches_semble_oracle`
        # failing against a golden built that way). So the char offset is
        # used only to locate the chunk in `source`; the byte offset is
        # then derived by encoding the text up to that point.
        golden = []
        char_cursor = 0
        for c in chunks:
            char_start = source.index(c.content, char_cursor)
            char_end = char_start + len(c.content)
            start = len(source[:char_start].encode("utf-8"))
            end = start + len(c.content.encode("utf-8"))
            golden.append(
                {
                    "byte_start": start,
                    "byte_end": end,
                    "line_start": c.start_line,
                    "line_end": c.end_line,
                    "content": c.content,
                }
            )
            char_cursor = char_end  # chunks are emitted in non-decreasing
            # order by construction (both the AST-merge and line-merge
            # paths walk the source once, forward); searching from the
            # previous chunk's end keeps two chunks with identical content
            # (e.g. two blank-ish lines) resolved in the correct
            # left-to-right order instead of both matching the first
            # occurrence.

        out_path = os.path.join(HERE, filename + ".golden.json")
        with open(out_path, "w", encoding="utf-8") as f:
            json.dump(golden, f, indent=2)
            f.write("\n")
        print(f"{filename}: {len(source)} chars -> {len(chunks)} chunk(s) -> {out_path}")


if __name__ == "__main__":
    main()
