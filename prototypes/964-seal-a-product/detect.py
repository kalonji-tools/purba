"""Probe for #964: detect a change by a pickle fingerprint before and after each assay."""
import hashlib, pickle, sys, time
def fp(p): return hashlib.blake2b(pickle.dumps(p, protocol=5)).digest()
for name, p in [("list of 300k int", sorted(range(300_000, 0, -1))), ("list of 1M str", [str(i)*3 for i in range(1_000_000)])]:
    best = min((lambda t0: (fp(p), time.perf_counter()-t0)[1])(time.perf_counter()) for _ in range(5))
    print(f"{name:<20} one fingerprint {best*1000:7.2f} ms -> 3 assays need 4: {best*4000:7.2f} ms")
class Cache:
    def __init__(self): self.n = 5; self._memo = None
    def total(self):
        if self._memo is None: self._memo = self.n * 2   # a read that fills a cache
        return self._memo
c = Cache(); before = fp(c); c.total(); print("a read that fills a cache changes the fingerprint:", before != fp(c))
