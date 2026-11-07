"""RESEARCH (#852): run every cost row and every safety case, and print them.

usage: run.py
Each safety case runs in a fresh interpreter, with Python's warning about a
fork in a multi-threaded process made visible.
"""

import json, os, platform, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
MODS = os.path.join(HERE, "mods")
DARWIN_ONLY = {"cf-prefs-both", "cf-prefs-parent-only", "mac-ver-both"}
COST = [("fork", 200), ("forkserver", 200), ("spawn", 50), ("posix_spawn", 50)]
REPEATS = 3


def header():
    print(f"python   {sys.version.split()[0]} {platform.python_implementation()}")
    print(f"system   {platform.platform()}  machine {platform.machine()}  cpus {os.cpu_count()}")
    if sys.platform == "darwin":
        print(f"macOS    {platform.mac_ver()[0]}")
    import multiprocessing

    print(f"default start method  {multiprocessing.get_start_method()}")
    try:
        import numpy

        print(f"numpy    {numpy.__version__}")
        numpy.show_config()
    except ImportError:
        print("numpy    absent")
    sys.stdout.flush()


def cost():
    print("\n## cost of each test")
    sys.stdout.flush()
    with tempfile.TemporaryDirectory() as tmp:
        env = dict(os.environ, LAB_COUNT=os.path.join(tmp, "count"))
        for module in ("lab_bare", "lab_effect"):
            for method, n in COST:
                for _ in range(REPEATS):
                    script = os.path.join(HERE, "cost.py")
                    command = [sys.executable, script, MODS, module, method, str(n)]
                    subprocess.run(command, env=env, check=False)
                    sys.stdout.flush()


def safety():
    from safety import CASES

    print("\n## fork safety")
    darwin = sys.platform == "darwin"
    runs = [(case, {}) for case in CASES if darwin or case not in DARWIN_ONLY]
    runs.append(("proxies-thread-both", {"OBJC_DISABLE_INITIALIZE_FORK_SAFETY": "YES"}))
    script = os.path.join(HERE, "safety.py")
    for case, extra in runs:
        done = subprocess.run(
            [sys.executable, "-W", "always::DeprecationWarning", script, case],
            env=dict(os.environ, **extra),
            capture_output=True,
            text=True,
            timeout=120,
        )
        lines = done.stdout.strip().splitlines()
        failed = {"case": case, "parent": f"exit {done.returncode}"}
        row = json.loads(lines[-1]) if lines else failed
        row["env"] = " ".join(f"{k}={v}" for k, v in extra.items())
        row["fork_warning"] = "multi-threaded" in done.stderr
        row["stderr"] = [
            line
            for line in done.stderr.splitlines()
            if line.strip() and "DeprecationWarning" not in line and "os.fork()" not in line
        ][:6]
        print(json.dumps(row))
        sys.stdout.flush()


if __name__ == "__main__":
    sys.path.insert(0, HERE)
    header()
    safety()
    cost()
