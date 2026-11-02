#!/usr/bin/env python3
"""Serves a `jacket doc` site the way GitHub Pages does.

A page is written as `api/Basic/print.html` and linked as `/api/Basic/print`, so the server
looks a path up as a file, then as `<path>.html`, then as a folder's `index.html`. It never
redirects, and a missing path gets `404.html` with status 404.

    python3 tools/dev/doc_serve.py [--port 8000] [--directory target/doc-site]
"""

import argparse
import functools
import http.server
import os


class Handler(http.server.SimpleHTTPRequestHandler):
    def translate_path(self, path):
        found = super().translate_path(path)
        if os.path.isfile(found):
            return found
        if not path.split("?", 1)[0].split("#", 1)[0].endswith("/") and os.path.isfile(found + ".html"):
            return found + ".html"
        return found

    def send_head(self):
        found = self.translate_path(self.path)
        if os.path.isdir(found) and not (
            self.path.split("?", 1)[0].endswith("/") and os.path.isfile(os.path.join(found, "index.html"))
        ):
            # A folder is only a page through its index.html, and never by redirect.
            self.send_error(404)
            return None
        return super().send_head()

    def send_error(self, code, message=None, explain=None):
        page = os.path.join(self.directory, "404.html")
        if code != 404 or not os.path.isfile(page):
            return super().send_error(code, message, explain)
        with open(page, "rb") as f:
            body = f.read()
        self.send_response(404)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        if self.command != "HEAD":
            self.wfile.write(body)


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--port", type=int, default=8000)
    parser.add_argument("--directory", default="target/doc-site")
    args = parser.parse_args()
    handler = functools.partial(Handler, directory=args.directory)
    with http.server.ThreadingHTTPServer(("127.0.0.1", args.port), handler) as server:
        print(f"serving {args.directory} at http://127.0.0.1:{args.port}/")
        server.serve_forever()


if __name__ == "__main__":
    main()
