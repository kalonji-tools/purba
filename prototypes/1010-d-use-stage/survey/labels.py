"""Hand labels, one per fixture, read from each fixture's source (dump.py output).

class: i builds one object · ii builds with configuration of a test double or plain data (ORM rows
count as data)
       · iii calls the code under test to change state (an act) · iv yields, or registers a
       finaliser, with a teardown
strict: the fixture calls a business operation of the code under test (an act a later test builds
on) broad:  the fixture calls any method of the code under test beyond the constructor
(config.update, a route,
        a context push, a model setter such as set_country, a read such as fetch_checkout_info)
Keys are (project, file tail, line)."""

L = {
    # flask (tests/ and examples/)
    ("flask", "examples/javascript/tests/conftest.py", 7): (
        "iv",
        0,
        0,
        "app.testing = True; yield; reset",
    ),
    ("flask", "examples/javascript/tests/conftest.py", 14): ("i", 0, 0, "app.test_client()"),
    ("flask", "examples/tutorial/tests/conftest.py", 16): (
        "iv",
        1,
        1,
        "create_app, init_db, insert rows; yield; unlink",
    ),
    ("flask", "examples/tutorial/tests/conftest.py", 36): ("i", 0, 0, "app.test_client()"),
    ("flask", "examples/tutorial/tests/conftest.py", 42): ("i", 0, 0, "app.test_cli_runner()"),
    ("flask", "examples/tutorial/tests/conftest.py", 61): ("i", 0, 0, "AuthActions(client)"),
    ("flask", "tests/conftest.py", 11): ("iv", 0, 0, "monkeypatch.delenv; yield"),
    ("flask", "tests/conftest.py", 28): ("ii", 0, 1, "Flask(...); app.config.update(...)"),
    ("flask", "tests/conftest.py", 38): ("iv", 0, 1, "with app.app_context(): yield"),
    ("flask", "tests/conftest.py", 44): ("iv", 0, 1, "with app.test_request_context(): yield"),
    ("flask", "tests/conftest.py", 50): ("i", 0, 0, "app.test_client()"),
    ("flask", "tests/conftest.py", 55): ("iv", 0, 0, "sys.path; yield; purge modules"),
    ("flask", "tests/conftest.py", 68): ("iv", 0, 0, "autouse; yield; assert no leaked context"),
    ("flask", "tests/conftest.py", 83): ("ii", 0, 0, "mkdir; syspath_prepend"),
    ("flask", "tests/conftest.py", 92): ("ii", 0, 0, "setattr sys.prefix"),
    ("flask", "tests/conftest.py", 98): ("ii", 0, 0, "fake site-packages"),
    ("flask", "tests/conftest.py", 108): ("iv", 0, 0, "returns a closure that adds a finaliser"),
    ("flask", "tests/test_async.py", 41): ("ii", 0, 1, "Flask + routes + blueprint"),
    ("flask", "tests/test_cli.py", 36): ("i", 0, 0, "CliRunner()"),
    ("flask", "tests/test_cli.py", 447): ("ii", 0, 1, "Flask + add_url_rule x2"),
    ("flask", "tests/test_cli.py", 458): ("ii", 0, 0, "FlaskGroup + partial"),
    ("flask", "tests/test_logging.py", 13): ("iv", 0, 0, "autouse; reset logging; yield; restore"),
    ("flask", "tests/test_user_error_handler.py", 224): (
        "ii",
        0,
        1,
        "app(app): four routes + config",
    ),
    # loguru (tests/)
    ("loguru", "tests/conftest.py", 142): ("ii", 0, 0, "autouse guard: warns on LOGURU_ env"),
    ("loguru", "tests/conftest.py", 154): (
        "iv",
        0,
        1,
        "autouse; logger.remove + re-init; yield; again",
    ),
    ("loguru", "tests/conftest.py", 168): ("ii", 0, 0, "a sink double with written/read/clear"),
    ("loguru", "tests/conftest.py", 180): ("i", 0, 0, "returns a class"),
    ("loguru", "tests/conftest.py", 194): ("ii", 0, 0, "returns a freeze_time double"),
    ("loguru", "tests/conftest.py", 338): ("iv", 0, 0, "yield from a parametrised patch"),
    ("loguru", "tests/conftest.py", 344): ("iv", 0, 0, "patch get_frame; yield"),
    ("loguru", "tests/conftest.py", 368): ("iv", 0, 0, "autouse; reset start method; yield"),
    ("loguru", "tests/conftest.py", 47): ("i", 0, 0, "pathlib.Path(str(tmp_path))"),
    ("loguru", "tests/test_colorama.py", 20): (
        "iv",
        0,
        0,
        "autouse; clear environ; yield; restore",
    ),
    ("loguru", "tests/test_colorama.py", 28): (
        "iv",
        0,
        0,
        "MagicMock colorama in sys.modules; yield",
    ),
    ("loguru", "tests/test_filesink_permissions.py", 10): ("iv", 0, 0, "autouse; umask; yield"),
    ("loguru", "tests/test_filesink_rotation.py", 18): (
        "iv",
        0,
        0,
        "temp dir; yield; logger.remove",
    ),
    ("loguru", "tests/test_locks.py", 32): ("iv", 0, 0, "yield; gc.collect"),
    ("loguru", "tests/test_multiprocessing.py", 17): ("i", 0, 0, "get_context('fork')"),
    ("loguru", "tests/test_multiprocessing.py", 22): ("i", 0, 0, "get_context('spawn')"),
    ("loguru", "tests/test_parse.py", 15): ("iv", 0, 0, "with StringIO: yield"),
    ("loguru", "tests/test_reinstall.py", 10): ("i", 0, 0, "get_context('fork')"),
    ("loguru", "tests/test_reinstall.py", 15): ("i", 0, 0, "get_context('spawn')"),
}

# saleor: saleor/checkout/tests/fixtures/*.py, in dump order (1-based). iii = calls
# add_variant_to_checkout,
# add_voucher_to_checkout, or the price factory that computes and saves prices.
SALEOR_III = {
    5,
    7,
    8,
    9,
    13,
    16,
    17,
    18,
    19,
    27,
    31,
    32,
    33,
    34,
    39,
    40,
    53,
    54,
    60,
    61,
    62,
    64,
    65,
    67,
    71,
    74,
    75,
    76,
}
SALEOR_I = {37, 38, 41, 49, 50}
# ii that still calls saleor code beyond the ORM: set_country, store_value_in_metadata,
# fetch_checkout_info/lines, calculations
SALEOR_BROAD_II = {3, 4, 12, 45, 47, 48, 55, 56, 69, 70, 72}
