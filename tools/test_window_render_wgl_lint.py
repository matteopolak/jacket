"""WGL lint keeps privacy and complete coverage required; no GUI runs."""

import json
from pathlib import Path
import tempfile
import unittest

import window_render_wgl_ci as runner


class WglLintContexts(unittest.TestCase):
    def test_private_sources_have_real_public_context_and_required_coverage(self):
        self.assertTrue(
            hasattr(runner, "lint_jobs"),
            "WGL runner needs enclosing-program lint contexts",
        )
        with tempfile.TemporaryDirectory() as temporary:
            jobs = runner.lint_jobs(
                Path("jailint"), runner.ROOT / "stdlib", Path(temporary)
            )
            self.assertEqual(len(jobs), 2)
            public, private = jobs
            self.assertNotIn("--context", public["command"])
            command = private["command"]
            context = command[command.index("--context") + 1]
            self.assertEqual(
                context, runner.ROOT / "stdlib/Windows/tests/render-events.jai"
            )
            self.assertEqual(
                {
                    p.relative_to(runner.ROOT / "stdlib").as_posix()
                    for p in private["required"]
                },
                {
                    "internal/rendering/window-contexts.jai",
                    "internal/window/lifecycle.jai",
                    "internal/input/native-fields.jai",
                    "internal/input/windows-devices/module.jai",
                },
            )
            for job in jobs:
                self.assertIn("-D", job["command"])
                self.assertIn("warnings", job["command"])
                self.assertEqual(
                    job["command"][job["command"].index("--manifest") + 1],
                    job["manifest"],
                )
            self.assertEqual(len(public["required"]), 17)
            self.assertIn(
                runner.ROOT / "stdlib/Gamepad/SDL_Input_Map/module.jai",
                public["required"],
            )
            for path in public["command"]:
                if isinstance(path, Path) and path.is_dir():
                    self.assertEqual(path, runner.ROOT / "stdlib")

    def test_missing_partial_or_unvisited_manifest_never_passes(self):
        self.assertTrue(
            hasattr(runner, "lint_coverage"), "strict WGL coverage gate is missing"
        )
        with tempfile.TemporaryDirectory() as temporary:
            manifest = Path(temporary) / "coverage.json"
            required = [Path(temporary) / "private.jai"]
            self.assertFalse(runner.lint_coverage(manifest, 0, required)["clean"])
            data = {
                "version": 1,
                "kind": "source-check",
                "incomplete": [],
                "contexts": [
                    {"complete": True, "visited_files": list(map(str, required))}
                ],
                "unvisited_files": [],
            }
            manifest.write_text(json.dumps(data))
            self.assertTrue(runner.lint_coverage(manifest, 0, required)["clean"])
            self.assertFalse(runner.lint_coverage(manifest, 1, required)["clean"])
            data["contexts"][0]["visited_files"] = []
            manifest.write_text(json.dumps(data))
            self.assertFalse(runner.lint_coverage(manifest, 0, required)["clean"])
            data["contexts"][0] = {
                "complete": False,
                "visited_files": list(map(str, required)),
            }
            manifest.write_text(json.dumps(data))
            self.assertFalse(runner.lint_coverage(manifest, 0, required)["clean"])
            manifest.write_text("{")
            self.assertFalse(runner.lint_coverage(manifest, 0, required)["clean"])


if __name__ == "__main__":
    unittest.main()
