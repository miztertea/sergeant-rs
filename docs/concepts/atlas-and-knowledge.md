# Atlas and knowledge sources

Atlas is the daemon-owned store of what Sergeant has *learned about the world your estate works in*: the files under a declared knowledge path, the content of a repository at the commit an admission pinned, the structure a grammar found inside those files, and the datasets that stay where they are and get read in place. It lives in the estate's one analytical database — `<data-dir>/atlas/atlas.duckdb` — alongside the operations projection that answers questions about Work, in its own schemas.

Everything in Atlas is **derived evidence**. The authority is elsewhere and stays elsewhere: the journal for what happened, Git for repository content, and the operator's own bytes for everything read off a filesystem. Atlas holds a representation of those bytes that can be queried; it never holds the only copy of anything, and it never becomes the thing it describes.

## One store, two rebuild disciplines

Sergeant keeps **one** DuckDB file, `atlas.duckdb`, with five schemas: `meta`, `ops`, `source`, `git` and `context`. One database is what makes a question that spans two of them — "which source generations belong to this Work?" — an ordinary join instead of a federation across separate stores.

What that one file does *not* have is one rebuild story, and confusing the two is the single most likely way to lose derived evidence.

The **operations projection** (the `ops` schema) is disposable. It is a pure fold of the journal, and the daemon drops the whole `ops` schema and re-folds it on every start. "Restart the daemon" is the only rebuild there is, and nothing may come to depend on state that lives only there.

**Everything else persists.** The `source` tables and the coverage rows are not a function of the journal — no replay reproduces them, because they are derived from source *bytes* plus the identity of the extractor that read those bytes.

So: **restarting the daemon rebuilds `ops` and costs nothing. Deleting `atlas.duckdb` is a different act, and it is not free.** Deleting the file still brings every `ops` row back from the journal on the next start — and it discards every persisted source generation, which is re-derivable only by re-scanning every declared source, at whatever those sources cost to read. Until that happens `sgt intelligence status` and `sgt doctor` will correctly report that this host has indexed nothing. If what you actually want is a fresh operations projection, restart the daemon; there is no reason to delete the file.

What keeps the journal authoritative across that split is a summary event. Each completed scan journals exactly one compact `source.scanned` record — the source, the generation, its content key, the counts by coverage status, and which extractors ran — and never a per-file event stream. The journal carries the trail; Atlas carries the detail. A crash between the two is a resolved case rather than an open one: a scan's rows land as one provisional generation, the summary is journaled, and a second transaction confirms it. No read path can see an unconfirmed generation, and the next daemon start resolves any a crash left behind — by asking the journal, because the two crash windows have opposite correct answers and the database cannot tell them apart on its own. A generation whose `source.scanned` summary never reached the journal is **evicted**: that scan never completed, and the eviction leaves an explicit `generation_evicted` coverage row naming the window it closed. A generation whose summary *is* durable is **confirmed** instead, exactly as the live path would have confirmed it — the journal already vouches for that scan and has already broadcast it, so discarding the rows would leave the trail claiming a scan the store had thrown away. A start that followed no crash finds nothing provisional and does neither.

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

Repository sources are read differently and never through the working tree. A repository is indexed at the commit its admission pinned, straight out of the Git object store, in batched object reads — no fetch, no pull, no branch switch, nothing written. If the mount's `HEAD` moves while a scan is running, the scan stays on the commit it pinned and the move is reported beside the result as a drift observation rather than blended into it. A Work surface **is** indexed as an *overlay* on its base commit — files the Work changed hashed from the surface, everything else keeping the base tree's own object ids, scoped to that Work and removed when the Work is retired. The daemon drives it: once when the surface is bound, again each time one of the Work's turns ends while the surface is still bound, and it is evicted when the surface is torn down. `sgt intelligence scan` does not drive it and neither does any query — a search verb stays a pure reader. Because the scan happens at a turn boundary and never mid-turn, an overlay describes the surface **as of the end of the Work's last completed turn**, and a `--work` answer says which instant that was rather than implying it is live.

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
| `online_only` | best-effort: this looks like a cloud-sync placeholder (zero allocated disk blocks, non-zero reported size) and was never opened, to avoid triggering a hydration download |
| `generation_evicted` | a whole generation's rows were removed, and why |

This vocabulary is what makes the secrets posture checkable rather than aspirational. Dotfiles, `.env` files, private keys, keystores and credential-shaped names are excluded at the acquisition boundary before a file is opened, and each source's own `ignore` globs extend that floor and can never narrow it — but an excluded path is **counted and reported as excluded**, with the pattern that refused it. A file that is missing from the record and a file that was deliberately refused look nothing alike, which is the entire point.

The same honesty applies upward. A file a grammar cannot parse is reported `error` and contributes no symbols at all, rather than the shorter symbol list tree-sitter's error tolerance would happily produce and that nothing downstream could distinguish from a complete one. A language no grammar in this build claims is `unsupported` and says so, rather than being parsed by an almost-right grammar. `sgt intelligence status` reports every indexed source's generation and its full coverage breakdown, and `sgt doctor` carries an `atlas` row that warns when paths could not be read or extracted.

One honest limit belongs here rather than in a footnote: a file that a sync client has listed but not yet materialized can look, at the filesystem level, exactly like a legitimate sparse file with a hole in it — a disk image, a punched-out log. This build's own signal (zero allocated blocks against a non-zero reported size, read from metadata the walk already fetches, so no extra syscall and never an `open()` that could itself trigger a download) reports `online_only` for that shape, but it is a heuristic and says so on every row it produces: a real sparse file can be misclassified (a false positive), and a sync client that reports full block allocation before the byte is actually fetched is not caught at all (a false negative). A file that really is empty (`size == 0`) is never flagged — calling it a placeholder would be the opposite mistake.

## What structure gets extracted

Text and Markdown become **document units** and heading-delimited **section units**, each carrying byte offsets into the original file, so every derived unit can be traced back to the bytes it came from. Rust, TOML, Markdown, Python, JavaScript, TypeScript and shell are additionally parsed with tree-sitter grammars into a symbol index, the occurrence sites that wrote each symbol, and import edges.

What is stored is **syntax, not semantics**. A symbol's label is what the grammar called the node — `function`, `struct`, `class`, `heading` — and an import's target is the text the file wrote, unresolved. Nothing follows a re-export, nothing decides which definition a name meant, and nothing claims to. Treat the symbol index as a very good index, not as a compiler's view.

CSV, JSON and Parquet files under a knowledge source are indexed as **tabular datasets read in place**: DuckDB opens the operator's own file through a canned, fully parameterized query, and no copy of those bytes lands in Sergeant's store. Each dataset records where it is, what it hashes to, its columns and a bounded row count, and each canned query's answer is stored carrying the generation it read, the identity of the question, and a hash of its own output — so an answer can be checked rather than trusted.

A tabular row's *text* becomes a retrievable context unit only through an operator-declared column allowlist, and the default is none. `context_fields` names the columns that may be exposed; without it a dataset is still discovered, registered, counted and profiled in aggregate, and not one row's text is published. A CSV of support tickets is an ordinary knowledge source whose `email` column is not, and no path pattern can express that — which is why the control is a column list rather than another glob. Narrowing the list later retracts what a wider one exposed: the declared columns are part of the reader's identity, so changing them supersedes the generation.

## The map surface

`sgt map` reads the world Atlas derived: `repos` for the indexed repository sources, `outline` for one source's titled structure, `symbol` for the symbol index by exact name, `references` for every recorded site of one name, and `stats` for what the map actually holds per source.

Every one of these is canned and parameterized. There is no client SQL, no client-named path, and no client-supplied match pattern — a client chooses a verb and supplies values, never a query. Every read is bounded by a row cap a client may lower and never raise. `map neighbors` and `map changed` are deliberately absent: they land with the work whose consumers need them, rather than shipping now as verbs with nothing behind them.

The daemon is Atlas's writer. Clients ask it questions over the API; they do not open the store and reach in. The one diagnostic exception is `sgt doctor`'s `atlas` row, which reads the store file directly when no daemon holds it — it checks for the file first and never creates one, and when a running daemon has the store locked it says to ask the daemon instead.

`sgt intelligence scan` drives a full scan of everything the estate declares through the daemon, on the intelligence lane: every `[[repo]]` repository (through the identical object-store path described above — never the folder walker, which would lose the pinned commit, the blob-object keys and drift observation), every `[[knowledge]]` source (through the folder walker), and every external Git source already added on this host (refreshed). Each row in the report names its own kind alongside its coverage, so the three stay honestly distinguishable even when two sources happen to hold identical bytes. `sgt knowledge scan` is kept as a working, unchanged spelling — it runs the identical scan — but `sgt intelligence scan` is the name to reach for now, since the trigger stopped being knowledge-folder-only. On a fresh installation the store is still empty until one of these (or `sgt intelligence add`, below) is run at least once, and `sgt doctor`'s `atlas` row says so rather than implying a fault. Scheduling and cadence are not built: this is one call, one scan, one report, and a recurring trigger is later work's, when retrieval needs one.

## Decisions

| ID | Decision |
|---|---|
| D1 | Atlas is derived evidence. The journal, Git, and the operator's original bytes remain the authority, and Atlas never becomes the only copy of anything. |
| D2 | One store, two rebuild disciplines: the `ops` schema is dropped and re-folded from the journal on every daemon start; the source schemas persist across restarts and are re-derivable only by re-scanning. |
| D3 | One compact `source.scanned` event per completed scan keeps the journal authoritative without a per-file event stream. Rows are provisional until that summary is journaled and confirmed; a crash leaves both or neither, never half. |
| D4 | A knowledge source is read-only evidence and never a mount: nothing is cloned, no worktree is cut, nothing is written back, and a path inside the estate's own mutable territory is refused by name. |
| D5 | Repositories are indexed from the object store at the admission-pinned commit; a `HEAD` that moves mid-scan is reported as drift, never blended into the result. |
| D6 | A generation is superseded on either of two triggers — the bytes it was derived from changed, or the extractor identities that read them changed — and the superseded generation leaves an explicit eviction row rather than vanishing. A re-scan finding the same bytes *and* the same extractors writes and evicts nothing. |
| D7 | `sgt intelligence scan` (`sgt knowledge scan` still works, unchanged) is the scan trigger, and it is estate-scoped: every declared `[[repo]]`, every declared `[[knowledge]]` source, and every already-added external Git source, in one call, each reported under its own kind. Scheduling and cadence stay unbuilt on purpose. `sgt intelligence add`/`list` acquires an external Git source into a bare, no-working-tree host cache outside every estate, at an allowlisted `https://`/`ssh://` locator only — refused before Git ever sees anything else — and reads it through the identical object-store plumbing a `[[repo]]` mount already uses. |
| D8 | Cached facts key on content identity **plus** extractor identity, so a changed parser re-derives under unchanged bytes and one file read two ways is two independent extractions. |
| D9 | Every path a scan sees leaves exactly one coverage row. Excluded bytes are counted and reported as excluded; there is no silently-skipped state. |
| D10 | Structural extraction is syntax-derived and labeled as such. A file a grammar cannot parse is an `error` with no symbols, never a partial parse. |
| D11 | Tabular data stays relational and is read in place. A row's text is exposed as a context unit only through an operator-declared column allowlist whose default is none. |
| D12 | Query surfaces are canned, parameterized and bounded. No client SQL, no client-named path, no client pattern — and the daemon is the sole writer. |
| D13 | A suspected cloud-sync placeholder (zero allocated blocks, non-zero reported size) is reported `online_only`, never `indexed` with zero units, and is never opened to check further — classification uses only metadata the walk already fetches (`lstat`/`stat`), because `open()` is what triggers a hydration download on some cloud filesystems. It is a heuristic, said so on every row: false positives (an ordinary sparse file) and false negatives (a placeholder a sync client reports fully allocated) are both possible. |
| D14 | Work-surface overlay indexing is driven by the daemon's own Work lifecycle — at surface bind, at each turn boundary while the surface is bound, and evicted at teardown — never by `sgt intelligence scan` and never by a query. An overlay is a snapshot as of the end of the Work's last completed turn, and the answer carries that instant. |

See [estates and Git surfaces](estates-and-git.md) for the repository and Work-surface boundary Atlas reads without disturbing, [host runtime and estates](host-runtime.md) for the daemon that owns the store, [security and trust](security-and-trust.md) for the trust model the secrets posture sits inside, and [`sergeant.toml`](../reference/sergeant-toml.md) for the exact `[[knowledge]]` schema.
