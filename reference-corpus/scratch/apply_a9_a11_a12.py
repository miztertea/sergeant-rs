import json, hashlib

IN = "/home/user/sergeant-rs/reference-corpus/scratch/P8.quotes.ndjson"
OUT = "/home/user/sergeant-rs/reference-corpus/behavior-units/P8.ndjson"

with open(IN) as f:
    units = [json.loads(l) for l in f]
by_id = {u['id']: u for u in units}

def append_note(u, text):
    cur = u.get('notes')
    if cur:
        # avoid duplicate append
        if text in cur:
            return
        u['notes'] = cur + " " + text
    else:
        u['notes'] = text

# ---------------------------------------------------------------------------
# A11: mechanism nouns -> role nouns in `statement` (mechanism-not-the-behavior
# cases only; obsolete-mechanism records keep the mechanism, since there the
# mechanism itself IS the documented subject).
# ---------------------------------------------------------------------------

A11 = {
    "BU-P8-007": (
        "An external caller needs a durable, retried, deduplicated, exactly-once-delivered notification of a Work's terminal/needs-input state, which must survive daemon/process restarts and complete on its own schedule independent of any coordinator session or model turn.",
        "Statement normalized per A11: source-adjacent phrasing named a 'coordinator pane' (tmux mechanism); read as 'coordinator session', already covered by the adjacent 'session' term.",
    ),
    "BU-P8-026": (
        "Fleet cleanup for a task is blocked until the callback-acknowledgement check succeeds for it, and immediately before the actual deletion, cleanup re-verifies the same condition under a callback lock and writes a terminal seal that rejects any new event generation, closing the race between the acknowledgement check and the deletion.",
        "Statement normalized per A11: source names the check as the `sgt-callback check-acked` command specifically; the durable behavior is the acknowledgement-gated checkpoint itself, not that command's name.",
    ),
    "BU-P8-029": (
        "A project's identity is its project YAML's filename (without extension) at a fixed per-user path, and that filename is what every command uses to name the project — there is no separate project-id field.",
        "Statement normalized per A11: source says 'every sgt-* command'; the specific script-name family is a mechanism detail, not the durable behavior (every command, whatever its name, resolves identity the same way).",
    ),
    "BU-P8-030": (
        "Every command resolves a repo's relative path against one machine-wide dev_root setting read at startup, so changing dev_root in one place repoints every relative path in every project YAML at once, instead of requiring every YAML to be edited.",
        "Statement normalized per A11: source says 'every sgt-* script'; script-family naming is a mechanism detail, not the durable resolution behavior.",
    ),
    "BU-P8-034": (
        "A repo's name field is not free-form when graph extraction is in play: it must match [A-Za-z0-9._-]+, contain no spaces, and never be '.' or '..', because Sergeant uses it unescaped as a prefix on merged source paths.",
        "Statement normalized per A11: source names the specific 'sgt-graphify' script; the durable behavior is the validation rule that applies whenever graph extraction runs, not that script's name.",
    ),
    "BU-P8-037": (
        "Agent instruction prose is concatenated in a fixed three-layer order (defaults, then group, then repo), context resolution emits every nonempty layer in one block with later layers appearing later, and when directives conflict the later, more repo-specific directive is the intended authority — Sergeant never structurally merges or deduplicates the free-form prose itself.",
        "Statement normalized per A11: source names the specific 'sgt-context' script; the durable behavior is what context resolution (whatever performs it) emits, not that script's name.",
    ),
    "BU-P8-038": (
        "Even though instruction prose itself is never structurally merged, dispatch still builds one single normalized in-memory review-routing context from the mission, the repo's role, its group name, its group description, and the merged default/group/repo instructions.",
        "Statement normalized per A11: source names the specific 'sgt-dispatch' script; the durable behavior belongs to the dispatch procedure, not that script's name.",
    ),
    "BU-P8-041": (
        "First-install requires a fixed set of tools (Bash>=3.2, Git, authenticated gh, a session multiplexer, yq, Python 3, lsof, Marcus td, and at least one of OpenCode/Goose/Claude Code as a persistent interactive worker terminal) plus optional tools (mise, Treehouse, Graphify, no-mistakes, Node/npm), and installation does not proceed past this check until an automated or manual verification confirms every required tool is present.",
        "Statement normalized per A11: source names 'tmux' specifically as the required session multiplexer; per D2 this exact tool is obsolete-mechanism-adjacent even though the prerequisite-gate checkpoint itself is durable (see existing note on this unit).",
    ),
    "BU-P8-043": (
        "Sergeant deliberately does not install any harness-specific conversation-injection plugin; worker updates always surface only through durable fleet state read via the fleet-status monitor.",
        "Statement normalized per A11: source names the specific 'sgt-watch' script; the durable behavior is that updates surface only through a durable-state read path, not that script's name.",
    ),
    "BU-P8-045": (
        "Registering a first project requires the copied YAML's name to match its filename, every repo to have a unique name and correct path, clone URLs present wherever repository sync may need to clone, roles/groups that identify real ownership, and agent instructions that state commands and observable constraints rather than vague quality slogans.",
        "Statement normalized per A11: source names the specific 'sgt-sync' script; the durable behavior is the clone-capable-URL requirement, not that script's name.",
    ),
    "BU-P8-051": (
        "The getting-started checklist's own definition of a completed installation is a fixed nine-item checklist: required commands resolve, the coordinator process is running, the project listing shows the project exactly once, context resolution resolves every owning repo and instruction layer, required repos are cloned, Marcus td is installed and initialized, GitHub CLI can access required repos, optional Treehouse/Graphify features pass their own verification, and required (plus any reviewed extra) skills are present.",
        "Statement normalized per A11: source says 'the coordinator runs in a tmux pane', 'sgt-list shows the project', and 'sgt-context resolves'; tmux/pane and the specific script names are old-Sergeant mechanism, not the durable completion condition each checklist item checks for.",
    ),
    "BU-P8-052": (
        "Repository ownership and instructions must be identified from resolved project-context output (the project listing, context resolution, and task listing), never inferred from the current working directory.",
        "Statement normalized per A11: source names the specific 'sgt-list / sgt-context / sgt-td-list' scripts; the durable behavior is that ownership comes from resolved project-context output, not those script names.",
    ),
    "BU-P8-071": (
        "Workers wake the coordinator by updating one shared per-task notify marker that the coordinator's fleet-status monitor polls, so simultaneous updates from multiple repos can at worst collapse into one delayed wakeup, never into duplicate delivery.",
        "Statement normalized per A11: source names the specific 'sgt-watch' script as the poller; the durable behavior is the coalescing property of whatever polls the marker, not that script's name.",
    ),
    "BU-P8-072": (
        "Worker health must never be equated with the in_progress status alone; it requires exact live-process identity plus recent, meaningful progress evidence, using a defined fallback chain (session activity, then recorded progress timestamp, then file mtime only as a last resort), and once that evidence exceeds the grace window the worker stays in_progress but a nonterminal 'live worker stalled' diagnostic is recorded rather than an automatic failure or kill.",
        "Statement normalized per A11: source names 'pane_activity' specifically (tmux mechanism); read as 'session activity', the durable evidence-fallback shape independent of tmux (see existing note on this unit).",
    ),
    "BU-P8-073": (
        "Sergeant defines exactly seven durable worker states (in_progress, needs_input, blocked, waiting, orphaned, done, failed), each carrying a distinct meaning and a distinct required operator action, including that needs_input requires the operator to read the exact message and respond once per generation, and orphaned requires full reconciliation of process/session/worktree/branch/task/handoff before any recovery.",
        "Statement normalized per A11: source lists 'process/pane/worktree/branch/task/handoff'; 'pane' is old-Sergeant mechanism, read here as 'session'.",
    ),
    "BU-P8-080": (
        "A notified worker applies a decision exactly once, restores truthful status, and records the applied ID/generation/status; it then runs the acknowledgement command from its own recorded session, which validates post-application proof, stages replay evidence in a private archive entry, records acknowledgement, and only then clears active plaintext transport — and if any later step of that sequence fails, rerunning the exact same acknowledgement command with the same response ID must converge existing archive/acknowledgement/transport state without ever reapplying the decision.",
        "Statement normalized per A11: source says 'from its own recorded pane'; 'pane' is old-Sergeant tmux mechanism, read here as 'session'.",
    ),
    "BU-P8-083": (
        "The final validation launch runs no-mistakes interactively in a coordinator-owned session against the canonical intent and never passes --yes; its default medium profile skips the review and document steps because they are already covered by the required independent reviews and readiness evidence, and an explicit --skip list, when given, fully replaces that default.",
        "Statement normalized per A11: source names the specific 'sgt-validate' script and a 'coordinator-owned pane'; both are mechanism detail — the durable behavior is what the final validation launch does, in a coordinator-owned session, whatever the launching script is called.",
    ),
    "BU-P8-085": (
        "Canonical intent content must never appear in process arguments (readable by any local process via ps or /proc/<pid>/cmdline); the final validation launch therefore probes no-mistakes' own --help output before creating a run and requires an --intent-file capability, delivering intent through a path instead of argv.",
        "Statement normalized per A11: source names the specific 'sgt-validate' script; the durable behavior is what the final validation launch does, not that script's name.",
    ),
    "BU-P8-088": (
        "A live, unreleased coordinator-session owner is never displaced; a claim only succeeds when the prior owner is takeover-eligible (dead/absent session, mismatched recorded identity, or an explicit release from its own session) and the claiming session proves it truly runs inside the session it names by walking its own process ancestry, not by merely exporting an environment variable.",
        "Statement normalized per A11: source describes a 'coordinator-pane' owner and pane-proof by process ancestry (tmux mechanism); read here as 'session' throughout (see existing note on this unit for why the underlying non-displacement property, not the pane-proof mechanism, is what's durable).",
    ),
    "BU-P8-089": (
        "Every ownership transfer is durably appended to an owner-only handover log recording timestamp, reason, repository, prior and new session, and both identity tuples, and a release token is consumed by the claim that uses it so it can never be replayed by a third pane later.",
        "Statement normalized per A11: source says 'prior and new pane'; 'pane' is old-Sergeant tmux mechanism, read here as 'session'.",
    ),
    "BU-P8-090": (
        "If a validation launch fails before the child commits its release, Sergeant rolls back only the checkout, session, temp files, and fleet-state markers that this specific invocation both created and can still prove it owns; preexisting state, reused panes, dangling paths, and concurrent replacements are all preserved untouched.",
        "Statement normalized per A11: source lists 'the checkout, pane, temp files'; 'pane' is old-Sergeant tmux mechanism, read here as 'session'.",
    ),
    "BU-P8-094": (
        "Supported Sergeant commands must be used before any manual process, session, Git, or fleet-file operation, and exact errors and state must be preserved before attempting recovery.",
        "Statement normalized per A11: source says 'any manual process, tmux, Git, or fleet-file operation'; 'tmux' is old-Sergeant mechanism, read here as 'session' (manual session manipulation).",
    ),
    "BU-P8-097": (
        "A pending response must never be overwritten; the stall-recovery command must never be used for an active response generation (it is reserved for the exact in_progress live-worker-stalled diagnostic), and if the worker already applied the response and its archive entry exists, the fix is to rerun the same acknowledgement command from the worker's recorded session, not to intervene manually.",
        "Statement normalized per A11: source names the specific 'sgt-recover' script and 'the worker's recorded pane'; both are mechanism detail, read here as 'the stall-recovery command' and 'session'.",
    ),
    "BU-P8-098": (
        "A missing worker session is classified deterministically from context: a missing session plus durable blocked/handoff state means waiting work, while a missing session from in_progress with no handoff means orphan evidence.",
        "Statement normalized per A11: source says 'a missing pane'; 'pane' is old-Sergeant tmux mechanism, read here as 'session' (see existing note on this unit).",
    ),
}

for uid, (new_stmt, note) in A11.items():
    u = by_id[uid]
    u['statement'] = new_stmt
    append_note(u, note)

# ---------------------------------------------------------------------------
# A9: backfill non-empty alternatives_considered.
# Mandatory (record-shapes.md §4 / adjudication A9): representation in
# {workflow, stage, engine-gap}, or named in a classification-ledger conflict.
# A few additional non-mandatory agents-invariant/shared-context units are
# also backfilled here for rigor where a facially plausible adjacent rung
# exists (harmless; not required, not prohibited).
# ---------------------------------------------------------------------------

ALTS = {
    # mandatory: representation == workflow, empty list
    "BU-P8-055": [
        "stage (rejected: eight causally-ordered checkpoints spanning context load through PR merge and cleanup exceed what a single stage crossing could represent)",
        "agents-invariant (rejected: this is a bounded, triggerable procedure with its own completion condition, not a standing rule independent of any one piece of work)",
    ],
    # mandatory: named in conflict X6
    "BU-P8-064": [
        "stage-context (rejected: this is exactly BU-P1-057's reading, overturned at adjudication A13/X6 by D2, which settles the conflict against the live-rule reading)",
        "agents-invariant (rejected: would treat the persistent-TTY requirement as still binding, which the current engine's headless `claude -p` turns structurally contradict)",
    ],
    # non-mandatory, added for rigor (facially plausible adjacent rungs)
    "BU-P8-006": [
        "helper (rejected: this is a standing content rule for every doc in the set, not machinery subordinate to one checkpoint)",
    ],
    "BU-P8-032": [
        "agents-invariant (rejected: groups are project-specific configuration data, not a standing rule independent of project state)",
    ],
    "BU-P8-068": [
        "stage-context (rejected: applies to reasoning about any dispatched worker's trust boundary at any point, not one specific stage crossing)",
    ],
    "BU-P8-094": [
        "stage-context (rejected: this is the whole troubleshooting document's governing precedence rule, applying across every diagnostic scenario, not one stage's context)",
    ],
}

for uid, alts in ALTS.items():
    u = by_id[uid]
    assert u['alternatives_considered'] == [], f"{uid} not empty, refusing to overwrite"
    u['alternatives_considered'] = alts

# ---------------------------------------------------------------------------
# A12: split BU-P8-077 into BU-P8-110 (00-set-drain) and BU-P8-111
# (10-await-convergence); retire the original id with a note.
# ---------------------------------------------------------------------------

orig = by_id["BU-P8-077"]
assert orig['workflow'] == 'drain-fleet'

path = "/home/user/sergeant-rs/reference/sergeant-upstream/docs/using-sergeant.md"
raw = open(path, encoding='utf-8').read()

start_a = raw.index("A drain refuses new pane starts")
end_a = raw.index("later delivery.") + len("later delivery.")
quote_a = raw[start_a:end_a]
assert quote_a in raw

start_b = raw.index("`--wait` activates the drain first")
end_b = raw.index("without terminating any of them.") + len("without terminating any of them.")
quote_b = raw[start_b:end_b]
assert quote_b in raw

hash_a = "sha256:" + hashlib.sha256(quote_a.encode('utf-8')).hexdigest()
hash_b = "sha256:" + hashlib.sha256(quote_b.encode('utf-8')).hexdigest()

bu_110 = {
    "id": "BU-P8-110",
    "statement": "A drain refuses new worker starts within its scope while still storing incoming responses generation-safely for later delivery.",
    "source": {
        "path": "reference/sergeant-upstream/docs/using-sergeant.md",
        "locator": "L237-238 (Pause admission with a drain)",
        "quote": quote_a,
        "quote_hash": hash_a,
    },
    "scope": "admission control / drain",
    "trigger": "an operator wants to pause admission of new work for a scope",
    "outcome": "admission is refused immediately for that scope and in-flight responses are never lost",
    "authority": "operator",
    "confidence": "high",
    "notes": "Split from BU-P8-077 at N1 adjudication A12: this unit covers the drain's admission-refusal / response-safety aspect (workflow drain-fleet, stage 00-set-drain). See BU-P8-111 for the wait/convergence aspect. Source says 'new pane starts'; normalized to 'worker' per A11 (pane is the obsolete tmux mechanism, not the durable admission-control behavior) — this matches the wording BU-P8-077 already used.",
    "representation": "stage",
    "workflow": "drain-fleet",
    "stage": "00-set-drain",
    "rationale": "Admission refusal must take effect immediately and durably the moment a drain is requested, with no worker start slipping through and no response lost — a checkpoint any reimplementation would still need to cross (§6.3) before entering the separate wait phase.",
    "alternatives_considered": [
        "agents-invariant (rejected: scoped to one drain's admission window, not a standing rule independent of any drain)",
        "helper (rejected: the refusal is itself the observable durable checkpoint operators watch for, not subordinate machinery inside another checkpoint)",
    ],
    "engine_gap": None,
}

bu_111 = {
    "id": "BU-P8-111",
    "statement": "--wait activates the drain and then waits for in-scope live workers to finish their current turn and exit, and on timeout it leaves the drain active, exits nonzero, and names the unresolved workers without terminating any of them.",
    "source": {
        "path": "reference/sergeant-upstream/docs/using-sergeant.md",
        "locator": "L238-241 (Pause admission with a drain)",
        "quote": quote_b,
        "quote_hash": hash_b,
    },
    "scope": "admission control / drain",
    "trigger": "an operator invokes drain --wait after admission has already been refused for the scope",
    "outcome": "the operator either observes a clean graceful stop, or an explicit, non-silent timeout naming exactly which workers are still unresolved, and no worker is ever force-terminated by this path",
    "authority": "operator",
    "confidence": "high",
    "notes": "Split from BU-P8-077 at N1 adjudication A12: this unit covers the drain's wait/convergence aspect (workflow drain-fleet, stage 10-await-convergence). See BU-P8-110 for the admission-refusal aspect. This unit's quote_hash reproduces BU-P8-077's original stored hash exactly (sha256:068818277494390208a5cf71732a2daae48726ae93564c2849ab4d36364c1034), corroborating that the original citation's hashed span was already this sentence, not the full L231-243 locator range.",
    "representation": "stage",
    "workflow": "drain-fleet",
    "stage": "10-await-convergence",
    "rationale": "Waiting for live in-scope workers to finish their current turn and reporting an honest, named timeout rather than forcing termination is a distinct durable checkpoint a fresh implementation would still need (§6.3) — separate from admission refusal because a drain can be set without --wait ever being invoked.",
    "alternatives_considered": [
        "agents-invariant (rejected: only meaningful once a specific drain's --wait is invoked, not a standing rule)",
        "helper (rejected: the timeout / unresolved-worker report is itself the durable, operator-visible outcome, not subordinate machinery inside another checkpoint)",
    ],
    "engine_gap": None,
}

ORIGINAL_STORED_HASH_B077 = "sha256:068818277494390208a5cf71732a2daae48726ae93564c2849ab4d36364c1034"
assert hash_b == ORIGINAL_STORED_HASH_B077, "sanity: split aspect B hash should equal BU-P8-077's pre-A2 original stored hash"

orig['stage'] = None  # unchanged; kept for provenance
retirement_note = ("RETIRED at N1 adjudication A12: split into BU-P8-110 (00-set-drain aspect) "
                    "and BU-P8-111 (10-await-convergence aspect). Retained only for id stability "
                    "and provenance history — do not cite BU-P8-077 in new work; cite the successor "
                    "units instead. (Other reference-corpus files, e.g. draft-workflows/drain-fleet/, "
                    "engine-pressure.md, synthesis.md, provenance-map.md, still cite BU-P8-077 directly "
                    "as of this split and were not rewired by this P8.ndjson-scoped fix.)")
append_note(orig, retirement_note)

units.append(bu_110)
units.append(bu_111)

# ---------------------------------------------------------------------------
# Write out final NDJSON, preserving original relative order plus appended
# new records at the end (P8-077's own line is left in place, now a
# tombstone, per "keep ids stable except where a ruling splits/retires them").
# ---------------------------------------------------------------------------

ids = [u['id'] for u in units]
assert len(ids) == len(set(ids)), "duplicate id!"
assert len(units) == 111, len(units)

with open(OUT, 'w', encoding='utf-8') as f:
    for u in units:
        f.write(json.dumps(u, ensure_ascii=False) + "\n")

print("wrote", len(units), "units to", OUT)
