"""Protocol coverage for the latency gate."""
import unittest
from bench.request_latency import METHODS, SCENARIOS, registered


class Coverage(unittest.TestCase):
    def test_every_registered_request_has_a_scenario(self):
        self.assertEqual(registered(), {variant for variant, _ in SCENARIOS})
        self.assertEqual(len(SCENARIOS), len(registered()))
        self.assertEqual(len(METHODS), len(set(METHODS)))

    def test_registry_coverage_accepts_standard_constants_and_extension_literals(self):
        source = '''impl Request {
            pub(super) const METHODS: &[(&str, Self)] = &[
                (lsp_types::request::HoverRequest::METHOD, Self::Hover),
                ("jai/source", Self::Source),
            ];
        }
        impl Notification {
            const METHODS: &[(&str, Self)] = &[("exit", Self::Exit)];
        }'''
        self.assertEqual(registered(source), {"Hover", "Source"})

    def test_gate_includes_every_measured_file_and_method(self):
        from bench.request_latency import make_limits, exceeds
        result = {'ms': [10, 15, 20], 'p50': 15, 'max': 20}
        report = {'results': [{'file': 'small.jai', 'requests': {'hover': result}}]}
        bound = make_limits(report)['small.jai']['hover']
        self.assertEqual(bound, {'max_p50_ms': 25, 'max_ms': 120})
        self.assertFalse(exceeds(result, 25, 120))
        self.assertTrue(exceeds(result, 14, 120))
        self.assertTrue(exceeds(result, 25, 19))
        self.assertFalse(exceeds(result, 15, 20))
        slow = {'ms': [200], 'p50': 200, 'max': 300}
        report['results'][0]['requests']['rename'] = slow
        self.assertEqual(make_limits(report)['small.jai']['rename'],
                         {'max_p50_ms': 250, 'max_ms': 375})

    def test_cli_fails_the_gate_and_still_writes_samples(self):
        import contextlib
        import io
        import json
        import tempfile
        from pathlib import Path
        from unittest.mock import patch
        from bench.request_latency import main
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'run.json'
            row = {'file': 'small.jai', 'requests': {'textDocument/hover': {'ms': [10, 20]}}}
            args = ['request_latency', '--server', '/server', '--output', str(output),
                    '--max-p50-ms', '14', 'small.jai']
            with patch('sys.argv', args), patch('bench.request_latency.run', return_value=row), contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(main(), 1)
            self.assertEqual(json.loads(output.read_text())['results'][0]['requests']['textDocument/hover']['p50'], 15)
