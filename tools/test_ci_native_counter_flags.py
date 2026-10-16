"""Compile-command contracts only; real Linux SDK build stays required in CI."""

from pathlib import Path
import unittest
from unittest.mock import patch

import build_ci_native_bindings as tool


class CounterFeatures(unittest.TestCase):
    def commands(self, system):
        _, profiles, _ = tool.selected_libraries()
        with (
            patch.object(tool.platform, "system", return_value=system),
            patch.object(tool.subprocess, "run") as run,
        ):
            tool.build_counter_probes(Path("/caller-output"), profiles)
        return [call.args[0] for call in run.call_args_list if "-c" in call.args[0]]

    def test_linux_features_precede_all_three_counter_translation_units(self):
        commands = self.commands("Linux")
        self.assertEqual(len(commands), 3)
        for command in commands:
            for flag in ("-D_DEFAULT_SOURCE=1", "-D_POSIX_C_SOURCE=200809L"):
                self.assertLess(command.index(flag), command.index("-std=c11"))

    def test_macos_does_not_receive_linux_feature_policy(self):
        commands = self.commands("Darwin")
        self.assertEqual(len(commands), 3)
        self.assertTrue(
            all(
                not any(
                    arg.startswith("-D_DEFAULT_SOURCE")
                    or arg.startswith("-D_POSIX_C_SOURCE")
                    for arg in command
                )
                for command in commands
            )
        )


if __name__ == "__main__":
    unittest.main()
