import unittest

import release_smoke as smoke


class Sums(unittest.TestCase):
    def test_sha256sum_lines_are_read_by_asset_name(self):
        text = f"{'a' * 64}  jacket-linux-x64.tar.gz\n{'B' * 64} *jacket-windows-x64.zip\nnot a line\n"
        self.assertEqual(
            smoke.parse_sums(text),
            {"jacket-linux-x64.tar.gz": "a" * 64, "jacket-windows-x64.zip": "b" * 64},
        )


class Names(unittest.TestCase):
    def test_archive_names_follow_the_platform(self):
        self.assertEqual(smoke.asset_name("windows-arm64"), "jacket-windows-arm64.zip")
        self.assertEqual(smoke.asset_name("macos-x64"), "jacket-macos-x64.tar.gz")


if __name__ == "__main__":
    unittest.main()
