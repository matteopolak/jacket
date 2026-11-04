"""The cargo test workflow supplies the real formatter that LSP integration tests drive."""
import types
import unittest
from unittest.mock import patch

import dev


class TestToolsTests(unittest.TestCase):
    def test_formatter_is_ready_before_cargo_tests_start(self):
        events = []
        args = types.SimpleNamespace(
            suite="cargo", crate="jai-language-server", filter=[], jobs="4",
            partition="slice:3/6", expr=None,
        )
        with patch.object(dev.shutil, "which", return_value="nextest"), \
             patch.object(dev, "ensure_jaifmt", side_effect=lambda jobs: events.append(("formatter", jobs))), \
             patch.object(dev, "run", side_effect=lambda cmd: events.append(("run", cmd)) or 0):
            self.assertEqual(dev.cmd_test(args), 0)
        nextest = next(i for i, (_, cmd) in enumerate(events) if cmd[:3] == ["cargo", "nextest", "run"])
        self.assertIn(("formatter", "4"), events[:nextest])


if __name__ == "__main__":
    unittest.main()
