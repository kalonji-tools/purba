# PROTOTYPE (#658). Shadows the standard library's abc, which Python loads at
# startup, and raises. Classes made before the run use the original ABCMeta.
import atexit, sys
def look():
    try:
        import abc, _collections_abc
        same = type(_collections_abc.Sized) is abc.ABCMeta
        stdlib = "lib/python" in (getattr(abc, "__file__", "") or "")
        print("SEEN import abc=%s | ABCMeta same as startup's: %s" % (
            "stdlib" if stdlib else "the failed test module", same))
    except BaseException as e:
        print("SEEN import abc=%s: %s" % (type(e).__name__, e))
atexit.register(look)
raise ValueError("abc broke")
def test_one(): assert True
