"""PROTOTYPE (#967). Form RECORD: an experiment is a class whose fields are its inputs. Never merge.

    class SlugOf(Experiment[str]):
        text: str

        def act(self) -> str:
            return slugify(self.text)

    plain = SlugOf(text="Hi").expect(product="hi")

The class is a dataclass to a type checker (`dataclass_transform`), so it sees a misspelled or mistyped input.
`act` is the act; every other method is an assay, and reads `self.product` or one input.
A case is an instance, named by its variable. A step of a chain is that variable: `after(plain)`.
"""

import ast
import inspect
import textwrap
from types import ModuleType
from typing import Any, Generic, Self, TypeVar, dataclass_transform

from engine import REGISTRY, After, Case, Raises, Use, Warns, caller_line, use
from engine import Experiment as _Exp

__all__ = ["Experiment", "after", "use"]
T = TypeVar("T")


def _self_read(fn: Any) -> str:
    tree = ast.parse(textwrap.dedent(inspect.getsource(fn)))
    for node in ast.walk(tree):
        if isinstance(node, ast.Attribute) and isinstance(node.value, ast.Name) and node.value.id == "self":
            return node.attr
    return "self"


@dataclass_transform(kw_only_default=True)
class Experiment(Generic[T]):
    def __init_subclass__(cls) -> None:
        fields = [k for k in cls.__dict__.get("__annotations__", {}) if not k.startswith("_")]
        params = [inspect.Parameter(k, inspect.Parameter.KEYWORD_ONLY,
                                    default=cls.__dict__.get(k, inspect.Parameter.empty)) for k in fields]
        method = cls.__dict__["act"]

        def act(**kw: Any) -> Any:
            obj = object.__new__(cls)
            obj.__dict__.update(kw)
            return method(obj)

        act.__signature__ = inspect.Signature(params)  # type: ignore[attr-defined]
        act.__purba_src__ = method  # type: ignore[attr-defined]
        e = _Exp(cls.__name__, act, inspect.getsourcefile(cls) or "", inspect.getsourcelines(cls)[1])
        for k, v in cls.__dict__.items():
            if inspect.isfunction(v) and k != "act" and not k.startswith("_"):
                read = _self_read(v)

                def assay(value: Any, v: Any = v, read: str = read) -> Any:
                    obj = object.__new__(cls)
                    obj.__dict__[read] = value
                    return v(obj)

                assay.__purba_reads__ = read  # type: ignore[attr-defined]
                assay.__purba_src__ = v  # type: ignore[attr-defined]
                e.assays[k] = assay
        cls.__purba__ = e  # type: ignore[attr-defined]
        REGISTRY.append(e)

    def __init__(self, **given: Any) -> None:
        self.__dict__["__case__"] = Case("", given, {}, line=caller_line())
        self.__dict__["__case__"].exp = type(self).__purba__  # type: ignore[attr-defined]

    @property
    def product(self) -> T:
        return self.__dict__["product"]

    def expect(self, **values: Any) -> Self:
        self.__dict__["__case__"].expect.update(values)
        return self

    def raises(self, type_: type[BaseException], message: str) -> Self:
        self.__dict__["__case__"].raises = Raises(type_, message)
        return self

    def warns(self, type_: type[Warning], message: str) -> Self:
        self.__dict__["__case__"].warns = Warns(type_, message)
        return self


def after(step: Experiment[Any], input: str | None = None) -> Any:
    return After(step.__dict__["__case__"], input, caller_line())


def collect(mod: ModuleType) -> None:
    for k, v in vars(mod).items():
        if isinstance(v, Experiment):
            c = v.__dict__["__case__"]
            c.name = k
            type(v).__purba__.cases.append(c)  # type: ignore[attr-defined]
    for e in REGISTRY:
        e.cases.sort(key=lambda c: c.line)


_ = Use  # the fields of a fixture keep `use(...)` as their default
