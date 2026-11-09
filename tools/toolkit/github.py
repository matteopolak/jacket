"""GitHub's REST API: one authenticated GET with errors that name the cause.

    get("/repos/o/r/commits/<sha>")    the decoded JSON answer

`$GITHUB_TOKEN` or `$GH_TOKEN` is sent when set (the unauthenticated rate limit is low).
"""

import json
import os
import urllib.error
from http.client import HTTPException

from toolkit.download import fetch

__all__ = ["LookupFailed", "NotFound", "get"]


class LookupFailed(Exception):
    """GitHub could not be reached or did not answer as expected."""


class NotFound(LookupFailed):
    """GitHub answered 404."""


def get(path: str, *, timeout: float = 30):
    """The JSON GitHub's API answers for `path` (starting with `/`)."""
    headers = {"Accept": "application/vnd.github+json"}
    token = os.environ.get("GITHUB_TOKEN") or os.environ.get("GH_TOKEN")
    if token:
        headers["Authorization"] = "Bearer " + token
    try:
        return json.loads(fetch("https://api.github.com" + path, timeout=timeout, headers=headers))
    except urllib.error.HTTPError as error:
        if error.code == 404:
            raise NotFound(f"GitHub has no {path}") from error
        hint = " (rate limited: set GITHUB_TOKEN)" if error.code in (403, 429) else ""
        raise LookupFailed(f"GitHub answered {error.code} for {path}{hint}") from error
    except (OSError, HTTPException, ValueError) as error:
        raise LookupFailed(f"could not reach GitHub for {path}: {error}") from error
