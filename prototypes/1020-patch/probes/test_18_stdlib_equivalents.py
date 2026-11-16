import contextlib
import os
import sys
from unittest import mock

import lending


class Stub:
    def lend(self, title):
        return "stub"


def test_setattr_is_patch_object_with_a_value():
    with mock.patch.object(lending, "Library", Stub):
        got = lending.borrow("Dune")
    assert got == "stub"


def test_setenv_is_patch_dict():
    with mock.patch.dict(os.environ, {"EDITOR": "vi"}):
        got = os.environ["EDITOR"]
    assert got == "vi"


def test_delenv_inside_patch_dict():
    with mock.patch.dict(os.environ, {"EDITOR": "vi"}):
        del os.environ["EDITOR"]
        got = "EDITOR" in os.environ
    assert got == False


def test_delattr_inside_patch_object():
    with mock.patch.object(lending, "today"):
        del lending.today
        got = hasattr(lending, "today")
    assert got == False


def test_chdir_is_contextlib_chdir():
    with contextlib.chdir("/"):
        got = os.getcwd()
    assert got == "/"


def test_syspath_prepend_is_patch_object():
    with mock.patch.object(sys, "path", ["/nowhere", *sys.path]):
        got = sys.path[0]
    assert got == "/nowhere"


def test_typo_target_still_caught():
    with mock.patch.object(lending, "Librarian", Stub):
        got = lending.borrow("Dune")
    assert got == "stub"


def test_stub_without_method_still_caught():
    class Bad:
        pass

    with mock.patch.object(lending, "Library", Bad):
        got = lending.borrow("Dune")
    assert got == "stub"
