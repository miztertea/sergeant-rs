# F5 gate 2 — hand-verified fixture corpus, recorded run (2026-08-30)

Companion to `tests/w3b_semantic_retrieval.rs`, which holds the corpus and
the queries as source and is the thing to re-run. This file is the recorded
output of that suite on the day it landed: **the numbers a reviewer checks
the ranking against.**

Gate 2, from the wave brief: *"Fidelity checked against a human-readable
fixture, not 'it compiled'. Semantically related code/text ranks above
unrelated, verifiably, on fixtures a reviewer can read."*

Lane `/var/tmp/hats5/w3b`, base `6b232209`, `rustc 1.98.0`,
`TMPDIR=/var/tmp/sgt-test-tmp`, debug build. Model
`minishlab/potion-code-16M-v2` at revision
`e9d2a44ca6a05ac6685f3b23709ea57eb7352d5b`, content hash
`blake3:6795c49688a583ba631121b89bd146a78f234ee9ed99ce7153e99f118f407c5d`.

Reproduce:

```sh
TMPDIR=/var/tmp/sgt-test-tmp cargo nextest run --test w3b_semantic_retrieval --no-capture
```

## The corpus

Nine one-sentence documents. Five are the intended answer to one query each;
four (`garden/roses.md`, `kernel/scheduling.md`, `kitchen/sourdough.md`,
`music/tuning.md`) are the unrelated documents every expected answer must
beat. Full text is in the test file's `CORPUS` constant — deliberately there
rather than duplicated here, so there is one copy of the fixture.

## Recorded rankings — all five cases, cosine, best first

```text
QUERY "how do we retry a failed payment charge"  (status Applied)
   +0.3688  knowledge:payments/decline-handling.md      <- expected
   +0.1494  knowledge:architecture/adr-042.md
   +0.1459  knowledge:security/never-commit-keys.md
   +0.1295  knowledge:kitchen/sourdough.md
   +0.1091  knowledge:ops/disk-pressure.md
   +0.0601  knowledge:config/loader.md
   +0.0411  knowledge:kernel/scheduling.md
   +0.0410  knowledge:music/tuning.md
   -0.0198  knowledge:garden/roses.md
QUERY "parse a JSON configuration file into a struct"  (status Applied)
   +0.3385  knowledge:config/loader.md                  <- expected
   +0.3320  knowledge:security/never-commit-keys.md
   +0.0551  knowledge:kernel/scheduling.md
   +0.0346  knowledge:music/tuning.md
   +0.0268  knowledge:garden/roses.md
   +0.0110  knowledge:architecture/adr-042.md
   +0.0093  knowledge:payments/decline-handling.md
   -0.0282  knowledge:kitchen/sourdough.md
   -0.0304  knowledge:ops/disk-pressure.md
QUERY "what did we decide about asynchronous settlement"  (status Applied)
   +0.6013  knowledge:architecture/adr-042.md           <- expected
   +0.2496  knowledge:payments/decline-handling.md
   +0.1665  knowledge:kernel/scheduling.md
   +0.1606  knowledge:kitchen/sourdough.md
   +0.1459  knowledge:config/loader.md
   +0.0470  knowledge:ops/disk-pressure.md
   +0.0064  knowledge:security/never-commit-keys.md
   +0.0064  knowledge:music/tuning.md
   -0.0296  knowledge:garden/roses.md
QUERY "running out of storage space"  (status Applied)
   +0.1796  knowledge:ops/disk-pressure.md              <- expected
   +0.1652  knowledge:config/loader.md
   +0.1042  knowledge:kernel/scheduling.md
   +0.1008  knowledge:architecture/adr-042.md
   +0.0274  knowledge:payments/decline-handling.md
   +0.0267  knowledge:security/never-commit-keys.md
   +0.0035  knowledge:music/tuning.md
   -0.0560  knowledge:garden/roses.md
   -0.0697  knowledge:kitchen/sourdough.md
QUERY "avoid putting credentials in source control"  (status Applied)
   +0.1885  knowledge:security/never-commit-keys.md     <- expected
   +0.0687  knowledge:music/tuning.md
   +0.0606  knowledge:config/loader.md
   +0.0261  knowledge:garden/roses.md
   +0.0219  knowledge:payments/decline-handling.md
   +0.0218  knowledge:kernel/scheduling.md
   +0.0142  knowledge:kitchen/sourdough.md
   -0.0213  knowledge:architecture/adr-042.md
   -0.0351  knowledge:ops/disk-pressure.md
```

**5 of 5 expected answers at rank 1**, and each above all four unrelated
documents. Margins over the runner-up range from +0.002 (case 2) to +0.35
(case 3) — case 2's margin is thin and is stated rather than smoothed over.

## Why this is a test of embeddings and not of word overlap

The same five queries through W2's BM25 half, same corpus
(`the_gate_2_wins_are_semantic_because_bm25_alone_does_not_produce_them`):

```text
QUERY "how do we retry a failed payment charge"
   bm25 first: knowledge:payments/decline-handling.md     agrees
QUERY "parse a JSON configuration file into a struct"
   bm25 first: knowledge:security/never-commit-keys.md    WRONG (expected config/loader.md)
QUERY "what did we decide about asynchronous settlement"
   bm25 first: knowledge:architecture/adr-042.md          agrees
QUERY "running out of storage space"
   bm25 first: knowledge:config/loader.md                 WRONG (expected ops/disk-pressure.md)
QUERY "avoid putting credentials in source control"
   bm25 first: knowledge:garden/roses.md                  WRONG (expected security/never-commit-keys.md)
```

**BM25 alone: 2 of 5. Semantic: 5 of 5.** The three BM25 misses are what
makes gate 2 a fidelity measurement rather than a restatement of the lexical
half, and the assertion that keeps it that way is in the suite: if BM25 ever
reproduced all five, the test fails and demands the corpus be rewritten.

## A2 §8's negative, non-vacuously

The decoy is an `external`-authority document restating case 1's answer. With
the filter open it ranks **first**; with `source=knowledge` it is gone and the
admissible answer is first:

```text
QUERY "unfiltered"  (status Applied)
   +0.8783  vendor-lib:docs/leak.md                     <- inadmissible, wins
   +0.3688  knowledge:payments/decline-handling.md
   ...
QUERY "source=knowledge"  (status Applied)
   +0.3688  knowledge:payments/decline-handling.md      <- decoy absent
   ...
```

## Recorded misses — what this model does NOT do

Kept because a fidelity gate whose corpus was tuned until nothing failed
would be measuring the tuning.

| Query | Expected | What the model returned first | Score of expected |
|---|---|---|---|
| `do not leak secrets` | `security/never-commit-keys.md` | `architecture/adr-042.md` (+0.1629) | +0.0867, rank 5 of 9 |

`potion-code-16M-v2` is a **code**-tuned static embedding model. Terse,
abstract, jargon-light phrasings ("do not leak secrets") map poorly; a
phrasing with ordinary domain nouns ("avoid putting credentials in source
control") lands at rank 1 with **zero** word overlap with the document. That
is the shape of the model's competence, recorded so a later reader does not
have to rediscover it, and it is one input to any future decision about a
second or different model (out of scope here — the wave brief lists "a second
embedding model" as not in scope).

## Incidental finding, kept

The corpus originally used the path `security/credentials.md`. It never
reached the index: A1's F10 secrets floor denies `**/credentials.*` at the
acquisition boundary (`runtime::atlas::deny::DEFAULT_DENY`). The gate-2 test
found it by failing on a missing document. The floor worked; the fixture was
renamed, and the reason is a comment in the test so nobody renames it back.
