"""The star-qualified (unreviewed) part of the upstream corpus: lists, binary guard, review gate."""

import json
import sys
import unittest
from pathlib import Path

import fetch_upstreams as fetch
import select_upstream_cases as sel

ROOT = Path(__file__).resolve().parents[2]


def directory(repo: str) -> str:
    return repo.replace("/", "--", 1)


class Lists(unittest.TestCase):
    def test_lists_are_disjoint(self):
        star, excluded = set(fetch.STAR_REPOSITORIES), set(fetch.EXCLUDED_REPOSITORIES)
        other = set(fetch.REPOSITORIES + fetch.DEPENDENCIES + fetch.LIBRARIES)
        self.assertEqual(len(star), len(fetch.STAR_REPOSITORIES))
        for a, b in ((star, excluded), (star, other)):
            self.assertFalse(a & b, a & b)

    def test_star_repositories_are_in_the_manifest(self):
        pinned = {p["repository"] for p in json.loads((ROOT / "corpus/upstreams.json").read_text())["projects"]}
        self.assertTrue(set(fetch.STAR_REPOSITORIES) <= pinned)


class BinaryGuard(unittest.TestCase):
    def test_suffixes_and_magic(self):
        for name in ("a.dll", "dir/b.so", "c.so.1", "d.DYLIB", "e.exe", "f.a", "g.lib", "h.o", "i.bin", "j.zip", "k.tar.gz", "l.wasm"):
            self.assertTrue(fetch.is_forbidden_file(name), name)
        for name in ("a.jai", "b.h", "README.md", "c.png", "fonts/d.ttf", "lib/e.jai"):
            self.assertFalse(fetch.is_forbidden_file(name), name)
        self.assertTrue(fetch.is_forbidden_file("notes.txt", b"MZ\x90\x00"))
        self.assertTrue(fetch.is_forbidden_file("x.jai", b"\x7fELF\x02\x01"))
        self.assertTrue(fetch.is_forbidden_file("x.h", b"\xcf\xfa\xed\xfe"))
        self.assertFalse(fetch.is_forbidden_file("x.jai", b"main :: () {}"))

    def test_selection_takes_no_binary(self):
        paths = ["a.jai", "lib/x.dll", "bin/tool.exe", "src/b.c", "README.md", "LICENSE", "data/c.png", "d.so.1"]
        self.assertEqual(fetch.select_paths("someone/star", paths), ["LICENSE", "README.md", "a.jai", "src/b.c"])

    def test_manifest_lists_no_files(self):
        manifest = json.loads((ROOT / "corpus/upstreams.json").read_text())
        self.assertEqual(manifest["format"], 2)
        for project in manifest["projects"]:
            self.assertNotIn("files", project, project["repository"])

    def test_star_repositories_take_no_resource_prefixes(self):
        self.assertFalse(set(fetch.STAR_REPOSITORIES) & set(fetch.RESOURCE_PREFIXES))

    def test_vendored_official_modules_are_left_out(self):
        paths = {"modules/Basic/module.jai", "modules/Basic/x.jai", "modules/Mine/module.jai", "modules/Math.jai", "src/Basic.jai"}
        self.assertTrue(fetch.vendored_official_module("modules/Basic/x.jai", paths))
        self.assertTrue(fetch.vendored_official_module("modules/Basic/module.jai", paths))
        self.assertTrue(fetch.vendored_official_module("modules/Math.jai", paths))
        self.assertFalse(fetch.vendored_official_module("modules/Mine/module.jai", paths))
        self.assertFalse(fetch.vendored_official_module("src/Basic.jai", paths))

    def test_renamed_and_unlisted_copies_are_left_out(self):
        # By path alone: lower-cased directory and file names, a module missing from the first
        # version of the list, and a directory in `modules` that has no `module.jai`.
        paths = {
            "src/modules/curl/module.jai",
            "jai/modules/autorun.jai",
            "codex_view/modules/Codex/module.jai",
            "codex_view/modules/Codex/read.jai",
            "vendor/modules/Windows/windows.jai",
            "src/curl/module.jai",
            "src/modules/colors/module.jai",
            "src/modules/mybasic.jai",
            "src/mine/curl.jai",
        }
        for path in (
            "src/modules/curl/module.jai",
            "jai/modules/autorun.jai",
            "codex_view/modules/Codex/module.jai",
            "codex_view/modules/Codex/read.jai",
            "vendor/modules/Windows/windows.jai",
            "src/curl/module.jai",
        ):
            self.assertTrue(fetch.vendored_official_module(path, paths), path)
        for path in ("src/modules/colors/module.jai", "src/modules/mybasic.jai", "src/mine/curl.jai"):
            self.assertFalse(fetch.vendored_official_module(path, paths), path)

    def test_a_repositorys_own_bindings_are_kept(self):
        paths = ["sokol/gl/module.jai", "sokol/gfx/module.jai"]
        self.assertEqual(fetch.select_paths("colinbellino/sokol-jai", paths), ["sokol/gfx/module.jai", "sokol/gl/module.jai"])
        self.assertEqual(fetch.select_paths("farzher/clipman", ["src/modules/curl/module.jai", "src/main.jai"]), ["src/main.jai"])


class ReviewGate(unittest.TestCase):
    def cases(self):
        return json.loads((ROOT / "tools/upstream-cases.json").read_text())

    def test_pending_cases_are_never_selected_for_a_platform(self):
        cases = [
            {"id": "a", "path": "o--r/x.jai", "only": ["windows-x64"], "review": "pending"},
            {"id": "b", "path": "o--s/x.jai", "only": ["windows-x64"]},
        ]
        self.assertEqual([c["id"] for c in sel.select(cases, "windows-x64")], ["b"])

    def test_pending_cases_are_unrestricted(self):
        pending = [c for c in self.cases() if sel.pending(c)]
        self.assertGreater(len(pending), 0)
        for case in pending:
            self.assertNotIn("only", case, case["id"])

    def test_no_reviewed_case_borrows_a_pending_repository(self):
        pending = {d for c in self.cases() if sel.pending(c) for d in sel.directories(c)}
        for case in self.cases():
            if not sel.pending(case):
                self.assertFalse(pending & set(sel.directories(case)), case["id"])

    def test_the_smoke_checks_use_no_star_repository(self):
        star = {directory(r) for r in fetch.STAR_REPOSITORIES}
        checks = json.loads((ROOT / "tools/corpus/upstream-smoke.json").read_text())["checks"]
        self.assertFalse({c.get("project") for c in checks} & star)


if __name__ == "__main__":
    unittest.main()
