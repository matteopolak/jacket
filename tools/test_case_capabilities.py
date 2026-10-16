"""Accounting tests only: mocked execution is never native platform proof."""

import contextlib
import io
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import stdlib_runtime
import stdlib_test_catalog as catalog
import upstream_smoke as smoke

NATIVE_COLOR = "native-windows-console-colors"


class DeferredColors(unittest.TestCase):
    def test_catalogue_has_no_new_deferred_omissions(self):
        cases = catalog.validated_entries()
        self.assertFalse(
            any(NATIVE_COLOR in c.get("required_capabilities", []) for c in cases)
        )
        required = {
            "extensions/json-windows-host-provider-repro",
            "extensions/tls-windows-target",
            "Input/tests/windows-device-decoding",
            "Runtime_Support/tests/windows-crash-bootstrap",
            "tests/portable-data-resource-windows",
        }
        found = {c["id"]: c for c in cases if c["id"] in required}
        self.assertEqual(set(found), required)
        for cpu in ("x64", "arm64"):
            for mode in ("interp", "native"):
                for case in found.values():
                    self.assertIsNone(
                        catalog.unavailable_reason(case, mode, "windows-" + cpu),
                        case["id"],
                    )

    def test_exact_smoke_consumers_and_ansi_release_eligibility(self):
        checks = json.loads(smoke.CHECKS.read_text())["checks"]
        deferred = {
            c["id"]
            for c in checks
            if NATIVE_COLOR in c.get("required_capabilities", [])
        }
        self.assertEqual(deferred, {"unotest"})
        deferred_steps = {
            (c["id"], index)
            for c in checks
            for index, step in enumerate(c["steps"])
            if NATIVE_COLOR in step.get("required_capabilities", [])
        }
        self.assertEqual(deferred_steps, {("jai-format", 0)})
        for platform in ("windows-x64", "windows-arm64"):
            for case in checks:
                decision = smoke.unavailable_decision(case, platform)
                if case["id"] == "unotest":
                    self.assertEqual(decision[0], "platform_unavailable")
                elif decision:
                    self.assertEqual(decision[0], "skip", case["id"])
            formatter = next(c for c in checks if c["id"] == "jai-format")
            self.assertIsNone(smoke.unavailable_decision(formatter, platform))
            self.assertEqual(formatter["steps"][0]["name"], "first.jai - test")
            self.assertEqual(
                formatter["steps"][0]["required_capabilities"], [NATIVE_COLOR]
            )
            for step in formatter["steps"][1:]:
                self.assertNotIn(NATIVE_COLOR, step.get("required_capabilities", []))

    def test_step_deferral_keeps_supported_release_and_format_steps(self):
        formatter = next(
            c
            for c in json.loads(smoke.CHECKS.read_text())["checks"]
            if c["id"] == "jai-format"
        )
        # Execute the real manifest dispatch against an empty independent fixture.
        # No upstream implementation is read and no compiler/application runs.
        check = {
            "id": formatter["id"],
            "project": "own-fixture",
            "steps": formatter["steps"],
        }
        for platform in ("windows-x64", "windows-arm64", "macos-arm64", "linux-x64"):
            executed = []

            def record_dispatch(step, ctx):
                executed.append(step["name"])
                return "ok"

            with tempfile.TemporaryDirectory() as temporary:
                upstream = Path(temporary) / "upstream"
                (upstream / "own-fixture").mkdir(parents=True)
                with (
                    patch.object(smoke, "UPSTREAM", upstream),
                    patch.dict(smoke.STEPS, {"run": record_dispatch}),
                    contextlib.redirect_stdout(io.StringIO()),
                ):
                    rows = smoke.run_check(
                        check,
                        SimpleNamespace(jaic=__file__, dlls=None, verbose=False),
                        platform,
                        Path(temporary),
                    )
            self.assertEqual(len(rows), 3)
            expected_names = [step["name"] for step in formatter["steps"]]
            self.assertEqual(
                executed,
                expected_names[1:]
                if platform.startswith("windows")
                else expected_names,
            )
            self.assertEqual(
                [r["status"] for r in rows],
                [
                    "platform_unavailable"
                    if platform.startswith("windows")
                    else "pass",
                    "pass",
                    "pass",
                ],
            )
            if platform.startswith("windows"):
                self.assertIsNone(rows[0]["passed"])
                self.assertIn("USER_DEFERRED", rows[0]["reason"])

    def test_exact_target_and_capability(self):
        default = {"family": "modules", "required_capabilities": [NATIVE_COLOR]}
        for cpu in ("x64", "arm64"):
            for mode in ("interp", "native"):
                reason = catalog.unavailable_reason(default, mode, "windows-" + cpu)
                self.assertIn("USER_DEFERRED", reason)
                self.assertIn("2026-10-16", reason)
        for platform in ("macos-arm64", "linux-x64", "windows-x64"):
            for mode in ("wasm-interp", "wasm-native"):
                self.assertIsNone(catalog.unavailable_reason(default, mode, platform))
        for platform in ("macos-x64", "macos-arm64", "linux-x64", "linux-arm64"):
            self.assertIsNone(catalog.unavailable_reason(default, "native", platform))

    def test_ansi_and_other_windows_cases_stay_required(self):
        for name in ("print-color-ansi", "windows-abi", "dll", "wgl", "source"):
            case = {"id": name, "family": "modules", "required_os": "windows"}
            for platform in ("windows-x64", "windows-arm64"):
                for mode in ("interp", "native"):
                    self.assertIsNone(catalog.unavailable_reason(case, mode, platform))
                self.assertIsNone(smoke.unavailable_decision(case, platform))

    def test_no_error_string_or_name_waiver(self):
        for case in (
            {"id": "Print_Color"},
            {"diagnostic": "native Windows console color handling is not implemented"},
        ):
            self.assertIsNone(smoke.unavailable_decision(case, "windows-x64"))

    def test_runtime_null_result_separate_counts_and_real_failure(self):
        cases = [
            {
                "id": "default-color",
                "family": "modules",
                "path": Path(__file__),
                "required_capabilities": [NATIVE_COLOR],
            },
            {"id": "ansi-color", "family": "modules", "path": Path(__file__)},
            {"id": "required-abi", "family": "modules", "path": Path(__file__)},
        ]
        with tempfile.TemporaryDirectory() as temporary:
            report = Path(temporary) / "runtime.json"
            with (
                patch.object(stdlib_runtime, "validated_entries", return_value=cases),
                patch.object(
                    stdlib_runtime, "host_platform", return_value="windows-arm64"
                ),
                patch.object(
                    stdlib_runtime,
                    "run_mode",
                    side_effect=[(True, "ok"), (False, "genuine ABI failure")],
                ) as run,
                patch(
                    "sys.argv",
                    [
                        "stdlib_runtime",
                        "--jaic",
                        __file__,
                        "--platform",
                        "windows-arm64",
                        "--modes",
                        "interp",
                        "--jobs",
                        "1",
                        "--report",
                        str(report),
                    ],
                ),
                contextlib.redirect_stdout(io.StringIO()),
            ):
                self.assertEqual(stdlib_runtime.main(), 1)
            data = json.loads(report.read_text())
        self.assertEqual(run.call_count, 2)
        self.assertEqual(data["selected_cases"], 3)
        self.assertEqual(data["counts"]["interp"]["passed"], 1)
        self.assertEqual(data["counts"]["interp"]["failed"], 1)
        self.assertEqual(data["counts"]["interp"]["platform_unavailable"], 1)
        deferred = next(r for r in data["results"] if r["name"] == "default-color")
        self.assertIsNone(deferred["passed"])
        self.assertIn("USER_DEFERRED", deferred["platform_unavailable"])
        self.assertEqual(len(data["results"]), 3)

    def test_smoke_reports_every_selected_case_and_retains_failures(self):
        checks = [
            {"id": "default-color", "required_capabilities": [NATIVE_COLOR]},
            {"id": "ansi-color"},
            {"id": "required-wgl"},
            {"id": "legacy-skip", "skip": {"*": "existing reason"}},
            {"id": "partial-ansi"},
        ]
        partial_steps = [
            {
                "name": "default-test",
                "status": "platform_unavailable",
                "passed": None,
                "reason": "USER_DEFERRED 2026-10-16",
            },
            {"name": "ansi-release", "status": "pass", "passed": True, "reason": None},
            {"name": "ansi-format", "status": "pass", "passed": True, "reason": None},
        ]
        with tempfile.TemporaryDirectory() as temporary:
            manifest = Path(temporary) / "checks.json"
            manifest.write_text(json.dumps({"checks": checks}))
            report = Path(temporary) / "smoke.json"
            output = io.StringIO()
            with (
                patch.dict("os.environ", {"JAIC_NATIVE_LIBS": temporary}),
                patch.object(
                    smoke,
                    "run_check",
                    side_effect=[
                        None,
                        smoke.Failure("genuine WGL failure"),
                        partial_steps,
                    ],
                ) as run,
                patch(
                    "sys.argv",
                    [
                        "upstream_smoke",
                        "--jaic",
                        __file__,
                        "--platform",
                        "windows-x64",
                        "--checks",
                        str(manifest),
                        "--screenshots",
                        str(Path(temporary) / "shots"),
                        "--report",
                        str(report),
                    ],
                ),
                contextlib.redirect_stdout(output),
            ):
                self.assertEqual(smoke.main(), 1)
            data = json.loads(report.read_text())
        self.assertEqual(
            [c.args[0]["id"] for c in run.call_args_list],
            ["ansi-color", "required-wgl", "partial-ansi"],
        )
        self.assertEqual(data["selected_cases"], 5)
        self.assertEqual(
            data["counts"], {"pass": 1, "FAIL": 1, "skip": 1, "platform_unavailable": 2}
        )
        self.assertEqual(len(data["results"]), 5)
        partial = data["results"][-1]
        self.assertIsNone(partial["passed"])
        self.assertEqual(partial["status"], "platform_unavailable")
        self.assertEqual(partial["steps"], partial_steps)
        deferred = data["results"][0]
        self.assertEqual(deferred["status"], "platform_unavailable")
        self.assertIsNone(deferred["passed"])
        self.assertIn("USER_DEFERRED", deferred["reason"])
        self.assertIn("platform_unavailable", output.getvalue())


if __name__ == "__main__":
    unittest.main()
