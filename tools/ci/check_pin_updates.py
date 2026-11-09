#!/usr/bin/env python3
"""Compare the tool pins with the latest upstream stable release and keep one issue about the ones behind.

Checked pins (docs/tools/pin-updates.md):

  rust    rust-toolchain.toml channel, against the stable channel manifest on static.rust-lang.org
  llvm    `llvm` in .github/pins.json, against the llvm/llvm-project releases
  just    `just`, against the casey/just releases
  nextest `nextest`, against the nextest-rs/nextest `cargo-nextest-*` releases

Usage (from the repository root, with `tools/` on `PYTHONPATH`):

    python3 tools/ci/check_pin_updates.py --dry-run    # print the table and the issue it would write
    python3 tools/ci/check_pin_updates.py              # update the issue (needs `gh`, GH_TOKEN, GITHUB_REPOSITORY)

The issue is found by a hidden marker in its body. It is created when a pin is behind, edited in place
when the list changes (not at all when it is the same), reopened if it was closed, and closed when every
pin is current. Only releases at least 14 days old count, the rule `check_action_pins.py` uses.

The Rust pin is a nightly, so it is compared by date: it is behind when it predates the stable release.

Exit status: 0 done, 3 an upstream answer could not be read. Set `GITHUB_TOKEN` or `GH_TOKEN` to raise
GitHub's rate limit.
"""

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tomllib
import urllib.error
from dataclasses import dataclass
from datetime import date, datetime, timezone
from http.client import HTTPException

from toolkit import github
from toolkit.age_policy import MIN_AGE, parse_time
from toolkit.download import fetch
from toolkit.github import LookupFailed
from toolkit.repo import ROOT

TITLE = "Tool pins are out of date"
MARKER = "<!-- check_pin_updates -->"
RUST_MANIFEST = "https://static.rust-lang.org/dist/channel-rust-stable.toml"
NIGHTLY = re.compile(r"nightly-(\d{4}-\d{2}-\d{2})")
# pins.json key -> (upstream repository, release tag prefix, name shown, newest releases read). LLVM lists its
# release candidates too; just and nextest have long release notes, so a small page keeps the answer short.
RELEASES = {
    "llvm": ("llvm/llvm-project", "llvmorg-", "llvm", 30),
    "just": ("casey/just", "", "just", 10),
    "nextest": ("nextest-rs/nextest", "cargo-nextest-", "nextest", 20),
}


@dataclass(frozen=True)
class Row:
    name: str
    pinned: str
    latest: str
    where: str


def parse_version(text: str) -> tuple[int, ...]:
    """`1.58.0` as (1, 58, 0); `ValueError` for anything that is not dotted numbers."""
    if not re.fullmatch(r"\d+(\.\d+)*", text):
        raise ValueError(f"not a release version: {text!r}")
    return tuple(int(part) for part in text.split("."))


def newer(latest: str, pinned: str) -> bool:
    """Whether `latest` is a later version than `pinned`; compares numbers, so 0.9.100 is after 0.9.99."""
    return parse_version(latest) > parse_version(pinned)


def latest_release(releases: list, prefix: str, now: datetime) -> str | None:
    """The highest version among the stable releases whose tag is `prefix` + dotted numbers and that are
    at least `MIN_AGE` old; None when there is none."""
    best = None
    for release in releases:
        tag = release.get("tag_name", "")
        if release.get("draft") or release.get("prerelease") or not tag.startswith(prefix):
            continue
        version = tag[len(prefix):]
        try:
            parse_version(version)
            published = parse_time(release["published_at"], "GitHub")
        except (ValueError, KeyError, TypeError):
            continue
        if now - published >= MIN_AGE and (best is None or newer(version, best)):
            best = version
    return best


def rust_pin(toolchain_text: str) -> str:
    """The `channel` of rust-toolchain.toml."""
    return tomllib.loads(toolchain_text)["toolchain"]["channel"]


def rust_row(channel: str, manifest: dict, now: datetime) -> Row | None:
    """A row when the pinned channel predates the stable release in `manifest` (a nightly by its date,
    a version by number); None when current or when the stable release is younger than `MIN_AGE`."""
    stable = manifest["pkg"]["rust"]["version"].split()[0]
    released = date.fromisoformat(manifest["date"])
    if now.date() - released < MIN_AGE:
        return None
    where = "rust-toolchain.toml"
    nightly = NIGHTLY.fullmatch(channel)
    if nightly:
        if date.fromisoformat(nightly.group(1)) >= released:
            return None
        return Row("rust", channel, f"stable {stable} ({released.isoformat()})", where)
    if newer(stable, channel):
        return Row("rust", channel, stable, where)
    return None


def outdated(pins: dict, toolchain_text: str, manifest: dict, releases: dict, now: datetime) -> list[Row]:
    """The pins behind upstream. `releases` maps a pins.json key to that repository's release list."""
    rows = []
    rust = rust_row(rust_pin(toolchain_text), manifest, now)
    if rust:
        rows.append(rust)
    for key, (_, prefix, name, count) in RELEASES.items():
        latest = latest_release(releases[key], prefix, now)
        if latest is None:
            raise LookupFailed(f"no stable {name} release with tag {prefix}X.Y.Z in the newest {count}")
        if newer(latest, pins[key]):
            rows.append(Row(name, pins[key], latest, ".github/pins.json"))
    return rows


def issue_body(rows: list[Row]) -> str:
    """The issue text; the same rows give the same text, so an unchanged list is never edited."""
    lines = [MARKER, "These pins are behind the latest stable upstream release:", "",
             "| Pin | Pinned | Latest | Edit |", "| --- | --- | --- | --- |"]
    lines += [f"| {r.name} | `{r.pinned}` | {r.latest} | `{r.where}` |" for r in rows]
    lines += ["", "See docs/tools/pin-updates.md for how to move a pin. This issue is kept up to date by the "
              "`pin updates` workflow and closes itself when every pin is current."]
    return "\n".join(lines) + "\n"


def plan(existing: dict | None, rows: list[Row]) -> tuple[str, str]:
    """(action, body): create, edit, reopen, close or nothing, given the issue found (number, state, body)."""
    body = issue_body(rows)
    if not rows:
        return ("close" if existing and existing["state"] == "OPEN" else "nothing"), body
    if existing is None:
        return "create", body
    if existing["state"] != "OPEN":
        return "reopen", body
    return ("nothing" if existing["body"] == body else "edit"), body


def gather(now: datetime) -> list[Row]:
    pins = json.loads((ROOT / ".github" / "pins.json").read_text())
    toolchain = (ROOT / "rust-toolchain.toml").read_text()
    try:
        manifest = tomllib.loads(fetch(RUST_MANIFEST, max_bytes=64 << 20).decode())
    except (OSError, HTTPException, ValueError, urllib.error.URLError) as error:
        raise LookupFailed(f"could not read {RUST_MANIFEST}: {error}") from error
    releases = {key: github.get(f"/repos/{repo}/releases?per_page={count}") for key, (repo, _, _, count) in RELEASES.items()}
    return outdated(pins, toolchain, manifest, releases, now)


def gh(*args: str, stdin: str | None = None) -> str:
    result = subprocess.run(["gh", *args], input=stdin, capture_output=True, text=True)
    if result.returncode:
        raise SystemExit(f"gh {' '.join(args[:2])} failed: {result.stderr.strip()}")
    return result.stdout


def find_issue(repo: str) -> dict | None:
    found = json.loads(gh("issue", "list", "--repo", repo, "--state", "all", "--limit", "30",
                          "--search", f'"{TITLE}" in:title', "--json", "number,state,title,body"))
    ours = [i for i in found if i["title"] == TITLE and MARKER in i["body"]]
    # Open first, then the most recently numbered closed one.
    return max(ours, key=lambda i: (i["state"] == "OPEN", i["number"]), default=None)


def apply(repo: str, action: str, body: str, existing: dict | None) -> str:
    if action == "create":
        return gh("issue", "create", "--repo", repo, "--title", TITLE, "--body-file", "-", stdin=body).strip()
    number = str(existing["number"]) if existing else ""
    if action == "reopen":
        gh("issue", "reopen", number, "--repo", repo)
    if action in ("edit", "reopen"):
        gh("issue", "edit", number, "--repo", repo, "--body-file", "-", stdin=body)
    if action == "close":
        gh("issue", "close", number, "--repo", repo, "--comment", "Every pin is current.")
    return f"{action} #{number}" if number else action


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--dry-run", action="store_true", help="print the result; do not touch issues or need gh")
    args = parser.parse_args()
    if not args.dry_run:
        if shutil.which("gh") is None:
            raise SystemExit("gh not found: install the GitHub CLI (https://cli.github.com), or use --dry-run")
        repo = os.environ.get("GITHUB_REPOSITORY")
        if not repo:
            raise SystemExit("GITHUB_REPOSITORY is not set (owner/name); use --dry-run to only print")
    try:
        rows = gather(datetime.now(timezone.utc))
    except LookupFailed as error:
        print(error, file=sys.stderr)
        return 3
    for row in rows:
        print(f"{row.name}: pinned {row.pinned}, latest {row.latest}")
    if not rows:
        print("every pin is current")
    if args.dry_run:
        print("\n--- issue body ---" if rows else "\nthe issue would be closed if open")
        if rows:
            print(issue_body(rows), end="")
        return 0
    existing = find_issue(repo)
    action, body = plan(existing, rows)
    print(apply(repo, action, body, existing) if action != "nothing" else "issue is up to date")
    return 0


if __name__ == "__main__":
    sys.exit(main())
