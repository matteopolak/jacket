"""Configuration regressions; these do not execute a Windows compiler or DLL.

Run against an explicitly selected accepted/staged workflow, never an old local
producer workflow. Native provider loading is checked by the workflow itself.
"""

import os
from pathlib import Path
import re
import unittest


class WindowsProviderEnvironment(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.workflow = Path(os.environ["WINDOWS_PROVIDER_TEST_WORKFLOW"]).read_text(
            encoding="utf-8"
        )

    def job(self, name):
        match = re.search(
            r"^  " + re.escape(name) + r":\n(.*?)(?=^  [\w-]+:|\Z)",
            self.workflow,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match, name)
        return match.group(1)

    def test_each_native_consumer_builds_matching_provider_before_consumption(self):
        for job, cpu, consumer in [
            ("native", "x64", "Platform I/O (interpreter and native O0/O2)"),
            ("native-arm64", "arm64", "Platform I/O (interpreter and native O0/O2)"),
            ("upstream-smoke", "x64", "Build and run the projects"),
        ]:
            with self.subTest(job=job):
                body = self.job(job)
                build = f"python -u tools/build_platform_io_runtime.py --stdlib stdlib --cpu {cpu}"
                self.assertIn(build, body)
                self.assertLess(body.index(build), body.index("- name: " + consumer))
                audit = body[body.index(build) :].splitlines()[1]
                self.assertIn("sha256sum", audit)
                self.assertIn("platform_io_windows_crash.dll", audit)
                self.assertIn("platform_io_windows_crash.lib", audit)
                self.assertIn(
                    'cygpath -w "$PWD/stdlib/internal/runtime/crash-native" >> "$GITHUB_PATH"',
                    body,
                )
                self.assertLess(
                    body.index(build),
                    body.index('>> "$GITHUB_PATH"', body.index(build)),
                )

    def test_normal_loader_probe_runs_in_subsequent_step_before_consumers(self):
        for job, export in [
            ("native", "platform_io_windows_crash_prepare"),
            ("upstream-smoke", "platform_io_windows_crash_prepare"),
            ("native-arm64", "platform_io_arm64_crash_init"),
        ]:
            with self.subTest(job=job):
                body = self.job(job)
                probe = "- name: Check normal Windows runtime provider loading"
                self.assertIn(probe, body)
                block = body[body.index(probe) :].split("      - name:", 1)[0]
                triple = "aarch64" if job == "native-arm64" else "x86_64"
                self.assertIn(f"clang --target={triple}-pc-windows-msvc", block)
                self.assertIn("tests/native-provider/windows-provider-load.c", block)
                self.assertIn(
                    'target/provider-load/provider-load.exe "$(cygpath -m "$PWD/stdlib/internal/runtime/crash-native/platform_io_windows_crash.dll")" '
                    + export,
                    block,
                )
                self.assertNotIn("cp ", block)
                self.assertNotIn("except", block)
                self.assertNotIn("continue-on-error", block)
                self.assertNotIn("if:", block)

    def test_probe_uses_real_loader_file_identity_and_does_not_invoke_crash_handlers(
        self,
    ):
        source = (
            Path(__file__).resolve().parents[1]
            / "tests/native-provider/windows-provider-load.c"
        ).read_text()
        self.assertIn('LoadLibraryW(L"platform_io_windows_crash.dll")', source)
        self.assertIn("GetProcAddress(provider, export_name)", source)
        self.assertIn("GetFileInformationByHandle", source)
        self.assertIn("nFileIndexLow != expected_info.nFileIndexLow", source)
        self.assertNotIn("SetDllDirectory", source)
        self.assertNotIn("SetDefaultDllDirectories", source)

    def test_consumer_failure_gates_and_real_graphics_checks_are_retained(self):
        for job in ["native", "native-arm64"]:
            body = self.job(job)
            self.assertIn("--test native -- --test-threads 2", body)
            self.assertIn("tools/window_render_wgl_ci.py", body)
            self.assertIn("Public renderer wheel propagation native O0 (issue 1)", body)
            self.assertIn("Public renderer wheel propagation native O2 (issue 1)", body)
            self.assertNotIn("MESA_GL_VERSION_OVERRIDE", body)
            self.assertNotIn("MESA_GLSL_VERSION_OVERRIDE", body)


if __name__ == "__main__":
    unittest.main()
