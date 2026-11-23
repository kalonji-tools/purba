"""Tests that research/configuration-dials/unittest/run.sh runs under each dial."""

import unittest
import warnings


class Dials(unittest.TestCase):
    def test_a_fails(self):
        self.assertEqual(1, 2, "a test that fails")

    def test_b_warns_deprecation(self):
        warnings.warn("old api", DeprecationWarning)
        self.assertTrue(True, "the test only warns")

    @unittest.expectedFailure
    def test_c_expected_failure_that_passes(self):
        self.assertTrue(True, "an expected failure that passes")


if __name__ == "__main__":
    unittest.main(warnings="error")
