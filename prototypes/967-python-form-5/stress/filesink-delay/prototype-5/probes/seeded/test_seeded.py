"""Seeded mistakes a tester of test_filesink_delay.py could plausibly make. Never part of the port.

Run from here with the port's helpers on the path:
    PYTHONPATH=../../tests python F/run.py .
Each test is named for its mistake; S-numbers match NOTES.md.
"""

import datetime
import os
import time
from pathlib import Path

from cases import before_each, case, use
from loguru import logger
from loguru_lab import TmpPath, freeze_time, reset_logger

before_each(reset_logger)


# S1: a wrong expected file name (350000 typed as 300000).
def test_s1_wrong_expected_name(tmp_path: Path = use(TmpPath)):
    frozen = freeze_time("1999-12-12")
    logger.add(tmp_path / "file.log", rotation=30, retention=2, delay=True, format="{message}")
    for i in range(1, 10):
        time.sleep(0.05)
        frozen.tick(datetime.timedelta(seconds=0.05))
        logger.info(str(i) * 20)
    assert sorted(os.listdir(tmp_path)) == [
        "file.1999-12-12_00-00-00_300000.log",
        "file.1999-12-12_00-00-00_400000.log",
        "file.log",
    ]


# S2: `not file.exists()` rewritten as `== 0`, as if a bool were an int.
def test_s2_bool_as_int(tmp_path: Path = use(TmpPath)):
    file = tmp_path / "test.log"
    logger.add(file, format="{message}", delay=True)
    existed = file.exists()
    logger.debug("Delayed")
    assert existed == 0
    assert file.read_text() == "Delayed\n"


# S3: forgot `logger.remove(i)`, so nothing is compressed.
def test_s3_forgot_remove(tmp_path: Path = use(TmpPath)):
    logger.add(tmp_path / "file.log", compression="gz", delay=True)
    logger.debug("a")
    assert sorted(os.listdir(tmp_path)) == ["file.log.gz"]


# S4 and S5: a case with a misspelled input, and a case with an input of the wrong type.
def test_s4_s5_case_inputs(tmp_path: Path):
    i = logger.add(tmp_path / "file.log", compression="gz", delay=True)
    logger.remove(i)
    assert list(tmp_path.iterdir()) == []


s4_misspelled = case(test_s4_s5_case_inputs, tmp_pth=Path("/nonexistent"))
s5_wrong_type = case(test_s4_s5_case_inputs, tmp_path="/nonexistent")


# S6: a date handed to freeze_time, which takes a str.
def test_s6_date_not_str(tmp_path: Path = use(TmpPath)):
    freeze_time(datetime.date(2001, 2, 3))
    i = logger.add(tmp_path / "file.log", rotation=0, delay=True, format="{message}")
    logger.debug("a")
    logger.remove(i)
    assert (tmp_path / "file.log").read_text() == "a\n"


# S7: a bare truth check, as the original's `assert not file.exists()`.
def test_s7_bare_truth(tmp_path: Path = use(TmpPath)):
    file = tmp_path / "test.log"
    logger.add(file, format="{message}", delay=True)
    assert not file.exists()


# S8: a typo of the input's name inside the act.
def test_s8_name_typo(tmp_path: Path = use(TmpPath)):
    i = logger.add(tmp_pth / "file.log", rotation=0, delay=True)
    logger.remove(i)
    assert list(tmp_path.iterdir()) == []


# S9: forgot `use(…)` around the fixture.
def test_s9_forgot_use(tmp_path: Path = TmpPath):
    i = logger.add(tmp_path / "file.log", rotation=0, delay=True)
    logger.remove(i)
    assert list(tmp_path.iterdir()) == []


# S10: the original's check_dir kept as a helper: it has `if`, `for` and asserts.
def check_dir(dir, *, files=None, size=None):
    actual_files = set(dir.iterdir())
    if size is not None:
        assert len(actual_files) == size
    if files is not None:
        assert len(actual_files) == len(files)
        for name, content in files:
            assert dir / name in actual_files
            if content is not None:
                assert (dir / name).read_text() == content


def test_s10_check_dir_helper(tmp_path: Path = use(TmpPath)):
    i = logger.add(tmp_path / "file.log", compression="gz", delay=True)
    logger.debug("a")
    logger.remove(i)
    check_dir(tmp_path, files=[("file.log.gz", None)])
    assert tmp_path.is_dir() == True


# S11: `"7" * 20` miscounted as 19 sevens in the literal.
def test_s11_miscounted_literal(tmp_path: Path = use(TmpPath)):
    frozen = freeze_time("1999-12-12")
    logger.add(tmp_path / "file.log", rotation=30, retention=2, delay=True, format="{message}")
    for i in range(1, 10):
        time.sleep(0.05)
        frozen.tick(datetime.timedelta(seconds=0.05))
        logger.info(str(i) * 20)
    assert (tmp_path / "file.1999-12-12_00-00-00_350000.log").read_text() == "7777777777777777777\n"
    assert (tmp_path / "file.log").read_text() == "99999999999999999999\n"


# S12: a content check of a file name with a typo (`file.lg`): the reading itself raises.
def test_s12_missing_file_read(tmp_path: Path = use(TmpPath)):
    freeze_time("2001-02-03")
    i = logger.add(tmp_path / "file.log", rotation=0, delay=True, format="{message}")
    logger.debug("a")
    logger.remove(i)
    assert (tmp_path / "file.lg").read_text() == "a\n"
    assert sorted(os.listdir(tmp_path)) == ["file.2001-02-03_00-00-00_000000.log", "file.log"]
