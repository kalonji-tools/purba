"""Tests that each probe of research/configuration-dials/pytest/run.sh selects with -k."""

import time
import warnings

import pytest


def test_sleeps_two_seconds():
    time.sleep(2)
    assert True, "the test only sleeps"


@pytest.mark.timeout(5)
def test_sleeps_two_seconds_marked_five():
    time.sleep(2)
    assert True, "the test only sleeps"


def test_warns_deprecation():
    warnings.warn("old api", DeprecationWarning)
    assert True, "the test only warns"


@pytest.mark.filterwarnings("ignore::DeprecationWarning")
def test_warns_deprecation_marked_ignore():
    warnings.warn("old api", DeprecationWarning)
    assert True, "the test only warns"


@pytest.mark.slowish
def test_carries_unregistered_marker():
    assert True, "the test only carries a marker"


@pytest.mark.xfail(reason="probe: it passes")
def test_xfail_that_passes():
    assert True, "an xfail test that passes"


@pytest.mark.xfail(reason="probe: it fails")
def test_xfail_that_fails():
    assert False, "an xfail test that fails"
