"""HTTP discovery, per-case capabilities and unavailable accounting policy tests.

Mocked dispatch verifies harness accounting, not native functionality.
"""

import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import http_conformance
import stdlib_runtime
import stdlib_test_catalog as catalog


class HttpCoverageTests(unittest.TestCase):
    def http_cases(self):
        entries = catalog.validated_entries()
        return {
            case["id"]: case
            for case in entries
            if case["id"] in http_conformance.HTTP_CONFORMANCE_CASE_IDS
        }

    def test_exact_discovery_and_single_roundtrip(self):
        cases = self.http_cases()
        self.assertEqual(set(cases), set(http_conformance.HTTP_CONFORMANCE_CASE_IDS))
        self.assertEqual(len(cases), 12)
        self.assertEqual(len(http_conformance.HTTP_CONFORMANCE_CASE_IDS), 12)
        entries = catalog.validated_entries()
        roundtrip = [
            case for case in entries if case["id"] == "Extensions/Http/tests/codec-roundtrip"
        ]
        self.assertEqual(len(roundtrip), 1)
        self.assertEqual(
            roundtrip[0]["path"],
            catalog.ROOT / "stdlib/Extensions/Http/tests/codec-roundtrip.jai",
        )

    def test_only_reviewed_io_cases_are_unavailable_on_wasm(self):
        cases = self.http_cases()
        native = {
            "client",
            "https",
            "server-connections",
            "server-extractors",
            "server-responses",
            "server-routing",
            "testing",
        }
        for name, case in cases.items():
            io = name.rsplit("/", 1)[-1] in native
            for mode in ["interp", "native", "wasm-interp", "wasm-native"]:
                with self.subTest(name=name, mode=mode):
                    reason = catalog.unavailable_reason(case, mode, "linux-x64")
                    if io and mode.startswith("wasm"):
                        self.assertIn("fiber", reason)
                        self.assertIn("native", reason)
                        self.assertEqual(
                            case["unsupported_modes"], ["wasm-interp", "wasm-native"]
                        )
                    else:
                        self.assertIsNone(reason)

    def test_http_invokes_each_exact_case_in_all_requested_modes(self):
        with patch.object(http_conformance.subprocess, "run") as run:
            run.return_value.returncode = 0
            self.assertTrue(
                http_conformance.run_harness(
                    Path("own-jaic"),
                    "interp,native,wasm-interp,wasm-native",
                    http_conformance.HTTP_CONFORMANCE_CASE_IDS,
                    1,
                )
            )
        self.assertEqual(run.call_count, 1)
        command = run.call_args.args[0]
        selected = [command[i + 1] for i, arg in enumerate(command) if arg == "--case"]
        self.assertEqual(selected, list(http_conformance.HTTP_CONFORMANCE_CASE_IDS))
        self.assertNotIn("--filter", command)
        self.assertEqual(
            command[command.index("--modes") + 1],
            "interp,native,wasm-interp,wasm-native",
        )

    def test_missing_http_case_fails_before_any_child_runs(self):
        entries = list(self.http_cases().values())
        entries = [
            case for case in entries if case["id"] != "Extensions/Http/tests/codec-roundtrip"
        ]
        with (
            patch.object(http_conformance, "validated_entries", return_value=entries),
            patch.object(http_conformance.subprocess, "run") as run,
        ):
            with self.assertRaisesRegex(
                RuntimeError, "Extensions/Http/tests/codec-roundtrip"
            ):
                http_conformance.run_harness(
                    Path("own-jaic"),
                    "wasm-native",
                    http_conformance.HTTP_CONFORMANCE_CASE_IDS,
                    1,
                )
        run.assert_not_called()

    def test_unavailable_is_not_pass_and_native_dispatch_is_preserved(self):
        cases = list(self.http_cases().values())
        with tempfile.TemporaryDirectory() as scratch:
            report = Path(scratch) / "report.json"
            with (
                patch.object(stdlib_runtime, "validated_entries", return_value=cases),
                patch.object(stdlib_runtime, "use_native_libs"),
                patch.object(
                    stdlib_runtime, "run_mode", return_value=(True, "policy only")
                ) as dispatch,
                patch(
                    "sys.argv",
                    [
                        "stdlib_runtime.py",
                        "--jaic",
                        __file__,
                        "--modes",
                        "interp,native,wasm-interp,wasm-native",
                        "--jobs",
                        "1",
                        "--report",
                        str(report),
                    ],
                ),
                patch("builtins.print"),
            ):
                self.assertEqual(stdlib_runtime.main(), 0)
            result = json.loads(report.read_text())
        self.assertEqual(dispatch.call_count, 34)
        for mode in ["interp", "native"]:
            self.assertEqual(result["counts"][mode]["passed"], 12)
            self.assertEqual(result["counts"][mode]["platform_unavailable"], 0)
        for mode in ["wasm-interp", "wasm-native"]:
            self.assertEqual(result["counts"][mode]["passed"], 5)
            self.assertEqual(result["counts"][mode]["platform_unavailable"], 7)
        unavailable = [row for row in result["results"] if row["platform_unavailable"]]
        self.assertEqual(len(unavailable), 14)
        self.assertTrue(all(row["passed"] is None for row in unavailable))

    def test_supported_failure_remains_fatal(self):
        cases = self.http_cases()
        selected = [
            cases["Extensions/Http/tests/codec"],
            cases["Extensions/Http/tests/client"],
        ]
        with tempfile.TemporaryDirectory() as scratch:
            report = Path(scratch) / "report.json"
            with (
                patch.object(
                    stdlib_runtime, "validated_entries", return_value=selected
                ),
                patch.object(
                    stdlib_runtime,
                    "run_mode",
                    return_value=(False, "supported failure"),
                ) as dispatch,
                patch(
                    "sys.argv",
                    [
                        "stdlib_runtime.py",
                        "--jaic",
                        __file__,
                        "--modes",
                        "wasm-interp",
                        "--jobs",
                        "1",
                        "--report",
                        str(report),
                    ],
                ),
                patch("builtins.print"),
            ):
                self.assertEqual(stdlib_runtime.main(), 1)
            result = json.loads(report.read_text())
        dispatch.assert_called_once()
        self.assertEqual(result["counts"]["wasm-interp"]["failed"], 1)
        self.assertEqual(result["counts"]["wasm-interp"]["platform_unavailable"], 1)


if __name__ == "__main__":
    unittest.main()
