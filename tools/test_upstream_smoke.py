import json
import struct
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import upstream_smoke as smoke  # noqa: E402

PLATFORMS = {"linux-x64", "linux-arm64", "macos-arm64", "macos-x64", "windows-x64"}


def checks():
    return json.loads(smoke.CHECKS.read_text())["checks"]


class CheckFile(unittest.TestCase):
    def test_ids_are_unique(self):
        ids = [c["id"] for c in checks()]
        self.assertEqual(len(ids), len(set(ids)))

    def test_every_check_has_steps_or_a_skip_for_every_platform(self):
        for c in checks():
            with self.subTest(c["id"]):
                if not c["steps"]:
                    self.assertIn("*", c.get("skip", {}), "a check without steps is a documented skip")
                    self.assertGreater(len(c["skip"]["*"]), 20, "say why")

    def test_platform_names_are_known(self):
        for c in checks():
            with self.subTest(c["id"]):
                names = set(c.get("only", [])) | (set(c.get("skip", {})) - {"*"})
                for step in c["steps"]:
                    names |= set(step.get("only", []))
                self.assertLessEqual(names, PLATFORMS)

    def test_a_limited_check_says_why(self):
        for c in checks():
            if "only" in c:
                self.assertTrue(c.get("only_reason"), c["id"])

    def test_projects_exist_in_the_manifest(self):
        pinned = {p["repository"].replace("/", "--", 1) for p in json.loads((smoke.ROOT / "corpus/upstreams.json").read_text())["projects"]}
        for c in checks():
            for name in [c["project"], *c.get("copy", []), *c.get("link", [])]:
                with self.subTest(c["id"], name=name):
                    self.assertTrue(name == "_modules" or name in pinned, name)

    def test_step_kinds_are_implemented(self):
        for c in checks():
            for step in c["steps"]:
                self.assertIn(step.get("kind", "run"), set(smoke.STEPS) | {"jaic"}, c["id"])


class SkipReason(unittest.TestCase):
    def test_only_and_skip(self):
        check = {"only": ["linux-x64"], "only_reason": "r", "skip": {"linux-x64": "no"}}
        self.assertEqual(smoke.skip_reason(check, "linux-x64"), "no")
        self.assertEqual(smoke.skip_reason({"only": ["linux-x64"], "only_reason": "r"}, "macos-arm64"), "r")
        self.assertIsNone(smoke.skip_reason({"only": ["linux-x64"]}, "linux-x64"))
        self.assertEqual(smoke.skip_reason({"skip": {"*": "everywhere"}}, "windows-x64"), "everywhere")


class Output(unittest.TestCase):
    def test_contains_ordered_and_absent(self):
        smoke.check_output("a b c", {"contains": ["b"], "ordered": ["a", "c"], "not_contains": ["z"]}, "t")
        with self.assertRaises(smoke.Failure):
            smoke.check_output("a b c", {"ordered": ["c", "a"]}, "t")
        with self.assertRaises(smoke.Failure):
            smoke.check_output("a b c", {"not_contains": ["b"]}, "t")


class Screenshots(unittest.TestCase):
    def bmp(self, path, pixels, width, height, top_down):
        rows = []
        for y in range(height):
            row = b"".join(bytes(pixels[y * width + x]) + b"\xff" for x in range(width))
            rows.append(row)
        if not top_down:
            rows.reverse()
        data = b"".join(rows)
        header = b"BM" + struct.pack("<IHHI", 54 + len(data), 0, 0, 54)
        dib = struct.pack("<IiiHHIIiiII", 40, width, -height if top_down else height, 1, 32, 0, len(data), 0, 0, 0, 0)
        Path(path).write_bytes(header + dib + data)

    def test_bitmap_rows_in_either_order(self):
        pixels = [(1, 2, 3), (4, 5, 6), (7, 8, 9), (10, 11, 12)]
        with tempfile.TemporaryDirectory() as d:
            for top_down in (True, False):
                path = Path(d) / f"s{top_down}.bmp"
                self.bmp(path, pixels, 2, 2, top_down)
                got = smoke.sample_bmp(path, cols=2, rows=2)
                self.assertEqual([bytes(p) for p in pixels], got)


if __name__ == "__main__":
    unittest.main()
