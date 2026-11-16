"""#1015, change 7: the run seeds `random` in each case, from its seed and the case's name. The
summary line prints the seed; a failed or refused case prints how to repeat it. runall.sh runs
this file twice with `--seed 1015`, and once with a seed the runner picks, and compares."""

from __future__ import annotations

import random

from cases import case


def test_draws(sides: int) -> None:
    """PASS: a draw is in range, whatever the seed."""
    roll = random.randint(1, sides)
    assert roll >= 1


six = case(test_draws, sides=6)


def test_shows_the_draw(sides: int) -> None:
    """FAIL: no die rolls 0; the failure shows the draw and how to repeat it. Says "random was
    seeded for this case; python run.py --seed" """
    roll = random.randint(1, sides)
    assert roll == 0


d6 = case(test_shows_the_draw, sides=6)
d20 = case(test_shows_the_draw, sides=20)


def test_one_way_draw() -> None:
    """REFUSED: the branch goes the way the draw sends it; the refusal also shows how to repeat it.
    Says "random was seeded for this case" """
    roll = random.random()
    kept = "low" if roll < 2 else "high"
    assert kept == "low"
