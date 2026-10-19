import importlib
import unittest
from pathlib import Path

import check_developer_tools as runner

HERE = Path(__file__).parent
# check_* modules that are not entries of their own: the runner, and the bridge check that
# check_developer_type_emission re-exports.
NOT_ENTRIES = {"check_developer_tools", "check_developer_public_bridge"}


class DeveloperChecksManifest(unittest.TestCase):
    def test_every_entry_names_a_module_with_main(self):
        for entry in runner.manifest_checks():
            self.assertTrue(callable(importlib.import_module(entry["module"]).main), entry)

    def test_every_check_module_is_listed_or_excused(self):
        listed = {entry["module"] for entry in runner.manifest_checks()}
        modules = {path.stem for path in HERE.glob("check_*.py")}
        self.assertEqual(modules - listed - NOT_ENTRIES, set(), "add it to developer_checks.json")
        self.assertEqual(listed - modules - {"build_developer_debug"}, set())

    def test_hosted_entries_have_a_label_and_known_hosts(self):
        for entry in runner.manifest_checks():
            if "hosts" in entry:
                self.assertTrue(entry.get("label"), entry)
                self.assertTrue(set(entry["hosts"]) <= {"Darwin", "Linux", "Windows"}, entry)

    def test_no_entry_is_repeated(self):
        modules = [entry["module"] for entry in runner.manifest_checks()]
        self.assertEqual(len(modules), len(set(modules)))

    def test_a_check_on_the_wrong_host_is_reported_not_passed(self):
        ran = []
        runner.UNAVAILABLE_EVIDENCE.clear()
        runner.host_evidence("mac only", lambda: ran.append(1), ["Darwin"], host="Linux")
        self.assertEqual(ran, [])
        self.assertEqual(runner.UNAVAILABLE_EVIDENCE[0]["status"], "unavailable")
        runner.UNAVAILABLE_EVIDENCE.clear()

    def test_labeled_runner_returns_stdout_or_the_process(self):
        import dev_common

        out = dev_common.labeled_runner("echo", expected=None, stdout=True)(["echo", "hi"])
        self.assertEqual(out.strip(), "hi")
        proc = dev_common.labeled_runner("echo", expected=None)(["echo", "hi"])
        self.assertEqual(proc.stdout.strip(), "hi")
        with self.assertRaises(Exception):
            dev_common.labeled_runner("echo", expected="other\n")(["echo", "hi"])


if __name__ == "__main__":
    unittest.main()
