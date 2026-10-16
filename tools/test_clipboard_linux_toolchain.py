"""Clipboard CI provisioning configuration checks; never install/run Rust or X11."""

from pathlib import Path
import os
import re
import shlex
import subprocess
import tempfile
import textwrap
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github/workflows/clipboard-linux.yml"


class ClipboardToolchainSetup(unittest.TestCase):
    def setUp(self):
        self.workflow = WORKFLOW.read_text()
        self.toolchain = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())[
            "toolchain"
        ]

    def job_env(self, name):
        match = re.search(
            r"^      " + re.escape(name) + r": (.+)$", self.workflow, re.MULTILINE
        )
        self.assertIsNotNone(match, "missing job-wide " + name)
        return match[1].strip("'\"")

    def install_command(self):
        commands = [
            shlex.split(line.strip())
            for line in self.workflow.splitlines()
            if line.strip().startswith("rustup toolchain install ")
        ]
        self.assertEqual(len(commands), 1)
        return commands[0]

    def test_installs_all_canonical_components_explicitly(self):
        command = self.install_command()
        self.assertEqual(
            command[:4], ["rustup", "toolchain", "install", self.toolchain["channel"]]
        )
        self.assertEqual(
            command[command.index("--profile") + 1], self.toolchain["profile"]
        )
        components = [
            command[i + 1] for i, arg in enumerate(command) if arg == "--component"
        ]
        self.assertCountEqual(components, self.toolchain["components"])
        self.assertRegex(self.toolchain["channel"], r"^nightly-\d{4}-\d{2}-\d{2}$")

    def test_state_is_persisted_job_wide_disjoint_and_unique_per_run_attempt(self):
        paths = []
        for name in ["RUSTUP_HOME", "CARGO_HOME"]:
            match = re.search(
                r'echo "' + name + r'=(.+)" >> "\$GITHUB_ENV"', self.workflow
            )
            self.assertIsNotNone(match, "missing persisted " + name)
            self.assertLess(
                match.start(), self.workflow.index("rustup toolchain install")
            )
            paths.append(match[1])
        self.assertNotEqual(*paths)
        for path in paths:
            self.assertTrue(path.startswith("$RUNNER_TEMP/clipboard-"), path)
            self.assertIn("$GITHUB_RUN_ID", path)
            self.assertIn("$GITHUB_RUN_ATTEMPT", path)
            self.assertNotIn("github.workspace", path)

    def test_every_later_cargo_proxy_uses_explicit_pinned_toolchain(self):
        self.assertEqual(self.job_env("RUSTUP_TOOLCHAIN"), self.toolchain["channel"])
        self.assertEqual(self.job_env("RUSTUP_AUTO_INSTALL"), "0")

    def test_actual_shell_publishes_only_the_two_isolated_state_paths(self):
        block = re.search(
            r"      - name: Select isolated Rust state for this job attempt\n"
            r"        run: \|\n((?:          [^\n]*\n)+)",
            self.workflow,
        )
        self.assertIsNotNone(block)
        base = os.environ.get("RUNNER_TEMP") or os.environ["TMPDIR"]
        with tempfile.TemporaryDirectory(
            prefix="clipboard-env-", dir=base
        ) as directory:
            temporary = Path(directory) / "runner temp"
            temporary.mkdir()
            output = Path(directory) / "github-env"
            result = subprocess.run(
                ["bash", "-euo", "pipefail", "-c", textwrap.dedent(block[1])],
                env={
                    "PATH": os.defpath,
                    "RUNNER_TEMP": str(temporary),
                    "GITHUB_ENV": str(output),
                    "GITHUB_RUN_ID": "731",
                    "GITHUB_RUN_ATTEMPT": "3",
                },
                capture_output=True,
                text=True,
                timeout=5,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(
                output.read_text().splitlines(),
                [
                    "RUSTUP_HOME=" + str(temporary / "clipboard-rustup-731-3"),
                    "CARGO_HOME=" + str(temporary / "clipboard-cargo-731-3"),
                ],
            )
            self.assertEqual(list(temporary.iterdir()), [])

    def test_installed_components_are_checked_before_fetch(self):
        pin = self.toolchain["channel"]
        for command in [
            f"rustup run {pin} rustfmt --version",
            f"rustup run {pin} cargo clippy --version",
        ]:
            self.assertIn(command, self.workflow)
            self.assertLess(
                self.workflow.index(command),
                self.workflow.index("cargo fetch --locked"),
            )

    def test_fresh_state_is_required_without_deleting_runner_files(self):
        self.assertIn('test ! -e "$RUSTUP_HOME"', self.workflow)
        self.assertIn('test ! -e "$CARGO_HOME"', self.workflow)
        self.assertLess(
            self.workflow.index('test ! -e "$RUSTUP_HOME"'),
            self.workflow.index("rustup toolchain install"),
        )
        for line in self.workflow.splitlines():
            command = line.strip()
            self.assertFalse(
                re.match(r"(?:sudo\s+)?(?:rm|mv|ln|cp)\s", command), command
            )
        self.assertNotIn("continue-on-error", self.workflow)
        self.assertNotIn("|| true", self.workflow)

    def test_source_and_real_display_dependencies_remain_required(self):
        self.assertIn(
            "sudo apt-get install -y libx11-dev xvfb xauth clang-format", self.workflow
        )
        self.assertIn("bash tools/install_ci_llvm_linux.sh", self.workflow)
        self.assertIn("cargo fetch --locked", self.workflow)

    def test_real_runtime_and_relocated_package_steps_are_byte_preserved(self):
        expected = """      - name: Prepare the real Linux runtime helper
        run: python3 tools/build_portable_runtime.py
      - name: Real X11 clients, Jai runtime and relocated package
        run: |
          mkdir -p "$CARGO_TARGET_DIR"
          xvfb-run -a python3 tools/check_portable_clipboard.py --linux-runtime | tee "$CARGO_TARGET_DIR/runtime-proof.log"
          receipt="$(tail -n 1 "$CARGO_TARGET_DIR/runtime-proof.log")"
          xvfb-run -a python3 tools/check_clipboard_linux_package.py --verification-receipt "$receipt" --output "$CARGO_TARGET_DIR/package-proof"
"""
        start = self.workflow.index(
            "      - name: Prepare the real Linux runtime helper"
        )
        self.assertEqual(self.workflow[start:], expected)

    def test_pin_and_regression_changes_trigger_this_workflow(self):
        for path in ["rust-toolchain.toml", "tools/test_clipboard_linux_toolchain.py"]:
            self.assertIn("      - '" + path + "'", self.workflow)
        self.assertIn(
            "python3 -m unittest discover -s tools -p test_clipboard_linux_toolchain.py -v",
            self.workflow,
        )


if __name__ == "__main__":
    unittest.main()
