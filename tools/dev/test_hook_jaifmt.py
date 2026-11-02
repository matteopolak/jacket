"""The pre-commit hook formats with this checkout's jaifmt or not at all."""
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import dev


class HookJaifmtTests(unittest.TestCase):
    def pick(self, stamp: str | None, installed: str = "/opt/bin/jaifmt") -> str | None:
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "jaifmt"
            out.write_text("")
            if stamp is not None:
                dev.jaifmt_stamp(out).write_text(stamp)
            with patch.object(dev, "jaifmt_path", return_value=out), \
                    patch.object(dev, "hook_jaic", return_value=None), \
                    patch.object(dev, "hook_warn"), \
                    patch("shutil.which", return_value=installed):
                found = dev.hook_jaifmt(dev.Hook(Path(tmp)))
                return None if found is None else ("ours" if found == str(out) else found)

    def test_uses_this_checkouts_build(self):
        self.assertEqual(self.pick(str(dev.ROOT)), "ours")

    def test_never_falls_back_to_an_installed_jaifmt(self):
        self.assertIsNone(self.pick("/some/other/checkout"))


if __name__ == "__main__":
    unittest.main()
