import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock

from toolkit.repo import ROOT
from windows_acceptance import (
    HTTP_GRPC_REQUIRED,
    PREREQUISITES,
    TERMINAL,
    checked_results,
    commands,
    main,
)


def events(names, status="passed"):
    rows = [dict(type="run", event="started", tests=len(names))]
    for name in names:
        if status != "skipped":
            rows.append(dict(type="test", event="started", name=name))
        rows.append(dict(type="test", event=status, name=name))
    counts = {event: len(names) if event == status else 0 for event in TERMINAL}
    rows.append(dict(type="run", event="finished", not_run=0, **counts))
    return "\n".join(json.dumps(row) for row in rows)


class WindowsAcceptance(unittest.TestCase):
    def test_required_ids_must_actually_pass(self):
        self.assertEqual(
            set(checked_results(events(PREREQUISITES), PREREQUISITES)),
            set(PREREQUISITES),
        )
        for status in ("skipped", "xfail", "flaky", "failed", "timed_out", "xpass"):
            with self.subTest(status=status), self.assertRaises(ValueError):
                checked_results(events(PREREQUISITES, status), PREREQUISITES)

    def test_empty_partial_list_and_started_are_not_execution(self):
        reports = [
            "",
            events([]),
            events(PREREQUISITES[:-1]),
            json.dumps(dict(type="list", name=PREREQUISITES[0])),
            json.dumps(dict(type="test", event="started", name=PREREQUISITES[0]))
            + "\n"
            + events([]),
        ]
        for text in reports:
            with self.subTest(text=text), self.assertRaises(ValueError):
                checked_results(text, PREREQUISITES)

    def test_incomplete_extra_cases_cannot_hide_behind_required_passes(self):
        base = [json.loads(line) for line in events(HTTP_GRPC_REQUIRED).splitlines()]
        reports = []
        rows = json.loads(json.dumps(base))
        rows[0]["tests"] += 1
        rows[-1]["not_run"] = 1
        reports.append(rows)
        rows = json.loads(json.dumps(base))
        rows.insert(-1, dict(type="test", event="started", name="Http::unfinished"))
        reports.append(rows)
        rows = json.loads(json.dumps(base))
        rows[0]["tests"] += 1
        reports.append(rows)
        rows = json.loads(json.dumps(base))
        rows[-1]["passed"] += 1
        reports.append(rows)
        for index, rows in enumerate(reports):
            with self.subTest(report=index), self.assertRaises(ValueError):
                checked_results(
                    "\n".join(json.dumps(row) for row in rows), HTTP_GRPC_REQUIRED
                )

    def test_optional_skips_count_without_inventing_execution(self):
        rows = [json.loads(line) for line in events(PREREQUISITES).splitlines()]
        rows[0]["tests"] += 1
        rows[-1]["skipped"] += 1
        rows.insert(-1, dict(type="test", event="skipped", name="doc: ignored"))
        cases = checked_results(
            "\n".join(json.dumps(row) for row in rows), PREREQUISITES
        )
        self.assertEqual(cases["doc: ignored"], "skipped")

    def test_duplicate_or_malformed_results_fail(self):
        for text in (
            events([*PREREQUISITES, PREREQUISITES[0]]),
            "not json",
            events(PREREQUISITES)
            + "\n"
            + json.dumps(dict(type="run", event="finished")),
        ):
            with self.subTest(text=text), self.assertRaises(ValueError):
                checked_results(text, PREREQUISITES)

    def test_private_failure_fields_are_not_exported(self):
        rows = [json.loads(line) for line in events(PREREQUISITES).splitlines()]
        for row in rows:
            if row.get("type") == "test":
                row["stdout"] = "/private/do-not-export"
                row["failures"] = [dict(file="/private/do-not-export")]
        text = "\n".join(json.dumps(row) for row in rows)
        self.assertNotIn(
            "do-not-export", json.dumps(checked_results(text, PREREQUISITES))
        )

    def test_commands_use_real_roots_and_exact_prerequisites(self):
        suites = commands(Path("jacket"))
        self.assertEqual(len(suites), 2)
        for _, command, _ in suites:
            roots = [
                command[i + 1] for i, value in enumerate(command) if value == "--path"
            ]
            self.assertTrue(roots)
            self.assertEqual(command[command.index("--timeout") + 1], "120")
            self.assertTrue(all(not root.endswith(".jai") for root in roots))
        self.assertIn("--exact", suites[0][1])
        self.assertTrue(all(name in suites[0][1] for name in PREREQUISITES))

    def test_zero_exit_does_not_hide_missing_execution(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            executable = root / "jacket.exe"
            executable.write_bytes(b"owned fake executable for subprocess mock")

            def run(command, **kwargs):
                kwargs["stdout"].write(events([]))
                return mock.Mock(returncode=0)

            with mock.patch("windows_acceptance.subprocess.run", side_effect=run):
                self.assertEqual(
                    main(["--jacket", str(executable), "--output", str(root / "out")]),
                    1,
                )
            report = json.loads((root / "out/result.json").read_text())
            self.assertEqual(len(report["suites"]), 2)
            self.assertTrue(all(not row["passed"] for row in report["suites"]))

    def test_fixture_bytes_survive_windows_checkout(self):
        import subprocess

        path = "stdlib/Extensions/Http/tests/fixtures/stream-file.txt"
        checked_out = subprocess.check_output(
            [
                "git",
                "-c",
                "core.autocrlf=true",
                "cat-file",
                "--filters",
                "--path=" + path,
                "HEAD:" + path,
            ],
            cwd=ROOT,
        )
        self.assertEqual(checked_out, b"a file served in pieces\n")

    def test_workflow_calls_checked_driver(self):
        workflow = (ROOT / ".github/workflows/ci.yml").read_text()
        job = workflow.split("  grpc-interop:\n", 1)[1].split("  stdlib-runtime:\n", 1)[
            0
        ]
        self.assertIn("tools/grpc/windows_acceptance.py", job)
        self.assertNotIn("--path stdlib/Extensions/Io/tests/", job)
        self.assertIn("grpc-windows/result.json", job)


if __name__ == "__main__":
    unittest.main()
