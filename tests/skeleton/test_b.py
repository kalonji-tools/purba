import os
import pathlib

pathlib.Path(os.environ["PURBA_SENTINEL"]).touch()
raise RuntimeError("purba imported test_b.py")
