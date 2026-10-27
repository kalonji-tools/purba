"""PROTOTYPE (#689): run items reversed (REORDER=reverse) or shuffled (REORDER=<seed>)."""

import os, random


def pytest_collection_modifyitems(items):
    how = os.environ.get("REORDER", "")
    if how == "reverse":
        items.reverse()
    elif how:
        random.Random(int(how)).shuffle(items)
