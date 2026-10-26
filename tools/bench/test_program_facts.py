import unittest

from bench import program_facts as p


class InstructionLines(unittest.TestCase):
    def test_counts_instruction_lines_not_symbols(self):
        llvm = ('\nt:\tfile format mach-o arm64\n\nDisassembly of section __TEXT,__text:\n\n'
                '00000001000005e8 <_main>:\n100005e8:      \tsub\tsp, sp, #0x20\n'
                '100005ec:      \tstp\tx29, x30, [sp, #0x10]\n')
        gnu = '0000000000401000 <_start>:\n  401000:\tf3 0f 1e fa\tendbr64\n  401004:\tret\n'
        self.assertEqual(len(p.INSTRUCTION.findall(llvm)), 2)
        self.assertEqual(len(p.INSTRUCTION.findall(gnu)), 2)


class Counters(unittest.TestCase):
    def test_reads_time_and_perf_counters(self):
        time = '  0.00 real 0.00 user\n           123456  instructions retired\n            98765  cycles elapsed\n'
        self.assertEqual(p.parse_counters(time, p.TIME_COUNTER), {'retired_instructions': 123456, 'cycles': 98765})
        perf = '5000,,instructions:u,100.00,,\n7000,,cycles:u,100.00,,\n'
        self.assertEqual(p.parse_counters(perf, p.PERF_COUNTER), {'retired_instructions': 5000, 'cycles': 7000})

    def test_median_and_launch_cost(self):
        runs = [{'cycles': 10, 'retired_instructions': 3}, {'cycles': 30}, {'cycles': 20, 'retired_instructions': 5}]
        self.assertEqual(p.median_counters(runs), {'cycles': 20, 'retired_instructions': 4})
        self.assertEqual(p.subtract({'cycles': 20, 'retired_instructions': 4}, {'cycles': 25}),
                         {'cycles': 0, 'retired_instructions': 4})


if __name__ == '__main__':
    unittest.main()
