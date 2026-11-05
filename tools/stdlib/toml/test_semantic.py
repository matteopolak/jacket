import datetime
import unittest

from semantic import semantic_equal


class SemanticComparison(unittest.TestCase):
    def test_numeric_types_are_distinct(self):
        self.assertFalse(semantic_equal({'x': 1}, {'x': True}))
        self.assertFalse(semantic_equal([1], [1.0]))
        self.assertTrue(semantic_equal({'x': [1, True, 1.0]}, {'x': [1, True, 1.0]}))

    def test_signed_zero_is_distinct(self):
        self.assertFalse(semantic_equal(-0.0, 0.0))
        self.assertTrue(semantic_equal(-0.0, -0.0))

    def test_offset_is_not_normalized_to_an_instant(self):
        utc = datetime.timezone.utc
        plus_one = datetime.timezone(datetime.timedelta(hours=1))
        first = datetime.datetime(2024, 2, 29, 12, tzinfo=plus_one)
        same_instant = datetime.datetime(2024, 2, 29, 11, tzinfo=utc)
        self.assertEqual(first, same_instant)
        self.assertFalse(semantic_equal(first, same_instant))
        self.assertTrue(semantic_equal(first, first))

    def test_local_categories_and_fraction_are_distinct(self):
        self.assertFalse(semantic_equal(datetime.date(2024, 2, 29), datetime.datetime(2024, 2, 29)))
        self.assertFalse(semantic_equal(datetime.time(12, 34, 56, 123000), datetime.time(12, 34, 56, 123001)))

    def test_table_order_is_not_semantic(self):
        self.assertTrue(semantic_equal({'a': 1, 'b': 'x'}, {'b': 'x', 'a': 1}))


if __name__ == '__main__':
    unittest.main()
