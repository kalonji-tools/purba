# PROTOTYPE (#658). As test_fresh, with this folder on sys.path, as #637 may decide.
import atexit, builtins, importlib, os, sys
builtins.runs = getattr(builtins, "runs", 0) + 1
if builtins.runs == 1:
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    def look():
        try:
            got = importlib.import_module("test_onpath")
            imp = "half-made module, has late: %s" % hasattr(got, "late")
        except BaseException as e:
            imp = "%s: %s" % (type(e).__name__, e)
        print("SEEN import=%s | body ran %d times" % (imp, builtins.runs))
    atexit.register(look)
raise ValueError("onpath broke")
def late(): return 2
def test_one(): assert True
