"""Code under test with global state, as a logger keeps its handlers."""

from __future__ import annotations

NAMES: list[str] = ["builtin"]


def register(name: str) -> None:
    NAMES.append(name)


def names() -> list[str]:
    return list(NAMES)


def reset() -> None:
    NAMES.clear()
