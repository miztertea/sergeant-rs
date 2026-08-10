import json, re, hashlib, os

REPO_ROOT = "/home/user/sergeant-rs"
SRC = "/home/user/sergeant-rs/reference-corpus/behavior-units/P8.ndjson"
OUT = "/home/user/sergeant-rs/reference-corpus/scratch/P8.quotes.ndjson"

# Units whose locator lists >1 range: which range index (0-based) to use as the
# hashed contiguous span, chosen as the richer/more-specific span; the other
# range is recorded in notes as un-hashed corroboration (A2 requires ONE
# contiguous span per hash).
MULTI_RANGE_PRIMARY = {
    "BU-P8-032": 1,  # use L64-71 (groups-fields table: agent_instructions + accessibility routing), not L43
    "BU-P8-075": 0,  # use L196-208 (wake-condition table); L224-227 corroborates human_response/deployment non-auto-eval
}

def parse_ranges(locator):
    ranges = []
    for m in re.finditer(r'L(\d+)(?:-(\d+))?', locator):
        start = int(m.group(1))
        end = int(m.group(2)) if m.group(2) else start
        ranges.append((start, end))
    return ranges

file_cache = {}
def get_lines(path):
    if path not in file_cache:
        full = os.path.join(REPO_ROOT, path)
        with open(full, 'r', encoding='utf-8') as f:
            file_cache[path] = f.read().split('\n')
    return file_cache[path]

def span_text(path, start, end):
    lines = get_lines(path)
    if start < 1 or end > len(lines) or start > end:
        return None
    return '\n'.join(lines[start-1:end])

out_lines = []
report = []
with open(SRC) as f:
    for line in f:
        line = line.strip()
        if not line:
            continue
        d = json.loads(line)
        uid = d['id']
        locator = d['source']['locator']
        path = d['source']['path']
        ranges = parse_ranges(locator)
        assert ranges, f"{uid}: no ranges in locator {locator!r}"
        idx = MULTI_RANGE_PRIMARY.get(uid, 0)
        start, end = ranges[idx]
        text = span_text(path, start, end)
        assert text is not None, f"{uid}: bad range {start}-{end} in {path}"

        span_bytes_full = text.encode('utf-8')
        digest = hashlib.sha256(span_bytes_full).hexdigest()

        if len(text) <= 500:
            quote = text
            has_span_bytes = False
        else:
            quote = text[:500]
            has_span_bytes = True

        d['source']['quote'] = quote
        d['source']['quote_hash'] = f"sha256:{digest}"
        if has_span_bytes:
            d['source']['span_bytes'] = len(span_bytes_full)
        else:
            d['source'].pop('span_bytes', None)

        report.append({
            "id": uid, "locator": locator, "used_range": [start, end],
            "multi_range": len(ranges) > 1,
            "other_ranges": ranges[:idx] + ranges[idx+1:] if len(ranges) > 1 else [],
            "quote_len_chars": len(text), "span_bytes": len(span_bytes_full),
            "truncated": has_span_bytes,
        })

        out_lines.append(json.dumps(d, ensure_ascii=False))

with open(OUT, 'w') as f:
    f.write('\n'.join(out_lines) + '\n')

with open('/home/user/sergeant-rs/reference-corpus/scratch/quote_report_p8.json', 'w') as f:
    json.dump(report, f, indent=1)

print("wrote", len(out_lines), "units to", OUT)
print("multi-range units:", [r['id'] for r in report if r['multi_range']])
print("truncated (>500 chars) units:", [r['id'] for r in report if r['truncated']])
