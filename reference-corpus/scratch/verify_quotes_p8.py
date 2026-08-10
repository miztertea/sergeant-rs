import json, re, hashlib, os

REPO_ROOT = "/home/user/sergeant-rs"
IN = "/home/user/sergeant-rs/reference-corpus/scratch/P8.quotes.ndjson"

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
        full = os.path.join(REPO_ROOT, path)
        with open(full, 'r', encoding='utf-8') as f:
            file_cache[path] = f.read()
    return file_cache[path]

MULTI_RANGE_PRIMARY = {"BU-P8-032": 1, "BU-P8-075": 0}

n_ok = 0
n_bad = 0
with open(IN) as f:
    for line in f:
        line = line.strip()
        if not line: continue
        d = json.loads(line)
        uid = d['id']
        src = d['source']
        locator = src['locator']
        path = src['path']
        ranges = parse_ranges(locator)
        idx = MULTI_RANGE_PRIMARY.get(uid, 0)
        start, end = ranges[idx]
        raw = get_raw(path)
        lines = raw.split('\n')
        span = '\n'.join(lines[start-1:end])
        span_bytes = span.encode('utf-8')
        digest = "sha256:" + hashlib.sha256(span_bytes).hexdigest()

        ok = True
        problems = []
        if digest != src['quote_hash']:
            ok = False
            problems.append(f"hash mismatch: recomputed={digest} stored={src['quote_hash']}")
        if len(span) <= 500:
            if src['quote'] != span:
                ok = False
                problems.append("quote != full span (untruncated case)")
            if 'span_bytes' in src:
                ok = False
                problems.append("span_bytes present but span <=500 chars")
        else:
            if src['quote'] != span[:500]:
                ok = False
                problems.append("quote != first 500 chars of span")
            if src.get('span_bytes') != len(span_bytes):
                ok = False
                problems.append(f"span_bytes mismatch: stored={src.get('span_bytes')} actual={len(span_bytes)}")
        # contiguity check: the exact span must appear verbatim in the raw file
        if span not in raw:
            ok = False
            problems.append("span not found verbatim in source file (non-contiguous?!)")

        if ok:
            n_ok += 1
        else:
            n_bad += 1
            print(uid, "FAIL:", problems)

print(f"\nverified OK: {n_ok}  FAILED: {n_bad}  total: {n_ok+n_bad}")
