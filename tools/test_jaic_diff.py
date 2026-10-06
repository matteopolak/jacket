import importlib.util
import unittest
from pathlib import Path

import jaigen

_spec = importlib.util.spec_from_file_location("jaic_diff", Path(__file__).with_name("jaic-diff.py"))
diff = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(diff)

_spec = importlib.util.spec_from_file_location("jaic_reduce", Path(__file__).with_name("jaic-reduce.py"))
reduce = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(reduce)


def case(cid="c"):
    return diff.Case(cid, Path("/x/main.jai"))


class GeneratorTests(unittest.TestCase):
    def test_a_seed_always_gives_the_same_program(self):
        self.assertEqual(jaigen.generate(7), jaigen.generate(7))
        self.assertNotEqual(jaigen.generate(7), jaigen.generate(8))

    def test_programs_end_by_printing_the_checksum(self):
        for seed in range(5):
            program = jaigen.generate(seed, 0.5)
            self.assertIn("main :: () {", program)
            self.assertIn('print("checksum %\\n"', program)

    def test_size_scales_the_program(self):
        self.assertLess(len(jaigen.generate(3, 0.3)), len(jaigen.generate(3, 2.0)))


class VerdictTests(unittest.TestCase):
    def test_identical_results_agree(self):
        same = {b: diff.Result("exit 0", "out\n", "") for b in diff.ALL_BACKENDS}
        self.assertEqual(diff.verdict(case(), same), "agree")

    def test_different_stdout_disagrees(self):
        results = {"interp": diff.Result("exit 0", "1\n"), "native": diff.Result("exit 0", "2\n")}
        self.assertEqual(diff.verdict(case(), results), "DISAGREE")

    def test_stderr_is_compared_only_when_every_backend_exited(self):
        exited = {"interp": diff.Result("exit 0", "", "a"), "native": diff.Result("exit 0", "", "b")}
        self.assertEqual(diff.verdict(case(), exited), "DISAGREE")
        trapped = {"interp": diff.Result("runtime error", "", "runtime error: index"),
                   "native": diff.Result("runtime error", "", "")}
        self.assertEqual(diff.verdict(case(), trapped), "agree")

    def test_unsupported_backends_do_not_count(self):
        results = {"interp": diff.Result("exit 0", "x"), "wasm": diff.Result("unsupported", note="threads")}
        self.assertEqual(diff.verdict(case(), results), "skip")
        results["native"] = diff.Result("exit 0", "x")
        self.assertEqual(diff.verdict(case(), results), "agree")

    def test_a_compile_error_on_one_backend_disagrees(self):
        results = {"interp": diff.Result("exit 0"), "wasm": diff.Result("compile error")}
        self.assertEqual(diff.verdict(case(), results), "DISAGREE")
        results["interp"] = diff.Result("compile error")
        self.assertEqual(diff.verdict(case(), results), "invalid")

    def test_output_varies_cases_compare_status_only(self):
        cid = next(iter(diff.OUTPUT_VARIES))
        results = {"interp": diff.Result("exit 0", "a"), "wasm": diff.Result("exit 0", "b")}
        self.assertEqual(diff.verdict(case(cid), results), "agree")

    def test_signals_and_runtime_errors_classify_alike(self):
        self.assertEqual(diff.Runner.classify("", "", 133).status, "runtime error")
        self.assertEqual(diff.Runner.classify("", "runtime error: x", 1).status, "runtime error")
        self.assertEqual(diff.Runner.classify("", "", 3).status, "exit 3")


class ReducerTests(unittest.TestCase):
    def test_signature_groups_backends_by_result(self):
        results = {"interp": diff.Result("exit 0", "1"), "wasm": diff.Result("exit 0", "1"),
                   "native-O2": diff.Result("exit 0", "2")}
        self.assertEqual(reduce.signature(results),
                         frozenset({frozenset({"interp", "wasm"}), frozenset({"native-O2"})}))
        results["native-O2"] = diff.Result("exit 0", "1")
        self.assertIsNone(reduce.signature(results))

    def test_blocks_pair_braces(self):
        lines = ["a :: () {", "    if x {", "        y;", "    }", "}"]
        self.assertEqual(reduce.blocks(lines), [(1, 3), (0, 4)])


if __name__ == "__main__":
    unittest.main()
