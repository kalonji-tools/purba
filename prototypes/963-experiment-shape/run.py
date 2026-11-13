"""PROTOTYPE (#963): `python run.py` runs every example and prints the report."""

import importlib
import sys

import experiment

for name in sys.argv[1:] or ["ex1_slugify", "ex2_money", "ex3_sign_up", "ex4_cart", "ex5_refused", "ex6_friction"]:
    importlib.import_module(name)
experiment.run_all()
