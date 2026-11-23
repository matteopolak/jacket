"""The native-libs cache key covers the sources of every library built from this repository."""
import json
import unittest

from toolkit.repo import ROOT


class CacheKey(unittest.TestCase):
    def test_every_local_folder_of_the_manifest_is_in_the_key(self):
        manifest = json.loads((ROOT / "tools/native/native-libs.json").read_text())
        action = (ROOT / ".github/actions/native-libs-cache/action.yml").read_text()
        key = next(line for line in action.splitlines() if line.strip().startswith('key="native-libs-'))
        locals_ = {lib["local"] for lib in manifest["libraries"].values() if "local" in lib}
        self.assertTrue(locals_)
        for folder in sorted(locals_):
            covered = f"'{folder}/**'" in key or folder.startswith("tools/native/")
            self.assertTrue(covered, f"{folder} is built into a library but is not in the cache key")


if __name__ == "__main__":
    unittest.main()
