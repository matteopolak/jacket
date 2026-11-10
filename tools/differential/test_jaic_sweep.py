import unittest

from differential import jaic_sweep


class ExampleCaseTests(unittest.TestCase):
    def test_every_example_case_reads_and_only_run_cases_expect_output(self):
        found = list(jaic_sweep.cases("examples"))
        self.assertTrue(found)
        for cid, _path, mode, expect, _args in found:
            self.assertEqual(expect is not None, mode == "run", cid)
            if mode == "run":
                self.assertIn("contains", expect, cid)


if __name__ == "__main__":
    unittest.main()
