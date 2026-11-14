"""RESEARCH (#694): writes one module for each call and where it runs.

A `load` module makes the call in its body, so the zygote makes it before it
forks; its test_call makes it again, and its test_none makes none. A `test`
module makes the call in its test alone. The calls are those of safety.py.
"""

import pathlib

CALLS = {
    "dns": "import socket\n    return socket.getaddrinfo('example.com', 443)",
    "http": (
        "import urllib.request\n"
        "    with urllib.request.urlopen('https://example.com', timeout=10) as r:\n"
        "        return r.status"
    ),
    "proxies": "import urllib.request\n    return urllib.request.getproxies()",
    "cf": (
        "import ctypes\n"
        "    cf = ctypes.CDLL('/System/Library/Frameworks/'\n"
        "        'CoreFoundation.framework/CoreFoundation')\n"
        "    cf.CFStringCreateWithCString.restype = ctypes.c_void_p\n"
        "    cf.CFStringCreateWithCString.argtypes = [\n"
        "        ctypes.c_void_p, ctypes.c_char_p, ctypes.c_uint32]\n"
        "    cf.CFPreferencesCopyAppValue.restype = ctypes.c_void_p\n"
        "    cf.CFPreferencesCopyAppValue.argtypes = [ctypes.c_void_p, ctypes.c_void_p]\n"
        "    key = cf.CFStringCreateWithCString(None, b'AppleLocale', 0x08000100)\n"
        "    app = ctypes.c_void_p.in_dll(cf, 'kCFPreferencesAnyApplication')\n"
        "    return cf.CFPreferencesCopyAppValue(key, app)"
    ),
}

out = pathlib.Path(__file__).parent / "mods"
for name, body in CALLS.items():
    call = f"def call():\n    {body}\n"
    (out / f"test_load_{name}.py").write_text(
        f"{call}\ncall()\n\ndef test_call():\n    call()\n\ndef test_none():\n    pass\n"
    )
    (out / f"test_test_{name}.py").write_text(f"{call}\ndef test_call():\n    call()\n")
