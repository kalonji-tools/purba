from collections.abc import Callable
from dataclasses import dataclass
from typing import Any, cast, overload


def fixture[F: Callable[..., Any]](fn: F) -> F:
    """Marks a function that builds a value for a test."""
    return fn


def fixture_each[F: Callable[..., Any]](*makers: Callable[[], object]) -> Callable[[F], F]:
    """Builds the fixture once for each maker. Its parameter `made` takes that maker's value."""

    def mark(fn: F) -> F:
        setattr(fn, "__each__", makers)
        return fn

    return mark


def using[F: Callable[..., Any]](**fixtures: Callable[..., object]) -> Callable[[F], F]:
    """Fills each named parameter from the given fixture."""

    def mark(fn: F) -> F:
        setattr(fn, "__using__", fixtures)
        return fn

    return mark


@dataclass(frozen=True)
class Each:
    fixtures: tuple[Callable[..., object], ...]


def each[A](*fixtures: Callable[[], A]) -> Callable[[], A]:
    """Runs the test once for each fixture."""
    return cast(Callable[[], A], Each(fixtures))


@dataclass(frozen=True)
class Marker:
    fixture: object
    stock: object = None


def use_any(fixture: Callable[..., object]) -> Any:
    """Fills the parameter from the given fixture."""
    return Marker(fixture)


@overload
def use[T](fixture: Callable[[], T]) -> T: ...
@overload
def use[A, T](fixture: Callable[[A], T], stock: Callable[[], A]) -> T: ...
def use(fixture: Callable[..., Any], stock: Callable[[], Any] | None = None) -> Any:
    """Fills the parameter from the given fixture, given the value of `stock`."""
    return Marker(fixture, stock)


@dataclass(frozen=True)
class From:
    """Fills the parameter from the given fixture."""

    fixture: Callable[..., object]
