# S4 Y3 (G5) — hand-verified ZIP fixture corpus

Wave brief `brief-y3-bounded-zip.md`. Sibling record: `build_zip_fixtures.py`,
which is what actually produced every `.zip` under `zip_fixtures/`.

## Why this file exists, and in what order it was produced

Every fixture is built with Python's standard-library `zipfile` module
(`build_zip_fixtures.py`), never by `sergeant-rs`'s own extractor, which does
not exist at the point this corpus was authored. The expected counts below
were read off `zipfile`'s own `ZipFile.infolist()` — independently, a second
way, not by running `archive::expand` and copying its answer — and are
reproducible by anyone with Python's stdlib and no other dependency:

```
$ cd zip_fixtures && python3 - <<'PY'
import zipfile, stat
for name in sorted(__import__("os").listdir(".")):
    with zipfile.ZipFile(name) as z:
        print(name, len(z.infolist()))
        for info in z.infolist():
            mode = (info.external_attr >> 16) & 0xFFFF
            print(" ", repr(info.filename), info.file_size, info.compress_size,
                  "dir" if info.is_dir() else ("symlink" if stat.S_ISLNK(mode) else "file"))
PY
```

Verbatim output at authoring time (2026-08-28):

```
01-plain-and-directory.zip 4
  'readme.txt' 17 19 file
  'notes/' 0 2 dir
  'notes/a.md' 10 12 file
  'notes/b.txt' 16 18 file
02-symlink.zip 2
  'safe.txt' 17 17 file
  'escape-link' 22 22 symlink
03-duplicate-name.zip 2
  'dup.txt' 16 18 file
  'dup.txt' 34 36 file
04-high-ratio-bomb.zip 1
  'bomb.bin' 8388608 8158 file
05-nested-inner.zip 1
  'leaf.md' 22 24 file
06-nested-outer.zip 1
  'inner.zip' 136 136 file
```

## Pass criterion, per fixture

**`01-plain-and-directory.zip`** — every entry admitted except the directory
marker, which is its own `Coverage::Discovered` row: 3 `ZipChild`s
(`readme.txt`, `notes/a.md`, `notes/b.txt`), 1 `Discovered` coverage row
(`notes/`), zero refusals.

**`02-symlink.zip`** — 1 `ZipChild` (`safe.txt`); `escape-link` produces a
`Coverage::Excluded` row naming "symlink" and NO child. `zipfile.ZipInfo`'s
own `external_attr` was set to `(0o120777) << 16` — `S_IFLNK | rwxrwxrwx` —
the exact bit shape `zip` 8.6.0's own `unix_mode()`/`is_symlink()` read
(VERIFIED against the crate's source, `y3-zip-bounds-research.md` and
`runtime/atlas/archive.rs`'s own module doc).

**`03-duplicate-name.zip`** — 1 `ZipChild` (`dup.txt`, content
`b"first occurrence"` — the FIRST occurrence by index, `zipfile` itself warns
`UserWarning: Duplicate name: 'dup.txt'` at build time but writes both
records anyway, exactly the shape `y3-zip-bounds-research.md` §1 describes:
the format does not forbid duplicate names). The second record produces a
`Coverage::Excluded` row naming "duplicate".

**`04-high-ratio-bomb.zip`** — a REAL DEFLATE stream, not a forged header:
8 MiB of one repeated byte compresses to 8158 bytes, a genuine ≈1028:1
ratio, comfortably past `MAX_COMPRESSION_RATIO` (200:1). Zero `ZipChild`s;
one `Coverage::Unsupported` row naming "MAX_COMPRESSION_RATIO", refused
before any byte is decompressed.

**`05-nested-inner.zip`** — not itself iterated directly by the worker test;
it exists to become `06-nested-outer.zip`'s one entry's own bytes. Read
directly, it has exactly one admitted child, `leaf.md`.

**`06-nested-outer.zip`** — 1 `ZipChild` (`inner.zip`, `is_nested_archive =
true`, depth 1 which is within `MAX_NESTING_DEPTH` = 2), whose own `nested`
expansion has exactly 1 grandchild (`leaf.md`, admitted from
`05-nested-inner.zip`'s own bytes, chained key per `child_key`'s own
contract).

## Not in this corpus, and why

**An empty-name entry.** `zipfile.ZipFile.writestr("", ...)` refuses an empty
`arcname` outright (`ValueError`), and `zip` 8.6.0's own writer likewise
refuses to `start_file` with an empty name (VERIFIED while authoring
`archive.rs`'s own test fixtures) — an empty *declared* name cannot survive
either library's own writer, only a hand-spliced central-directory record
with a zeroed name-length field can produce one on the READ side.
`runtime/atlas/archive.rs`'s own `an_empty_name_is_refused` test builds
exactly that byte-spliced fixture directly (not persisted here, since it is
not independently reproducible by a stdlib `zipfile` writer the way every
other fixture in this corpus is) and is this claim's decisive check.

**The overlapping/self-referential (quine) fixture.** Same reasoning: no
ordinary writer (Python's `zipfile`, or `zip`'s own `ZipWriter`) can produce
two central-directory records that legitimately share a compressed-data
range — the construction is inherently a hand-spliced one.
`runtime/atlas/archive.rs`'s own
`overlapping_files_refuse_the_whole_archive_before_any_entry_opens` builds it
directly, sanity-checks it against `zip`'s own `has_overlapping_files()`
before asserting anything about `archive::expand`'s behaviour (so a fixture
bug cannot masquerade as a finding about the crate), and is this wave's
decisive check for the research's closed open item.

**A genuinely corrupt/malformed archive** (not a hostile-but-well-formed one
like `04-high-ratio-bomb.zip`) is exercised directly in
`runtime/atlas/archive.rs`'s `a_genuinely_corrupt_archive_is_refused_not_panicked_on`
against a plain non-ZIP byte string — no fixture file needed for that claim
either.
