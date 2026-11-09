#!/usr/bin/env python3
"""Post a release to Zulip (.github/workflows/zulip-release.yml, docs/tools/community-chat.md).

Usage:
  announce_release.py --tag vX.Y.Z [--channel announcements] [--topic Releases] [--dry-run]

Reads the release with `gh api` (GH_TOKEN, GITHUB_REPOSITORY) and posts its notes and download
links as one message. Zulip credentials come from ZULIP_SITE, ZULIP_BOT_EMAIL and ZULIP_BOT_KEY.
`--dry-run` prints the message instead. A different `--channel` previews a past release.
"""

import argparse
import base64
import json
import os
import re
import subprocess
import sys
import urllib.parse
import urllib.request

# Zulip rejects messages over 10000 characters.
LIMIT = 9000


def release(tag: str) -> dict:
    repo = os.environ["GITHUB_REPOSITORY"]
    out = subprocess.run(
        ["gh", "api", f"repos/{repo}/releases/tags/{tag}"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    return json.loads(out)


def size(n: int) -> str:
    for unit in ("B", "KiB", "MiB", "GiB"):
        if n < 1024 or unit == "GiB":
            return f"{n:.0f} {unit}" if unit == "B" else f"{n:.1f} {unit}"
        n /= 1024
    raise AssertionError


def absolute_links(body: str, repo: str, tag: str) -> str:
    """Point `](docs/x.md)` links at that file in the tagged tree; Zulip cannot resolve repository paths."""
    base = f"https://github.com/{repo}/blob/{tag}/"
    return re.sub(r"\]\((?!https?:|mailto:|#|/)([^)\s]+)\)", lambda m: f"]({base}{m.group(1)})", body)


def message(rel: dict) -> str:
    title = rel["name"] or rel["tag_name"]
    parts = [f"**[{title}]({rel['html_url']})** is out."]
    body = absolute_links((rel.get("body") or "").strip(), os.environ["GITHUB_REPOSITORY"], rel["tag_name"])
    if body:
        if len(body) > LIMIT:
            body = body[:LIMIT].rsplit("\n", 1)[0] + f"\n\n[Read the rest]({rel['html_url']})"
        parts.append(body)
    assets = sorted(rel.get("assets", []), key=lambda a: a["name"])
    if assets:
        rows = "\n".join(
            f"- [{a['name']}]({a['browser_download_url']}) ({size(a['size'])})" for a in assets
        )
        parts.append(f"```spoiler Downloads\n{rows}\n```")
    return "\n\n".join(parts)


def post(channel: str, topic: str, content: str) -> None:
    site = os.environ["ZULIP_SITE"].rstrip("/")
    auth = base64.b64encode(
        f"{os.environ['ZULIP_BOT_EMAIL']}:{os.environ['ZULIP_BOT_KEY']}".encode()
    ).decode()
    data = urllib.parse.urlencode(
        {"type": "channel", "to": channel, "topic": topic, "content": content}
    ).encode()
    request = urllib.request.Request(
        f"{site}/api/v1/messages", data, {"Authorization": f"Basic {auth}"}
    )
    with urllib.request.urlopen(request) as response:
        result = json.load(response)
    if result.get("result") != "success":
        sys.exit(f"Zulip refused the message: {result}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--tag", required=True)
    parser.add_argument("--channel", default="announcements")
    parser.add_argument("--topic", default="Releases")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    content = message(release(args.tag))
    if args.dry_run:
        print(content)
    else:
        post(args.channel, args.topic, content)


if __name__ == "__main__":
    main()
