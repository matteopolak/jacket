#!/usr/bin/env python3
"""Compare the pre-fix exact-buffer failure with sanitized native suites.

Run `PYTHONPATH=tools python3 tools/meow/validate.py` on an AES-capable Linux x64 host after
building jaic. Every compiler invocation and run is saved under target/meow-evidence.
"""
import json
import re
from pathlib import Path
import subprocess

from toolkit.repo import ROOT, cargo_binary
from repo_output import output_dir

OUT = output_dir('meow-evidence')
OUT.mkdir(parents=True, exist_ok=True)
JAIC = cargo_binary('jaic')
FINALIZER = ROOT / 'stdlib/internal/hash/meow/finalize.jai'
BASE = 'd072effb268b18d2b16ebf27d69d36c14859909d'


def command(args, name, expect_failure=False):
    result = subprocess.run([str(a) for a in args], cwd=ROOT, capture_output=True)
    (OUT / (name + '.stdout')).write_bytes(result.stdout)
    (OUT / (name + '.stderr')).write_bytes(result.stderr)
    (OUT / (name + '.json')).write_text(json.dumps({'args': [str(a) for a in args], 'returncode': result.returncode}))
    if expect_failure:
        # Expanded instructions inherit their callsite's source location.
        test = ROOT / 'stdlib/meow_hash/tests/hash.jai'
        call_line = next(i for i, line in enumerate(test.read_text().splitlines(), 1)
                         if 'MeowHash(MeowDefaultSeed, xx v.count' in line)
        callsite = ('stdlib/meow_hash/tests/hash.jai:' + str(call_line) + ':').encode()
        assert (result.returncode != 0 and b'heap-buffer-overflow' in result.stderr
                and re.search(rb'READ of size (8|16) ', result.stderr)
                and re.search(rb'#0 [^\n]* in check_vector[^\n]*' + re.escape(callsite),
                              result.stderr)), result.stderr.decode()
    else:
        assert result.returncode == 0, result.stderr.decode()


def compile_test(test, name, opt, sanitize):
    exe = OUT / name
    args = [JAIC, ROOT / test, '-' + opt, '-exe', exe]
    if sanitize:
        args += ['-sanitize', 'address']
    if Path(test).stem == 'exact' or name == 'baseline-exact':
        args += ['-emit_ir', OUT / (name + '.ll')]
    command(args, name + '-build')
    return exe


assert 'aes' in Path('/proc/cpuinfo').read_text().split(), 'Native AES required'
candidate = FINALIZER.read_bytes()
try:
    FINALIZER.write_bytes(subprocess.check_output(['git', 'show', BASE + ':stdlib/internal/hash/meow/finalize.jai'], cwd=ROOT))
    exe = compile_test('stdlib/meow_hash/tests/hash.jai', 'baseline-exact', 'O0', True)
    command([exe], 'baseline-exact-run', expect_failure=True)
finally:
    FINALIZER.write_bytes(candidate)

for opt in ['O0', 'O2']:
    for sanitized in [False, True]:
        for test in ['stdlib/meow_hash/tests/hash.jai', 'stdlib/meow_hash/tests/vectors.jai',
                     'stdlib/meow_hash/tests/exact.jai', 'stdlib/Extensions/Meow_Portable/tests/portable.jai']:
            name = Path(test).stem + '-' + opt + ('-asan' if sanitized else '')
            exe = compile_test(test, name, opt, sanitized)
            command([exe], name + '-run')

for opt in ['O0', 'O2']:
    name = 'asan-prefix-' + opt
    exe = compile_test('stdlib/meow_hash/tests/driven/asan-prefix.jai', name, opt, True)
    command([exe], name + '-run')
