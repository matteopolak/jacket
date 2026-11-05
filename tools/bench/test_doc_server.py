"""Check the matched benchmark server's compressed bytes and deterministic cache policy."""
import gzip
import http.client
from pathlib import Path
import tempfile
import threading
import unittest
from functools import partial

from bench.doc_server import Handler, Server, accepts_gzip


class ServerTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        root = Path(self.temporary.name)
        (root / 'index.html').write_text('<h1>Home</h1>')
        (root / 'guide.html').write_text('<p>shared content</p>' * 100)
        (root / '404.html').write_text('<h1>Missing</h1>')
        (root / 'search-index.json').write_text('{"entries":[]}')
        (root / 'versions.json').write_text('[]')
        self.server = Server(('127.0.0.1', 0), partial(Handler, directory=root))
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()

    def tearDown(self):
        self.server.shutdown()
        self.thread.join()
        self.server.server_close()
        self.temporary.cleanup()

    def request(self, path, method='GET', headers=None):
        connection = http.client.HTTPConnection(*self.server.server_address, timeout=5)
        connection.request(method, path, headers=headers or {})
        response = connection.getresponse()
        result = response.status, dict(response.getheaders()), response.read()
        connection.close()
        return result

    def test_gzip_and_identity_have_exact_decoded_bytes_and_separate_validators(self):
        status, plain_headers, plain = self.request('/guide')
        compressed_status, headers, compressed = self.request('/guide', headers={'Accept-Encoding': 'gzip'})
        self.assertEqual((status, compressed_status), (200, 200))
        self.assertEqual(headers['Content-Encoding'], 'gzip')
        self.assertEqual(gzip.decompress(compressed), plain)
        self.assertLess(len(compressed), len(plain))
        self.assertNotEqual(headers['ETag'], plain_headers['ETag'])
        self.assertEqual(headers['Vary'], 'Accept-Encoding')
        self.assertNotIn('Last-Modified', headers)
        self.assertEqual(headers['Cache-Control'], 'no-cache')
        (Path(self.temporary.name) / 'guide.html').touch()
        self.assertEqual(self.request('/guide', headers={'Accept-Encoding': 'gzip'})[1]['ETag'], headers['ETag'])
        head_status, head_headers, body = self.request('/guide', 'HEAD', {'Accept-Encoding': 'gzip'})
        self.assertEqual(head_status, 200)
        self.assertEqual(body, b'')
        self.assertEqual(int(head_headers['Content-Length']), len(compressed))
        self.assertEqual(head_headers['ETag'], headers['ETag'])

    def test_explicit_gzip_refusal_overrides_wildcard(self):
        self.assertFalse(accepts_gzip('gzip;q=0, *;q=1'))
        self.assertTrue(accepts_gzip('br, gzip;q=0.8'))
        self.assertFalse(accepts_gzip('gzip;q=invalid'))

    def test_conditional_asset_and_mutable_versions_policy(self):
        status, headers, _ = self.request('/search-index.json?v=sha')
        self.assertEqual(status, 200)
        self.assertEqual(headers['Cache-Control'], 'public, max-age=3600')
        status, conditional, body = self.request('/search-index.json?v=sha', headers={'If-None-Match': headers['ETag']})
        self.assertEqual(status, 304)
        self.assertEqual(conditional['ETag'], headers['ETag'])
        self.assertEqual(body, b'')
        self.assertEqual(self.request('/versions.json')[1]['Cache-Control'], 'no-cache')
        self.assertEqual(self.request('/does-not-exist')[0], 404)
        self.assertEqual(self.request('/does-not-exist')[2], b'<h1>Missing</h1>')


if __name__ == '__main__':
    unittest.main()
