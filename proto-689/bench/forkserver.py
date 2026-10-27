"""PROTOTYPE (#689): a process for each test, with the module body run once per module.

usage: forkserver.py MODS METHOD N
METHOD  fork        fork from a parent that imported the module
        forkserver  forkserver with the module preloaded (set_forkserver_preload)
        bare        forkserver with nothing preloaded
        spawn       a fresh interpreter for each test
Prints the cost of the first test (server start included), the cost of each later
test, and how many times the module body ran (LAB_COUNT).
"""

import importlib, multiprocessing as mp, os, sys, time

MODS, METHOD, N = sys.argv[1], sys.argv[2], int(sys.argv[3])
EXITS = []
NAME = os.environ.get("LAB_MODULE", "lab_effect")
sys.path.insert(0, MODS)


def run_test(name):
    importlib.import_module(name).test()


def fork_one(mod):
    pid = os.fork()
    if pid == 0:
        try:
            mod.test()
            os._exit(0)
        except BaseException:
            os._exit(1)
    _, status = os.waitpid(pid, 0)
    EXITS.append(os.waitstatus_to_exitcode(status))


def main():
    count = os.environ["LAB_COUNT"]
    open(count, "w").close()
    start = time.perf_counter()
    if METHOD == "fork":
        mod = importlib.import_module(NAME)
        one = lambda: fork_one(mod)
    else:
        ctx = mp.get_context("forkserver" if METHOD in ("forkserver", "bare") else "spawn")
        if METHOD == "forkserver":
            ctx.set_forkserver_preload(["__main__", NAME])

        def one():
            p = ctx.Process(target=run_test, args=(NAME,))
            p.start()
            p.join()
            EXITS.append(p.exitcode)

    one()
    first = time.perf_counter() - start
    start = time.perf_counter()
    for _ in range(N - 1):
        one()
    each = (time.perf_counter() - start) / (N - 1)
    body = len(open(count).read())
    print(
        f"{METHOD:11s} first {first * 1000:8.1f} ms   each later {each * 1000:7.2f} ms"
        f"   module body ran {body} times in {N} tests   exit codes {sorted(set(EXITS))}"
    )


if __name__ == "__main__":
    main()
