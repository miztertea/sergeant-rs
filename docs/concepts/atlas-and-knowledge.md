# Atlas and knowledge sources

Atlas is the daemon-owned store of what Sergeant has *learned about the world your estate works in*: the files under a declared knowledge path, the content of a repository at the commit an admission pinned, the structure a grammar found inside those files, and the datasets that stay where they are and get read in place. It is a second analytical database — `<data-dir>/atlas/atlas.duckdb` — deliberately separate from the operations projection that answers questions about Work.

Everything in Atlas is **derived evidence**. The authority is elsewhere and stays elsewhere: the journal for what happened, Git for repository content, and the operator's own bytes for everything read off a filesystem. Atlas holds a representation of those bytes that can be queried; it never holds the only copy of anything, and it never becomes the thing it describes.

## Two stores, two rebuild disciplines

Sergeant keeps two DuckDB files and they do not share a rebuild story. Confusing them is the single most likely way to lose derived evidence, so the difference is worth stating plainly.

The **operations projection** (`sergeant.duckdb`, holding the `ops` schema) is disposable. The daemon deletes it and re-folds it from the journal on every start. "Delete this file" and "restart the daemon" are the same operation, and nothing may come to depend on state that lives only there.

**Atlas persists.** Its `source` tables and its coverage rows are not a function of the journal — no replay reproduces them, because they are derived from source *bytes* plus the identity of the extractor that read those bytes. Deleting `atlas.duckdb` is therefore not free and not a no-op: it is re-derivable only by re-scanning every declared source, which costs whatever those sources cost to read, and until that happens coverage will correctly report that the derived evidence is gone.

What keeps the journal authoritative across the split is a summary event. Each completed scan journals exactly one compact `source.scanned` record — the source, the generation, its content key, the counts by coverage status, and which extractors ran — and never a per-file event stream. The journal carries the trail; Atlas carries the detail. A crash between the two is a resolved case rather than an open one: a scan's rows land as one provisional generation, the summary is journaled, and a second transaction confirms it. No read path can see an unconfirmed generation, and the next daemon start resolves any a crash left behind — by asking the journal, because the two crash windows have opposite correct answers and the database cannot tell them apart on its own. A generation whose `source.scanned` summary never reached the journal is **evicted**: that scan never completed, and the eviction leaves an explicit `generation_evicted` coverage row naming the window it closed. A generation whose summary *is* durable is **confirmed** instead, exactly as the live path would have confirmed it — the journal already vouches for that scan and has already broadcast it, so discarding the rows would leave the trail claiming a scan the store had thrown away. A start that followed no crash finds nothing provisional and does neither.

## Sources are evidence; repositories are authority

A `[[repo]]` entry in `sergeant.toml` declares something the estate *mutates*: it is cloned into a mount, Work surfaces are cut from it as linked worktrees, and a branch carries the result. A `[[knowledge]]` entry declares something the estate only *reads*.

```toml
[[knowledge]]
name = "team notes"
path = "/home/me/OneDrive/Team Notes"
ignore = ["archive/**"]
```

Nothing is cloned, no worktree is cut, and nothing is ever written back. A declared path that resolves inside a repository mount, inside the surfaces directory, or inside the data directory is refused by name when the manifest is parsed — those are exactly the places the estate mutates, and evidence about a world you are changing underneath yourself is not evidence. `sgt knowledge add` and `sgt knowledge list` declare and read these entries back; both are pure manifest operations with no daemon involved.

A cloud-synced folder is an ordinary directory as far as Sergeant is concerned. The sync client owns transport and authentication and leaves bytes on the filesystem; Sergeant reads those bytes. There is no provider integration behind this, and none is implied.

Repository sources are read differently and never through the working tree. A repository is indexed at the commit its admission pinned, straight out of the Git object store, in batched object reads — no fetch, no pull, no branch switch, nothing written. If the mount's `HEAD` moves while a scan is running, the scan stays on the commit it pinned and the move is reported beside the result as a drift observation rather than blended into it. A Work surface is indexed as an *overlay* on its base commit: files the Work changed are hashed from the surface, everything else keeps the base tree's own object ids, and the overlay is scoped to that Work and removed when the Work is retired.

## Generations, and when a re-scan costs nothing

A **generation** is one source's world at one moment, identified by content. For local knowledge that identity is a hash over every acquired file's path and content hash; for a repository it is the tree, which is why a commit that changed no file — an empty commit, a reworded message — is recognized as the same world. A generation is superseded on either of two triggers: the source bytes changed, or the **extractor identities** that read those bytes changed — a grammar upgrade, a version bump, a `context_fields` change. Either way the superseded generation leaves an explicit eviction row rather than disappearing. A re-scan that finds the same bytes *and* the same extractors is the one case that costs nothing: it writes nothing and evicts nothing. (A third case writes nothing and evicts nothing for a different reason: a source root that could not be read at all changed no bytes, so the standing generation is kept and the unavailability is recorded as a coverage row against it.)

Cached facts are keyed on **content identity plus extractor identity**, and both halves matter. Repository content keys on Git's own blob object id, so bytes Git already hashed are never hashed again and two identical files share one extraction by construction; local knowledge keys on a content hash of its own. The extractor half is what notices a change the bytes cannot show: upgrade a grammar and the same file's symbols are re-derived, because serving the previous parser's rows under unchanged bytes would be stale evidence that no content comparison could detect. It also means one file read two ways is two independent extractions — revising a grammar re-derives symbols without invalidating a single document unit.

## Coverage is evidence, not a log line

Every path a scan sees leaves exactly one coverage row, and there is no eighth "silently skipped" state:

| Status | Meaning |
|---|---|
| `discovered` | seen by the walk (recorded on its own only for a container whose children carry their own rows) |
| `indexed` | bytes read, an extractor ran, units written |
| `excluded` | refused at the acquisition boundary, *before* the bytes were read |
| `unavailable` | present but unreadable right now — permissions, vanished mid-scan, a link this build does not follow |
| `unsupported` | readable, but no extractor in this build claims it |
| `error` | an extractor was chosen and failed |
| `generation_evicted` | a whole generation's rows were removed, and why |

This vocabulary is what makes the secrets posture checkable rather than aspirational. Dotfiles, `.env` files, private keys, keystores and credential-shaped names are excluded at the acquisition boundary before a file is opened, and each source's own `ignore` globs extend that floor and can never narrow it — but an excluded path is **counted and reported as excluded**, with the pattern that refused it. A file that is missing from the record and a file that was deliberately refused look nothing alike, which is the entire point.

The same honesty applies upward. A file a grammar cannot parse is reported `error` and contributes no symbols at all, rather than the shorter symbol list tree-sitter's error tolerance would happily produce and that nothing downstream could distinguish from a complete one. A language no grammar in this build claims is `unsupported` and says so, rather than being parsed by an almost-right grammar. `sgt intelligence status` reports every indexed source's generation and its full coverage breakdown, and `sgt doctor` carries an `atlas` row that warns when paths could not be read or extracted.

One honest limit belongs here rather than in a footnote: a file that a sync client has listed but not yet materialized, and that the filesystem presents as a readable empty file, is indexed as the empty file it appears to be. Detecting that case is best-effort heuristics by nature and is not shipped yet.

## What structure gets extracted

Text and Markdown become **document units** and heading-delimited **section units**, each carrying byte offsets into the original file, so every derived unit can be traced back to the bytes it came from. Rust, TOML, Markdown, Python, JavaScript, TypeScript and shell are additionally parsed with tree-sitter grammars into a symbol index, the occurrence sites that wrote each symbol, and import edges.

What is stored is **syntax, not semantics**. A symbol's label is what the grammar called the node — `function`, `struct`, `class`, `heading` — and an import's target is the text the file wrote, unresolved. Nothing follows a re-export, nothing decides which definition a name meant, and nothing claims to. Treat the symbol index as a very good index, not as a compiler's view.

CSV, JSON and Parquet files under a knowledge source are indexed as **tabular datasets read in place**: DuckDB opens the operator's own file through a canned, fully parameterized query, and no copy of those bytes lands in Sergeant's store. Each dataset records where it is, what it hashes to, its columns and a bounded row count, and each canned query's answer is stored carrying the generation it read, the identity of the question, and a hash of its own output — so an answer can be checked rather than trusted.

A tabular row's *text* becomes a retrievable context unit only through an operator-declared column allowlist, and the default is none. `context_fields` names the columns that may be exposed; without it a dataset is still discovered, registered, counted and profiled in aggregate, and not one row's text is published. A CSV of support tickets is an ordinary knowledge source whose `email` column is not, and no path pattern can express that — which is why the control is a column list rather than another glob. Narrowing the list later retracts what a wider one exposed: the declared columns are part of the reader's identity, so changing them supersedes the generation.

## The map surface

`sgt map` reads the world Atlas derived: `repos` for the indexed repository sources, `outline` for one source's titled structure, `symbol` for the symbol index by exact name, `references` for every recorded site of one name, and `stats` for what the map actually holds per source.

Every one of these is canned and parameterized. There is no client SQL, no client-named path, and no client-supplied match pattern — a client chooses a verb and supplies values, never a query. Every read is bounded by a row cap a client may lower and never raise. `map neighbors` and `map changed` are deliberately absent: they land with the work whose consumers need them, rather than shipping now as verbs with nothing behind them.

The daemon is Atlas's writer. Clients ask it questions over the API; they do not open the store and reach in.

One thing this release does not yet include is a way to *start* a scan. Atlas's writers and its read surfaces both ship; nothing between them is wired to a command, a route, or a scheduled job. On a fresh installation the store is therefore empty, `sgt intelligence status` reports nothing indexed, and `sgt doctor`'s `atlas` row says so rather than implying a fault. That is stated here rather than left to be discovered.

## Decisions

| ID | Decision |
|---|---|
| D1 | Atlas is derived evidence. The journal, Git, and the operator's original bytes remain the authority, and Atlas never becomes the only copy of anything. |
| D2 | Two databases, two rebuild disciplines: the operations projection is deleted and re-folded from the journal on every daemon start; Atlas persists across restarts and is re-derivable only by re-scanning. |
| D3 | One compact `source.scanned` event per completed scan keeps the journal authoritative without a per-file event stream. Rows are provisional until that summary is journaled and confirmed; a crash leaves both or neither, never half. |
| D4 | A knowledge source is read-only evidence and never a mount: nothing is cloned, no worktree is cut, nothing is written back, and a path inside the estate's own mutable territory is refused by name. |
| D5 | Repositories are indexed from the object store at the admission-pinned commit; a `HEAD` that moves mid-scan is reported as drift, never blended into the result. |
| D6 | A generation is superseded on either of two triggers — the bytes it was derived from changed, or the extractor identities that read them changed — and the superseded generation leaves an explicit eviction row rather than vanishing. A re-scan finding the same bytes *and* the same extractors writes and evicts nothing. |
| D7 | Cached facts key on content identity **plus** extractor identity, so a changed parser re-derives under unchanged bytes and one file read two ways is two independent extractions. |
| D8 | Every path a scan sees leaves exactly one coverage row. Excluded bytes are counted and reported as excluded; there is no silently-skipped state. |
| D9 | Structural extraction is syntax-derived and labeled as such. A file a grammar cannot parse is an `error` with no symbols, never a partial parse. |
| D10 | Tabular data stays relational and is read in place. A row's text is exposed as a context unit only through an operator-declared column allowlist whose default is none. |
| D11 | Query surfaces are canned, parameterized and bounded. No client SQL, no client-named path, no client pattern — and the daemon is the sole writer. |

See [estates and Git surfaces](estates-and-git.md) for the repository and Work-surface boundary Atlas reads without disturbing, [host runtime and estates](host-runtime.md) for the daemon that owns the store, [security and trust](security-and-trust.md) for the trust model the secrets posture sits inside, and [`sergeant.toml`](../reference/sergeant-toml.md) for the exact `[[knowledge]]` schema.
