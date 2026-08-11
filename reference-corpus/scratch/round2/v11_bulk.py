import json

def load(path):
    with open(path, encoding='utf-8') as f:
        return [json.loads(line) for line in f if line.strip()]

def dump(path, records):
    with open(path, 'w', encoding='utf-8') as f:
        for r in records:
            f.write(json.dumps(r, ensure_ascii=False))
            f.write('\n')

def is_bare(a):
    return not any(c in a for c in ['(', '—', ':', ' - ', '--'])

def reason(alt, rec):
    stage = rec.get('stage')
    workflow = rec.get('workflow')
    scope = rec.get('scope')
    stage_ref = f"`{stage}`" if stage else "this stage"
    if alt == 'helper':
        return (f"helper (rejected: {stage_ref} is the durable, independently checkable "
                 f"outcome itself (§6.3), not a deterministic subordinate mechanic invoked "
                 f"from inside another stage)")
    if alt == 'stage-context':
        return (f"stage-context (rejected: {stage_ref} is itself the durable, independently "
                 f"reachable checkpoint (§6.3), not advisory guidance folded into an "
                 f"adjacent stage's own context)")
    if alt in ('agents-invariant', 'AGENTS.md invariant'):
        return (f"{alt} (rejected: scoped to {scope}, not a standing rule that holds "
                 f"independent of any one run)")
    if alt == 'shared-context':
        return (f"shared-context (rejected: this behavior belongs to `{workflow}`'s own "
                 f"procedure, not a rule consulted by multiple workflows)")
    if alt == 'shared-helper':
        return (f"shared-helper (rejected: this behavior belongs to `{workflow}`'s own "
                 f"procedure, not a mechanism reused across multiple workflows)")
    raise ValueError(f"no template for alt={alt!r} on {rec['id']}")

def main():
    total = 0
    for fn in ['P6', 'P7', 'P8']:
        path = f'behavior-units/{fn}.ndjson'
        records = load(path)
        changed = 0
        for r in records:
            if r.get('representation') != 'stage':
                continue
            alts = r.get('alternatives_considered') or []
            new_alts = []
            touched = False
            for a in alts:
                if is_bare(a):
                    new_alts.append(reason(a, r))
                    touched = True
                else:
                    new_alts.append(a)
            if touched:
                r['alternatives_considered'] = new_alts
                changed += 1
        dump(path, records)
        print(fn, 'units changed:', changed)
        total += changed
    print('total units changed:', total)

if __name__ == '__main__':
    main()
