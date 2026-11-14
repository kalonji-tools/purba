"""PROTOTYPE (#967). Form RECORD_STRICT: the form agreed on 2026-11-14. Never merge.

One form, two levels. A small test subclasses `Experiment[P]` and writes no `expect`: purba checks each
expected value against its assay's return type before the run. A test grows by subclassing
`TypedExperiment[P]` instead and declaring `expect`, and then the editor sees the expected values too:

    class AmountOf(TypedExperiment[Money]):
        text: str                                # the inputs are fields
        currency: str | None

        def act(self) -> Money:                  # abstract in the base: a class without it is refused
            return parse_amount(self.text, self.currency)

        def cents(self) -> int:                  # every other method is an assay
            return self.product.cents

        def expect(self, *, cents: int = UNREAD) -> Self:   # one typed keyword for each assay
            return self

    euros = AmountOf(text="12.50", currency="EUR").expect(cents=1250)
    later = Spent(money=euros.product)           # a typed chain: a step's product, or one of its inputs

A type checker sees each case: a misspelled or mistyped input, a misspelled or mistyped expected value,
a missing value for an assay that reads an input, a missing `act`, and a chain that feeds the wrong type.
purba's parse refuses what a type checker does not see: see `refusals` below.
"""

import ast
import inspect
import textwrap
import types
import typing
from abc import ABC, abstractmethod
from types import ModuleType
from typing import TYPE_CHECKING, Any, Generic, Self, TypeVar, dataclass_transform

from engine import REGISTRY, After, Case, Raises, Warns, caller_line, use
from engine import Experiment as _Exp

__all__ = ["UNREAD", "Experiment", "TypedExperiment", "use"]
T = TypeVar("T")
UNREAD: Any = object()  # no value: only a case that raises may leave an assay of the product unread
LITERAL_TYPES = (str, int, float, bool, bytes, type(None))


def _self_read(fn: Any) -> str:
    tree = ast.parse(textwrap.dedent(inspect.getsource(fn)))
    for node in ast.walk(tree):
        if isinstance(node, ast.Attribute) and isinstance(node.value, ast.Name) and node.value.id == "self":
            return node.attr
    return "self"


def _literal_type(tp: Any) -> bool:
    """A type whose values a literal can write: str, int, … and the collections of them."""
    if tp in LITERAL_TYPES:
        return True
    origin = typing.get_origin(tp)
    if origin in (list, tuple, set, frozenset, dict, typing.Union, types.UnionType):
        return all(_literal_type(a) for a in typing.get_args(tp) if a is not Ellipsis)
    return False


def _name(tp: Any) -> str:
    if typing.get_origin(tp) is None and hasattr(tp, "__name__"):
        return tp.__name__
    return str(tp).replace("typing.", "")


@dataclass_transform(kw_only_default=True)
class _Base(ABC, Generic[T]):
    def __init_subclass__(cls) -> None:
        if cls.__module__ == __name__:  # the two levels themselves
            return
        parents = [b for b in cls.__mro__[1:] if isinstance(b, type) and issubclass(b, _Base) and b.__module__ != __name__]
        notes: dict[str, Any] = {}
        for b in [*reversed(parents), cls]:
            notes.update(b.__dict__.get("__annotations__", {}))
        fields = [k for k in notes if not k.startswith("_")]
        params = [inspect.Parameter(k, inspect.Parameter.KEYWORD_ONLY,
                                    default=getattr(cls, k, inspect.Parameter.empty)) for k in fields]
        method = cls.__dict__.get("act") or (getattr(cls, "act") if parents else None)
        refused: list[str] = []
        if parents:
            refused.append(f"`{cls.__name__}` subclasses the experiment `{parents[0].__name__}`, so its act and"
                           " assays sit in another class: give the shared value as a field default")
        if method is None:
            refused.append("the experiment has no `act`")
            method = lambda self: None  # noqa: E731
        cls.__purba_made__ = []  # type: ignore[attr-defined]

        def act(**kw: Any) -> Any:
            obj = object.__new__(cls)
            obj.__dict__.update(kw)
            return method(obj)

        act.__signature__ = inspect.Signature(params)  # type: ignore[attr-defined]
        act.__purba_src__ = method  # type: ignore[attr-defined]
        e = _Exp(cls.__name__, act, inspect.getsourcefile(cls) or "", inspect.getsourcelines(cls)[1])
        e.refused += refused
        for k, v in cls.__dict__.items():
            if inspect.isfunction(v) and k not in ("act", "expect") and not k.startswith("_"):
                read = _self_read(v)

                def assay(value: Any, v: Any = v, read: str = read) -> Any:
                    obj = object.__new__(cls)
                    obj.__dict__[read] = value
                    return v(obj)

                assay.__purba_reads__ = read  # type: ignore[attr-defined]
                assay.__purba_src__ = v  # type: ignore[attr-defined]
                assay.__purba_method__ = v  # type: ignore[attr-defined]
                e.assays[k] = assay
        for k in sorted(set(fields) & set(e.assays)):
            e.refused.append(f"`{k}` is a field and an assay: name the assay for its question")
        typed = issubclass(cls, TypedExperiment)
        if typed and not parents:  # a subclass is refused already, and its `expect` sits in the parent
            _check_expect(cls, e)
        if not typed and "expect" in cls.__dict__:
            e.refused.append("the experiment declares `expect`, so it subclasses `TypedExperiment`, not `Experiment`")
        cls.__purba__ = e  # type: ignore[attr-defined]
        cls.__purba_fields__ = set(fields)  # type: ignore[attr-defined]
        if typed:
            setattr(cls, "expect", _record_expect)  # setattr: a type checker must not see a base `expect`
        REGISTRY.append(e)

    def __init__(self, **given: Any) -> None:
        self.__dict__["__case__"] = Case("", given, {}, line=caller_line())
        self.__dict__["__case__"].exp = type(self).__purba__  # type: ignore[attr-defined]
        type(self).__purba_made__.append(self.__dict__["__case__"])  # type: ignore[attr-defined]

    @abstractmethod
    def act(self) -> T: ...

    @property
    def product(self) -> T:
        return self.__dict__["product"]

    def raises(self, type_: type[BaseException], message: str) -> Self:
        self.__dict__["__case__"].raises = Raises(type_, message)
        return self

    def warns(self, type_: type[Warning], message: str) -> Self:
        self.__dict__["__case__"].warns = Warns(type_, message)
        return self

    if not TYPE_CHECKING:  # hidden from type checkers, so a misspelled attribute stays an error

        def __getattribute__(self, name: str) -> Any:
            """On a case, `case.product` and `case.<field>` name a step of a chain."""
            d = object.__getattribute__(self, "__dict__")
            if "__case__" in d:
                if name == "product":
                    return After(d["__case__"], None, caller_line())
                if name in type(self).__purba_fields__:
                    return After(d["__case__"], name, caller_line())
            return object.__getattribute__(self, name)


class Experiment(_Base[T]):
    """The small level: `expect` takes any keyword. purba checks each value against its assay's type before the run."""

    def expect(self, **values: Any) -> Self:
        return _record_expect(self, **values)


class TypedExperiment(_Base[T]):
    """The grown level: the experiment declares `expect`, one typed keyword for each assay, so the editor sees each value."""


def _fits(value: Any, tp: Any) -> bool:
    """Does a literal fit a type, by type all the way down: True does not fit int."""
    origin, args = typing.get_origin(tp), typing.get_args(tp)
    if origin is None:
        return type(value) is (type(None) if tp is None else tp)
    if origin in (list, set, frozenset):
        return type(value) is origin and all(_fits(v, args[0]) for v in value)
    if origin is tuple:
        if len(args) == 2 and args[1] is Ellipsis:
            return type(value) is tuple and all(_fits(v, args[0]) for v in value)
        return type(value) is tuple and len(value) == len(args) and all(_fits(v, a) for v, a in zip(value, args))
    if origin is dict:
        return type(value) is dict and all(_fits(k, args[0]) and _fits(v, args[1]) for k, v in value.items())
    if origin in (typing.Union, types.UnionType):
        return any(_fits(value, a) for a in args)
    return False


def _check_values(cls: Any) -> None:
    """Before the run, at either level: each expected value fits the type its assay hands."""
    e = cls.__purba__
    product_t = typing.get_args(cls.__orig_bases__[0])[0] if getattr(cls, "__orig_bases__", None) else Any
    types_of = {k: typing.get_type_hints(fn.__purba_method__).get("return") for k, fn in e.assays.items()}
    types_of["product"] = product_t
    for c in e.cases:
        for k, v in c.expect.items():
            tp = types_of.get(k)
            if tp is None or tp is Any:
                continue  # an unknown assay is refused elsewhere
            if not _literal_type(tp):
                e.refused.append(f"assay `{k}` hands a {_name(tp)}: an expected value is a literal,"
                                 " so hand a str, int, … or a collection of them")
            elif not _fits(v, tp):
                e.refused.append(f"case `{c.name}` (line {c.line}) expects `{k}` = {v!r}, a {type(v).__name__},"
                                 f" and the assay hands {_name(tp)}")


def _record_expect(self: Any, **values: Any) -> Any:
    case = self.__dict__["__case__"]
    case.expect.update({k: v for k, v in values.items() if v is not UNREAD})
    return self


def _check_expect(cls: type, e: _Exp) -> None:
    decl = cls.__dict__.get("expect")
    if decl is None:
        e.refused.append("the experiment declares no `expect`: one typed keyword for each assay")
        return
    product_t = typing.get_args(cls.__orig_bases__[0])[0]  # type: ignore[attr-defined]
    hints = typing.get_type_hints(decl)
    sig = inspect.signature(decl)
    keys = {k: p for k, p in sig.parameters.items() if k != "self"}
    assays: dict[str, tuple[str, Any]] = {k: (fn.__purba_reads__, typing.get_type_hints(fn.__purba_method__).get("return"))
                                          for k, fn in e.assays.items()}
    if "product" in keys:
        assays["product"] = ("product", product_t)
    for k, p in keys.items():
        if p.kind is not inspect.Parameter.KEYWORD_ONLY:
            e.refused.append(f"`expect` takes `{k}` by position: each value is a keyword, after `*`")
        if k not in assays:
            e.refused.append(f"`expect` names `{k}`, which is no assay")
            continue
        read, ret = assays[k]
        if hints.get(k) != ret:
            e.refused.append(f"`expect` types `{k}` as {_name(hints.get(k))}, and the assay hands {_name(ret)}")
        elif not _literal_type(ret):
            e.refused.append(f"assay `{k}` hands a {_name(ret)}: an expected value is a literal, so hand a str, int, … or a collection of them")
        if read == "product" and p.default is not UNREAD:
            e.refused.append(f"`expect` must default `{k}` to UNREAD: it reads the product, and a case that raises has none")
        if read != "product" and p.default is not inspect.Parameter.empty:
            e.refused.append(f"`expect` gives `{k}` a default: it reads an input, so every case gives it a value")
    for k in e.assays:
        if k not in keys:
            e.refused.append(f"`expect` has no keyword for the assay `{k}`")


def collect(mod: ModuleType) -> None:
    for k, v in vars(mod).items():
        if isinstance(v, _Base):
            c = v.__dict__["__case__"]
            c.name = k
            type(v).__purba__.cases.append(c)  # type: ignore[attr-defined]
            for key, val in c.given.items():
                if inspect.ismethod(val) and isinstance(val.__self__, _Base):
                    type(v).__purba__.refused.append(  # type: ignore[attr-defined]
                        f"case `{k}` (line {c.line}) gives `{key}` the assay `{val.__name__}` of a case:"
                        " a chain takes the step's `.product` or one of its fields")
    for e in REGISTRY:
        e.cases.sort(key=lambda c: c.line)
    for v in vars(mod).values():
        if isinstance(v, type) and issubclass(v, _Base) and v.__module__ == mod.__name__:
            named = {id(c) for c in v.__purba__.cases}  # type: ignore[attr-defined]
            lines = sorted({c.line for c in v.__purba_made__ if id(c) not in named})  # type: ignore[attr-defined]
            if lines:
                v.__purba__.refused.append(  # type: ignore[attr-defined]
                    f"line {', '.join(map(str, lines))} builds cases with no name, as a loop or a list does:"
                    " each case is one line that names it")
    _refuse_a_name_bound_twice(mod)
    for v in vars(mod).values():
        if isinstance(v, type) and issubclass(v, _Base) and v.__module__ == mod.__name__:
            _check_values(v)


def _refuse_a_name_bound_twice(mod: ModuleType) -> None:
    """A case whose name the file binds again is lost without a word, so purba refuses it."""
    classes = {k: v for k, v in vars(mod).items() if isinstance(v, type) and issubclass(v, _Base) and v.__module__ != __name__}
    tree = ast.parse(open(mod.__file__ or "", encoding="utf-8").read())
    seen: dict[str, list[tuple[int, str]]] = {}
    for node in tree.body:
        if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
            root = node.value
            while isinstance(root, ast.Call) and isinstance(root.func, ast.Attribute) and isinstance(root.func.value, ast.Call):
                root = root.func.value
            if isinstance(root, ast.Call) and isinstance(root.func, ast.Name) and root.func.id in classes:
                seen.setdefault(node.targets[0].id, []).append((node.lineno, root.func.id))
    for name, hits in seen.items():
        if len(hits) > 1:
            lines = " and ".join(str(ln) for ln, _ in hits)
            for _, cls in hits:
                classes[cls].__purba__.refused.append(  # type: ignore[attr-defined]
                    f"the case name `{name}` is bound {len(hits)} times in this file, at lines {lines}: one case is lost")
