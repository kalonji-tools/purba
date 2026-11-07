"""RESEARCH (#852): the cost of each test, by the way a process starts it.

usage: cost.py MODS MODULE METHOD N
METHOD  fork         fork from a parent that imported the module (the zygote)
        forkserver   multiprocessing's forkserver, the module preloaded
        spawn        multiprocessing's spawn: a fresh interpreter for each test
        posix_spawn  os.posix_spawn of a fresh interpreter that imports the module
Prints the cost of the first test, the cost of each later test, and how many
times the module body ran (LAB_COUNT). Copied from proto-689/bench/forkserver.py
at 1e0ef96, with posix_spawn added.
"""

import importlib, multiprocessing as mp, os, sys, time

MODS, NAME, METHOD, N = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4])
EXITS = []
sys.path.insert(0, MODS)
CODE = f"import sys; sys.path.insert(0, {MODS!r}); import {NAME}; {NAME}.test()"


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


def posix_spawn_one():
    pid = os.posix_spawn(sys.executable, [sys.executable, "-c", CODE], os.environ)
    _, status = os.waitpid(pid, 0)
    EXITS.append(os.waitstatus_to_exitcode(status))


def main():
    count = os.environ["LAB_COUNT"]
    open(count, "w").close()
    start = time.perf_counter()
    if METHOD == "fork":
        mod = importlib.import_module(NAME)
        one = lambda: fork_one(mod)
    elif METHOD == "posix_spawn":
        one = posix_spawn_one
    else:
        ctx = mp.get_context(METHOD)
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
        f"{NAME:10s} {METHOD:11s} first {first * 1000:8.1f} ms   each later {each * 1000:7.2f} ms"
        f"   module body ran {body} times in {N} tests   exit codes {sorted(set(EXITS))}"
    )


if __name__ == "__main__":
    main()
