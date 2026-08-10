#!/usr/bin/env python3
"""validate-structure.py — the §9.7 structural validator.

Adapted from `reference-corpus/lint.py`'s package-structure check (its
check 5) into a workflow-owned helper this workflow can run on itself and
on what it generates (`docs/gauntlet/contracts/N2.md` Outcome §1: "It must
itself pass reference-corpus-grade lint (the validator runs on its own
tree)"). This script does not read anything under `reference-corpus/` —
doing so from inside a workflow run would violate this workflow's own
blindness rule (`../CONTEXT.md`) — it only re-derives the same *structural*
checks reference-corpus/lint.py applies to draft-workflows/ packages, not
lint.py's deeper NDJSON quote/hash reproduction (record-shapes.md §3's
citation discipline is `../_config/evidence-policy.md`'s job to state and
`70-lint`'s actor's job to apply by hand or a future helper — reproducing
966-unit-corpus-scale hash verification is out of scope for a per-package
structural pass).

TWO MODES
---------
    python3 validate-structure.py
        Validates THIS workflow's own tree (the parent of `scripts/`) —
        an ADMITTED package: index.md status must be `published`, and the
        tree must live under `.sergeant/workflows/`, never
        `.sergeant/drafts/`.

    python3 validate-structure.py PATH
        Validates the workflow package at PATH as a GENERATED DRAFT — used
        by `70-lint` against whatever `60-draft` just materialized under
        `.sergeant/drafts/workflows/<candidate>/`. index.md status must be
        `draft`, the tree must live under `.sergeant/drafts/workflows/`,
        and (draft-only) every workflow/stage must carry provenance.

Independent of which single tree is named above, this script also runs one
repository-wide check — "no draft is accidentally placed in
`.sergeant/workflows/`, and no admitted package sits under
`.sergeant/drafts/`" — since that check is inherently about the relationship
between the two trees, not about either one alone.

CHECKS (§9.7, "The initial validator ... should check")
---------------------------------------------------------
  S1  valid front matter (index.md, required fields, status/location agree)
  S2  unique stage IDs (no duplicate workflow.toml stage entries)
  S3  declared stage order matches directories (lexical order agreement)
  S4  every declared (actor) stage has a CONTEXT.md with a resolving,
      correctly-layered Inputs table
  S5  referenced shared contexts/helpers (`@@name` tokens) resolve inside
      `.sergeant/`
  S6  no path traversal (no `..` in any Inputs-table path or `@@` token)
  S7  no draft accidentally placed in `.sergeant/workflows/` (repo-wide)
  S8  every workflow/stage has provenance (draft mode only: provenance.md
      exists, is non-empty, and cites at least one behavior-unit id)
  S9  no engine-gap record omits a failed-lower-rung field (scans any
      NDJSON under the tree for `representation: engine-gap` records)
  S10 no generated script is executable solely because a Markdown
      instruction forgot to classify it (every executable file under the
      tree is referenced by name from some CONTEXT.md or _config file in
      the same tree)
  S11 a stage `output/README.md` that declares an expected artifact also
      declares a disposition for it (docs/icm/convention.md §1a, D9:
      "every declared output carries a disposition")
  S12 if any stage declares an output artifact, the workflow's closing
      stage's own CONTEXT.md names a finalize step (docs/icm/convention.md
      §1a, D9: "a workflow that declares any output ends with a
      deterministic finalize step")
  S13 `version` is a bare integer (never a quoted string) in both
      `index.md` front matter and `workflow.toml`, and the two agree
      (record-shapes.md §1: "a monotonically increasing integer")
  S14 (admitted mode only) this package's name is listed in the root
      catalog, `.sergeant/index.md` (convention.md §1 rule 1 /
      record-shapes.md §1 rule 2)
  S15 (admitted mode only) `scripts/finalize.py`'s evidence-preservation
      guard (GP-5b) actually holds: `scripts/test-finalize-evidence-guard.py`
      is run against a disposable git sandbox and must exit 0. Only
      meaningful for this workflow's own tree, since drafts never carry a
      `finalize.py` of their own.

Exit 0 iff clean. Stdlib only, except S15, which shells out to `git` inside
a temp directory (never this checkout) via
`scripts/test-finalize-evidence-guard.py`.
"""

import os
import re
import subprocess
import sys
import tomllib

STAGE_DIR_RE = re.compile(r"^\d")
AT_NAME_RE = re.compile(r"@@([A-Za-z0-9_-]+)")
BU_ID_RE = re.compile(r"\bBU-[A-Za-z0-9_]*-?\d+\b|\bBU-\d+\b")
FRONTMATTER_RE = re.compile(r"^---\n(.*?\n)---\n", re.DOTALL)


class Lint:
    def __init__(self):
        self.errors = []

    def fail(self, code, msg):
        self.errors.append(f"[{code}] {msg}")


def find_repo_root(start):
    """Walk upward from `start` until a directory containing `.sergeant/`
    is found (that directory IS the repo root). Falls back to `start`
    itself if none is found within 8 levels."""
    cur = os.path.abspath(start)
    for _ in range(8):
        if os.path.isdir(os.path.join(cur, ".sergeant")):
            return cur
        parent = os.path.dirname(cur)
        if parent == cur:
            break
        cur = parent
    return os.path.abspath(start)


def parse_front_matter(text):
    m = FRONTMATTER_RE.match(text)
    if not m:
        raise ValueError("no front matter block found (must open and close with `---`)")
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
        if rest in (">-", "|-", ">", "|"):
            i += 1
            block = []
            while i < len(lines) and (lines[i].startswith("  ") or not lines[i].strip()):
                if lines[i].strip():
                    block.append(lines[i].strip())
                i += 1
            data[key] = " ".join(block)
            continue
        if rest == "":
            i += 1
            items = []
            while i < len(lines) and lines[i].strip().startswith("- "):
                items.append(lines[i].strip()[2:].strip())
                i += 1
            data[key] = items
            continue
        data[key] = rest.strip()
        i += 1
    return data


def check_front_matter(lint, pkg_dir, pkg_name, expect_status):
    index_path = os.path.join(pkg_dir, "index.md")
    if not os.path.isfile(index_path):
        lint.fail("S1", f"{pkg_name}: missing index.md")
        return None
    with open(index_path, encoding="utf-8") as f:
        text = f.read()
    try:
        fm = parse_front_matter(text)
    except ValueError as e:
        lint.fail("S1", f"{pkg_name}: index.md: {e}")
        return None

    for field in ("kind", "name", "status", "version", "description"):
        if field not in fm or fm[field] in (None, ""):
            lint.fail("S1", f"{pkg_name}: index.md missing required field `{field}`")

    if fm.get("kind") != "workflow":
        lint.fail("S1", f"{pkg_name}: index.md `kind` is `{fm.get('kind')}`, expected `workflow`")

    if fm.get("name") and fm["name"] != pkg_name:
        lint.fail("S1", f"{pkg_name}: index.md `name: {fm['name']}` does not match directory name `{pkg_name}`")

    desc = fm.get("description", "")
    if desc and desc.strip().lower().replace("-", " ") in (
        pkg_name.replace("-", " "),
        f"{pkg_name.replace('-', ' ')}s".strip(),
    ):
        lint.fail("S1", f"{pkg_name}: index.md `description` only restates `name` (record-shapes.md §1)")

    status = fm.get("status")
    if status not in ("draft", "published"):
        lint.fail("S1", f"{pkg_name}: index.md `status: {status}` must be `draft` or `published`")
    elif status != expect_status:
        lint.fail("S1", f"{pkg_name}: index.md `status: {status}` but this tree is being validated as `{expect_status}`")

    return fm


def check_workflow_toml(lint, pkg_dir, pkg_name):
    """Returns (declared stage list, raw `[workflow]` toml section), or
    (None, None) on hard parse failure. The raw section is returned (not
    just `stages`) so callers can also inspect `version`'s TOML type
    (S13) without re-parsing the file."""
    toml_path = os.path.join(pkg_dir, "workflow.toml")
    if not os.path.isfile(toml_path):
        lint.fail("S3", f"{pkg_name}: missing workflow.toml")
        return None, None
    try:
        with open(toml_path, "rb") as f:
            data = tomllib.load(f)
    except tomllib.TOMLDecodeError as e:
        lint.fail("S3", f"{pkg_name}: workflow.toml: {e}")
        return None, None

    section = data.get("workflow", {})
    stages = section.get("stages")
    if not isinstance(stages, list) or not stages:
        lint.fail("S3", f"{pkg_name}: workflow.toml has no non-empty `workflow.stages` array")
        return None, section

    if section.get("name") != pkg_name:
        lint.fail("S1", f"{pkg_name}: workflow.toml `name: {section.get('name')}` does not match directory name")

    seen = set()
    dups = set()
    for s in stages:
        if s in seen:
            dups.add(s)
        seen.add(s)
    if dups:
        lint.fail("S2", f"{pkg_name}: workflow.toml declares duplicate stage id(s): {sorted(dups)}")

    stage_dirs = sorted(
        d for d in os.listdir(pkg_dir)
        if STAGE_DIR_RE.match(d) and os.path.isdir(os.path.join(pkg_dir, d))
    )
    if stages != stage_dirs:
        lint.fail(
            "S3",
            f"{pkg_name}: workflow.toml stages {stages} != stage-directory lexical order {stage_dirs}",
        )

    return stages, section


def check_version_types(lint, pkg_name, fm, toml_section):
    """S13: `version` MUST carry a monotonically increasing integer
    *value* (record-shapes.md §1) in both `index.md` front matter and
    `workflow.toml` — but the two files do not use the same TOML/YAML
    representation of that value, and this check deliberately does not
    force them to.

    `src/domain/workflow.rs`'s `WorkflowSection.version` is a Rust `String`
    (`#[serde(deny_unknown_fields)]`, no custom deserializer) — a bare TOML
    integer in `workflow.toml` fails to load with a hard parse error
    (measured: `invalid type: integer `1`, expected a string`), not a lint
    warning. So `workflow.toml`'s `version` MUST be a quoted TOML string
    whose *content* is nonetheless digits-only (record-shapes.md's integer
    *value* requirement, satisfied through the type the engine actually
    accepts) — never `"v1"`, `"1.0"`, or similar. `index.md`'s `version` is
    never read by the engine at all (`docs/icm/convention.md`: "the current
    loader ... reads only a declared `workflow.toml` and its stage
    `CONTEXT.md` files"), so record-shapes.md's rule applies there at face
    value: a bare integer, not a quoted string. Both forms' *numeric value*
    must agree."""
    if fm is None or toml_section is None:
        return

    idx_raw = fm.get("version")
    toml_version = toml_section.get("version")

    idx_ok = idx_raw is not None and re.fullmatch(r"\d+", str(idx_raw))
    if idx_raw is not None and not idx_ok:
        lint.fail("S13", f"{pkg_name}: index.md `version: {idx_raw!r}` is not a bare integer (record-shapes.md §1)")

    toml_ok = isinstance(toml_version, str) and re.fullmatch(r"\d+", toml_version)
    if toml_version is not None and not toml_ok:
        if isinstance(toml_version, int):
            lint.fail("S13", f"{pkg_name}: workflow.toml `version = {toml_version}` is a TOML integer, which src/domain/workflow.rs's loader rejects (measured) — it must be a quoted, digits-only string (e.g. `version = \"{toml_version}\"`)")
        else:
            lint.fail("S13", f"{pkg_name}: workflow.toml `version = {toml_version!r}` must be a digits-only string (record-shapes.md §1's integer value, in the string type the engine's loader requires)")

    if idx_ok and toml_ok and int(idx_raw) != int(toml_version):
        lint.fail("S13", f"{pkg_name}: index.md `version: {idx_raw}` disagrees with workflow.toml `version = \"{toml_version}\"`")


def check_inputs_table(lint, pkg_dir, pkg_name, stage_id, all_stages_in_order, repo_root):
    stage_dir = os.path.join(pkg_dir, stage_id)
    context_path = os.path.join(stage_dir, "CONTEXT.md")
    if not os.path.isfile(context_path):
        lint.fail("S4", f"{pkg_name}/{stage_id}: no CONTEXT.md — every actor stage must have one")
        return

    with open(context_path, encoding="utf-8") as f:
        text = f.read()

    m = re.search(r"## Inputs\n\n(.*?)\n\n", text, re.DOTALL)
    if not m:
        lint.fail("S4", f"{pkg_name}/{stage_id}: CONTEXT.md has no `## Inputs` table")
        return
    rows = [r for r in m.group(1).split("\n") if r.strip().startswith("|") and "---" not in r]
    if not rows or rows[0].strip().lower().startswith("| file"):
        rows = rows[1:] if rows else rows

    stage_index = all_stages_in_order.index(stage_id)

    for row in rows:
        cols = [c.strip() for c in row.strip().strip("|").split("|")]
        if not cols or not cols[0] or cols[0].lower() == "file":
            continue
        file_col, layer = cols[0], (cols[1].strip() if len(cols) > 1 else "")

        if file_col in ("—", "-", "--", "n/a", "N/A"):
            continue

        resolved = os.path.normpath(os.path.join(stage_dir, file_col))

        # S6: no path traversal. `..` is normal and expected here — every
        # legitimate Inputs row climbs out of the stage directory to reach
        # a sibling (`../CONTEXT.md`, `../00-x/output/y.md`, `../_config/z.md`,
        # record-shapes.md §1a's own worked example does this). What must
        # never happen is the RESOLVED path landing outside this workflow's
        # own directory or `.sergeant/common/` — an absolute escape, not the
        # `..` token itself.
        common_root = os.path.normpath(os.path.join(repo_root, ".sergeant", "common"))
        workflow_root = os.path.normpath(pkg_dir)
        under_workflow = os.path.commonpath([resolved, workflow_root]) == workflow_root
        under_common = os.path.isdir(common_root) and os.path.commonpath([resolved, common_root]) == common_root
        if not (under_workflow or under_common):
            lint.fail("S6", f"{pkg_name}/{stage_id}: Inputs row `{file_col}` resolves outside this workflow's own directory and outside `.sergeant/common/` (path traversal)")
            continue

        if layer not in ("L1", "L3", "L4"):
            lint.fail("S4", f"{pkg_name}/{stage_id}: Inputs row `{file_col}` has layer `{layer or '(blank)'}`, must be L1, L3, or L4 (record-shapes.md §1a rule 2)")

        if layer == "L1" and stage_index != 0:
            lint.fail("S4", f"{pkg_name}/{stage_id}: Inputs row `{file_col}` cites L1 orientation outside the first stage (convention.md §1a rule 5)")

        if layer == "L4":
            # An L4 row names a PER-RUN artifact (docs/icm/convention.md §1a
            # rule 4): the authored tree's output/ directories hold only a
            # README.md declaring the expected shape, never the artifact
            # itself — so, unlike L1/L3, a named artifact file (as opposed
            # to `output/README.md` itself) is legitimately absent here and
            # is NOT a defect. What must still hold: the row resolves under
            # some EARLIER stage's own output/ directory, and that stage
            # actually declares its output shape (its output/README.md
            # exists) — so there is something concrete for this row to mean.
            rel = os.path.relpath(resolved, pkg_dir)
            parts = rel.split(os.sep)
            if len(parts) < 2 or parts[1] != "output":
                lint.fail("S4", f"{pkg_name}/{stage_id}: Inputs row `{file_col}` marked L4 but does not resolve under a stage's `output/`")
                continue
            source_stage = parts[0]
            if source_stage not in all_stages_in_order:
                lint.fail("S4", f"{pkg_name}/{stage_id}: Inputs row `{file_col}` resolves under unknown stage `{source_stage}`")
                continue
            if all_stages_in_order.index(source_stage) >= stage_index:
                lint.fail("S4", f"{pkg_name}/{stage_id}: Inputs row `{file_col}` is an L4 input from `{source_stage}`, not earlier than `{stage_id}` (record-shapes.md §1a rule 3)")
                continue
            source_readme = os.path.join(pkg_dir, source_stage, "output", "README.md")
            if not os.path.isfile(resolved) and not os.path.isfile(source_readme):
                lint.fail("S4", f"{pkg_name}/{stage_id}: Inputs row `{file_col}` names an artifact that doesn't exist yet (expected, per-run) but `{source_stage}` doesn't even declare its output shape (missing output/README.md)")
            continue

        # L1 / L3: stable, authored-at-write-time material — must actually
        # be present in the tree right now.
        if not os.path.isfile(resolved):
            lint.fail("S4", f"{pkg_name}/{stage_id}: Inputs row path `{file_col}` does not resolve to an existing file ({resolved})")
            continue


def check_provenance(lint, pkg_dir, pkg_name, stages):
    provenance_path = os.path.join(pkg_dir, "provenance.md")
    if not os.path.isfile(provenance_path):
        lint.fail("S8", f"{pkg_name}: draft package has no provenance.md (§9.6: 'every workflow and stage should be mapped to the behavior units that justify it')")
        return
    with open(provenance_path, encoding="utf-8") as f:
        text = f.read()
    if not text.strip():
        lint.fail("S8", f"{pkg_name}: provenance.md is empty")
        return
    cited = set(BU_ID_RE.findall(text))
    if not cited:
        lint.fail("S8", f"{pkg_name}: provenance.md cites no behavior-unit id (or is explicitly marked as a justified design inference — §9.6)")
    for stage_id in stages:
        if stage_id not in text:
            lint.fail("S8", f"{pkg_name}: provenance.md never mentions stage `{stage_id}`")


def check_at_references(lint, pkg_dir, pkg_name, repo_root):
    """S5: every `@@name` token anywhere under this package's authored
    Markdown — not just a stage's own CONTEXT.md — MUST resolve to
    `.sergeant/common/contexts/<name>.md`. `output/` is skipped: it holds
    per-run artifacts (Layer 4), and a generated artifact that happens to
    quote a literal `@@something` from the SUBJECT repository's own docs
    must never be mistaken for this workflow's own shared-context syntax.

    The literal token `@@name` (case-insensitive) is convention.md §4's own
    documentation placeholder for "a name goes here" — this file, and
    `references/draft-package-template.md`, both use it that way to
    *describe* the syntax without invoking it. Treating that mention as a
    real reference needing resolution would demand a shared context
    literally named "name", which no author would ever mean; it is
    excluded here so authors can write the token in its own defining prose
    without periphrasis.
    """
    for dirpath, dirnames, filenames in os.walk(pkg_dir):
        dirnames[:] = [d for d in dirnames if d != "output"]
        for fn in filenames:
            if not fn.endswith(".md"):
                continue
            path = os.path.join(dirpath, fn)
            rel = os.path.relpath(path, pkg_dir)
            try:
                with open(path, encoding="utf-8") as f:
                    text = f.read()
            except OSError:
                continue
            for at_name in AT_NAME_RE.findall(text):
                if at_name.lower() == "name":
                    continue
                resolved = os.path.join(repo_root, ".sergeant", "common", "contexts", f"{at_name}.md")
                if not os.path.isfile(resolved):
                    lint.fail("S5", f"{pkg_name}: {rel}: `@@{at_name}` does not resolve to {os.path.relpath(resolved, repo_root)}")


ARTIFACT_DECLARED_RE = re.compile(r"\*\*Expected artifacts?:\*\*")
DISPOSITION_TABLE_HEADER_RE = re.compile(r"^\|\s*File\s*\|\s*Disposition\s*\|", re.MULTILINE)
DISPOSITION_LINE_RE = re.compile(r"\*\*Disposition:\*\*\s*`[a-zA-Z_-]+`")


def check_dispositions(lint, pkg_dir, pkg_name, stages):
    """S11: an `output/README.md` that declares an expected artifact MUST
    also declare a disposition for it — either the single-artifact
    `**Disposition:**` line or a `| File | Disposition |` table row per
    artifact (docs/icm/convention.md §1a, D9 working rule: "every declared
    output carries a disposition"). A README with neither is exactly the
    ambiguity `scripts/finalize.py` itself refuses to guess at — this
    check surfaces it at lint time instead of at finalize time."""
    for stage_id in stages:
        readme_path = os.path.join(pkg_dir, stage_id, "output", "README.md")
        if not os.path.isfile(readme_path):
            continue
        with open(readme_path, encoding="utf-8") as f:
            text = f.read()
        if not ARTIFACT_DECLARED_RE.search(text):
            continue
        if not (DISPOSITION_TABLE_HEADER_RE.search(text) or DISPOSITION_LINE_RE.search(text)):
            lint.fail("S11", f"{pkg_name}/{stage_id}: output/README.md declares an expected artifact but no `**Disposition:**` line or Disposition table (docs/icm/convention.md §1a, D9)")


def check_finalize_step(lint, pkg_dir, pkg_name, stages):
    """S12: if any stage declares an output artifact, the workflow's
    closing (last-in-order) stage's own CONTEXT.md MUST name a finalize
    step (docs/icm/convention.md §1a, D9: "a workflow that declares any
    output ends with a deterministic finalize step"). This only checks
    that a finalize step is *named*, not that it runs correctly — the
    actor's own judgment governs the helper's result (convention.md §5)."""
    if not stages:
        return
    any_output = False
    for stage_id in stages:
        readme_path = os.path.join(pkg_dir, stage_id, "output", "README.md")
        if os.path.isfile(readme_path):
            with open(readme_path, encoding="utf-8") as f:
                if ARTIFACT_DECLARED_RE.search(f.read()):
                    any_output = True
                    break
    if not any_output:
        return
    last_stage = stages[-1]
    last_context = os.path.join(pkg_dir, last_stage, "CONTEXT.md")
    if not os.path.isfile(last_context):
        return  # already reported by S4
    with open(last_context, encoding="utf-8") as f:
        text = f.read()
    if "finalize" not in text.lower():
        lint.fail("S12", f"{pkg_name}: outputs are declared but the closing stage `{last_stage}` names no finalize step (docs/icm/convention.md §1a, D9)")


def check_root_catalog(lint, pkg_name, repo_root, mode):
    """S14 (admitted mode only): a published workflow absent from the root
    catalog is a violation (convention.md §1 rule 1 / record-shapes.md §1
    rule 2). A draft candidate is correctly NOT catalogued yet, so this is
    skipped in draft mode."""
    if mode != "admitted":
        return
    catalog_path = os.path.join(repo_root, ".sergeant", "index.md")
    if not os.path.isfile(catalog_path):
        lint.fail("S14", f"{pkg_name}: .sergeant/index.md (root catalog) does not exist")
        return
    with open(catalog_path, encoding="utf-8") as f:
        text = f.read()
    if pkg_name not in text:
        lint.fail("S14", f"{pkg_name}: not listed in .sergeant/index.md (root catalog)")


def check_finalize_evidence_guard(lint, pkg_dir, pkg_name, mode):
    """S15 (admitted mode only): `scripts/finalize.py`'s evidence-
    preservation guard (GP-5b) is proven, not assumed — run
    `scripts/test-finalize-evidence-guard.py` (a disposable git-sandbox
    test, touches nothing under `pkg_dir`) and fail lint if it does not
    exit 0. Skipped for draft candidates: a generated package never carries
    its own `finalize.py`, so there is nothing this check could exercise."""
    if mode != "admitted":
        return
    test_script = os.path.join(pkg_dir, "scripts", "test-finalize-evidence-guard.py")
    if not os.path.isfile(test_script):
        lint.fail("S15", f"{pkg_name}: scripts/test-finalize-evidence-guard.py not found — the evidence-preservation guard (GP-5b) is unverified")
        return
    try:
        r = subprocess.run(
            [sys.executable, test_script],
            capture_output=True, text=True, timeout=60,
        )
    except (OSError, subprocess.TimeoutExpired) as e:
        lint.fail("S15", f"{pkg_name}: could not run scripts/test-finalize-evidence-guard.py: {e}")
        return
    if r.returncode != 0:
        detail = (r.stdout + r.stderr).strip().splitlines()
        tail = " | ".join(detail[-3:]) if detail else "(no output)"
        lint.fail("S15", f"{pkg_name}: scripts/test-finalize-evidence-guard.py failed (exit {r.returncode}): {tail}")


def check_engine_gap_records(lint, pkg_dir, pkg_name):
    required = {
        "behavior", "source_evidence", "lower_rungs_attempted",
        "why_each_fails", "minimum_runtime_capability_required",
        "observable_acceptance_test",
    }
    import json
    n = 0
    for dirpath, _dirnames, filenames in os.walk(pkg_dir):
        for fn in filenames:
            if not fn.endswith(".ndjson"):
                continue
            path = os.path.join(dirpath, fn)
            with open(path, encoding="utf-8") as f:
                for lineno, raw in enumerate(f, 1):
                    line = raw.strip()
                    if not line:
                        continue
                    try:
                        record = json.loads(line)
                    except json.JSONDecodeError:
                        continue
                    if record.get("representation") != "engine-gap":
                        continue
                    n += 1
                    loc = f"{os.path.relpath(path, pkg_dir)}:{lineno}"
                    eg = record.get("engine_gap")
                    if not isinstance(eg, dict):
                        lint.fail("S9", f"{pkg_name}: {loc}: representation is engine-gap but `engine_gap` is not an object")
                        continue
                    missing = required - set(eg.keys())
                    if missing:
                        lint.fail("S9", f"{pkg_name}: {loc}: engine_gap missing field(s) {sorted(missing)} (record-shapes.md §5)")
                    for k in required & set(eg.keys()):
                        v = eg[k]
                        if v is None or (isinstance(v, (str, list, dict)) and len(v) == 0):
                            lint.fail("S9", f"{pkg_name}: {loc}: engine_gap.{k} is empty")
    return n


def check_unclassified_executables(lint, pkg_dir, pkg_name):
    md_text = []
    for dirpath, _dirnames, filenames in os.walk(pkg_dir):
        for fn in filenames:
            if fn.endswith(".md"):
                try:
                    with open(os.path.join(dirpath, fn), encoding="utf-8") as f:
                        md_text.append(f.read())
                except OSError:
                    pass
    haystack = "\n".join(md_text)

    for dirpath, _dirnames, filenames in os.walk(pkg_dir):
        for fn in filenames:
            path = os.path.join(dirpath, fn)
            if not os.access(path, os.X_OK):
                continue
            if fn not in haystack and os.path.relpath(path, pkg_dir) not in haystack:
                lint.fail(
                    "S10",
                    f"{pkg_name}: {os.path.relpath(path, pkg_dir)} is executable but is not named by any CONTEXT.md/_config file in this package (unclassified machinery, convention.md §5 rule 1)",
                )


def check_no_misplaced_drafts(lint, repo_root):
    admitted_root = os.path.join(repo_root, ".sergeant", "workflows")
    draft_root = os.path.join(repo_root, ".sergeant", "drafts", "workflows")

    def scan(root, expect_status, label):
        if not os.path.isdir(root):
            return
        for name in sorted(os.listdir(root)):
            pkg_dir = os.path.join(root, name)
            index_path = os.path.join(pkg_dir, "index.md")
            if not os.path.isdir(pkg_dir) or not os.path.isfile(index_path):
                continue
            with open(index_path, encoding="utf-8") as f:
                text = f.read()
            try:
                fm = parse_front_matter(text)
            except ValueError:
                continue
            status = fm.get("status")
            if status and status != expect_status:
                lint.fail("S7", f"{name}: index.md declares `status: {status}` while living under {label} (convention.md §2.3)")

    scan(admitted_root, "published", ".sergeant/workflows/")
    scan(draft_root, "draft", ".sergeant/drafts/workflows/")


def validate(pkg_dir, mode, repo_root, lint):
    pkg_name = os.path.basename(os.path.normpath(pkg_dir))
    expect_status = "published" if mode == "admitted" else "draft"

    if mode == "admitted":
        if "/drafts/" in pkg_dir.replace(os.sep, "/") + "/":
            lint.fail("S7", f"{pkg_name}: validated as admitted but path contains `drafts/`")
    else:
        if "/drafts/workflows/" not in pkg_dir.replace(os.sep, "/") + "/":
            lint.fail("S7", f"{pkg_name}: validated as a draft but is not under `.sergeant/drafts/workflows/`")

    fm = check_front_matter(lint, pkg_dir, pkg_name, expect_status)
    stages, toml_section = check_workflow_toml(lint, pkg_dir, pkg_name)
    check_version_types(lint, pkg_name, fm, toml_section)
    if stages:
        for stage_id in stages:
            check_inputs_table(lint, pkg_dir, pkg_name, stage_id, stages, repo_root)
        if mode == "draft":
            check_provenance(lint, pkg_dir, pkg_name, stages)
        check_dispositions(lint, pkg_dir, pkg_name, stages)
        check_finalize_step(lint, pkg_dir, pkg_name, stages)

    check_at_references(lint, pkg_dir, pkg_name, repo_root)
    check_root_catalog(lint, pkg_name, repo_root, mode)
    check_finalize_evidence_guard(lint, pkg_dir, pkg_name, mode)
    n_gap = check_engine_gap_records(lint, pkg_dir, pkg_name)
    check_unclassified_executables(lint, pkg_dir, pkg_name)
    return n_gap


def main():
    if len(sys.argv) > 2:
        print("usage: validate-structure.py [PATH]", file=sys.stderr)
        return 2

    lint = Lint()

    if len(sys.argv) == 2:
        target = os.path.abspath(sys.argv[1])
        mode = "draft"
    else:
        target = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        mode = "admitted"

    if not os.path.isdir(target):
        print(f"error: {target} is not a directory", file=sys.stderr)
        return 2

    repo_root = find_repo_root(target)

    n_gap = validate(target, mode, repo_root, lint)
    check_no_misplaced_drafts(lint, repo_root)

    print(f"validated: {target}  (mode={mode})")
    print(f"engine-gap records checked: {n_gap}")
    print()

    if lint.errors:
        print(f"FAIL: {len(lint.errors)} defect(s) found\n")
        for e in lint.errors:
            print(" -", e)
        return 1

    print("PASS: structure is clean")
    return 0


if __name__ == "__main__":
    sys.exit(main())
