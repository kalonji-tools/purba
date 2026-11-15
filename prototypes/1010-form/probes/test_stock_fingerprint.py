"""A stock is watched by its type. Before and after each check the runner takes its fingerprint,
so a check that reads it through any name is seen, and a pure read needs no checkpoint and no
copy."""

from __future__ import annotations

import hashlib
import shutil
import tempfile
from pathlib import Path

from cases import Fingerprints, Samples, use


class LogDir(Samples[Path], Fingerprints[Path]):
    """A folder for each case, discarded after it; its fingerprint is every path and every byte
    under it."""

    def sample(self) -> Path:
        return Path(tempfile.mkdtemp(prefix="p980-"))

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
        return Path(tempfile.mkdtemp(prefix="p980-"))

    def discard(self, value: Path, /) -> None:
        shutil.rmtree(value)


def test_reads_through_an_alias(d: Path = use(LogDir)):
    """PASS: two checks read the stock through an alias and a derived name, and change nothing."""
    (d / "a.log").write_text("line\n")
    same = d
    log = d / "a.log"
    assert (same / "a.log").read_text() == "line\n"
    assert log.exists() is True


def test_a_reading_that_rotates(d: Path = use(LogDir)):
    """REFUSED: the first check's reading renames the file. Says "reading it changed the stock
    LogDir" """
    (d / "a.log").write_text("line\n")
    assert (d / "a.log").rename(d / "a.log.1").name == "a.log.1"
    assert sorted(p.name for p in d.iterdir()) == ["a.log.1"]


def test_a_stock_with_no_watch(d: Path = use(SampleOnly)):
    """REFUSED before the run: SampleOnly offers neither Fingerprints nor Checkpoints."""
    (d / "a.log").write_text("line\n")
    assert (d / "a.log").read_text() == "line\n"
