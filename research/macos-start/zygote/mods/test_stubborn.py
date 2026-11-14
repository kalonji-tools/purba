# Ctrl-C x2: the test swallows each interrupt and keeps working.
import time

def test_stubborn():
    end = time.time() + 8
    while time.time() < end:
        try:
            time.sleep(0.1)
        except KeyboardInterrupt:
            pass
