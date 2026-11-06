"""Local docs setup is explicit; Astro development exports once without running examples."""

from pathlib import Path
import tempfile
import types
import unittest
from unittest.mock import patch

import dev


def args(**extra):
    values = dict(
        wasm=False,
        no_test_examples=False,
        validate_examples=False,
        setup=False,
        dev=False,
        serve=False,
        port="8123",
        jobs="2",
    )
    values.update(extra)
    return types.SimpleNamespace(**values)


class DocsCommandTests(unittest.TestCase):
    def test_missing_builder_dependencies_fail_before_rust_builds(self):
        with tempfile.TemporaryDirectory() as temporary:
            with (
                patch.object(dev, "ROOT", Path(temporary)),
                patch.object(dev, "cargo_build") as cargo,
            ):
                self.assertEqual(dev.cmd_docs(args()), 2)
                cargo.assert_not_called()

    def test_setup_checks_policy_before_explicit_install_and_development_exports_once(
        self,
    ):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            astro = root / "website/docs/node_modules/astro/bin/astro.mjs"
            astro.parent.mkdir(parents=True)
            astro.write_text("inert fixture")
            (astro.parent.parent / "package.json").write_text(
                '{"bin":{"astro":"./bin/astro.mjs"}}'
            )
            calls = []
            with (
                patch.object(dev, "ROOT", root),
                patch.object(dev.shutil, "which", return_value="/owned/npm"),
                patch.object(dev, "output_dir", side_effect=lambda name: root / name),
                patch.object(dev, "cargo_build", return_value=0) as cargo,
                patch.object(dev, "cargo_binary", side_effect=lambda name, profile="debug": root / profile / name),
                patch.object(dev, "rebuild_jaifmt", side_effect=lambda compiler, **kw: calls.append((["native-formatter", str(compiler), str(kw.get("out"))], {})) or True) as formatter,
                patch.object(
                    dev,
                    "run",
                    side_effect=lambda command, **kw: calls.append((command, kw)) or 0,
                ),
                patch.object(dev, "docs_dev", return_value=0) as development,
            ):
                self.assertEqual(dev.cmd_docs(args(setup=True, dev=True)), 0)
                self.assertEqual(development.call_args.args[0](), 0)
            policy = next(
                i
                for i, (command, _) in enumerate(calls)
                if any("check_docs_dependencies.py" in str(arg) for arg in command)
            )
            install = next(
                i for i, (command, _) in enumerate(calls) if command[-1] == "ci"
            )
            export = next(command for command, _ in calls if "--export-only" in command)
            self.assertLess(policy, install)
            self.assertNotIn("--release", cargo.call_args.args[0], "source docs must use debug Rust builds")
            self.assertEqual(formatter.call_count, 2, "each semantic refresh checks the matching formatter")
            self.assertEqual(formatter.call_args.args[0], root / "debug/jaic")
            self.assertEqual(formatter.call_args.kwargs["out"], root / "debug/jaifmt")
            native = next(i for i, (command, _) in enumerate(calls) if command[0] == "native-formatter")
            extraction = next(i for i, (command, _) in enumerate(calls) if "--export-only" in command)
            self.assertLess(native, extraction)
            self.assertNotIn("--validate-examples", export)
            self.assertNotIn("--out", export)
            self.assertEqual(development.call_count, 1)
            self.assertEqual(development.call_args.kwargs["port"], "8123")


    def test_failed_matching_formatter_build_stops_before_export(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            with (
                patch.object(dev, "ROOT", root),
                patch.object(dev, "astro_cli", return_value=root / "astro.mjs"),
                patch.object(dev, "output_dir", side_effect=lambda name: root / name),
                patch.object(dev, "cargo_build", return_value=0),
                patch.object(dev, "cargo_binary", side_effect=lambda name, profile="debug": root / profile / name),
                patch.object(dev, "rebuild_jaifmt", return_value=False) as formatter,
                patch.object(dev, "run") as run,
                patch.object(dev, "docs_dev") as development,
            ):
                self.assertEqual(dev.cmd_docs(args(dev=True)), 1)
                formatter.assert_called_once_with(root / "debug/jaic", out=root / "debug/jaifmt")
                run.assert_not_called()
                development.assert_not_called()


if __name__ == "__main__":
    unittest.main()
