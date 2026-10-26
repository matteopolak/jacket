import unittest

from bench.phases import PhaseTiming
from bench.stages import Bucket, JaicTimings, with_total

JAIC = '''note: something
jaic-timing: front end 3.0 1
jaic-timing: workspaces 0.01 1
jaic-timing: prepare output 0.1 1
jaic-timing: codegen 0.5 2
jaic-timing: link 0.25 1
jaic-timing: debug info 0.05 1
jaic-timing: total 4.1 1
jaic-timing: interp 1.0 12
'''


class JaicTimingTests(unittest.TestCase):
    def test_phases_map_to_buckets(self):
        stages = JaicTimings().parse(JAIC)
        self.assertAlmostEqual(stages[Bucket.FRONT_END], 2.0)       # front end minus the interpreter
        self.assertAlmostEqual(stages[Bucket.COMPILE_TIME], 1.0)    # interp
        self.assertAlmostEqual(stages[Bucket.CODEGEN], 0.65)        # prepare output + codegen + debug info
        self.assertAlmostEqual(stages[Bucket.LINK], 0.25)
        self.assertAlmostEqual(stages[Bucket.TOTAL], 4.1)
        self.assertAlmostEqual(stages[Bucket.OTHER], 4.1 - 2.0 - 1.0 - 0.65 - 0.25)

    def test_the_program_run_under_interp_is_compile_time_execution(self):
        stages = JaicTimings().parse('jaic-timing: front end 0.1 1\njaic-timing: run 2.0 1\n'
                                     'jaic-timing: total 2.2 1\njaic-timing: interp 0.0 0\n')
        self.assertAlmostEqual(stages[Bucket.COMPILE_TIME], 2.0)
        self.assertAlmostEqual(stages[Bucket.OTHER], 0.1)

    def test_old_output_without_an_interp_line_still_parses(self):
        stages = JaicTimings().parse('jaic-timing: front end 1.0 1\njaic-timing: total 1.5 1\n')
        self.assertEqual(stages[Bucket.COMPILE_TIME], 0.0)
        self.assertAlmostEqual(stages[Bucket.FRONT_END], 1.0)

    def test_overlapping_phases_never_make_other_negative(self):
        # A metaprogram's workspaces are code-generated inside its front end.
        stages = JaicTimings().parse('jaic-timing: front end 1.0 1\njaic-timing: codegen 0.9 1\n'
                                     'jaic-timing: total 1.0 1\njaic-timing: interp 0.5 3\n')
        self.assertEqual(stages[Bucket.OTHER], 0.0)

    def test_no_total_means_no_stages(self):
        self.assertEqual(JaicTimings().parse('hello\nsome other output\n'), {})
        self.assertEqual(JaicTimings().parse('jaic-timing: front end 1.0 1\n'), {})


class PhaseTimingTests(unittest.TestCase):
    BUILD = ('bench-phase: parsed 0.010\nbench-phase: typechecked 0.030\nbench-phase: typechecked 0.040\n'
             'bench-phase: built 0.100\nbench-phase: pre_write 0.110\nbench-phase: post_write 0.200\n'
             'bench-phase: complete 0.205\n')

    def test_a_build_splits_into_front_end_codegen_and_link(self):
        stages = PhaseTiming().parse('hello\n' + self.BUILD)
        self.assertAlmostEqual(stages[Bucket.FRONT_END], 0.040)   # the last typechecked message
        self.assertAlmostEqual(stages[Bucket.CODEGEN], 0.070)
        self.assertAlmostEqual(stages[Bucket.LINK], 0.090)
        self.assertNotIn(Bucket.COMPILE_TIME, stages)

    def test_a_custom_link_is_timed_by_the_metaprogram_itself(self):
        stages = PhaseTiming().parse('bench-phase: typechecked 0.05\nbench-phase: built 0.08\n'
                                     'bench-phase: ready_link 0.12\nbench-phase: link 0.03\n'
                                     'bench-phase: post_write 0.30\nbench-phase: complete 0.31\n')
        self.assertAlmostEqual(stages[Bucket.CODEGEN], 0.07)  # to ready_link, so it includes writing objects
        self.assertAlmostEqual(stages[Bucket.LINK], 0.03)

    def test_a_check_has_only_a_front_end(self):
        stages = PhaseTiming().parse('bench-phase: parsed 0.01\nbench-phase: typechecked 0.05\nbench-phase: complete 0.06\n')
        self.assertEqual(stages, {Bucket.FRONT_END: 0.05, Bucket.CODEGEN: 0.0})

    def test_a_build_that_never_completed_has_no_stages(self):
        self.assertEqual(PhaseTiming().parse('bench-phase: parsed 0.01\n'), {})
        self.assertEqual(PhaseTiming().parse('nothing'), {})

    def test_the_total_and_other_come_from_the_wall_time(self):
        stages = with_total(PhaseTiming().parse(self.BUILD), 0.25)
        self.assertEqual(stages[Bucket.TOTAL], 0.25)
        self.assertAlmostEqual(stages[Bucket.OTHER], 0.25 - 0.040 - 0.070 - 0.090)


class WithTotalTests(unittest.TestCase):
    def test_a_compiler_that_prints_nothing_still_has_a_total(self):
        self.assertEqual(with_total({}, 0.42), {Bucket.TOTAL: 0.42})

    def test_total_is_the_wall_time_and_other_the_remainder(self):
        stages = with_total({Bucket.FRONT_END: 0.2, Bucket.LINK: 0.1, Bucket.TOTAL: 0.35, Bucket.OTHER: 0.05}, 0.5)
        self.assertEqual(stages[Bucket.TOTAL], 0.5)
        self.assertAlmostEqual(stages[Bucket.OTHER], 0.2)

    def test_overlapping_buckets_leave_no_negative_remainder(self):
        self.assertEqual(with_total({Bucket.FRONT_END: 0.4, Bucket.CODEGEN: 0.4}, 0.5)[Bucket.OTHER], 0.0)


if __name__ == '__main__':
    unittest.main()
