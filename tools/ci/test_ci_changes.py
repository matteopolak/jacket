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

    def test_windows_jobs_run_for_lowering_linking_and_the_stdlib(self):
        for path in [
            "crates/jaic-llvm/src/lower/symbols.rs",
            "crates/jaic-llvm/src/link.rs",
            "crates/jaic-llvm/src/linker.rs",
            "crates/jaic-llvm/src/link_options.rs",
            "crates/jaic/src/abi.rs",
            "crates/jaic/src/target.rs",
            "crates/jaic/src/ir/lower.rs",
            "stdlib/Basic/module.jai",
            "stdlib/Math/module.jai",
            "crates/jaic/src/msvc.rs",
            "crates/jacket/src/main.rs",
            "stdlib/POSIX/declarations.jai",
            "stdlib/internal/runtime/x.jai",
        ]:
            with self.subTest(path=path):
                self.assertTrue(
                    ci_changes.matches(ci_changes.AREAS["windows"], [path], False)
                )

    def test_a_stdlib_math_change_selects_windows_stdlib(self):
        areas = ci_changes.AREAS
        results = {a: ci_changes.matches(p, ["stdlib/Math/module.jai"], False) for a, p in areas.items()}
        self.assertIn("windows-stdlib", ci_changes.selected_jobs(results | {"code": True}, ""))

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

    def test_each_mac_runs_few_long_jobs_in_the_full_tier(self):
        tests = ci_changes.test_matrix(True)
        stdlib = ci_changes.stdlib_matrix(True)
        for runner in self.MACOS:
            legs = [leg for leg in tests if leg["runner"] == runner]
            self.assertEqual([leg["part"] for leg in legs], ["rust", "checks"])
            self.assertEqual({leg["parts"] for leg in legs}, {",rust,", ",targets,lint,diff,"})
        macos = [leg for leg in tests if leg["runner"] in self.MACOS]
        macos += [leg for leg in stdlib if leg["platform"].startswith("macos")]
        self.assertEqual(len(macos), 8)
        for leg in macos:
            self.assertGreaterEqual(leg["timeout"], 120, leg)

    def test_every_test_leg_names_its_parts_with_delimiters(self):
        for full in (False, True):
            for leg in ci_changes.test_matrix(full):
                with self.subTest(leg=leg):
                    self.assertTrue(leg["parts"].startswith(",") and leg["parts"].endswith(","))
                    # `checks` is the one leg that merges parts under a name of its own.
                    self.assertTrue(f",{leg['part']}," in leg["parts"] or leg["part"] == "checks")

    def test_the_test_job_selects_steps_by_parts(self):
        from pathlib import Path

        text = (Path(__file__).resolve().parents[2] / ".github/workflows/ci.yml").read_text()
        job = text[text.index("\n  test:\n") : text.index("\n  vscode-extension:\n")]
        self.assertNotIn("matrix.part ==", job)
        self.assertIn("contains(matrix.parts, ',rust,')", job)

    def test_the_full_tier_covers_every_platform(self):
        runners = {leg["runner"] for leg in ci_changes.test_matrix(True)}
        self.assertEqual(
            runners, {"ubuntu-24.04", "ubuntu-24.04-arm", "macos-15", "macos-15-intel"}
        )
        platforms = {leg["platform"] for leg in ci_changes.stdlib_matrix(True)}
        self.assertEqual(
            platforms, {"linux-x64", "linux-arm64", "macos-arm64", "macos-x64"}
        )

    def test_host_independent_work_runs_on_linux_x64_only(self):
        legs = ci_changes.test_matrix(True)
        for part in ("jai", "portability", "lint-windows", "lint-macos"):
            self.assertEqual({leg["runner"] for leg in legs if leg["part"] == part}, {"ubuntu-24.04"})
        self.assertEqual({leg["runner"] for leg in legs if leg["part"] == "split"}, {"ubuntu-24.04-arm"})
        for leg in legs:
            if leg["part"] == "rust" and leg["runner"] != "ubuntu-24.04":
                self.assertIn("not test(wasm_target)", leg["expr"])
        for leg in ci_changes.test_matrix(False):
            if leg["runner"] != "ubuntu-24.04":
                self.assertIn("not test(wasm_target)", leg["expr"])

    def test_webassembly_runs_on_linux_x64_only(self):
        for full in (False, True):
            for leg in ci_changes.stdlib_matrix(full):
                with self.subTest(full=full, leg=leg):
                    self.assertEqual("wasm-" in leg.get("modes", ""), leg["platform"] == "linux-x64" and "modes" in leg)
                    self.assertEqual(bool(leg.get("wasm")), leg["platform"] == "linux-x64" and "jacket_test" in leg)

    def test_stdlib_coverage_is_measured_by_one_leg(self):
        for full in (False, True):
            with self.subTest(full=full):
                legs = [leg for leg in ci_changes.stdlib_matrix(full) if leg.get("coverage")]
                self.assertEqual([(leg["platform"], leg["shard"]) for leg in legs], [("linux-x64", "suites")])

    def test_jacket_test_is_divided_once_per_platform(self):
        for full in (False, True):
            legs = ci_changes.stdlib_matrix(full)
            for platform in {leg["platform"] for leg in legs}:
                with self.subTest(full=full, platform=platform):
                    shares = [leg["jacket_test"] for leg in legs if leg["platform"] == platform and leg.get("jacket_test")]
                    if shares == ["all"]:
                        continue
                    count = ci_changes.TOOLING_PARTS
                    self.assertEqual(shares, [f"hash:{k}/{count}" for k in range(1, count + 1)])


if __name__ == "__main__":
    unittest.main()
