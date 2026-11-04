import ast
from collections import Counter
import json
import re
import tempfile
import textwrap
import unittest
from pathlib import Path

import ci_changes
import job_inputs
from toolkit.repo import ROOT

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
    def test_checkout_attributes_select_all_actual_consuming_areas(self):
        # Matching must not depend on whether a new/deleted nested policy is in Repo.files.
        for path in [
            ".gitattributes",
            "stdlib/new/.gitattributes",
            "deleted/deep/.gitattributes",
            ".github/actions/new/.gitattributes",
        ]:
            for area in ci_changes.input_areas():
                with self.subTest(path=path, area=area):
                    self.assertTrue(selects(area, path))

    def test_document_checkout_policies_select_the_main_code_job(self):
        import contextlib
        import io
        from unittest import mock

        for path in ["docs/.gitattributes", "docs/tools/.gitattributes", "docs/new/deleted/.gitattributes"]:
            with self.subTest(path=path):
                output = io.StringIO()
                with mock.patch.object(ci_changes, "changed_files", return_value=[path]), mock.patch.dict("os.environ", {"GITHUB_EVENT_NAME": "pull_request", "JOBS": "test"}, clear=True), contextlib.redirect_stdout(output):
                    ci_changes.main()
                lines = output.getvalue().splitlines()
                self.assertIn("code=true", lines)
                jobs = next(line.removeprefix("jobs=") for line in lines if line.startswith("jobs="))
                self.assertEqual(json.loads(jobs), ["test"])

    def test_attributes_keep_windows_pr_and_manual_request_policy(self):
        import contextlib
        import io
        from unittest import mock

        for event, requested, expected in [
            ("pull_request", "windows-native", []),
            ("workflow_dispatch", "windows-native", ["windows-native"]),
            ("workflow_dispatch", "nix", ["nix"]),
        ]:
            with self.subTest(event=event, requested=requested):
                output = io.StringIO()
                with (
                    mock.patch.object(
                        ci_changes, "changed_files", return_value=[".gitattributes"]
                    ),
                    mock.patch.dict(
                        "os.environ",
                        {"GITHUB_EVENT_NAME": event, "JOBS": requested},
                        clear=True,
                    ),
                    contextlib.redirect_stdout(output),
                ):
                    ci_changes.main()
                jobs = next(
                    line.removeprefix("jobs=")
                    for line in output.getvalue().splitlines()
                    if line.startswith("jobs=")
                )
                self.assertEqual(json.loads(jobs), expected)

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
            "crates/jaic-native/src/link.rs",
            "crates/jaic/src/abi.rs",
            "crates/jaic/src/toolchain/discover.rs",
            "crates/jaic/src/ir/lower.rs",
            "crates/jacket/src/main.rs",
            "crates/jaic-cli-args/src/options.rs",
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

    def test_toml_runner_and_pins_select_native_windows(self):
        for path in [
            "tools/stdlib/toml/run.py",
            "tools/stdlib/toml/adapter.jai",
            "tools/stdlib/toml/pin.json",
            "tools/stdlib/toml/LICENSE-toml-test",
        ]:
            with self.subTest(path=path):
                self.assertTrue(selects("windows", path))

    def test_toml_profiles_select_each_native_platform_once(self):
        jobs = job_inputs.jobs_of((ROOT / job_inputs.WORKFLOW).read_text())

        def predicate(job, step):
            tail = jobs[job].split("- name: " + step + "\n", 1)[1]
            return re.search(r"if: (.+)", tail).group(1)

        def selected(expression, row):
            # Evaluate only literal booleans/comparisons after replacing matrix references.
            expression = expression.removeprefix("${{ ").removesuffix(" }}")
            expression = expression.replace("!cancelled()", "True").replace("always()", "True")
            expression = re.sub(r"startsWith\(matrix\.(\w+), '([^']*)'\)", lambda m: repr(str(row.get(m[1], "")).startswith(m[2])), expression)
            expression = re.sub(r"matrix\.(\w+)", lambda m: repr(row.get(m[1], "")), expression)
            tree = ast.parse(expression.replace("&&", "and").replace("||", "or"), mode="eval")
            allowed = (ast.Expression, ast.Constant, ast.BoolOp, ast.And, ast.Or, ast.Compare, ast.Eq)
            self.assertTrue(all(isinstance(node, allowed) for node in ast.walk(tree)))
            return eval(compile(tree, "<matrix predicate>", "eval"), {"__builtins__": {}}, {})

        native = predicate("stdlib-runtime", "Complete pinned TOML decoder and encoder conformance")
        windows = predicate("windows-stdlib", "Complete pinned TOML decoder and encoder conformance")
        wasi = predicate("stdlib-runtime", "Complete pinned TOML conformance under WASI")
        for full in (False, True):
            with self.subTest(full=full):
                rows = ci_changes.stdlib_matrix(full)
                expected = {"linux-x64", "macos-arm64"} | ({"linux-arm64", "macos-x64"} if full else set())
                self.assertEqual(Counter(row["platform"] for row in rows if selected(native, row)), Counter({platform: 1 for platform in expected}))
                self.assertEqual(Counter(row["platform"] for row in rows if selected(wasi, row)), Counter({"linux-x64": 1}))
                expected_cpu = {"x64", "arm64"} if full else {"x64"}
                self.assertEqual(Counter(row["cpu"] for row in ci_changes.windows_stdlib_matrix(full) if selected(windows, row)), Counter({cpu: 1 for cpu in expected_cpu}))

    def test_focused_images_intel_tracks_the_inputs_it_actually_checks(self):
        repo = job_inputs.Repo()
        patterns = job_inputs.inputs("images-ui-intel", repo)
        for path in (
            "stdlib/Extensions/UI/render.jai",
            "stdlib/Extensions/Image/module.jai",
            "stdlib/Ico_File/tests/filename.jai",
            "stdlib/internal/resource-wire/ico-load.jai",
            "stdlib/internal/cpu/module.jai",
            "stdlib/Extensions/SIMD/tests/cpu-dispatch.jai",
            "stdlib/Extensions/Web/template.jai",
            "tools/ci/check_images_ui.py",
        ):
            with self.subTest(path=path):
                self.assertTrue(ci_changes.matches(patterns, [path], workflow=False))

    def test_sdl3_ignores_the_workflow_and_actions_but_not_its_own_inputs(self):
        self.assertTrue(selects("sdl3", "stdlib/Extensions/SDL3/module.jai"))
        self.assertTrue(selects("sdl3", "tools/bindings/sdl3_gen.py"))
        self.assertFalse(selects("sdl3", ".github/workflows/ci.yml"))
        self.assertFalse(selects("sdl3", ".github/actions/setup-unix/action.yml"))
        self.assertFalse(selects("sdl3", "stdlib/Sound_Player/module.jai"))

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

    def test_compiler_argument_metadata_triggers_macos_sanitizers(self):
        for module in ("arguments", "cli", "options", "tuning"):
            path = f"crates/jaic-cli-args/src/{module}.rs"
            with self.subTest(path=path):
                self.assertTrue(selects("sanitizers-macos", path))

    def test_macos_asan_runs_on_pushes_only_for_sanitizer_relevant_paths(self):
        self.assertTrue(selects("sanitizers-macos", "crates/jaic-llvm/src/lib.rs"))
        self.assertTrue(selects("sanitizers-macos", "crates/jaic-native/src/link.rs"))
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

    def test_narrow_policy_does_not_invent_a_checkout_dependency(self):
        self.assertFalse(self.runs("sanitizers-macos", ".gitattributes"))
        self.assertFalse(self.runs("sanitizers-macos", "new/.gitattributes"))

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

    def test_automatic_jobs_run_when_all_areas_do(self):
        expected = [job for job in ci_changes.JOBS if job not in ci_changes.MANUAL_ONLY]
        self.assertEqual(ci_changes.selected_jobs(self.ALL, ""), expected)

    def test_focused_intel_requires_an_explicit_manual_name(self):
        self.assertNotIn("images-ui-intel", ci_changes.selected_jobs(self.ALL, ""))
        self.assertNotIn("images-ui-intel", ci_changes.selected_jobs(self.ALL, "test,docs"))
        self.assertEqual(
            ci_changes.selected_jobs(self.ALL, "images-ui-intel"), ["images-ui-intel"]
        )

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
        import re

        text = (
            ROOT / ".github/workflows/ci.yml"
        ).read_text()
        gated = re.findall(
            r"contains\(fromJSON\(needs\.changes\.outputs\.jobs\), '([a-z0-9-]+)'\)",
            text,
        )
        # `sanitizers-macos` is no job: main() reads it to add the macOS leg to sanitizer_matrix.
        self.assertEqual(
            sorted(gated), sorted(set(ci_changes.JOBS) - {"sanitizers-macos"})
        )


class SanitizerModes(unittest.TestCase):
    def test_full_macos_modes_cover_every_existing_sweep_once(self):
        expected = Counter({
            ("O0", "corpus"): 1, ("O0", "regressions"): 1,
            ("O0", "negative"): 1, ("O0", "stdlib"): 1,
            ("O2", "corpus"): 1, ("O2", "regressions"): 1,
            ("O2", "stdlib"): 1,
        })
        for selected in (False, True):
            legs = [leg for leg in ci_changes.sanitizer_matrix(True, selected) if leg["runner"] == "macos-15"]
            self.assertEqual(len(legs), 2)
            self.assertEqual({leg["mode"] for leg in legs}, {"O0", "O2"})
            actual = Counter()
            for leg in legs:
                self.assertEqual(leg["shard"], "1/1")
                mode = leg["mode"]
                for suite in ["corpus", "regressions", "stdlib", *(["negative"] if mode == "O0" else [])]:
                    actual[mode, suite] += 1
            self.assertEqual(actual, expected)

    def test_fast_mac_and_linux_assignments_are_unchanged(self):
        for full in (False, True):
            for selected in (False, True):
                legs = ci_changes.sanitizer_matrix(full, selected)
                count = 2 if full else 3
                self.assertEqual(
                    [leg for leg in legs if leg["runner"] == "ubuntu-24.04"],
                    [{"runner": "ubuntu-24.04", "shard": f"{index}/{count}"} for index in range(1, count + 1)],
                )
                if not full:
                    self.assertEqual(
                        [leg for leg in legs if leg["runner"] == "macos-15"],
                        [{"runner": "macos-15", "shard": "1/1"}] if selected else [],
                    )
                keys = [(leg["runner"], leg["shard"], leg.get("mode") == "O2") for leg in legs]
                self.assertEqual(len(keys), len(set(keys)), keys)

    def test_workflow_modes_keep_commands_limits_and_unique_names(self):
        text = (ROOT / ".github/workflows/ci.yml").read_text()
        job = text[text.index("\n  sanitizers:\n"):text.index("\n  sdl3:\n")]
        self.assertIn("timeout-minutes: 45", job)
        self.assertIn("${{ matrix.mode && format(', {0}', matrix.mode) || '' }}", job)
        self.assertIn("${{ matrix.mode == 'O2' && '-O2' || '' }}", job)
        steps = {}
        for block in job.split("      - name: ")[1:]:
            steps[block.splitlines()[0]] = block
        for mode, opposite in [("O0", "O2"), ("O2", "O0")]:
            corpus = steps[f"Corpus and compiler tests under ASan and UBSan (-{mode})"]
            stdlib = steps[f"Stdlib module tests under ASan and UBSan (-{mode})"]
            self.assertIn(f"matrix.mode != '{opposite}'", corpus)
            self.assertIn(f"matrix.mode != '{opposite}'", stdlib)
            command = "python3 tools/differential/jaic_sweep.py --jaic target/debug/jaic --jobs 4 --timeout 300 "
            command += f"--opt {mode} --headless --sanitize address,undefined --shard "
            command += "${{ matrix.shard }} corpus regressions" + (" negative" if mode == "O0" else "")
            self.assertEqual(corpus.split("        run: ", 1)[1].splitlines()[0], command)
            self.assertEqual(stdlib.split("        run: ", 1)[1].splitlines()[0], f"tools/ci/stdlib_jacket_sanitize.sh {mode} " + "${{ matrix.shard }}")
            self.assertIn("always() && !cancelled() && steps.build-jaic.outcome == 'success'", stdlib)
            if mode == "O2":
                self.assertIn("needs.changes.outputs.full == 'true'", corpus)
                self.assertIn("needs.changes.outputs.full == 'true'", stdlib)


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

    def test_full_intel_target_work_is_shared_without_sharding_native_rust(self):
        legs = [row for row in ci_changes.test_matrix(True) if row["runner"] == "macos-15-intel"]
        self.assertEqual([row["part"] for row in legs], ["rust", "checks"])
        self.assertEqual([row["shard"] for row in legs], ["1/1", "1/1"])
        self.assertEqual([row.get("targets_part") for row in legs], ["2/2", "1/2"])
        self.assertTrue(all(",targets," in row["parts"] for row in legs))
        self.assertEqual(legs[0]["expr"], ci_changes.NOT_WASM)
        self.assertEqual([row["timeout"] for row in legs], [150, 150])
        for full in (False, True):
            for row in ci_changes.test_matrix(full):
                if not full or row["runner"] != "macos-15-intel":
                    self.assertNotIn("targets_part", row)

    def test_each_target_and_doc_share_is_complete_and_unique(self):
        for full in (False, True):
            groups = {}
            for row in ci_changes.test_matrix(full):
                if ",targets," in row["parts"]:
                    groups.setdefault(row["runner"], []).append(row.get("targets_part", row["shard"]))
            for runner, shares in groups.items():
                with self.subTest(full=full, runner=runner):
                    self.assertEqual(Counter(shares), Counter(ci_changes.shards(len(shares))))

    def test_target_steps_use_their_share_and_native_rust_keeps_its_shard(self):
        text = (ROOT / ".github/workflows/ci.yml").read_text()
        job = text[text.index("\n  test:\n"):text.index("\n  vscode-extension:\n")]
        self.assertIn("JACKET_STDLIB_TARGETS_PART: ${{ matrix.targets_part || matrix.shard }}", job)
        self.assertIn("JACKET_PART: ${{ matrix.targets_part || matrix.shard }}", job)
        self.assertIn('--partition "slice:${{ matrix.shard }}"', job)
        self.assertIn('[[ "$JACKET_STDLIB_TARGETS_PART" == 1/* ]] && objc=(--test objc_selectors)', job)

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
            rust_parts = ",rust,targets," if runner == "macos-15-intel" else ",rust,"
            self.assertEqual(
                {leg["parts"] for leg in legs}, {rust_parts, ",targets,lint,diff,"}
            )
        macos = [leg for leg in tests if leg["runner"] in self.MACOS]
        macos += [leg for leg in stdlib if leg["platform"].startswith("macos")]
        self.assertEqual(len(macos), 8)
        for leg in macos:
            self.assertGreaterEqual(leg["timeout"], 120, leg)

    def test_full_intel_producer_work_uses_both_existing_jobs(self):
        legs = [
            row
            for row in ci_changes.stdlib_matrix(True)
            if row["platform"] == "macos-x64"
        ]
        self.assertEqual(len(legs), 2)
        catalogue = next(row for row in legs if row["shard"] == "1/1")
        runtime = next(row for row in legs if row["shard"] == "suites")
        self.assertEqual(
            set(catalogue.get("suites", "").split()), {"tooling", "bindings"}
        )
        self.assertEqual(
            set(runtime["suites"].split()), {"portable", "platform", "audio", "window"}
        )
        self.assertEqual(
            Counter(suite for row in legs for suite in row.get("suites", "").split()),
            Counter(
                {
                    suite: 1
                    for suite in (
                        "portable",
                        "platform",
                        "audio",
                        "window",
                        "bindings",
                        "tooling",
                    )
                }
            ),
        )
        self.assertEqual((catalogue["modes"], catalogue["o2"]), ("interp,native", True))
        self.assertEqual(runtime["jacket_test"], "all")
        self.assertEqual([row["timeout"] for row in legs], [150, 150])

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

        text = (
            ROOT / ".github/workflows/ci.yml"
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
                    parts = [share.removeprefix("hash:").split("/") for share in shares]
                    self.assertTrue(all(share.startswith("hash:") for share in shares))
                    denominators = {int(n) for _, n in parts}
                    self.assertEqual(len(denominators), 1)
                    count = denominators.pop()
                    numerators = [int(k) for k, _ in parts]
                    self.assertEqual(sorted(numerators), list(range(1, count + 1)))
                    self.assertEqual(len(numerators), len(set(numerators)))



class StdlibNarrowing(unittest.TestCase):
    PR = {"GITHUB_EVENT_NAME": "pull_request"}

    def narrowed(self, files, env=None):
        return ci_changes.stdlib_changed(files, env or self.PR)

    def test_module_sources_and_golden_outputs_narrow(self):
        files = ["stdlib/Sort/module.jai", "stdlib/Sort/tests/a.stdout"]
        self.assertEqual(json.loads(self.narrowed(files)), files)

    def test_anything_else_runs_everything(self):
        for other in [
            "crates/jaic/src/lib.rs",
            ".github/workflows/ci.yml",
            "stdlib/Sort/native.c",
            "jacket.toml",
            "tools/ci/select_tests.py",
        ]:
            with self.subTest(other=other):
                self.assertEqual(self.narrowed(["stdlib/Sort/module.jai", other]), "")

    def test_main_and_staging_pushes_run_everything(self):
        for ref in ["refs/heads/main", "refs/heads/staging"]:
            env = {"GITHUB_EVENT_NAME": "push", "GITHUB_REF": ref}
            self.assertEqual(self.narrowed(["stdlib/Sort/module.jai"], env), "")

    def test_a_branch_push_narrows(self):
        env = {"GITHUB_EVENT_NAME": "push", "GITHUB_REF": "refs/heads/topic"}
        self.assertNotEqual(self.narrowed(["stdlib/Sort/module.jai"], env), "")

    def test_scheduled_manual_and_undiffable_runs_run_everything(self):
        for event in ["schedule", "workflow_dispatch", ""]:
            env = {"GITHUB_EVENT_NAME": event}
            self.assertEqual(self.narrowed(["stdlib/Sort/module.jai"], env), "")
        self.assertEqual(self.narrowed(None), "")
        self.assertEqual(self.narrowed([]), "")

    def test_a_large_change_runs_everything(self):
        files = [f"stdlib/M{k}/module.jai" for k in range(101)]
        self.assertEqual(self.narrowed(files), "")


if __name__ == "__main__":
    unittest.main()
