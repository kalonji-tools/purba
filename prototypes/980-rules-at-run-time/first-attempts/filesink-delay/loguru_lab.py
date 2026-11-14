"""First attempt: loguru's conftest fixtures, ported as close to the original as possible.

Adapted from loguru's tests/conftest.py (MIT, Delgan/loguru@28349be); see LICENSE.loguru.
"""

import builtins
import contextlib
import datetime
import os
import tempfile
import time
from collections.abc import Callable, Iterator
from pathlib import Path
from unittest import mock

import freezegun
import freezegun.api

import loguru
import loguru._file_sink
import loguru._logger


def reset_logger() -> None:
    loguru.logger.remove()
    loguru.logger.__init__(  # type: ignore[misc]
        loguru._logger.Core(), None, 0, False, False, False, False, True, [], {}
    )
    loguru._logger.context.set({})


def tmp_path() -> Path:
    return Path(tempfile.mkdtemp(prefix="filesink-delay-"))


def check_dir(dir, *, files=None, size=None):
    actual_files = set(dir.iterdir())
    seen = set()
    if size is not None:
        assert len(actual_files) == size
    if files is not None:
        assert len(actual_files) == len(files)
        for name, content in files:
            filepath = dir / name
            assert filepath in actual_files
            assert filepath not in seen
            if content is not None:
                assert filepath.read_text() == content
            seen.add(filepath)


FreezeTime = Callable[..., contextlib.AbstractContextManager]


def freeze_time() -> FreezeTime:
    ctimes: dict[str, float] = {}
    freezegun_localtime = freezegun.api.fake_localtime
    builtins_open = builtins.open
    fakes = {"zone": "UTC", "offset": 0}

    def fake_localtime(t=None):
        struct = freezegun_localtime(t)
        override = {"tm_zone": fakes["zone"], "tm_gmtoff": fakes["offset"]}
        names = ["tm_year", "tm_mon", "tm_mday", "tm_hour", "tm_min", "tm_sec",
                 "tm_wday", "tm_yday", "tm_isdst", "tm_zone", "tm_gmtoff"]
        return time.struct_time([override.get(n, getattr(struct, n, None)) for n in names])

    def patched_open(filepath, *args, **kwargs):
        if not os.path.exists(filepath):
            tz = datetime.timezone(datetime.timedelta(seconds=fakes["offset"]), name=fakes["zone"])
            ctimes[filepath] = datetime.datetime.now().replace(tzinfo=tz).timestamp()
        return builtins_open(filepath, *args, **kwargs)

    @contextlib.contextmanager
    def freeze(date, timezone=("UTC", 0)) -> Iterator[freezegun.api.FrozenDateTimeFactory]:
        date = datetime.datetime.strptime(date, "%Y-%m-%d")
        zone, offset = timezone
        tz_offset = datetime.timedelta(seconds=offset)
        date = date.replace(tzinfo=datetime.timezone(tz_offset, zone))
        with contextlib.ExitStack() as stack:
            stack.enter_context(mock.patch.dict(fakes, {"zone": zone, "offset": offset}))
            stack.enter_context(mock.patch.object(loguru._file_sink, "get_ctime", ctimes.__getitem__))
            stack.enter_context(mock.patch.object(loguru._file_sink, "set_ctime", ctimes.__setitem__))
            stack.enter_context(mock.patch.object(builtins, "open", patched_open))
            stack.enter_context(mock.patch.object(freezegun.api, "fake_localtime", fake_localtime))
            with freezegun.freeze_time(date, tz_offset=tz_offset) as frozen:
                yield frozen

    return freeze
