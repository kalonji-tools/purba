"""Wall time of the #979 runner and this one (#980) on the regression folders. The two runners take turns, seven
times each, and the best time of each is kept: the machine is shared, and a single run swings by a factor of two."""

import os
import subprocess
import sys
import time
from pathlib import Path

here = Path(__file__).resolve().parent
runners = {"979": here.parent / "979-setup-ends-before-checks" / "run.py", "980": here / "run.py"}
for folder in ("colorama", "defaults", "filesink-delay", "threading"):
    rounds = 3 if folder == "threading" else 7
    best = {name: float("inf") for name in runners}
    for _ in range(rounds):
        for name, script in runners.items():
            t0 = time.perf_counter()
            subprocess.run([sys.executable, str(script), "."], cwd=here / "regress" / folder, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=os.environ)
            best[name] = min(best[name], time.perf_counter() - t0)
    print(f"{folder:<15} " + "   ".join(f"{name}: {t:6.2f} s" for name, t in best.items()) + f"   (best of {rounds}, taking turns)")
