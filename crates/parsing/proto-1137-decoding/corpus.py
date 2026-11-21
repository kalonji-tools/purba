"""PROTOTYPE #1137: writes one module per row of the corpus into the folder given."""

import pathlib
import sys

BOM = b"\xef\xbb\xbf"

# Each test compares the decoded text with an escape, which is ASCII, so the
# right-hand side means the same under every way to decode.
CORPUS = {
    "test_latin1.py": b'# -*- coding: latin-1 -*-\ndef test_one():\n    s = "\xe9"\n    assert s == "\\u00e9"\n',
    "test_latin1_80.py": b'# coding: latin-1\ndef test_one():\n    s = "\x80"\n    assert s == "\\x80"\n',
    "test_iso8859_80.py": b'# coding: iso-8859-1\ndef test_one():\n    s = "\x80"\n    assert s == "\\x80"\n',
    "test_cp1252.py": b'# coding: cp1252\ndef test_one():\n    s = "\x80"\n    assert s == "\\u20ac"\n',
    "test_euc_jp.py": b'# coding: euc_jp\ndef test_one():\n    s = "' + "日本".encode("euc_jp") + b'"\n    assert s == "\\u65e5\\u672c"\n',
    "test_latin1_ascii.py": b"# coding: latin-1\ndef test_one():\n    assert 1 == 1\n",
    "test_ascii_e9.py": b'# coding: ascii\ndef test_one():\n    s = "\xe9"\n    assert s == "\\u00e9"\n',
    "test_bom_latin1.py": BOM + b"# coding: latin-1\ndef test_one():\n    assert 1 == 1\n",
    "test_bad_code.py": b'def test_one():\n    s = "\xff"\n    assert s == "\\xff"\n',
    "test_bad_comment.py": b"# caf\xe9\ndef test_one():\n    assert 1 == 1\n",
    "test_crlf.py": b'# coding: latin-1\r\ndef test_one():\r\n    s = """\xe9\r\nb"""\r\n    assert s == "\\u00e9\\nb"\r\n',
}

if __name__ == "__main__":
    folder = pathlib.Path(sys.argv[1])
    folder.mkdir(parents=True, exist_ok=True)
    for name, data in CORPUS.items():
        (folder / name).write_bytes(data)
        print(name)
