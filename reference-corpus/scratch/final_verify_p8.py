import json, re, hashlib, os

REPO_ROOT = "/home/user/sergeant-rs"
FILE = "/home/user/sergeant-rs/reference-corpus/behavior-units/P8.ndjson"

def parse_ranges(locator):
    ranges = []
    for m in re.finditer(r'L(\d+)(?:-(\d+))?', locator):
        start = int(m.group(1))
        end = int(m.group(2)) if m.group(2) else start
        ranges.append((start, end))
    return ranges

file_cache = {}
def get_raw(path):
    if path not in file_cache:
        with open(os.path.join(REPO_ROOT, path), 'r', encoding='utf-8') as f:
            file_cache[path] = f.read()
    return file_cache[path]

MULTI_RANGE_PRIMARY = {"BU-P8-032": 1, "BU-P8-075": 0}
# BU-P8-110/111 and BU-P8-077(retired) use exact-substring citation, not
# whole-line ranges; handle by explicit literal search instead of line-range.
EXPLICIT_SUBSTRING = {}

units = []
ids = []
required_top = ["id","statement","source","scope","trigger","outcome","authority",
                 "confidence","notes","representation","workflow","stage",
                 "rationale","alternatives_considered","engine_gap"]
required_source = ["path","locator","quote","quote_hash"]
valid_reps = {"agents-invariant","workflow","stage","stage-context","helper",
              "shared-helper","shared-context","obsolete-mechanism","engine-gap"}

n_ok = n_bad = 0
with open(FILE) as f:
    for lineno, line in enumerate(f, 1):
        line = line.rstrip("\n")
        if not line:
            print(f"line {lineno}: BLANK LINE (invalid ndjson)")
            continue
        d = json.loads(line)  # raises if invalid json
        units.append(d)
        ids.append(d['id'])

        problems = []
        for k in required_top:
            if k not in d:
                problems.append(f"missing top field {k}")
        for k in required_source:
            if k not in d.get('source', {}):
                problems.append(f"missing source field {k}")
        if d.get('representation') not in valid_reps:
            problems.append(f"invalid representation {d.get('representation')!r}")
        if d.get('rationale') in (None, ""):
            problems.append("rationale null/empty")
        if not isinstance(d.get('alternatives_considered'), list):
            problems.append("alternatives_considered not a list")

        src = d['source']
        path = src['path']
        quote = src['quote']
        qhash = src['quote_hash']
        raw = get_raw(path)

        # verify contiguity: quote (or, if truncated, quote+more up to span_bytes) is a real substring
        if 'span_bytes' in src:
            # need to find full span: search raw for a substring starting with `quote`
            # whose byte length matches span_bytes, and whose hash matches
            idxs = [m.start() for m in re.finditer(re.escape(quote), raw)]
            found = False
            for i in idxs:
                # try to extend to span_bytes length in bytes -> approx via utf-8 slicing
                raw_bytes = raw.encode('utf-8')
                # find byte offset of i in utf-8
                byte_off = len(raw[:i].encode('utf-8'))
                span_b = raw_bytes[byte_off:byte_off+src['span_bytes']]
                digest = "sha256:" + hashlib.sha256(span_b).hexdigest()
                if digest == qhash:
                    found = True
                    break
            if not found:
                problems.append("could not reproduce quote_hash from any occurrence of quote+span_bytes")
        else:
            if quote not in raw:
                problems.append("quote not found verbatim (non-contiguous) in source file")
            else:
                digest = "sha256:" + hashlib.sha256(quote.encode('utf-8')).hexdigest()
                if digest != qhash:
                    problems.append(f"hash mismatch: recomputed={digest} stored={qhash}")

        if problems:
            n_bad += 1
            print(d['id'], "PROBLEMS:", problems)
        else:
            n_ok += 1

print()
print("total lines:", len(units))
print("unique ids:", len(set(ids)) == len(ids), f"({len(set(ids))} unique / {len(ids)} total)")
print("verified OK:", n_ok, " FAILED:", n_bad)

# id continuity check (allow 110/111 appended, 077 retired-in-place)
nums = sorted(int(i.split('-')[-1]) for i in ids)
print("id range:", nums[0], "-", nums[-1], " count:", len(nums))
