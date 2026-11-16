import json, sys

EX = json.load(open(sys.argv[1]))
for cat in sys.argv[2:]:
    items = EX.get(cat, [])
    seen = set()
    out = []
    for it in items:
        if it[0] not in seen:
            out.append(it)
            seen.add(it[0])
    for it in items:
        if len(out) >= 3:
            break
        if it not in out:
            out.append(it)
    print(f"== {cat} ({len(items)})")
    for s, loc, src in out[:3]:
        print("  ", loc, "|", src)
