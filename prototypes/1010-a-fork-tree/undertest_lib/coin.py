"""The code under test for the injected control `controls/test_coin.py`.

Its act reads `random`.
"""

from __future__ import annotations

import os
import random


def toss() -> str:
    path = os.environ.get("PIPE_LOG")
    if path:
        with open(path, "a") as f:
            f.write(f"{os.getpid()} toss\n")
    return random.choice(["heads", "tails"])
