#!/usr/bin/env python3
"""Build and check both TOML adapters against every pinned profile case, without exclusions.

    python3 tools/stdlib/toml/run.py --jaic target/debug/jaic --cache /tmp/toml-test --report /tmp/toml.json

Local callers must use the repository's shared heavy-job gate. Cache downloads are verified
before extraction/execution; report IDs must exactly match the pinned profile inventory.
"""
from __future__ import annotations

import argparse
from collections import Counter
import gzip
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import platform
import shutil
import struct
import subprocess
import tarfile
import urllib.request

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
PIN = json.loads((HERE / 'pin.json').read_text())


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def fetch(path: Path, url: str, digest: str) -> bytes:
    if not path.is_file():
        request = urllib.request.Request(url, headers={'User-Agent': 'jacket-toml-conformance'})
        data = urllib.request.urlopen(request, timeout=60).read()
        if sha256(data) != digest:
            raise RuntimeError(f'checksum mismatch for {url}')
        path.write_bytes(data)
    data = path.read_bytes()
    if sha256(data) != digest:
        raise RuntimeError(f'cached checksum mismatch: {path}')
    return data


def runner_asset(override=None, *, system=None, machine=None):
    """Choose a published controller explicitly; never silently substitute another CPU.

        asset = runner_asset('windows-amd64.exe')  # an x64 controller can drive a native ARM adapter
    """
    system = (system or platform.system()).lower()
    machine = (machine or platform.machine()).lower()
    cpu = {'x86_64': 'amd64', 'amd64': 'amd64', 'aarch64': 'arm64', 'arm64': 'arm64'}.get(machine, machine)
    asset = override or f'{system}-{cpu}' + ('.exe' if system == 'windows' else '')
    if asset not in PIN['runner_sha256']:
        raise ValueError(f'no pinned runner asset for {asset}; select an explicit compatible published controller')
    return asset


def windows_adapter_cpu(path):
    """Read the parser executable's COFF machine, independently of controller/Python emulation.

        assert windows_adapter_cpu(adapter) == expected_cpu
    """
    data = path.read_bytes()
    if len(data) < 64 or data[:2] != b'MZ':
        raise ValueError('adapter is not a PE executable')
    offset = struct.unpack_from('<I', data, 0x3c)[0]
    if offset > len(data) - 6 or data[offset:offset + 4] != b'PE\0\0':
        raise ValueError('adapter has an invalid PE header')
    machine = struct.unpack_from('<H', data, offset + 4)[0]
    if machine not in (0x8664, 0xaa64):
        raise ValueError('adapter is not x64 or ARM64 native PE')
    return {0x8664: 'x64', 0xaa64: 'arm64'}[machine]


def prepare(cache: Path, asset_override=None) -> tuple[Path, dict[str, str]]:
    cache.mkdir(parents=True, exist_ok=True)
    asset = runner_asset(asset_override)
    url = f'https://github.com/toml-lang/toml-test/releases/download/{PIN["release"]}/toml-test-{PIN["release"]}-{asset}.gz'
    packed = fetch(cache / f'{asset}.gz', url, PIN['runner_sha256'][asset])
    runner = cache / ('toml-test.exe' if os.name == 'nt' else 'toml-test')
    runner.write_bytes(gzip.decompress(packed))
    runner.chmod(0o755)
    archive = fetch(cache / 'suite.tar.gz', **{'url': PIN['archive']['url'], 'digest': PIN['archive']['sha256']})
    files: dict[str, bytes] = {}
    with tarfile.open(fileobj=io.BytesIO(archive)) as stream:
        for entry in stream.getmembers():
            parts = PurePosixPath(entry.name).parts
            if len(parts) < 2 or not entry.isfile():
                continue
            relative = PurePosixPath(*parts[1:])
            if '..' in relative.parts:
                raise RuntimeError('archive traversal')
            if relative.parts[0] == 'tests' or str(relative) == 'LICENSE':
                files[str(relative)] = stream.extractfile(entry).read()
    if files['LICENSE'] != (HERE / 'LICENSE-toml-test').read_bytes():
        raise RuntimeError('pinned license differs')
    inventory = files['tests/files-toml-1.1.0'].decode().splitlines()
    expected: dict[str, str] = {}
    for name in inventory:
        if not name:
            continue
        path = PurePosixPath(name)
        if path.suffix == '.toml':
            case = str(path.with_suffix(''))
        elif path.suffix == '.json' and path.parts[0] == 'valid':
            case = 'encoder/' + str(PurePosixPath(*path.parts[1:]).with_suffix(''))
        else:
            raise RuntimeError(f'unexpected inventory entry: {name}')
        if case in expected:
            raise RuntimeError(f'duplicate inventory ID: {case}')
        expected[case] = sha256(files['tests/' + name])
    counts = Counter(case.split('/')[0] for case in expected)
    if counts != Counter(PIN['expected']):
        raise RuntimeError(f'inventory counts changed: {counts}')
    (cache / 'inventory.json').write_text(json.dumps(expected, indent=2, sort_keys=True) + '\n')
    return runner, expected


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--jaic', type=Path)
    parser.add_argument('--adapter', type=Path, help='use a prebuilt native adapter instead of building')
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--prepare-only', action='store_true')
    parser.add_argument('--runner-asset', choices=sorted(PIN['runner_sha256']), help='explicit published controller when host CPU has no release asset')
    parser.add_argument('--expected-windows-cpu', choices=('x64', 'arm64'), help='require native parser PE machine, independently of controller architecture')
    parser.add_argument('--record-statements', action='store_true', help='also exercise the optional source projection')
    args = parser.parse_args()
    cache = args.cache.resolve()
    runner, expected = prepare(cache, args.runner_asset)
    if args.prepare_only:
        print(json.dumps({'release': PIN['release'], 'profile': PIN['profile'], 'cases': len(expected)}))
        return 0
    suffix = '.exe' if os.name == 'nt' else ''
    adapter = args.adapter.resolve() if args.adapter else cache / f'decoder{suffix}'
    sources = [*sorted((ROOT / 'stdlib').rglob('*.jai')), HERE / 'adapter.jai']
    snapshot = {str(path.relative_to(ROOT)): sha256(path.read_bytes()) for path in sources} if not args.adapter else None
    if not args.adapter:
        if not args.jaic:
            parser.error('--jaic or --adapter is required')
        env = dict(os.environ, JAIC_STDLIB=str(ROOT / 'stdlib'))
        command = [str(args.jaic.resolve()), str(HERE / 'adapter.jai'), '-O0', '-exe', str(adapter), '-output_path', str(cache / 'build')]
        subprocess.run(command, cwd=ROOT, env=env, check=True)
        if snapshot != {str(path.relative_to(ROOT)): sha256(path.read_bytes()) for path in sources}:
            raise RuntimeError("source changed during adapter build")
    encoder = cache / f'encoder{suffix}'
    shutil.copyfile(adapter, encoder)
    encoder.chmod(0o755)
    command = [str(runner), 'test', '-toml=1.1', f'-decoder={adapter}', f'-encoder={encoder}', '-json', '-v', '-parallel=2']
    adapter_cpu = windows_adapter_cpu(adapter) if os.name == 'nt' or args.expected_windows_cpu else None
    if args.expected_windows_cpu and adapter_cpu != args.expected_windows_cpu:
        raise RuntimeError('adapter PE machine differs from required native parser CPU')
    env = dict(os.environ, TOML_CONFORMANCE_STATEMENTS="1" if args.record_statements else "0")
    result = subprocess.run(command, capture_output=True, text=True, timeout=600, env=env)
    report = json.loads(result.stdout)
    report['host'] = {'system': platform.system(), 'machine': platform.machine()}
    report['runner_asset'] = runner_asset(args.runner_asset)
    report['adapter_windows_cpu'] = adapter_cpu
    report['runner_harness_sha256'] = sha256(Path(__file__).read_bytes())
    report['record_statements'] = args.record_statements
    report['pin'] = PIN
    report['adapter_sha256'] = sha256(adapter.read_bytes())
    report['source_sha256'] = snapshot
    report['compiler_sha256'] = sha256(args.jaic.read_bytes()) if not args.adapter else None
    report['inventory_sha256'] = sha256(json.dumps(expected, sort_keys=True).encode())
    report['runner_exit_code'] = result.returncode
    report['runner_stderr'] = result.stderr
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + '\n')
    outcomes = report['tests']
    actual = Counter(row['path'] for row in outcomes)
    if actual != Counter({case: 1 for case in expected}):
        raise RuntimeError('runner outcome IDs differ from complete pinned inventory')
    if report['skipped'] or any(row['skipped'] for row in outcomes):
        raise RuntimeError('conformance exclusions are forbidden')
    if report['toml'] != '1.1.0' or not report['version'].endswith(PIN['release']):
        raise RuntimeError('runner/profile mismatch')
    summary = {key: report[key] for key in report if key.startswith(('passed_', 'failed_'))}
    print(json.dumps(summary, sort_keys=True))
    return int(result.returncode != 0 or any(row['failure'] for row in outcomes))


if __name__ == '__main__':
    raise SystemExit(main())
