"""PROTOTYPE (#689) control: each test's leak is known in advance."""

import functools, os
import click
from click.globals import push_context

CONFIG = {"mode": "default"}
SEEN = []


def helper():  # the leak hides behind a call
    SEEN.append(1)


@functools.lru_cache
def cached(n):  # state inside C: no fingerprint reaches it
    return n * 2


def set_env():
    os.environ["PLANTED_VIA_CALL"] = "1"


def test_p_clean():  # truth: clean
    assert sum([1, 2]) == 3


def test_p_global_dict():  # truth: leaks CONFIG   · text can see it
    CONFIG["mode"] = "x"
    assert CONFIG["mode"] == "x"


def test_p_env():  # truth: leaks env      · text can see it
    os.environ["PLANTED"] = "1"
    assert os.environ["PLANTED"] == "1"


def test_p_cwd(tmp_path):  # truth: leaks cwd      · text can see it
    os.chdir(tmp_path)
    assert os.getcwd() == str(tmp_path)


def test_p_helper():  # truth: leaks SEEN     · text cannot see it
    helper()
    assert SEEN


def test_p_env_via_call():  # truth: leaks env      · text cannot see it
    set_env()
    assert os.environ["PLANTED_VIA_CALL"] == "1"


def test_p_click_context():  # truth: leaks click's context stack · text cannot see it
    push_context(click.Context(click.Command("x")))
    assert click.get_current_context().command.name == "x"


def test_p_lru_cache():  # truth: leaks the cache · neither can see it
    assert cached(2) == 4


def test_p_monkeypatch(monkeypatch):  # truth: clean, monkeypatch restores
    monkeypatch.setenv("PLANTED_MP", "1")
    assert os.environ["PLANTED_MP"] == "1"
