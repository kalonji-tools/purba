"""Writes cases/<way>/: fixtures.py and test_c1..c7, from one spec.

Every test body is the same in every way; only the signatures differ.
usage: python -I -B gen.py
"""

import re
import shutil
from dataclasses import dataclass
from pathlib import Path

HERE = Path(__file__).parent

WAYS = {
    "w01_record_690": "type",
    "w03_alias": "alias",
    "w06_pytest": "name",
    "w08_decorator_map": "using",
    "w09_default_bare": "bare",
    "w10_default_untyped_marker": "any",
    "w11_default_typed_marker": "typed",
    "w12_annotated_provider": "annotated",
}
BY_TYPE = {"type", "alias", "name"}

ROLE = {"database": "Database", "source": "SourceDb", "target": "TargetDb", "engine": "EngineDb", "big_source": "SourceDb"}
ALIAS = {"database": "DbTx", "source": "SourceTx", "target": "TargetTx", "engine": "EngineTx", "big_source": "SourceTx"}
PROVIDER = {
    "database": "database",
    "source": "source",
    "target": "target",
    "engine": "each(sqlite, postgres)",
    "big_source": "big_source",
}


@dataclass(frozen=True)
class R:
    """A reagent: a Tx drawn from a stock fixture."""

    name: str
    stock: str


@dataclass(frozen=True)
class P:
    """A plain input, built by the fixture of the same name."""

    name: str
    type: str


def param(style: str, p: R | P) -> tuple[str, tuple[str, str] | None]:
    if isinstance(p, R):
        shown, prov = "Tx[Database]", PROVIDER[p.stock]
        by_type = {"type": f"Tx[{ROLE[p.stock]}]", "alias": ALIAS[p.stock], "name": shown}
    else:
        shown, prov = p.type, p.name
        by_type = {"type": shown, "alias": shown, "name": shown}
    if style in by_type:
        return f"{p.name}: {by_type[style]}", None
    if style == "using":
        return f"{p.name}: {shown}", (p.name, prov)
    if style == "bare":
        return f"{p.name}: {shown} = {prov}", None
    if style == "any":
        return f"{p.name}: {shown} = use_any({prov})", None
    if style == "typed":
        arg = f"tx, {prov}" if isinstance(p, R) else prov
        return f"{p.name}: {shown} = use({arg})", None
    if style == "annotated":
        return f"{p.name}: Annotated[{shown}, From({prov})]", None
    raise ValueError(style)


def define(style: str, name: str, params: list[R | P], ret: str, body: list[str], fixture: bool) -> str:
    rendered = [param(style, p) for p in params]
    lines = ["@fixture"] if fixture else []
    mapping = [pair for _, pair in rendered if pair]
    if mapping:
        lines.append("@using(" + ", ".join(f"{k}={v}" for k, v in mapping) + ")")
    args = ", ".join(text for text, _ in rendered)
    head = f"def {name}({args}) -> {ret}:"
    if len(head) > 100:
        head = f"def {name}(\n" + "".join(f"    {text},\n" for text, _ in rendered) + f") -> {ret}:"
    lines.append(head)
    lines += [f"    {line}" for line in body]
    return "\n".join(lines)


BIG = ['big = Database("big source")', 'big.table.extend(["a", "b", "c"])']

CASES: dict[str, list[tuple[str, list[R | P], str, list[str], bool]]] = {
    "test_c1_copy": [
        ("test_copy", [R("src", "source"), R("dst", "target")], "None",
         ['src.insert("a")', "copy(src, dst)", 'assert dst.rows() == ["a"], "the target holds what the source held"'], False),
    ],
    "test_c2_one_input": [
        ("test_insert", [R("db", "database")], "None",
         ['db.insert("a")', 'assert db.rows() == ["a"], "the row is in the table"'], False),
    ],
    "test_c3_two_types": [
        ("test_fetch", [P("client", "Client"), R("db", "database")], "None",
         ['db.insert("a")', 'assert client.fetch(db) == ["a"], "the client reads the row"'], False),
    ],
    "test_c4_five_inputs": [
        ("test_signup",
         [P("client", "Client"), R("db", "database"), P("settings", "Settings"), P("clock", "Clock"), P("mailer", "Mailer")],
         "None",
         ['client.signup(db, mailer, clock, "ada")',
          'assert settings.name == "test", "the client runs with test settings"',
          'assert db.rows() == ["ada@0"], "the user is stored"',
          'assert mailer.outbox == ["welcome ada"], "the user is welcomed"'], False),
    ],
    "test_c5_fixture_chain": [
        ("seeded", [R("src", "source")], "Seeded",
         ['src.insert("a")', 'src.insert("b")', "return Seeded(2)"], True),
        ("test_copy_seeded", [P("seeded", "Seeded"), R("src", "source"), R("dst", "target")], "None",
         ["copy(src, dst)", 'assert len(dst.rows()) == seeded.rows, "the target holds each seeded row"'], False),
    ],
    "test_c6_many_providers": [
        ("test_roundtrip", [R("engine", "engine")], "None",
         ['engine.insert("a")', 'assert engine.rows() == ["a"], "the row is in the table"'], False),
    ],
    "test_c7_override": [
        ("test_copy_all", [R("src", "big_source"), R("dst", "target")], "None",
         ["copy(src, dst)", 'assert dst.rows() == ["a", "b", "c"], "the target holds every row"'], False),
        ("test_source_is_big", [R("src", "big_source")], "None",
         ['assert len(src.rows()) == 3, "the source holds three rows"'], False),
        ("test_copy_twice", [R("src", "big_source"), R("dst", "target")], "None",
         ["copy(src, dst)", "copy(src, dst)", 'assert len(dst.rows()) == 6, "each copy appends every row"'], False),
    ],
}


def override(style: str) -> str | None:
    """C7: where the source is replaced for one module."""
    if style in ("type", "alias"):
        return define(style, "source", [], "SourceDb", [*BIG, "return SourceDb(big)"], True)
    if style == "name":
        return define(style, "source", [], "Database", [*BIG, "return big"], True)
    return None


def fixtures_module(style: str) -> str:
    out: list[str] = []
    stock = lambda name, ret, body: define(style, name, [], ret, body, True)  # noqa: E731
    if style in ("type", "alias"):
        out.append('SourceDb = NewType("SourceDb", Database)\nTargetDb = NewType("TargetDb", Database)\n'
                   'EngineDb = NewType("EngineDb", Database)')
        if style == "alias":
            out.append("type DbTx = Tx[Database]\ntype SourceTx = Tx[SourceDb]\n"
                       "type TargetTx = Tx[TargetDb]\ntype EngineTx = Tx[EngineDb]")
        out.append(stock("database", "Database", ['return Database("main")']))
        out.append(stock("source", "SourceDb", ['return SourceDb(Database("source"))']))
        out.append(stock("target", "TargetDb", ['return TargetDb(Database("target"))']))
    else:
        out.append(stock("database", "Database", ['return Database("main")']))
        out.append(stock("source", "Database", ['return Database("source")']))
        out.append(stock("target", "Database", ['return Database("target")']))
    if style in BY_TYPE:
        out.append('def sqlite() -> Database:\n    return Database("sqlite")')
        out.append('def postgres() -> Database:\n    return Database("postgres")')
        ret, value = {"type": ("EngineDb", "EngineDb(made)"), "alias": ("EngineDb", "EngineDb(made)"),
                      "name": ("Tx[Database]", "Tx(made)")}[style]
        out.append(f"@fixture\n@fixture_each(sqlite, postgres)\ndef engine(made: Database) -> {ret}:\n    return {value}")
    else:
        out.append(stock("big_source", "Database", [*BIG, "return big"]))
        out.append(stock("sqlite", "Database", ['return Database("sqlite")']))
        out.append(stock("postgres", "Database", ['return Database("postgres")']))
    if style == "name":
        for name, src in (("db", "database"), ("src", "source"), ("dst", "target")):
            out.append(f"@fixture\ndef {name}({src}: Database) -> Tx[Database]:\n    return Tx({src})")
    else:
        out.append("@fixture\ndef tx[D: Database](db: D) -> Tx[D]:\n    return Tx(db)")
    out.append(stock("settings", "Settings", ['return Settings("test")']))
    out.append(define(style, "client", [P("settings", "Settings")], "Client", ["return Client(settings)"], True))
    out.append(stock("clock", "Clock", ["return Clock(0)"]))
    out.append(stock("mailer", "Mailer", ["return Mailer()"]))
    return header(style, "\n\n\n".join(out), defined=set()) + "\n\n\n" + "\n\n\n".join(out) + "\n"


SOURCES = {
    "typing": ["Annotated", "NewType"],
    "sketch": ["From", "each", "fixture", "fixture_each", "use", "use_any", "using"],
    "subject": ["Client", "Clock", "Database", "Mailer", "Seeded", "Settings", "Tx", "copy"],
}
FROM_FIXTURES = ["SourceDb", "TargetDb", "EngineDb", "DbTx", "SourceTx", "TargetTx", "EngineTx",
                 "database", "source", "target", "big_source", "sqlite", "postgres", "tx",
                 "settings", "client", "clock", "mailer"]


def header(style: str, code: str, defined: set[str], test: bool = False) -> str:
    bare = re.sub(r'"[^"]*"', '""', code)
    bare = re.sub(r"^def \w+", "def _", bare, flags=re.M)
    used = lambda name: re.search(rf"(?<![\w.]){name}(?!\w)", bare) is not None  # noqa: E731
    lines: list[str] = []
    for module, names in SOURCES.items():
        found = [n for n in names if used(n) and n not in defined]
        if found:
            lines.append(f"from {module} import {', '.join(found)}")
    if test:
        allowed = [n for n in FROM_FIXTURES if n[0].isupper() or style not in BY_TYPE]
        found = [n for n in allowed if used(n) and n not in defined]
        if found:
            lines.insert(1 if lines and lines[0].startswith("from typing") else 0,
                         f"from fixtures import {', '.join(sorted(found))}")
    std = [line for line in lines if line.startswith("from typing")]
    rest = sorted(line for line in lines if not line.startswith("from typing"))
    return "\n".join(std + ([""] if std and rest else []) + rest)


def test_module(style: str, case: str) -> str:
    out: list[str] = []
    defined: set[str] = set()
    if case == "test_c7_override" and (fix := override(style)):
        out.append(fix)
        defined.add("source")
    for name, params, ret, body, is_fixture in CASES[case]:
        out.append(define(style, name, params, ret, body, is_fixture))
        defined.add(name)
    code = "\n\n\n".join(out)
    return header(style, code, defined, test=True) + "\n\n\n" + code + "\n"


def main() -> None:
    for way, style in WAYS.items():
        target = HERE / way
        if target.exists():
            shutil.rmtree(target)
        target.mkdir()
        for shared in ("subject.py", "sketch.py"):
            shutil.copy(HERE / "_shared" / shared, target / shared)
        (target / "fixtures.py").write_text(fixtures_module(style))
        for case in CASES:
            (target / f"{case}.py").write_text(test_module(style, case))
    print("wrote", len(WAYS), "ways x", len(CASES), "cases")


if __name__ == "__main__":
    main()
