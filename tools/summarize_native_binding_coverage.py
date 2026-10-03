#!/usr/bin/env python3
"""Refresh native-family source evidence from authored outputs and reports.

Only an explicitly selected frozen own compiler is executed, in parse mode.
External libraries and supplied binaries are never opened, linked or run.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

from check_native_binding_sources import digest, parse, procedures
from rewrite_native_api_contracts import normalize_contract


REPORTS = {
    'graphics': 'stdlib/.coverage/graphics-source-parse.json',
    'platform': 'stdlib/.coverage/platform-sdk-bindings.json',
    'window_audio': 'stdlib/.coverage/window-audio-bindings.json',
    'codecs': 'stdlib/.coverage/codec-native-bindings.json',
}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--parser', required=True, type=Path)
    parser.add_argument('--output', type=Path, default=Path('stdlib/.coverage/native-platform-bindings.json'))
    options = parser.parse_args()
    binary_before = digest(options.parser)
    reports = {name: json.loads(Path(path).read_text()) for name, path in REPORTS.items()}
    families = {}
    families['graphics'] = [row['path'] for row in reports['graphics']['files']]
    families['platform'] = [row['output'] for row in reports['platform']['files'] if 'output' in row]
    families['window_audio'] = [row['path'] for row in reports['window_audio']['files']]
    families['codecs'] = sorted({path for row in reports['codecs']['modules'] for path in row['files']})
    all_paths = sorted(set(path for paths in families.values() for path in paths))
    rows = []
    for name in all_paths:
        path = Path(name)
        before = digest(path)
        source = path.read_text()
        _, inventory = normalize_contract(source)
        bodies = procedures(source)
        row = {'path': name, 'owners': [family for family, paths in families.items() if name in paths],
            'sha256_before': before, **parse(options.parser, path),
            'authored_body_occurrences': len(bodies),
            'empty_body_occurrences': [{'name': body['name'], 'line': body['line']} for body in bodies if body['empty_body']],
            'foreign_declaration_occurrences': inventory['native_declarations'],
            'library_locators': inventory['library_locators'],
            'native_symbols_resolved': False, 'semantic_resolution_verified': False}
        row['sha256_after'] = digest(path)
        row['source_stable_during_check'] = before == row['sha256_after']
        rows.append(row)
    binary_after = digest(options.parser)
    report = {'format': 1, 'scope': 'included independently authored standard-library native-facing modules',
        'status': 'partial-authoring-with-explicit-gaps; source syntax checked, native behavior unverified',
        'supplied_implementation_bodies_reused': False, 'supplied_native_artifacts_used': False,
        'external_native_runtimes_reimplemented': False, 'native_behavior_verified': False,
        'native_abi_verified': False, 'foreign_declarations_are_behavior_passes': False,
        'speculative_sdl3_or_vulkan_current_namespace_in_public_stdlib': False,
        'parser': str(options.parser), 'parser_sha256_before': binary_before,
        'parser_sha256_after': binary_after, 'parser_stable': binary_before == binary_after,
        'parser_build_inputs_verified': False,
        'minimum_free_disk_bytes': 2 * 1024 * 1024 * 1024,
        'component_reports': [{'family': name, 'path': REPORTS[name],
            'sha256': digest(Path(REPORTS[name])),
            'component_summary': reports[name].get('totals', reports[name].get('summary')),
            'files': families[name]} for name in REPORTS],
        'coverage_interpretation': {
            'body_occurrences': 'Named name::(...) and inline procedure body occurrences, including conditional/overloaded procedures and genuine disabled hooks; not a runtime-pass count. Component wrapper-implementation inventories may separately count target-specific dispatch branches, so their totals need not sum to this lexical count.',
            'foreign_declarations': 'Native API or unavailable compatibility declaration occurrences, not verified installed symbols.',
            'adapter_counts': 'Read each component report for exact unavailable Jai adapters, ABI guards and custom native compatibility requirements.',
            'whole_file_syntax': 'Only syntax; imported module graph and target/native semantics remain unverified.',
            'graphics_body_witnesses': 'Ten authored body witnesses separate procedure syntax from earlier retained declaration syntax gaps.'},
        'graphics_exact_unimplemented_adapters': [{'path': row['path'], **adapter}
            for row in reports['graphics']['files'] for adapter in row['unimplemented_adapters']],
        'documentation': ['docs/stdlib/native-platform-bindings.md', 'docs/stdlib/platform-sdk-bindings.md',
            'docs/stdlib/window-audio-bindings.md', 'docs/stdlib/codec-native-bindings.md'],
        'current_registry_provenance': {
            'vulkan': json.loads(Path('stdlib/.coverage/vulkan-upstream-provenance.json').read_text()),
            'opengl_report': 'stdlib/.coverage/gl-current-registry.json'},
        'files': rows,
        'totals': {'source_files': len(rows), 'whole_file_syntax_passes': sum(row['syntax_pass'] for row in rows),
            'whole_file_syntax_gaps': sum(not row['syntax_pass'] for row in rows),
            'authored_body_occurrences': sum(row['authored_body_occurrences'] for row in rows),
            'foreign_declaration_occurrences': sum(row['foreign_declaration_occurrences'] for row in rows),
            'empty_body_occurrences': sum(len(row['empty_body_occurrences']) for row in rows),
            'source_stability_gaps': sum(not row['source_stable_during_check'] for row in rows),
            'native_behavior_passes': 0, 'native_behavior_runs': 0, 'native_abi_runs': 0}}
    options.output.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report['totals']))


if __name__ == '__main__':
    main()
