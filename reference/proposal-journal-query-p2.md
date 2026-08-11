---
type: proposal
title: "Sergeant-rs P2-JOURNAL: Queryable Durable Trajectory"
description: >-
  Proposal for a typed, read-only Journal query surface over Sergeant's
  append-only event history and disposable DuckDB projection, preserving
  journal-sequence provenance while making cross-Work search and investigation
  available through the daemon API and CLI. A future TUI Journal view may
  consume the same contract, but the TUI redesign remains a separate proposal.
status: proposed
resource: sergeant-rs
tags:
  - sergeant-rs
  - journal
  - duckdb
  - trajectory
  - search
  - query
  - api
  - cli
  - proposal
timestamp: 2026-08-11
repository: https://github.com/miztertea/sergeant-rs
audit_revision: dcc963568668479e42b2ece9b17a05c53e86144a
concurrent_owner_input: ff6bc55f6308b38cc442b7c6ef091c5d840246a7
relationship: >-
  Companion to reference/proposal-depot-rust-execution-surface.md §§18–23
  and the separately proposed T-series terminal interface. This proposal adds
  a core read/query capability and a CLI rendering; it does not implement or
  amend the TUI redesign, the web dashboard, retention policy, or workflow
  execution semantics.
---

# Sergeant-rs P2-JOURNAL
## Queryable Durable Trajectory

**Status:** Proposed  
**Audit basis:** [`miztertea/sergeant-rs@dcc9635`](https://github.com/miztertea/sergeant-rs/tree/dcc963568668479e42b2ece9b17a05c53e86144a)  
**Concurrent owner-approved inputs:** [PR #58 at `ff6bc55`](https://github.com/miztertea/sergeant-rs/pull/58), including the adjudicated North Star, retention ruling, and separately proposed T-series document  
**Relationship to P0:** Complete the read side implied by §§18–23; preserve the P0 journal, projection, graph, and one-owner contracts  
**Proposed program identifier:** `P2-JOURNAL`  
**Primary objective:** Let an operator find and inspect normalized trajectory evidence across all Work without reading journal files, decoding blobs, or writing SQL  
**Primary product surface:** Typed daemon query API plus `sgt journal`  
**Future client:** A `Journal` destination in the separately proposed TUI, consuming the same API contract  
**Hard boundary:** No arbitrary SQL, semantic/vector search, blob-content indexing, new event families, new authority model, saved-query store, alert engine, or second source of truth  

Sections are numbered for contract citation, following the repository's proposal convention.

---

# 1. Executive Summary

Sergeant already preserves the thing most agent products lose: the durable trajectory of the work.

Every accepted state change is an immutable event in a crash-tolerant, segmented NDJSON journal. The journal records sequence, time, source, Work, execution, correlation, causation, event kind, and kind-specific payload. Large evidence remains in the content-addressed blob store. On daemon start, the complete journal is folded into current Work state, DuckDB analytical tables, and the temporal graph. The architecture is already explicit:

```text
JSONL journal    durable trajectory and only truth
DuckDB           disposable projection and query engine
Graph            disposable relationship projection
Blob store       large referenced evidence
```

That division was not incidental. The original proposal says the three read questions are different:

```text
what happened?                         JSONL chronology
how are the things related?            graph
what patterns exist across executions? DuckDB analytics
```

Yet the current operator surface exposes only fragments of that promise.

`GET /v1/events` can replay after a sequence, optionally filter one Work, and retain only the newest matching rows. It is a transport and evidence-tail endpoint, not a cross-history query interface. DuckDB is currently reachable only through five canned aggregate questions and one-Work graph neighborhoods. `Analytics::table_rows` exists strictly as a test instrument and deliberately has no production route. A human looking for a prior question, failure, tool call, backend behavior, stage decision, or repeated phrase must already know the Work, guess a sequence window, or inspect journal and blob evidence manually.

P2-JOURNAL closes that read-side gap without weakening Sergeant's architecture.

It introduces a typed, read-only Journal query contract:

```text
operator query
    ↓
validated JournalQuery
    ↓
parameterized, allow-listed DuckDB query
    ↓
sequence-anchored JournalHit rows
    ↓
optional exact-event lookup from the authoritative journal
```

The query result remains event-centered. DuckDB may find, filter, and enrich a row, but the row's evidentiary identity is always the journal sequence. The search surface does not manufacture a new history, infer hidden reasoning, or promote a projection into truth.

The initial query language is deliberately bounded. It supports literal text search over normalized event-local content and structured filters over event envelope and existing projection fields:

```text
text
kind
work_id
execution_id
workflow
stage_id
backend
repository
source_type / source_name
correlation_id / causation_id
tool
model
sequence and time bounds
```

It does not accept SQL, regex, fuzzy matching, semantic embeddings, arbitrary JSON paths, negation, grouped Boolean expressions, or user-defined computed fields. Different filters combine with `AND`; repeated values within one field combine with `OR`.

The API shape is two read-only routes:

```text
GET /v1/journal           bounded search and pagination
GET /v1/journal/{seq}     exact authoritative event by sequence
```

The CLI renders that same contract:

```text
sgt journal
sgt journal "retry budget"
sgt journal "ambiguous revision" --workflow repo-to-icm
sgt journal --kind conversation.ask --backend claude
sgt journal --work <id> --stage 80-adversarial-review
sgt journal --seq 1284
```

A future TUI may add:

```text
Home    Fleet    Workflows    Journal
```

with a query bar, structured filters, a result list, and an exact-event inspector. That presentation is not implemented here. The T-series proposal remains independently reviewable and independently gated. P2-JOURNAL supplies the core read capability that any terminal, browser, harness, or future client would consume.

The first implementation does **not** begin with DuckDB's full-text-search extension. DuckDB's official FTS extension creates an index that does not update automatically when the source table changes, which would create a second refresh obligation beside Sergeant's existing journal-to-projection catch-up. It also introduces tokenization, stemming, ranking, and extension-loading behavior before literal evidence search has been measured. P2-JOURNAL starts with static, parameterized SQL over the existing `events` table and measures the real journals at 50,000 and 1,000,000 events. Only a measured budget failure may license a stronger search index.

The central rule is:

> **DuckDB finds the evidence. The journal proves it.**

Every normative decision is assigned its Ponytail Minimality Ladder rung in §20.

---

# 2. Audit Basis and Method

## 2.1 Code and record basis

This proposal was audited against current `main` at commit [`dcc963568668479e42b2ece9b17a05c53e86144a`](https://github.com/miztertea/sergeant-rs/commit/dcc963568668479e42b2ece9b17a05c53e86144a).

That revision includes:

- the complete P0 journal, projection, graph, API, CLI, TUI, and dashboard substrate;
- the P1 performance harness and baselines;
- the S-series coverage and stabilization work;
- N3's two-phase external-effect boundary, actor-authored asks, per-stage executor binding, and group-commit follow-on;
- the Cerberus performance re-baseline and live-adapter remediation line.

The audit also uses owner-adjudicated, documentation-only inputs currently carried by [PR #58](https://github.com/miztertea/sergeant-rs/pull/58):

- `NORTH-STAR.md`;
- the retention design ruling;
- the separately proposed T-series terminal interface;
- the U-series scope draft and dogfood evidence.

Those documents are treated as concurrent owner direction, not as code already present at the audit revision. Where this proposal depends on a rule from that lane, it names that status.

## 2.2 Repository material reviewed

The review included:

- [`CLAUDE.md`](https://github.com/miztertea/sergeant-rs/blob/dcc963568668479e42b2ece9b17a05c53e86144a/CLAUDE.md), especially journal truth, one-owner storage, equal clients, and code-as-code;
- [`reference/proposal-depot-rust-execution-surface.md`](https://github.com/miztertea/sergeant-rs/blob/dcc963568668479e42b2ece9b17a05c53e86144a/reference/proposal-depot-rust-execution-surface.md), especially §§18–23;
- [`src/domain/event.rs`](https://github.com/miztertea/sergeant-rs/blob/dcc963568668479e42b2ece9b17a05c53e86144a/src/domain/event.rs), the forward-compatible event envelope;
- [`src/runtime/journal.rs`](https://github.com/miztertea/sergeant-rs/blob/dcc963568668479e42b2ece9b17a05c53e86144a/src/runtime/journal.rs), including segmented replay and `replay_after`;
- [`src/runtime/analytics.rs`](https://github.com/miztertea/sergeant-rs/blob/dcc963568668479e42b2ece9b17a05c53e86144a/src/runtime/analytics.rs), including schema, catch-up, canned queries, raw-table refusal, and graph storage;
- [`src/runtime/graph.rs`](https://github.com/miztertea/sergeant-rs/blob/dcc963568668479e42b2ece9b17a05c53e86144a/src/runtime/graph.rs), especially `source_seq` provenance and absent node families;
- [`src/api.rs`](https://github.com/miztertea/sergeant-rs/blob/dcc963568668479e42b2ece9b17a05c53e86144a/src/api.rs), including current routes, projection catch-up, event history, and lock boundaries;
- [`src/cli.rs`](https://github.com/miztertea/sergeant-rs/blob/dcc963568668479e42b2ece9b17a05c53e86144a/src/cli.rs), for current command grammar and `--json` behavior;
- [`docs/perf/baseline-cerberus-2026-08-11.md`](https://github.com/miztertea/sergeant-rs/blob/dcc963568668479e42b2ece9b17a05c53e86144a/docs/perf/baseline-cerberus-2026-08-11.md), the current real-host performance record;
- issues [#7](https://github.com/miztertea/sergeant-rs/issues/7), [#8](https://github.com/miztertea/sergeant-rs/issues/8), [#10](https://github.com/miztertea/sergeant-rs/issues/10), and [#17](https://github.com/miztertea/sergeant-rs/issues/17);
- the dogfood finding that transcript evidence is not human-legible without blob decoding, preserved in [PR #58's run manifest](https://github.com/miztertea/sergeant-rs/blob/cerberus/day2-adjudications/docs/gauntlet/runs/dogfood-2026-08-11/run-manifest.md);
- the separately proposed T-series document delivered by the owner and carried in [PR #58](https://github.com/miztertea/sergeant-rs/pull/58).

## 2.3 IdeaOS and research material reviewed

The proposal uses the following IdeaOS records as design lenses, not as license to turn Sergeant into Garrison:

- [Ponytail Minimality Ladder](https://app.notion.com/p/39a27ada618f8100babadb321a70de9b);
- [IdeaOS Agent Instructions](https://app.notion.com/p/39a27ada618f815aab89daafc635514f);
- [Work-Centered Intelligence](https://app.notion.com/p/3ac27ada618f81728a73fbd7ac90c61c);
- [Portable Trajectory Record Format](https://app.notion.com/p/39a27ada618f81ef9658e33adce377ee);
- [Behavioral Execution Records](https://app.notion.com/p/39a27ada618f81a9bcaaf749d663cef0);
- [Privacy-Preserving Agent Observability](https://app.notion.com/p/39a27ada618f81238f9bf4699f5d8309);
- [Ecological Interface Design: Theoretical Foundations](https://app.notion.com/p/3ac27ada618f81909dd5d48e1f9b9912);
- [Shared-Engine Human-Agent Workbench](https://app.notion.com/p/39a27ada618f81999694e0fbb019ca50).

The Portable Trajectory record makes a distinction directly relevant here: an append-only event stream preserves chronology and interoperability, while readable snapshots, metrics, and diagrams are derived views. It also warns that a useful trajectory is broader than a final answer but narrower than indiscriminate capture. P2-JOURNAL applies that principle to Sergeant's existing record; it does not define a new portable trajectory schema.

## 2.4 DuckDB primary sources reviewed

The implementation alternatives were checked against current DuckDB-owned documentation:

- [Full-Text Search Extension](https://duckdb.org/docs/lts/core_extensions/full_text_search);
- [Full-Text Search guide](https://duckdb.org/docs/stable/guides/sql_features/full_text_search);
- [Prepared Statements](https://duckdb.org/docs/lts/sql/query_syntax/prepared_statements);
- [JSON Processing Functions](https://duckdb.org/docs/lts/data/json/json_functions);
- [Text Functions](https://duckdb.org/docs/current/sql/functions/text);
- [Indexes](https://duckdb.org/docs/current/sql/indexes);
- [Indexing performance guide](https://duckdb.org/docs/lts/guides/performance/indexing).

These sources establish that DuckDB already supplies parameter binding, literal string operations, JSON extraction, automatically maintained zonemaps, and optional ART/FTS indexes. They also establish the important FTS limitation: an FTS index does not update automatically when its input table changes.

## 2.5 Evidence hierarchy

The proposal uses this order:

```text
current implementation at the audit revision
        ↓
committed contracts, ledger, lessons, and measured baselines
        ↓
owner-adjudicated concurrent records
        ↓
official dependency documentation
        ↓
IdeaOS research and adjacent-system concepts
        ↓
proposal hypothesis
```

Where a lower layer conflicts with the implementation, the implementation wins unless an owner ruling explicitly changes the contract.

**Decision P2-01 (R2):** P2-JOURNAL extends the current journal/DuckDB/API pattern. It does not introduce a second database, search service, or client-side storage path.

---

# 3. Doctrine

## 3.1 Query evidence; do not create facts

**Decision P2-02 (R2):** The unit returned by Journal search is an existing event anchored by its journal sequence.

DuckDB may:

- select events;
- apply exact filters;
- join existing projection context;
- identify which field matched;
- calculate a bounded excerpt;
- order and paginate rows.

It may not:

- infer an event that was never written;
- infer hidden reasoning;
- assign success or causation not present in the record;
- claim file, artifact, commit, or finding facts whose event families do not exist;
- rewrite chronology for relevance.

The graph already embodies the same rule: every edge carries the journal `source_seq` that justifies it. Journal query applies that inspectability rule to search results.

## 3.2 The journal is truth; DuckDB is an answer engine

**Decision P2-03 (R2):** Search executes against DuckDB, but every hit names the journal sequence from which it was derived.

The query surface never opens `sergeant.duckdb` from a client. Only the daemon owns the connection. A missing or failed DuckDB projection makes the answer unavailable; it does not damage Work or history. Catch-up or restart rebuilds it from the journal.

```text
journal event
    ↓ fold
DuckDB row
    ↓ query
JournalHit
    ↓ exact inspection when needed
journal event at seq
```

This distinction permits fast, rich queries without promoting a disposable file into authority.

## 3.3 Events are the visibility floor

**Decision P2-04 (R2):** Every Journal query starts from the `events` table. Specialized tables enrich or filter events; they never decide which event kinds exist.

`Analytics::apply` records every normalized event in `events` before it applies event-family-specific folds. Unknown or newer event kinds can therefore remain visible even when `messages`, `tool_calls`, `usage`, graph derivation, or another specialized table has not learned their shape.

This matters immediately. N3 introduced `conversation.ask`. A global Journal surface must not lose that event merely because a more specialized projection has not yet modeled it as a message or graph node.

## 3.4 Chronology, relationship, and aggregation remain distinct

**Decision P2-05 (R2):** Journal, Graph, and Analytics remain sibling lenses:

```text
Journal    which recorded events match this question?
Graph      how are recorded entities related, and which event proves each edge?
Analytics  what aggregate pattern exists across trajectories?
```

P2-JOURNAL does not replace:

- `GET /v1/events`, which remains the raw history/SSE transport;
- `GET /v1/graph/work/{id}`, which remains the Work neighborhood;
- `GET /v1/analytics/{name}`, which remains the canned aggregate surface.

A Journal hit may carry identifiers that let a client navigate to those surfaces. It does not fold them into one universal query product.

## 3.5 Normalized evidence, not every retained byte

**Decision P2-06 (R1):** Initial text search covers the normalized journal event kind and payload. It does not search blob contents, raw Claude stream JSON, worktree files, Git objects, screenshots, or arbitrary artifacts.

Sergeant deliberately moves large evidence into the blob store. Searching every blob would introduce:

- content-type detection;
- decompression and decoding policy;
- secret and redaction risk;
- potentially unbounded index size;
- coupling to adapter-specific raw formats;
- a new retention and invalidation problem.

Those costs are not required to make the normalized trajectory queryable.

The API response explicitly reports its content scope so a client cannot imply more complete search than occurred:

```json
{
  "content_scope": {
    "normalized_events": true,
    "blob_content": false,
    "raw_transcripts": false,
    "worktree_files": false
  }
}
```

## 3.6 Content search is not telemetry

**Decision P2-07 (R1):** Journal queries are not emitted as metrics, labels, OTel attributes, or journal events.

Privacy-Preserving Agent Observability correctly forbids free-form content from becoming unbounded metric labels. P2-JOURNAL is a local, authenticated evidence-inspection surface over content Sergeant already records; it is not a telemetry export. Persisting the user's search terms would create a new sensitive history and recursively pollute the journal being searched.

Therefore:

- query text is not journaled;
- query text is not exported through OTel;
- no persistent recent-search history is created;
- the CLI/TUI may keep only ephemeral in-process history if later justified.

## 3.7 Surfaces use the API; they do not become query engines

**Decision P2-08 (R2):** The daemon owns query semantics. CLI, TUI, web, and harness clients only construct typed requests and render typed results.

The adjudicated North Star says a surface adds usability, never functionality. A TUI that opens DuckDB, reads journal segments, or scans blobs would violate both the one-owner rule and that product boundary. P2-JOURNAL therefore lands the query capability below the client layer.

## 3.8 Search is deterministic before it is clever

**Decision P2-09 (R1/R2):** Initial search is exact structured filtering plus literal case-normalized substring search. No relevance ranking changes chronological order.

The first question is not “what is the smartest search engine?” It is:

> Can an operator reliably locate the event that contains the evidence they remember?

Chronological sequence remains the stable ordering. Search results do not receive BM25 or semantic scores in the first implementation.

## 3.9 Ponytail is binding

**Decision P2-10 (R2):** Every implementation choice follows the repository's ladder:

```text
R1  skip what is not demonstrated
R2  reuse current journal, schema, connection, API, and CLI patterns
R3  use stdlib
R4  use native platform capability
R5  use installed dependencies
R6  add one small local composition
R7  only then add minimum new machinery and name failed lower rungs
```

A search index, new table, connection pool, query language, cache, or saved-state store is not admitted merely because it is conventional in search products.

---

# 4. Current Surface, as Implemented

## 4.1 Durable journal

The current journal is:

```text
<data-dir>/journal/00000001.ndjson
<data-dir>/journal/00000002.ndjson
...
```

It provides:

- one writer per journal directory;
- one complete event per line;
- monotonically increasing daemon-local sequences;
- size-based rotation at 8 MiB by default;
- torn-tail quarantine and truncation;
- fail-closed malformed-line and sequence-gap handling;
- complete replay from sequence 1;
- `replay_after`, which skips segments that cannot contain wanted events;
- group commit at the authoritative core-lock boundary.

The event envelope preserves unknown top-level and source fields on replay. That forward-compatibility detail is important: a search projection may know only the current envelope columns, while exact event inspection must return the complete authoritative event.

## 4.2 DuckDB projection

The current schema contains:

```text
events
work
stages
executions
messages
tool_calls
usage
repositories
graph_nodes
graph_edges
```

Every row is derived from the journal. The database is recreated on startup. There is no schema migration path because the file is disposable.

The schema deliberately omits:

```text
artifacts
git_changes
files
commits
findings
```

because current events do not report those facts. P2-JOURNAL does not add empty tables or pretend those dimensions exist.

## 4.3 Current analytical access

The daemon currently exposes five canned questions:

```text
blocked_time_per_work
backend_retries
execution_touched
tool_calls_before_failure
token_totals_per_work
```

It also exposes one Work's graph neighborhood.

The internal `table_rows` helper can dump an allow-listed table, but its own documentation says it has no production caller and exists solely to prove rebuild identity in tests. Routing that method directly to users would expose storage shape rather than design a query contract.

## 4.4 Current event-history access

`GET /v1/events` accepts:

```text
from
work_id
limit
```

The implementation notes an important performance property: `from` is the scan bound. `work_id` and `limit` shape the already-read answer. A global search that repeatedly asks from sequence 0 and filters client-side would parse history under the core lock and transfer irrelevant events to every client.

That is exactly the private-shortcut pressure the equal-client API rule is meant to prevent.

## 4.5 Current performance evidence

The Cerberus baseline measured:

| Scale / operation | Current result |
|---|---:|
| 50,000-event startup and DuckDB rebuild | 924 ms |
| 50,000-event rebuild rate | 54,553 events/s |
| 50,000-event journal size | 27.5 MB |
| 50,000-event DuckDB file after clean shutdown | 32.0 MiB |
| `blocked_time_per_work` cold call, 50k | 401.8 ms |
| graph read, 1,011-event Work, sequential p50 | 14.1 ms |
| graph read, 20-way p50 | 31.0 ms |

These numbers establish two things:

1. the current projection is already fast enough to justify querying it before adding a search service;
2. cold and concurrent read costs are real and must be measured rather than dismissed.

Issues #7, #8, and #10 remain useful baselines for P2-JOURNAL's query-load tests.

## 4.6 Retention and query coordination

The adjudicated retention ruling keeps the journal as the only truth and defers archival machinery until measured rebuild time exceeds 30 seconds. When that trigger fires, the committed ladder is:

```text
compress cold journal segments first
        ↓ only if insufficient
snapshot/truncate machinery
```

P2-JOURNAL must not create a query index whose identity or update protocol makes cold-segment compression, full replay, or DuckDB deletion unsafe. Any search acceleration remains disposable and rebuildable from retained events.

## 4.7 Transcript legibility is adjacent, not identical

The first product-fitness dogfood run recorded E7:

> reading what an actor said requires decoding blob hashes from journal payloads.

The adjudicated North Star assigns `sgt work transcript` to the earlier legibility-and-safety wave. P2-JOURNAL improves normalized message and event discovery across Work, but it does not absorb the Work-transcript contract or search raw archived transcripts.

The boundary is:

```text
sgt work transcript   readable one-Work conversation/evidence reconstruction
sgt journal           cross-Work search over normalized trajectory events
```

The two may share rendering helpers later. Neither is a substitute for the other.

---

# 5. Problem Definition

## 5.1 Operator questions the current surface cannot answer directly

A user should be able to ask:

```text
Where did an agent ask about the retry budget?

Which Work mentioned an ambiguous revision?

Show every conversation.ask emitted by Claude.

Which events belong to execution 01K...?

Where did the 80-adversarial-review stage fail?

Which Work targeting repository service used this tool?

Find the event that caused this tool request.

What happened around sequence 12,840?
```

Today, the answer requires one or more of:

- already knowing the Work ID;
- requesting broad `/v1/events` history;
- grepping NDJSON files;
- manually joining execution and Work identities;
- decoding payload JSON;
- opening DuckDB outside the daemon, which the architecture forbids;
- using one of five aggregate reports that answers a different question.

## 5.2 The product gap

The gap is not storage.

The gap is not event capture.

The gap is not an analytical database.

The gap is the missing **typed read contract** between those capabilities and the operator.

```text
rich durable record
        +
rich disposable projection
        -
query surface
        =
operationally inaccessible history
```

## 5.3 Success condition

A successful P2-JOURNAL implementation lets an ordinary client:

1. express a bounded evidence question without SQL;
2. receive a bounded, paginated list of matching events;
3. understand the Work/execution/stage context of each hit;
4. inspect the exact authoritative event by sequence;
5. navigate to existing Work, Graph, or Analytics views;
6. know what content was and was not searched;
7. receive an honest projection-unavailable error rather than a partial answer.

---

# 6. Scope Contract

## 6.1 In scope

P2-JOURNAL may:

1. add a typed `JournalQuery` and `JournalHit` read model;
2. add `GET /v1/journal` for filtered, sequence-paginated DuckDB search;
3. add `GET /v1/journal/{seq}` for exact journal-event inspection;
4. add `sgt journal` using the same API;
5. perform literal text search over normalized event kind and payload;
6. apply allow-listed structured filters through parameterized SQL;
7. enrich hits with existing Work, execution, stage, workflow, backend, and repository context;
8. report query and projection sequence boundaries;
9. return deterministic match field and excerpt metadata;
10. measure structured and text queries at 50,000 and 1,000,000 events;
11. test query concurrency against the mutation path;
12. document a future TUI rendering contract without modifying the TUI;
13. update API, CLI, tests, README, and ledger when implementation is eventually authorized.

## 6.2 Explicit non-goals

**Decision P2-11 (R1):** The following are outside P2-JOURNAL:

- arbitrary client SQL;
- raw table-dump routes;
- regex, fuzzy, phonetic, vector, embedding, or semantic search;
- BM25/relevance ordering in the first version;
- automatic DuckDB FTS indexing;
- a new external search service;
- a second database or index file outside the disposable projection;
- saved searches, named views, query sharing, or persistent search history;
- alerts, subscriptions, notification rules, or continuous-query execution;
- `--follow` or filtered live-stream semantics;
- mutation from query results;
- new event kinds or backend instrumentation solely to improve search;
- searching raw blob contents or provider transcripts;
- file, diff, artifact, commit, or finding search without source events;
- reconstructing private chain of thought;
- retention, archival, compression, snapshot, or GC implementation;
- new authentication, RBAC, tenancy, or non-loopback exposure;
- a TUI or dashboard redesign;
- replacing the existing Events, Graph, Analytics, or Work APIs;
- total-result counts on every query;
- search-result caching;
- a DuckDB connection pool.

P2-JOURNAL is a bounded read capability, not a general data platform.

---

# 7. Query Contract

## 7.1 Query fields

**Decision P2-12 (R7, minimum new contract):** The daemon owns one typed query object with this initial allowlist:

```text
text
kind[]
work_id
execution_id
workflow
stage_id
backend
repository[]
source_type
source_name
correlation_id
causation_id
tool
model
seq
after_seq
before_seq
as_of_seq
after_time
before_time
order
limit
```

R1–R6 do not supply a public typed contract: the existing event route cannot search and the canned analytics names cannot express event-level questions. The R7 is one struct and one compiled query path, not a query language framework.

## 7.2 Combination semantics

**Decision P2-13 (R6):** Query combination is fixed:

```text
fields of different names       AND
repeated values of one field    OR
text and structured filters     AND
```

Examples:

```text
kind=stage.failed OR kind=work.failed
AND backend=claude
AND text contains "permission"
```

There is no user-authored parenthesis, negation, precedence, or expression tree.

## 7.3 Literal text semantics

**Decision P2-14 (R5):** `text` performs a literal, case-normalized substring match over:

```text
events.kind
events.payload
```

The candidate SQL form is:

```sql
contains(
  lower(nfc_normalize(search_column)),
  lower(nfc_normalize(?))
)
```

DuckDB already provides `contains`, `lower`, and `nfc_normalize`. The installed version's Unicode behavior must be measured and pinned before the exact expression becomes contract text.

Important honesty:

- `payload` is serialized normalized JSON;
- a match may occur in a JSON key as well as a value;
- no stemming or synonym expansion occurs;
- no blob content is searched;
- an empty text value is treated as absent, not “match everything twice.”

If J0 measurement shows key-name noise makes literal payload search unusable, a derived `search_text` field may be considered at a later rung. It is not pre-authorized here.

## 7.4 Structured filter semantics

### Envelope filters

These map directly to `events` columns:

```text
seq
after_seq / before_seq
kind
work_id
execution_id
source_type / source_name
correlation_id / causation_id
after_time / before_time
```

### Work-context filters

These scope events by existing Work projection fields:

```text
workflow
backend
repository
```

A repository filter means “events belonging to Work whose materialized surface targets this repository.” It does not mean the individual event touched a file in that repository.

### Stage filter

`stage_id` matches:

1. the execution's pinned stage where `execution_id` is present;
2. an allow-listed `payload.stage_id` extraction for stage events.

It does not infer stage ownership from event adjacency.

### Tool filter

`tool` selects `tool.requested` and `tool.completed` events associated with a tool call of that exact normalized name. It does not return every event from an execution that happened to use the tool.

### Model filter

`model` scopes events to executions whose `usage` rows report that exact model. The response must identify that this is execution-level context, not proof that every event in the execution was generated by a separately identifiable model turn.

## 7.5 Fields deliberately absent

**Decision P2-15 (R1):** The first query contract does not expose `current_state`.

Current Work state is mutable context, not an event property. Filtering historical events by present state is useful in some analyses but easy to misread as “state at the time of the event.” Users may filter exact failure/blocked/completed event kinds or open the Work surface for current state.

Likewise absent:

```text
arbitrary JSON path
file path
commit SHA
artifact name
finding severity
hidden reason
```

Those fields either need a source event family or a separately adjudicated query contract.

## 7.6 Bounds

**Decision P2-16 (R6):** The request is bounded before SQL compilation:

```text
text length                  ≤ 1,024 Unicode scalar values
repeated values per field    ≤ 32
limit default                100
limit maximum                500
order                         asc | desc only
```

Unknown fields, malformed timestamps, contradictory sequence cursors, and values beyond bounds return structured `400 invalid_journal_query` errors.

---

# 8. Sequence, Time, and Pagination

## 8.1 Sequence is the primary order

**Decision P2-17 (R2):** Journal sequence is the canonical sort and cursor.

Timestamps are useful filters but not identity. Sequence is:

- daemon-assigned;
- unique;
- monotonically increasing;
- already the SSE resume coordinate;
- already the graph provenance coordinate;
- stable across DuckDB rebuilds;
- unaffected by future cold-segment compression.

## 8.2 Default page

The default request:

```text
GET /v1/journal
```

means:

```text
latest 100 normalized events
ordered by seq descending
as of the projection sequence caught up for this request
```

It is a useful Journal landing page without constructing a separate “recent activity” store.

## 8.3 Keyset pagination

**Decision P2-18 (R2/R6):** Pagination is sequence-keyset pagination, never `OFFSET`.

```text
before_seq=<exclusive upper cursor>   older results

after_seq=<exclusive lower cursor>    newer results
```

`before_seq` and `after_seq` are mutually exclusive for one page request.

The API fetches `limit + 1` rows to determine `has_more`, then returns at most `limit`.

**Decision P2-48 (R1):** The default query does not run an unbounded `COUNT(*)` solely to paint a total. Page navigation needs only `has_more` and stable sequence cursors.

## 8.4 Query snapshot coordinate

**Decision P2-19 (R2):** Every result records `as_of_seq`.

On the first request, `as_of_seq` defaults to the DuckDB projection's `last_seq` after catch-up. Subsequent older-page requests carry the same cap:

```text
seq <= as_of_seq
AND seq < before_seq
```

This keeps the event set stable while new work appends above the snapshot.

The projection may have caught up farther by the time a later page executes. Stable context fields are still safe; mutable current-state fields are not part of `JournalHit`, which is why §7.5 excludes them.

## 8.5 Time filters

`after_time` and `before_time` accept the same fixed-width RFC3339 UTC shape Sergeant writes:

```text
YYYY-MM-DDTHH:MM:SS.mmmZ
```

They compare against `events.ts_ms`. An event whose timestamp cannot be parsed by the current projection remains searchable by sequence and text but cannot satisfy a numeric time filter. The projection does not guess.

---

# 9. Result Contract

## 9.1 Search response

**Decision P2-20 (R7, minimum new read model):** `GET /v1/journal` returns one stable shape:

```json
{
  "query": {
    "text": "retry budget",
    "kind": ["conversation.ask"],
    "backend": "claude",
    "order": "desc",
    "limit": 100
  },
  "projection": {
    "last_seq": 1290
  },
  "page": {
    "as_of_seq": 1290,
    "has_more": false,
    "next_before_seq": null,
    "next_after_seq": 1284
  },
  "content_scope": {
    "normalized_events": true,
    "blob_content": false,
    "raw_transcripts": false,
    "worktree_files": false
  },
  "results": [
    {
      "seq": 1284,
      "event_id": "01K...",
      "timestamp": "2026-08-11T12:27:14.531Z",
      "kind": "conversation.ask",
      "source": {
        "type": "backend",
        "name": "claude"
      },
      "workspace_id": null,
      "work_id": "01K...",
      "execution_id": "01K...",
      "correlation_id": null,
      "causation_id": "01K...",
      "payload": {
        "text": "Should the retry budget be 3 attempts?"
      },
      "context": {
        "intent": "Add retry handling to the settlement worker",
        "workflow": "software-change",
        "backend": "claude",
        "stage_id": "10-implement",
        "attempt": 1,
        "repositories": ["service"]
      },
      "match": {
        "field": "payload",
        "excerpt": "Should the retry budget be 3 attempts?"
      }
    }
  ]
}
```

The exact field names are contract candidates; the semantics are binding for proposal review.

## 9.2 Search hits are projection rows

A `JournalHit` carries the known event envelope columns and full normalized payload stored in DuckDB. It is not claimed to be a byte-identical reserialization of the original journal line because the current `events` table does not preserve unknown envelope fields.

That is why the sequence is not merely metadata. It is the route to exact evidence.

## 9.3 Exact event response

**Decision P2-21 (R2):** `GET /v1/journal/{seq}` returns the complete authoritative `Event` read from the journal, including:

- `schema`;
- source extras;
- top-level unknown fields;
- full payload;
- every optional identity field.

The implementation reuses journal segment seeking and `replay_after(seq - 1)` rather than duplicating the whole serialized event into DuckDB.

Outcomes:

```text
200  exact event
404  no committed event at that sequence
500  journal read/corruption failure, fail closed
```

Projection unavailability does not prevent exact event lookup because the exact route reads truth, not DuckDB.

## 9.4 Context is enrichment, not evidence replacement

**Decision P2-22 (R2):** Context fields may explain a hit but never replace its event.

Allowed initial enrichment:

```text
Work intent
workspace
workflow
resolved Work backend
stage id and attempt
repository names
```

Not included:

```text
current Work state
invented outcome
file changes
artifact disposition
human-readable causal narrative
```

## 9.5 Match metadata

**Decision P2-23 (R6):** When `text` is present, the server returns:

```text
field     kind | payload
excerpt   bounded text around the first literal match
```

The excerpt is deterministic presentation metadata. It is never substituted for the payload and never journaled.

When no text filter exists, `match` is `null`.

## 9.6 No relevance score

**Decision P2-24 (R1):** The first result contract has no relevance score.

A phrase either matches the literal contract or it does not. Sequence order remains the meaningful default. Relevance ranking is not needed to prove the product gap closed.

---

# 10. API Surface

## 10.1 Search route

```text
GET /v1/journal
```

Example:

```text
/v1/journal?text=retry%20budget
           &kind=conversation.ask
           &backend=claude
           &before_seq=2000
           &as_of_seq=2500
           &limit=100
```

All parameters are optional. Repeated `kind` and `repository` values use repeated query keys.

## 10.2 Exact route

```text
GET /v1/journal/{seq}
```

This is an exact evidence fetch, not a search result page.

## 10.3 Authorization

**Decision P2-25 (R2):** Both routes use the existing bearer gate and remain loopback-only under the current daemon contract.

P2-JOURNAL does not broaden listener scope or create a query-token exception. Any future non-loopback or multi-user access must revisit authorization and the dashboard token ruling before exposing cross-Work content search.

## 10.4 Read-only semantics

**Decision P2-26 (R2):** Journal queries use `GET`, carry no `command_id`, produce no command event, and mutate no durable state.

A query may update the disposable DuckDB projection through the existing catch-up path. That is read-model maintenance, not Work mutation.

## 10.5 Error vocabulary

The routes use the existing structured error shape:

```json
{
  "error": {
    "code": "invalid_journal_query",
    "message": "before_seq and after_seq cannot be combined"
  }
}
```

Initial codes:

```text
invalid_journal_query     400
journal_event_not_found   404
projection_unavailable    503
internal                  500
```

**Decision P2-45 (R2):** A DuckDB failure remains `projection_unavailable`; it is never reported as journal corruption or Work failure. This reuses the existing fail-closed projection vocabulary.

---

# 11. Query Compilation and Execution

## 11.1 No client SQL

**Decision P2-27 (R1):** The request never contains SQL, table names, column names, functions, order expressions, or JSON paths supplied by the client.

The daemon compiles `JournalQuery` from a fixed map of internal fragments.

## 11.2 Parameter binding

**Decision P2-28 (R5):** Every client value is bound as a DuckDB parameter.

DuckDB's prepared-statement support exists specifically to substitute values without string concatenation. The only generated SQL text is selected from internal allowlists:

- fixed column expressions;
- fixed joins;
- fixed sort direction enum;
- fixed optional predicates.

Tests must prove hostile text remains a value:

```text
' OR 1=1 --
%_
JSON punctuation
newlines
Unicode
```

## 11.3 Existing connection and lock

**Decision P2-29 (R2):** P2-JOURNAL reuses the one `Analytics` connection and mutex.

It does not add:

- a read replica;
- a connection pool;
- a second DuckDB file;
- a snapshot database;
- a client connection.

This serializes DuckDB queries initially. The mutation core remains on its separate lock.

**Decision P2-49 (R2):** Concurrent Journal, Graph, and Analytics reads may queue behind the one analytics mutex until measurement proves that serialization is an operator problem. No pool or replica is pre-authorized.

## 11.4 Core-lock boundary

**Decision P2-30 (R2):** The expensive DuckDB selection runs with no core lock held.

The existing `with_analytics` sequence remains the model:

```text
read projection last_seq
        ↓
briefly obtain journal tail from Core
        ↓
catch DuckDB up
        ↓
release Core
        ↓
execute Journal query under Analytics ownership
```

A deliberately stalled Journal query must not prevent an independent Work submission, response, retry, or cancel from acquiring the core beyond the bounded catch-up interval.

Whether synchronous DuckDB execution also needs `block_in_place` is a J0 measurement question. The contract requires runtime responsiveness, not a predetermined thread primitive.

## 11.5 Query from `events`, enrich from siblings

The SQL starts from:

```sql
FROM events e
```

Optional joins or semi-joins use:

```text
work
executions
stages
repositories
usage
tool_calls
```

Graph tables are not required for Journal search. Querying them to answer ordinary envelope questions would make a secondary derivation the gateway to primary chronology.

## 11.6 Unknown event compatibility

**Decision P2-31 (R2):** A structurally valid event unknown to specialized folds still appears in unfiltered, envelope-filtered, and text-filtered Journal results.

A structured filter that depends on a specialized table may naturally exclude an event with no such modeled relation. The API documentation says so; it does not silently claim universal semantic enrichment.

---

# 12. Search Acceleration Decision

## 12.1 Start without FTS

**Decision P2-32 (R1/R2):** Initial P2-JOURNAL uses the existing `events` table and ordinary DuckDB predicates.

Reasons:

1. the current measured scale is small enough to test before indexing;
2. the `events` table is already populated by the canonical fold;
3. literal evidence search does not require stemming or ranking;
4. DuckDB automatically maintains zonemaps;
5. sequence and primary-key lookups already have appropriate structure;
6. DuckDB's FTS index does not update automatically when the source table changes;
7. FTS would introduce a second catch-up/rebuild contract;
8. extension autoload may add environment and supply-chain behavior the current single-binary prototype has not admitted.

## 12.2 No speculative ART indexes

**Decision P2-33 (R1):** Do not add explicit ART indexes in the first build.

DuckDB's own guidance says ART primarily benefits point and very highly selective queries, creates a secondary data copy, affects load/update cost, and should be added only with enough memory and a measured selective workload. The current schema already creates primary-key indexes where correctness requires them.

## 12.3 No cache or saved-result layer

**Decision P2-50 (R1):** The first implementation creates no result cache, saved-query table, prepared-result store, or persistent recent-search history. DuckDB already owns the rebuildable read model; a second derived cache has no measured need.

## 12.4 Measurement ladder if text scans fail

If J0 or J1 breaches the agreed search budget, the next contract must adjudicate in order:

```text
R2  improve predicate order, selected columns, and sequence/time pruning
R2  narrow JSON extraction to measured event families
R6  add one derived search_text expression/view if enough
R7  add a rebuildable search_text table during the existing fold
R7  only then consider DuckDB FTS with an explicit refresh contract
```

An FTS implementation must answer:

- how the index catches up after each journal tail;
- how restart rebuilds it;
- how an index failure becomes one 503 rather than a silently incomplete result;
- how stemming and stopwords affect evidence expectations;
- how offline installation works in measured environments;
- how its disk and memory cost interact with retention rulings.

No lower milestone may jump directly to the last rung.

---

# 13. CLI Surface

## 13.1 Command

**Decision P2-34 (R2/R5):** The CLI adds one top-level command using Clap, which is already installed:

```text
sgt journal [TEXT] [FILTERS]
```

No nested search DSL is required.

## 13.2 Examples

```sh
# Latest normalized events
sgt journal

# Literal event text
sgt journal "retry budget"

# Text plus context filters
sgt journal "ambiguous revision" --workflow repo-to-icm

# Actor-authored questions from Claude
sgt journal --kind conversation.ask --backend claude

# One Work and stage
sgt journal --work 01K... --stage 80-adversarial-review

# One execution
sgt journal --execution 01K...

# Tool evidence
sgt journal --tool Bash --work 01K...

# Exact event from journal truth
sgt journal --seq 1284

# Older page
sgt journal "retry" --before-seq 1284 --as-of-seq 2000 --limit 100
```

## 13.3 Flags

Candidate flags mirror the API without renaming its concepts:

```text
--kind <kind>              repeatable
--work <id>
--execution <id>
--workflow <name>
--stage <id>
--backend <name>
--repo <name>              repeatable
--source-type <type>
--source-name <name>
--correlation <id>
--causation <event-id>
--tool <name>
--model <name>
--seq <n>
--after-seq <n>
--before-seq <n>
--as-of-seq <n>
--after-time <RFC3339>
--before-time <RFC3339>
--order <asc|desc>
--limit <n>
```

The existing global `--json` flag emits the complete response body.

## 13.4 Human output

The default output is a compact event table:

```text
SEQ    WHEN                  WORK       STAGE                  KIND
1284   12:27:14.531          01K…       10-implement          conversation.ask
       Should the retry budget be 3 attempts?

1272   12:24:03.014          01K…       10-implement          tool.completed
       Bash · completed
```

Exact sequence mode prints the complete event envelope and payload in readable JSON.

## 13.5 No compact mini-language yet

**Decision P2-35 (R1):** The CLI does not initially parse text such as:

```text
kind:conversation.ask backend:claude since:7d
```

Clap flags already provide typed, documented, shell-completable input. A mini-language adds quoting, escaping, errors, precedence, and a second parser before repeated use has proved the need.

A future TUI may render filter chips without exposing an expression grammar.

## 13.6 No follow mode

**Decision P2-36 (R1):** `sgt journal` is a bounded query, not a filtered live subscription.

The existing SSE endpoint remains the live event transport. Continuous filtered queries, alerts, and subscriptions need their own state, cancellation, reconnect, and missed-event contracts.

---

# 14. Future TUI Presentation Contract

This section describes compatibility, not implementation. The T-series proposal remains separate and proposed.

## 14.1 Navigation

After both proposals are accepted and their sequencing is adjudicated, the terminal information architecture may become:

```text
Home    Fleet    Workflows    Journal
```

P2-JOURNAL does not require that navigation change and does not modify `src/tui.rs`.

## 14.2 Plausible screen

A Ratatui client could render:

```text
┌─ JOURNAL ──────────────────────────────────────────────────────────────────┐
│ Search  retry budget                                                       │
│ Filters kind:conversation.ask  backend:claude  as-of:1290                 │
├──────────────────────────────┬─────────────────────────────────────────────┤
│ 1284  conversation.ask      │ EVENT 1284                                  │
│ ? retry budget              │ Work       Add retry handling               │
│                              │ Workflow   software-change                   │
│ 1192  conversation.user     │ Stage      10-implement #1                   │
│ Yes, use 3 attempts         │ Source     backend/claude                    │
│                              │                                             │
│ 1189  stage.needs_input     │ Should the retry budget be 3 attempts?       │
│                              │                                             │
├──────────────────────────────┴─────────────────────────────────────────────┤
│ Enter exact event · w open Work · g graph · / commands · Esc back          │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 14.3 Client behavior

A future TUI should:

- send typed filters to `/v1/journal`;
- preserve `as_of_seq` while paging;
- fetch `/v1/journal/{seq}` when exact inspection opens;
- open the canonical Work surface when `work_id` exists;
- open the existing graph view rather than drawing inferred edges;
- say when blob/raw-transcript content was not searched;
- never open DuckDB or journal files directly.

## 14.4 No dependency inversion

**Decision P2-37 (R1):** P2-JOURNAL does not depend on the T-series redesign landing first.

The API and CLI are independently useful. The TUI proposal may later consume them without changing their semantics.

---

# 15. Privacy, Security, and Evidence Boundaries

## 15.1 Existing content exposure

Journal search can reveal message and payload content already available to the local bearer holder. It does not make that content newly durable, but it makes discovery materially easier.

That usability gain is the point and also the privacy risk.

## 15.2 Initial security boundary

**Decision P2-38 (R2):** Initial authorization is exactly the existing single-user, loopback bearer boundary.

This proposal does not claim enterprise confidentiality, tenant isolation, role filtering, or field-level redaction.

Before any non-loopback or multi-user exposure, a later contract must decide:

- who may search which Work;
- whether message payloads are visible;
- whether search terms themselves are sensitive audit events;
- how redaction markers and evidence completeness appear;
- how retention applies to searchable content.

## 15.3 No raw-secret expansion

**Decision P2-39 (R1):** Search does not dereference blob references or secret identifiers.

A payload that contains a `blob_ref` is searchable by that reference string. The blob's bytes are not loaded into DuckDB and not returned automatically.

## 15.4 Bounded excerpts

Excerpts are capped and derived only after a match. The server does not concatenate unrelated message history around a hit. A client wanting more context opens the Work or exact event.

## 15.5 Evidence-completeness marker

**Decision P2-47 (R2):** The response's `content_scope` is required. Evidence completeness must be visible in the contract rather than inferred from documentation. A UI may say:

```text
Searched normalized journal events through seq 1290.
Raw transcript and blob content were not searched.
```

It may not say simply “searched everything.”

---

# 16. Retention, Eviction, and Future Compression

## 16.1 Terminal projection eviction

The owner-adjudicated retention ruling plans to evict heavy in-memory state for terminal Work while retaining a light fleet index and re-deriving full Work views on demand.

**Decision P2-40 (R2):** Journal query must remain independent of whether a terminal Work is retained in the in-memory registry.

It queries DuckDB, which is rebuilt from the complete retained journal. Opening a hit's Work may trigger the future on-demand derivation path; search itself does not require it.

## 16.2 Cold-segment compression

**Decision P2-41 (R2):** Exact event lookup must remain compatible with future transparent cold-segment decompression.

The Journal API calls a journal abstraction. It does not assume every segment is an uncompressed filesystem path forever.

## 16.3 No retention knob

**Decision P2-42 (R1):** P2-JOURNAL adds no query-retention or search-index-retention setting. It searches whatever authoritative history and disposable projection the daemon currently owns.

“All time” means all retained events, not all events that once existed under a future policy.

## 16.4 No new persistent index identity

Any later acceleration structure must be disposable, rebuilt from retained events, and invalid when its source sequence does not match the journal-derived projection boundary.

---

# 17. Relationship to Existing Surfaces and Programs

## 17.1 T-series

The separately proposed terminal redesign defines:

- Home;
- Fleet;
- Workflows;
- canonical Work thread;
- Work-local Evidence;
- respond/retry/cancel;
- attention and liveness.

P2-JOURNAL defines global historical query. T-series may add `Journal` later, but neither proposal silently edits the other.

## 17.2 Work transcript / North Star Wave 0

**Decision P2-51 (R2):** `sgt work transcript` remains the Work-legibility feature owned by the North Star's earlier wave. P2-JOURNAL may search normalized conversation events and link to the transcript, but it does not postpone or replace that requirement.

## 17.3 Analytics

**Decision P2-43 (R2):** The five canned aggregate questions stay intact. P2-JOURNAL does not turn every search into a report or expose SQL.

A future measured query that recurs often and has stable semantics may graduate into a named canned analytic. That is learning-time determinism, not a user-created saved query.

## 17.4 Graph

**Decision P2-44 (R2):** Journal results carry Work/execution/event IDs needed to open existing graph neighborhoods. They do not add global graph search or a graph database.

## 17.5 Web dashboard

The web dashboard is outside this proposal. If reactivated later, it consumes the same Journal API after its authentication and test prerequisites are satisfied.

## 17.6 N4 and later execution semantics

P2-JOURNAL does not depend on Docker execute stages. New event kinds from N4 should appear automatically in default Journal search because the `events` table is the visibility floor.

Structured filters for new executor-specific dimensions require a later additive contract.

## 17.7 Open issues

**Decision P2-53 (R2):** Existing read-path issues are baselines and coordination inputs, not pre-adjudicated P2 fixes.

- **#7** supplies a graph-read concurrency comparison, not a required fix.
- **#8** supplies a retained-RSS hypothesis for read bursts; P2 measures its own repeated-query trend.
- **#10** supplies a cold-query scaling precedent.
- **#17** owns retention/archival policy; P2 remains compatible with its ruling.

P2-JOURNAL should file new issues only from measured search behavior, not relabel these observations preemptively.

---

# 18. Testing and Measurement

## 18.1 Pure query normalization

Table-driven tests cover:

- every field;
- repeated-field OR semantics;
- cross-field AND semantics;
- empty values;
- bounds;
- contradictory cursors;
- exact sequence mode;
- timestamp parsing;
- order and limit normalization;
- stable normalized response echo.

## 18.2 SQL-injection and allowlist tests

Every client-controlled value is mutation-probed with hostile strings. Tests assert:

- no value changes SQL structure;
- sort direction cannot carry raw text;
- no table or JSON path comes from the request;
- wildcard characters remain literal under the text contract;
- unknown query keys produce `400` rather than being ignored.

## 18.3 Event visibility

Fixtures include:

- every current event family;
- `conversation.ask`;
- a structurally valid unknown event kind;
- an event with unknown envelope/source fields;
- events with and without Work/execution identity;
- an unparseable projection timestamp.

Acceptance:

- default and text queries find unknown event kinds from `events`;
- exact lookup preserves unknown fields from the journal;
- time filters fail closed only for the unparseable timestamp row;
- no specialized-table omission erases the base event.

## 18.4 Rebuild identity

**Decision P2-46 (R2):** Query-result identity across rebuild is an acceptance invariant, not merely a performance check.

For a fixed journal and query corpus:

```text
query results before deleting DuckDB
=
query results after deleting DuckDB and rebuilding
```

Comparison includes:

- normalized query;
- sequence ordering;
- hit payloads;
- context fields;
- pagination cursors;
- content-scope metadata.

## 18.5 Catch-up freshness

A live daemon test:

1. queries at sequence N;
2. appends matching and non-matching events;
3. queries again;
4. proves `projection.last_seq` advanced;
5. proves new matching rows appear exactly once;
6. proves a page pinned to `as_of_seq=N` remains stable.

## 18.6 Pagination

Generated datasets test ascending and descending pages across:

- zero rows;
- one row;
- exact limit;
- limit + 1;
- sparse matches;
- new events appended between pages;
- identical timestamps;
- segment rotation boundaries.

No result may repeat or disappear when the client follows returned cursors against one `as_of_seq`.

## 18.7 Exact event lookup

Tests prove:

- first, middle, last, and missing sequences;
- lookup across segment boundaries;
- lookup after restart;
- unknown fields preserved;
- bounded replay rather than replay from sequence 1;
- no long core-lock stall at the measured segment size.

## 18.8 Content boundary

A fixture places a unique phrase only inside a blob and another phrase in the normalized event.

Acceptance:

```text
normalized phrase found
blob-only phrase not found
content_scope says blob_content=false
```

This test must fail if an implementation later starts dereferencing blobs silently.

## 18.9 Performance matrix

J0 establishes a dated baseline on the current Cerberus class and the repository's existing synthetic journal harness.

**Decision P2-52 (R2):** The mandatory scales are the current 50,000-event baseline and the retention ruling's 1,000,000-event decision point.

Required scales:

```text
50,000 events    current measured baseline scale
1,000,000 events retention-ruling decision scale
```

Required query classes:

1. latest page, no filters;
2. exact Work ID;
3. exact execution ID;
4. exact kind;
5. workflow + stage + backend;
6. common literal text;
7. absent literal text;
8. sparse literal text near the oldest history;
9. exact event lookup;
10. 20 concurrent Journal queries;
11. Journal query while submitting Work;
12. repeated query waves followed by RSS settle.

## 18.10 Provisional performance red lines

These are proposal-level usability ceilings, finalized after J0 measurement:

| Operation | 50k events | 1M events |
|---|---:|---:|
| exact identity/sequence first page p95 | ≤100 ms | ≤250 ms |
| structured multi-filter first page p95 | ≤250 ms | ≤750 ms |
| literal text first page p95 | ≤500 ms | ≤2 s |
| exact event lookup p95 | ≤100 ms | ≤250 ms |

A red line breach does not automatically authorize FTS. It triggers the §12.4 ladder and an adjudication.

## 18.11 Mutation-path isolation

A deliberately slow DuckDB query runs while an independent client submits, responds, retries, and cancels.

The test proves:

- no DuckDB scan occurs under the core lock;
- the mutation path is delayed only by the bounded projection catch-up interval, not by the whole search;
- the query may queue other analytics reads under the one analytics mutex, which is the admitted initial tradeoff.

## 18.12 Memory and cleanup

Repeated query waves measure:

```text
RSS before
RSS peak
RSS after settle
fds
threads
DuckDB file size
```

A monotonic retained-RSS trend becomes a new issue with the same discipline used for #8. One burst is not called a leak.

## 18.13 CLI tests

Tests cover:

- no-argument latest page;
- positional text;
- every flag mapping;
- repeated flags;
- human table output;
- exact event output;
- `--json` byte-stable field names;
- structured errors;
- data-dir auto-spawn behavior unchanged;
- no leaked daemon processes.

## 18.14 Mutation-probe discipline

**Decision P2-54 (R2):** Every behavior-changing implementation and fix runs the repository's full code gauntlet. Every fixing commit carries a pinning test that fails when reverted, and query-builder guards receive independent mutation probes in disposable worktrees under the current S-series protocol.

---

# 19. Program Shape

**Decision P2-55 (R1):** P2-JOURNAL is a capability proposal, not a priority override. Its scheduling must respect the adjudicated North Star waves and current H/U/N/T lanes.

## P2-J0 — Adjudication and measured query spike

Outcome:

- proposal challenged on spec fidelity, invariants, simplicity, privacy, and test honesty;
- representative operator queries harvested from real journals;
- parameterized scan spike against current `events` table;
- 50k and 1M measurements;
- Unicode/literal semantics measured against the installed DuckDB;
- provisional budgets amended or accepted;
- decision recorded on whether ordinary scans are sufficient;
- zero production route or CLI behavior if the contract is not accepted.

## P2-J1 — Typed query engine and API

Outcome:

- `JournalQuery` validation and normalization;
- parameterized SQL compilation;
- `JournalHit` response;
- sequence-keyset pagination;
- `GET /v1/journal`;
- exact `GET /v1/journal/{seq}`;
- projection freshness and exact-event tests;
- content-scope and privacy pins;
- no CLI or TUI dependency required for acceptance.

## P2-J2 — CLI, performance, and documentation

Outcome:

- `sgt journal`;
- human and JSON renderings;
- 50k/1M performance matrix;
- concurrent-query and mutation-isolation tests;
- repeated-query RSS census;
- README/API documentation;
- final ledger entry and any measured follow-up issues;
- T-series integration note, with no TUI code.

## Future T-series integration — outside P2-JOURNAL

Only after both proposals are accepted may a T-series contract add the Journal navigation and rendering described in §14. That work remains TUI code and receives its own visual, responsive, focus, and interaction tests.

---

# 20. Ponytail Decision Register

The rung is the lowest viable resolution, not the importance of the decision.

| ID | Rung | Decision | Why this rung |
|---|---|---|---|
| P2-01 | R2 | Extend the current journal/DuckDB/API stack | All required storage and ownership already exist |
| P2-02 | R2 | Event is the result unit | Existing immutable fact boundary |
| P2-03 | R2 | DuckDB finds; journal proves | Existing truth/projection split |
| P2-04 | R2 | `events` is the visibility floor | Already records every normalized event |
| P2-05 | R2 | Journal, Graph, Analytics remain sibling lenses | Existing proposal boundary |
| P2-06 | R1 | Exclude blob/raw-transcript search | Not required; creates major new policy |
| P2-07 | R1 | Do not journal or metricize queries | Avoid recursive sensitive history |
| P2-08 | R2 | Daemon owns query semantics | Equal-client and one-owner rules already bind |
| P2-09 | R1/R2 | Literal deterministic search first | No ranking machinery required |
| P2-10 | R2 | Ponytail binding | Existing repository method |
| P2-11 | R1 | Explicit non-goals | Remove unearned product surface |
| P2-12 | R7 | Add one typed `JournalQuery` | No current API shape can express the need; lower rungs fail |
| P2-13 | R6 | Fixed AND/OR combination | One local rule avoids expression machinery |
| P2-14 | R5 | Use installed DuckDB text functions | Dependency already provides the operations |
| P2-15 | R1 | Exclude current-state filter | Avoid historical/present ambiguity |
| P2-16 | R6 | Bound text, lists, and rows | Small local validation protects cost |
| P2-17 | R2 | Sequence is primary order | Existing durable coordinate |
| P2-18 | R2/R6 | Keyset pagination | Reuse sequence; tiny cursor logic |
| P2-19 | R2 | Return `as_of_seq` | Existing projection boundary |
| P2-20 | R7 | Add one stable search-result shape | New public read contract is necessary |
| P2-21 | R2 | Exact event route reuses journal replay | Preserves full envelope without duplication |
| P2-22 | R2 | Context enriches, never replaces event | Existing projection relationship |
| P2-23 | R6 | Deterministic match excerpt | Small presentation metadata |
| P2-24 | R1 | No relevance score | Chronology is sufficient initially |
| P2-25 | R2 | Reuse bearer/loopback auth | No new trust boundary |
| P2-26 | R2 | GET, no command ID, no query event | Existing read semantics |
| P2-27 | R1 | Reject client SQL | Protect ownership and query grammar |
| P2-28 | R5 | Parameter binding | Installed DuckDB capability |
| P2-29 | R2 | Reuse one Analytics connection | No measured need for pool/replica |
| P2-30 | R2 | Execute query outside Core lock | Existing lock separation |
| P2-31 | R2 | Unknown events remain searchable | Existing forward-compatible event table |
| P2-32 | R1/R2 | No FTS initially | Existing scan path unmeasured; FTS refresh conflict |
| P2-33 | R1 | No speculative ART indexes | No measured selective-query failure |
| P2-34 | R2/R5 | One `sgt journal` command via Clap | Reuse CLI pattern and dependency |
| P2-35 | R1 | No mini query language | Flags already solve typed input |
| P2-36 | R1 | No follow mode | Existing SSE; filtered subscription unearned |
| P2-37 | R1 | TUI integration remains separate | Preserve proposal authority boundary |
| P2-38 | R2 | Reuse current single-user authorization | Current product contract |
| P2-39 | R1 | Do not dereference blobs | Avoid secret/content expansion |
| P2-40 | R2 | Query independent of terminal Work eviction | DuckDB/journal already support history |
| P2-41 | R2 | Journal abstraction hides future compression | Existing abstraction should own storage form |
| P2-42 | R1 | No retention/query knobs | No policy mechanism to obey them |
| P2-43 | R2 | Keep canned analytics unchanged | Existing aggregate contract works |
| P2-44 | R2 | Keep graph unchanged | Existing source-seq relationship view works |
| P2-45 | R2 | Use current projection-unavailable semantics | Existing fail-closed read behavior |
| P2-46 | R2 | Rebuild identity is acceptance-critical | Projection is defined as a pure fold |
| P2-47 | R2 | Content-scope marker required | Existing evidence-honesty doctrine |
| P2-48 | R1 | No total count by default | Not needed for navigation; avoids extra scan |
| P2-49 | R2 | One analytics mutex initially | Existing ownership and concurrency model |
| P2-50 | R1 | No cache or saved results | No measured repeat-query pressure |
| P2-51 | R2 | `sgt work transcript` stays separate | North Star already assigns its ownership |
| P2-52 | R2 | 50k and 1M measurement scales | Existing baseline and retention ruling |
| P2-53 | R2 | Existing #7/#8/#10 become baselines | Reuse measured evidence before filing duplicates |
| P2-54 | R2 | Code changes take full gauntlet | Binding repository rule |
| P2-55 | R1 | Program does not override current roadmap | Proposal defines capability, not priority |

No R7 search index, database, cache, service, or parser framework is authorized by this proposal.

---

# 21. Acceptance Criteria

P2-JOURNAL is complete only when all of the following hold:

1. The journal remains the only authoritative event record.
2. Only the daemon accesses DuckDB and journal storage.
3. `GET /v1/journal` accepts only the documented typed fields.
4. Client input can never become SQL syntax, an identifier, table name, column name, JSON path, or sort expression.
5. Every hit carries an existing journal sequence.
6. The base query starts from `events`.
7. Unknown event kinds remain discoverable without specialized-table support.
8. Literal text search semantics are measured, documented, and pinned.
9. Search covers normalized event kind and payload only.
10. Blob-only and raw-transcript-only phrases do not match.
11. `content_scope` states those omissions.
12. Structured filters have fixed AND/OR semantics.
13. Query bounds are enforced with structured 400 errors.
14. Default results are newest-first and bounded.
15. Pagination is sequence-keyset based with no duplicates or gaps.
16. `as_of_seq` stabilizes the event set across pages.
17. No default query performs a total count.
18. Search results do not claim current Work state.
19. Search results do not invent files, commits, artifacts, findings, or outcomes.
20. Exact event lookup returns the authoritative forward-compatible event envelope.
21. Exact event lookup does not require DuckDB.
22. Projection failure returns 503 and does not affect Work.
23. Query terms are not journaled, exported as telemetry, or persisted as search history.
24. Search runs without holding the core lock across DuckDB execution.
25. A stalled search does not stall Work mutation beyond the bounded catch-up interval.
26. Rebuilding DuckDB produces identical query results for the same journal and `as_of_seq`.
27. Search at 50k and 1M events is measured against the full matrix.
28. Performance red lines are either met or amended through adjudication before shipping.
29. No FTS/ART/search index lands without a measured lower-rung failure.
30. Repeated-query RSS behavior is measured and honestly classified.
31. `sgt journal` maps one-to-one to the API contract.
32. `sgt journal --seq` displays the exact event.
33. Human output is readable and `--json` is stable.
34. Existing `sgt analytics`, graph, events, Work, and SSE behavior remains unchanged.
35. `sgt work transcript` remains separately owned and is not falsely claimed complete.
36. The TUI proposal remains independently reviewable and no TUI code is required to ship P2-JOURNAL.
37. Future TUI integration can consume the API without storage shortcuts.
38. Retention and cold-segment compression remain possible without query-state migration.
39. All behavior changes run the full multi-axis gauntlet.
40. Every fix carries a falsifiable pinning test and independent mutation evidence.
41. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and the shipping gate pass.
42. No daemon, client, or temporary artifact is leaked after the suites.
43. The final ledger entry records mission outcome, environmental behavior, Ponytail decisions, measurements, and deferred findings separately.

---

# 22. Source-to-Decision Map

| Proposal decision | Direct source |
|---|---|
| Journal truth; projections disposable | [`CLAUDE.md`](https://github.com/miztertea/sergeant-rs/blob/dcc963568668479e42b2ece9b17a05c53e86144a/CLAUDE.md), P0 [§§18–23](https://github.com/miztertea/sergeant-rs/blob/dcc963568668479e42b2ece9b17a05c53e86144a/reference/proposal-depot-rust-execution-surface.md) |
| JSONL / DuckDB / blob division | P0 [§21](https://github.com/miztertea/sergeant-rs/blob/dcc963568668479e42b2ece9b17a05c53e86144a/reference/proposal-depot-rust-execution-surface.md) |
| Chronology / graph / analytics sibling lenses | P0 [§§22–23](https://github.com/miztertea/sergeant-rs/blob/dcc963568668479e42b2ece9b17a05c53e86144a/reference/proposal-depot-rust-execution-surface.md) |
| Event-centered, sequence-anchored hits | [`event.rs`](https://github.com/miztertea/sergeant-rs/blob/dcc963568668479e42b2ece9b17a05c53e86144a/src/domain/event.rs), [`graph.rs`](https://github.com/miztertea/sergeant-rs/blob/dcc963568668479e42b2ece9b17a05c53e86144a/src/runtime/graph.rs) |
| `events` as visibility floor | [`analytics.rs`](https://github.com/miztertea/sergeant-rs/blob/dcc963568668479e42b2ece9b17a05c53e86144a/src/runtime/analytics.rs) |
| Exact event from journal | [`journal.rs::replay_after`](https://github.com/miztertea/sergeant-rs/blob/dcc963568668479e42b2ece9b17a05c53e86144a/src/runtime/journal.rs) |
| One owner / equal clients | [`CLAUDE.md`](https://github.com/miztertea/sergeant-rs/blob/dcc963568668479e42b2ece9b17a05c53e86144a/CLAUDE.md), [`api.rs`](https://github.com/miztertea/sergeant-rs/blob/dcc963568668479e42b2ece9b17a05c53e86144a/src/api.rs) |
| No arbitrary SQL | [`analytics.rs` canned-query and `table_rows` documentation](https://github.com/miztertea/sergeant-rs/blob/dcc963568668479e42b2ece9b17a05c53e86144a/src/runtime/analytics.rs) |
| Parameter binding | [DuckDB Prepared Statements](https://duckdb.org/docs/lts/sql/query_syntax/prepared_statements) |
| Literal text primitives | [DuckDB Text Functions](https://duckdb.org/docs/current/sql/functions/text) |
| JSON extraction only where allow-listed | [DuckDB JSON Functions](https://duckdb.org/docs/lts/data/json/json_functions) |
| Defer FTS | [DuckDB FTS extension, including non-updating index warning](https://duckdb.org/docs/lts/core_extensions/full_text_search) |
| Defer ART indexes | [DuckDB Indexes](https://duckdb.org/docs/current/sql/indexes), [Indexing guide](https://duckdb.org/docs/lts/guides/performance/indexing) |
| 50k baseline and read costs | [`baseline-cerberus-2026-08-11.md`](https://github.com/miztertea/sergeant-rs/blob/dcc963568668479e42b2ece9b17a05c53e86144a/docs/perf/baseline-cerberus-2026-08-11.md) |
| 1M scale, compression-first compatibility | [Adjudicated retention ruling in PR #58](https://github.com/miztertea/sergeant-rs/blob/cerberus/day2-adjudications/docs/gauntlet/notes/retention-design-ruling-draft-2026-08-11.md) |
| Surface adds usability, never functionality | [Adjudicated North Star in PR #58](https://github.com/miztertea/sergeant-rs/blob/cerberus/day2-adjudications/NORTH-STAR.md) |
| Journal future TUI destination remains separate | [T-series proposal in PR #58](https://github.com/miztertea/sergeant-rs/blob/cerberus/day2-adjudications/reference/proposal-tui-t-series.md) |
| Work transcript remains separate | [Dogfood E7](https://github.com/miztertea/sergeant-rs/blob/cerberus/day2-adjudications/docs/gauntlet/runs/dogfood-2026-08-11/run-manifest.md) and [North Star Wave 0](https://github.com/miztertea/sergeant-rs/blob/cerberus/day2-adjudications/NORTH-STAR.md) |
| Trajectory broader than final answer, narrower than indiscriminate transcript | [Portable Trajectory Record Format](https://app.notion.com/p/39a27ada618f81ef9658e33adce377ee) |
| Execution evidence survives its session | [Behavioral Execution Records](https://app.notion.com/p/39a27ada618f81a9bcaaf749d663cef0) |
| Do not turn free text into telemetry labels; time windows must be honest | [Privacy-Preserving Agent Observability](https://app.notion.com/p/39a27ada618f81238f9bf4699f5d8309) |
| Reveal work-domain constraints without forcing reconstruction | [Ecological Interface Design](https://app.notion.com/p/3ac27ada618f81909dd5d48e1f9b9912) |
| Human and agent surfaces use one engine/model | [Shared-Engine Human-Agent Workbench](https://app.notion.com/p/39a27ada618f81999694e0fbb019ca50) |
| Minimality decisions and rung order | [Ponytail Minimality Ladder](https://app.notion.com/p/39a27ada618f8100babadb321a70de9b) |

---

# 23. Final Position

Sergeant already remembers the work.

It remembers more than a process table, more than a final answer, and more than a provider transcript. It records intent, workflow, stage, execution, messages, tool activity, waits, questions, responses, usage, recovery, and terminal state in one durable chronology. It rebuilds that chronology into an analytical database and a relationship graph.

The missing capability is not another store.

It is the ability to ask the history an ordinary question and receive an answer that remains tied to evidence.

P2-JOURNAL supplies that capability through the smallest architecture consistent with Sergeant's own rules:

```text
one journal
one disposable DuckDB projection
one typed read contract
one CLI rendering
many future equal clients
```

It does not hand users SQL. It does not index every retained byte. It does not rank chronology by an opaque relevance score. It does not move search into the TUI. It does not create a second truth.

It makes the current truth usable.

> **DuckDB finds the evidence. The journal proves it.**
