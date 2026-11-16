"""#1015, change 6: a module attribute that the act replaced is reported under the case, never
refused. The runner compares at the first check, so a replacement undone before it is not seen."""

from __future__ import annotations

from unittest import mock

import mailer


class FakeSMTP:
    def send(self, to: str) -> str:
        return f"faked to {to}"


def test_replaced_and_kept() -> None:
    """PASS, with a report. Says "⚠ the act replaced mailer.SMTP" """
    mailer.SMTP = FakeSMTP  # ty: ignore[invalid-assignment]
    sent = mailer.notify("ann")
    assert sent == "faked to ann"


def test_patched_under_a_setup() -> None:
    """PASS, with a report: the checks read under the `with`, so the patch still holds. Says "⚠ the
    act replaced mailer.SMTP" """
    with mock.patch.object(mailer, "SMTP", FakeSMTP):
        sent = mailer.notify("ann")
        assert sent == "faked to ann"


def test_patch_undone_before_the_check() -> None:
    """PASS, and no report: the patch was undone before the first check, which the runner cannot
    see. Says "✓ test_patch_undone_before_the_check › sent == 'faked to ann'" """
    with mock.patch.object(mailer, "SMTP", FakeSMTP):
        sent = mailer.notify("ann")
    assert sent == "faked to ann"


def test_many_replaced() -> None:
    """FAIL, with three names and a count of the rest; the report does not change the verdict.
    Says "⚠ the act replaced mailer.HOST" Says "⚠ and 2 more" """
    mailer.HOST, mailer.PORT, mailer.RETRIES, mailer.TIMEOUT = "smtp.test", 2525, 0, 0.5
    mailer.SMTP = FakeSMTP  # ty: ignore[invalid-assignment]
    sent = mailer.notify("ann")
    assert sent == "sent to ann through smtp.test"


def test_replaced_and_refused(loud: bool = False) -> None:
    """REFUSED: the branch goes one way; the report still shows. Says "⚠ the act replaced
    mailer.SMTP" """
    mailer.SMTP = FakeSMTP  # ty: ignore[invalid-assignment]
    sent = mailer.notify("ANN" if loud else "ann")
    assert sent == "faked to ann"


def test_nothing_replaced() -> None:
    """PASS, and no report: the act calls the code and replaces nothing."""
    sent = mailer.notify("ann")
    assert sent == "sent to ann through mail.example"
