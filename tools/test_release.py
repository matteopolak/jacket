import contextlib
import io
import re
import subprocess
import tarfile
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import release as rel


def done(code=0, out="", err=""):
    return subprocess.CompletedProcess([], code, out, err)


def run_main(argv, environment=None):
    """(exit code, stdout) of `release.py argv` with the given environment and no CI variables."""
    out = io.StringIO()
    with mock.patch.dict("os.environ", environment or {}, clear=True), contextlib.redirect_stdout(out):
        code = rel.main(argv)
    return code, out.getvalue()


LOG = """# Changelog

## [Unreleased]

## [0.2.0] - 2026-01-02

### Added

- b

## [0.1.0] - 2026-01-01

- a
"""


class Notes(unittest.TestCase):
    def test_takes_the_section_up_to_the_next_heading(self):
        self.assertEqual(rel.changelog_section(LOG, "0.2.0"), "### Added\n\n- b\n")

    def test_tag_prefix_and_last_section(self):
        self.assertEqual(rel.changelog_section(LOG, "v0.1.0"), "- a\n")

    def test_missing_or_empty_section_is_empty(self):
        self.assertEqual(rel.changelog_section(LOG, "0.3.0"), "")
        self.assertEqual(rel.changelog_section(LOG, "Unreleased"), "")

    def test_does_not_match_a_version_prefix(self):
        self.assertEqual(rel.changelog_section("## [0.20.0]\n- x\n", "0.2.0"), "")

    def test_the_real_changelog_has_the_current_top_section(self):
        text = rel.ROOT.joinpath("CHANGELOG.md").read_text(encoding="utf-8")
        version = re.search(r"^## \[(\d+\.\d+\.\d+)\]", text, re.M)[1]
        self.assertTrue(rel.changelog_section(text, version))

    def test_the_step_prints_the_section_and_fails_without_one(self):
        with tempfile.TemporaryDirectory() as scratch:
            path = Path(scratch) / "CHANGELOG.md"
            path.write_text(LOG)
            self.assertEqual(run_main(["notes", "v0.2.0", "--changelog", str(path)]), (0, "### Added\n\n- b\n"))
            with self.assertRaises(SystemExit) as raised, mock.patch("sys.stderr"):
                run_main(["notes", "0.3.0", "--changelog", str(path)])
            self.assertEqual(raised.exception.code, 1)


class Metadata(unittest.TestCase):
    def meta(self, tag="", editor="", integrations="", event="workflow_dispatch", version="0.5.0"):
        return rel.release_metadata(version, tag, editor, integrations, event)

    def test_a_run_without_a_tag_is_a_dry_run(self):
        values = self.meta()
        self.assertEqual((values["dry_run"], values["is_current_release"], values["publish_stores"]), ("true", "false", "false"))
        self.assertEqual((values["editor_version"], values["editor_pinned"], values["editor_valid"]), ("0.5.0", "false", "true"))

    def test_the_current_tag_pushed_publishes_everything(self):
        values = self.meta(tag="v0.5.0", event="push")
        self.assertEqual((values["is_current_release"], values["publish_stores"], values["publish_editor_stores"]),
                         ("true", "true", "true"))
        self.assertEqual(values["editor_version"], "0.5.0")

    def test_a_historical_tag_never_publishes_to_the_editor_stores(self):
        values = self.meta(tag="v0.4.0", event="push")
        self.assertEqual((values["is_current_release"], values["publish_stores"], values["publish_editor_stores"]),
                         ("false", "true", "false"))

    def test_a_dispatch_publishes_the_editor_stores_only_with_a_pinned_editor_version(self):
        self.assertEqual(self.meta(tag="v0.5.0", integrations="true")["publish_editor_stores"], "false")
        pinned = self.meta(tag="v0.5.0", integrations="true", editor="0.5.1")
        self.assertEqual((pinned["publish_editor_stores"], pinned["editor_pinned"], pinned["editor_version"]), ("true", "true", "0.5.1"))
        self.assertEqual(self.meta(tag="v0.5.0", editor="0.5.1")["publish_editor_stores"], "false")

    def test_the_editor_version_must_be_x_y_z(self):
        self.assertEqual(self.meta(editor="1.2")["editor_valid"], "false")
        self.assertEqual(self.meta(editor="1.2.3")["editor_valid"], "true")
        self.assertEqual(self.meta(tag="v1.2.3-rc1")["editor_valid"], "false")

    def test_the_step_writes_github_output(self):
        with tempfile.TemporaryDirectory() as scratch:
            output = Path(scratch) / "out"
            run_main(["meta", "--tag", "v0.30.4", "--event", "push"], {"GITHUB_OUTPUT": str(output)})
            lines = output.read_text().splitlines()
        self.assertEqual(lines[0], f"version={rel.workspace_version(rel.ROOT)}")
        self.assertIn("tag=v0.30.4", lines)
        self.assertEqual([line.split("=")[0] for line in lines], [
            "version", "tag", "is_current_release", "dry_run", "publish_stores", "publish_editor_stores",
            "editor_version", "editor_valid", "editor_pinned"])

    def test_environment_variables_are_the_defaults(self):
        code, out = run_main(["meta", "--dry-run"], {"TAG": "v0.30.4", "EVENT": "push"})
        self.assertIn("tag=v0.30.4", out)
        self.assertIn("publish_stores=true", out)


def fake_gh(tag="v1", assets="SHA256SUMS\njacket-linux-x64.tar.gz\n", archive_members=("jacket-linux-x64/jacket",), view_error=None):
    def run(*args):
        if args[:2] == ("release", "view") and "tagName" in args:
            return done(1, "", view_error) if view_error else done(0, tag + "\n")
        if args[:2] == ("release", "view"):
            return done(0, assets)
        if args[:2] == ("release", "download"):
            directory = Path(args[args.index("--dir") + 1])
            with tarfile.open(directory / args[args.index("--pattern") + 1], "w:gz") as tar:
                for name in archive_members:
                    tar.addfile(tarfile.TarInfo(name), io.BytesIO(b""))
            return done()
        raise AssertionError(args)

    return run


class Published(unittest.TestCase):
    def check(self, run, platform=None):
        with tempfile.TemporaryDirectory() as scratch, contextlib.redirect_stdout(io.StringIO()):
            return rel.published_release("o/r", platform, run=run, scratch=scratch)

    def test_no_release_yet_is_a_skip(self):
        self.assertEqual(self.check(fake_gh(view_error="release not found")), (False, None))

    def test_other_gh_failures_fail(self):
        with self.assertRaises(SystemExit), mock.patch("sys.stderr"):
            self.check(fake_gh(view_error="HTTP 502"))

    def test_old_release_without_platform_archives_is_a_skip(self):
        self.assertEqual(self.check(fake_gh(assets="SHA256SUMS\nold.zip\n")), (False, None))

    def test_published_release(self):
        self.assertEqual(self.check(fake_gh()), (True, None))

    def test_formula_needs_the_jacket_binary_in_the_archive(self):
        self.assertEqual(self.check(fake_gh(), "linux-x64"), (True, True))
        self.assertEqual(self.check(fake_gh(archive_members=("jacket-linux-x64/jaic",)), "linux-x64"), (True, False))

    def test_platform_names(self):
        self.assertEqual(rel.PLATFORMS["macOS-ARM64"], "macos-arm64")


class Publish(unittest.TestCase):
    """The branches of the GitHub release step, with `gh` replaced."""

    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.dist = Path(scratch.name) / "dist"
        self.dist.mkdir()
        (self.dist / "b.tar.gz").write_text("")
        (self.dist / "a.tar.gz").write_text("")
        self.calls = []

    def publish(self, view, tag="v1.2.3", current="1.2.3"):
        def fake(command, **options):
            self.calls.append(command)
            return view if command[:3] == ["gh", "release", "view"] else done()

        argv = ["publish", "--tag", tag, "--current-version", current, "--repo", "o/r", "--dist", str(self.dist), "--notes", "n.md"]
        with mock.patch("subprocess.run", fake), mock.patch("sys.stderr", io.StringIO()):
            return run_main(argv)

    def verbs(self):
        return [" ".join(call[1:3]) for call in self.calls]

    def test_a_missing_release_is_created_and_marked_latest_when_current(self):
        self.publish(done(1, "", "HTTP 404: Release Not Found"))
        self.assertEqual(self.verbs(), ["release view", "release create"])
        create = self.calls[1]
        self.assertEqual(create[3:6], ["v1.2.3", f"{self.dist}/a.tar.gz", f"{self.dist}/b.tar.gz"])
        self.assertIn("--latest", create)
        self.assertEqual(create[create.index("--title") + 1], "Jacket 1.2.3")
        self.assertEqual(create[-1], "--verify-tag")

    def test_a_historical_tag_is_not_latest(self):
        self.publish(done(1, "", "release not found"), tag="v1.0.0")
        self.assertIn("--latest=false", self.calls[1])
        self.assertNotIn("--latest", self.calls[1])

    def test_an_existing_mutable_release_is_uploaded_and_edited(self):
        self.publish(done(0, '{"isImmutable": false}'))
        self.assertEqual(self.verbs(), ["release view", "release upload", "release edit"])
        self.assertIn("--clobber", self.calls[1])
        self.assertEqual(self.calls[2][-1], "--latest")
        self.publish(done(0, '{"isImmutable": false}'), tag="v1.0.0")
        self.assertNotIn("--latest", self.calls[-1])

    def test_an_immutable_release_fails_before_changing_anything(self):
        with self.assertRaises(SystemExit) as raised:
            self.publish(done(0, '{"isImmutable": true}'))
        self.assertEqual(raised.exception.code, 1)
        self.assertEqual(self.verbs(), ["release view"])

    def test_any_other_failure_fails(self):
        with self.assertRaises(SystemExit):
            self.publish(done(1, "", "HTTP 502"))
        self.assertEqual(self.verbs(), ["release view"])


class DryRun(unittest.TestCase):
    """--dry-run prints commands and runs nothing."""

    ENVIRONMENT = {"TAP_TOKEN": "tap-secret", "OVSX_PAT": "ovsx-secret", "VSCE_PAT": "vsce-secret",
                   "DEPLOY_KEY": "deploy-secret", "RUNNER_TEMP": "/runner/tmp"}

    def dry(self, argv):
        with mock.patch("subprocess.run", side_effect=AssertionError("a dry run ran a command")):
            return run_main([*argv, "--dry-run"], self.ENVIRONMENT)[1]

    def test_publishing_steps_run_nothing_and_hide_secrets(self):
        out = self.dry(["all", "--tag", "v1.2.3", "--version", "1.2.3", "--current-version", "1.2.3",
                        "--editor-version", "1.2.3", "--repo", "o/r"])
        self.assertTrue(out)
        for secret in ("tap-secret", "ovsx-secret", "deploy-secret", "vsce-secret"):
            self.assertNotIn(secret, out)

    def test_the_portfolio_bump_is_the_last_step(self):
        out = self.dry(["all", "--tag", "v1.2.3", "--version", "1.2.3", "--repo", "o/r"])
        steps = re.findall(r"^== (.*)$", out, re.M)
        self.assertEqual(steps[-1], "portfolio")
        self.assertEqual(steps, list(rel.PUBLISHING))
        self.assertTrue(out.rstrip().endswith("+ gh workflow run jai-web.yml -R matteopolak/portfolio -f jai_ref=v1.2.3"))

    def test_the_portfolio_step_refuses_a_malformed_tag(self):
        with self.assertRaises(SystemExit):
            self.dry(["portfolio", "--tag", "main"])

    def test_steps_without_a_tag_or_secret_do_nothing(self):
        with mock.patch("subprocess.run", side_effect=AssertionError):
            self.assertIn("No tag", run_main(["homebrew", "--version", "1.2.3"], self.ENVIRONMENT)[1])
            self.assertIn("not set", run_main(["open-vsx"])[1])
            self.assertIn("No tag", run_main(["nvim-mirror", "--version", "1.2.3", "--editor-version", "1.2.3"], self.ENVIRONMENT)[1])

    def test_the_stamp_writes_nothing_in_a_dry_run(self):
        before = (rel.ROOT / "editors/nvim/lua/jai/version.lua").read_text()
        self.dry(["nvim-stamp", "--version", "9.9.9"])
        self.assertEqual((rel.ROOT / "editors/nvim/lua/jai/version.lua").read_text(), before)


class Steps(unittest.TestCase):
    def test_every_step_is_listed(self):
        code, out = run_main(["--list"])
        names = [line.split()[0] for line in out.splitlines()]
        self.assertEqual(names, ["meta", "notes", "published", "publish", "marketplace", "open-vsx", "homebrew",
                                 "nvim-stamp", "nvim-mirror", "portfolio", "all"])

    def test_the_stamp_writes_the_generated_plugin_version(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            (root / "editors/nvim/lua/jai").mkdir(parents=True)
            with mock.patch.object(rel, "ROOT", root):
                run_main(["nvim-stamp", "--version", "1.2.3"])
            self.assertEqual((root / "editors/nvim/lua/jai/version.lua").read_text(),
                             rel.generate_version_files.render("1.2.3")["editors/nvim/lua/jai/version.lua"])


if __name__ == "__main__":
    unittest.main()
