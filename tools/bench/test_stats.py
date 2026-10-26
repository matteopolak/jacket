import unittest

from bench.stats import Stat, median_absolute_deviation, percentile, ratio


class StatTests(unittest.TestCase):
    def test_median_and_range(self):
        stat = Stat.of([3.0, 1.0, 2.0, 10.0, 2.5])
        self.assertEqual((stat.n, stat.median, stat.min, stat.max), (5, 2.5, 1.0, 10.0))

    def test_mad_ignores_one_outlier(self):
        # Deviations from the median 2.5 are 0.5, 1.5, 0.5, 7.5, 0: their median is 0.5.
        self.assertEqual(median_absolute_deviation([3.0, 1.0, 2.0, 10.0, 2.5]), 0.5)
        self.assertEqual(median_absolute_deviation([4.0, 4.0, 4.0]), 0.0)

    def test_percentile_interpolates(self):
        values = [10.0, 20.0, 30.0, 40.0]
        self.assertEqual(percentile(values, 0.0), 10.0)
        self.assertEqual(percentile(values, 1.0), 40.0)
        self.assertAlmostEqual(percentile(values, 0.5), 25.0)
        self.assertAlmostEqual(percentile(values, 0.95), 38.5)
        self.assertEqual(percentile([7.0], 0.95), 7.0)

    def test_p95_of_many_samples(self):
        stat = Stat.of([float(i) for i in range(1, 101)])
        self.assertAlmostEqual(stat.p95, 95.05)
        self.assertEqual(stat.median, 50.5)

    def test_json_round_trip(self):
        stat = Stat.of([1.0, 2.0, 4.0])
        self.assertEqual(Stat.from_json(stat.to_json()), stat)

    def test_empty_input_is_an_error(self):
        with self.assertRaises(ValueError):
            Stat.of([])

    def test_ratio_guards_zero(self):
        self.assertEqual(ratio(3, 2), 1.5)
        self.assertIsNone(ratio(3, 0))


if __name__ == '__main__':
    unittest.main()
