"""Mirrors the shape of loguru's tests/conftest.py `parse`:
a plain shared helper whose `if` goes one way per call."""


def parse(text, *, strip=False):
    tokens = text.replace("<red>", "\x1b[31m").replace("</red>", "\x1b[0m")
    if strip:
        return tokens.replace("\x1b[31m", "").replace("\x1b[0m", "")
    return tokens
