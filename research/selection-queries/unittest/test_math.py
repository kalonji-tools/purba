import unittest


class Math(unittest.TestCase):
    def test_add(self):
        self.assertEqual(1 + 1, 2, "addition")

    def test_add_slow(self):
        self.assertEqual(2 + 2, 4, "addition, slow")


class Store(unittest.TestCase):
    def test_query_db(self):
        self.assertTrue(True, "query")

    def test_migrate_db_slow(self):
        self.assertTrue(True, "migrate")
