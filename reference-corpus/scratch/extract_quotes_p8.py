import json, re, hashlib, sys, os

REPO_ROOT = "/home/user/sergeant-rs"
SRC = "/home/user/sergeant-rs/reference-corpus/behavior-units/P8.ndjson"

def parse_ranges(locator):
    # strip trailing parenthetical section note(s), but keep scanning whole string for L\d+ patterns
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
    # lines is 0-indexed list; file line N (1-indexed) is lines[N-1]
    if start < 1 or end > len(lines) or start > end:
        return None
    return '\n'.join(lines[start-1:end])

results = []
with open(SRC) as f:
    for lineno, line in enumerate(f, 1):
        line = line.strip()
        if not line:
            continue
        d = json.loads(line)
        locator = d['source']['locator']
        path = d['source']['path']
        ranges = parse_ranges(locator)
        info = {"id": d['id'], "locator": locator, "ranges": ranges, "path": path}
        if not ranges:
            info["error"] = "no line range found in locator"
        else:
            primary = ranges[0]
            text = span_text(path, primary[0], primary[1])
            if text is None:
                info["error"] = f"range out of bounds: {primary}"
            else:
                info["primary_range"] = primary
                info["span_text"] = text
                info["span_bytes"] = len(text.encode('utf-8'))
                info["multi_range"] = len(ranges) > 1
                if len(ranges) > 1:
                    info["other_ranges"] = ranges[1:]
        results.append(info)

# Write results as json for inspection
with open('/home/user/sergeant-rs/reference-corpus/scratch/spans_p8.json', 'w') as f:
    json.dump(results, f, indent=1)

# print summary of problems
for r in results:
    if 'error' in r or r.get('multi_range'):
        print(r['id'], r.get('locator'), '->', r.get('error', ''), 'MULTI' if r.get('multi_range') else '')
print("total", len(results))
