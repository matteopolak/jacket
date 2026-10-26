"""Downloads pinned by SHA-256, cached under `$XDG_CACHE_HOME/jaic/` (default `~/.cache/jaic/`)."""

import os
import sys
import time
import urllib.error
import urllib.request
from collections.abc import Mapping
from http.client import HTTPException, IncompleteRead
from pathlib import Path


from toolkit.hashing import sha256_file

__all__ = ["cache_dir", "fetch", "pinned"]

ATTEMPTS = 6
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
        print(f"downloading {url}", file=sys.stderr)
        partial.write_bytes(fetch(url, timeout=timeout))
        partial.replace(path)
    digest = sha256_file(path)
    if digest != sha256:
        path.unlink()
        raise SystemExit(f"{url}: SHA-256 {digest} does not match the pinned {sha256}")
    return path


def fetch(
    url: str,
    *,
    timeout: float = 120,
    attempts: int = ATTEMPTS,
    headers: Mapping[str, str] | None = None,
    max_bytes: int | None = None,
) -> bytes:
    """The body of `url` (`ValueError` when it is longer than `max_bytes`, which is never read past). Rate limits, 5xx answers, dropped connections and truncated bodies are retried
    with a growing pause (honouring `Retry-After`); other 4xx answers (a wrong pin) fail at once."""
    request = urllib.request.Request(url, headers=dict(headers)) if headers else url
    for attempt in range(1, attempts + 1):
        try:
            with urllib.request.urlopen(request, timeout=timeout) as response:
                body = response.read() if max_bytes is None else response.read(max_bytes + 1)
                # `read(amount)` does not notice a connection that closed early: compare with the header.
                announced = response.headers.get("Content-Length", "")
                wanted = int(announced) if announced.isdigit() else None
                if wanted is not None and len(body) < min(wanted, (max_bytes or wanted) + 1):
                    raise IncompleteRead(body, wanted - len(body))
            if max_bytes is not None and len(body) > max_bytes:
                raise ValueError(f"{url}: more than {max_bytes} bytes")
            return body
        except urllib.error.HTTPError as error:
            if error.code not in TRANSIENT or attempt == attempts:
                raise
            wait = _retry_after(error) or 5 * attempt
        except (OSError, HTTPException):
            if attempt == attempts:
                raise
            wait = 5 * attempt
        print(f"  attempt {attempt} failed; retrying in {wait}s", file=sys.stderr)
        time.sleep(wait)
    raise AssertionError("unreachable")


def _retry_after(error: urllib.error.HTTPError) -> int | None:
    value = error.headers.get("Retry-After") if error.headers else None
    if value and value.isdigit():
        return min(int(value), 60)
    return None
