# The issue's Done-when lines: globals isolated, module body once, abort keeps others' lines.
import os
LOG = os.environ.get("BODY_LOG", "/dev/null")
with open(LOG, "a") as f:
    f.write("body\n")
counter = 0

def test_a_changes_global():
    global counter
    counter = 99

def test_b_reads_global():
    assert counter == 0, f"counter is {counter}"

def test_c_aborts():
    os.abort()

def test_d_after_abort():
    pass
