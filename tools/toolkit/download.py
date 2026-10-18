"""Downloads pinned by SHA-256, cached under `$XDG_CACHE_HOME/jaic/` (default `~/.cache/jaic/`)."""

import os
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

from toolkit.hashing import sha256_file

__all__ = ["cache_dir", "pinned"]

ATTEMPTS = 5
TRANSIENT = {408, 429, 500, 502, 503, 504}


def cache_dir() -> Path:
    base = os.environ.get("XDG_CACHE_HOME") or str(Path.home() / ".cache")
    return Path(base) / "jaic"


def pinned(url: str, sha256: str, name: str, *, timeout: float = 120) -> Path:
    """The cached file `name`, downloaded from `url` first if missing; exits if its digest is not `sha256`.

    Rate limits, server errors and dropped connections are retried with backoff (honouring
    `Retry-After`), so a CI check does not fail on one busy response.
    """
    path = cache_dir() / name
    if not path.exists():
        path.parent.mkdir(parents=True, exist_ok=True)
        partial = path.with_name(path.name + ".part")
        partial.write_bytes(_fetch(url, timeout))
        partial.replace(path)
    digest = sha256_file(path)
    if digest != sha256:
        path.unlink()
        raise SystemExit(f"{url}: SHA-256 {digest} does not match the pinned {sha256}")
    return path


def _fetch(url: str, timeout: float) -> bytes:
    print(f"downloading {url}", file=sys.stderr)
    for attempt in range(1, ATTEMPTS + 1):
        try:
            with urllib.request.urlopen(url, timeout=timeout) as response:
                return response.read()
        except urllib.error.HTTPError as error:
            if error.code not in TRANSIENT or attempt == ATTEMPTS:
                raise
            wait = _retry_after(error) or 2**attempt
        except (urllib.error.URLError, TimeoutError, ConnectionError):
            if attempt == ATTEMPTS:
                raise
            wait = 2**attempt
        print(f"  attempt {attempt} failed; retrying in {wait}s", file=sys.stderr)
        time.sleep(wait)
    raise AssertionError("unreachable")


def _retry_after(error: urllib.error.HTTPError) -> int | None:
    value = error.headers.get("Retry-After") if error.headers else None
    if value and value.isdigit():
        return min(int(value), 60)
    return None
