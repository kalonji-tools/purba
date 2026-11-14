# Adapted from loguru's tests/conftest.py (MIT, Delgan/loguru@28349be); see LICENSE.loguru.
"""What loguru's conftest gives test_defaults.py: its two per-test autouse resets.

The session-scoped `check_env_variables` warning has no counterpart (the form has no session scope).
"""

import multiprocessing

import loguru


def reset_logger() -> None:
    loguru.logger.remove()
    loguru.logger.__init__(  # type: ignore[misc]
        loguru._logger.Core(), None, 0, False, False, False, False, True, [], {}
    )
    loguru._logger.context.set({})


def reset_multiprocessing_start_method() -> None:
    multiprocessing.set_start_method(None, force=True)  # type: ignore[arg-type]
