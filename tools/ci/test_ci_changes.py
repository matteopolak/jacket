import unittest

import ci_changes


class Matching(unittest.TestCase):
    def test_globs(self):
        self.assertTrue(
            ci_changes.matches(
                ["stdlib/Extensions/SDL3/**"], ["stdlib/Extensions/SDL3/a/b.jai"]
            )
        )
        self.assertFalse(
            ci_changes.matches(["tools/sdl3*.py"], ["tools/native/sdl3/x.py"])
        )
        self.assertTrue(ci_changes.matches(["nix/**"], [".github/workflows/ci.yml"]))
        self.assertFalse(ci_changes.matches(["nix/**"], ["stdlib/Basic/module.jai"]))

    def test_sdl3_runs_only_for_its_own_paths(self):
        sdl3 = ci_changes.AREAS["sdl3"]
        self.assertTrue(ci_changes.matches(sdl3, ["tools/bindings/sdl3_gen.py"], False))
        self.assertFalse(ci_changes.matches(sdl3, [".github/workflows/ci.yml"], False))
        self.assertFalse(
            ci_changes.matches(sdl3, ["tools/native/native-libs.json"], False)
        )

    def test_shell_scripts_run_shellcheck(self):
        shell = ci_changes.AREAS["shell"]
        self.assertTrue(ci_changes.matches(shell, ["install.sh"]))
        self.assertTrue(ci_changes.matches(shell, ["tools/ci/windows-llvm/prepare.sh"]))
        self.assertTrue(ci_changes.matches(shell, [".githooks/pre-commit"]))
        self.assertFalse(ci_changes.matches(shell, ["stdlib/Basic/module.jai"]))

    def test_docs_only_changes_skip_code(self):
        self.assertFalse(
            ci_changes.code_changed(
                ["docs/tools/cli.md", "README.md", "docs/images/a.png"]
            )
        )
        self.assertTrue(
            ci_changes.code_changed(["docs/tools/cli.md", "tools/ci/ci_changes.py"])
        )
        # Docs that a generator or test reads, and Markdown fixtures, count as code.
        self.assertTrue(
            ci_changes.code_changed(["docs/specs/stdlib/modules/socket.md"])
        )
        self.assertTrue(ci_changes.code_changed(["docs/tools/jailint.md"]))
        self.assertTrue(
            ci_changes.code_changed(["stdlib/Basic/tests/fixtures/notes.md"])
        )


class JobSelection(unittest.TestCase):
    ALL = {area: True for area in ci_changes.AREAS} | {"code": True}

    def test_every_area_has_a_job_and_every_job_a_known_area(self):
        areas = set(ci_changes.AREAS) | {"code"}
        self.assertEqual({a for a in ci_changes.JOBS.values() if a}, areas)

    def test_everything_runs_when_all_areas_do(self):
        self.assertEqual(ci_changes.selected_jobs(self.ALL, ""), list(ci_changes.JOBS))

    def test_false_areas_drop_their_jobs(self):
        results = self.ALL | {"windows": False, "code": False}
        jobs = ci_changes.selected_jobs(results, "")
        self.assertIn("docs", jobs)
        self.assertIn("nix", jobs)
        self.assertNotIn("windows-native", jobs)
        self.assertNotIn("test", jobs)

    def test_a_manual_list_narrows_the_jobs(self):
        self.assertEqual(
            ci_changes.selected_jobs(self.ALL, "nix, test"), ["test", "nix"]
        )
        self.assertEqual(ci_changes.selected_jobs(self.ALL | {"nix": False}, "nix"), [])

    def test_ci_yml_gates_exactly_the_listed_jobs(self):
        from pathlib import Path
        import re

        text = (
            Path(__file__).resolve().parents[2] / ".github/workflows/ci.yml"
        ).read_text()
        gated = re.findall(
            r"contains\(fromJSON\(needs\.changes\.outputs\.jobs\), '([a-z0-9-]+)'\)",
            text,
        )
        self.assertEqual(sorted(gated), sorted(ci_changes.JOBS))


if __name__ == "__main__":
    unittest.main()
