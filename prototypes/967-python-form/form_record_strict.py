"""PROTOTYPE (#967). Form RECORD_STRICT: the form agreed on 2026-11-14. Never merge.

    class AmountOf(Experiment[Money]):
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

__all__ = ["UNREAD", "Experiment", "use"]
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
class Experiment(ABC, Generic[T]):
    def __init_subclass__(cls) -> None:
        parents = [b for b in cls.__mro__[1:] if b is not Experiment and isinstance(b, type) and issubclass(b, Experiment)]
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
        if not parents:  # a subclass is refused already, and its `expect` sits in the parent
            _check_expect(cls, e)
        cls.__purba__ = e  # type: ignore[attr-defined]
        cls.__purba_fields__ = set(fields)  # type: ignore[attr-defined]
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
        if isinstance(v, Experiment):
            c = v.__dict__["__case__"]
            c.name = k
            type(v).__purba__.cases.append(c)  # type: ignore[attr-defined]
            for key, val in c.given.items():
                if inspect.ismethod(val) and isinstance(val.__self__, Experiment):
                    type(v).__purba__.refused.append(  # type: ignore[attr-defined]
                        f"case `{k}` (line {c.line}) gives `{key}` the assay `{val.__name__}` of a case:"
                        " a chain takes the step's `.product` or one of its fields")
    for e in REGISTRY:
        e.cases.sort(key=lambda c: c.line)
    for v in vars(mod).values():
        if isinstance(v, type) and issubclass(v, Experiment) and v is not Experiment and v.__module__ == mod.__name__:
            named = {id(c) for c in v.__purba__.cases}  # type: ignore[attr-defined]
            lines = sorted({c.line for c in v.__purba_made__ if id(c) not in named})  # type: ignore[attr-defined]
            if lines:
                v.__purba__.refused.append(  # type: ignore[attr-defined]
                    f"line {', '.join(map(str, lines))} builds cases with no name, as a loop or a list does:"
                    " each case is one line that names it")
    _refuse_a_name_bound_twice(mod)


def _refuse_a_name_bound_twice(mod: ModuleType) -> None:
    """A case whose name the file binds again is lost without a word, so purba refuses it."""
    classes = {k: v for k, v in vars(mod).items() if isinstance(v, type) and issubclass(v, Experiment) and v is not Experiment}
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
