"""A toy fixtures resolver. NOT purba's design: a model of the #689 rules.

What it models (from the #689 record on PR #695):
- an input is named by the type of a parameter; the key is the annotation
  as `typing.get_type_hints(include_extras=True)` returns it;
- fixtures builds one value per key per test; a builder that names a key
  gets the value the test gets;
- two parameters of one test with the same key are refused.

What it adds, so each way can run:
- a key with no builder is DERIVED from the type it wraps (NewType,
  Annotated, `type X = ...`, Callable[[], T], tuple[T, ...]); a derived
  value calls the wrapped type's builder afresh, never its per-test value.

The parts a prototype can falsify are not here. They are the Python
runtime (what get_type_hints returns) and the type checkers (runs/).
"""

from __future__ import annotations

import collections.abc
import inspect
import typing
from dataclasses import dataclass
from typing import Any, Callable, TypeAliasType

BUILDERS: dict[Any, Callable[..., Any]] = {}


@dataclass(frozen=True)
class Role:
    name: str


class Refused(Exception):
    pass


def fixture(fn: Callable[..., Any]) -> Callable[..., Any]:
    BUILDERS[typing.get_type_hints(fn, include_extras=True)["return"]] = fn
    return fn


def inputs(fn: Callable[..., Any]) -> dict[str, Any]:
    hints = typing.get_type_hints(fn, include_extras=True)
    return {
        name: hints[name]
        for name, param in inspect.signature(fn).parameters.items()
        if param.default is inspect.Parameter.empty
    }


def unwrap(key: Any) -> Any | None:
    """The type a key wraps, or None. Each branch is a runtime fact."""
    if hasattr(key, "__supertype__"):  # NewType
        return key.__supertype__
    if typing.get_origin(key) is typing.Annotated:
        return key.__origin__
    if isinstance(key, TypeAliasType):  # `type X = Y`
        return key.__value__
    return None


def bases(key: Any) -> set[Any]:
    """Every type a key stands for, other than itself."""
    out: set[Any] = set()
    inner = unwrap(key)
    while inner is not None:
        out.add(inner)
        key, inner = inner, unwrap(inner)
    origin, members = typing.get_origin(key), typing.get_args(key)
    if origin is collections.abc.Callable and members[0] == []:
        out.add(members[1])
    elif origin is tuple:
        out.update(members)
    elif origin is None and isinstance(key, type):
        out.update(base for base in key.__mro__[1:] if base is not object)
    return out


class Run:
    """One test's values: one per key.

    strict adds two refusals the prototype tests:
    - a builder that names two inputs of one key (the #689 refusal, for builders);
    - a STRAY builder: one that names a type the test names only through
      roles (or, by name, under a name the test does not use for it).
    """

    def __init__(self, named: dict[str, Any], by_name: bool, strict: bool) -> None:
        self.values: dict[Any, Any] = {}
        self.by_name = by_name
        self.strict = strict
        keys = list(named.values())
        self.roles_of = set().union(*(bases(k) for k in keys)) - set(keys)
        self.names_of = {k: {n for n, kk in named.items() if kk == k} for k in keys}

    def build(self, key: Any, name: str | None = None) -> Any:
        memo = (name, key) if self.by_name else key
        if memo not in self.values:
            self.values[memo] = self.fresh(key)
        return self.values[memo]

    def check(self, fn: Callable[..., Any], params: dict[str, Any]) -> None:
        keys = list(params.values())
        if not self.by_name and any(keys.count(k) > 1 for k in keys):
            raise Refused(f"builder {getattr(fn, "__name__", fn)} names two inputs of one type")
        for name, key in params.items():
            if key in self.roles_of:
                raise Refused(f"builder {getattr(fn, "__name__", fn)} names {key!r}, which the test names only through roles")
            names = self.names_of.get(key, set())
            if self.by_name and len(names) > 1 and name not in names:
                raise Refused(f"builder {getattr(fn, "__name__", fn)} names {key!r} as {name!r}, the test as {sorted(names)}")

    def fresh(self, key: Any) -> Any:
        if key in BUILDERS:
            fn = BUILDERS[key]
            params = inputs(fn)
            if self.strict:
                self.check(fn, params)
            return fn(**{n: self.build(k, n) for n, k in params.items()})
        inner = unwrap(key)
        if inner is not None:
            return self.fresh(inner)
        origin, members = typing.get_origin(key), typing.get_args(key)
        if origin is collections.abc.Callable and members[0] == []:
            return lambda: self.fresh(members[1])
        if origin is tuple:
            return tuple(self.fresh(member) for member in members)
        raise Refused(f"no builder for {key!r}")


def call(
    test: Callable[..., Any], refuse: bool = True, by_name: bool = False, strict: bool = False
) -> dict[str, Any]:
    named = inputs(test)
    if refuse and not by_name:
        keys = list(named.values())
        twins = [k for k in keys if keys.count(k) > 1]
        if twins:
            raise Refused(f"two inputs of one type: {twins[0]!r}")
    run = Run(named, by_name, strict)
    args = {n: run.build(k, n) for n, k in named.items()}
    test(**args)
    return args
