"""FORM.md: with no checkpoint, two checks that read one stock are REFUSED (before the run).

Expect: `refused` in the tally for test_two_checks_name. Also: does an alias or a local
that holds the stock escape the rule?
"""

from pathlib import Path

from cases import use
from stockdir import Both, Dir


def test_two_checks_name(d: Path = use(Dir)):
    (d / "a").write_text("1")
    assert (d / "a").read_text() == "1"
    assert (d / "a").exists() is True


def test_two_checks_alias(d: Path = use(Dir)):
    (d / "a").write_text("1")
    same = d
    assert (same / "a").read_text() == "1"
    assert (same / "a").exists() is True


def test_two_checks_derived(d: Path = use(Dir)):
    f = d / "a"
    f.write_text("1")
    assert f.read_text() == "1"
    assert f.exists() is True


def test_both_which_value(d: Path = use(Both)):
    name = d.name
    assert name[:12] == "probe-sample"
