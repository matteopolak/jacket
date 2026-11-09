"""The 14-day minimum age for third-party code, and the JSON answer cache its checks share.

Used by `tools/ci/check_dependency_age.py` (cargo) and `tools/ci/check_action_pins.py` (GitHub Actions).
"""

import json
from datetime import datetime, timedelta
from pathlib import Path

__all__ = ["MIN_AGE", "load_cache", "parse_time", "recent", "save_cache"]

MIN_AGE = timedelta(days=14)


def parse_time(text, what):
    """The timezone-aware time in ISO 8601 `text`; `ValueError` naming `what` when it has no timezone."""
    stamp = datetime.fromisoformat(text.replace("Z", "+00:00"))
    if stamp.tzinfo is None:
        raise ValueError(f"{what} returned a timestamp without a timezone")
    return stamp


def recent(fetched, now, ttl):
    """Whether a cache entry stamped `fetched` (ISO text) is younger than `ttl` and not from the future."""
    try:
        stamp = parse_time(fetched, "cache")
    except (TypeError, ValueError):
        return False
    return stamp <= now and now - stamp < ttl


def load_cache(path):
    """The JSON object in `path`; empty when the file is missing or unreadable."""
    try:
        data = json.loads(Path(path).read_text())
        return data if isinstance(data, dict) else {}
    except (OSError, ValueError):
        return {}


def save_cache(path, cache):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(cache, sort_keys=True))
