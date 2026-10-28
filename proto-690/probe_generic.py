"""Runtime facts the generic adapter depends on. usage: python probe_generic.py"""

import platform
import sys
import typing
from typing import NewType

sys.path.insert(0, "ways")
from adapter import Tx  # noqa: E402
from subject import Database  # noqa: E402


def show(label, thunk):
    try:
        value = thunk()
    except Exception as error:  # noqa: BLE001
        value = f"{type(error).__name__}: {error}"
    print(f"{platform.python_version():8} {label:58} {value}")


SourceDb = NewType("SourceDb", Database)
TargetDb = NewType("TargetDb", Database)


def tx[D: Database](db: D) -> Tx[D]:
    return Tx(db)


show("Tx[SourceDb] == Tx[TargetDb]", lambda: Tx[SourceDb] == Tx[TargetDb])
show("Tx[SourceDb], spelled twice, is one key", lambda: Tx[SourceDb] == Tx[SourceDb] and hash(Tx[SourceDb]) == hash(Tx[SourceDb]))
show("get_type_hints(tx)['return']", lambda: typing.get_type_hints(tx)["return"])
show("the bound of tx's type parameter", lambda: tx.__type_params__[0].__bound__)
show("isinstance(value, Tx[SourceDb])", lambda: isinstance(tx(SourceDb(Database())), Tx[SourceDb]))
show("a value built by Tx(db) records its role", lambda: hasattr(tx(SourceDb(Database())), "__orig_class__"))
