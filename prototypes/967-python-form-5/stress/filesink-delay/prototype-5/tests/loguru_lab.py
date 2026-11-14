"""The fixtures that loguru's test_filesink_delay.py takes from its conftest, in this form.

Adapted from loguru's tests/conftest.py (MIT, Delgan/loguru@28349be); see LICENSE.loguru.

- `reset_logger` is the autouse fixture, run by `before_each`. The original also resets after the
  test; here the case's process ends instead.
- `TmpPath` stands for pytest's `tmp_path`: a fresh directory for each case, removed after it.
- `freeze_time(date)` stands for the `freeze_time` fixture. It starts the freeze and never stops it:
  the case's process ends instead of the `with` block.
"""

import builtins
import datetime
import os
import shutil
import tempfile
import time
from pathlib import Path
from typing import cast
from unittest import mock

import freezegun
import freezegun.api
from freezegun.api import FrozenDateTimeFactory

import loguru
import loguru._file_sink
import loguru._logger
from cases import Checkpoints, Samples


def reset_logger() -> None:
    loguru.logger.remove()
    core = loguru._logger.Core()
    loguru.logger.__init__(core, None, 0, False, False, False, False, True, [], {})  # loguru.pyi declares no __init__ arguments  # ty: ignore[too-many-positional-arguments]  # pyright: ignore[reportCallIssue]
    loguru._logger.context.set({})


class TmpPath(Samples[Path], Checkpoints[Path, Path]):
    """pytest's `tmp_path`: an empty directory for one case, removed after it.

    It is a stock, so two checks may read it only between a checkpoint and a restore. The checks of
    this file only read the directory, but the runner cannot know that: the checkpoint copies the
    directory aside, and the restore puts the copy back.
    """

    def sample(self) -> Path:
        return Path(tempfile.mkdtemp(prefix="filesink-delay-"))

    def discard(self, value: Path, /) -> None:
        shutil.rmtree(value, ignore_errors=True)

    def open(self) -> Path:  # never called: a stock that is also `Samples` takes its value from `sample()`
        return self.sample()

    def checkpoint(self, value: Path, /) -> Path:
        mark = Path(tempfile.mkdtemp(prefix="filesink-delay-mark-"))
        shutil.copytree(value, mark, dirs_exist_ok=True)
        return mark

    def restore(self, value: Path, mark: Path, /) -> None:
        shutil.rmtree(value)
        shutil.copytree(mark, value)
        shutil.rmtree(mark)


def freeze_time(date: str, zone: str = "UTC", offset: int = 0) -> FrozenDateTimeFactory:
    """Freeze the clock at `date` ("%Y-%m-%d"), as the conftest's `freeze_time` does on entering its `with`.

    One path: the original's other date formats, aware-date refusal and `include_tm_zone` options are
    not needed by this file, so they are left out.
    """
    ctimes: dict[str, float] = {}
    freezegun_localtime = freezegun.api.fake_localtime
    builtins_open = builtins.open

    def fake_localtime(t: float | None = None) -> time.struct_time:
        struct = freezegun_localtime(t)
        names = ["tm_year", "tm_mon", "tm_mday", "tm_hour", "tm_min", "tm_sec", "tm_wday", "tm_yday", "tm_isdst"]
        return time.struct_time([*(getattr(struct, n) for n in names), zone, offset])

    def patched_open(filepath, *args, **kwargs):  # noqa: ANN001, ANN002, ANN003
        if not os.path.exists(filepath):
            tz = datetime.timezone(datetime.timedelta(seconds=offset), name=zone)
            ctimes[filepath] = datetime.datetime.now().replace(tzinfo=tz).timestamp()
        return builtins_open(filepath, *args, **kwargs)

    tz_offset = datetime.timedelta(seconds=offset)
    moment = datetime.datetime.strptime(date, "%Y-%m-%d").replace(tzinfo=datetime.timezone(tz_offset, zone))

    mock.patch.object(loguru._file_sink, "get_ctime", ctimes.__getitem__).start()
    mock.patch.object(loguru._file_sink, "set_ctime", ctimes.__setitem__).start()
    mock.patch.object(builtins, "open", patched_open).start()
    mock.patch.object(freezegun.api, "fake_localtime", fake_localtime).start()
    return cast(FrozenDateTimeFactory, freezegun.freeze_time(moment, tz_offset=tz_offset).start())
