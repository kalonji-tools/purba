# Adapted from loguru's tests/test_colorama.py (MIT, Delgan/loguru@28349be); see LICENSE.loguru.
import builtins
import os
import sys
from unittest.mock import MagicMock

import pytest
from cases import before_each, case, use
from loguru_conftest import (
    StreamFilenoException,
    StreamIsattyException,
    StreamIsattyFalse,
    StreamIsattyTrue,
    reset_logger,
    reset_multiprocessing_start_method,
)

from loguru import logger
from loguru._colorama import should_colorize, should_wrap


def clear_environment():
    os.environ.clear()


before_each(reset_logger)
before_each(reset_multiprocessing_start_method)
before_each(clear_environment)


def patch_colorama():
    ansi_to_win32_class = MagicMock()
    winapi_test = MagicMock(return_value=True)
    enable_vt_processing = MagicMock(return_value=False)
    win32 = MagicMock(winapi_test=winapi_test)
    winterm = MagicMock(enable_vt_processing=enable_vt_processing)
    colorama = MagicMock(AnsiToWin32=ansi_to_win32_class, win32=win32, winterm=winterm)
    sys.modules["colorama"] = colorama
    sys.modules["colorama.win32"] = win32
    sys.modules["colorama.winterm"] = winterm
    return colorama


def test_stream_wrapped_on_windows_if_no_vt_support(
    patched, monkeypatch=use(pytest.MonkeyPatch), patch_colorama=use(patch_colorama)
):
    stream = StreamIsattyTrue()
    with monkeypatch.context() as context:
        context.setattr(os, "name", "nt")
        context.setattr(sys, patched, stream, raising=False)
        patch_colorama.win32.winapi_test.return_value = True
        patch_colorama.winterm.enable_vt_processing.return_value = False
        logger.add(stream, colorize=True)
        assert patch_colorama.AnsiToWin32.called is True


stream_wrapped_on_windows_if_no_vt_support__stdout__ = case(
    test_stream_wrapped_on_windows_if_no_vt_support, patched="__stdout__"
)
stream_wrapped_on_windows_if_no_vt_support__stderr__ = case(
    test_stream_wrapped_on_windows_if_no_vt_support, patched="__stderr__"
)


def test_stream_not_wrapped_on_windows_if_vt_support(
    patched, monkeypatch=use(pytest.MonkeyPatch), patch_colorama=use(patch_colorama)
):
    stream = StreamIsattyTrue()
    with monkeypatch.context() as context:
        context.setattr(os, "name", "nt")
        context.setattr(sys, patched, stream, raising=False)
        patch_colorama.win32.winapi_test.return_value = True
        patch_colorama.winterm.enable_vt_processing.return_value = True
        logger.add(stream, colorize=True)
        assert patch_colorama.AnsiToWin32.called is False


stream_not_wrapped_on_windows_if_vt_support__stdout__ = case(
    test_stream_not_wrapped_on_windows_if_vt_support, patched="__stdout__"
)
stream_not_wrapped_on_windows_if_vt_support__stderr__ = case(
    test_stream_not_wrapped_on_windows_if_vt_support, patched="__stderr__"
)


def test_stream_is_none():
    assert should_colorize(None) is False


def test_is_a_tty():
    assert should_colorize(StreamIsattyTrue()) is True


def test_is_not_a_tty():
    assert should_colorize(StreamIsattyFalse()) is False


def test_is_a_tty_exception():
    assert should_colorize(StreamIsattyException()) is False


def test_pycharm_fixed(patched, expected, monkeypatch=use(pytest.MonkeyPatch)):
    stream = StreamIsattyFalse()
    with monkeypatch.context() as context:
        context.setattr(sys, patched, stream, raising=False)
        context.setitem(os.environ, "PYCHARM_HOSTED", "1")
        assert should_colorize(stream) is expected


pycharm_fixed__stdout__ = case(test_pycharm_fixed, patched="__stdout__", expected=True)
pycharm_fixed__stderr__ = case(test_pycharm_fixed, patched="__stderr__", expected=True)
pycharm_fixed_stdout = case(test_pycharm_fixed, patched="stdout", expected=True)
pycharm_fixed_stderr = case(test_pycharm_fixed, patched="stderr", expected=False)
pycharm_fixed_none = case(test_pycharm_fixed, patched="", expected=False)


def test_github_actions_fixed(patched, expected, monkeypatch=use(pytest.MonkeyPatch)):
    stream = StreamIsattyFalse()
    with monkeypatch.context() as context:
        context.setitem(os.environ, "CI", "1")
        context.setitem(os.environ, "GITHUB_ACTIONS", "1")
        context.setattr(sys, patched, stream, raising=False)
        assert should_colorize(stream) is expected


github_actions_fixed__stdout__ = case(test_github_actions_fixed, patched="__stdout__", expected=True)
github_actions_fixed__stderr__ = case(test_github_actions_fixed, patched="__stderr__", expected=True)
github_actions_fixed_stdout = case(test_github_actions_fixed, patched="stdout", expected=False)
github_actions_fixed_stderr = case(test_github_actions_fixed, patched="stderr", expected=False)
github_actions_fixed_none = case(test_github_actions_fixed, patched="", expected=False)


def test_mintty_fixed_windows(patched, expected, monkeypatch=use(pytest.MonkeyPatch)):
    stream = StreamIsattyFalse()
    with monkeypatch.context() as context:
        context.setattr(os, "name", "nt")
        context.setitem(os.environ, "TERM", "xterm")
        context.setattr(sys, patched, stream, raising=False)
        assert should_colorize(stream) is expected


mintty_fixed_windows__stdout__ = case(test_mintty_fixed_windows, patched="__stdout__", expected=True)
mintty_fixed_windows__stderr__ = case(test_mintty_fixed_windows, patched="__stderr__", expected=True)
mintty_fixed_windows_stdout = case(test_mintty_fixed_windows, patched="stdout", expected=False)
mintty_fixed_windows_stderr = case(test_mintty_fixed_windows, patched="stderr", expected=False)
mintty_fixed_windows_none = case(test_mintty_fixed_windows, patched="", expected=False)


def test_dumb_term_not_colored(patched, expected, monkeypatch=use(pytest.MonkeyPatch)):
    stream = StreamIsattyTrue()
    with monkeypatch.context() as context:
        context.setitem(os.environ, "TERM", "dumb")
        context.setattr(sys, patched, stream, raising=False)
        assert should_colorize(stream) is expected


dumb_term_not_colored__stdout__ = case(test_dumb_term_not_colored, patched="__stdout__", expected=False)
dumb_term_not_colored__stderr__ = case(test_dumb_term_not_colored, patched="__stderr__", expected=False)
dumb_term_not_colored_stdout = case(test_dumb_term_not_colored, patched="stdout", expected=True)
dumb_term_not_colored_stderr = case(test_dumb_term_not_colored, patched="stderr", expected=True)
dumb_term_not_colored_none = case(test_dumb_term_not_colored, patched="", expected=True)


def test_honor_no_color_standard(patched, no_color, expected, monkeypatch=use(pytest.MonkeyPatch)):
    stream = StreamIsattyTrue()
    with monkeypatch.context() as context:
        context.setitem(os.environ, "NO_COLOR", no_color)
        context.setattr(sys, patched, stream, raising=False)
        assert should_colorize(stream) is expected


honor_no_color_1__stdout__ = case(test_honor_no_color_standard, patched="__stdout__", no_color="1", expected=False)
honor_no_color_1__stderr__ = case(test_honor_no_color_standard, patched="__stderr__", no_color="1", expected=False)
honor_no_color_1_stdout = case(test_honor_no_color_standard, patched="stdout", no_color="1", expected=False)
honor_no_color_1_stderr = case(test_honor_no_color_standard, patched="stderr", no_color="1", expected=False)
# Only standard out and err should be affected.
honor_no_color_1_none = case(test_honor_no_color_standard, patched="", no_color="1", expected=True)
# An empty value for NO_COLOR should not be applied:
honor_no_color_empty__stdout__ = case(test_honor_no_color_standard, patched="__stdout__", no_color="", expected=True)
honor_no_color_empty__stderr__ = case(test_honor_no_color_standard, patched="__stderr__", no_color="", expected=True)
honor_no_color_empty_stdout = case(test_honor_no_color_standard, patched="stdout", no_color="", expected=True)
honor_no_color_empty_stderr = case(test_honor_no_color_standard, patched="stderr", no_color="", expected=True)
honor_no_color_empty_none = case(test_honor_no_color_standard, patched="", no_color="", expected=True)


def test_honor_force_color_standard(patched, force_color, expected, monkeypatch=use(pytest.MonkeyPatch)):
    stream = StreamIsattyFalse()
    with monkeypatch.context() as context:
        context.setitem(os.environ, "FORCE_COLOR", force_color)
        context.setattr(sys, patched, stream, raising=False)
        assert should_colorize(stream) is expected


honor_force_color_1__stdout__ = case(test_honor_force_color_standard, patched="__stdout__", force_color="1", expected=True)
honor_force_color_1__stderr__ = case(test_honor_force_color_standard, patched="__stderr__", force_color="1", expected=True)
honor_force_color_1_stdout = case(test_honor_force_color_standard, patched="stdout", force_color="1", expected=True)
honor_force_color_1_stderr = case(test_honor_force_color_standard, patched="stderr", force_color="1", expected=True)
# Only standard out and err should be affected.
honor_force_color_1_none = case(test_honor_force_color_standard, patched="", force_color="1", expected=False)
# An empty value for FORCE_COLOR should not be applied:
honor_force_color_empty__stdout__ = case(test_honor_force_color_standard, patched="__stdout__", force_color="", expected=False)
honor_force_color_empty__stderr__ = case(test_honor_force_color_standard, patched="__stderr__", force_color="", expected=False)
honor_force_color_empty_stdout = case(test_honor_force_color_standard, patched="stdout", force_color="", expected=False)
honor_force_color_empty_stderr = case(test_honor_force_color_standard, patched="stderr", force_color="", expected=False)
honor_force_color_empty_none = case(test_honor_force_color_standard, patched="", force_color="", expected=False)


def test_no_color_takes_precedence_over_force_color(monkeypatch=use(pytest.MonkeyPatch)):
    stream_tty = StreamIsattyTrue()
    stream_not_tty = StreamIsattyFalse()

    with monkeypatch.context() as context:
        context.setitem(os.environ, "NO_COLOR", "1")
        context.setitem(os.environ, "FORCE_COLOR", "1")

        context.setattr(sys, "__stderr__", stream_tty, raising=False)
        tty_colored = should_colorize(stream_tty)

        context.setattr(sys, "__stderr__", stream_not_tty, raising=False)
        not_tty_colored = should_colorize(stream_not_tty)
        assert tty_colored is False
        assert not_tty_colored is False


def test_mintty_not_fixed_linux(patched, expected, monkeypatch=use(pytest.MonkeyPatch)):
    stream = StreamIsattyFalse()
    with monkeypatch.context() as context:
        context.setitem(os.environ, "TERM", "xterm")
        context.setattr(sys, patched, stream, raising=False)
        assert should_colorize(stream) is expected


mintty_not_fixed_linux__stdout__ = case(test_mintty_not_fixed_linux, patched="__stdout__", expected=False)
mintty_not_fixed_linux__stderr__ = case(test_mintty_not_fixed_linux, patched="__stderr__", expected=False)
mintty_not_fixed_linux_stdout = case(test_mintty_not_fixed_linux, patched="stdout", expected=False)
mintty_not_fixed_linux_stderr = case(test_mintty_not_fixed_linux, patched="stderr", expected=False)
mintty_not_fixed_linux_none = case(test_mintty_not_fixed_linux, patched="", expected=False)


def test_jupyter_fixed(patched, out_class, expected, monkeypatch=use(pytest.MonkeyPatch)):
    stream = StreamIsattyFalse()

    class Shell:
        pass

    ipython = MagicMock()
    ipykernel = MagicMock()
    instance = MagicMock()
    instance.__class__ = Shell
    ipython.get_ipython.return_value = instance
    ipykernel.zmqshell.ZMQInteractiveShell = Shell
    ipykernel.iostream.OutStream = out_class

    with monkeypatch.context() as context:
        context.setattr(sys, patched, stream, raising=False)
        context.setattr(builtins, "__IPYTHON__", True, raising=False)
        context.setitem(sys.modules, "IPython", ipython)
        context.setitem(sys.modules, "ipykernel", ipykernel)
        assert should_colorize(stream) is expected


jupyter_fixed_stdout_false = case(test_jupyter_fixed, patched="stdout", out_class=StreamIsattyFalse, expected=True)
jupyter_fixed_stderr_false = case(test_jupyter_fixed, patched="stderr", out_class=StreamIsattyFalse, expected=True)
jupyter_fixed__stdout___false = case(test_jupyter_fixed, patched="__stdout__", out_class=StreamIsattyFalse, expected=False)
jupyter_fixed__stderr___false = case(test_jupyter_fixed, patched="__stderr__", out_class=StreamIsattyFalse, expected=False)
jupyter_fixed_stdout_true = case(test_jupyter_fixed, patched="stdout", out_class=StreamIsattyTrue, expected=False)
jupyter_fixed_stderr_true = case(test_jupyter_fixed, patched="stderr", out_class=StreamIsattyTrue, expected=False)
jupyter_fixed_none_false = case(test_jupyter_fixed, patched="", out_class=StreamIsattyFalse, expected=False)


def test_jupyter_missing_lib(monkeypatch=use(pytest.MonkeyPatch)):
    # Missing ipykernal so jupyter block will err, should handle gracefully
    stream = StreamIsattyFalse()
    with monkeypatch.context() as context:
        context.setattr(sys, "stdout", stream, raising=False)
        context.setattr(builtins, "__IPYTHON__", True, raising=False)
        assert should_colorize(stream) is False


def test_dont_wrap_on_linux(patched, monkeypatch=use(pytest.MonkeyPatch), patch_colorama=use(patch_colorama)):
    stream = StreamIsattyTrue()
    with monkeypatch.context() as context:
        context.setattr(sys, patched, stream, raising=False)
        wrapped = should_wrap(stream)
        assert wrapped is False
        assert patch_colorama.win32.winapi_test.called is False


dont_wrap_on_linux__stdout__ = case(test_dont_wrap_on_linux, patched="__stdout__")
dont_wrap_on_linux__stderr__ = case(test_dont_wrap_on_linux, patched="__stderr__")


def test_dont_wrap_if_not_original_stdout_or_stderr(
    patched, monkeypatch=use(pytest.MonkeyPatch), patch_colorama=use(patch_colorama)
):
    stream = StreamIsattyTrue()
    with monkeypatch.context() as context:
        context.setattr(os, "name", "nt")
        context.setattr(sys, patched, stream, raising=False)
        wrapped = should_wrap(stream)
        assert wrapped is False
        assert patch_colorama.win32.winapi_test.called is False


dont_wrap_if_not_original_stdout = case(test_dont_wrap_if_not_original_stdout_or_stderr, patched="stdout")
dont_wrap_if_not_original_stderr = case(test_dont_wrap_if_not_original_stdout_or_stderr, patched="stderr")
dont_wrap_if_not_original_none = case(test_dont_wrap_if_not_original_stdout_or_stderr, patched="")


def test_dont_wrap_if_terminal_has_vt_support(
    patched, monkeypatch=use(pytest.MonkeyPatch), patch_colorama=use(patch_colorama)
):
    stream = StreamIsattyTrue()
    with monkeypatch.context() as context:
        context.setattr(os, "name", "nt")
        context.setattr(sys, patched, stream, raising=False)
        patch_colorama.win32.winapi_test.return_value = True
        patch_colorama.winterm.enable_vt_processing.return_value = True
        wrapped = should_wrap(stream)
        assert wrapped is False
        assert patch_colorama.winterm.enable_vt_processing.called is True


dont_wrap_if_terminal_has_vt_support__stdout__ = case(test_dont_wrap_if_terminal_has_vt_support, patched="__stdout__")
dont_wrap_if_terminal_has_vt_support__stderr__ = case(test_dont_wrap_if_terminal_has_vt_support, patched="__stderr__")


def test_dont_wrap_if_winapi_false(patched, monkeypatch=use(pytest.MonkeyPatch), patch_colorama=use(patch_colorama)):
    stream = StreamIsattyTrue()
    with monkeypatch.context() as context:
        context.setattr(os, "name", "nt")
        context.setattr(sys, patched, stream, raising=False)
        patch_colorama.win32.winapi_test.return_value = False
        patch_colorama.winterm.enable_vt_processing.return_value = False
        wrapped = should_wrap(stream)
        assert wrapped is False
        assert patch_colorama.win32.winapi_test.called is True


dont_wrap_if_winapi_false__stdout__ = case(test_dont_wrap_if_winapi_false, patched="__stdout__")
dont_wrap_if_winapi_false__stderr__ = case(test_dont_wrap_if_winapi_false, patched="__stderr__")


def test_wrap_if_winapi_true_and_no_vt_support(
    patched, monkeypatch=use(pytest.MonkeyPatch), patch_colorama=use(patch_colorama)
):
    stream = StreamIsattyTrue()
    with monkeypatch.context() as context:
        context.setattr(os, "name", "nt")
        context.setattr(sys, patched, stream, raising=False)
        patch_colorama.win32.winapi_test.return_value = True
        patch_colorama.winterm.enable_vt_processing.return_value = False
        wrapped = should_wrap(stream)
        assert wrapped is True
        assert patch_colorama.winterm.enable_vt_processing.called is True
        assert patch_colorama.win32.winapi_test.called is True


wrap_if_winapi_true_and_no_vt_support__stdout__ = case(test_wrap_if_winapi_true_and_no_vt_support, patched="__stdout__")
wrap_if_winapi_true_and_no_vt_support__stderr__ = case(test_wrap_if_winapi_true_and_no_vt_support, patched="__stderr__")


def test_wrap_if_winapi_true_and_vt_check_fails(
    patched, monkeypatch=use(pytest.MonkeyPatch), patch_colorama=use(patch_colorama)
):
    stream = StreamIsattyTrue()
    with monkeypatch.context() as context:
        context.setattr(os, "name", "nt")
        context.setattr(sys, patched, stream, raising=False)
        patch_colorama.win32.winapi_test.return_value = True
        patch_colorama.winterm.enable_vt_processing.side_effect = RuntimeError
        wrapped = should_wrap(stream)
        assert wrapped is True
        assert patch_colorama.winterm.enable_vt_processing.called is True
        assert patch_colorama.win32.winapi_test.called is True


wrap_if_winapi_true_and_vt_check_fails__stdout__ = case(test_wrap_if_winapi_true_and_vt_check_fails, patched="__stdout__")
wrap_if_winapi_true_and_vt_check_fails__stderr__ = case(test_wrap_if_winapi_true_and_vt_check_fails, patched="__stderr__")


def test_wrap_if_winapi_true_and_stream_has_no_fileno(
    patched, monkeypatch=use(pytest.MonkeyPatch), patch_colorama=use(patch_colorama)
):
    stream = StreamFilenoException()
    with monkeypatch.context() as context:
        context.setattr(os, "name", "nt")
        context.setattr(sys, patched, stream, raising=False)
        patch_colorama.win32.winapi_test.return_value = True
        wrapped = should_wrap(stream)
        assert wrapped is True
        assert patch_colorama.winterm.enable_vt_processing.called is False
        assert patch_colorama.win32.winapi_test.called is True


wrap_if_winapi_true_and_stream_has_no_fileno__stdout__ = case(
    test_wrap_if_winapi_true_and_stream_has_no_fileno, patched="__stdout__"
)
wrap_if_winapi_true_and_stream_has_no_fileno__stderr__ = case(
    test_wrap_if_winapi_true_and_stream_has_no_fileno, patched="__stderr__"
)


def test_wrap_if_winapi_true_and_old_colorama_version(
    patched, monkeypatch=use(pytest.MonkeyPatch), patch_colorama=use(patch_colorama)
):
    stream = StreamIsattyTrue()
    with monkeypatch.context() as context:
        context.setattr(os, "name", "nt")
        context.setattr(sys, patched, stream, raising=False)
        patch_colorama.win32.winapi_test.return_value = True
        del patch_colorama.winterm.enable_vt_processing
        wrapped = should_wrap(stream)
        assert wrapped is True
        assert patch_colorama.win32.winapi_test.called is True


wrap_if_winapi_true_and_old_colorama_version__stdout__ = case(
    test_wrap_if_winapi_true_and_old_colorama_version, patched="__stdout__"
)
wrap_if_winapi_true_and_old_colorama_version__stderr__ = case(
    test_wrap_if_winapi_true_and_old_colorama_version, patched="__stderr__"
)
