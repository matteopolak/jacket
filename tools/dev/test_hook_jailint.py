"""The pre-commit hook lints with this checkout's jailint or not at all."""
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import dev


class HookJailintTests(unittest.TestCase):
    def pick(self, *, builds: bool, fresh: bool = False, existing: bool = True) -> tuple[str | None, bool]:
        with tempfile.TemporaryDirectory() as tmp:
            binary = Path(tmp) / "jailint"
            if existing:
                binary.write_text("")
                if not fresh:
                    os.utime(binary, (0, 0))
            hook = dev.Hook(Path(tmp))
            with patch.object(dev, "cargo_binary", return_value=binary), \
                    patch.object(dev, "run_captured", return_value=(0 if builds else 101, "error: no")), \
                    patch.object(dev, "stale", side_effect=lambda path, *_: not fresh), \
                    patch.object(dev, "hook_warn"), \
                    patch("shutil.which", side_effect=lambda name: "/opt/bin/" + name):
                found = dev.hook_jailint(hook)
            return (None if found is None else ("ours" if found == str(binary) else found)), hook.failed

    def test_uses_the_build_from_this_checkout(self):
        self.assertEqual(self.pick(builds=True), ("ours", False))

    def test_a_failed_build_fails_the_hook_instead_of_using_another_jailint(self):
        self.assertEqual(self.pick(builds=False, fresh=False), (None, True))

    def test_never_falls_back_to_an_installed_jailint(self):
        with patch.dict(os.environ, {"JAI_HOOK_NO_BUILD": "1"}):
            self.assertEqual(self.pick(builds=False, existing=False), (None, False))
            self.assertEqual(self.pick(builds=False, fresh=False), (None, False))
            self.assertEqual(self.pick(builds=False, fresh=True), ("ours", False))


if __name__ == "__main__":
    unittest.main()
