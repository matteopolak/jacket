"""Preparation/order checks; native functionality is a separate required gate."""

from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from types import SimpleNamespace

import build_generator_platform as provider
import check_developer_cpp_custom as workflow
from ci_test_scratch import assigned_scratch


class RuntimeSetup(unittest.TestCase):
    def test_linux_build_is_real_shared_provider_with_dl(self):
        with tempfile.TemporaryDirectory(dir=assigned_scratch()) as directory:
            with (
                patch.object(provider.platform, "system", return_value="Linux"),
                patch.object(
                    provider.build_developer_bindings,
                    "llvm_prefix",
                    return_value=Path("/official/llvm"),
                ),
                patch.object(provider.subprocess, "run") as command,
            ):
                output = provider.build(output_dir=directory)
            argv = command.call_args.args[0]
            self.assertEqual(output.name, "libgenerator_platform.so")
            self.assertIn("-shared", argv)
            self.assertIn("-fPIC", argv)
            self.assertIn("-ldl", argv)
            self.assertIn(
                str(
                    provider.ROOT
                    / "stdlib/internal/developer-tools/generator-platform-native.c"
                ),
                argv,
            )
            self.assertEqual(
                (Path(directory) / "generator-platform-nm.txt").read_text(),
                "/official/llvm/bin/llvm-nm",
            )

    def test_original_inputs_precede_first_real_emitter(self):
        commands = []

        def record(command, expected=""):
            commands.append([str(item) for item in command])
            if command[0] == workflow.JAIC:
                raise StopIteration

        with (
            patch.object(
                workflow.build_developer_bindings,
                "llvm_prefix",
                return_value=Path("/official/llvm"),
            ),
            patch.object(workflow.build_generator_platform, "build") as helper,
            patch.object(workflow, "prepare_callback_input") as callback,
            patch.object(workflow, "run", side_effect=record),
        ):
            with self.assertRaises(StopIteration):
                workflow.main()
        helper.assert_called_once_with(Path("/official/llvm"))
        callback.assert_called_once()
        self.assertTrue(commands[-1][1].endswith("bindings-cpp-custom.jai"))
        self.assertEqual(len(commands[:-1]), 5)
        self.assertTrue(
            any(item.endswith("bindings-cpp-custom-native.cpp") for item in commands[0])
        )
        self.assertFalse(
            any(item.endswith("developer-tools-cpp-custom.cpp") for item in commands[0])
        )

    def test_existing_required_cpp_assertions_remain(self):
        root = Path(__file__).resolve().parents[1]
        emitter = (root / "stdlib/Bindings_Generator/tests/driven/bindings-cpp-custom.jai").read_text()
        consumer = (
            root / "stdlib/Bindings_Generator/tests/driven/bindings-cpp-custom-roundtrip.jai"
        ).read_text()
        self.assertIn("foreign_prints == ordinary_functions", emitter)
        self.assertIn("outer_prints == outer_before + 1", emitter)
        self.assertIn("inner_prints == inner_before + 1", emitter)
        self.assertIn("foreign_prints == foreign_before &&", emitter)
        self.assertIn('G :: #import "developer-tools-cpp-custom-bridge";', consumer)
        self.assertIn(
            "make_custom_record :: (result: *Custom_Record, value: s32)", emitter
        )
        self.assertIn("assert(C.original_callback_value(10) == 103);", consumer)
        self.assertIn("assert(G.custom_integer(a = 10, b = 3) == 103);", consumer)
        self.assertIn("assert(pointer == *record && record.number == 42);", consumer)

    def test_default_workflow_still_requires_c_model_regressions(self):
        with (
            patch.object(workflow, "check_cpp_custom") as cpp,
            patch.object(
                workflow,
                "check_c_model_regressions",
                side_effect=RuntimeError("required model regression failed"),
            ) as model,
        ):
            with self.assertRaisesRegex(RuntimeError, "required model regression"):
                workflow.main()
        cpp.assert_called_once_with()
        model.assert_called_once_with()

    def test_wrong_adapter_pair_must_be_rejected_for_its_actual_abi(self):
        with tempfile.TemporaryDirectory(dir=assigned_scratch()) as directory:
            root = Path(directory)
            (root / "target").mkdir()
            consumer = root / "consumer.jai"
            consumer.write_text('G :: #import "developer-tools-cpp-custom-bridge";\n')
            for status, diagnostic, accepted in [
                (1, "make_custom_record: parameter `value` given twice", True),
                (0, "make_custom_record: parameter `value` given twice", False),
                (1, "unrelated import error", False),
            ]:
                with (
                    self.subTest(status=status, diagnostic=diagnostic),
                    patch.object(workflow, "ROOT", root),
                    patch.object(
                        workflow.subprocess,
                        "run",
                        return_value=SimpleNamespace(
                            returncode=status, stderr=diagnostic
                        ),
                    ),
                ):
                    if accepted:
                        workflow.reject_wrong_adapter_schema(consumer)
                    else:
                        with self.assertRaises(RuntimeError):
                            workflow.reject_wrong_adapter_schema(consumer)
            self.assertIn(
                'G :: #import "developer-tools-cpp-custom-generated";',
                (root / "target/developer-tools-cpp-wrong-abi.jai").read_text(),
            )


if __name__ == "__main__":
    unittest.main()
