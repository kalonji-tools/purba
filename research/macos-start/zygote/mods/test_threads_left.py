# Q4: a test that leaves a thread alive when it returns.
import threading, time

def late(n):
    time.sleep(0.3)
    raise ValueError(f"late {n}")

def test_non_daemon_raises_later():
    threading.Thread(target=late, args=(1,)).start()

def test_daemon_raises_later():
    threading.Thread(target=late, args=(2,), daemon=True).start()

def test_next():
    pass
