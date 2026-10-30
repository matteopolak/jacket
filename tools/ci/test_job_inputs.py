import tempfile
import textwrap
import unittest
from pathlib import Path

import job_inputs as ji

WORKFLOW = textwrap.dedent(
    """\
    name: ci
    jobs:
      changes:
        steps:
          - run: python3 tools/ci/ci_changes.py
      build:
        # tools/ignored.py is only a comment.
        steps:
          - uses: ./.github/actions/setup
            with:
              path: stdlib/Mod
          - run: |
              python3 tools/build.py --out target/x
              bash tools/step.sh
              python3 tools/run_tests.py -p 'test_a*.py' ci
              python3 -m unittest discover -s tools/ci -p test_b.py
              cargo build --locked -p app -p other
          - shell: powershell
            run: .\\tools\\win\\run.ps1
      other-job:
        steps:
          - run: echo nothing
    """
)

FILES = {
    ".github/actions/setup/action.yml": "runs:\n  steps:\n    - run: bash tools/setup.sh\n    - uses: ./.github/actions/inner\n",
    ".github/actions/inner/action.yml": "runs:\n  steps:\n    - run: python3 tools/inner.py\n",
    "tools/build.py": 'import helper\nfrom lib import util\nimport os\nDATA = "tools/data.json"\n"""x"""\n',
    "tools/helper.py": "",
    "tools/lib/util.py": "import deeper\n",
    "tools/lib/deeper.py": "",
    "tools/data.json": "{}",
    "tools/step.sh": '. "$(dirname "$0")/common.sh"\npython3 tools/sub.py\n# tools/comment.py\n',
    "tools/common.sh": "",
    "tools/sub.py": "",
    "tools/setup.sh": "",
    "tools/inner.py": "",
    "tools/ignored.py": "",
    "tools/comment.py": "",
    "tools/win/run.ps1": "& tools\\win\\inner.ps1\n",
    "tools/win/inner.ps1": "",
    "tools/test_a1.py": "",
    "tools/ci/test_b.py": "",
    "tools/ci/test_other.py": "",
    "tools/ci/ci_changes.py": "",
    "stdlib/Mod/module.jai": '#import "Dep";\n#load "part.jai";\n',
    "stdlib/Mod/part.jai": "",
    "stdlib/Dep/module.jai": '#import,file "../Deep/module.jai";\n',
    "stdlib/Deep/module.jai": "",
    "stdlib/Unrelated/module.jai": "",
    "stdlib/prelude/module.jai": "",
    "crates/app/Cargo.toml": '[package]\nname = "app"\n[dependencies]\nlib = { path = "../lib" }\nserde = "1"\n'
    "[target.'cfg(unix)'.dependencies]\nunix = { workspace = true }\n",
    "crates/lib/Cargo.toml": '[package]\nname = "lib"\n[dev-dependencies]\nhelp = { path = "../help" }\n',
    "crates/help/Cargo.toml": '[package]\nname = "help"\n',
    "crates/unix/Cargo.toml": '[package]\nname = "unix"\n',
    "crates/other/Cargo.toml": '[package]\nname = "other"\n',
    "crates/unused/Cargo.toml": '[package]\nname = "unused"\n',
    "Cargo.toml": '[workspace]\nmembers = ["crates/*"]\n[workspace.dependencies]\nunix = { path = "crates/unix" }\n',
    "Cargo.lock": "",
    "rust-toolchain.toml": "",
    ".github/workflows/ci.yml": WORKFLOW,
    "install.sh": "",
}


class Fake(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        for name, text in FILES.items():
            path = Path(self.tmp.name) / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
        self.repo = ji.Repo(self.tmp.name, files=list(FILES))

    def inputs(self, job="build", **kw):
        kw.setdefault("crates", {})
        return ji.inputs(job, self.repo, WORKFLOW, **kw)


class WorkflowPaths(Fake):
    def test_run_lines_with_inputs_and_actions(self):
        got = self.inputs()
        self.assertIn("tools/build.py", got)
        self.assertIn("tools/step.sh", got)
        self.assertIn("stdlib/Mod/**", got)
        self.assertIn(".github/actions/setup/**", got)
        # Composite actions are followed recursively.
        self.assertIn("tools/setup.sh", got)
        self.assertIn("tools/inner.py", got)
        self.assertIn(".github/actions/inner/**", got)

    def test_comments_and_other_jobs_are_ignored(self):
        got = self.inputs()
        self.assertNotIn("tools/ignored.py", got)
        self.assertNotIn("tools/ci/ci_changes.py", got)
        self.assertEqual(self.inputs("other-job"), set())

    def test_windows_paths_in_a_step(self):
        self.assertIn("tools/win/run.ps1", self.inputs())

    def test_test_runner_arguments_stand_for_the_tests(self):
        got = self.inputs()
        self.assertIn("tools/**/test_a*.py", got)
        self.assertIn("tools/ci/**", got)
        self.assertNotIn("tools/run_tests.py", got)
        self.assertIn("tools/ci/test_b.py", got)
        self.assertNotIn("tools/ci/test_other.py", got)


class ScriptClosure(Fake):
    def test_python_imports_and_path_strings(self):
        got = self.inputs()
        for path in (
            "tools/helper.py",
            "tools/lib/util.py",
            "tools/lib/deeper.py",
            "tools/data.json",
        ):
            self.assertIn(path, got)

    def test_shell_siblings_and_paths_but_not_comments(self):
        got = self.inputs()
        self.assertIn("tools/common.sh", got)
        self.assertIn("tools/sub.py", got)
        self.assertNotIn("tools/comment.py", got)

    def test_powershell_paths(self):
        self.assertIn("tools/win/inner.ps1", self.inputs())

    def test_a_new_script_called_from_a_step_needs_no_edit(self):
        changed = WORKFLOW.replace(
            "bash tools/step.sh",
            "bash tools/step.sh\n              python3 tools/fresh.py",
        )
        self.repo = ji.Repo(self.tmp.name, files=list(FILES) + ["tools/fresh.py"])
        got = ji.inputs("build", self.repo, changed, crates={})
        self.assertIn("tools/fresh.py", got)

    def test_const_runs_join_path_pieces(self):
        files = dict(FILES) | {
            "tools/build.py": 'from pathlib import Path\nX = Path(__file__) / "tools" / "data.json"\n'
        }
        (Path(self.tmp.name) / "tools/build.py").write_text(files["tools/build.py"])
        self.assertIn("tools/data.json", self.inputs())


class StdlibClosure(Fake):
    def test_a_stdlib_folder_follows_its_imports_and_loads(self):
        got = self.inputs()
        self.assertIn("stdlib/Dep/module.jai", got)
        self.assertIn("stdlib/Deep/module.jai", got)
        self.assertIn("stdlib/prelude/module.jai", got)
        self.assertNotIn("stdlib/Unrelated/module.jai", got)

    def test_a_new_import_needs_no_edit(self):
        (Path(self.tmp.name) / "stdlib/Mod/module.jai").write_text(
            '#import "Unrelated";\n'
        )
        self.repo = ji.Repo(self.tmp.name, files=list(FILES))
        self.assertIn("stdlib/Unrelated/module.jai", self.inputs())

    def test_whole_stdlib_jobs_declare_the_folder(self):
        got = self.inputs(extra={"build": ["stdlib/**"]})
        self.assertIn("stdlib/**", got)
        self.assertEqual(ji.collapse(got).count("stdlib/**"), 1)
        self.assertNotIn("stdlib/Dep/module.jai", ji.collapse(got))


class Crates(Fake):
    def test_path_dependencies_close_over_the_workspace(self):
        graph = ji.crate_graph(self.repo)
        self.assertEqual(graph["app"], ("crates/app", ["lib", "unix"]))
        got = self.inputs(crates={"build": ["app"]}, graph=graph)
        for pattern in (
            "crates/app/**",
            "crates/lib/**",
            "crates/help/**",
            "crates/unix/**",
            "Cargo.lock",
            "rust-toolchain.toml",
        ):
            self.assertIn(pattern, got)
        self.assertNotIn("crates/unused/**", got)
        self.assertNotIn("crates/other/**", got)

    def test_cargo_p_outside_the_table_is_drift(self):
        self.assertEqual(
            ji.drift(WORKFLOW, {"build": ["app"]}),
            ["build: cargo -p other is not in CRATES['build']"],
        )
        self.assertEqual(ji.drift(WORKFLOW, {"build": ["app", "other"]}), [])
        self.assertEqual(ji.drift(WORKFLOW, {}, only={"other-job"}), [])


class RealWorkflow(unittest.TestCase):
    def test_the_crate_table_covers_every_cargo_p(self):
        text = (ji.ROOT / ji.WORKFLOW).read_text()
        self.assertEqual(ji.drift(text, only=ji.GATED), [])

    def test_gated_jobs_exist_and_need_no_table_row_without_cargo(self):
        jobs = ji.jobs_of((ji.ROOT / ji.WORKFLOW).read_text())
        self.assertLessEqual(ji.GATED, set(jobs))
        self.assertLessEqual(set(ji.CRATES), set(jobs))

    def test_a_stdlib_math_change_is_an_input_of_the_windows_stdlib_job(self):
        got = ji.inputs("windows-stdlib")
        self.assertIn("stdlib/**", got)
        self.assertIn("crates/jacket/**", got)
        self.assertIn("crates/jaic-llvm/**", got)

    def test_every_workspace_crate_is_known(self):
        graph = ji.crate_graph(ji.Repo())
        for names in ji.CRATES.values():
            self.assertLessEqual(set(names), set(graph))


if __name__ == "__main__":
    unittest.main()
