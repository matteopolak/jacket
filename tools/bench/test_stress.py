import os
import unittest

from bench import procmem
from bench.cmd_stress import OS_BLOCK, heading, slope


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

    def test_heading_counts_the_steps_a_timed_run_took(self):
        timed = {'seed': 2, 'steps': 4210, 'minutes': 30.0, 'workspace': 'repo'}
        self.assertEqual(heading(timed), '# Language server stress session (seed 2, 4210 steps in 30 minutes, repo)')
        counted = {'seed': 1, 'steps': 300, 'minutes': 0, 'workspace': 'repo'}
        self.assertEqual(heading(counted), '# Language server stress session (seed 1, 300 steps, repo)')


if __name__ == '__main__':
    unittest.main()
