"""A chain runs as a fork tree: each stage's act runs once, and its process forks for the stage's
checks and for each later stage. It is replayed instead when, after the act, a thread other than
the main one is alive, the chain reads a stock, or the act left a file open. `act` writes each call
to a file, and `count` reads it, so a check sees how often an act ran in this run. Each name is
used by one probe only."""

from __future__ import annotations

import hashlib
import shutil
import tempfile
from pathlib import Path

from cases import Fingerprints, Samples, case, use
from chain import Journal, Worker, act, count

# ── the tree: each act runs once ──────────────────────


def test_open(name: str) -> str:
    """PASS: the root's act ran once."""
    opened = act(name)
    assert count(opened) == 1
    return opened


root = case(test_open, name="tree")


def test_next(opened: str = use(root)) -> str:
    """PASS: forked from `root`, whose act ran once; its own act runs once too."""
    step = act(opened + "-next")
    assert count(opened) == 1
    assert count(step) == 1
    return step


next_step = case(test_next)


def test_leaf(step: str = use(next_step)) -> None:
    """PASS: two stages down the tree, each act above it still ran once."""
    assert count("tree") == 1
    assert count(step) == 1


def test_sibling(opened: str = use(root)) -> None:
    """PASS: a second later stage of `root` forks from the same act; nothing ran it again."""
    assert count(opened) == 1


# ── the fallbacks: the chain is replayed ──────────────────


def test_with_thread(name: str) -> Worker:
    """PASS: the act leaves a thread alive."""
    worker = Worker(act(name))
    assert worker.thread.is_alive() is True
    return worker


threaded = case(test_with_thread, name="thread")


def test_after_thread(worker: Worker = use(threaded)) -> None:
    """PASS, by replay: the act of `threaded` ran twice, once in its case and once here. Says "ran
    by replay: a thread other than the main one is alive after the act (thread)" """
    assert count(worker.thread.name) == 2


class Db(Samples[Path], Fingerprints[Path]):
    """A folder for each case, discarded after it."""

    def sample(self) -> Path:
        return Path(tempfile.mkdtemp(prefix="form-db-"))

    def discard(self, value: Path, /) -> None:
        shutil.rmtree(value)

    def fingerprint(self, value: Path, /) -> object:
        rows = value / "rows"
        return hashlib.sha256(rows.read_bytes()).hexdigest() if rows.exists() else None


def test_with_stock(name: str, db: Path = use(Db)) -> Path:
    """PASS: the act writes to a stock."""
    (db / "rows").write_text(act(name))
    assert (db / "rows").read_text() == name
    return db


stocked = case(test_with_stock, name="stock")


def test_after_stock(db: Path = use(stocked)) -> None:
    """PASS, by replay: the act of `stocked` ran twice. Says "ran by replay: the chain reads a stock
    (Db)" """
    assert (db / "rows").read_text() == "stock"
    assert count("stock") == 2


def test_with_open_file(name: str) -> Journal:
    """PASS: the act leaves a file open."""
    journal = Journal(act(name))
    assert journal.lines() == [name]
    return journal


journaled = case(test_with_open_file, name="file")


def test_after_open_file(journal: Journal = use(journaled)) -> None:
    """PASS, by replay: the act of `journaled` ran twice. Says "ran by replay: the act left a file,
    socket or pipe open" """
    assert journal.lines() == ["file"]
    assert count("file") == 2
