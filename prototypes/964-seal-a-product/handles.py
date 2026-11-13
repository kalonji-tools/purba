"""Probe for #964: which values a fork does not seal, and what fileno() says of each."""

import copy
import io
import mmap
import os
import socket
import tempfile

fd, path = tempfile.mkstemp(dir=os.path.dirname(os.path.abspath(__file__)))
os.close(fd)
f = open(path, "rb")
try:
    copy.deepcopy(f)
    print("deepcopy of an open file: copied")
except TypeError as exc:
    print("deepcopy of an open file: TypeError:", exc)
for name, value in [("file", f), ("socket", socket.socket()), ("StringIO", io.StringIO("x")),
                    ("BytesIO", io.BytesIO(b"x")), ("mmap", mmap.mmap(-1, 16))]:
    try:
        print(f"{name:<9} fileno() -> {value.fileno()}")
    except AttributeError:
        print(f"{name:<9} fileno() -> no such method")
    except io.UnsupportedOperation:
        print(f"{name:<9} fileno() -> UnsupportedOperation")
f.close()
os.remove(path)

m = mmap.mmap(-1, 16)
pid = os.fork()
if pid == 0:
    m[0:3] = b"abc"
    os._exit(0)
os.waitpid(pid, 0)
print("anonymous mmap: the parent sees the child's write:", bytes(m[0:3]))

a, b = socket.socketpair()
b.send(b"xyz")
pid = os.fork()
if pid == 0:
    a.recv(3)
    os._exit(0)
os.waitpid(pid, 0)
a.setblocking(False)
try:
    print("socket: after the child's recv, the parent reads", a.recv(3))
except BlockingIOError:
    print("socket: after the child's recv, the parent finds it empty")
