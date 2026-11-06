#!/usr/bin/env python3
"""Serve frozen production output with matched gzip/cache policy for browser measurements.

PYTHONPATH=tools python3 tools/bench/doc_server.py --directory /tmp/site --port 8000
This controlled local policy models HTTP transfer; it does not measure a hosted CDN.
"""
import argparse
from collections import OrderedDict
from functools import partial
import gzip
from hashlib import sha256
from http.server import ThreadingHTTPServer
import io
import mimetypes
from pathlib import Path
import threading

from dev.doc_serve import Handler as PreviewHandler


class Representations:
    """Bound compressed/identity working sets by bytes; validators depend on bytes, not mtimes."""
    def __init__(self, limit=32 * 1024 * 1024):
        self.limit = limit
        self.bytes = 0
        self.values = OrderedDict()
        self.lock = threading.Lock()

    def read(self, path, compressed):
        """Return encoded bytes and their ETag; read(Path('guide.html'), True) selects gzip."""
        stat = path.stat()
        key = (path, stat.st_mtime_ns, stat.st_size, compressed)
        with self.lock:
            if key in self.values:
                self.values.move_to_end(key)
                return self.values[key]
            raw = path.read_bytes()
            body = gzip.compress(raw, compresslevel=6, mtime=0) if compressed else raw
            value = body, '"' + sha256(body).hexdigest() + '"'
            if len(body) <= self.limit:
                while self.values and (self.bytes + len(body) > self.limit or len(self.values) >= 256):
                    _, previous = self.values.popitem(last=False)
                    self.bytes -= len(previous[0])
                self.values[key] = value
                self.bytes += len(body)
            return value


class Server(ThreadingHTTPServer):
    """One bounded representation cache per frozen output server."""
    def __init__(self, address, handler):
        self.representations = Representations()
        super().__init__(address, handler)


def accepts_gzip(value):
    """Honor explicit gzip refusal before a wildcard, e.g. gzip;q=0, *;q=1."""
    encodings = {}
    for item in value.lower().split(','):
        name, _, options = item.strip().partition(';')
        try:
            quality = float(options.strip().removeprefix('q=')) if options else 1
        except ValueError:
            quality = 0
        encodings[name] = quality
    return 0 < encodings.get('gzip', encodings.get('*', 0)) <= 1


class Handler(PreviewHandler):
    """Use slashless static routes, content-hash ETags and identical explicit cache headers."""
    protocol_version = 'HTTP/1.1'
    def send_head(self):
        """Negotiate one real static response; GET /guide resolves guide.html without redirect."""
        root = Path(self.directory).resolve()
        path = Path(self.translate_path(self.path)).resolve()
        if path.is_dir() and self.path.split('?', 1)[0].endswith('/'):
            path /= 'index.html'
        found = path.is_relative_to(root) and path.is_file()
        status = 200 if found else 404
        if not found:
            path = root / '404.html'
            if not path.is_file():
                self.send_error(404)
                return None
        mime = mimetypes.guess_type(path)[0] or 'application/octet-stream'
        compressible = mime.startswith('text/') or mime in {
            'application/json', 'application/javascript', 'application/wasm',
            'application/xml', 'image/svg+xml',
        }
        compressed = compressible and accepts_gzip(self.headers.get('Accept-Encoding', ''))
        body, etag = self.server.representations.read(path, compressed)
        conditional = [value.strip().removeprefix('W/') for value in self.headers.get('If-None-Match', '').split(',')]
        not_modified = status == 200 and (etag in conditional or '*' in conditional)
        self.send_response(304 if not_modified else status)
        self.send_header('ETag', etag)
        self.send_header('Vary', 'Accept-Encoding')
        relative = path.relative_to(root)
        published_asset = relative.parts[0] in {'assets', '_astro'}
        mutable = (path.suffix == '.html' and not published_asset) or relative == Path('versions.json') or status != 200
        self.send_header('Cache-Control', 'no-cache' if mutable else 'public, max-age=3600')
        if not not_modified:
            self.send_header('Content-Type', mime)
            self.send_header('Content-Length', str(len(body)))
            if compressed:
                self.send_header('Content-Encoding', 'gzip')
        self.end_headers()
        return None if not_modified else io.BytesIO(body)


def main():
    """Bind loopback, e.g. --directory /tmp/site --port 8100; never expose a deployment."""
    parser = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    parser.add_argument('--directory', required=True, type=Path)
    parser.add_argument('--port', default=8000, type=int)
    args = parser.parse_args()
    root = args.directory.resolve(strict=True)
    if not root.is_dir():
        parser.error('--directory must name frozen production output')
    with Server(('127.0.0.1', args.port), partial(Handler, directory=root)) as server:
        print(f'Matched gzip/ETag server: http://127.0.0.1:{server.server_port}/', flush=True)
        server.serve_forever()


if __name__ == '__main__':
    main()
