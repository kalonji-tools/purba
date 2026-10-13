# purba must never import this module, because the run names test_a.py alone.
# An import leaves the file that $PURBA_SENTINEL names before it raises, so an
# import whose error purba swallows still shows.
import os

open(os.environ["PURBA_SENTINEL"], "w").close()
raise RuntimeError("test_b.py was imported")
