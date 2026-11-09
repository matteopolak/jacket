#!/usr/bin/env python3
"""Check the SHA-pinned GitHub Actions in `.github/workflows` and `.github/actions` and update them.

A pin reads `uses: owner/repo[/path]@<40-hex commit> # vX.Y.Z`. For each one the script checks that

  * the commit is the one the commented tag points to (annotated tags are followed to their commit),
  * that release is at least 14 days old (`MIN_AGE`, shared with `check_dependency_age.py`),
  * the pin carries a version comment and is not an unpinned ref like `@v4`,

and reports a newer stable release when there is one. Only releases at least 14 days old are offered.
A newer major version is reported as such. Pre-releases and drafts are ignored. Only the newest 100
releases of a repository are read.

Usage (from the repository root, with `tools/` on `PYTHONPATH`):

    python3 tools/ci/check_action_pins.py             # policy and updates
    python3 tools/ci/check_action_pins.py --policy    # policy only; what CI runs
    python3 tools/ci/check_action_pins.py --update    # rewrite out-of-date pins to eligible releases

Exit status: 0 up to date, 1 policy errors, 2 updates available (nothing is wrong), 3 GitHub could not
be reached or answered unexpectedly. Policy errors win over updates. `--update` returns 0 once the
updates are written.

Set `GITHUB_TOKEN` or `GH_TOKEN` to raise GitHub's rate limit (60 requests an hour without one).
API answers are cached in `target/action-pins-cache.json` for six hours; `--refresh` ignores the cache.
"""

import argparse
import re
import sys
import urllib.error
import urllib.parse
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone

from repo_output import output_dir
from toolkit.age_policy import MIN_AGE, load_cache, parse_time, recent, save_cache
from toolkit import github
from toolkit.github import LookupFailed, NotFound
from toolkit.repo import ROOT

CACHE_TTL = timedelta(hours=6)
USES = re.compile(
    r"""(?P<head>\buses:\s*)(?P<ref>[^\s#'"]+)(?P<tail>[ \t]*(?:\#[ \t]*(?P<tag>\S+))?)"""
)
SHA = re.compile(r"[0-9a-f]{40}")
STABLE = re.compile(r"v?(\d+)(?:\.(\d+))?(?:\.(\d+))?")
OK, POLICY, UPDATES, UNREACHABLE = 0, 1, 2, 3


@dataclass(frozen=True)
class Pin:
    path: str
    line: int
    repo: str  # owner/name, without any sub-path
    action: str  # as written: owner/name[/sub/path]
    sha: str
    tag: str | None


@dataclass(frozen=True)
class Update:
    repo: str
    current: str
    tag: str
    sha: str
    major: bool


def pin_files(root=ROOT):
    return sorted(
        [
            *(root / ".github/workflows").glob("*.y*ml"),
            *(root / ".github/actions").glob("*/action.y*ml"),
        ]
    )


def find_pins(name, text):
    """`(pins, errors)` for the remote `uses:` lines of one file; local and `docker://` uses are not pins."""
    pins, errors = [], []
    for number, line in enumerate(text.splitlines(), 1):
        if line.lstrip().startswith("#"):
            continue
        match = USES.search(line)
        if not match or match["ref"].startswith(("./", "docker://")):
            continue
        action, _, ref = match["ref"].partition("@")
        if not SHA.fullmatch(ref):
            errors.append(
                f"{name}:{number}: {match['ref']} is not pinned to a commit SHA"
            )
            continue
        owner, repo, *_ = action.split("/")
        pins.append(Pin(name, number, f"{owner}/{repo}", action, ref, match["tag"]))
    return pins, errors


def version(tag):
    """The comparable version of a stable tag such as `v1.2.3` or `v4`, or `None` for anything else."""
    match = STABLE.fullmatch(tag or "")
    return tuple(int(part or 0) for part in match.groups()) if match else None


class GitHub:
    """GitHub's REST API; answers are kept in a JSON cache file for `CACHE_TTL`."""

    def __init__(self, cache_path, now, refresh=False):
        self.cache_path, self.now, self.refresh = cache_path, now, refresh
        self.cache = {} if cache_path is None else load_cache(cache_path)
        self.stored = dict(self.cache)

    def __call__(self, path):
        entry = self.cache.get(path)
        if (
            not self.refresh
            and isinstance(entry, dict)
            and "body" in entry
            and recent(entry.get("fetched"), self.now, CACHE_TTL)
        ):
            return entry["body"]
        body = github.get(path)
        self.cache[path] = {"fetched": self.now.isoformat(), "body": body}
        return body

    def save(self):
        if self.cache_path is not None and self.cache != self.stored:
            save_cache(self.cache_path, self.cache)


def tag_commit(api, repo, tag):
    """The commit tag `tag` points to, following annotated tags; `None` when the tag does not exist."""
    try:
        target = api(f"/repos/{repo}/git/ref/tags/{urllib.parse.quote(tag, safe='')}")[
            "object"
        ]
        while target["type"] == "tag":
            target = api(f"/repos/{repo}/git/tags/{target['sha']}")["object"]
    except NotFound:
        return None
    return target["sha"] if target["type"] == "commit" else None


def released(api, repo, tag, sha, releases):
    """When `tag` was released: its GitHub release's publish time, else its commit's committer time."""
    for release in releases:
        if release["tag_name"] == tag and release.get("published_at"):
            return parse_time(release["published_at"], "GitHub")
    return parse_time(
        api(f"/repos/{repo}/commits/{sha}")["commit"]["committer"]["date"], "GitHub"
    )


def newest_eligible(api, repo, releases, now):
    """`(tag, sha)` of the newest stable release at least `MIN_AGE` old, or `None`."""
    candidates = sorted(
        (
            r
            for r in releases
            if not r.get("draft")
            and not r.get("prerelease")
            and version(r["tag_name"])
            and r.get("published_at")
        ),
        key=lambda r: version(r["tag_name"]),
        reverse=True,
    )
    for release in candidates:
        if parse_time(release["published_at"], "GitHub") > now - MIN_AGE:
            continue
        sha = tag_commit(api, repo, release["tag_name"])
        if sha:
            return release["tag_name"], sha
    return None


def check_repo(api, repo, pins, now, policy_only=False):
    """`(errors, updates)` for the pins of one repository."""
    errors, updates = [], []
    releases = None
    for pin in pins:
        where = f"{pin.path}:{pin.line}: {pin.action}@{pin.sha[:12]}"
        if not pin.tag:
            errors.append(f"{where} has no version comment (`# vX.Y.Z`)")
            continue
        if releases is None:
            releases = api(f"/repos/{repo}/releases?per_page=100")
        actual = tag_commit(api, repo, pin.tag)
        if actual is None:
            errors.append(f"{where}: tag {pin.tag} does not exist in {repo}")
        elif actual != pin.sha:
            errors.append(
                f"{where}: tag {pin.tag} points to {actual[:12]}, not the pinned commit"
            )
        else:
            made = released(api, repo, pin.tag, pin.sha, releases)
            if made > now - MIN_AGE:
                errors.append(
                    f"{where} # {pin.tag}: released {made.date()}, younger than 14 days"
                )
    if policy_only or releases is None:
        return errors, updates
    best = newest_eligible(api, repo, releases, now)
    if best:
        tag, sha = best
        for current in sorted(
            {p.tag for p in pins if p.tag}, key=lambda t: version(t) or ()
        ):
            old = version(current)
            if old is not None and version(tag) > old:
                updates.append(
                    Update(repo, current, tag, sha, version(tag)[0] > old[0])
                )
    return errors, updates


def check(api, pins, now, policy_only=False):
    """`(errors, updates)` over every pin."""
    by_repo = {}
    for pin in pins:
        by_repo.setdefault(pin.repo, []).append(pin)
    errors, updates = [], []
    for repo, group in sorted(by_repo.items()):
        found = check_repo(api, repo, group, now, policy_only)
        errors += found[0]
        updates += found[1]
    return errors, updates


def rewrite(text, updates):
    """`text` with each outdated pin's SHA and `# tag` comment replaced, and the number of pins changed."""
    changed = 0

    def swap(match):
        nonlocal changed
        action, _, ref = match["ref"].partition("@")
        owner, repo, *_ = action.split("/")
        for update in updates:
            if update.repo == f"{owner}/{repo}" and match["tag"] == update.current:
                changed += 1
                tail = match["tail"].replace(match["tag"], update.tag, 1)
                return f"{match['head']}{action}@{update.sha}{tail}"
        return match[0]

    return USES.sub(swap, text), changed


def describe(update):
    kind = " (new major version)" if update.major else ""
    return f"{update.repo}: {update.current} -> {update.tag} ({update.sha[:12]}){kind}"


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument(
        "--policy",
        action="store_true",
        help="check only that pins match their tags and are old enough",
    )
    parser.add_argument(
        "--update",
        action="store_true",
        help="rewrite out-of-date pins to the newest release at least 14 days old",
    )
    parser.add_argument(
        "--refresh", action="store_true", help="ignore cached API answers"
    )
    parser.add_argument(
        "--cache",
        default=output_dir("action-pins-cache.json"),
        help="API answer cache (default: target/action-pins-cache.json)",
    )
    args = parser.parse_args(argv)
    if args.policy and args.update:
        parser.error("--update needs the release lookups that --policy skips")
    now = datetime.now(timezone.utc)
    files = {path: path.read_text() for path in pin_files()}
    pins, errors = [], []
    for path, text in files.items():
        found = find_pins(path.relative_to(ROOT).as_posix(), text)
        pins += found[0]
        errors += found[1]
    api = GitHub(args.cache, now, args.refresh)
    try:
        more, updates = check(api, pins, now, args.policy)
    except LookupFailed as error:
        print(
            f"check_action_pins: {error}\nFix the network or skip the pre-push check with JAI_HOOK_SKIP=actions.",
            file=sys.stderr,
        )
        return UNREACHABLE
    finally:
        api.save()
    errors += more
    for error in errors:
        print(f"error: {error}", file=sys.stderr)
    if errors:
        return POLICY
    if args.update and updates:
        for path, text in files.items():
            new, changed = rewrite(text, updates)
            if changed:
                path.write_text(new)
                print(
                    f"updated {changed} pin(s) in {path.relative_to(ROOT).as_posix()}"
                )
        for update in updates:
            print(f"  {describe(update)}")
        return OK
    for update in updates:
        print(f"update available: {describe(update)}")
    if updates:
        print(
            "Apply with: PYTHONPATH=tools python3 tools/ci/check_action_pins.py --update"
        )
        return UPDATES
    print(f"Action pins passed ({len(pins)} pins)")
    return OK


if __name__ == "__main__":
    sys.exit(main())
