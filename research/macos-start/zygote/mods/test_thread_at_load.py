# Q1: a module whose import starts a thread.
import os, threading
LOG = os.environ.get("BODY_LOG", "/dev/null")
with open(LOG, "a") as f:
    f.write("body\n")
threading.Thread(target=threading.Event().wait, daemon=True).start()

def test_one():
    pass

def test_two():
    pass
