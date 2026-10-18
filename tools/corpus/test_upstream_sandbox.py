import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from bench.sandbox import Sandbox
from corpus import upstream_sandbox as us


class PolicyTests(unittest.TestCase):
    def test_a_sandbox_is_used_when_the_platform_has_one(self):
        self.assertTrue(us.policy(False, {}, available=lambda: ""))

    def test_a_platform_without_one_refuses_outside_ci(self):
        with self.assertRaises(us.Unsandboxed):
            us.policy(False, {}, available=lambda: "no sandbox on win32")

    def test_ci_runners_run_unconfined(self):
        self.assertFalse(us.policy(False, {"GITHUB_ACTIONS": "true"}, available=lambda: "no sandbox"))

    def test_only_the_explicit_flag_opens_a_local_run(self):
        self.assertFalse(us.policy(True, {}, available=lambda: ""))


class WrapTests(unittest.TestCase):
    def test_confine_wraps_and_redirects_temporary_files(self):
        with tempfile.TemporaryDirectory() as scratch:
            box = us.make_sandbox([scratch], [scratch])
            command, env = us.confine(box, Path(scratch), ["jaic", "x.jai"], {"A": "1"})
            self.assertEqual(command[-2:], ["jaic", "x.jai"])
            self.assertNotEqual(command, ["jaic", "x.jai"])
            self.assertTrue(env["TMPDIR"].startswith(str(Path(scratch))))
            self.assertEqual(env["A"], "1")

    def test_confine_keeps_a_step_environment_override(self):
        with tempfile.TemporaryDirectory() as scratch:
            box = us.make_sandbox([scratch], [scratch])
            _, env = us.confine(box, Path(scratch), ["x"], {"HOME": "/work/home"})
            self.assertEqual(env["HOME"], "/work/home")

    def test_smoke_steps_are_wrapped_only_when_the_check_has_a_box(self):
        import upstream_smoke as smoke

        with tempfile.TemporaryDirectory() as work:
            ctx = {"work": Path(work), "box": us.make_sandbox([work], [work])}
            argv, _ = smoke.confined(ctx, ["./prog"], {})
            self.assertEqual(argv[-1], "./prog")
            self.assertNotEqual(argv, ["./prog"])
            self.assertEqual(smoke.confined({"work": Path(work), "box": None}, ["./prog"], {"A": "1"}), (["./prog"], {"A": "1"}))
            argv, _ = smoke.confined(ctx, ["./srv"], {}, loopback=True)
            self.assertNotEqual(argv, ["./srv"])

    def test_no_sandbox_leaves_the_command_alone(self):
        self.assertEqual(us.confine(None, Path("."), ["a"], {"B": "2"}), (["a"], {"B": "2"}))

    def test_reads_cover_the_jaic_build_stdlib_and_sources_only(self):
        reads = us.jaic_reads("/b/target/debug/jaic", Path("/r/stdlib"), Path("/r/corpus/upstream"), {"JAIC_NATIVE_LIBS": "/n"})
        self.assertEqual([str(p) for p in reads][1:5], ["/r/stdlib", "/r/corpus/upstream", "/n", sys.base_prefix])
        self.assertTrue(str(reads[0]).endswith("/b/target") or str(reads[0]).endswith("b/target"))


@unittest.skipUnless(sys.platform == "darwin" and not Sandbox.available(), "needs sandbox-exec")
class ConfinementTests(unittest.TestCase):
    def test_a_confined_program_cannot_write_outside_its_scratch_directory(self):
        with tempfile.TemporaryDirectory() as scratch, tempfile.TemporaryDirectory() as outside:
            box = us.make_sandbox([], [scratch])
            command, env = us.confine(box, Path(scratch), ["/usr/bin/touch", str(Path(outside) / "x")])
            self.assertNotEqual(subprocess.run(command, env=env, capture_output=True).returncode, 0)
            command, env = us.confine(box, Path(scratch), ["/usr/bin/touch", str(Path(scratch) / "x")])
            self.assertEqual(subprocess.run(command, env=env, capture_output=True).returncode, 0)


if __name__ == "__main__":
    unittest.main()
