"""Windows cross-runner accounting tests; mocked commands are not native proof."""

import contextlib
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import windows_cross as cross


class Selection(unittest.TestCase):
    def test_cached_filenames_without_source_paired_abi8_receipt_never_qualify(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "libbinding_callback_context.so").write_bytes(b"metadata fixture")
            self.assertFalse(
                cross.verified_cross_host_bindings(
                    root, [], {"binding_callback_context"}, ".so"
                )
            )
            receipt = root / "ci-provider-receipt.json"
            receipt.write_text(json.dumps({"native_abi": 7}))
            self.assertFalse(
                cross.verified_cross_host_bindings(
                    root, [], {"binding_callback_context"}, ".so"
                )
            )
            receipt.write_text(
                json.dumps({"native_abi": 8, "profiles": [], "source_inputs": {}})
            )
            self.assertFalse(
                cross.verified_cross_host_bindings(
                    root, [], {"binding_callback_context"}, ".so"
                )
            )

    def test_original_overloaded_ifx_is_only_exact_reference_rejection(self):
        source = cross.ROOT / "tests/compiler-regressions/proc-type-defaults-ifx.jai"
        case = {"id": "compiler/proc-type-defaults-ifx", "path": source}
        self.assertIsNotNone(cross.fixture_reference_rejected_reason(case))
        independent = dict(case, id="producer/ifx-official-selection")
        self.assertIsNone(cross.fixture_reference_rejected_reason(independent))
        with tempfile.TemporaryDirectory() as directory:
            changed = Path(directory) / "changed.jai"
            changed.write_text("main :: () {}\n")
            self.assertIsNone(
                cross.fixture_reference_rejected_reason(dict(case, path=changed))
            )

    def test_native_cli_requires_actual_windows_host(self):
        for arguments in (
            ["build", "--host", "--jaic", __file__, "--out", "."],
            ["run", "--dir", "."],
        ):
            with (
                patch("sys.argv", ["windows_cross", *arguments]),
                patch.object(
                    cross.stdlib_runtime, "host_platform", return_value="linux-x64"
                ),
                patch.object(cross, "build") as build,
                patch.object(cross, "run") as run,
                contextlib.redirect_stderr(io.StringIO()),
                self.assertRaises(SystemExit) as stopped,
            ):
                cross.main()
            self.assertEqual(stopped.exception.code, 2)
            build.assert_not_called()
            run.assert_not_called()

    def test_sdk_control_uses_real_target_compiler_and_keeps_failures(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "probe.exe"
            for host, cpu, compiler in (
                (False, "x64", "x86_64-w64-mingw32-gcc"),
                (False, "arm64", "aarch64-w64-mingw32-gcc"),
                (True, "arm64", "clang"),
            ):
                args = SimpleNamespace(host=host, cpu=cpu)
                with patch.object(
                    cross.subprocess,
                    "run",
                    return_value=subprocess.CompletedProcess(
                        [], 1, "", "genuine SDK link failure"
                    ),
                ) as run:
                    self.assertIn(
                        "genuine SDK link failure",
                        cross.build_socket_sdk_probe(args, output),
                    )
                    self.assertEqual(run.call_args.args[0][0], compiler)
                    self.assertIn("-lws2_32", run.call_args.args[0])
                with patch.object(
                    cross.subprocess,
                    "run",
                    return_value=subprocess.CompletedProcess([], 0, "", ""),
                ):
                    self.assertIn(
                        "without producing", cross.build_socket_sdk_probe(args, output)
                    )

    def test_exact_stale_cpuid_oracle_is_separate_and_hash_bound(self):
        cases = cross.stdlib_runtime.validated_entries()
        invalid = {c["id"] for c in cases if cross.fixture_stale_contract_reason(c)}
        # The CPUID fixture has been corrected against real hardware, so no case is the pinned
        # legacy source any more and every one is required.
        self.assertEqual(invalid, set())
        with tempfile.TemporaryDirectory() as directory:
            corrected = Path(directory) / "corrected.jai"
            corrected.write_text("// independently corrected oracle\nmain :: () {}\n")
            self.assertIsNone(
                cross.fixture_stale_contract_reason(
                    {"id": "compiler/lang-asm", "path": corrected}
                )
            )

    def test_exact_platform_and_build_host_requirements(self):
        cases = {c["id"]: c for c in cross.stdlib_runtime.validated_entries()}
        for cpu in ("x64", "arm64"):
            target = "windows-" + cpu + "-mingw"
            for name in (
                "compiler/socket-select-canonical-timeval",
                "extensions/io-uring-dependency-repro",
            ):
                self.assertIsNotNone(
                    cross.fixture_unavailable_reason(cases[name], target, "linux-x64")
                )
            for name in (
                "contracts/program-print-plugin-file-output",
                "compiler/rules-compile-time-execution",
            ):
                self.assertIn(
                    "build host",
                    cross.fixture_unavailable_reason(cases[name], target, "linux-x64"),
                )
                self.assertIsNone(
                    cross.fixture_unavailable_reason(
                        cases[name], "windows-" + cpu, "windows-" + cpu
                    )
                )
            self.assertIn(
                "imgui",
                cross.fixture_unavailable_reason(
                    cases["native/imgui-headless-frames"], target, "linux-x64"
                ),
            )
            self.assertIsNone(
                cross.fixture_unavailable_reason(
                    cases["native/imgui-headless-frames"],
                    "windows-" + cpu,
                    "windows-" + cpu,
                )
            )

    def test_real_compiler_and_provider_failures_stay_required(self):
        cases = {c["id"]: c for c in cross.stdlib_runtime.validated_entries()}
        required = (
            "compiler/proc-type-defaults-ifx",
            "compiler/static-switch-add-context",
            "extensions/io-configured-production",
            "extensions/json-windows-host-provider-repro",
            "extensions/tls-windows-target",
        )
        for cpu in ("x64", "arm64"):
            for name in required:
                self.assertIsNone(
                    cross.fixture_unavailable_reason(
                        cases[name], "windows-" + cpu + "-mingw", "linux-x64"
                    ),
                    name,
                )
        self.assertIsNotNone(
            cross.fixture_unavailable_reason(
                cases["compiler/lang-asm"], "windows-arm64-mingw", "linux-x64"
            )
        )
        self.assertIsNone(
            cross.fixture_unavailable_reason(
                cases["compiler/lang-asm"], "windows-x64-mingw", "linux-x64"
            )
        )

    def test_missing_main_cannot_hide_genuine_compile_errors(self):
        args = SimpleNamespace(host=False, cpu="arm64", jaic=__file__)
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(
                cross.subprocess,
                "run",
                side_effect=[
                    subprocess.CompletedProcess(
                        [], 1, "", "has no `main` procedure; genuine_error"
                    ),
                    subprocess.CompletedProcess([], 1, "", "genuine_error"),
                ],
            ) as run:
                error = cross.build_one(args, Path(__file__), Path(directory) / "test")
            self.assertNotEqual(error, cross.COMPILE_TIME_ONLY)
            self.assertIn("genuine_error", error)
            self.assertIn("-check", run.call_args.args[0])

    def test_cross_providers_keep_host_and_target_separate(self):
        args = SimpleNamespace(host=False, cpu="arm64")
        with tempfile.TemporaryDirectory() as directory:
            with (
                patch.object(
                    cross.stdlib_runtime, "host_platform", return_value="linux-x64"
                ),
                patch.object(cross.stdlib_runtime, "use_native_libs") as host,
                patch.object(cross, "ensure_cross_host_bindings") as bindings,
                patch.object(cross.subprocess, "run") as build,
                patch("build_native_libs.output_dir", return_value=Path(directory)),
                patch.dict(os.environ, {}, clear=False),
            ):
                cross.use_cross_libs(args)
                host.assert_called_once_with("linux-x64")
                bindings.assert_called_once_with("linux-x64")
                self.assertEqual(
                    build.call_args_list[0].args[0][-1], "windows-arm64-mingw"
                )
                target_command = build.call_args_list[1].args[0]
                self.assertIn("build_windows_binding_providers.py", target_command[1])
                self.assertEqual(
                    target_command[-3:], ["windows-arm64-mingw", "--output", directory]
                )
                self.assertEqual(os.environ["JAIC_CROSS_LIBS"], directory)

    def test_build_reports_unavailable_and_retains_failure_even_keep_going(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            paths = [root / name for name in ("unix.jai", "required.jai")]
            catalogue = [
                {
                    "id": "unix",
                    "family": "modules",
                    "path": paths[0],
                    "required_os": "linux",
                },
                {"id": "required", "family": "modules", "path": paths[1]},
            ]
            selected = [
                ("stdlib-" + c["id"], c["path"], 0, None, True) for c in catalogue
            ]
            args = SimpleNamespace(
                out=directory,
                stdlib=True,
                host=False,
                cpu="arm64",
                jaic=__file__,
                keep_going=True,
            )
            with (
                patch.object(cross, "ROOT", root),
                patch.object(cross, "use_cross_libs"),
                patch.object(
                    cross.stdlib_runtime, "host_platform", return_value="linux-x64"
                ),
                patch.object(
                    cross.stdlib_runtime, "validated_entries", return_value=catalogue
                ),
                patch.object(cross, "cases", return_value=selected),
                patch.object(
                    cross, "build_one", return_value="genuine link failure"
                ) as build,
                patch.object(cross.shutil, "copytree"),
                patch.object(cross, "c_structs_library", return_value=None),
                patch.object(cross, "C_STRUCTS", {}),
                contextlib.redirect_stdout(io.StringIO()),
            ):
                self.assertEqual(cross.build(args), 1)
            self.assertEqual(build.call_count, 1)
            report = json.loads((root / "build-report.json").read_text())
            self.assertEqual(report["counts"]["platform_unavailable"], 1)
            self.assertEqual(report["counts"]["failed"], 1)
            self.assertEqual(len(report["results"]), 2)
            self.assertTrue(all(r["passed"] is None for r in report["results"]))

    def test_native_run_keeps_arguments_and_stderr_assertions(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "expected.json").write_text(
                json.dumps(
                    {
                        "required": {
                            "exit_code": 0,
                            "stdout": "ok\n",
                            "contract": {
                                "stdout": "ok\n",
                                "stderr": "owned diagnostic\n",
                            },
                            "fixture": {"arguments": ["enabled"]},
                        }
                    }
                )
            )
            args = SimpleNamespace(dir=directory)
            for stderr, expected in (
                (b"owned diagnostic\n", 0),
                (b"owned diagnostic\r\n", 0),
                (b"wrong\n", 1),
            ):
                with (
                    patch.object(
                        cross.subprocess,
                        "run",
                        return_value=subprocess.CompletedProcess(
                            [], 0, b"ok\n", stderr
                        ),
                    ) as run,
                    contextlib.redirect_stdout(io.StringIO()),
                ):
                    self.assertEqual(cross.run(args), expected)
                self.assertEqual(run.call_args.args[0][-1], "enabled")


if __name__ == "__main__":
    unittest.main()
