"""Fetch a pinned URL for the build tools, retrying: hosted runners drop connections and the
hosts behind raw.githubusercontent.com and release downloads answer 5xx now and then."""
import http.client
import time
import urllib.error
import urllib.request


def fetch(url, timeout=120, attempts=6):
    """The body of `url`. Network errors, 5xx answers and truncated bodies are retried with a
    growing pause; 4xx answers (a wrong pin) fail at once."""
    for attempt in range(1, attempts + 1):
        try:
            with urllib.request.urlopen(url, timeout=timeout) as response:
                return response.read()
        except urllib.error.HTTPError as error:
            if error.code < 500 or attempt == attempts:
                raise
        except (OSError, http.client.HTTPException):
            if attempt == attempts:
                raise
        time.sleep(10 * attempt)
