import csv, sys, collections

rows = list(csv.reader(open(sys.argv[1]), delimiter="\t"))
hdr = rows[0]
T = collections.defaultdict(lambda: [0, 0])
for r in rows[1:]:
    k = r[0]
    if "|" in k:
        base, kind = k.rsplit("|", 1)
        T[base][0 if kind == "fn" else 1] += int(r[-1])
    else:
        T[k][0] += int(r[-1])
for k in sorted(T):
    f, m = T[k]
    print(f"{k:40s} fn={f:5d} meth={m:5d} all={f + m:5d}")
