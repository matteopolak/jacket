"""Downloads pinned by SHA-256, cached under `$XDG_CACHE_HOME/jaic/` (default `~/.cache/jaic/`)."""

import os
import shutil
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
from collections.abc import Mapping
from http.client import HTTPException, IncompleteRead
from pathlib import Path


from toolkit.hashing import sha256_file

__all__ = ["cache_dir", "fetch", "fetch_to", "pinned", "pinned_to", "request"]

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


def pinned_to(url: str, sha256: str, destination: Path, *, timeout: float = 120) -> None:
    """Copy the pinned file to `destination`, downloading it into the cache first if needed.

    For example `pinned_to(url, digest, corpus / "a.png")`; exits like `pinned` on a wrong digest.
    """
    shutil.copyfile(pinned(url, sha256, f"pinned/{sha256}", timeout=timeout), destination)


# Some hosts (get.nexte.st) refuse urllib's default "Python-urllib/x.y" agent with 403.
USER_AGENT = "jacket-tools"


# GitHub refuses or throttles anonymous downloads from shared CI runners (a 403 on an archive URL).
# CI exports the job's read-only `GITHUB_TOKEN`, sent to GitHub's own hosts only.
GITHUB_HOSTS = {"github.com", "codeload.github.com", "api.github.com"}


def _is_github(url: str) -> bool:
    return (urllib.parse.urlsplit(url).hostname or "") in GITHUB_HOSTS


def request(url: str, headers: Mapping[str, str] | None = None) -> urllib.request.Request:
    """The request `fetch` sends: our User-Agent, `headers`, and `GITHUB_TOKEN` for a GitHub host."""
    sent = {"User-Agent": USER_AGENT, **(headers or {})}
    token = os.environ.get("GITHUB_TOKEN")
    if token and _is_github(url):
        sent.setdefault("Authorization", f"Bearer {token}")
    return urllib.request.Request(url, headers=sent)


class GitHubRedirect(urllib.request.HTTPRedirectHandler):
    """Follows redirects, dropping the token when one leaves GitHub's hosts: release assets redirect
    to signed storage URLs, which must not receive it and refuse an extra Authorization."""

    def redirect_request(self, req, fp, code, msg, headers, newurl):
        moved = super().redirect_request(req, fp, code, msg, headers, newurl)
        if moved is not None and not _is_github(newurl):
            moved.remove_header("Authorization")
        return moved


# Installed for the process so `urlopen` (which the tests replace) follows redirects this way.
urllib.request.install_opener(urllib.request.build_opener(GitHubRedirect))


def _retrying(url: str, consume, *, timeout: float, attempts: int, headers: Mapping[str, str] | None):
    """`consume(response)` for `url`, opened again after a rate limit, a 5xx answer, a dropped connection or a
    truncated body, with a growing pause (honouring `Retry-After`); other 4xx answers (a wrong pin) fail at once."""
    prepared = request(url, headers)
    for attempt in range(1, attempts + 1):
        try:
            with urllib.request.urlopen(prepared, timeout=timeout) as response:
                return consume(response)
        except urllib.error.HTTPError as error:
            if error.code not in TRANSIENT or attempt == attempts:
                # `str(HTTPError)` leaves out the URL; keep the type and code, name the URL.
                raise urllib.error.HTTPError(url, error.code, f"{error.msg} for {url}", error.headers, None) from error
            wait = _retry_after(error) or 5 * attempt
        except (OSError, HTTPException):
            if attempt == attempts:
                raise
            wait = 5 * attempt
        print(f"  attempt {attempt} failed; retrying in {wait}s", file=sys.stderr)
        time.sleep(wait)
    raise AssertionError("unreachable")


def _announced(response) -> int | None:
    """The body length the server announced, if it did."""
    announced = response.headers.get("Content-Length", "")
    return int(announced) if announced.isdigit() else None


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

    def read(response) -> bytes:
        body = response.read() if max_bytes is None else response.read(max_bytes + 1)
        # `read(amount)` does not notice a connection that closed early: compare with the header.
        wanted = _announced(response)
        if wanted is not None and len(body) < min(wanted, (max_bytes or wanted) + 1):
            raise IncompleteRead(body, wanted - len(body))
        if max_bytes is not None and len(body) > max_bytes:
            raise ValueError(f"{url}: more than {max_bytes} bytes")
        return body

    return _retrying(url, read, timeout=timeout, attempts=attempts, headers=headers)


def fetch_to(
    url: str,
    path: Path,
    *,
    timeout: float = 120,
    attempts: int = ATTEMPTS,
    headers: Mapping[str, str] | None = None,
) -> None:
    """Stream `url` to `path` without holding it in memory (for archives), retrying as `fetch` does. The file
    appears whole or not at all: it is written beside `path` as `NAME.part` and moved into place."""
    path.parent.mkdir(parents=True, exist_ok=True)

    def write(response) -> None:
        # A name of its own, so two downloads of one path never write the same file.
        with tempfile.NamedTemporaryFile(dir=path.parent, prefix=path.name + ".", suffix=".part", delete=False) as out:
            partial = Path(out.name)
            try:
                shutil.copyfileobj(response, out, 1 << 20)
                out.flush()
                wanted = _announced(response)
                if wanted is not None and out.tell() < wanted:
                    raise IncompleteRead(b"", wanted - out.tell())
            except BaseException:
                out.close()
                partial.unlink(missing_ok=True)
                raise
        partial.replace(path)

    _retrying(url, write, timeout=timeout, attempts=attempts, headers=headers)


def _retry_after(error: urllib.error.HTTPError) -> int | None:
    value = error.headers.get("Retry-After") if error.headers else None
    if value and value.isdigit():
        return min(int(value), 60)
    return None
