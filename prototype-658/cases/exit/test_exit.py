# PROTOTYPE (#658). Row 9 of the inventory: sys.exit at import. oxitest's
# cleanup caught only Exception, so SystemExit left the module in place.
import atexit, sys
atexit.register(lambda: print("SEEN entry=%s" % (
    "absent" if "test_exit" not in sys.modules else sys.modules["test_exit"])))
sys.exit(3)
def test_one(): assert True
