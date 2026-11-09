import json
import os
import shlex
import subprocess
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


def grpc_workflow_job():
    """Read the owning job so contract tests exercise its actual commands/paths."""
    workflow = (ROOT / ".github/workflows/ci.yml").read_text()
    return workflow.split("  grpc-interop:\n", 1)[1].split("  stdlib-runtime:\n", 1)[0]


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
        job = grpc_workflow_job()
        self.assertIn("tools/grpc/windows_acceptance.py", job)
        self.assertNotIn("--path stdlib/Extensions/Io/tests/", job)
        self.assertIn("grpc-windows/result.json", job)

    def test_result_uploads_wait_for_the_step_that_writes_them(self):
        blocks = grpc_workflow_job().split("      - uses: actions/upload-artifact@")
        for artifact, step in (
            ("grpc-interop-", "independent-peer-acceptance"),
            ("grpc-windows-acceptance-", "windows-native-acceptance"),
        ):
            (block,) = [b for b in blocks if "name: " + artifact in b]
            self.assertIn("steps." + step + ".outcome != 'skipped'", block)

    def test_workflow_preserves_only_failed_hosted_diagnostics(self):
        blocks = grpc_workflow_job().split("      - uses: actions/upload-artifact@")
        matches = [
            block for block in blocks if "name: grpc-failure-diagnostics-" in block
        ]
        self.assertEqual(
            len(matches), 1, "owning failed-step diagnostics must be archived"
        )
        block = matches[0]
        condition = next(
            line.strip()
            for line in block.splitlines()
            if line.strip().startswith("if:")
        )
        self.assertIn("failure()", condition)
        self.assertIn("steps.windows-native-acceptance.outcome == 'failure'", condition)
        self.assertIn(
            "steps.independent-peer-acceptance.outcome == 'failure'", condition
        )
        paths = block.split("          path: |\n", 1)[1].split(
            "          if-no-files-found:", 1
        )[0]
        self.assertEqual(
            {line.strip() for line in paths.splitlines() if line.strip()},
            {
                "target/grpc-windows/*.jsonl",
                "target/grpc-windows/*.stderr.log",
                "target/grpc-interop/*.log",
            },
        )

    def test_workflow_tracked_guard_preserves_generated_output_truth(self):
        commands = [
            shlex.split(line.strip())
            for line in grpc_workflow_job().splitlines()
            if line.strip().startswith(("git diff ", "git status "))
        ]
        guards = [command for command in commands if command[:2] == ["git", "diff"]]
        statuses = [command for command in commands if command[:2] == ["git", "status"]]
        self.assertEqual(
            len(guards), 1, "the owning job must reject tracked input changes"
        )
        self.assertEqual(
            len(statuses), 1, "the owning job must retain dirty-path diagnostics"
        )
        with tempfile.TemporaryDirectory() as folder:
            directory = Path(folder)
            env = dict(
                os.environ,
                GIT_CONFIG_NOSYSTEM="1",
                GIT_CONFIG_GLOBAL=os.devnull,
                GIT_AUTHOR_NAME="Fixture",
                GIT_AUTHOR_EMAIL="fixture@example.invalid",
                GIT_COMMITTER_NAME="Fixture",
                GIT_COMMITTER_EMAIL="fixture@example.invalid",
                GIT_AUTHOR_DATE="2000-01-01T00:00:00Z",
                GIT_COMMITTER_DATE="2000-01-01T00:00:00Z",
            )

            def git(*arguments, input=None):
                return subprocess.check_output(
                    ["git", *arguments], cwd=directory, env=env, input=input
                )

            git("init", "--quiet")
            source = directory / "source.txt"
            source.write_text("tracked baseline\n")
            git("add", "source.txt")
            tree = git("write-tree").decode().strip()
            # This Git object is disposable fixture data, not a project commit.
            commit = (
                git(
                    "-c",
                    "commit.gpgsign=false",
                    "commit-tree",
                    tree,
                    input=b"test: source audit fixture\n",
                )
                .decode()
                .strip()
            )
            git("update-ref", "HEAD", commit)
            self.assertEqual(
                subprocess.run(
                    guards[0], cwd=directory, env=env, capture_output=True
                ).returncode,
                0,
            )
            self.assertFalse(
                subprocess.check_output(statuses[0], cwd=directory, env=env).strip()
            )
            (directory / "generated-provider.lib").write_bytes(b"generated output")
            self.assertEqual(
                subprocess.run(
                    guards[0], cwd=directory, env=env, capture_output=True
                ).returncode,
                0,
            )
            self.assertTrue(
                subprocess.check_output(statuses[0], cwd=directory, env=env).strip(),
                "untracked generated output must not be relabeled a clean working tree",
            )
            source.write_text("tracked mutation\n")
            self.assertEqual(
                subprocess.run(
                    guards[0], cwd=directory, env=env, capture_output=True
                ).returncode,
                1,
            )
            git("add", "source.txt")
            self.assertEqual(
                subprocess.run(
                    guards[0], cwd=directory, env=env, capture_output=True
                ).returncode,
                1,
            )


if __name__ == "__main__":
    unittest.main()
