"""Build-plan/layout tests only; no native harness or fault execution."""

import importlib.util
import hashlib
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch


class NativePreparation(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        spec = importlib.util.spec_from_file_location(
            "upstream_native_preparation", os.environ["UPSTREAM_NATIVE_TEST_SOURCE"]
        )
        cls.tool = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.tool)

    def test_clipboard_source_and_both_artifacts_must_match_receipt(self):
        # Metadata fixtures only, not source/build/runtime evidence.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            native = root / "native"
            native.mkdir()
            inputs = {}
            for name in ["data.c", "data.h", "x11.c"]:
                relative = "stdlib/internal/clipboard/native/" + name
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("/* hash policy fixture */\n")
                inputs[relative] = hashlib.sha256(path.read_bytes()).hexdigest()
            receipt = {"inputs": inputs}
            for filename, key in [
                ("libportable_clipboard.a", "archive_sha256"),
                ("libportable_clipboard.so", "shared_sha256"),
            ]:
                path = native / filename
                path.write_bytes(b"non-executable hash fixture")
                receipt[key] = hashlib.sha256(path.read_bytes()).hexdigest()
            record = native / "portable-clipboard-build.json"
            record.write_text(json.dumps(receipt))
            self.tool.verify_clipboard_build(native, root)
            original = receipt["shared_sha256"]
            receipt["shared_sha256"] = "0" * 64
            record.write_text(json.dumps(receipt))
            with self.assertRaisesRegex(ValueError, "clipboard.*artifact"):
                self.tool.verify_clipboard_build(native, root)
            receipt["shared_sha256"] = original
            receipt["inputs"]["stdlib/internal/clipboard/native/x11.c"] = "0" * 64
            record.write_text(json.dumps(receipt))
            with self.assertRaisesRegex(ValueError, "clipboard.*source"):
                self.tool.verify_clipboard_build(native, root)

    def test_all_six_plans_prepare_matching_helpers_in_normal_consumer_locations(self):
        for system in ["macos", "linux", "windows"]:
            for cpu in ["x64", "arm64"]:
                with self.subTest(system=system, cpu=cpu):
                    platform = system + "-" + cpu
                    root = Path("/owned-checkout")
                    native = root / "artifacts/native-libs" / platform
                    plan = self.tool.native_plan(
                        platform, native, root / "stdlib", root
                    )
                    self.assertEqual(
                        len(plan["commands"]), 3 if system == "linux" else 2
                    )
                    crash, audio = plan["commands"][:2]
                    if system == "linux":
                        clipboard = plan["commands"][2]
                        self.assertIn(
                            str(root / "tools/build_portable_clipboard.py"), clipboard
                        )
                        self.assertEqual(
                            Path(clipboard[clipboard.index("--output") + 1]), native
                        )
                        self.assertIn(
                            native / "libportable_clipboard.a", plan["required"]
                        )
                        self.assertIn(
                            native / "libportable_clipboard.so", plan["required"]
                        )
                    self.assertIn(str(root / "tools/audio_build_native.py"), audio)
                    self.assertEqual(audio[audio.index("--platform") + 1], platform)
                    self.assertEqual(Path(audio[audio.index("--out") + 1]), native)
                    self.assertNotIn("--test-runtime", audio)
                    self.assertNotIn("--require-device", audio)
                    if system == "windows":
                        self.assertIn(
                            str(root / "tools/build_platform_io_runtime.py"), crash
                        )
                        self.assertEqual(crash[crash.index("--cpu") + 1], cpu)
                        triple = "aarch64" if cpu == "arm64" else "x86_64"
                        self.assertIn(f"clang --target={triple}-pc-windows-msvc", audio)
                        self.assertIn(native / "jai_audio_output.lib", plan["required"])
                        self.assertIn(
                            root
                            / "stdlib/internal/runtime/crash-native/platform_io_windows_crash.lib",
                            plan["required"],
                        )
                    else:
                        self.assertIn(
                            str(root / "tools/build_portable_runtime.py"), crash
                        )
                        self.assertEqual(crash[crash.index("--platform") + 1], platform)
                        self.assertIn(
                            root
                            / "stdlib/internal/runtime/crash-native/libportable_crash.a",
                            plan["required"],
                        )
                        self.assertIn(
                            native / "libjai_audio_output.a", plan["required"]
                        )

    def test_target_mismatch_rejected_before_any_build(self):
        with (
            patch.object(self.tool, "host_platform", return_value="macos-arm64"),
            patch.object(self.tool.subprocess, "run") as run,
        ):
            with self.assertRaisesRegex(ValueError, "actual host"):
                self.tool.prepare_native_dependencies(
                    "windows-x64", Path("/native"), Path("/stdlib")
                )
            run.assert_not_called()

    def test_native_build_failure_is_not_waived(self):
        with (
            patch.object(self.tool, "host_platform", return_value="linux-x64"),
            patch.object(
                self.tool.subprocess,
                "run",
                side_effect=RuntimeError("real build failure"),
            ),
        ):
            with self.assertRaisesRegex(RuntimeError, "real build failure"):
                self.tool.prepare_native_dependencies(
                    "linux-x64", Path("/native"), Path("/stdlib")
                )

    def test_exact_failed_build_status_and_streams_survive(self):
        failure = self.tool.subprocess.CalledProcessError(
            19, ["cc"], output="build output", stderr="SDK error"
        )
        with (
            patch.object(self.tool, "host_platform", return_value="linux-x64"),
            patch.object(self.tool.subprocess, "run", side_effect=failure),
        ):
            with self.assertRaises(self.tool.subprocess.CalledProcessError) as caught:
                self.tool.prepare_native_dependencies(
                    "linux-x64", Path("/native"), Path("/stdlib")
                )
            self.assertIs(caught.exception, failure)
            self.assertEqual(caught.exception.returncode, 19)
            self.assertEqual(caught.exception.stderr, "SDK error")

    def test_zero_build_exit_without_required_artifacts_is_still_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with (
                patch.object(self.tool, "host_platform", return_value="linux-x64"),
                patch.object(self.tool.subprocess, "run") as run,
            ):
                with self.assertRaisesRegex(
                    FileNotFoundError, "required source-built helper"
                ):
                    self.tool.prepare_native_dependencies(
                        "linux-x64", root / "native", root / "stdlib", root
                    )
                self.assertEqual(run.call_count, 3)
                self.assertTrue(
                    all(call.kwargs["check"] for call in run.call_args_list)
                )

    def test_wrong_platform_or_corrupt_audio_receipt_is_fatal(self):
        # Receipt/hash fixtures only, not binary/native build evidence.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            native, stdlib = root / "native", root / "stdlib"
            plan = self.tool.native_plan("linux-x64", native, stdlib, root)
            for path in plan["required"]:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b"non-executable hash-contract fixture")
            artifacts = {
                p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                for p in plan["required"]
                if p.parent == native
                and p.name.startswith(("libjai_audio_output.", "libstb_vorbis."))
            }
            for target, changed in [
                ("linux-arm64", artifacts),
                ("linux-x64", {**artifacts, "libjai_audio_output.a": "0" * 64}),
            ]:
                (native / "audio-build.json").write_text(
                    json.dumps({"platform": target, "artifacts": changed})
                )
                with (
                    patch.object(self.tool, "host_platform", return_value="linux-x64"),
                    patch.object(self.tool.subprocess, "run"),
                    self.assertRaises(ValueError),
                ):
                    self.tool.prepare_native_dependencies(
                        "linux-x64", native, stdlib, root
                    )

    def test_stale_crash_source_target_or_output_receipt_is_fatal(self):
        # Non-executable source/receipt fixtures, not native build evidence.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            native, stdlib = root / "native", root / "stdlib"
            plan = self.tool.native_plan("linux-x64", native, stdlib, root)
            for path in plan["required"]:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b"non-executable hash-contract fixture")
            audio = {
                p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                for p in plan["required"]
                if p.parent == native
                and p.name.startswith(("libjai_audio_output.", "libstb_vorbis."))
            }
            (native / "audio-build.json").write_text(
                json.dumps({"platform": "linux-x64", "artifacts": audio})
            )
            inputs = {}
            for name in ["data.c", "data.h", "x11.c"]:
                relative = "stdlib/internal/clipboard/native/" + name
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("/* source hash fixture */\n")
                inputs[relative] = hashlib.sha256(path.read_bytes()).hexdigest()
            (native / "portable-clipboard-build.json").write_text(
                json.dumps(
                    {
                        "inputs": inputs,
                        "archive_sha256": hashlib.sha256(
                            (native / "libportable_clipboard.a").read_bytes()
                        ).hexdigest(),
                        "shared_sha256": hashlib.sha256(
                            (native / "libportable_clipboard.so").read_bytes()
                        ).hexdigest(),
                    }
                )
            )
            source = plan["runtime"] / "runtime.c"
            source.write_text("/* original hash-contract fixture, never compiled */\n")
            artifacts = {
                p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                for p in plan["required"]
                if p.parent == plan["runtime"]
            }
            recipe = {
                "platform": "linux-x64",
                "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
                "artifacts": artifacts,
            }
            for changed in [
                dict(recipe, platform="linux-arm64"),
                dict(recipe, source_sha256="0" * 64),
                dict(recipe, artifacts={**artifacts, "libportable_crash.a": "0" * 64}),
            ]:
                (plan["runtime"] / "portable-crash-build.json").write_text(
                    json.dumps(changed)
                )
                with (
                    patch.object(self.tool, "host_platform", return_value="linux-x64"),
                    patch.object(self.tool.subprocess, "run"),
                    self.assertRaisesRegex(ValueError, "portable crash"),
                ):
                    self.tool.prepare_native_dependencies(
                        "linux-x64", native, stdlib, root
                    )


if __name__ == "__main__":
    unittest.main()
