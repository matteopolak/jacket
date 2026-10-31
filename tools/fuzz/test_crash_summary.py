import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import crash_summary
from crash_summary import Replay

PANIC = """\
warning: something unrelated
thread 'main' panicked at fuzz/harness/src/lib.rs:41:9:
attempt to add with overflow
second line
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
"""

BACKTRACE = """\
thread 'main' panicked at crates/jaic-syntax/src/lexer.rs:7:5:
index out of bounds: the len is 3 but the index is 7
stack backtrace:
   0: rust_begin_unwind
   1: core::panicking::panic_fmt
   2: jaic::lexer::lex
"""


class Rendering(unittest.TestCase):
    def test_hex_dump_lines_and_truncation(self):
        dump = crash_summary.hex_dump(bytes(range(40)), limit=32)
        lines = dump.splitlines()
        self.assertEqual(
            lines[0],
            "00000000  00 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f  ................",
        )
        self.assertTrue(lines[1].startswith("00000010  10 11 12"))
        self.assertEqual(lines[-1], "... 8 more bytes")
        self.assertEqual(crash_summary.hex_dump(b"A" * 16).splitlines()[0][-16:], "A" * 16)

    def test_escaped_text_shows_control_bytes_and_invalid_utf8(self):
        self.assertEqual(
            crash_summary.escape_text(b"a\nb\t\x00\\ \xff"), "a\\nb\\t\\x00\\\\ \\xff"
        )
        self.assertEqual(crash_summary.escape_text("é✓".encode()), "é✓")

    def test_escaped_text_is_cut(self):
        text = crash_summary.escape_text(b"x" * 50, limit=10)
        self.assertEqual(text, "x" * 10 + "... (40 more characters)")

    def test_a_fence_outgrows_backticks_in_its_content(self):
        self.assertEqual(crash_summary.fence("a"), "```\na\n```")
        self.assertTrue(crash_summary.fence("``` x").startswith("````\n"))


class Panics(unittest.TestCase):
    def test_the_message_runs_to_the_backtrace_hint(self):
        message, backtrace = crash_summary.extract_panic(PANIC)
        self.assertEqual(
            message,
            "thread 'main' panicked at fuzz/harness/src/lib.rs:41:9:\nattempt to add with overflow\nsecond line",
        )
        self.assertEqual(backtrace, "")

    def test_the_backtrace_starts_at_its_header(self):
        message, backtrace = crash_summary.extract_panic(BACKTRACE)
        self.assertIn("index out of bounds", message)
        self.assertNotIn("stack backtrace", message)
        self.assertTrue(backtrace.startswith("stack backtrace:\n   0: rust_begin_unwind"))

    def test_no_panic_gives_nothing(self):
        self.assertEqual(crash_summary.extract_panic("all good\n"), ("", ""))

    def test_how_a_replay_ended(self):
        self.assertIn("hang", crash_summary.describe_end(Replay(None, ""), 10))
        self.assertIn("signal 11", crash_summary.describe_end(Replay(-11, ""), 10))
        self.assertIn("status 3", crash_summary.describe_end(Replay(3, ""), 10))
        self.assertIn("not reproduced", crash_summary.describe_end(Replay(0, ""), 10))


class Summary(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.dir = Path(self.tmp.name)

    def test_only_libfuzzer_findings_are_listed(self):
        for name in ("crash-1", "timeout-2", "notes.txt", "oom-3"):
            (self.dir / name).write_bytes(b"x")
        self.assertEqual(
            sorted(p.name for p in crash_summary.findings(self.dir)),
            ["crash-1", "oom-3", "timeout-2"],
        )
        self.assertEqual(crash_summary.findings(self.dir / "missing"), [])

    def test_a_panicking_input_gets_its_message_and_both_dumps(self):
        (self.dir / "crash-abc").write_bytes(b"main :: () {\n}\n")
        text = crash_summary.summarize(
            "check", self.dir, 3, replayer=lambda path: Replay(101, PANIC)
        )
        self.assertIn("### fuzz `check`: 1 finding(s)", text)
        self.assertIn("#### `crash-abc` (15 bytes)", text)
        self.assertIn("attempt to add with overflow", text)
        self.assertIn("6d 61 69 6e 20 3a 3a", text)
        self.assertIn("main :: () {\\n}\\n", text)

    def test_files_past_the_limit_are_listed_not_replayed(self):
        for index in range(3):
            path = self.dir / f"crash-{index}"
            path.write_bytes(b"z" * (index + 1))
            os.utime(path, (index, index))
        seen = []

        def replayer(path):
            seen.append(path.name)
            return Replay(101, PANIC)

        text = crash_summary.summarize("check", self.dir, 1, replayer=replayer)
        self.assertEqual(seen, ["crash-0"])
        self.assertIn("`crash-1` (2 bytes), `crash-2` (3 bytes)", text)

    def test_a_hang_is_reported_without_a_message(self):
        (self.dir / "timeout-1").write_bytes(b"loop")
        text = crash_summary.summarize(
            "check", self.dir, 3, replayer=lambda path: Replay(None, "")
        )
        self.assertIn("still running after 10 s", text)

    def test_nothing_found_writes_nothing(self):
        with mock.patch.dict(os.environ, {"GITHUB_STEP_SUMMARY": str(self.dir / "out.md")}):
            self.assertEqual(crash_summary.main(["check", "--artifacts", str(self.dir)]), 0)
        self.assertFalse((self.dir / "out.md").exists())

    def test_the_summary_is_appended_to_the_step_summary(self):
        (self.dir / "crash-1").write_bytes(b"x")
        out = self.dir / "out.md"
        out.write_text("earlier\n")
        with (
            mock.patch.dict(os.environ, {"GITHUB_STEP_SUMMARY": str(out)}),
            mock.patch.object(crash_summary, "build_replay", return_value=True),
            mock.patch.object(crash_summary, "replay_input", return_value=Replay(101, PANIC)),
        ):
            crash_summary.main(["check", "--artifacts", str(self.dir / ".")])
        self.assertTrue(out.read_text().startswith("earlier\n### fuzz `check`"))


if __name__ == "__main__":
    unittest.main()
