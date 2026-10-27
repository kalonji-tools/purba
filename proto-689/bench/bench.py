import os, sys, time, subprocess, importlib, multiprocessing as mp

MODS = sys.argv[1]
sys.path.insert(0, MODS)


def per(label, n, fn):
    fn()  # warm
    t = time.perf_counter()
    for _ in range(n):
        fn()
    dt = (time.perf_counter() - t) / n
    print(f"{label:52s} {dt * 1e6:12.1f} us/test   n={n}")
    return dt


def fork_call(mod):
    def go():
        r, w = os.pipe()
        pid = os.fork()
        if pid == 0:
            os.close(r)
            try:
                mod.test()
                os.write(w, b"P")
            except BaseException:
                os.write(w, b"F")
            os._exit(0)
        os.close(w)
        out = os.read(r, 1)
        os.close(r)
        os.waitpid(pid, 0)
        assert out == b"P", out

    return go


def reimport(name):
    def go():
        sys.modules.pop(name, None)
        importlib.import_module(name).test()

    return go


def spawn(args):
    return lambda: subprocess.run(args, check=True)


def mp_target(name):
    sys.path.insert(0, MODS)
    importlib.import_module(name).test()


def mp_call(ctx, name):
    def go():
        p = ctx.Process(target=mp_target, args=(name,))
        p.start()
        p.join()
        assert p.exitcode == 0

    return go


if __name__ == "__main__":
    print(sys.version.split()[0], os.cpu_count(), "cpus")
    for name in ("lab_light", "lab_heavy"):
        mod = importlib.import_module(name)
        print(f"--- {name}")
        per("in process: call the test", 20000, mod.test)
        per("module beaker: drop from sys.modules, re-import", 2000, reimport(name))
        per("process beaker: os.fork() from the warm parent", 1000, fork_call(mod))
        per(
            "process beaker: mp forkserver Process()",
            200,
            mp_call(mp.get_context("forkserver"), name),
        )
        per("process beaker: mp spawn Process()", 50, mp_call(mp.get_context("spawn"), name))
        code = f"import sys; sys.path.insert(0, {MODS!r}); import {name}; {name}.test()"
        per(
            "process beaker: fresh interpreter, import, call",
            100,
            spawn([sys.executable, "-c", code]),
        )
    print("--- reference")
    per("fresh interpreter: python -c pass", 100, spawn([sys.executable, "-c", "pass"]))
    per(
        "fresh interpreter: python -I -S -c pass",
        100,
        spawn([sys.executable, "-I", "-S", "-c", "pass"]),
    )
