import sys
src = "def f():\n    assert False, 'kept'\n\ndbg = __debug__\n"
for opt in (-1, 0):
    ns = {}
    exec(compile(src, "m.py", "exec", optimize=opt), ns)
    try:
        ns["f"](); r = "stripped"
    except AssertionError as e:
        r = f"kept ({e})"
    print(f"optimize={opt:2}: flags.optimize={sys.flags.optimize}"
          f" assert {r}, __debug__={ns['dbg']}")
