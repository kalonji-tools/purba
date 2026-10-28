"""A stub of a public surface purba does not have. Every name here is a sketch.

Nothing resolves: each helper only records what a resolver would read, so the
probe can print what the signature hands purba at runtime.
"""

from collections.abc import Callable
from dataclasses import dataclass
from typing import Any, cast


def fixture[F: Callable[..., Any]](fn: F) -> F:
    return fn


def using[F: Callable[..., Any]](**providers: Callable[..., object]) -> Callable[[F], F]:
    """Ward's @using, pytest-lazy-fixtures' lf(): parameter name -> provider."""

    def mark(fn: F) -> F:
        fn.__purba_using__ = providers  # type: ignore[attr-defined]
        return fn

    return mark


@dataclass(frozen=True)
class Marker:
    provider: object
    arg: object = None


def use_any(provider: Callable[..., object]) -> Any:
    """FastAPI's `= Depends(fn)`: typed Any, so any annotation accepts it."""
    return Marker(provider)


def use[A, T](fn: Callable[[A], T], stock: Callable[[], A]) -> T:
    """A typed default: the checker infers T from the fixtures it names."""
    return cast(T, Marker(fn, stock))


@dataclass(frozen=True)
class From:
    """FastAPI's Depends inside Annotated: the provider as metadata."""

    provider: Callable[..., object]


@dataclass(frozen=True)
class Role:
    """dishka's FromComponent, wireup's qualifier: a role as a marker object."""

    name: str


class _Provide:
    def __class_getitem__(cls, provider: object) -> Marker:
        return Marker(provider)


Provide: Any = _Provide  # dependency-injector's Provide[Container.x], typed Any


class Fixtures:
    """svcs's container: a locator the test asks in its body."""

    def get[T](self, key: type[T]) -> T:
        raise NotImplementedError
