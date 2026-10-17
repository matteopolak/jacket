#!/usr/bin/env python3
"""Run spec-derived heap programs and independent C safety/ABI evidence.

Requires our source-built compiler/formatter and freshly built native libraries.
Uses only NEW producer sources through the ordinary bindings staging boundary.
This scope does not replace the full public inventory/configuration verifier.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
from contextlib import ExitStack

from config import RPMALLOC_SDK_BUILD_OPTIONS
from rpmalloc_profiles import PROFILES
from prepare_rpmalloc import prepare
from verify import select_stdlib, place_private_unit, ROOT
from configure_rpmalloc import select_for_verification


def run_check(command, env):
    """Retain a failed launch as evidence and continue the remaining checks."""
    command = [str(x) for x in command]
    try:
        result = subprocess.run(command, env=env, capture_output=True, text=True, timeout=120)
        return dict(command=command, exit=result.returncode, stdout=result.stdout, stderr=result.stderr)
    except subprocess.TimeoutExpired:
        return dict(command=command, exit=None, stderr='Timeout after 120 seconds')
    except OSError as error:
        return dict(command=command, exit=None, stderr=f'Could not execute check: {error}')


def verify(cleanup):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--formatter', type=Path, required=True)
    parser.add_argument('--native-libs', type=Path, required=True)
    parser.add_argument('--stdlib', type=Path,
                        help='Use this actual production library without source overlays')
    parser.add_argument('--macos-both-cpus', action='store_true',
                        help='Execute both Mac targets; requires universal native archives and x64 execution support')
    parser.add_argument('--rpmalloc-first-class-heaps', action='store_true')
    parser.add_argument('--report', type=Path, default=ROOT/'.work/rpmalloc-heaps.json')
    args = parser.parse_args()
    if args.macos_both_cpus and platform.system() != 'Darwin':
        parser.error('--macos-both-cpus requires a Mac host')
    compiler = args.compiler.resolve(strict=True)
    formatter = args.formatter.resolve(strict=True)
    libraries = args.native_libs.resolve(strict=True)
    os.chdir(ROOT)
    work = ROOT/'.work/runs'/('canonical-heaps-'+str(os.getpid()))
    work.mkdir(parents=True)
    stdlib = select_stdlib(work, production=args.stdlib)
    select_for_verification(stdlib, args.rpmalloc_first_class_heaps, args.stdlib is not None)
    facade_sources = sorted((stdlib/'rpmalloc').glob('*.jai'))

    unit = place_private_unit('rpmalloc-heap-marshalling',
                              ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-heap-marshalling.jai', stdlib, {}, cleanup)
    cache_unit = place_private_unit('rpmalloc-no-thread-cache',
        ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-no-thread-cache.jai', stdlib, {}, cleanup)
    prepared = prepare(ROOT/'.work/vendor/rpmalloc', work/'rpmalloc-source')
    (ROOT/'.work/tmp').mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, TMPDIR=str(ROOT/'.work/tmp'), JAIC_STDLIB=str(stdlib),
               JAIC_NATIVE_LIBS=str(libraries),
               JAIC_TEST_RPMALLOC_FIRST_CLASS_HEAPS='true' if args.rpmalloc_first_class_heaps else 'false')
    programs = [ROOT/'stdlib/Bindings_Generator/tests/driven/sdk'/name for name in
                ('rpmalloc-public-heaps.jai', 'rpmalloc-public-heap-lifecycle.jai',
                 'rpmalloc-public-configurable.jai', 'rpmalloc-public-global-cache.jai', 'rpmalloc-public-allocator-proc.jai',
                 'rpmalloc-public-builder-dump.jai', 'rpmalloc-public-dump-failure.jai')]+[unit,
                     place_private_unit('rpmalloc-native-error-callback',
                                        ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-native-error-callback.jai', stdlib, {}, cleanup)]
    if args.rpmalloc_first_class_heaps:
        programs += [ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-public-raw-heaps.jai',
                     ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-public-raw-heap-lifecycle.jai',
                     place_private_unit('rpmalloc-raw-heap-marshalling',
                                        ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-raw-heap-marshalling.jai', stdlib, {}, cleanup)]
    else:
        programs += [place_private_unit('rpmalloc-raw-base-marshalling',
                                        ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-raw-base-marshalling.jai', stdlib, {}, cleanup)]
    programs += [cache_unit]
    commands = []
    for program in programs:
        for target_os in ('macos', 'linux', 'windows'):
            for cpu in ('x64', 'arm64'):
                commands.append([compiler, program, '-check', '-no_dce', '-os', target_os, '-cpu', cpu])
        native_targets = (['x86_64-apple-darwin', 'aarch64-apple-darwin'] if args.macos_both_cpus
                          else [None])
        for index, target in enumerate(native_targets):
            exe = work/(program.stem+'-'+str(index))
            commands.append([compiler, program, *(['-target', target] if target else []), '-exe', exe])
            commands.append([exe])
    for profile in PROFILES:
        options = profile['options']
        for name in ('rpmalloc_heap_safety', 'rpmalloc_sdk_page_granularity', 'rpmalloc_snapshot_safety'):
            exe = work/(name+'-'+profile['sdk'])
            commands.append(['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', '-O1', '-g',
                             '-fsanitize=address,undefined', '-fno-sanitize-recover=all', '-isystem', prepared,
                             *['-D'+k+'='+str(v) for k, v in options.items()],
                             ROOT/'stdlib/Bindings_Generator/tests/driven/sdk'/(name+'.c'), '-o', exe])
            commands.append([exe])
    resource = subprocess.run(['clang', '-print-resource-dir'], check=True, capture_output=True,
                              text=True).stdout.strip()
    for target in ('x86_64-apple-darwin', 'aarch64-apple-darwin', 'x86_64-linux-gnu',
                   'aarch64-linux-gnu', 'x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc'):
        # Global caching does not change record fields; both thread-cache
        # layouts must independently compile for every target and public shape.
        for profile in PROFILES:
            if not profile["global_cache"]:
                continue
            definitions = {key: profile["options"][key] for key in
                           ("ENABLE_THREAD_CACHE", "ENABLE_STATISTICS", "BINDING_RPMALLOC_RAW",
                            "BINDING_RPMALLOC_PUBLIC_HEAPS")}
            commands.append(["clang", "-target", target, "-std=c11", "-ffreestanding", "-nostdinc",
                             "-isystem", resource+"/include", "-fsyntax-only",
                             *["-D"+key+"="+str(value) for key, value in definitions.items()],
                             ROOT/"stdlib/internal/bindings/rpmalloc-heaps/public-layout.h"])
    scopes = ['stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-cache-exercise.jai',
              'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-no-thread-cache.jai',
              'stdlib/Bindings_Generator/tests/driven/sdk/generated', 'stdlib/rpmalloc', 'stdlib/internal/bindings/rpmalloc-heaps',
              *[str(p.relative_to(ROOT)) for p in programs],
              'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-heap-marshalling.jai',
              'stdlib/Bindings_Generator/tests/driven/sdk/generated/rpmalloc-heap-facade-layout.jai',
              'stdlib/internal/bindings/rpmalloc-build',
              'stdlib/internal/bindings/rpmalloc-sdk', 'stdlib/internal/bindings/rpmalloc-callbacks',
              'stdlib/internal/bindings/rpmalloc-checked']
    if args.rpmalloc_first_class_heaps:
        scopes += ['stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-raw-heap-marshalling.jai',
                   'stdlib/Bindings_Generator/tests/driven/sdk/generated/rpmalloc-raw-heap-facade-layout.jai']
    else:
        scopes += ['stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-raw-base-marshalling.jai',
                   'stdlib/Bindings_Generator/tests/driven/sdk/generated/rpmalloc-raw-base-heap-facade-layout.jai']
    commands += [[formatter, '--check', '--config', 'jaifmt.toml', *scopes],
                 [compiler.with_name('jailint'), '-D', 'warnings', '--config', 'jailint.toml',
                  *facade_sources,
                  *[stdlib/'internal/bindings'/name for name in
                    ('rpmalloc-heaps', 'rpmalloc-sdk', 'rpmalloc-callbacks', 'rpmalloc-checked', 'rpmalloc-build')]],
                 ['clang-format', '--style=LLVM', '--dry-run', '--Werror',
                  'stdlib/internal/bindings/rpmalloc-heaps/bridge.c',
                  'stdlib/internal/bindings/rpmalloc-heaps/heap-snapshot.inc',
                  'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc_heap_safety.c',
                  'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc_sdk_page_granularity.c',
                  'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc_snapshot_safety.c'],
                 ['python3', 'tools/native_bindings/generate.py', '--check', '--formatter', formatter],
                 ['python3', 'tools/native_bindings/vendor_rpmalloc.py', '--check', '--formatter', formatter],
                 ['python3', '-m', 'unittest', 'discover', '-s', 'tools/native_bindings', '-p', 'test_*.py']]
    # Staged private units are standalone programs, not newly imported modules.
    # Explicit roots preserve their context additions and check every body.
    commands += [[compiler.with_name('jailint'), '-D', 'warnings', '--config', 'jailint.toml',
                  '--context', program, program] for program in programs]
    for target_os in ('macos', 'linux', 'windows'):
        for cpu in ('x64', 'arm64'):
            commands.append(dict(argv=[compiler, ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/negative/rpmalloc-dump-wrong-builder.jai',
                                       '-check', '-no_dce', '-os', target_os, '-cpu', cpu],
                                 expect_failure='dump_statistics requires Basic.String_Builder',
                                 scope='expected_public_type_rejection'))
    for name, diagnostic in [('rpmalloc-invalid-mode', 'invalid rpmalloc Allocator_Mode'),
                             ('rpmalloc-destroy-default', 'the default rpmalloc heap cannot be destroyed')]:
        for index, target in enumerate(native_targets):
            exe = work/(name+'-'+str(index))
            commands.append([compiler, ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/negative'/(name+'.jai'),
                             *(['-target', target] if target else []), '-exe', exe])
            commands.append(dict(argv=[exe], expect_failure=diagnostic, scope='expected_public_assertion'))
    report = dict(scope='Public heap behavior/configurations; separate private ABI/native counters and host C sanitizers',
                  stdlib=str(stdlib), production_library=args.stdlib is not None,
                  first_class_heaps=args.rpmalloc_first_class_heaps,
                  private_configuration_sha256=hashlib.sha256(
                      (stdlib/'internal/bindings/rpmalloc-build/module.jai').read_bytes()).hexdigest(),
                  compiler_sha256=hashlib.sha256(compiler.read_bytes()).hexdigest(),
                  native_artifacts={p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                                    for p in libraries.glob('lib*rpmalloc*') if p.is_file()},
                  prepared_sdk=json.loads((prepared/'provenance.json').read_text()),
                  sources={str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
                           for p in [*programs,
                                     *sorted((ROOT/'stdlib/rpmalloc').glob('*.jai')),
                                     *sorted((ROOT/'stdlib/internal/bindings/rpmalloc-heaps').iterdir()),
                                     ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc_heap_safety.c',
                                     ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc_sdk_page_granularity.c',
                                     ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc_snapshot_safety.c',
                                     ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-cache-exercise.jai',
                                     Path(__file__).resolve(), ROOT/'tools/native_bindings/prepare_rpmalloc.py']
                           if p.is_file()},
                  checks=[])
    args.report.parent.mkdir(parents=True, exist_ok=True)
    for command in commands:
        expectation = command if isinstance(command, dict) else None
        row = run_check(expectation['argv'] if expectation else command, env)
        if expectation:
            observed = row['exit']
            row.update(observed_exit=observed, expected_failure=expectation['expect_failure'],
                       scope=expectation['scope'])
            row['exit'] = 0 if (observed is not None and observed != 0
                               and expectation['expect_failure'] in row.get('stderr', '')) else 1
        report['checks'].append(row)
        args.report.write_text(json.dumps(report, indent=2)+'\n')
        print(row['exit'], ' '.join(row['command'][:5]), flush=True)
    raise SystemExit(any(row['exit'] != 0 for row in report['checks']))


def main():
    with ExitStack() as cleanup:
        verify(cleanup)


if __name__ == '__main__':
    main()
