"""The shape of check_developer_tools.py: which checks it lists, how it skips by host and how it splits into parts.

Running the checks themselves needs the source-built tools; see that module's docstring.
"""

import importlib
import os
import platform
import unittest
from pathlib import Path
from unittest import mock

from developer_checks import check_developer_tools as suite
from developer_checks import dev_common

HERE = Path(__file__).parent
# check_* modules that are not entries of their own: the suite, and the bridge check that
# check_developer_type_emission re-exports.
NOT_ENTRIES = {"check_developer_tools", "check_developer_public_bridge"}


class Listing(unittest.TestCase):
    def test_every_entry_names_a_module_with_main(self):
        for module, _ in suite.CHECKS:
            self.assertTrue(callable(importlib.import_module(f"developer_checks.{module}").main), module)

    def test_every_check_module_is_listed_or_excused(self):
        listed = {module for module, _ in suite.CHECKS}
        modules = {path.stem for path in HERE.glob("check_*.py")}
        self.assertEqual(modules - listed - NOT_ENTRIES, set(), "add it to CHECKS in check_developer_tools.py")
        self.assertEqual(listed - modules - {"build_developer_debug"}, set())

    def test_no_entry_is_repeated(self):
        modules = [module for module, _ in suite.CHECKS]
        self.assertEqual(len(modules), len(set(modules)))

    def test_hosts_are_known_platform_names(self):
        for module, hosts in suite.CHECKS:
            self.assertTrue(set(hosts) <= {"Darwin", "Linux", "Windows"}, module)

    def test_a_check_on_the_wrong_host_is_skipped_not_passed(self):
        for module, hosts in suite.CHECKS:
            method = getattr(suite.ManifestChecks, "test_" + module)
            self.assertEqual(getattr(method, "__unittest_skip__", False), bool(hosts) and platform.system() not in hosts, module)

    def test_every_need_names_a_listed_check(self):
        listed = {module for module, _ in suite.CHECKS}
        for needs in suite.NEEDS.values():
            self.assertLessEqual(set(needs), listed)

    def test_a_check_runs_once_however_many_tests_need_it(self):
        calls = []
        with mock.patch.object(suite, "_done", set()), mock.patch.object(suite.importlib, "import_module") as load:
            load.return_value.main = lambda: calls.append(1)
            suite.run_check("check_developer_msf")
            suite.run_check("check_developer_msf")
        self.assertEqual(calls, [1])


class Parts(unittest.TestCase):
    def names(self, part=None):
        environment = {suite.PART_ENV: part} if part else {}
        with mock.patch.dict(os.environ, environment):
            loaded = unittest.defaultTestLoader.loadTestsFromModule(suite)
        # Whole classes without a part; the kept tests alone with one.
        return [test.id() for group in loaded for test in (group if isinstance(group, unittest.TestSuite) else [group])]

    def test_the_parts_partition_the_suite(self):
        whole = self.names()
        parts = [self.names(f"{k}/4") for k in range(1, 5)]
        self.assertEqual(sorted(sum(parts, [])), sorted(whole))
        self.assertTrue(all(parts))

    def test_a_malformed_part_is_rejected(self):
        for part in ("0/4", "5/4", "x"):
            with self.subTest(part=part), self.assertRaises(SystemExit):
                self.names(part)


class SharedRunner(unittest.TestCase):
    def test_labeled_runner_returns_stdout_or_the_process(self):
        out = dev_common.labeled_runner("echo", expected=None, stdout=True)(["echo", "hi"])
        self.assertEqual(out.strip(), "hi")
        proc = dev_common.labeled_runner("echo", expected=None)(["echo", "hi"])
        self.assertEqual(proc.stdout.strip(), "hi")
        with self.assertRaises(Exception):
            dev_common.labeled_runner("echo", expected="other\n")(["echo", "hi"])


if __name__ == "__main__":
    unittest.main()
