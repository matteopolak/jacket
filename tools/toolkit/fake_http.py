"""Stand-ins for `urllib.request.urlopen` results in tests."""
from __future__ import annotations

import io
from email.message import Message


def response(body: bytes, headers: dict[str, str] | None = None) -> io.BytesIO:
    """A readable response with `body` and the headers a server would send (Content-Length by default)."""
    result = io.BytesIO(body)
    message = Message()
    for name, value in {'Content-Length': str(len(body)), **(headers or {})}.items():
        message[name] = value
    result.headers = message  # type: ignore[attr-defined]
    return result
