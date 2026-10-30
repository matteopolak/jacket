import tempfile
import textwrap
import unittest
from pathlib import Path

import ci_changes
import job_inputs

_REAL = {}


def real(area):
    """The computed input patterns of `area` in this checkout (computed once)."""
    if "repo" not in _REAL:
        repo = job_inputs.Repo()
        _REAL["repo"] = (
            repo,
            repo.read(job_inputs.WORKFLOW),
            job_inputs.crate_graph(repo),
        )
    repo, text, graph = _REAL["repo"]
    if area not in _REAL:
        _REAL[area] = ci_changes.area_patterns(area, repo, text, graph)
    return _REAL[area]


def selects(area, path):
    return ci_changes.area_changed(area, real(area), [path])


class RealWorkflow(unittest.TestCase):
    def test_shared_apt_setup_runs_windows_cross_builds(self):
        for path in [
            "apt_lib.sh",
            "apt_install.sh",
            "install_ci_llvm_linux.sh",
        ]:
            with self.subTest(path=path):
                self.assertTrue(selects("windows", "tools/ci/" + path))

    def test_windows_jobs_run_for_lowering_linking_and_the_stdlib(self):
        for path in [
            "crates/jaic-llvm/src/lower/symbols.rs",
            "crates/jaic-llvm/src/link.rs",
            "crates/jaic/src/abi.rs",
            "crates/jaic/src/msvc.rs",
            "crates/jaic/src/ir/lower.rs",
            "crates/jacket/src/main.rs",
            "stdlib/Basic/module.jai",
            "stdlib/Math/module.jai",
            "stdlib/POSIX/declarations.jai",
            "stdlib/internal/runtime/x.jai",
            "tests/compiler/native/c-aggregate-abi/main.jai",
        ]:
            with self.subTest(path=path):
                self.assertTrue(selects("windows", path))

    def test_a_stdlib_math_change_selects_windows_stdlib(self):
        results = {
            area: selects(area, "stdlib/Math/module.jai")
            for area in ci_changes.input_areas()
        }
        self.assertIn(
            "windows-stdlib", ci_changes.selected_jobs(results | {"code": True}, "")
        )

    def test_sdl3_ignores_the_workflow_and_actions_but_not_its_own_inputs(self):
        self.assertTrue(selects("sdl3", "stdlib/Extensions/SDL3/module.jai"))
        self.assertTrue(selects("sdl3", "tools/bindings/sdl3_gen.py"))
        self.assertFalse(selects("sdl3", ".github/workflows/ci.yml"))
        self.assertFalse(selects("sdl3", ".github/actions/setup-unix/action.yml"))
        self.assertFalse(selects("sdl3", "stdlib/Clipboard/module.jai"))

    def test_the_workflow_runs_every_other_area(self):
        for area in ci_changes.input_areas():
            with self.subTest(area=area):
                self.assertEqual(
                    selects(area, ".github/workflows/ci.yml"),
                    area not in ci_changes.OWN_PATHS_ONLY,
                )

    def test_an_action_runs_the_jobs_that_use_it(self):
        self.assertTrue(selects("windows", ".github/actions/setup-windows/action.yml"))
        self.assertFalse(selects("nix", ".github/actions/setup-windows/action.yml"))

    def test_shell_scripts_run_shellcheck(self):
        for path in [
            "install.sh",
            "tools/ci/windows-llvm/prepare.sh",
            ".githooks/pre-commit",
        ]:
            with self.subTest(path=path):
                self.assertTrue(selects("shell", path))
        self.assertFalse(selects("shell", "stdlib/Basic/module.jai"))

    def test_the_flake_builds_the_compiler_and_the_stdlib(self):
        for path in [
            "flake.nix",
            "nix/jaic.nix",
            "crates/jaic/src/lib.rs",
            "stdlib/Basic/module.jai",
            "Cargo.lock",
        ]:
            with self.subTest(path=path):
                self.assertTrue(selects("nix", path))
        self.assertFalse(selects("nix", "editors/vscode/package.json"))

    def test_packaging_follows_the_installers_and_the_release_script(self):
        for path in [
            "install.sh",
            "install.ps1",
            "packaging/homebrew/jacket.rb.tmpl",
            "tools/release.py",
        ]:
            with self.subTest(path=path):
                self.assertTrue(selects("packaging", path))
        self.assertFalse(selects("packaging", "crates/jaic/src/lib.rs"))

    def test_clipboard_follows_its_module_and_what_it_imports(self):
        self.assertTrue(selects("clipboard", "stdlib/Clipboard/module.jai"))
        self.assertTrue(selects("clipboard", "stdlib/Basic/module.jai"))
        self.assertTrue(
            selects("clipboard", "tools/portable/check_portable_clipboard.py")
        )
        self.assertFalse(selects("clipboard", "crates/jai-wasm/src/lib.rs"))

    def test_macos_asan_runs_on_pushes_only_for_sanitizer_relevant_paths(self):
        self.assertTrue(selects("sanitizers-macos", "crates/jaic-llvm/src/lib.rs"))
        self.assertTrue(selects("sanitizers-macos", "tools/differential/jaic_sweep.py"))
        self.assertFalse(selects("sanitizers-macos", "stdlib/Hash/module.jai"))
        self.assertFalse(selects("sanitizers-macos", "crates/jailint/src/lib.rs"))


JOBS_WORKFLOW = textwrap.dedent(
    """\
    jobs:
      sdl3:
        steps:
          - run: target/debug/jaic app/main.jai
      windows-stdlib:
        steps:
          - uses: ./.github/actions/setup
          - run: python3 tools/win.py
      windows-native:
        steps:
          - run: cargo build -p app
      sanitizers:
        steps:
          - run: python3 tools/differential/sweep.py
          - run: python3 tools/other.py
      nix:
        steps:
          - run: nix build .#app
    """
)

FAKE = {
    ".github/workflows/ci.yml": JOBS_WORKFLOW,
    ".github/actions/setup/action.yml": "runs:\n  steps:\n    - run: bash tools/setup.sh\n",
    "tools/setup.sh": "",
    "tools/win.py": "import helper\n",
    "tools/helper.py": "",
    "tools/other.py": "",
    "tools/differential/sweep.py": "",
    "app/main.jai": '#import "Mod";\n',
    "stdlib/Mod/module.jai": '#import "Dep";\n',
    "stdlib/Dep/module.jai": "",
    "stdlib/Other/module.jai": "",
    "stdlib/prelude/module.jai": "",
    "crates/app/Cargo.toml": '[package]\nname = "app"\n[dependencies]\nlib = { path = "../lib" }\n',
    "crates/lib/Cargo.toml": '[package]\nname = "lib"\n',
    "crates/unused/Cargo.toml": '[package]\nname = "unused"\n',
    "crates/app/src/main.rs": "",
    "crates/lib/src/lib.rs": "",
    "crates/unused/src/lib.rs": "",
    "Cargo.toml": '[workspace]\nmembers = ["crates/*"]\n',
    "Cargo.lock": "",
    "rust-toolchain.toml": "",
    "flake.nix": "{ src = ./nix/app.nix; }",
    "nix/app.nix": "{ fileset = [ ../crates ../Cargo.toml ]; }",
}


class FakeWorkflow(unittest.TestCase):
    """The derivation, on a workflow of its own: no hand-kept path appears in ci_changes.py."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        for name, text in FAKE.items():
            path = Path(self.tmp.name) / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
        self.repo = job_inputs.Repo(self.tmp.name, files=list(FAKE))
        self.crates = {"windows-native": ["app"], "nix": ["app"]}
        self.graph = job_inputs.crate_graph(self.repo)

    def runs(self, area, *paths):
        patterns = ci_changes.area_patterns(
            area, self.repo, JOBS_WORKFLOW, self.graph, crates=self.crates
        )
        return ci_changes.area_changed(area, patterns, list(paths))

    def test_workflow_paths_and_the_scripts_they_import(self):
        self.assertTrue(self.runs("windows", "tools/win.py"))
        self.assertTrue(self.runs("windows", "tools/helper.py"))
        self.assertTrue(self.runs("windows", "tools/setup.sh"))
        self.assertFalse(self.runs("windows", "tools/other.py"))

    def test_a_new_script_in_a_step_needs_no_edit(self):
        workflow = JOBS_WORKFLOW.replace(
            "python3 tools/win.py",
            "python3 tools/win.py\n          - run: bash tools/fresh.sh",
        )
        repo = job_inputs.Repo(self.tmp.name, files=list(FAKE) + ["tools/fresh.sh"])
        patterns = ci_changes.area_patterns(
            "windows", repo, workflow, self.graph, crates=self.crates
        )
        self.assertTrue(
            ci_changes.area_changed("windows", patterns, ["tools/fresh.sh"])
        )

    def test_stdlib_closure_follows_imports(self):
        self.assertTrue(self.runs("sdl3", "stdlib/Dep/module.jai"))
        self.assertTrue(self.runs("sdl3", "stdlib/prelude/module.jai"))
        self.assertFalse(self.runs("sdl3", "stdlib/Other/module.jai"))

    def test_a_new_import_needs_no_edit(self):
        (Path(self.tmp.name) / "stdlib/Dep/module.jai").write_text('#import "Other";\n')
        repo = job_inputs.Repo(self.tmp.name, files=list(FAKE))
        patterns = ci_changes.area_patterns("sdl3", repo, JOBS_WORKFLOW, self.graph)
        self.assertTrue(
            ci_changes.area_changed("sdl3", patterns, ["stdlib/Other/module.jai"])
        )

    def test_crate_closure_follows_path_dependencies(self):
        self.assertTrue(self.runs("windows", "crates/lib/src/lib.rs"))
        self.assertTrue(self.runs("windows", "Cargo.lock"))
        self.assertFalse(self.runs("windows", "crates/unused/src/lib.rs"))

    def test_whole_stdlib_jobs_run_for_any_stdlib_file(self):
        self.assertTrue(self.runs("windows", "stdlib/Other/module.jai"))

    def test_a_nix_file_names_what_it_reads(self):
        self.assertTrue(self.runs("nix", "nix/app.nix"))
        self.assertTrue(self.runs("nix", "crates/app/src/main.rs"))
        self.assertFalse(self.runs("nix", "tools/other.py"))

    def test_policy_narrows_the_sanitizer_leg(self):
        self.assertTrue(self.runs("sanitizers-macos", "tools/differential/sweep.py"))
        # tools/other.py is an input of the sanitizer job, but not of the macOS leg.
        self.assertFalse(self.runs("sanitizers-macos", "tools/other.py"))

    def test_the_workflow_runs_every_area_but_the_own_paths_only_ones(self):
        self.assertTrue(self.runs("nix", ".github/workflows/ci.yml"))
        self.assertFalse(self.runs("sdl3", ".github/workflows/ci.yml"))

    def test_inputs_that_cannot_be_computed_run_every_area(self):
        original = job_inputs.Repo
        job_inputs.Repo = lambda *a, **k: (_ for _ in ()).throw(
            RuntimeError("no checkout")
        )
        self.addCleanup(setattr, job_inputs, "Repo", original)
        got = ci_changes.area_results(["README.md"])
        self.assertEqual(got, {area: True for area in ci_changes.input_areas()})


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
    ALL = {area: True for area in ci_changes.input_areas()} | {"code": True}

    def test_every_area_has_a_job_and_every_job_a_known_area(self):
        areas = set(ci_changes.input_areas()) | {"code"}
        self.assertEqual({a for a in ci_changes.JOBS.values() if a}, areas)

    def test_every_computed_area_job_is_gated_in_job_inputs(self):
        jobs = {
            job
            for area in ci_changes.input_areas()
            for job in ci_changes.area_jobs(area)
        }
        self.assertLessEqual(jobs, job_inputs.GATED)

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
        self.assertEqual(
            sorted(gated), sorted(set(ci_changes.JOBS) - {"sanitizers-macos"})
        )


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
            self.assertEqual(
                {leg["parts"] for leg in legs}, {",rust,", ",targets,lint,diff,"}
            )
        macos = [leg for leg in tests if leg["runner"] in self.MACOS]
        macos += [leg for leg in stdlib if leg["platform"].startswith("macos")]
        self.assertEqual(len(macos), 8)
        for leg in macos:
            self.assertGreaterEqual(leg["timeout"], 120, leg)

    def test_every_test_leg_names_its_parts_with_delimiters(self):
        for full in (False, True):
            for leg in ci_changes.test_matrix(full):
                with self.subTest(leg=leg):
                    self.assertTrue(
                        leg["parts"].startswith(",") and leg["parts"].endswith(",")
                    )
                    # `checks` is the one leg that merges parts under a name of its own.
                    self.assertTrue(
                        f",{leg['part']}," in leg["parts"] or leg["part"] == "checks"
                    )

    def test_the_test_job_selects_steps_by_parts(self):
        from pathlib import Path

        text = (
            Path(__file__).resolve().parents[2] / ".github/workflows/ci.yml"
        ).read_text()
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
            self.assertEqual(
                {leg["runner"] for leg in legs if leg["part"] == part}, {"ubuntu-24.04"}
            )
        self.assertEqual(
            {leg["runner"] for leg in legs if leg["part"] == "split"},
            {"ubuntu-24.04-arm"},
        )
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
                    self.assertEqual(
                        "wasm-" in leg.get("modes", ""),
                        leg["platform"] == "linux-x64" and "modes" in leg,
                    )
                    self.assertEqual(
                        bool(leg.get("wasm")),
                        leg["platform"] == "linux-x64" and "jacket_test" in leg,
                    )

    def test_stdlib_coverage_is_measured_by_one_leg(self):
        for full in (False, True):
            with self.subTest(full=full):
                legs = [
                    leg for leg in ci_changes.stdlib_matrix(full) if leg.get("coverage")
                ]
                self.assertEqual(
                    [(leg["platform"], leg["shard"]) for leg in legs],
                    [("linux-x64", "suites")],
                )

    def test_jacket_test_is_divided_once_per_platform(self):
        for full in (False, True):
            legs = ci_changes.stdlib_matrix(full)
            for platform in {leg["platform"] for leg in legs}:
                with self.subTest(full=full, platform=platform):
                    shares = [
                        leg["jacket_test"]
                        for leg in legs
                        if leg["platform"] == platform and leg.get("jacket_test")
                    ]
                    if shares == ["all"]:
                        continue
                    count = ci_changes.TOOLING_PARTS
                    self.assertEqual(
                        shares, [f"hash:{k}/{count}" for k in range(1, count + 1)]
                    )


if __name__ == "__main__":
    unittest.main()
