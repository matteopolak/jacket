#!/usr/bin/env python3
"""Inventory independent-library contracts without executing original inputs."""
from __future__ import annotations

import argparse
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import hashlib
import gzip
import json
import os
from pathlib import Path
import subprocess

from check_corpus import ROOT, inventory
from inventory_corpus_features import decode
from library_source_inventory import scan_source, tokens


FAMILIES = {
    'basic-collections': 'Basic Bit_Array Bucket_Array Flat_Pool Hash_Table IntroSort RadixSort Pool Sort Treemap Relative_Pointers Soa',
    'allocation-memory': 'Default_Allocator Overwriting_Allocator Unmapping_Allocator Memory Remap_Context Deep_Copy Bit_Operations Hash Crc meow_hash xxHash',
    'strings-serialization': 'String Unicode Base64 Command_Line Print_Color Print_Vars Text_File_Handler Wav_File Ico_File Zip_File_Directory Adpcm md5',
    'math-numeric': 'Math Sloppy_Math Srgb Float16 Random PCG Machine_X64',
    'os-file-process': 'File File_Async File_Utilities File_Watcher Process System Clipboard Mail Shared_Memory_Channel',
    'thread-socket': 'Thread Atomics Socket Input Keymap Gamepad',
    'compiler-reflection': 'Compiler Reflection Runtime_Support Runtime_Support_Crash_Handler Code_Visit Tagged_Union Jai_Lexer Default_Metaprogram Minimal_Metaprogram Metaprogram_Plugins Debug Check Protocol_For_Memory_Visualization Preload Autorun Bindings_Generator BuildCpp Codex Example_Plugin Iprof MacOS_Bundler Performance_Report Program_Print Project_Generator Simple_Package Toolchains generate_c_header linux_build debug_info executable_formats',
    'native-platform-bindings': 'SDL POSIX POSIX_old Linux macos Windows Android Objective_C Metal GL Vulkan X11 Window_Creation Window_Type Icon Windows_Utf8 Windows_Registry Windows_Resources Curl ImGui freetype-2.12.1 freetype255 lz4 meshoptimizer stb_image stb_image_resize stb_image_write stb_vorbis pl_mpeg d3d11 d3d12 d3d_compiler dxgi dxc_compiler rpmalloc telemetry3 nvidia_aftermath nvtt MojoShader Thekla_Atlas Thekla_Baker Sound_Player',
    'ui-drawing': 'Simp GetRect GetRect_LeftHanded',
    'compatibility-fixtures': 'TestScope TestModule_Params TestModule_TypedParams',
}
OWNERS = {name: family for family, names in FAMILIES.items() for name in names.split()}


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def reviewed_paths() -> set[Path]:
    """Retain the source-set identity, including newly added unrooted files."""
             ROOT / 'stdlib', ROOT / 'prelude']
    return {path.resolve() for root in roots for path in root.rglob('*.jai')}


def changed_paths(before: dict[Path, str], after: set[Path]) -> list[str]:
    changed = set(before) ^ after
    changed.update(path for path in set(before) & after if sha(path.read_bytes()) != before[path])
    return sorted(path.relative_to(ROOT).as_posix() for path in changed)


def nested_declarations(items: list[dict]):
    for item in items:
        yield item
        yield from nested_declarations(item['members'])


def balanced(ts: list[tuple], start: int) -> int:
    pairs = {'(': ')', '[': ']', '{': '}'}
    stack = [pairs[ts[start][1]]]
    for index in range(start + 1, len(ts)):
        value = ts[index][1]
        if value in pairs:
            stack.append(pairs[value])
        elif value in {')', ']', '}'}:
            if not stack or stack.pop() != value:
                return len(ts)
            if not stack:
                return index + 1
    return len(ts)


def contract_hash(ts: list[tuple]) -> str:
    """Ignore implementation locators; retain modifiers/defaults/type spellings."""
    result, skip_locator = [], False
    for kind, value, _ in ts:
        if value in {'#foreign', '#elsewhere', '#compiler', '#intrinsic'}:
            skip_locator = value in {'#foreign', '#elsewhere'}
            continue
        if skip_locator:
            if value.startswith('#'):
                skip_locator = False
            else:
                continue
        if kind == 'number':
            value = value.replace('_', '')
        result.append([kind, value])
    return sha(json.dumps(result, separators=(',', ':'), ensure_ascii=False).encode())


def declarations(ts: list[tuple], *, record: bool = False, enum: bool = False) -> list[dict]:
    """Lexical declarations, excluding procedure-local statements and bodies.

    Both conditional branches are potential contracts. This is intentionally not
    the compiler's semantic export resolver and never declares API equivalence.
    """
    result, at, exported = [], 0, True
    while at < len(ts):
        kind, name, line = ts[at]
        if name in {'#scope_file', '#scope_module', '#scope_export'}:
            exported = name == '#scope_export'
            at += 1
            continue
        if name in {'#if', '#else'}:
            end = at + 1
            while end < len(ts) and ts[end][1] not in {'{', ';'}:
                if ts[end][1] in {'(', '['}:
                    end = balanced(ts, end)
                else:
                    end += 1
            if end < len(ts) and ts[end][1] == '{':
                stop = balanced(ts, end)
                branches = declarations(ts[end + 1:stop - 1], record=record, enum=enum)
                result.extend(branches if exported or record else [])
                at = stop
            else:
                at = end + 1
            continue
        begin = at
        if name == 'using':
            at += 1
            if at < len(ts) and ts[at][1] == '#as':
                at += 1
            if at >= len(ts):
                break
            kind, name, line = ts[at]
        separator = at + 1
        if name == 'operator':
            while separator < len(ts) and ts[separator][1] not in {'::', ';', '{'}:
                separator += 1
            name = ''.join(t[1] for t in ts[at:separator])
        valid = (kind == 'id' and separator < len(ts)
                 and ts[separator][1] in {':', '::', ':='})
        if enum and kind == 'id' and separator < len(ts) and ts[separator][1] == ';':
            valid = True
        if not valid:
            if ts[at][1] in {'(', '[', '{'}:
                at = balanced(ts, at)
            else:
                at += 1
            continue
        first = separator + 1
        value = ts[first][1] if first < len(ts) else ''
        decl_kind = ('enum-member' if enum else 'field' if record and ts[separator][1] == ':'
                     else 'record' if value in {'struct', 'union'}
                     else 'enum' if value in {'enum', 'enum_flags'}
                     else 'procedure' if value == '(' else 'storage'
                     if ts[separator][1] in {':', ':='} else 'constant')
        cursor = separator if ts[separator][1] == ';' else first
        while cursor < len(ts) and ts[cursor][1] not in {';', '{'}:
            if ts[cursor][1] in {'(', '['}:
                cursor = balanced(ts, cursor)
            else:
                cursor += 1
        # Named result defaults can contain Type.{...} before the actual
        # procedure body. Retain that literal in the signature rather than
        # misclassifying its braces as an empty implementation.
        while (decl_kind == 'procedure' and cursor < len(ts)
               and ts[cursor][1] == '{' and cursor > first and ts[cursor - 1][1] == '.'):
            cursor = balanced(ts, cursor)
            while cursor < len(ts) and ts[cursor][1] not in {';', '{'}:
                cursor = balanced(ts, cursor) if ts[cursor][1] in {'(', '['} else cursor + 1
        # A storage/constant aggregate initializer is part of its public default,
        # whereas a procedure or record brace begins implementation/members.
        while (cursor < len(ts) and ts[cursor][1] == '{'
               and decl_kind not in {'record', 'enum', 'procedure'}):
            cursor = balanced(ts, cursor)
            while cursor < len(ts) and ts[cursor][1] not in {';', '{'}:
                cursor = balanced(ts, cursor) if ts[cursor][1] in {'(', '['} else cursor + 1
        head = ts[begin:cursor]
        item = {'name': name, 'kind': decl_kind, 'line': line,
                'header_sha256': contract_hash(head), 'members': [],
                'implementation_form': 'declaration'}
        if cursor < len(ts) and ts[cursor][1] == '{':
            stop = balanced(ts, cursor)
            body = ts[cursor + 1:stop - 1]
            if decl_kind in {'record', 'enum'}:
                item['members'] = declarations(body, record=True, enum=decl_kind == 'enum')
            elif decl_kind == 'procedure':
                item['implementation_form'] = 'source-body' if body else 'empty-body'
            cursor = stop
        elif any(t[1] == '#foreign' for t in head):
            item['implementation_form'] = 'foreign-declaration'
        elif any(t[1] in {'#compiler', '#intrinsic', '#elsewhere'} for t in head):
            item['implementation_form'] = 'compiler-declaration'
        item['contract_sha256'] = sha(json.dumps(
            [item['header_sha256'], [(m['name'], m['contract_sha256']) for m in item['members']]],
            separators=(',', ':')).encode())
        if exported or record:
            result.append(item)
        at = cursor + int(cursor < len(ts) and ts[cursor][1] == ';')
    return result


def module_roots(directory: Path) -> list[tuple[str, Path]]:
    if not directory.is_dir():
        return []
    roots = [(p.stem, p) for p in directory.glob('*.jai')]
    roots += [(p.parent.name, p) for p in directory.glob('*/module.jai')]
    return sorted(roots)


def source_contract(path: Path) -> dict:
    raw = path.read_bytes()
    metadata = scan_source(path)
    return {'path': path.relative_to(ROOT).as_posix(), 'sha256': sha(raw),
            'declarations': declarations(tokens(decode(raw))),
            'loads': metadata['loads'], 'imports': metadata['imports'],
            'module_parameters': metadata['module_parameters']}


def closure(entry: Path, allowed: Path, cache: dict) -> list[dict]:
    pending, seen, result = [entry], set(), []
    while pending:
        path = pending.pop().resolve()
        if path in seen or not path.is_file():
            continue
        if not path.is_relative_to(allowed.resolve()):
            # Authored Preload legitimately loads the authored prelude sibling.
            if not path.is_relative_to((ROOT / 'prelude').resolve()):
                raise ValueError(f'load escapes permitted source tree: {path}')
        seen.add(path)
        if path not in cache:
            cache[path] = source_contract(path)
        row = cache[path]
        result.append(row)
        for load in row['loads']:
            if load['resolved_path'] is not None:
                pending.append(Path(load['resolved_path']))
    return sorted(result, key=lambda row: row['path'])


def surfaces() -> tuple[list[dict], dict, list[dict]]:
    pins = {source.path.resolve(): source for source in inventory(ROOT)}
    cache, rows, support = {}, [], []
              ('maintained-openjai', ROOT / 'corpus/upstream/withlang-dev--open-jai/modules'),
              ('authored-default', ROOT / 'stdlib'),
              ('authored-legacy', ROOT / 'stdlib/legacy')]
    for origin, directory in groups:
        roots = module_roots(directory)
        preferred = {name: directory / f'{name}.jai' if (directory / f'{name}.jai').is_file() else entry
                     for name, entry in roots}
        for name, entry in roots:
            files = closure(entry, directory, cache)
            if not origin.startswith('authored'):
                for row in files:
                    pin = pins.get((ROOT / row['path']).resolve())
                    if pin is None or row['sha256'] != pin.expected_sha256:
                        raise ValueError(f'original input is not the verified pinned source: {row["path"]}')
            api = [dict(decl, source=row['path']) for row in files for decl in row['declarations']]
            owner = OWNERS.get(name, OWNERS.get(name.removesuffix('_Legacy'), 'unassigned'))
            rows.append({'module': name, 'origin': origin, 'family': owner,
                         'role': 'fixture' if OWNERS.get(name) == 'compatibility-fixtures' else 'library-surface',
                         'named_search_root_candidate_selected': entry == preferred[name],
                         'entrypoint': entry.relative_to(ROOT).as_posix(),
                         'source_files': [{'path': row['path'], 'sha256': row['sha256']} for row in files],
                         'module_parameters': [p for row in files for p in row['module_parameters']],
                         'contracts': api})
        claimed = {path['path'] for row in rows if row['origin'] == origin for path in row['source_files']}
        for path in sorted(directory.rglob('*.jai')):
            relative = path.relative_to(ROOT).as_posix()
            if relative in claimed:
                continue
            # The nested legacy tree has its own source surface.
            if origin == 'authored-default' and path.is_relative_to(ROOT / 'stdlib/legacy'):
                continue
            if path.resolve() not in cache:
                cache[path.resolve()] = source_contract(path)
            row = cache[path.resolve()]
            if not origin.startswith('authored'):
                pin = pins.get(path.resolve())
                if pin is None or row['sha256'] != pin.expected_sha256:
                    raise ValueError(f'unverified pinned support input: {relative}')
            fixture = bool({part.lower() for part in path.parts} & {'examples', 'example', 'tests', 'test'})
            support.append({'origin': origin, 'role': 'fixture' if fixture else 'unrooted-support-source',
                            'path': relative, 'sha256': row['sha256'], 'contracts': row['declarations']})
    return rows, cache, support


def compare(reference: dict, authored: dict | None) -> dict:
    actual = Counter((d['name'], d['contract_sha256']) for d in authored['contracts']) if authored else Counter()
    wanted = Counter((d['name'], d['contract_sha256']) for d in reference['contracts'])
    names = {name for name, _ in actual}
    missing = [{'name': name, 'contract_sha256': signature, 'count': count,
                'name_present_with_different_contract': name in names}
               for (name, signature), count in sorted((wanted - actual).items())]
    return {'module': reference['module'], 'reference_origin': reference['origin'],
            'authored_origin': authored['origin'] if authored else None,
            'expected_lexical_contracts': sum(wanted.values()),
            'identical_lexical_contracts': sum((wanted & actual).values()),
            'missing_or_different_contracts': missing,
            'semantic_api_equivalence': 'not-established', 'behavior_acceptance': 'not-established'}


def check_file(compiler: Path, path: Path, action: str, timeout: float) -> dict:
    environment = {key: value for key, value in os.environ.items() if not key.startswith('JAI_RS_')}
    environment.update(JAI_RS_STDLIB=str(ROOT / 'stdlib'), JAI_RS_PRELOAD=str(ROOT / 'stdlib/prelude/module.jai'),
                       JAI_RS_RUNTIME_SUPPORT='off')
    command = [str(compiler), action, str(path)]
    result = {'path': path.relative_to(ROOT).as_posix(), 'stage': action, 'command': command,
              'configured_environment': {k: v for k, v in environment.items() if k.startswith('JAI_RS_')},
              'status': 'failed', 'exit_code': None, 'diagnostic': ''}
    try:
        child = subprocess.run(command, cwd=ROOT, env=environment, stdout=subprocess.DEVNULL,
                               stderr=subprocess.PIPE, timeout=timeout)
        result.update(status='passed' if child.returncode == 0 else 'failed', exit_code=child.returncode)
        lines = child.stderr.decode(errors='replace').splitlines()
        result['diagnostic'] = next((line for line in lines if line.strip()), '')[:2000]
    except subprocess.TimeoutExpired:
        result.update(status='timeout')
    return result


def compact_report(result: dict, report_path: Path) -> dict:
    """Commit small, reviewable coverage facts while retaining the full local report."""
    return {
        'format': 1,
        'generated_at_utc': datetime.now(timezone.utc).isoformat(),
        'source_policy': result['source_policy'],
        'limitations': result['limitations'],
        'source_snapshot_valid': not (result['summary']['changed_during_inventory']
                                      or result['summary']['family_reports_changed_during_inventory']),
        'semantic_api_equivalence': 'not-established',
        'whole_library_behavior_acceptance': 'not-established',
        'compiler_sha256': result['compiler_sha256'],
        'compiler_build_inputs_verified': result['compiler_build_inputs_verified'],
        'inventory_report': {'path': report_path.relative_to(ROOT).as_posix(),
                             'sha256': sha(report_path.read_bytes()), 'bytes': report_path.stat().st_size},
        'summary': result['summary'],
        'modules': [{key: row[key] for key in ('module', 'origin', 'family', 'role', 'entrypoint',
                                               'named_search_root_candidate_selected')}
                    | {'source_files': len(row['source_files']), 'lexical_contract_occurrences': len(row['contracts'])}
                    for row in result['surfaces']],
        'comparisons': [{key: row[key] for key in ('module', 'reference_origin', 'authored_origin',
                                                  'expected_lexical_contracts', 'identical_lexical_contracts')}
                        | {'missing_or_different_lexical_contracts': sum(
                            item['count'] for item in row['missing_or_different_contracts'])}
                        for row in result['comparisons']],
        'family_reports': [{key: row[key] for key in ('path', 'sha256')} for row in result['family_reports']],
        'source_check_failures': [row for row in result['source_checks'] if row['status'] != 'passed'],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT / 'artifacts/stdlib-rewrite/api-inventory.json.gz')
    parser.add_argument('--summary-output', type=Path, default=ROOT / 'stdlib/api-coverage.json')
    parser.add_argument('--compiler', type=Path, help='already frozen repository-built CLI for source-only parsing')
    parser.add_argument('--check', action='store_true', help='also check authored module entrypoints')
    parser.add_argument('--timeout', type=float, default=10)
    options = parser.parse_args()
    before_paths = reviewed_paths()
    before_hashes = {path: sha(path.read_bytes()) for path in before_paths}
    rows, cache, support = surfaces()
    authored = {(row['module'], row['origin']): row for row in rows
                if row['origin'].startswith('authored') and row['named_search_root_candidate_selected']}
    references = [row for row in rows if not row['origin'].startswith('authored') and row['role'] != 'fixture']
    comparisons = [compare(row, authored.get((row['module'], 'authored-default'))) for row in references]
    comparisons.extend(compare(row, authored[(row['module'], 'authored-legacy')])
                       for row in references if (row['module'], 'authored-legacy') in authored)
    reports = []
    report_hashes = {}
    for path in sorted((ROOT / 'stdlib/.coverage').glob('*.json')):
        raw = path.read_bytes()
        report_hashes[path.resolve()] = sha(raw)
        reports.append({'path': path.relative_to(ROOT).as_posix(), 'sha256': sha(raw),
                        'report': json.loads(raw)})
    checks, compiler_hash = [], None
    if options.compiler:
        compiler = options.compiler.resolve()
        if not compiler.is_relative_to((ROOT / 'target/standard-library-snapshots').resolve()) or compiler.name != 'jai-rs':
            raise ValueError('select a frozen repository-built standard-library snapshot CLI')
        compiler_hash = sha(compiler.read_bytes())
        sources = sorted(path for path in cache if path.is_relative_to((ROOT / 'stdlib').resolve()))
        with ThreadPoolExecutor(max_workers=4) as workers:
            checks.extend(workers.map(lambda path: check_file(compiler, path, 'parse', options.timeout), sources))
            if options.check:
                entries = [ROOT / row['entrypoint'] for row in rows if row['origin'] == 'authored-default']
                checks.extend(workers.map(lambda path: check_file(compiler, path, 'check-library', options.timeout), entries))
        if sha(compiler.read_bytes()) != compiler_hash:
            raise ValueError('frozen compiler bytes changed during source checks')
    source_hashes = {path: row['sha256'] for path, row in cache.items()}
    # A missing/added source, a changed source, or a rewritten family receipt
    # invalidates the combined snapshot just as an edited module body does.
    changed = sorted(set(changed_paths(before_hashes, reviewed_paths())) | {
        path.relative_to(ROOT).as_posix() for path, digest in source_hashes.items()
        if before_hashes.get(path) != digest
    })
    reports_changed = changed_paths(report_hashes, {
        path.resolve() for path in (ROOT / 'stdlib/.coverage').glob('*.json')
    })
    production_sources = [row for path, row in cache.items()
                          if path.is_relative_to(ROOT / 'stdlib')
                          and not path.is_relative_to(ROOT / 'stdlib/legacy')
                          and not path.is_relative_to(ROOT / 'stdlib/tests')]
    result = {'format': 1, 'source_policy': 'maintained-newer-api-priority-historical-compatibility-separate',
              'limitations': ['lexical declaration inventory, not semantic selected exports',
                              'all literal load/conditional branches inventoried',
                              'matching hashes establish contract spelling only',
                              'source bodies and foreign declarations do not establish working behavior'],
              'summary': {'module_surfaces': dict(Counter(row['origin'] for row in rows)),
                          'named_module_counts': dict(Counter(row['origin'] for row in rows
                              if row['named_search_root_candidate_selected'])),
                          'unrooted_sources': dict(Counter(f"{row['origin']}:{row['role']}" for row in support)),
                          'authored_implementation_forms': dict(Counter(d['implementation_form'] for row in rows
                              if row['origin'] == 'authored-default' for d in row['contracts'])),
                          'authored_unique_production_files': len(production_sources),
                          'authored_unique_production_implementation_forms': dict(Counter(
                              d['implementation_form'] for row in production_sources
                              for d in nested_declarations(row['declarations']))),
                          'unassigned_modules': sorted({row['module'] for row in rows if row['family'] == 'unassigned'}),
                          'source_checks': dict(Counter(f"{row['stage']}:{row['status']}" for row in checks)),
                          'changed_during_inventory': changed,
                          'family_reports_changed_during_inventory': reports_changed},
              'compiler_sha256': compiler_hash, 'compiler_build_inputs_verified': False,
              'surfaces': rows, 'support_sources': support, 'comparisons': comparisons,
              'family_reports': reports, 'source_checks': checks}
    options.output.parent.mkdir(parents=True, exist_ok=True)
    staged = options.output.with_name(options.output.name + '.tmp')
    opener = gzip.open if options.output.suffix == '.gz' else open
    with opener(staged, 'wt', encoding='utf-8') as output:
        # Stream the public contract report instead of materializing another
        # full JSON string while parallel integration owns the memory budget.
        json.dump(result, output, separators=(',', ':'))
        output.write('\n')
    staged.replace(options.output)
    compact = compact_report(result, options.output.resolve())
    options.summary_output.parent.mkdir(parents=True, exist_ok=True)
    staged_summary = options.summary_output.with_name(options.summary_output.name + '.tmp')
    staged_summary.write_text(json.dumps(compact, indent=2) + '\n')
    staged_summary.replace(options.summary_output)
    print(json.dumps(result['summary'], sort_keys=True))
    return 1 if changed or reports_changed or any(row['status'] != 'passed' for row in checks) else 0


if __name__ == '__main__':
    raise SystemExit(main())
