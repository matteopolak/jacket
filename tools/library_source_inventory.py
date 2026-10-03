#!/usr/bin/env python3
"""Read pinned Jai library metadata without executing source or native assets."""
from __future__ import annotations
import collections
from collections.abc import Collection
import hashlib
import json
from pathlib import Path
import re
import sys

from check_corpus import Source, inventory
from inventory_corpus_features import decode

_NUMBER = re.compile(r'0[xXhH][0-9a-fA-F_]+|0[bB][01_]+|[0-9][0-9_]*(?:\.[0-9][0-9_]*)?(?:[eE][+-]?[0-9][0-9_]*)?')
_TOKEN = re.compile(r'#[A-Za-z_]\w*|[A-Za-z_]\w*|' + _NUMBER.pattern + r'|::|:=|==|!=|->|\S')
_DELIMITER = re.compile(r'[ \t]+([A-Za-z_]\w*)')

def tokens(text: str) -> list[tuple[str, str, int]]:
    out = []
    offset, line = 0, 1
    while offset < len(text):
        if text[offset].isspace():
            line += text[offset] == '\n'
            offset += 1
            continue
        if text.startswith('//', offset):
            end = text.find('\n', offset)
            offset = len(text) if end < 0 else end
            continue
        if text.startswith('/*', offset):
            depth = 1
            offset += 2
            while offset < len(text) and depth:
                if text.startswith('/*', offset):
                    depth += 1
                    offset += 2
                elif text.startswith('*/', offset):
                    depth -= 1
                    offset += 2
                else:
                    line += text[offset] == '\n'
                    offset += 1
            continue
        if text[offset] == '"':
            start, at = offset, line
            offset += 1
            while offset < len(text):
                if text[offset] == '\\':
                    offset += 2
                elif text[offset] == '"':
                    offset += 1
                    break
                else:
                    line += text[offset] == '\n'
                    offset += 1
            raw = text[start:offset]
            try:
                value = json.loads(raw)
            except ValueError:
                value = raw[1:-1]
            out.append(('string', value, at))
            continue
        match = _TOKEN.match(text, offset)
        value, at, offset = match[0], line, match.end()
        kind = ('directive' if value.startswith('#') else 'id'
                if re.fullmatch(r'[A-Za-z_]\w*', value) else 'number'
                if _NUMBER.fullmatch(value) else 'punct')
        out.append((kind, value, at))
        if value == '#string':
            delimiter = _DELIMITER.match(text, offset)
            if delimiter:
                start, offset = offset, delimiter.end()
                end = re.compile(r'(?m)^[ \t]*' + re.escape(delimiter[1])
                                 + r'[ \t]*\r?$').search(text, offset)
                stop = len(text) if end is None else end.end()
                line += text[start:stop].count('\n')
                offset = stop
                out.append(('here-string', '', at))
    return out

def parameter(segment: list[tuple[str, str, int]]) -> dict | None:
    if not segment or segment[0][0] != 'id':
        return None
    assignment = next((i for i, t in enumerate(segment)
                       if t[1] in {':=', '='}), None)
    default = segment[assignment + 1:] if assignment is not None else []
    scalar = None
    if len(default) == 1:
        kind, value, _ = default[0]
        if kind == 'number' and not value.lower().startswith('0h'):
            value = value.replace('_', '')
            if value.lower().startswith(('0x', '0b')):
                scalar = int(value, 0)
            elif value.isdigit():
                scalar = int(value)
        elif kind == 'string':
            scalar = value
        elif value in {'true', 'false'}:
            scalar = value == 'true'
    type_end = assignment if assignment is not None else len(segment)
    return {'name': segment[0][1], 'line': segment[0][2],
            'required': assignment is None, 'default_scalar': scalar,
            'default_references': sorted({t[1] for t in default
                                           if t[0] == 'id' and t[1] not in {'true', 'false', 'null'}}),
            'type_references': sorted({t[1] for t in segment[1:type_end] if t[0] == 'id'})}

def scan_source(path: Path | str) -> dict:
    path = Path(path)
    raw = path.read_bytes()
    ts = tokens(decode(raw))
    result = {'path': str(path), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest(),
              'imports': [], 'loads': [], 'foreign_libraries': [], 'module_parameters': [],
              'platform_branches': [], 'directive_counts': dict(sorted(collections.Counter(
                  t[1] for t in ts if t[0] == 'directive').items())),
              'compile_time_effects_present': any(t[1] == '#run' for t in ts),
              'compiler_intrinsics_present': any(t[1] == '#compiler' for t in ts),
              'foreign_prototype_count': sum(t[1] == '#foreign' for t in ts),
              'directives': [],
              'has_module_parameters': any(t[1] == '#module_parameters' for t in ts),
              'has_compile_time_execution': any(t[1] in {'#run', '#insert'} for t in ts)}
    for n, (_, value, line) in enumerate(ts):
        if value in {'#import', '#load', '#library', '#system_library', '#module_parameters', '#run', '#insert'}:
            at, flags = n + 1, []
            while at + 1 < len(ts) and ts[at][1] == ',':
                flags.append(ts[at + 1][1])
                at += 2
            operand = ts[at][1] if at < len(ts) and ts[at][0] == 'string' and 'string' not in flags else None
            result['directives'].append({'kind': 'library' if value == '#system_library' else value[1:],
                                         'spelling': value[1:], 'line': line, 'flags': flags,
                                         'literal_operand': operand})
        if value in {'#import', '#load', '#library', '#system_library'}:
            j, flags = n + 1, []
            while j + 1 < len(ts) and ts[j][1] == ',':
                flags.append(ts[j + 1][1])
                j += 2
            literal = ts[j][1] if j < len(ts) and ts[j][0] == 'string' else None
            item = {'line': line, 'flags': flags, 'target': None if 'string' in flags else literal,
                    'dynamic': literal is None, 'here_string': j < len(ts) and ts[j][0] == 'here-string'}
            if value == '#load':
                resolved = path.parent / literal if literal is not None else None
                item.update(resolved_path=str(resolved) if resolved else None,
                            source_exists=resolved.is_file() if resolved else None)
                result['loads'].append(item)
            elif value == '#import':
                result['imports'].append(item)
            else:
                item['kind'] = 'system' if value == '#system_library' or 'system' in flags else 'local'
                item['binding'] = ts[n - 2][1] if n >= 2 and ts[n - 1][1] == '::' else None
                result['foreign_libraries'].append(item)
        if value == '#module_parameters':
            j = n + 1
            while j < len(ts) and ts[j][1] == '(':
                j, depth, segment = j + 1, 1, []
                while j < len(ts) and depth:
                    token = ts[j]
                    if token[1] in {'(', '[', '{'}:
                        depth += 1
                    if token[1] in {')', ']', '}'}:
                        depth -= 1
                    if depth == 0 or (depth == 1 and token[1] == ','):
                        item = parameter(segment)
                        if item is not None:
                            result['module_parameters'].append(item)
                        segment = []
                    else:
                        segment.append(token)
                    j += 1
        if value == '#if':
            j, names = n + 1, []
            while j < len(ts) and ts[j][1] not in {'{', ';'}:
                if ts[j][0] == 'id':
                    names.append(ts[j][1])
                j += 1
            if any(v in {'OS', 'CPU', 'OS_IS_UNIX', 'POSIX_THREADS'}
                   or v.startswith(('OS_', 'CPU_')) for v in names):
                item = {'line': line, 'references': sorted(set(names)), 'cases': []}
                if j < len(ts) and ts[j][1] == '{' and j > n and ts[j - 1][1] == '==':
                    depth, j = 1, j + 1
                    while j < len(ts) and depth:
                        v = ts[j][1]
                        if v == '{':
                            depth += 1
                        elif v == '}':
                            depth -= 1
                        elif v == 'case' and depth == 1:
                            k, names = j + 1, []
                            while k < len(ts) and ts[k][1] != ';':
                                if ts[k][0] == 'id':
                                    names.append(ts[k][1])
                                k += 1
                            item['cases'].append({'line': ts[j][2], 'match_symbols': names,
                                                  'default': not names})
                        j += 1
                result['platform_branches'].append(item)
    platform_names = {'WINDOWS', 'LINUX', 'MACOS', 'ANDROID', 'IOS', 'PS4', 'PS5', 'XBOX', 'FREEBSD', 'ARM64', 'X64', 'X86'}
    symbols = set()
    for branch in result['platform_branches']:
        symbols.update(branch['references'])
        for case in branch['cases']:
            symbols.update(case['match_symbols'])
    result['platform_spellings'] = sorted(symbols & platform_names)
    result['has_foreign_libraries'] = bool(result['foreign_libraries'])
    return result

def declared_load_closure(root: Path | str, source_rows: list[dict]) -> list[dict]:
    # All declared literal branches; no parameter or OS/CPU evaluation.
    rows = {row['path']: row for row in source_rows}
    pending, seen, closure = [str(root)], set(), []
    while pending:
        path = pending.pop()
        if path in seen or path not in rows:
            continue
        seen.add(path)
        row = rows[path]
        closure.append(row)
        pending.extend(load['resolved_path'] for load in row['loads']
                       if load['resolved_path'] in rows)
    return sorted(closure, key=lambda row: row['path'])


_BOOTSTRAP = {'Preload', 'Runtime_Support', 'Runtime_Support_Crash_Handler'}
_METAPROGRAMS = {
    'Autorun', 'Default_Metaprogram', 'Minimal_Metaprogram', 'Example_Plugin',
    'Metaprogram_Plugins', 'MacOS_Bundler', 'linux_build',
}
_STUBS = {
    'withlang-dev/open-jai:modules/File/module.jai':
        ('filesystem operations return fixed statuses, empty contents, or zero lengths', [15, 23, 31, 39, 43, 55]),
    'withlang-dev/open-jai:modules/Process/module.jai':
        ('run_command returns fixed output without an OS process operation', [24, 29, 32]),
    'withlang-dev/open-jai:modules/Thread/module.jai':
        ('void-pointer callbacks and synchronous simulated thread and mutex state', [11, 12, 45, 65, 113, 128, 136]),
}


def _purpose(source: Source, relative: Path, candidate: bool) -> tuple[str, str]:
    """Purpose labels describe evidence; they are not expected rejection rules."""
    stem = relative.stem
    test_path = {part.lower() for part in relative.parts} & {'test', 'tests'}
    if test_path or (candidate and (stem.startswith('Test') or relative.parent.name.startswith('Test'))):
        return 'test-fixture', 'test path or named tutorial/test module; no rejection inferred'
    if candidate and stem in _BOOTSTRAP:
        return 'compiler-bootstrap', 'bootstrap source identity and module parameter declarations'
    if candidate and (stem in _METAPROGRAMS or 'Build' in relative.parts or 'Build_Utils' in relative.parts):
        return 'compiler-metaprogram', 'compiler plugin/build module identity and source declarations'
    if candidate and (stem.startswith('generate') or 'Generator' in stem or 'Generator' in relative.parent.name):
        return 'generator-library', 'generator module identity and exported generation procedures'
    if candidate and relative.name == 'module.jai' and relative.parent.name == 'Compiler':
        return 'compiler-interface', 'Compiler module root and compiler binding declarations'
    if source.id in _STUBS:
        return 'verified-compatibility-stub', _STUBS[source.id][0]
    if source.project == 'reference' and relative.parts[0] == 'modules':
        return 'standard-library', 'source from the pinned supplied standard-library tree'
    if source.project == 'withlang-dev/open-jai':
        return 'upstream-compiler-compatibility', 'module shipped by an alternate compiler implementation'
    if source.project == 'ostef/Vk-Engine' and 'Source' in relative.parts:
        return 'project-component', 'engine Source component module tree'
    return ('upstream-library' if candidate else 'support'), 'source path and module root convention'


def _source_role(relative: Path, candidate: bool, reviewed_negative: bool) -> tuple[str, str]:
    if reviewed_negative:
        return 'expected-negative', 'reviewed corpus/acceptance.json negative contract'
    parts = {part.lower() for part in relative.parts}
    test_name = relative.stem in {'tests', 'test'} or relative.stem.endswith('_tests')
    test_module = candidate and (relative.stem.startswith('Test') or relative.parent.name.startswith('Test'))
    if parts & {'test', 'tests'} or test_name or test_module:
        return 'test-fixture', 'test path or explicit test module naming; no rejection inferred'
    if parts & {'examples', 'example', 'how_to', 'exercises', 'local_modules'}:
        return ('example-library-entrypoint' if candidate else 'example-or-tutorial'), 'scoped example or tutorial source path'
    if candidate:
        return 'library-entrypoint', 'module root or top-level named module source'
    if relative.name.lower() in {'generate.jai', 'build.jai', 'generator.jai'}:
        return 'generator-or-build-support', 'generator/build source without a conventional root'
    return 'library-support', 'non-root source; no standalone library acceptance inferred'


def classify_source(source: Source, reviewed_negative_ids: Collection[str]) -> tuple[str, str, bool, bool, bool]:
    """Return role, basis, and entrypoint eligibility without reading the source."""
    relative = Path(source.id.split(':', 1)[1])
    in_standard = source.project == 'reference' and relative.parts[0] == 'modules'
    conventional = relative.name == 'module.jai' or (in_standard and len(relative.parts) == 2)
    named = (source.project != 'reference' and relative.parent.name.lower() in {'modules', 'my_modules'}
             and not ({part.lower() for part in relative.parts} & {'test', 'tests', 'examples', 'example'}))
    candidate = (conventional or named) and source.id not in reviewed_negative_ids
    role, basis = _source_role(relative, candidate, source.id in reviewed_negative_ids)
    return role, basis, candidate, conventional, named


def scan(root: Path) -> dict:
    """Inventory all pinned sources and expose conventional and named roots.

    `candidates` holds source IDs; `sources` contains their rich metadata. The
    reviewed corpus negatives control expected outcomes; upstream harness
    expectations remain observations until the main acceptance tool reviews them.
    """
    root = root.resolve()
    pinned = inventory(root)
    reviewed = {
        case['id']: case for case in json.loads((root / 'corpus/acceptance.json').read_text())['cases']
        if case['kind'] == 'negative'
    }
    rows, negative_references = [], {}
    for source in pinned:
        if source.sha256 != source.expected_sha256:
            raise ValueError(f'pinned source missing or changed: {source.id}')
        relative = Path(source.id.split(':', 1)[1])
        role, basis, candidate, conventional, named = classify_source(source, reviewed)
        observation = scan_source(source.path)
        observation['source'] = str(source.path)
        observation['path'] = source.path.relative_to(root).as_posix()
        observation.update(id=source.id, source_origin=source.project, relative_path=relative.as_posix(),
                           revision=source.revision, pin_sha256=source.expected_sha256, pin_match=True,
                           module_candidate=candidate, conventional_entrypoint=conventional,
                           named_module_candidate=named and not conventional)
        for load in observation['loads']:
            if load['resolved_path'] is not None:
                resolved = Path(load['resolved_path']).resolve()
                load['resolved_path'] = resolved.relative_to(root).as_posix() if resolved.is_relative_to(root) else str(resolved)
        observation['role'], observation['role_basis'] = role, basis
        observation['purpose'], observation['purpose_basis'] = _purpose(source, relative, candidate)
        if source.id in _STUBS:
            observation['implementation_evidence'] = {'reason': _STUBS[source.id][0], 'lines': _STUBS[source.id][1]}
        if source.id in reviewed:
            observation['reviewed_negative'] = reviewed[source.id]['negative']
        rows.append(observation)
        ts = tokens(decode(source.path.read_bytes()))
        project_root = root / 'reference' if source.project == 'reference' else root / 'corpus/upstream' / source.project.replace('/', '--')
        for index, token in enumerate(ts):
            if (token[1] == 'expect_compile_failure' and index + 2 < len(ts)
                    and ts[index + 1][1] == '(' and ts[index + 2][0] == 'string'):
                target = (project_root / ts[index + 2][1]).resolve()
                if not target.is_relative_to(project_root.resolve()):
                    continue
                key = target.relative_to(root).as_posix()
                negative_references.setdefault(key, []).append({'harness_id': source.id, 'line': token[2]})
    by_path = {row['path']: row for row in rows}
    for path, evidence in negative_references.items():
        if path in by_path:
            by_path[path]['upstream_expected_failure_evidence'] = evidence
    candidates = [row['id'] for row in rows if row['module_candidate']]
    core = {}
    for name in ('File', 'Process', 'Thread'):
        core[name] = {'entrypoint': path, 'sha256': by_path[path]['sha256'],
                      'module_parameters': by_path[path]['module_parameters'],
                      'declared_local_load_closure': [row['path'] for row in declared_load_closure(path, rows)]}
    return {
        'format': 1, 'method': 'pinned read-only lexical source inventory; all branches, without target evaluation',
        'source_excerpts_retained': False, 'execution_performed': False,
        'summary': {
            'source_rows': len(rows), 'reference_files': sum(row['source_origin'] == 'reference' for row in rows),
            'upstream_files': sum(row['source_origin'] != 'reference' for row in rows),
            'candidate_count': len(candidates),
            'conventional_entrypoints': sum(row['conventional_entrypoint'] for row in rows),
            'additional_named_candidates': sum(row['named_module_candidate'] for row in rows),
            'candidate_roles': dict(collections.Counter(row['role'] for row in rows if row['module_candidate'])),
            'candidate_purposes': dict(collections.Counter(row['purpose'] for row in rows if row['module_candidate'])),
            'pin_mismatches': [],
        },
        'sources': rows, 'candidates': candidates, 'core_original_entrypoints': core,
        'negative_references': [{'target_path': path, 'source_present': path in by_path,
                                 'filesystem_exists': (root / path).is_file(), 'evidence': evidence}
                                for path, evidence in sorted(negative_references.items())],
        'reviewed_negative_ids': sorted(reviewed),
        'limitations': ['Lexical observations are not parse, check, or runtime acceptance.',
                        'All branch dependencies are declared; selected target/parameters are not evaluated.',
                        'Default expressions are summarized as scalar values or identifier references.'],
    }


if __name__ == '__main__':
    if len(sys.argv) != 2:
        raise SystemExit('usage: library_source_inventory.py REPOSITORY_ROOT')
    print(json.dumps(scan(Path(sys.argv[1])), indent=2))
