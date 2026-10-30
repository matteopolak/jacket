#!/usr/bin/env python3
"""The release command: every step the release workflow and a maintainer run, by name.

    python3 tools/release.py <step> [options] [--dry-run]
    python3 tools/release.py --list

Steps, in release order (docs/tools/releases.md):

    meta         compute the release metadata and write it to $GITHUB_OUTPUT
    notes        print the CHANGELOG.md section of a version
    published    is there a published release the installer jobs can install (writes $GITHUB_OUTPUT)
    publish      create the GitHub release from dist/ and notes.md, or update a mutable one
    marketplace  publish the packaged VSIX to the VS Code Marketplace
    open-vsx     publish the packaged VSIX to Open VSX
    homebrew     push the rendered formula to matteopolak/homebrew-tap
    nvim-stamp   stamp the version and the release's toolchain manifest into editors/nvim
    nvim-mirror  replace matteopolak/jacket.nvim's main branch with editors/nvim and tag it
    portfolio    dispatch the portfolio's jai-web.yml for the tag
    all          the publishing steps (publish .. portfolio) in that order

Every option defaults to the environment variable the workflow sets (TAG, CURRENT_VERSION, GITHUB_REPOSITORY,
...); secrets (GH_TOKEN, TAP_TOKEN, OVSX_PAT, VSCE_PAT, DEPLOY_KEY) are only ever read from the environment.
`--dry-run` runs nothing and writes nothing: it prints each command the step would run, with secrets hidden.
"""

import argparse
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

from generators import generate_version_files
from toolkit.repo import ROOT
from toolkit.version import workspace_version

TAP_REPOSITORY = "matteopolak/homebrew-tap"
MIRROR_REPOSITORY = "matteopolak/jacket.nvim"
PORTFOLIO_REPOSITORY = "matteopolak/portfolio"
PORTFOLIO_WORKFLOW = "jai-web.yml"
NAMESPACE = "matteopolak"
# GitHub's published ed25519 host key fingerprint, so the mirror push cannot go to an impostor.
GITHUB_HOST_KEY = "SHA256:+DiY3wvvV6TuJJhbpZisF/zLDA0zPMSvHdkr4UvCOqU"
BOT = ["-c", "user.name=github-actions[bot]", "-c", "user.email=41898282+github-actions[bot]@users.noreply.github.com"]
EDITOR_VERSION = re.compile(r"[0-9]+\.[0-9]+\.[0-9]+")
PLATFORMS = {
    "Linux-X64": "linux-x64",
    "Linux-ARM64": "linux-arm64",
    "macOS-X64": "macos-x64",
    "macOS-ARM64": "macos-arm64",
}
# What `all` runs, after the build and packaging jobs.
PUBLISHING = ("publish", "marketplace", "open-vsx", "homebrew", "nvim-stamp", "nvim-mirror", "portfolio")


class Shell:
    """Runs commands, or with `dry_run` only prints them (secrets replaced by ***)."""

    def __init__(self, dry_run=False, secrets=()):
        self.dry_run = dry_run
        self.secrets = [secret for secret in secrets if secret]

    def show(self, command, cwd=None):
        text = shlex.join(str(part) for part in command)
        for secret in self.secrets:
            text = text.replace(secret, "***")
        print("+ " + text + (f"  (in {cwd})" if cwd else ""), flush=True)

    def run(self, command, cwd=None, check=True, capture=False, env=None):
        if self.dry_run:
            self.show(command, cwd)
            return subprocess.CompletedProcess(command, 0, "", "")
        return subprocess.run(command, cwd=cwd, check=check, text=True, capture_output=capture, env=env)

    def note(self, message):
        print(("[dry run] " if self.dry_run else "") + message, flush=True)


def env(name):
    return os.environ.get(name) or None


def write_outputs(lines):
    """Append `key=value` lines to $GITHUB_OUTPUT, or print them without it."""
    path = env("GITHUB_OUTPUT")
    if path:
        with open(path, "a") as out:
            out.write("\n".join(lines) + "\n")
    print("\n".join(lines))


def flag(value):
    return "true" if value else "false"


# meta

def release_metadata(version, tag, editor_input, publish_integrations, event):
    """The values every job reads instead of repeating event and input expressions."""
    tag = tag or ""
    is_current = bool(tag) and tag == f"v{version}"
    editor_pinned = bool(editor_input)
    editor_version = editor_input or tag.removeprefix("v") or version
    editor_valid = EDITOR_VERSION.fullmatch(editor_version) is not None
    pushed = event == "push"
    integrations = publish_integrations == "true"
    return {
        "version": version,
        "tag": tag,
        "is_current_release": flag(is_current),
        "dry_run": flag(not tag),
        "publish_stores": flag(pushed or integrations),
        "publish_editor_stores": flag(is_current and (pushed or (integrations and editor_pinned))),
        "editor_version": editor_version,
        "editor_valid": flag(editor_valid),
        "editor_pinned": flag(editor_pinned),
    }


def step_meta(args, shell):
    values = release_metadata(workspace_version(ROOT), args.tag, args.editor_version, args.publish_integrations, args.event)
    lines = [f"{key}={value}" for key, value in values.items()]
    if shell.dry_run:
        shell.note("would append to $GITHUB_OUTPUT:")
        print("\n".join(lines))
    else:
        write_outputs(lines)


# notes

def changelog_section(changelog, version):
    """The text under `## [version]` up to the next `## [`; empty when there is none or it is blank."""
    version = version.removeprefix("v")
    lines, inside = [], False
    for line in changelog.splitlines():
        if line.startswith("## ["):
            if inside:
                break
            inside = line.startswith(f"## [{version}]")
            continue
        if inside:
            lines.append(line)
    return "\n".join(lines).strip("\n") + "\n" if "".join(lines).strip() else ""


def step_notes(args, shell):
    notes = changelog_section(args.changelog.read_text(encoding="utf-8"), args.version)
    if not notes:
        print(f"{args.changelog.name} has no section for {args.version.removeprefix('v')}", file=sys.stderr)
        raise SystemExit(1)
    sys.stdout.write(notes)


# published

def gh(*arguments):
    return subprocess.run(["gh", *arguments], capture_output=True, text=True)


def notice(title, message):
    print(f"::notice title={title}::{message}")
    summary = env("GITHUB_STEP_SUMMARY")
    if summary:
        with open(summary, "a") as out:
            out.write(message + "\n")


def published_release(repo, platform=None, run=gh, scratch=None):
    """(published, installable): `installable` is None unless `platform` names the archive to inspect."""
    latest = run("release", "view", "--repo", repo, "--json", "tagName", "-q", ".tagName")
    if latest.returncode != 0:
        if "release not found" in latest.stderr.lower():
            notice(
                "No published release",
                f"{repo} has no published release yet, so the steps that install a published release are skipped; "
                "the locally built archive steps still run.",
            )
            return False, None
        sys.stderr.write(latest.stderr)
        raise SystemExit(1)
    tag = latest.stdout.strip()
    print(f"Latest release: {tag}")
    assets = run("release", "view", tag, "--repo", repo, "--json", "assets", "-q", ".assets[].name")
    if assets.returncode != 0:
        sys.stderr.write(assets.stderr)
        raise SystemExit(1)
    # Releases cut before the archives were named jacket-<platform> cannot be installed by the current installers either.
    if not any(name.startswith("jacket-") for name in assets.stdout.split()):
        notice(
            "No installable release",
            f"{repo} release {tag} predates the jacket-<platform> archives, so the steps that install a published "
            "release are skipped; the locally built archive steps still run.",
        )
        return False, None
    if platform is None:
        return True, None
    # The formula is rendered from this checkout's template, so it can install a binary (such as `jacket`)
    # that the latest release's archive does not ship yet.
    archive = f"jacket-{platform}.tar.gz"
    directory = Path(scratch or tempfile.mkdtemp())
    download = run("release", "download", tag, "--repo", repo, "--pattern", archive, "--dir", str(directory))
    if download.returncode != 0:
        sys.stderr.write(download.stderr)
        raise SystemExit(1)
    with tarfile.open(directory / archive) as tar:
        names = tar.getnames()
    if f"jacket-{platform}/jacket" in names:
        return True, True
    notice(
        "Release older than the formula",
        f"{repo} release {tag} has no jacket binary, which the formula template installs, so the formula install "
        "and test are skipped until the next release.",
    )
    return True, False


def step_published(args, shell):
    if not args.repo:
        raise SystemExit("published: pass --repo or set GITHUB_REPOSITORY")
    platform = None
    if args.formula:
        key = f"{os.environ['RUNNER_OS']}-{os.environ['RUNNER_ARCH']}"
        if key not in PLATFORMS:
            raise SystemExit(f"no release archive for {key}")
        platform = PLATFORMS[key]
    if shell.dry_run:
        shell.show(["gh", "release", "view", "--repo", args.repo, "--json", "tagName", "-q", ".tagName"])
        shell.note("then the release's asset names, and with --formula the runner's archive (`gh release download`)")
        return
    published, installable = published_release(args.repo, platform, scratch=env("RUNNER_TEMP"))
    lines = [f"published={flag(published)}"]
    if installable is not None:
        lines.append(f"installable={flag(installable)}")
    write_outputs(lines)


# publish

def step_publish(args, shell):
    """Create the GitHub release for the tag from dist/ and notes.md; update it when it exists and is mutable."""
    tag, repo = args.tag, args.repo
    title = f"Jacket {tag.removeprefix('v')}"
    current = tag == f"v{args.current_version}"
    files = [str(path) for path in sorted(args.dist.glob("*"))] or [str(args.dist / "*")]
    view = ["gh", "release", "view", tag, "--repo", repo, "--json", "isImmutable"]
    upload = ["gh", "release", "upload", tag, *files, "--repo", repo, "--clobber"]
    edit = ["gh", "release", "edit", tag, "--repo", repo, "--title", title, "--notes-file", str(args.notes)]
    create = ["gh", "release", "create", tag, *files, "--repo", repo, "--title", title, "--notes-file", str(args.notes),
              "--latest" if current else "--latest=false", "--verify-tag"]
    if shell.dry_run:
        shell.show(view)
        shell.note("if the release exists and is immutable: fail; if it exists: upload (below), then edit; if not found: create")
        shell.show(upload)
        shell.show(edit + (["--latest"] if current else []))
        shell.show(create)
        return
    result = subprocess.run(view, capture_output=True, text=True)
    if result.returncode == 0:
        if json.loads(result.stdout)["isImmutable"] is True:
            print(f"GitHub release {tag} is immutable; its archives and checksums cannot be replaced safely.", file=sys.stderr)
            raise SystemExit(1)
        subprocess.run(upload, check=True)
        subprocess.run(edit + (["--latest"] if current else []), check=True)
    elif "release not found" in result.stderr.lower():
        subprocess.run(create, check=True)
    else:
        sys.stderr.write(result.stderr)
        raise SystemExit(1)


# editor stores

def packaged_vsix(directory):
    found = sorted(directory.glob("jacket-vscode-*.vsix"))
    if len(found) != 1:
        raise SystemExit(f"expected one jacket-vscode-*.vsix in {directory}, found {len(found)}")
    return found[0]


def step_marketplace(args, shell):
    token = env("VSCE_PAT")
    if not token:
        print("VSCE_PAT is not set; skipping the Marketplace")
        return
    vsix = args.vsix_dir / "jacket-vscode-*.vsix" if shell.dry_run else packaged_vsix(args.vsix_dir)
    # vsce reads the token from VSCE_PAT.
    shell.run(["pnpm", "exec", "vsce", "publish", "--packagePath", vsix, "--skip-duplicate"], cwd=ROOT / "editors/vscode")


def step_open_vsx(args, shell):
    token = env("OVSX_PAT")
    if not token:
        print("OVSX_PAT is not set; skipping Open VSX")
        return
    shell.secrets = [token]
    cwd = ROOT / "editors/vscode"
    vsix = args.vsix_dir / "jacket-vscode-*.vsix" if shell.dry_run else packaged_vsix(args.vsix_dir)
    # The namespace (package.json's publisher) must exist before the first publish. Creating
    # it again fails with "Namespace already exists", which is the state we want.
    created = shell.run(["pnpm", "exec", "ovsx", "create-namespace", NAMESPACE, "-p", token], cwd=cwd, check=False,
                        capture=True)
    if not shell.dry_run:
        output = created.stdout + created.stderr
        print(output, end="" if output.endswith("\n") else "\n")
        if created.returncode != 0 and "already exists" not in output.lower():
            raise SystemExit(1)
    shell.run(["pnpm", "exec", "ovsx", "publish", vsix, "-p", token, "--skip-duplicate"], cwd=cwd)


# homebrew

def step_homebrew(args, shell):
    if not args.tag:
        print("No tag (dry run); not pushing the formula")
        return
    token = env("TAP_TOKEN")
    if not token:
        print("HOMEBREW_TAP_TOKEN is not set; skipping the Homebrew tap")
        return
    shell.secrets = [token]
    tap = args.work / "tap"
    shell.run(["git", "clone", "--depth", "1", f"https://x-access-token:{token}@github.com/{TAP_REPOSITORY}.git", tap])
    if not shell.dry_run:
        (tap / "Formula").mkdir(exist_ok=True)
        shutil.copy2(args.formula, tap / "Formula" / "jacket.rb")
    else:
        shell.note(f"would copy {args.formula} to {tap}/Formula/jacket.rb")
    shell.run(["git", "add", "Formula/jacket.rb"], cwd=tap)
    if not shell.dry_run and subprocess.run(["git", "diff", "--cached", "--quiet"], cwd=tap).returncode == 0:
        print(f"Formula/jacket.rb is already at {args.version}")
        return
    shell.run(["git", *BOT, "commit", "-q", "-m", f"jacket {args.version}"], cwd=tap)
    shell.run(["git", "push", "origin", "HEAD"], cwd=tap)
    shell.note(f"Pushed jacket {args.version} to {TAP_REPOSITORY}")


# neovim

def step_nvim_stamp(args, shell):
    """The plugin downloads the compiler version; its mirror tag can advance separately."""
    lua = ROOT / "editors/nvim" / generate_version_files.OUTPUTS[1].removeprefix("editors/nvim/")
    stamped = generate_version_files.render(args.version)[generate_version_files.OUTPUTS[1]]
    if shell.dry_run:
        shell.note(f"would write {lua.relative_to(ROOT)} for {args.version}")
    else:
        lua.write_text(stamped)
        if f"return '{args.version}'\n" not in lua.read_text():
            raise SystemExit(f"{lua} does not say {args.version}")
    if args.tag:
        shell.run(["gh", "release", "download", args.tag, "--repo", args.repo, "-p", "toolchain-manifest.json", "-D", "."],
                  cwd=ROOT / "editors/nvim")
        if not shell.dry_run:
            print((ROOT / "editors/nvim/toolchain-manifest.json").read_text(), end="")


def step_nvim_mirror(args, shell):
    if not args.tag:
        print("No tag (dry run); not pushing the plugin")
        return
    deploy_key = env("DEPLOY_KEY")
    if not deploy_key:
        print("NVIM_MIRROR_DEPLOY_KEY is not set; skipping the Neovim plugin mirror")
        return
    shell.secrets = [deploy_key]
    key, known = args.work / "mirror_key", args.work / "known_hosts"
    mirror = args.work / "mirror"
    editor = args.editor_version
    message = f"jacket.nvim {editor} (toolchain {args.version})"
    tag = f"v{editor}"
    if shell.dry_run:
        shell.note(f"would write the deploy key to {key} (mode 600) and pin github.com's ed25519 host key to {GITHUB_HOST_KEY}")
    else:
        key.write_text(deploy_key + "\n")
        key.chmod(0o600)
        scan = subprocess.run(["ssh-keyscan", "-t", "ed25519", "github.com"], capture_output=True, text=True)
        known.write_text(scan.stdout)
        fingerprint = subprocess.run(["ssh-keygen", "-lf", str(known)], capture_output=True, text=True).stdout
        if GITHUB_HOST_KEY not in fingerprint:
            print("unexpected github.com host key")
            raise SystemExit(1)
    git_env = dict(os.environ, GIT_SSH_COMMAND=f"ssh -i {key} -o IdentitiesOnly=yes -o UserKnownHostsFile={known} -o StrictHostKeyChecking=yes")
    git = lambda *arguments, **options: shell.run(["git", *arguments], cwd=mirror, env=git_env, **options)  # noqa: E731
    shell.run(["git", "clone", f"git@github.com:{MIRROR_REPOSITORY}.git", mirror], env=git_env)
    git("checkout", "-B", "main")
    git("rm", "-rq", "--ignore-unmatch", ".")
    if shell.dry_run:
        shell.note(f"would copy editors/nvim, LICENSE and LICENSE-EXCEPTION into {mirror}")
    else:
        shutil.copytree(ROOT / "editors/nvim", mirror, dirs_exist_ok=True, symlinks=True)
        for name in ("LICENSE", "LICENSE-EXCEPTION"):
            shutil.copy2(ROOT / name, mirror)
    git("add", "-A")
    if not shell.dry_run and subprocess.run(["git", "diff", "--cached", "--quiet"], cwd=mirror, env=git_env).returncode == 0:
        print("The mirror already has this content")
    else:
        shell.run(["git", *BOT, "commit", "-q", "-m", message], cwd=mirror, env=git_env)
    exists = False
    if shell.dry_run:
        shell.note(f"if {tag} exists on the mirror it must point at this commit (else fail) and only main is pushed")
    else:
        listing = subprocess.run(["git", "ls-remote", "--exit-code", "--tags", "origin", f"refs/tags/{tag}"], cwd=mirror,
                                 env=git_env, capture_output=True)
        exists = listing.returncode == 0
    if exists:
        git("fetch", "-q", "origin", f"refs/tags/{tag}:refs/tags/{tag}")
        tagged = subprocess.run(["git", "rev-parse", f"refs/tags/{tag}^{{commit}}"], cwd=mirror, env=git_env,
                                capture_output=True, text=True).stdout.strip()
        current = subprocess.run(["git", "rev-parse", "HEAD"], cwd=mirror, env=git_env, capture_output=True, text=True).stdout.strip()
        if tagged != current:
            print(f"Mirror tag {tag} already points to different plugin content; publish a new editor_version instead of retagging.",
                  file=sys.stderr)
            raise SystemExit(1)
        print(f"The mirror tag {tag} already points to this content")
        git("push", "origin", "main")
    else:
        shell.run(["git", *BOT, "tag", "-a", tag, "-m", message], cwd=mirror, env=git_env)
        git("push", "origin", "main", tag)
    shell.note(f"Pushed jacket.nvim {editor} for toolchain {args.version} to {MIRROR_REPOSITORY}")


# portfolio

def step_portfolio(args, shell):
    """The last step: the portfolio builds its site against the new tag (needs GH_TOKEN with workflow access to it)."""
    if not args.tag:
        print("No tag (dry run); not dispatching the portfolio")
        return
    if not re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+", args.tag):
        raise SystemExit(f"portfolio: expected a tag like v1.2.3, got {args.tag}")
    shell.run(["gh", "workflow", "run", PORTFOLIO_WORKFLOW, "-R", PORTFOLIO_REPOSITORY, "-f", f"jai_ref={args.tag}"])


# command line

STEP_HELP = {}

def build_parser():
    STEP_HELP.clear()
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--list", action="store_true", help="list the steps")
    steps = parser.add_subparsers(dest="step", metavar="step")

    def step(name, function, help_text, **defaults):
        STEP_HELP[name] = help_text
        sub = steps.add_parser(name, help=help_text, description=help_text)
        sub.add_argument("--dry-run", action="store_true", help="print the commands instead of running them")
        sub.set_defaults(function=function, **defaults)
        return sub

    def option(sub, name, variable, default=None, **keywords):
        sub.add_argument(name, default=env(variable) if variable else default, **keywords)

    sub = step("meta", step_meta, "compute the release metadata")
    option(sub, "--tag", "TAG", help="the selected compiler tag; empty for a run without one (a dry run)")
    option(sub, "--editor-version", "EDITOR_INPUT", help="an explicit VS Code and Neovim plugin version")
    option(sub, "--publish-integrations", "PUBLISH_INTEGRATIONS", help="`true` to publish the external integrations")
    option(sub, "--event", "EVENT", help="the GitHub event name (`push` for a tag push)")

    sub = step("notes", step_notes, "print the CHANGELOG.md section of a version (exit 1 when there is none or it is empty)")
    sub.add_argument("version", help="the version or tag, with or without the leading v")
    sub.add_argument("--changelog", type=Path, default=ROOT / "CHANGELOG.md")

    sub = step("published", step_published, "is a release published that the installer jobs can install")
    option(sub, "--repo", "GITHUB_REPOSITORY", help="owner/name")
    sub.add_argument("--formula", action="store_true", help="also check that this runner's archive ships `jacket` (RUNNER_OS, RUNNER_ARCH)")

    sub = step("publish", step_publish, "create or update the GitHub release from dist/ and notes.md")
    option(sub, "--tag", "TAG")
    option(sub, "--current-version", "CURRENT_VERSION", help="the workflow revision's version; its own tag is marked latest")
    option(sub, "--repo", "GITHUB_REPOSITORY")
    sub.add_argument("--dist", type=Path, default=Path("dist"))
    sub.add_argument("--notes", type=Path, default=Path("notes.md"))

    for name, function, text in (("marketplace", step_marketplace, "publish the packaged VSIX to the VS Code Marketplace (VSCE_PAT)"),
                                 ("open-vsx", step_open_vsx, "publish the packaged VSIX to Open VSX (OVSX_PAT)")):
        sub = step(name, function, text)
        sub.add_argument("--vsix-dir", type=Path, default=ROOT / "vsix")

    runner_temp = Path(env("RUNNER_TEMP") or tempfile.gettempdir())
    sub = step("homebrew", step_homebrew, "push the rendered formula to the Homebrew tap (TAP_TOKEN)")
    option(sub, "--tag", "TAG")
    option(sub, "--version", "VERSION")
    sub.add_argument("--formula", type=Path, default=runner_temp / "packages/homebrew/jacket.rb")
    sub.add_argument("--work", type=Path, default=runner_temp)

    sub = step("nvim-stamp", step_nvim_stamp, "stamp the version and toolchain manifest into editors/nvim")
    option(sub, "--tag", "TAG")
    option(sub, "--version", "VERSION")
    option(sub, "--repo", "GITHUB_REPOSITORY")

    sub = step("nvim-mirror", step_nvim_mirror, "mirror editors/nvim to matteopolak/jacket.nvim and tag it (DEPLOY_KEY)")
    option(sub, "--tag", "TAG")
    option(sub, "--version", "VERSION")
    option(sub, "--editor-version", "EDITOR_VERSION")
    sub.add_argument("--work", type=Path, default=runner_temp)

    sub = step("portfolio", step_portfolio, "dispatch the portfolio's jai-web.yml for the tag")
    option(sub, "--tag", "TAG")

    sub = step("all", lambda args, shell: step_all(args), "run the publishing steps in release order")
    option(sub, "--tag", "TAG")
    option(sub, "--version", "VERSION")
    option(sub, "--editor-version", "EDITOR_VERSION")
    option(sub, "--current-version", "CURRENT_VERSION")
    option(sub, "--repo", "GITHUB_REPOSITORY")
    return parser


# The options `all` hands to each publishing step (the others take none).
ALL_OPTIONS = {
    "publish": ("tag", "current_version", "repo"),
    "homebrew": ("tag", "version"),
    "nvim-stamp": ("tag", "version", "repo"),
    "nvim-mirror": ("tag", "version", "editor_version"),
    "portfolio": ("tag",),
}


def step_all(args):
    """Each publishing step in order, with the options `all` was given."""
    for name in PUBLISHING:
        argv = [name]
        for key in ALL_OPTIONS.get(name, ()):
            if getattr(args, key) is not None:
                argv += [f"--{key.replace('_', '-')}", getattr(args, key)]
        if args.dry_run:
            argv.append("--dry-run")
        print(f"== {name}", flush=True)
        main(argv)


def main(argv=None):
    parser = build_parser()
    args = parser.parse_args(argv)
    if args.list or not args.step:
        for name, text in STEP_HELP.items():
            print(f"{name:12} {text}")
        return 0
    args.function(args, Shell(args.dry_run))
    return 0


if __name__ == "__main__":
    sys.exit(main())
