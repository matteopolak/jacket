"""The graph audit runs unconfined where the policy allows it (a CI runner) and never otherwise."""
import unittest
from pathlib import Path

from corpus import upstream_sandbox
from lsp import project_graph


class ConfinementTests(unittest.TestCase):
    def test_a_runner_without_a_sandbox_runs_unconfined(self):
        self.assertIsNone(project_graph.confinement([Path("/in")], [Path("/out")], policy=lambda no_sandbox: False))

    def test_an_available_sandbox_confines(self):
        sandbox = project_graph.confinement([Path("/in")], [Path("/out")], policy=lambda no_sandbox: True)
        self.assertEqual(sandbox.reads, [Path("/in")])
        self.assertEqual(sandbox.writes, [Path("/out")])

    def test_a_missing_sandbox_outside_ci_refuses(self):
        def refuse(no_sandbox):
            return upstream_sandbox.policy(no_sandbox, env={}, available=lambda: "no bwrap")

        with self.assertRaises(upstream_sandbox.Unsandboxed):
            project_graph.confinement([], [], policy=refuse)


class JaicTests(unittest.TestCase):
    def test_a_named_jaic_is_used_as_given(self):
        self.assertEqual(project_graph.find_jaic(Path("out/jaic"), which=lambda name: None), Path("out/jaic"))

    def test_the_jaic_on_path_is_used_when_none_is_named(self):
        self.assertEqual(project_graph.find_jaic(None, which=lambda name: "/bin/jaic"), Path("/bin/jaic"))

    def test_no_jaic_is_an_error_that_says_how_to_name_one(self):
        with self.assertRaises(SystemExit) as raised:
            project_graph.find_jaic(None, which=lambda name: None)
        self.assertIn("--jaic", str(raised.exception))


if __name__ == "__main__":
    unittest.main()
