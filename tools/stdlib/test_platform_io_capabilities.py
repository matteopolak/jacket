"""Selection/accounting controls; no synthetic bytes count as OS execution."""

import unittest
import json
import subprocess
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import Mock, patch

from check_platform_io import CONTRACTS, main, run_wasm_contract, wasm_contract_reason
from stdlib_test_catalog import unavailable_reason, validated_entries

ROOT = Path(__file__).resolve().parents[1]
CASES = {c["id"]: c for c in validated_entries()}
SOURCES = {name: CASES[name]["path"] for name in CONTRACTS}
NATIVE_ONLY = [n for n in CONTRACTS if wasm_contract_reason(SOURCES[n])]
SUPPORTED = [n for n in CONTRACTS if n not in NATIVE_ONLY]


class PlatformCapabilityTests(unittest.TestCase):
    def test_full_runner_preserves_native_and_cross_checks(self):
        with TemporaryDirectory() as temporary:
            arguments = [
                "check_platform_io.py",
                "--jaic",
                str(Path(__file__).resolve()),
                "--compiler-build-root",
                str(ROOT / "tools"),
                "--stdlib",
                str(ROOT / "stdlib"),
                "--output-dir",
                temporary,
                "--native",
                "--cross-check",
                "--wasm",
                "--case",
                "Process/tests/lifecycle",
                "--case",
                "Thread/tests/synchronization",
            ]
            rejection = subprocess.CompletedProcess(
                [], 1, b"", b"conflicts with the configured runtime instance"
            )
            with (
                patch("sys.argv", arguments),
                patch("check_platform_io.os.name", "posix"),
                patch("check_platform_io.run") as run,
                patch("check_platform_io.subprocess.run", return_value=rejection),
            ):
                main()
            commands = [list(map(str, call.args[0])) for call in run.call_args_list]
            for stem in ("lifecycle", "synchronization"):
                selected = [
                    command
                    for command in commands
                    if len(command) > 1 and command[1].endswith(stem + ".jai")
                ]
                self.assertEqual(
                    len(selected), 8
                )  # four desktop checks, two interpreter, two native builds
                self.assertEqual(sum("-interp" in command for command in selected), 2)
                self.assertEqual(sum("-exe" in command for command in selected), 2)
                self.assertFalse(any("wasm" in command for command in selected))
            rows = json.loads((Path(temporary) / "wasm-results.json").read_text())[
                "results"
            ]
            self.assertEqual(len(rows), 8)
            self.assertTrue(all(row["passed"] is None for row in rows))

    def test_every_contract_is_a_catalogue_case(self):
        for name, path in SOURCES.items():
            self.assertTrue(path.is_file(), name)
        self.assertTrue(NATIVE_ONLY)
        self.assertTrue(SUPPORTED)

    def test_native_only_contracts_do_not_invoke_wasm_or_count_passed(self):
        for name in NATIVE_ONLY:
            with self.subTest(name=name):
                attempt = Mock()
                rows = run_wasm_contract(
                    "jaic", SOURCES[name], Path("out"), {}, attempt
                )
                attempt.assert_not_called()
                self.assertEqual(
                    [r["mode"] for r in rows],
                    ["wasm-check", "wasm-interp", "wasm-native-O0", "wasm-native-O2"],
                )
                self.assertTrue(
                    all(
                        r["status"] == "platform_unavailable"
                        and r["passed"] is None
                        and r["reason"]
                        for r in rows
                    )
                )

    def test_supported_wasm_fixtures_retain_all_modes(self):
        for name in SUPPORTED:
            attempt = Mock(return_value=True)
            rows = run_wasm_contract(
                "jaic", SOURCES[name], Path("out"), {}, attempt
            )
            self.assertEqual(attempt.call_count, 6)
            self.assertTrue(
                all(r["status"] == "passed" and r["passed"] is True for r in rows)
            )
            self.assertIn("-no_dce", attempt.call_args_list[0].args[0])
            self.assertIn("-interp", attempt.call_args_list[1].args[0])
            self.assertEqual(
                [attempt.call_args_list[i].args[0][0] for i in (3, 5)], ["node", "node"]
            )

    def test_real_build_or_execution_failure_remains_failure(self):
        attempt = Mock(side_effect=[True, False, False, True, False])
        rows = run_wasm_contract(
            "jaic", SOURCES[SUPPORTED[0]], Path("out"), {}, attempt
        )
        self.assertEqual([r["passed"] for r in rows], [True, False, False, False])
        self.assertEqual(attempt.call_count, 5)
        self.assertTrue(all(r["status"] == "failed" for r in rows[1:]))

    def test_exact_catalogue_cases_remain_required_on_native_hosts(self):
        for name in NATIVE_ONLY:
            case = CASES[name]
            for mode in ("wasm-interp", "wasm-native"):
                self.assertIsNotNone(unavailable_reason(case, mode, "linux-arm64"))
            for host in (
                "linux-arm64",
                "linux-x64",
                "windows-x64",
                "windows-arm64",
                "macos-arm64",
            ):
                for mode in ("interp", "native"):
                    self.assertIsNone(unavailable_reason(case, mode, host))
        # The metadata must not become a blanket native-family/module skip.
        self.assertIsNone(
            unavailable_reason(CASES[SUPPORTED[0]], "wasm-native", "linux-arm64")
        )


if __name__ == "__main__":
    unittest.main()
