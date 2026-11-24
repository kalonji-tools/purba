import pytest


@pytest.mark.db
def test_query_db():
    assert True, "query"


@pytest.mark.slow
@pytest.mark.db
def test_migrate_db():
    assert True, "migrate"
