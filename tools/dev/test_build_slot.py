"""Tests for the machine-wide build slot."""

import os
import subprocess
import sys
import tempfile
import time
import unittest
from unittest import mock
from pathlib import Path

import build_slot

SCRIPT = Path(build_slot.__file__)


class SlotTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.dir = Path(self.tmp.name)
        self.env = {k: v for k, v in os.environ.items() if k != build_slot.HELD}
        self.env.update(JACKET_BUILD_SLOT_DIR=self.tmp.name, JACKET_BUILD_SLOTS="1")

    def spawn(self, *command):
        return subprocess.Popen([sys.executable, str(SCRIPT), "run", "--", *command], env=self.env, start_new_session=True,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)

    def test_free_slot_is_taken_and_released(self):
        with build_slot.slot("a", 1, self.dir):
            self.assertIsNone(self.try_other())
        other = self.try_other()
        self.assertIsNotNone(other)
        other.close()

    def try_other(self):
        return build_slot.try_take(self.dir, 1, "b")

    def test_count_slots_run_together(self):
        first = build_slot.try_take(self.dir, 2, "a")
        second = build_slot.try_take(self.dir, 2, "b")
        self.assertIsNotNone(second)
        self.assertIsNone(build_slot.try_take(self.dir, 2, "c"))
        first.close()
        self.assertIsNotNone(build_slot.try_take(self.dir, 2, "c"))

    def test_waiter_names_the_holder_once_and_runs_after_release(self):
        holder = self.spawn("sleep", "30")
        deadline = time.time() + 10
        while time.time() < deadline and not (self.dir / "slot-0").exists():
            time.sleep(0.05)
        time.sleep(0.3)
        waiter = self.spawn("echo", "ran")
        time.sleep(1.5)
        self.assertIsNone(waiter.poll())
        holder.terminate()
        holder.communicate(timeout=20)
        out, err = waiter.communicate(timeout=20)
        self.assertEqual(out.strip(), "ran")
        self.assertEqual(err.count("waiting for a build slot"), 1)
        self.assertIn("sleep 30", err)
        self.assertIn("pid ", err)

    def test_interrupt_releases_the_slot(self):
        import signal
        holder = self.spawn("sleep", "30")
        time.sleep(1)
        os.killpg(holder.pid, signal.SIGINT)
        holder.communicate(timeout=20)
        other = self.try_other()
        self.assertIsNotNone(other)
        other.close()

    def test_exit_status_passes_through(self):
        self.assertEqual(self.spawn("sh", "-c", "exit 7").wait(timeout=20), 7)

    def test_nested_job_does_not_wait(self):
        inner = [sys.executable, str(SCRIPT), "run", "--", "echo", "inner"]
        done = subprocess.run([sys.executable, str(SCRIPT), "run", "--", *inner], env=self.env,
                              capture_output=True, text=True, timeout=20)
        self.assertEqual(done.stdout.strip(), "inner")

    def test_bad_slot_count_fails(self):
        with mock.patch.dict(os.environ, {"JACKET_BUILD_SLOTS": "0"}):
            with self.assertRaises(SystemExit):
                build_slot.slot_count()


if __name__ == "__main__":
    unittest.main()
