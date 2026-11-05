import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock

from toolkit.repo import ROOT
from windows_acceptance import PREREQUISITES, checked_results, commands, main


def events(names, status="passed"):
    return "\n".join(
        json.dumps(row)
        for row in [
            *[dict(type="test", event=status, name=name) for name in names],
            dict(type="run", event="finished"),
        ]
    )


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
        rows = [
            dict(
                type="test",
                event="passed",
                name=name,
                stdout="/private/do-not-export",
                failures=[dict(file="/private/do-not-export")],
            )
            for name in PREREQUISITES
        ]
        text = "\n".join(
            json.dumps(row) for row in [*rows, dict(type="run", event="finished")]
        )
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
