"""Every platform `toolkit.platforms` builds for is a target `crates/jaic/src/target.rs` names (the compiler
also targets CPUs and systems the tools never build for: i686, s390x, bare metal)."""

import re
import unittest

from toolkit.platforms import PLATFORMS, TRIPLES
from toolkit.repo import ROOT

SOURCE = (ROOT / "crates/jaic/src/target.rs").read_text()
RUST_ARCH = {"x64": "x86_64", "arm64": "aarch64"}


def rust_os_names():
    body = re.search(r"OS_NAMES: \[&str; \d+\] = \[(.*?)\];", SOURCE, re.S).group(1)
    return set(re.findall(r'"(\w+)"', body))


def rust_cpu_names():
    body = re.search(r"fn cpu_name\(.*?\n}\n", SOURCE, re.S).group(0)
    return set(re.findall(r'=> "(\w+)"', body)) - {"wasm"}


def rust_display_suffixes():
    """The `<vendor>-<os>[-<env>]` part `Target`'s Display writes, per `(OS, env)` arm."""
    body = re.search(r"let rest = match.*?\n        };", SOURCE, re.S).group(0)
    return set(re.findall(r'=> "([\w-]+)"', body))


class PlatformTables(unittest.TestCase):
    def test_rust_parsing_finds_names(self):
        # Guards the regexes above against a refactor of target.rs.
        self.assertTrue(rust_os_names() and rust_cpu_names() and rust_display_suffixes())

    def test_every_platform_names_a_compiler_os_and_cpu(self):
        self.assertLessEqual({p.split("-")[0] for p in PLATFORMS}, rust_os_names())
        self.assertLessEqual({p.split("-")[1] for p in PLATFORMS}, rust_cpu_names())

    def test_every_platform_is_a_rust_target(self):
        suffixes = rust_display_suffixes()
        for platform, triple in TRIPLES.items():
            os_name, cpu = platform.split("-")
            if os_name == "macos":
                # Clang spells it `-apple-macos`, rustc `-apple-darwin`.
                self.assertIn("apple-darwin", suffixes)
                self.assertTrue(triple.endswith("-apple-macos"), triple)
            else:
                self.assertIn(triple, {f"{RUST_ARCH[cpu]}-{s}" for s in suffixes})


if __name__ == "__main__":
    unittest.main()
