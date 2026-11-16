"""Code under test that sends mail through a class it holds, so a test may replace the class."""

from __future__ import annotations

HOST: str = "mail.example"
PORT: int = 25
RETRIES: int = 3
TIMEOUT: float = 10.0


class SMTP:
    def send(self, to: str) -> str:
        return f"sent to {to} through {HOST}"


def notify(to: str) -> str:
    return SMTP().send(to)
