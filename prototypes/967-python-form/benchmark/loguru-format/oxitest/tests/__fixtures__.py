"""Fixtures for the loguru format tests.

Ported from loguru's tests/conftest.py (MIT, Delgan/loguru@28349be). Only
``reset_logger`` is a fixture: it must run around every test without being
asked for, and its teardown must run even when the test fails. The original
``writer`` fixture is a plain class in the test module instead, because
nothing about it needs the runner (see NOTES.md).
"""

from collections.abc import Iterator

import loguru
import loguru._logger
import oxitest as oxi


def _reset() -> None:
    loguru.logger.remove()
    # Re-run the logger's private constructor, exactly as loguru's own conftest
    # does. The public stub declares no __init__, so both checkers object.
    core = loguru._logger.Core()
    loguru.logger.__init__(core, None, 0, False, False, False, False, True, [], {})  # pyright: ignore[reportCallIssue]  # ty: ignore[too-many-positional-arguments]
    loguru._logger.context.set({})


# `Iterator[None]`, not `oxi.Yields[None]`: neither ty nor pyright accepts
# `Yields[...]` as a return annotation (see NOTES.md).
@oxi.fixture(lifetime="function", autouse=True)
def reset_logger() -> Iterator[None]:
    """Give every test a pristine global logger, and leave one behind."""
    _reset()
    yield
    _reset()
