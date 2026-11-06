"""The Cargo test workflow supplies the owned formatter to LSP and documentation subprocesses."""
import types
import os
from pathlib import Path
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
             patch.object(dev, "ensure_jaifmt", side_effect=lambda jobs: events.append(("formatter", jobs)) or Path("owned/jaifmt")), \
             patch.object(dev, "run", side_effect=lambda cmd, **kw: events.append(("run", cmd)) or 0):
            self.assertEqual(dev.cmd_test(args), 0)
        nextest = next(i for i, (_, cmd) in enumerate(events) if cmd[:3] == ["cargo", "nextest", "run"])
        self.assertIn(("formatter", "4"), events[:nextest])


    def test_owned_formatter_is_available_to_test_children_without_changing_parent_path(self):
        calls = []
        formatter = Path('owned tools') / 'jaifmt'
        args = types.SimpleNamespace(suite='cargo', crate='jacket', filter=[], jobs='2',
                                     partition='slice:1/1', expr=None)
        with patch.dict(os.environ, {'PATH': '/installed/older-tools'}), \
             patch.object(dev.shutil, 'which', return_value='nextest'), \
             patch.object(dev, 'ensure_jaifmt', return_value=formatter), \
             patch.object(dev, 'run', side_effect=lambda command, **kw: calls.append((command, kw)) or 0):
            self.assertEqual(dev.cmd_test(args), 0)
            self.assertEqual(os.environ['PATH'], '/installed/older-tools')
        for prefix in [['cargo', 'nextest', 'run'], ['cargo', 'test', '--doc']]:
            call = next(kw for command, kw in calls if command[:3] == prefix)
            paths = call.get('env', {}).get('PATH', '').split(os.pathsep)
            self.assertEqual(paths, [str(formatter.parent.resolve()), '/installed/older-tools'])


if __name__ == "__main__":
    unittest.main()
