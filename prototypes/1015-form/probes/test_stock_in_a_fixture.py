"""A fixture may use a stock. The stock keeps its rules: the runner watches it by its type, through
the fixture's value, and refuses a stock with no watch before the run."""

from __future__ import annotations

import hashlib
import shutil
import tempfile
from pathlib import Path

from cases import Fingerprints, Samples, use


class LogDir(Samples[Path], Fingerprints[Path]):
    def sample(self) -> Path:
        return Path(tempfile.mkdtemp(prefix="p1010-"))

    def discard(self, value: Path, /) -> None:
        shutil.rmtree(value)

    def fingerprint(self, value: Path, /) -> object:
        h = hashlib.sha256()
        for p in sorted(value.rglob("*")):
            h.update(str(p.relative_to(value)).encode())
            if p.is_file():
                h.update(p.read_bytes())
        return h.hexdigest()


class SampleOnly(Samples[Path]):
    def sample(self) -> Path:
        return Path(tempfile.mkdtemp(prefix="p1010-"))

    def discard(self, value: Path, /) -> None:
        shutil.rmtree(value)


def app_log(folder: Path = use(LogDir)) -> Path:
    return folder / "app.log"


def bare_log(folder: Path = use(SampleOnly)) -> Path:
    return folder / "app.log"


def test_stock_through_a_fixture(log: Path = use(app_log), folder: Path = use(LogDir)) -> None:
    """PASS: the fixture and the test share the case's one sample of LogDir."""
    log.write_text("line\n")
    assert sorted(p.name for p in folder.iterdir()) == ["app.log"]


def test_reading_changes_the_stock(log: Path = use(app_log)) -> None:
    """REFUSED: the reading renames the file, which the runner sees through the fixture's value.
    Says "reading it changed the stock LogDir" """
    log.write_text("line\n")
    assert log.rename(log.with_name("app.log.1")).name == "app.log.1"


def test_stock_with_no_watch(log: Path = use(bare_log)) -> None:
    """REFUSED before the run. Says "the input log (through bare_log): the stock SampleOnly offers
    neither Fingerprints nor Checkpoints" """
    log.write_text("line\n")
    assert log.read_text() == "line\n"
