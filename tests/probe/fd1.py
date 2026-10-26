import fcntl
import os
import sys


def where(label):
    try:
        flags = fcntl.fcntl(1, fcntl.F_GETFL)
        mode = {os.O_RDONLY: "r", os.O_WRONLY: "w", os.O_RDWR: "rw"}[flags & 3]
    except OSError as error:
        os.write(2, f"{label}: fd 1 closed ({error})\n".encode())
        return
    try:
        path = fcntl.fcntl(1, 50, bytes(1024)).rstrip(b"\0").decode()
    except OSError as error:
        path = f"no path ({error})"
    st = os.fstat(1)
    line = f"{label}: fd 1 open {mode} flags={flags:#x} path={path} mode={st.st_mode:o}\n"
    os.write(2, line.encode())


if __name__ == "__main__":
    where("startup")
    if sys.argv[1:] == ["import"]:
        import purba  # noqa: F401

        where("after import purba")
