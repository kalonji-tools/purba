# Port notes: test_add_option_format.py → oxitest 4.0.0

All 8 tests and every case are ported with the same checks. oxitest ran 20 items, all passed under `strict = "abort"`; pytest also runs 20. ty and pyright are clean. Two oxitest problems forced workarounds; one behaviour is looser than pytest.

## Test map

| Original | Port | Cases |
|---|---|---|
| `test_format` (6) | `test_format`, `FormatCase` | `message_field`, `no_fields`, `level_twice`, `level_attributes`, `function_constant`, `function_reads_record` |
| `test_progressive_format` | same name | — |
| `test_function_format_without_exception` | same name | — |
| `test_function_format_with_exception` | same name | — |
| `test_invalid_format` (3) | `test_invalid_format`, `InvalidFormatCase` | `negative_int`, `float`, `plain_object` |
| `test_invalid_markups` (5) | `test_invalid_markups`, `MarkupCase` | `unclosed_tag`, `close_without_open`, `close_before_open`, `bare_close`, `unknown_tag` |
| `test_markup_in_field` (2) | `test_markup_in_field`, `ColorizeCase` | `colorized`, `plain` |
| `test_invalid_format_builtin` | same name | — |
| conftest `reset_logger` | autouse fixture in `tests/__fixtures__.py` | |
| conftest `writer` | plain `Writer` class | |

## Forced changes and workarounds

1. **Stub-only types fail at collection.** oxitest calls `get_type_hints()` on every parametrize case class. `loguru.FormatFunction` exists only in loguru's `.pyi`, so importing it under `TYPE_CHECKING` raises `NameError`. Fix: a runtime stand-in in an `else:` branch; checkers still see the real alias.
2. **`oxi.Yields[None]` fails both checkers.** ty reports `invalid-type-form` and pyright reports `reportInvalidTypeForm`. Fix: `Iterator[None]`, which oxitest's own how-to examples use.
3. **ty skips mypy-style `# type: ignore[code]` comments.** The two deliberate type errors (private constructor, `format=<object>`) carry per-checker suppressions; pyright needs two rules there.

## Choices that are not forced

- **No `Fixture[T]` injection.** oxitest's use-fixtures guide says to skip a fixture unless one of four conditions holds. `writer` meets none of them, so each test builds a `Writer()`.
- The parameter `format` is renamed `fmt`. The last test needs the builtin `format`, and the new name avoids shadowing it.
- `1 / 0  # noqa: B018` became `_ = 1 / 0`, because pyright flags an unused expression. It raises the same error.
- `import loguru._logger` is explicit, because both checkers reject the implicit submodule access.

## Stricter or looser than the original

- **Stricter:** strict mode aborts the run on any assert without a message. Every assert now has one (probe: exit 3). Case fields are typed, and both type checkers pass.
- **Same:** `oxi.raises(match=...)` uses `re.search`, as pytest does.
- **Looser:** an exception in `reset_logger`'s teardown is only a printed notice. All tests still pass and the run exits 0. pytest would report an error (probed).

Probes on copies: a raising autouse setup errors all 20 tests; a teardown marker is written 20 times; a wrong expected value and a wrong regex give 2 failed, 18 passed.
