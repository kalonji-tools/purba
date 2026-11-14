"""A minimal directory stock for the probes: a fresh temp dir, removed after the case."""

import shutil
import tempfile
from pathlib import Path

from cases import Checkpoints, Samples


class Dir(Samples[Path]):
    def sample(self) -> Path:
        return Path(tempfile.mkdtemp(prefix="probe-stock-"))

    def discard(self, value: Path, /) -> None:
        shutil.rmtree(value, ignore_errors=True)


class Both(Samples[Path], Checkpoints[Path, str]):
    """Samples and Checkpoints at once: which of sample() or open() gives the value?"""

    def sample(self) -> Path:
        return Path(tempfile.mkdtemp(prefix="probe-sample-"))

    def discard(self, value: Path, /) -> None:
        shutil.rmtree(value, ignore_errors=True)

    def open(self) -> Path:
        return Path(tempfile.mkdtemp(prefix="probe-open-"))

    def checkpoint(self, value: Path, /) -> str:
        return "mark"

    def restore(self, value: Path, mark: str, /) -> None:
        pass
