#!/usr/bin/env python3
"""Which upstream sweep cases run on a platform, and which repositories they need.

CI jobs for platform-specific cases (`windows-smoke`, the x64 step of `upstream-smoke`) select from
tools/upstream-cases.json instead of listing ids by hand: a case that names the platform in `only`
and is not `disabled` is selected, so a new Windows-only case needs no workflow edit.

    select_upstream_cases.py --platform windows-x64 --ids     # one id per line
    select_upstream_cases.py --platform windows-x64 --repos   # owner/repo, for fetch_upstreams.py --only
"""
import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CASES = ROOT / "tools/upstream-cases.json"


def load(path: Path = CASES) -> list[dict]:
    return json.loads(path.read_text())


def select(cases: list[dict], platform: str) -> list[dict]:
    """Enabled cases restricted to (among others) `platform`."""
    return [c for c in cases if "disabled" not in c and platform in c.get("only", [])]


def directories(case: dict) -> list[str]:
    """The corpus directories a case reads: its project, copies and links, in first-seen order."""
    names = [case["path"].split("/")[0], *case.get("copy", []), *case.get("link", [])]
    return list(dict.fromkeys(names))


def repositories(cases: list[dict]) -> list[str]:
    """`owner/repo` of every corpus directory the cases use (directories are `owner--repo`)."""
    return sorted({name.replace("--", "/", 1) for case in cases for name in directories(case)})


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--platform", required=True)
    what = ap.add_mutually_exclusive_group(required=True)
    what.add_argument("--ids", action="store_true")
    what.add_argument("--repos", action="store_true")
    ap.add_argument("--cases", type=Path, default=CASES)
    args = ap.parse_args()
    chosen = select(load(args.cases), args.platform)
    if not chosen:
        print(f"no enabled case is restricted to {args.platform}", file=sys.stderr)
        return 1
    print("\n".join(c["id"] for c in chosen) if args.ids else " ".join(repositories(chosen)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
