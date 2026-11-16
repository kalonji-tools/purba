import os
import pathlib
import subprocess

M = pathlib.Path("/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/e9de2c2b-a74e-4d6e-b5d0-498e964f8ad4/scratchpad/mut")
SRC = M / "src"
RUNS = M / "runs"
SCRIPTS = M / "scripts"

# project -> (own non-test source dirs, baseline pytest args)
PROJ = {
    "attrs": (["src"], []),
    "click": (["src/click"], []),
    "dateutil": (["src/dateutil"], ["tests"]),
    "loguru": (["loguru"], ["tests"]),
    "requests": (["src/requests"], ["tests"]),
    "schedule": (["schedule"], ["test_schedule.py"]),
    "six": (["."], ["test_six.py"]),
}


def src_dirs(p):
    return [str((SRC / p / d).resolve()) for d in PROJ[p][0]]


def run_pytest(p, args, out, timeout=None):
    env = dict(os.environ)
    env.update(
        RT_OUT=str(out),
        RT_SRC=":".join(src_dirs(p)),
        PYTHONPATH=str(SCRIPTS),
        PYTHONDONTWRITEBYTECODE="1",
        PYTHONHASHSEED="0",
    )
    cmd = [str(M / "venvs" / p / "bin" / "python"), "-m", "pytest", "-p", "raisetrace", "-p", "no:cacheprovider",
           "-q", *args]
    try:
        r = subprocess.run(cmd, cwd=SRC / p, env=env, capture_output=True, text=True, timeout=timeout)
        return r.returncode, r.stdout[-6000:] + r.stderr[-3000:]
    except subprocess.TimeoutExpired as e:
        return "timeout", str(e)
