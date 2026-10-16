"""Selection/accounting controls; no synthetic bytes count as OS execution."""

import unittest
import json
import subprocess
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import Mock, patch

from check_platform_io import main, run_wasm_contract
from platform_io_capabilities import (
    PLATFORM_CONTRACT_CAPABILITIES,
    RUNTIME_CONTRACT_CAPABILITIES,
    wasm_contract_reason,
)
from stdlib_test_catalog import unavailable_reason, validated_entries

ROOT = Path(__file__).resolve().parents[1]


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
                "process-contract",
                "--case",
                "thread-contract",
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
            for stem in ("process-contract", "thread-contract"):
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

    def test_every_dedicated_fixture_has_an_explicit_contract(self):
        self.assertEqual(
            {p.stem for p in (ROOT / "tests/platform-io").glob("*-contract.jai")},
            set(PLATFORM_CONTRACT_CAPABILITIES),
        )
        with self.assertRaises(KeyError):
            wasm_contract_reason("unreviewed-contract")

    def test_native_only_contracts_do_not_invoke_wasm_or_count_passed(self):
        for stem in (
            "process-contract",
            "system-contract",
            "thread-contract",
            "thread-group-contract",
        ):
            with self.subTest(stem=stem):
                attempt = Mock()
                rows = run_wasm_contract(
                    "jaic", Path(stem + ".jai"), Path("out"), {}, attempt
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
        for stem, capabilities in PLATFORM_CONTRACT_CAPABILITIES.items():
            if capabilities:
                continue
            attempt = Mock(return_value=True)
            rows = run_wasm_contract(
                "jaic", Path(stem + ".jai"), Path("out"), {}, attempt
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
            "jaic", Path("file-contract.jai"), Path("out"), {}, attempt
        )
        self.assertEqual([r["passed"] for r in rows], [True, False, False, False])
        self.assertEqual(attempt.call_count, 5)
        self.assertTrue(all(r["status"] == "failed" for r in rows[1:]))

    def test_exact_catalogue_cases_remain_required_on_native_hosts(self):
        cases = {c["id"]: c for c in validated_entries()}
        for name in RUNTIME_CONTRACT_CAPABILITIES:
            case = cases[name]
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
            unavailable_reason(
                cases["contracts/base64-and-crc-contracts"],
                "wasm-native",
                "linux-arm64",
            )
        )


if __name__ == "__main__":
    unittest.main()
