"""A toy fixtures resolver. NOT purba's design: a model of the #689 rules.

What it models (from the #689 record on PR #695):
- an input is named by the type of a parameter; the key is the annotation
  as `typing.get_type_hints(include_extras=True)` returns it;
- fixtures builds one value per key per test; a builder that names a key
  gets the value the test gets;
- two parameters of one test with the same key are refused;
- a resource lives longer than a test, and only a builder takes it.

What it adds, so each way can run:
- a key with no builder is DERIVED from the type it wraps (NewType,
  Annotated, `type X = ...`, Callable[[], T], tuple[T, ...]); a derived
  value calls the wrapped type's builder afresh, never its per-test value;
- a GENERIC builder, `def tx[D: Database](db: D) -> Tx[D]`, builds every
  `Tx[X]`: D binds to X, and the builder's `db` becomes the key X;
- at the end of a test, each per-test value with an `end()` is ended.

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
GENERIC: dict[Any, Callable[..., Any]] = {}  # origin -> builder of origin[TypeVar]
LONG: set[Any] = set()  # keys a resource builder returns
SESSION: dict[Any, Any] = {}  # values that outlive one test


@dataclass(frozen=True)
class Role:
    name: str


class Refused(Exception):
    pass


def register(fn: Callable[..., Any], long: bool) -> Callable[..., Any]:
    key = typing.get_type_hints(fn, include_extras=True)["return"]
    params = typing.get_args(key)
    if params and all(isinstance(p, typing.TypeVar) for p in params):
        GENERIC[typing.get_origin(key)] = fn
    else:
        BUILDERS[key] = fn
    if long:
        LONG.add(key)
    return fn


def fixture(fn: Callable[..., Any]) -> Callable[..., Any]:
    return register(fn, long=False)


def resource(fn: Callable[..., Any]) -> Callable[..., Any]:
    return register(fn, long=True)


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


def user_generic(key: Any) -> bool:
    """A class of the tester's, given type arguments: `Tx[SourceDb]`."""
    origin = typing.get_origin(key)
    return origin is not None and origin not in (tuple, collections.abc.Callable, typing.Annotated)


def expand(key: Any) -> set[Any]:
    """The key, and each type argument of a user generic, all the way down."""
    out = {key}
    if user_generic(key):
        for arg in typing.get_args(key):
            out |= expand(arg)
    return out


def is_long(key: Any) -> bool:
    if key in LONG:
        return True
    inner = unwrap(key)
    return inner is not None and key not in BUILDERS and is_long(inner)


def find(key: Any) -> tuple[Callable[..., Any], dict[Any, Any]] | None:
    """The builder of a key, and how its type variables bind."""
    if key in BUILDERS:
        return BUILDERS[key], {}
    if user_generic(key) and typing.get_origin(key) in GENERIC:
        fn = GENERIC[typing.get_origin(key)]
        made = typing.get_type_hints(fn, include_extras=True)["return"]
        binding = dict(zip(typing.get_args(made), typing.get_args(key), strict=True))
        for var, arg in binding.items():
            bound = var.__bound__
            if bound is not None and arg is not bound and bound not in bases(arg):
                raise Refused(f"{arg!r} is not a {bound!r}")
        return fn, binding
    return None


class Run:
    """One test's values: one per key. Resources live in SESSION.

    strict adds the refusals the prototype tests:
    - a builder that names two inputs of one key (the #689 refusal, for builders);
    - a STRAY builder: one that names a type the test names only through
      roles (or, by name, under a name the test does not use for it);
    - a test that names a resource.
    """

    def __init__(self, named: dict[str, Any], by_name: bool, strict: bool) -> None:
        self.values: dict[Any, Any] = {}
        self.by_name = by_name
        self.strict = strict
        keys = list(named.values())
        seen: set[Any] = set().union(*(expand(k) for k in keys))
        self.roles_of = set().union(*(bases(k) for k in seen)) - seen
        self.names_of = {k: {n for n, kk in named.items() if kk == k} for k in keys}

    def build(self, key: Any, name: str | None = None) -> Any:
        if is_long(key):
            if key not in SESSION:
                SESSION[key] = self.fresh(key)
            return SESSION[key]
        memo = (name, key) if self.by_name else key
        if memo not in self.values:
            self.values[memo] = self.fresh(key)
        return self.values[memo]

    def stray(self, key: Any) -> bool:
        if key in self.roles_of:
            return True
        return user_generic(key) and any(arg in self.roles_of for arg in typing.get_args(key))

    def check(self, fn: Callable[..., Any], params: dict[str, Any]) -> None:
        keys = list(params.values())
        name_of = getattr(fn, "__name__", fn)
        if not self.by_name and any(keys.count(k) > 1 for k in keys):
            raise Refused(f"builder {name_of} names two inputs of one type")
        for name, key in params.items():
            if self.stray(key):
                raise Refused(f"builder {name_of} names {key!r}, which the test names only through roles")
            names = self.names_of.get(key, set())
            if self.by_name and len(names) > 1 and name not in names:
                raise Refused(f"builder {name_of} names {key!r} as {name!r}, the test as {sorted(names)}")

    def fresh(self, key: Any) -> Any:
        found = find(key)
        if found is not None:
            fn, binding = found
            params = {n: binding.get(k, k) for n, k in inputs(fn).items()}
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

    def end(self) -> None:
        for value in reversed(list(self.values.values())):
            end = getattr(value, "end", None)
            if callable(end):
                end()


def call(
    test: Callable[..., Any], refuse: bool = True, by_name: bool = False, strict: bool = False
) -> dict[str, Any]:
    named = inputs(test)
    if refuse and not by_name:
        keys = list(named.values())
        twins = [k for k in keys if keys.count(k) > 1]
        if twins:
            raise Refused(f"two inputs of one type: {twins[0]!r}")
    if strict:
        for key in named.values():
            if is_long(key):
                raise Refused(f"the test names {key!r}, a resource that only a builder takes")
    run = Run(named, by_name, strict)
    try:
        args = {n: run.build(k, n) for n, k in named.items()}
        test(**args)
    finally:
        run.end()
    return args
