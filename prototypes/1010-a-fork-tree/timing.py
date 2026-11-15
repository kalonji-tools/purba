"""Wall time of replay and the fork tree (#1010), best of 5, the two modes taking turns.

`python timing.py` times each target of runall.sh. `python timing.py cost` times `chains/` with
`create_app` made to cost 0, 1, 5 and 20 ms (APP_COST_MS): a sensitivity, not corpus evidence.
"""

import os
import subprocess
import sys
import time
from pathlib import Path

here = Path(__file__).resolve().parent
p980 = here.parent / "980-rules-at-run-time"
FOLDERS_980 = (
    "first-attempts/filesink-delay",
    "first-attempts/threading",
    "first-attempts/add-option-catch",
    "regress/filesink-delay",
    "regress/add-option-catch",
    "regress/threading",
    "regress/defaults",
    "regress/colorama",
    "probes",
)


def best_of(folder: Path, target: str, env: dict[str, str]) -> dict[str, float]:
    best = {"replay": float("inf"), "tree": float("inf")}
    for _ in range(5):
        for mode in best:
            argv = [sys.executable, str(here / "run.py"), *(["--tree"] if mode == "tree" else [])]
            t0 = time.perf_counter()
            subprocess.run(
                [*argv, target], cwd=folder, env=env,
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False,
            )
            best[mode] = min(best[mode], time.perf_counter() - t0)
    return best


def row(name: str, best: dict[str, float]) -> None:
    print(f"{name:<54} replay {best['replay']:6.2f} s   tree {best['tree']:6.2f} s", flush=True)


env = {**os.environ, "PYTHONPATH": str(here / "undertest_lib")}
env.pop("PIPE_LOG", None)
if sys.argv[1:] == ["cost"]:
    for ms in ("0", "1", "5", "20"):
        best = best_of(here / "chains", ".", {**env, "APP_COST_MS": ms})
        row(f"chains, create_app costing {ms} ms", best)
    sys.exit(0)
targets = [(p980 / d, ".") for d in FOLDERS_980]
targets += [(here / "pipeline", "test_pipeline.py"), (here / "pipeline", "test_part.py")]
targets += [(here / "chains", ".")]
total = {"replay": 0.0, "tree": 0.0}
for folder, target in targets:
    best = best_of(folder, target, env)
    for mode in total:
        total[mode] += best[mode]
    row(f"{folder.relative_to(here.parent)}/{target}".replace("/.", ""), best)
row("sum of the bests", total)
