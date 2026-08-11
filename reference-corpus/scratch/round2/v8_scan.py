import json, re, subprocess, sys

REPO = "/home/user/sergeant-rs"

STOP = set("""
a an and are as at be but by for from has have if in into is it its of on
or that the their there this to was were will with not no never only when
where which who whom whose while than then so such must may can could
should would within without between into onto over under again once
each other same both any all every one two per own itself itself's
""".split())

def sig_words(text):
    words = re.findall(r"[a-zA-Z][a-zA-Z0-9_-]{2,}", text.lower())
    return [w for w in words if w not in STOP]

def load(fn, ref):
    out = subprocess.run(["git", "show", f"{ref}:reference-corpus/behavior-units/{fn}.ndjson"],
                          capture_output=True, text=True, cwd=REPO)
    d = {}
    for line in out.stdout.splitlines():
        if not line.strip():
            continue
        r = json.loads(line)
        d[r["id"]] = r
    return d

def main():
    rows = []
    for fn in ["P5", "P6", "P7", "P8"]:
        old = load(fn, "b0eeeba")
        cur_path = f"{REPO}/reference-corpus/behavior-units/{fn}.ndjson"
        with open(cur_path, encoding="utf-8") as f:
            cur = {}
            for line in f:
                if not line.strip():
                    continue
                r = json.loads(line)
                cur[r["id"]] = r
        for uid, rec in cur.items():
            o = old.get(uid)
            if o is None:
                continue  # A12 new unit, not re-anchored
            oh = o.get("source", {}).get("quote_hash")
            nh = rec.get("source", {}).get("quote_hash")
            if oh == nh:
                continue  # not re-anchored
            stmt = rec.get("statement", "")
            quote = rec.get("source", {}).get("quote", "") or ""
            sw = set(sig_words(stmt))
            qw = set(sig_words(quote))
            if not sw:
                continue
            overlap = sw & qw
            ratio = len(overlap) / len(sw)
            rows.append((fn, uid, ratio, len(sw), len(overlap), stmt[:90], quote[:90].replace("\n"," ")))
    rows.sort(key=lambda r: r[2])
    for r in rows[:60]:
        print(f"{r[0]} {r[1]:12s} ratio={r[2]:.2f} ({r[4]}/{r[3]}) | STMT: {r[5]!r} | QUOTE: {r[6]!r}")

if __name__ == "__main__":
    main()
