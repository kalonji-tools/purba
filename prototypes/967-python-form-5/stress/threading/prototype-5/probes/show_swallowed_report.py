"""Runs run.py on probe m2b, then prints what went into the swapped sys.stdout and sys.stderr buffers.

Usage: python show_swallowed_report.py <F/run.py> <probe file>
"""

import io
import os
import runpy
import sys
from typing import cast

run_py, probe = sys.argv[1], sys.argv[2]
sys.argv = [run_py, probe]
real_out = os.dup(1)
code: object = 0
try:
    runpy.run_path(run_py, run_name="__main__")
except SystemExit as stop:
    code = stop.code
os.write(real_out, b"--- swallowed stdout ---\n" + cast(io.StringIO, sys.stdout).getvalue().encode())
os.write(real_out, b"--- swallowed stderr ---\n" + cast(io.StringIO, sys.stderr).getvalue().encode())
os.write(real_out, f"--- exit {code}\n".encode())
