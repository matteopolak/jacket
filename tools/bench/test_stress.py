import os
import unittest

from bench import procmem
from bench.cmd_stress import OS_BLOCK, slope


class StressTests(unittest.TestCase):
    def test_slope_is_the_trend(self):
        self.assertAlmostEqual(slope([(0, 10.0), (1, 12.0), (2, 14.0)]), 2.0)
        self.assertEqual(slope([(0, 5.0)]), 0.0)
        self.assertEqual(slope([(1, 5.0), (1, 9.0)]), 0.0)

    def test_process_memory_reads_this_process(self):
        resident, private = procmem.read(os.getpid(), None)
        self.assertGreater(resident, 1)
        self.assertGreater(private, 1)

    def test_os_block_names_its_bodies(self):
        lines = (OS_BLOCK % {'n': 1}).split('\n')
        self.assertIn('linux_value :=', lines[3])
        self.assertIn('windows_value :=', lines[10])


if __name__ == '__main__':
    unittest.main()
