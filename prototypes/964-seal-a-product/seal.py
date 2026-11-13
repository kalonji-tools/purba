"""Probe for #964: seal a product between assays. deepcopy per assay vs fork per assay.

Each arm runs 3 assays on one product and reports wall time for the 3 assays,
plus whether a mutating assay leaked into the next one.
"""

import copy
import io
import os
import sys
import threading
import time


def big_list():
    return sorted(range(300_000, 0, -1))


def big_strs():
    return [str(i) * 3 for i in range(1_000_000)]


READ_FEW = {
    "first": lambda p: p[0],
    "last": lambda p: p[-1],
    "size": lambda p: len(p),
}
READ_ALL = {
    "sum": lambda p: sum(map(len, p)) if isinstance(p[0], str) else sum(p),
    "max": lambda p: max(p),
    "sorted": lambda p: p == sorted(p),
}


def mutating():
    def pop_then_size(p):
        p.pop()
        return len(p)

    return {"pop": pop_then_size, "size": lambda p: len(p)}


def by_copy(product, assays):
    out = {}
    for name, fn in assays.items():
        try:
            out[name] = fn(copy.deepcopy(product))
        except Exception as exc:  # noqa: BLE001
            out[name] = f"<{type(exc).__name__}: {exc}>"
    return out


def by_fork(product, assays):
    out = {}
    for name, fn in assays.items():
        r, w = os.pipe()
        pid = os.fork()
        if pid == 0:
            os.close(r)
            try:
                val = repr(fn(product))
            except Exception as exc:  # noqa: BLE001
                val = f"<{type(exc).__name__}: {exc}>"
            os.write(w, val.encode())
            os._exit(0)
        os.close(w)
        data = b""
        while chunk := os.read(r, 65536):
            data += chunk
        os.close(r)
        os.waitpid(pid, 0)
        out[name] = data.decode()
    return out


def by_none(product, assays):
    return {name: fn(product) for name, fn in assays.items()}


def timed(arm, product, assays, rounds=5):
    best = None
    res = None
    for _ in range(rounds):
        t0 = time.perf_counter()
        res = arm(product, assays)
        dt = time.perf_counter() - t0
        best = dt if best is None else min(best, dt)
    return best, res


def main():
    print(f"Python {sys.version.split()[0]}  pid={os.getpid()}")
    print()
    print(f"{'product':<26}{'assays':<10}{'no seal':>10}{'deepcopy':>11}{'fork':>10}")
    for pname, make in [("list of 300k int", big_list), ("list of 1M str", big_strs), ("int 42", lambda: 42)]:
        product = make()
        for aname, assays in [("read few", READ_FEW), ("read all", READ_ALL)]:
            if pname == "int 42" and aname == "read all":
                continue
            if pname == "int 42":
                assays = {"value": lambda p: p, "neg": lambda p: -p, "str": lambda p: str(p)}
            n, _ = timed(by_none, product, assays)
            c, _ = timed(by_copy, product, assays, rounds=3)
            f, _ = timed(by_fork, product, assays)
            print(f"{pname:<26}{aname:<10}{n*1000:>8.2f}ms{c*1000:>9.2f}ms{f*1000:>8.2f}ms")
    print()
    print("A mutating assay, then a reading of size (product = [1, 2, 3]):")
    for arm in (by_none, by_copy, by_fork):
        print(f"  {arm.__name__:<9}", arm([1, 2, 3], mutating()))
    print()
    print("Products deepcopy cannot copy, or copies wrongly:")
    lock = threading.Lock()
    gen = (i for i in range(3))
    buf = io.StringIO("abc")
    shared = [1]
    pair = (shared, shared)
    cases = {
        "a Lock": (lock, {"locked": lambda p: p.locked()}),
        "a generator": (gen, {"next": lambda p: next(p), "next again": lambda p: next(p)}),
        "a StringIO": (buf, {"read": lambda p: p.read(), "read again": lambda p: p.read()}),
        "an identity": (pair, {"same object": lambda p: p[0] is p[1]}),
    }
    for label, (product, assays) in cases.items():
        print(f"  {label:<13} copy={by_copy(product, assays)}  fork={by_fork(product, assays)}")
    # A real file: the offset lives in the kernel, so a fork shares it.
    path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "f.txt")
    with open(path, "w") as fh:
        fh.write("abcdef")
    fh = open(path, "rb", buffering=0)
    try:
        assays = {"read 3": lambda p: p.read(3), "read 3 again": lambda p: p.read(3)}
        print(f"  {'an open file':<13} fork={by_fork(fh, assays)}")
    finally:
        fh.close()
        os.remove(path)


if __name__ == "__main__":
    main()
