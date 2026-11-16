"""PROTOTYPE #1025: `each(xs)`, a proxy whose comparison holds when it holds for every item."""
class each:
    def __init__(self, xs):
        self.xs = list(xs)
    def _all(self, op, sym, other):
        bad = [(i, x) for i, x in enumerate(self.xs) if not op(x, other)]
        if bad:
            raise AssertionError(
                "\n".join([f"each item {sym} {other!r}"] + [f"  [{i}]  {x!r}" for i, x in bad]
                          + [f"  {len(bad)} of {len(self.xs)} items break it"]))
        return True
    def __eq__(self, o): return self._all(lambda a, b: a == b, "==", o)
    def __ne__(self, o): return self._all(lambda a, b: a != b, "!=", o)
    def __lt__(self, o): return self._all(lambda a, b: a < b, "<", o)
    def __le__(self, o): return self._all(lambda a, b: a <= b, "<=", o)
    def __gt__(self, o): return self._all(lambda a, b: a > b, ">", o)
    def __ge__(self, o): return self._all(lambda a, b: a >= b, ">=", o)
