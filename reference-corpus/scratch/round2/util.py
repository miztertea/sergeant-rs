import json

def load(path):
    with open(path, encoding='utf-8') as f:
        return [json.loads(line) for line in f if line.strip()]

def dump(path, records):
    with open(path, 'w', encoding='utf-8') as f:
        for r in records:
            f.write(json.dumps(r, ensure_ascii=False))
            f.write('\n')

def by_id(records):
    return {r['id']: r for r in records}
