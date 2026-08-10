#!/usr/bin/env python3
"""
Structural lint for the N1 reference corpus.

Checks (per A15 / adjudication-round1.md, docs/icm/record-shapes.md §3-§4):

  1. NDJSON validity + all record-shapes §3/§4 required fields, including
     `source.quote`, using the amended representation vocabulary.
  2. Unique unit ids across the whole corpus.
  3. Every `quote` appears contiguously in its cited file AND sha256 of that
     exact span equals `quote_hash` (span_bytes units: verify the 500-char
     prefix appears, then hash the located full span). `citation: disputed`
     units are exempt from verification but counted.
  4. Every `engine-gap` unit's nested `engine_gap` object carries the full
     §6.7 template with canonical field names.
  5. Package structure:
       - workflow.toml `stages` order == stage-directory lexical order.
       - actor (stage) directories have a CONTEXT.md with an Inputs table
         whose paths resolve.
       - index.md front matter parses with required fields.
       - no workflow package exists outside draft-workflows/.
       - each package's provenance.md cites only existing unit ids.
  6. Every source-inventory.md "decompose" file appears in provenance-map.md.
  7. `rationale` is non-null everywhere; `alternatives_considered` is
     non-empty for every workflow/stage/engine-gap unit and every unit named
     in a classification-ledger.md §3 conflict.

Stdlib only. Exit 0 iff clean (no defects of any kind found).
"""

import hashlib
import json
import os
import re
import sys
import tomllib

ROOT = os.path.dirname(os.path.abspath(__file__))          # reference-corpus/
REPO_ROOT = os.path.dirname(ROOT)                            # sergeant-rs/

BEHAVIOR_UNITS_DIR = os.path.join(ROOT, "behavior-units")
DRAFT_WORKFLOWS_DIR = os.path.join(ROOT, "draft-workflows")
SOURCE_INVENTORY = os.path.join(ROOT, "source-inventory.md")
PROVENANCE_MAP = os.path.join(ROOT, "provenance-map.md")
CLASSIFICATION_LEDGER = os.path.join(ROOT, "classification-ledger.md")

REPRESENTATION_VOCAB = {
    "agents-invariant",
    "workflow",
    "stage",
    "stage-context",
    "helper",
    "shared-helper",
    "shared-context",
    "obsolete-mechanism",
    "engine-gap",
}

CONFIDENCE_VOCAB = {"high", "medium", "low"}

ENGINE_GAP_KEYS = {
    "behavior",
    "source_evidence",
    "lower_rungs_attempted",
    "why_each_fails",
    "minimum_runtime_capability_required",
    "observable_acceptance_test",
}

BU_ID_RE = re.compile(r"\bBU-P\d+-\d+\b")

QUOTE_HASH_RE = re.compile(r"^sha256:[0-9a-f]{64}$")


class Lint:
    def __init__(self):
        self.errors = []   # list[str]
        self.n_checks_run = 0

    def fail(self, msg):
        self.errors.append(msg)

    def report(self):
        return self.errors


def is_disputed(record):
    notes = record.get("notes") or ""
    if record.get("citation") == "disputed":
        return True
    if "citation: disputed" in notes or "citation:disputed" in notes:
        return True
    return False


def load_units(lint):
    """Returns (units: dict[id -> record], disputed_count, ndjson_errors)."""
    units = {}
    disputed = 0
    if not os.path.isdir(BEHAVIOR_UNITS_DIR):
        lint.fail(f"behavior-units directory not found: {BEHAVIOR_UNITS_DIR}")
        return units, disputed

    for fn in sorted(os.listdir(BEHAVIOR_UNITS_DIR)):
        if not fn.endswith(".ndjson"):
            continue
        path = os.path.join(BEHAVIOR_UNITS_DIR, fn)
        with open(path, encoding="utf-8") as f:
            for lineno, raw in enumerate(f, 1):
                line = raw.strip()
                if not line:
                    continue
                loc = f"{os.path.relpath(path, ROOT)}:{lineno}"
                try:
                    record = json.loads(line)
                except json.JSONDecodeError as e:
                    lint.fail(f"[1:ndjson] {loc}: invalid JSON: {e}")
                    continue

                uid = record.get("id")
                if not uid or not isinstance(uid, str):
                    lint.fail(f"[1:required-field] {loc}: missing/invalid `id`")
                    uid = f"<{loc}>"

                # --- check 1: required fields (§3 behavior-unit record) ---
                for field in ("statement", "scope", "trigger", "outcome", "authority", "confidence"):
                    v = record.get(field)
                    if not v or not isinstance(v, str):
                        lint.fail(f"[1:required-field] {uid} ({loc}): missing/empty `{field}`")

                confidence = record.get("confidence")
                if confidence is not None and confidence not in CONFIDENCE_VOCAB:
                    lint.fail(f"[1:confidence] {uid} ({loc}): confidence `{confidence}` not in {sorted(CONFIDENCE_VOCAB)}")

                source = record.get("source")
                if not isinstance(source, dict):
                    lint.fail(f"[1:required-field] {uid} ({loc}): missing/invalid `source` object")
                    source = {}
                for field in ("path", "locator", "quote", "quote_hash"):
                    v = source.get(field)
                    if not v or not isinstance(v, str):
                        lint.fail(f"[1:required-field] {uid} ({loc}): missing/empty `source.{field}`")

                quote = source.get("quote")
                if isinstance(quote, str) and len(quote) > 500:
                    lint.fail(f"[1:quote-length] {uid} ({loc}): source.quote is {len(quote)} chars, must be <=500")

                qhash = source.get("quote_hash")
                if isinstance(qhash, str) and not QUOTE_HASH_RE.match(qhash):
                    lint.fail(f"[1:quote-hash-shape] {uid} ({loc}): source.quote_hash `{qhash}` is not `sha256:<64 hex>`")

                span_bytes = source.get("span_bytes")
                if span_bytes is not None and not isinstance(span_bytes, int):
                    lint.fail(f"[1:span-bytes] {uid} ({loc}): source.span_bytes must be an integer")

                # --- check 1 (cont'd): required classification fields (§4) ---
                representation = record.get("representation")
                if representation not in REPRESENTATION_VOCAB:
                    lint.fail(f"[1:representation] {uid} ({loc}): representation `{representation}` not in amended vocabulary {sorted(REPRESENTATION_VOCAB)}")

                if "rationale" not in record:
                    lint.fail(f"[1:required-field] {uid} ({loc}): missing `rationale` key")
                if "alternatives_considered" not in record or not isinstance(record.get("alternatives_considered"), list):
                    lint.fail(f"[1:required-field] {uid} ({loc}): missing/invalid `alternatives_considered` list")
                if "engine_gap" not in record:
                    lint.fail(f"[1:required-field] {uid} ({loc}): missing `engine_gap` key (must be present, null unless representation=='engine-gap')")

                # stage required (non-null) when representation is `stage` (a
                # named stage boundary), with no exemption — including a
                # retired/superseded record: it still names the materialized
                # stage directory its content now lives under (or, before
                # this closure round, `is_retired()` exempted it here; that
                # carve-out is removed as of N1 closure V13, since every
                # `stage`-representation record, retired or not, now carries
                # a real non-null `stage`). `stage-context` legitimately
                # attaches to a workflow's own Layer-1 CONTEXT.md with no
                # single owning stage (verified against the corpus: every
                # such case's own `rationale` states it's workflow-wide
                # context, not a missing field) — record-shapes.md's "stage
                # or actor-stage" conditional binds the `stage` rung
                # specifically, not every rung that happens to touch a stage.
                if representation == "stage" and record.get("stage") is None:
                    lint.fail(f"[1:conditional-field] {uid} ({loc}): representation `{representation}` requires non-null `stage`")

                if is_disputed(record):
                    disputed += 1

                # --- check 2: unique ids ---
                if uid in units:
                    lint.fail(f"[2:duplicate-id] {uid}: also defined at {units[uid][1]} and {loc}")
                else:
                    units[uid] = (record, loc)

    return units, disputed


def verify_quotes(lint, units, disputed_ids):
    file_cache = {}

    def get_bytes(path):
        if path not in file_cache:
            full = os.path.join(REPO_ROOT, path)
            with open(full, "rb") as f:
                file_cache[path] = f.read()
        return file_cache[path]

    n_verified = 0
    for uid, (record, loc) in units.items():
        if uid in disputed_ids:
            continue
        source = record.get("source") or {}
        quote = source.get("quote")
        qhash = source.get("quote_hash", "")
        path = source.get("path")
        if not (isinstance(quote, str) and isinstance(qhash, str) and isinstance(path, str)):
            # already flagged as a required-field defect
            continue
        qhash_hex = qhash[len("sha256:"):] if qhash.startswith("sha256:") else qhash

        try:
            filebytes = get_bytes(path)
        except OSError as e:
            lint.fail(f"[3:source-file] {uid} ({loc}): cannot read source.path `{path}`: {e}")
            continue

        quote_b = quote.encode("utf-8")
        span_bytes = source.get("span_bytes")

        if span_bytes is None:
            idx = filebytes.find(quote_b)
            if idx == -1:
                lint.fail(f"[3:quote-not-found] {uid} ({loc}): source.quote does not appear contiguously in `{path}`")
                continue
            digest = hashlib.sha256(quote_b).hexdigest()
            if digest != qhash_hex:
                lint.fail(f"[3:hash-mismatch] {uid} ({loc}): sha256({path!r} quote span) = {digest} != quote_hash {qhash_hex}")
                continue
            n_verified += 1
        else:
            if not isinstance(span_bytes, int) or span_bytes <= 0:
                lint.fail(f"[3:span-bytes] {uid} ({loc}): invalid span_bytes {span_bytes!r}")
                continue
            idx = filebytes.find(quote_b)
            matched = False
            while idx != -1:
                span = filebytes[idx: idx + span_bytes]
                if len(span) == span_bytes and hashlib.sha256(span).hexdigest() == qhash_hex:
                    matched = True
                    break
                idx = filebytes.find(quote_b, idx + 1)
            if not matched:
                lint.fail(f"[3:span-not-located] {uid} ({loc}): 500-char prefix not found, or no located {span_bytes}-byte span in `{path}` hashes to quote_hash")
                continue
            n_verified += 1

    return n_verified


def check_engine_gap_template(lint, units):
    n_checked = 0
    for uid, (record, loc) in units.items():
        if record.get("representation") != "engine-gap":
            continue
        n_checked += 1
        eg = record.get("engine_gap")
        if not isinstance(eg, dict):
            lint.fail(f"[4:engine-gap] {uid} ({loc}): representation is engine-gap but `engine_gap` is not an object")
            continue
        keys = set(eg.keys())
        missing = ENGINE_GAP_KEYS - keys
        if missing:
            lint.fail(f"[4:engine-gap] {uid} ({loc}): engine_gap missing canonical field(s): {sorted(missing)}")
        for k in ENGINE_GAP_KEYS & keys:
            v = eg[k]
            if v is None or (isinstance(v, (str, list, dict)) and len(v) == 0):
                lint.fail(f"[4:engine-gap] {uid} ({loc}): engine_gap.{k} is empty")
    return n_checked


# ---------------------------------------------------------------------------
# Check 5: package structure
# ---------------------------------------------------------------------------

FRONTMATTER_RE = re.compile(r"^---\n(.*?\n)---\n", re.DOTALL)
STAGE_DIR_RE = re.compile(r"^\d")


def parse_front_matter(text):
    """Minimal YAML-front-matter-ish parser sufficient for this corpus's
    index.md shape: flat `key: value` lines, `key: >-` folded scalars, and
    `key:` + `- item` list blocks. Returns dict[str, object] or raises
    ValueError."""
    m = FRONTMATTER_RE.match(text)
    if not m:
        raise ValueError("no front matter block found (must open with `---` and close with `---`)")
    body = m.group(1)
    data = {}
    lines = body.split("\n")
    i = 0
    while i < len(lines):
        line = lines[i]
        if not line.strip():
            i += 1
            continue
        km = re.match(r"^([A-Za-z_][A-Za-z0-9_]*):\s*(.*)$", line)
        if not km:
            raise ValueError(f"unparseable front-matter line: {line!r}")
        key, rest = km.group(1), km.group(2)
        if rest == ">-" or rest == "|-" or rest == ">" or rest == "|":
            # folded/literal block scalar: consume indented continuation lines
            i += 1
            block = []
            while i < len(lines) and (lines[i].startswith("  ") or not lines[i].strip()):
                if lines[i].strip():
                    block.append(lines[i].strip())
                i += 1
            data[key] = " ".join(block)
            continue
        if rest == "":
            # possible list block
            i += 1
            items = []
            while i < len(lines) and lines[i].strip().startswith("- "):
                items.append(lines[i].strip()[2:].strip())
                i += 1
            data[key] = items
            continue
        # scalar
        data[key] = rest.strip()
        i += 1
    return data


def check_index_front_matter(lint, pkg_name, index_path):
    with open(index_path, encoding="utf-8") as f:
        text = f.read()
    try:
        fm = parse_front_matter(text)
    except ValueError as e:
        lint.fail(f"[5:index-front-matter] {pkg_name}: {index_path}: {e}")
        return

    for field in ("kind", "name", "status", "version", "description"):
        if field not in fm or fm[field] in (None, ""):
            lint.fail(f"[5:index-front-matter] {pkg_name}: index.md front matter missing required field `{field}`")

    if fm.get("kind") != "workflow":
        lint.fail(f"[5:index-front-matter] {pkg_name}: index.md `kind` is `{fm.get('kind')}`, expected `workflow`")

    if fm.get("name") and fm["name"] != pkg_name:
        lint.fail(f"[5:index-front-matter] {pkg_name}: index.md `name: {fm['name']}` does not match directory name `{pkg_name}`")

    if fm.get("status") not in ("draft", "published"):
        lint.fail(f"[5:index-front-matter] {pkg_name}: index.md `status: {fm.get('status')}` must be draft or published")

    # All packages here live under draft-workflows/, which per convention.md
    # §2.3 means status MUST be draft.
    if fm.get("status") == "published":
        lint.fail(f"[5:index-front-matter] {pkg_name}: index.md `status: published` illegal under draft-workflows/ (convention.md §2.3)")


def check_inputs_table_paths(lint, pkg_dir, pkg_name, stage_name):
    context_path = os.path.join(pkg_dir, stage_name, "CONTEXT.md")
    if not os.path.isfile(context_path):
        lint.fail(f"[5:stage-context-missing] {pkg_name}/{stage_name}: no CONTEXT.md found")
        return

    with open(context_path, encoding="utf-8") as f:
        text = f.read()

    m = re.search(r"## Inputs\n\n(.*?)\n\n", text, re.DOTALL)
    if not m:
        lint.fail(f"[5:inputs-table-missing] {pkg_name}/{stage_name}: CONTEXT.md has no `## Inputs` table")
        return

    table = m.group(1)
    rows = [r for r in table.split("\n") if r.strip().startswith("|") and "---" not in r]
    if not rows:
        lint.fail(f"[5:inputs-table-empty] {pkg_name}/{stage_name}: `## Inputs` table has no rows")
        return

    header_row = rows[0]
    for row in rows[1:]:
        cols = [c.strip() for c in row.strip().strip("|").split("|")]
        if not cols:
            continue
        file_col = cols[0]
        if not file_col or file_col == "File":
            continue
        if file_col in ("—", "-", "--", "n/a", "N/A"):
            # explicit "no contract-bearing upstream dependency" placeholder row
            continue
        resolved = os.path.normpath(os.path.join(pkg_dir, stage_name, file_col))
        common_root = os.path.normpath(os.path.join(ROOT, "..", ".sergeant", "common"))
        workflow_root = os.path.normpath(pkg_dir)
        under_workflow = os.path.commonpath([resolved, workflow_root]) == workflow_root if os.path.isabs(resolved) == os.path.isabs(workflow_root) else False
        if not os.path.isfile(resolved):
            lint.fail(f"[5:inputs-path] {pkg_name}/{stage_name}: Inputs row path `{file_col}` does not resolve to an existing file ({resolved})")
            continue
        if not under_workflow:
            lint.fail(f"[5:inputs-path-scope] {pkg_name}/{stage_name}: Inputs row path `{file_col}` resolves outside the workflow's own directory")


def check_package_structure(lint, all_unit_ids):
    if not os.path.isdir(DRAFT_WORKFLOWS_DIR):
        lint.fail(f"[5:missing] draft-workflows/ directory not found")
        return

    packages = sorted(
        d for d in os.listdir(DRAFT_WORKFLOWS_DIR)
        if os.path.isdir(os.path.join(DRAFT_WORKFLOWS_DIR, d))
    )

    for pkg_name in packages:
        pkg_dir = os.path.join(DRAFT_WORKFLOWS_DIR, pkg_name)

        toml_path = os.path.join(pkg_dir, "workflow.toml")
        index_path = os.path.join(pkg_dir, "index.md")
        provenance_path = os.path.join(pkg_dir, "provenance.md")
        context_path = os.path.join(pkg_dir, "CONTEXT.md")

        for required, label in (
            (toml_path, "workflow.toml"),
            (index_path, "index.md"),
            (provenance_path, "provenance.md"),
            (context_path, "CONTEXT.md"),
        ):
            if not os.path.isfile(required):
                lint.fail(f"[5:package-missing-file] {pkg_name}: missing `{label}`")

        # workflow.toml order == stage-directory lexical order
        stage_dirs = sorted(
            d for d in os.listdir(pkg_dir)
            if os.path.isdir(os.path.join(pkg_dir, d)) and STAGE_DIR_RE.match(d)
        )
        if os.path.isfile(toml_path):
            try:
                with open(toml_path, "rb") as f:
                    toml_data = tomllib.load(f)
                stages = toml_data.get("workflow", {}).get("stages")
                if not isinstance(stages, list):
                    lint.fail(f"[5:workflow-toml] {pkg_name}: workflow.toml has no `workflow.stages` array")
                else:
                    if stages != stage_dirs:
                        lint.fail(
                            f"[5:stage-order] {pkg_name}: workflow.toml stages {stages} != "
                            f"stage-directory lexical order {stage_dirs}"
                        )
            except (tomllib.TOMLDecodeError, OSError) as e:
                lint.fail(f"[5:workflow-toml] {pkg_name}: cannot parse workflow.toml: {e}")

        # every stage directory is an actor stage: CONTEXT.md + Inputs table
        for stage_name in stage_dirs:
            check_inputs_table_paths(lint, pkg_dir, pkg_name, stage_name)

        # index.md front matter
        if os.path.isfile(index_path):
            check_index_front_matter(lint, pkg_name, index_path)

        # provenance.md cites only existing unit ids
        if os.path.isfile(provenance_path):
            with open(provenance_path, encoding="utf-8") as f:
                ptext = f.read()
            cited = set(BU_ID_RE.findall(ptext))
            unknown = sorted(cited - all_unit_ids)
            for uid in unknown:
                lint.fail(f"[5:provenance-unknown-id] {pkg_name}/provenance.md: cites unknown unit id `{uid}`")

    # no package outside draft-workflows/
    for dirpath, dirnames, filenames in os.walk(ROOT):
        if os.path.commonpath([os.path.abspath(dirpath), DRAFT_WORKFLOWS_DIR]) == DRAFT_WORKFLOWS_DIR:
            continue
        if "scratch" + os.sep in dirpath + os.sep:
            continue
        if "workflow.toml" in filenames:
            lint.fail(f"[5:package-outside-draft-workflows] {dirpath}/workflow.toml is a workflow package outside draft-workflows/")


# ---------------------------------------------------------------------------
# Check 6: source-inventory decompose coverage in provenance-map.md
# ---------------------------------------------------------------------------

DISPOSITIONS = {"decompose", "helper-evidence", "obsolete-candidate", "reference-only"}
BACKTICK_RE = re.compile(r"`([^`]+)`")


def parse_source_inventory_rows(text):
    """Per-line, per-cell table parser. A single whole-text regex is too
    brittle here: the first cell isn't always exactly `` `path` `` — some
    rows append trailing annotations after the closing backtick (e.g.
    `` `bin/sgt-callback` (Python) ``) — so this splits each row on `|` and
    only requires that cell 0 contain a backtick-quoted path and cell 2 be a
    known disposition, wherever the rest of the row does."""
    rows = []
    for line in text.splitlines():
        if not line.startswith("|"):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) < 3:
            continue
        m = BACKTICK_RE.search(cells[0])
        if not m:
            continue
        disposition = cells[2].strip("* ")
        if disposition not in DISPOSITIONS:
            continue
        rows.append((m.group(1), disposition))
    return rows


def check_decompose_coverage(lint):
    if not os.path.isfile(SOURCE_INVENTORY):
        lint.fail(f"[6:missing] source-inventory.md not found")
        return
    if not os.path.isfile(PROVENANCE_MAP):
        lint.fail(f"[6:missing] provenance-map.md not found")
        return

    with open(SOURCE_INVENTORY, encoding="utf-8") as f:
        inv_text = f.read()
    with open(PROVENANCE_MAP, encoding="utf-8") as f:
        map_text = f.read()

    rows = parse_source_inventory_rows(inv_text)
    decompose_files = sorted({p for p, disp in rows if disp == "decompose"})

    n = 0
    for path in decompose_files:
        n += 1
        # provenance-map.md's per-file table keys rows by the same
        # repo-relative (to reference/sergeant-upstream) backtick path.
        needle = f"`{path}`"
        if needle not in map_text:
            lint.fail(f"[6:decompose-not-mapped] source-inventory.md `decompose` file `{path}` does not appear in provenance-map.md")
    return n


# ---------------------------------------------------------------------------
# Check 7: rationale non-null; alternatives_considered non-empty where required
# ---------------------------------------------------------------------------

def extract_conflict_named_ids(lint):
    if not os.path.isfile(CLASSIFICATION_LEDGER):
        lint.fail(f"[7:missing] classification-ledger.md not found")
        return set()
    with open(CLASSIFICATION_LEDGER, encoding="utf-8") as f:
        text = f.read()
    m = re.search(r"^## 3\. Cross-partition conflicts\n(.*?)(?=^## \d)", text, re.DOTALL | re.MULTILINE)
    if not m:
        lint.fail("[7:ledger-structure] classification-ledger.md: could not locate `## 3. Cross-partition conflicts` section")
        return set()
    return set(BU_ID_RE.findall(m.group(1)))


def check_rationale_and_alternatives(lint, units, conflict_named_ids):
    boundary_reps = {"workflow", "stage", "engine-gap"}
    for uid, (record, loc) in units.items():
        rationale = record.get("rationale")
        if rationale is None or (isinstance(rationale, str) and not rationale.strip()):
            lint.fail(f"[7:rationale-null] {uid} ({loc}): rationale is null/empty")

        alts = record.get("alternatives_considered")
        if not isinstance(alts, list):
            continue  # already flagged under check 1
        needs_alts = record.get("representation") in boundary_reps or uid in conflict_named_ids
        if needs_alts and len(alts) == 0:
            reason = record.get("representation") if record.get("representation") in boundary_reps else "conflict-named (classification-ledger.md §3)"
            lint.fail(f"[7:alternatives-empty] {uid} ({loc}): alternatives_considered is empty but unit is {reason}")


# ---------------------------------------------------------------------------


def main():
    lint = Lint()

    units, disputed = load_units(lint)
    all_unit_ids = set(units.keys())

    disputed_ids = {uid for uid, (r, _) in units.items() if is_disputed(r)}
    n_verified = verify_quotes(lint, units, disputed_ids)

    n_engine_gap = check_engine_gap_template(lint, units)

    check_package_structure(lint, all_unit_ids)

    n_decompose = check_decompose_coverage(lint)

    conflict_named_ids = extract_conflict_named_ids(lint)
    check_rationale_and_alternatives(lint, units, conflict_named_ids)

    print(f"units loaded: {len(units)}")
    print(f"quotes verified: {n_verified}  disputed (exempt): {len(disputed_ids)}")
    print(f"engine-gap units checked: {n_engine_gap}")
    print(f"decompose files checked: {n_decompose}")
    print(f"conflict-named units (classification-ledger.md §3): {len(conflict_named_ids)}")
    print()

    if lint.errors:
        print(f"FAIL: {len(lint.errors)} defect(s) found\n")
        for e in lint.errors:
            print(" -", e)
        return 1

    print("PASS: corpus is lint-clean")
    return 0


if __name__ == "__main__":
    sys.exit(main())
