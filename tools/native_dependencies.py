#!/usr/bin/env python3
"""Pinned native dependency evidence and source-only rebuilds; never a linker allowlist."""
from __future__ import annotations

import argparse
from dataclasses import asdict, dataclass
from enum import Enum
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import subprocess
import tempfile

from check_corpus import ROOT
from inventory_corpus_features import decode, mask_noncode


class DependencyKind(str, Enum):
    SYSTEM = 'system'
    LOCAL = 'local'


@dataclass(frozen=True)
class SourceEvidence:
    path: str
    sha256: str
    line: int
    name: str
    kind: DependencyKind
    options: tuple[str, ...]


@dataclass(frozen=True)
class Fingerprint:
    path: str
    sha256: str


@dataclass(frozen=True)
class RebuildRecipe:
    repository: str
    revision: str
    source_path: str
    source_sha256: str
    documentation: str


@dataclass(frozen=True)
class BuildReceipt:
    recipe: RebuildRecipe
    target: str
    compiler: Fingerprint
    archiver: Fingerprint
    inputs: tuple[Fingerprint, ...]
    artifact: Fingerprint
    wrapper_sha256: str
    flags: tuple[str, ...]
    format: int = 1
    kind: str = 'source-rebuild-evidence'
    link_authority: bool = False
    limitations: tuple[str, ...] = (
        'Build/archive only: no dependency object or library loaded or executed.',
        'Corpus binding version/layout/symbol compatibility and runtime remain unverified.',
    )


VMA = RebuildRecipe(
    'GPUOpen-LibrariesAndSDKs/VulkanMemoryAllocator',
    '1d8f600fd424278486eade7ed3e877c99f0846b1', 'include/vk_mem_alloc.h',
    '90ce12fc4a2466235a09ae02905dd0c13aee80c1bbf11b331ab61230c2ceb112',
    'https://gpuopen-librariesandsdks.github.io/VulkanMemoryAllocator/html/quick_start.html',
)
VMA_WRAPPER = '#define VMA_IMPLEMENTATION\n#include "vk_mem_alloc.h"\n'
VMA_FLAGS = ('-std=c++17', '-O2', '-fPIC', '-DVMA_VULKAN_VERSION=1003000',
             '-DVMA_STATIC_VULKAN_FUNCTIONS=1', '-DVMA_DYNAMIC_VULKAN_FUNCTIONS=1')
VMA_VIRTUAL_FLAGS = ('-std=c++17', '-O2', '-fPIC', '-DVMA_VULKAN_VERSION=1003000',
                     '-DVMA_STATIC_VULKAN_FUNCTIONS=0', '-DVMA_DYNAMIC_VULKAN_FUNCTIONS=0')


def sha256(path: Path) -> str:
    with path.open('rb') as stream:
        digest = hashlib.sha256()
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(chunk)
        return digest.hexdigest()


def protected_roots(root: Path) -> tuple[Path, ...]:
    return tuple((root / name).resolve() for name in ('reference', 'vendor', 'corpus/upstream'))


def outside_inputs(path: Path, root: Path = ROOT) -> Path:
    resolved = path.resolve()
    if any(resolved.is_relative_to(base) for base in protected_roots(root)):
        raise ValueError(f'native dependency path is inside protected input source: {path}')
    return resolved


def fingerprint(path: Path, root: Path = ROOT) -> Fingerprint:
    path = outside_inputs(path, root)
    if not path.is_file():
        raise ValueError(f'expected a regular file: {path}')
    return Fingerprint(str(path), sha256(path))


def declarations(raw: str) -> tuple[SourceEvidence, ...]:
    # Mask first, then read the literal at each real directive position. Target
    # branches remain evidence candidates: this scanner is not a Jai evaluator.
    code = mask_noncode(raw)
    found = []
    for match in re.finditer(r'#(?:system_library|library)\b', code):
        literal = re.match(r'#(system_library|library)((?:\s*,\s*[A-Za-z_]\w*)*)\s*"([^"\r\n]+)"', raw[match.start():])
        if not literal:
            continue
        options = tuple(re.findall(r'[A-Za-z_]\w*', literal[2]))
        kind = DependencyKind.SYSTEM if literal[1] == 'system_library' or 'system' in options else DependencyKind.LOCAL
        found.append(SourceEvidence('', '', raw.count('\n', 0, match.start()) + 1, literal[3], kind, options))
    return tuple(found)


def corpus_evidence(root: Path = ROOT) -> list[dict]:
    manifest = json.loads((root / 'corpus/upstreams.json').read_text())
    if manifest['format'] != 1:
        raise ValueError('unsupported upstream manifest')
    projects = []
    for project in manifest['projects']:
        evidence = []
        for item in project['files']:
            if not item['path'].endswith('.jai'):
                continue
            relative = Path(item['path'])
            if relative.is_absolute() or '..' in relative.parts:
                raise ValueError('unsafe corpus source path')
            base = root / 'corpus/upstream' / project['repository'].replace('/', '--')
            path = base / relative
            if not path.resolve().is_relative_to(base.resolve()) or sha256(path) != item['sha256']:
                raise ValueError(f'pinned source missing or changed: {project["repository"]}/{relative}')
            for row in declarations(decode(path.read_bytes())):
                evidence.append(asdict(SourceEvidence(item['path'], item['sha256'], row.line, row.name, row.kind, row.options)))
        projects.append({'repository': project['repository'], 'revision': project['revision'], 'declarations': evidence})
    return projects


def verify_manifest(root: Path = ROOT) -> dict:
    manifest = json.loads((root / 'corpus/native-dependencies.json').read_text())
    normalized = json.loads(json.dumps(corpus_evidence(root)))
    if manifest['format'] != 1 or manifest['projects'] != normalized:
        raise ValueError('native dependency evidence does not match pinned corpus')
    if manifest['rebuild_recipes'] != {'vma': asdict(VMA)}:
        raise ValueError('native dependency recipe differs from reviewed pin')
    return manifest


def sdk_inventory(configured: tuple[str, ...] = (), root: Path = ROOT) -> dict:
    from native_sdk_inventory import SdkRoot, inventory
    return inventory(tuple(SdkRoot.parse(item) for item in configured), root)


def clean_environment() -> dict[str, str]:
    allowed = {'TMPDIR', 'SYSTEMROOT', 'WINDIR'}
    clean = {key: value for key, value in os.environ.items() if key in allowed}
    clean['PATH'] = '/usr/bin:/bin:/opt/homebrew/bin'
    return clean


def installed_tool(path: Path, root: Path = ROOT) -> Fingerprint:
    value = fingerprint(path, root)
    installed = ('/usr/bin', '/usr/lib', '/usr/local/bin', '/usr/local/Cellar',
                 '/opt/homebrew', '/Library/Developer', '/Applications/Xcode.app')
    if not path.is_absolute() or not any(Path(value.path).is_relative_to(base) for base in map(Path, installed)):
        raise ValueError('compiler and archiver must be explicit absolute installed tool paths')
    return value


def dependency_paths(depfile: Path) -> tuple[Path, ...]:
    text = depfile.read_text().replace('\\\n', '')
    if ':' not in text:
        raise ValueError('invalid compiler dependency file')
    return tuple(Path(item) for item in shlex.split(text.split(':', 1)[1]))


def build_vma(header: Path, includes: Path, compiler: Path, archiver: Path,
              output: Path, target: str, root: Path = ROOT, *, virtual_only: bool = False) -> BuildReceipt:
    flags = VMA_VIRTUAL_FLAGS if virtual_only else VMA_FLAGS
    header, includes = outside_inputs(header, root), outside_inputs(includes, root)
    compiler_fp, archiver_fp = installed_tool(compiler, root), installed_tool(archiver, root)
    if sha256(header) != VMA.source_sha256 or header.name != 'vk_mem_alloc.h':
        raise ValueError('VMA header does not match reviewed official source pin')
    if not (includes / 'vulkan/vulkan.h').is_file():
        raise ValueError('Vulkan include root lacks vulkan/vulkan.h')
    if not re.fullmatch(r'(?:aarch64|arm64|x86_64)-(?:apple-(?:darwin|macosx)[0-9.]*|unknown-linux-gnu)', target):
        raise ValueError('source-only VMA recipe supports explicit macOS/Linux triples only')
    output = outside_inputs(output, root)
    if not output.is_relative_to((root / 'artifacts/native-dependencies').resolve()):
        raise ValueError('rebuild outputs must remain under artifacts/native-dependencies')
    if output.exists():
        raise ValueError('refusing to replace an existing dependency build directory')
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='vma-build-', dir=output.parent) as tmp:
        stage = Path(tmp)
        wrapper, obj, deps, archive = (stage / name for name in ('vma.cpp', 'vma.o', 'vma.d', 'libVkMemAlloc.a'))
        wrapper.write_text(VMA_WRAPPER)
        commands = [[compiler_fp.path, f'--target={target}', *flags, '-I', str(header.parent), '-I', str(includes),
                     '-MD', '-MF', str(deps), '-c', str(wrapper), '-o', str(obj)],
                    [archiver_fp.path, 'rcsD', str(archive), str(obj)]]
        with (stage / 'build.log').open('w') as log:
            for command in commands:
                log.write(shlex.join(command) + '\n')
                log.flush()
                result = subprocess.run(command, env=clean_environment(), cwd=stage, timeout=300,
                                        stdout=log, stderr=subprocess.STDOUT)
                if result.returncode:
                    log.flush()
                    tail = (stage / 'build.log').read_text()[-4000:]
                    raise ValueError(f'trusted source rebuild failed ({result.returncode}):\n{tail}')
        headers = sorted(set(path.resolve() for path in dependency_paths(deps) if path.resolve() != wrapper))
        receipt = BuildReceipt(VMA, target, compiler_fp, archiver_fp,
                               tuple(fingerprint(path, root) for path in headers),
                               Fingerprint(str(output / archive.name), sha256(archive)),
                               sha256(wrapper), flags)
        # Publish only after all build commands and source fingerprinting succeed.
        stage_receipt = stage / 'receipt.json'
        stage_receipt.write_text(json.dumps(asdict(receipt), indent=2) + '\n')
        stage.rename(output)
    return receipt


def parse_receipt(raw: dict) -> BuildReceipt:
    def entry(value: dict) -> Fingerprint:
        if (not isinstance(value, dict) or set(value) != {'path', 'sha256'}
                or not isinstance(value['path'], str) or not Path(value['path']).is_absolute()
                or not isinstance(value['sha256'], str) or not re.fullmatch('[a-f0-9]{64}', value['sha256'])):
            raise ValueError('invalid receipt file fingerprint')
        return Fingerprint(**value)
    if (not isinstance(raw, dict) or raw.get('format') != 1
            or raw.get('kind') != 'source-rebuild-evidence' or raw.get('recipe') != asdict(VMA)
            or not isinstance(raw.get('target'), str) or raw.get('flags') not in (list(VMA_FLAGS), list(VMA_VIRTUAL_FLAGS))
            or raw.get('link_authority') is not False
            or raw.get('wrapper_sha256') != hashlib.sha256(VMA_WRAPPER.encode()).hexdigest()
            or not isinstance(raw.get('inputs'), list) or not raw['inputs']):
        raise ValueError('receipt configuration mismatch')
    try:
        return BuildReceipt(VMA, raw['target'], entry(raw['compiler']), entry(raw['archiver']),
                            tuple(entry(value) for value in raw['inputs']), entry(raw['artifact']),
                            raw['wrapper_sha256'], tuple(raw['flags']))
    except KeyError as error:
        raise ValueError('receipt lacks required build evidence') from error


def verify_receipt(path: Path, target: str, root: Path = ROOT) -> Fingerprint:
    receipt = parse_receipt(json.loads(outside_inputs(path, root).read_text()))
    if receipt.target != target:
        raise ValueError('receipt configuration or target mismatch')
    entries = [receipt.compiler, receipt.archiver, *receipt.inputs, receipt.artifact]
    for entry in entries:
        actual = fingerprint(Path(entry.path), root)
        if actual.sha256 != entry.sha256:
            raise ValueError(f'receipt fingerprint mismatch: {actual.path}')
    artifact = fingerprint(Path(receipt.artifact.path), root)
    if not Path(artifact.path).is_relative_to((root / 'artifacts/native-dependencies').resolve()):
        raise ValueError('receipt artifact outside managed rebuild directory')
    if not any(item.sha256 == VMA.source_sha256 for item in receipt.inputs):
        raise ValueError('receipt lacks reviewed VMA source')
    return artifact


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='action', required=True)
    sub.add_parser('verify-manifest')
    inventory = sub.add_parser('inventory')
    inventory.add_argument('--report', type=Path, default=ROOT / 'artifacts/native-sdk-inventory.json')
    inventory.add_argument('--sdk', action='append', default=[], help='name=absolute SDK root (repeatable)')
    build = sub.add_parser('build-vma')
    for name in ('header', 'includes', 'compiler', 'archiver', 'output'):
        build.add_argument(f'--{name}', type=Path, required=True)
    build.add_argument('--target', required=True)
    receipt = sub.add_parser('verify-receipt')
    receipt.add_argument('receipt', type=Path)
    receipt.add_argument('--target', required=True)
    args = parser.parse_args()
    if args.action == 'verify-manifest':
        data = verify_manifest()
        print(f'verified native declarations for {len(data["projects"])} pinned projects')
    elif args.action == 'inventory':
        path = outside_inputs(args.report)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(sdk_inventory(tuple(args.sdk)), indent=2) + '\n')
        print(path)
    elif args.action == 'build-vma':
        print(json.dumps(asdict(build_vma(args.header, args.includes, args.compiler, args.archiver, args.output, args.target).artifact)))
    else:
        print(json.dumps(asdict(verify_receipt(args.receipt, args.target))))


if __name__ == '__main__':
    main()
