import json, sys

EX = json.load(open(sys.argv[1]))
for cat in sorted(EX):
    items = EX[cat]
    print(f"== {cat} ({len(items)})")
    for s, loc, src in (
        items[:400] if len(sys.argv) > 2 and cat in sys.argv[2:] else items[:6]
    ):
        print("  ", loc, "|", src[:150])
