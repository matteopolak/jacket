import unittest

import ci_changes


class Matching(unittest.TestCase):
    def test_shared_apt_setup_runs_windows_cross_builds(self):
        for path in [
            "apt_lib.sh",
            "apt_install.sh",
            "install_ci_llvm_linux.sh",
            "test_apt_lib.py",
        ]:
            with self.subTest(path=path):
                self.assertTrue(
                    ci_changes.matches(
                        ci_changes.AREAS["windows"], ["tools/ci/" + path], False
                    )
                )

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
        # `sanitizers-macos` is no job: main() reads it to add the macOS leg to sanitizer_matrix.
        self.assertEqual(sorted(gated), sorted(set(ci_changes.JOBS) - {"sanitizers-macos"}))

    def test_macos_asan_runs_on_pushes_only_for_sanitizer_relevant_paths(self):
        area = ci_changes.AREAS["sanitizers-macos"]
        self.assertTrue(ci_changes.matches(area, ["crates/jaic-llvm/src/lib.rs"]))
        self.assertTrue(ci_changes.matches(area, ["tools/differential/jaic_sweep.py"]))
        self.assertFalse(ci_changes.matches(area, ["stdlib/Hash/module.jai"], False))
        self.assertFalse(ci_changes.matches(area, ["crates/jailint/src/lib.rs"], False))


class Tiers(unittest.TestCase):
    # GitHub runs at most five macOS jobs at once for the repository.
    MACOS = ("macos-15", "macos-15-intel")

    def test_every_shard_set_is_complete(self):
        for full in (False, True):
            legs = ci_changes.test_matrix(full)
            groups = {}
            for leg in legs:
                groups.setdefault((leg["runner"], leg["part"]), []).append(leg["shard"])
            for (runner, part), got in groups.items():
                with self.subTest(full=full, runner=runner, part=part):
                    self.assertEqual(got, ci_changes.shards(len(got)))

    def test_the_fast_tier_fits_the_macos_limit(self):
        tests = ci_changes.test_matrix(False)
        stdlib = ci_changes.stdlib_matrix(False)
        macos = [leg for leg in tests if leg["runner"] in self.MACOS]
        macos += [leg for leg in stdlib if leg["platform"].startswith("macos")]
        self.assertLessEqual(len(macos), 5)

    def test_the_full_tier_covers_every_platform(self):
        runners = {leg["runner"] for leg in ci_changes.test_matrix(True)}
        self.assertEqual(
            runners, {"ubuntu-24.04", "ubuntu-24.04-arm", "macos-15", "macos-15-intel"}
        )
        platforms = {leg["platform"] for leg in ci_changes.stdlib_matrix(True)}
        self.assertEqual(
            platforms, {"linux-x64", "linux-arm64", "macos-arm64", "macos-x64"}
        )


if __name__ == "__main__":
    unittest.main()
