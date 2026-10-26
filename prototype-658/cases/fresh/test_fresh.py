# PROTOTYPE (#658). A new name whose body raises; the folder is off sys.path.
import atexit, importlib, sys
def look():
    entry = sys.modules.get("test_fresh", "absent")
    shown = entry
    if entry not in ("absent", None):
        shown = "half-made, has late: %s" % hasattr(entry, "late")
    try:
        got = importlib.import_module("test_fresh")
        imp = "half-made module, has late: %s" % hasattr(got, "late")
    except BaseException as e:
        imp = "%s: %s" % (type(e).__name__, e)
    print("SEEN entry=%s | import=%s" % (shown, imp))
atexit.register(look)
def helper(): return 1
raise ValueError("fresh broke")
def late(): return 2
def test_one(): assert helper() == 1
