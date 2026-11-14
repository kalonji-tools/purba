# Adapted from loguru's tests/conftest.py (MIT, Delgan/loguru@28349be); see LICENSE.loguru.
# Only what tests/test_colorama.py uses: the stub streams and the autouse resets.
import io
import multiprocessing

import loguru


class StubStream(io.StringIO):
    def fileno(self):
        return 1


class StreamIsattyTrue(StubStream):
    def isatty(self):
        return True


class StreamIsattyFalse(StubStream):
    def isatty(self):
        return False


class StreamIsattyException(StubStream):
    def isatty(self):
        raise RuntimeError


class StreamFilenoException(StreamIsattyTrue):
    def fileno(self):
        raise RuntimeError


def reset_logger():
    """conftest's autouse `reset_logger`, setup half only: the case's process ends after it."""
    loguru.logger.remove()
    loguru.logger.__init__(
        loguru._logger.Core(), None, 0, False, False, False, False, True, [], {}
    )
    loguru._logger.context.set({})


def reset_multiprocessing_start_method():
    """conftest's autouse `reset_multiprocessing_start_method`, setup half only."""
    multiprocessing.set_start_method(None, force=True)
