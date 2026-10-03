#!/usr/bin/env python3
"""Fresh source rebuild and measured ABI evidence for the reviewed VMA virtual API.

Persisted receipts remain evidence. Only this invocation's freshly rebuilt archive
is returned to the typed driver; existing receipt artifact bytes are never linked.
"""
from __future__ import annotations

import argparse
from dataclasses import asdict, dataclass
import json
from pathlib import Path
import subprocess
import sys

# -I disables inherited Python import roots. Import only our repository helpers.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import native_dependencies as native


RECIPE = 'vma-virtual-3.3.0'
ABI_PROBE = r'''
#include "vk_mem_alloc.h"
#include <cstddef>
#include <cstdio>
#include <type_traits>
static_assert(std::is_same_v<decltype(&vmaCreateVirtualBlock), VkResult(*)(const VmaVirtualBlockCreateInfo*, VmaVirtualBlock*)>);
static_assert(std::is_same_v<decltype(&vmaDestroyVirtualBlock), void(*)(VmaVirtualBlock)>);
static_assert(std::is_same_v<decltype(&vmaVirtualAllocate), VkResult(*)(VmaVirtualBlock, const VmaVirtualAllocationCreateInfo*, VmaVirtualAllocation*, VkDeviceSize*)>);
static_assert(std::is_same_v<decltype(&vmaVirtualFree), void(*)(VmaVirtualBlock, VmaVirtualAllocation)>);
static_assert(std::is_same_v<decltype(&vmaGetVirtualAllocationInfo), void(*)(VmaVirtualBlock, VmaVirtualAllocation, VmaVirtualAllocationInfo*)>);
static_assert(std::is_same_v<decltype(&vmaIsVirtualBlockEmpty), VkBool32(*)(VmaVirtualBlock)>);
static_assert(sizeof(void*) == 8 && sizeof(VkDeviceSize) == 8 && sizeof(VkResult) == 4);
static_assert(std::is_pointer_v<VmaVirtualBlock> && std::is_pointer_v<VmaVirtualAllocation>);
#define OFFSET(T, F) static_cast<unsigned long long>(offsetof(T, F))
#define SIZE(T) static_cast<unsigned long long>(sizeof(T))
int main() {
    VmaVirtualBlockCreateInfo block_info{};
    block_info.size = 4096;
    VmaVirtualBlock block{};
    if (vmaCreateVirtualBlock(&block_info, &block) != VK_SUCCESS || !block) return 11;
    VmaVirtualAllocationCreateInfo allocation_info{};
    allocation_info.size = 42;
    allocation_info.alignment = 16;
    int token = 7;
    allocation_info.pUserData = &token;
    VmaVirtualAllocation allocation{};
    VkDeviceSize offset{};
    if (vmaVirtualAllocate(block, &allocation_info, &allocation, &offset) != VK_SUCCESS || !allocation) return 12;
    VmaVirtualAllocationInfo actual{};
    vmaGetVirtualAllocationInfo(block, allocation, &actual);
    if (actual.size != 42 || actual.offset != offset || offset % 16 || actual.pUserData != &token) return 13;
    if (vmaIsVirtualBlockEmpty(block)) return 14;
    vmaVirtualFree(block, allocation);
    if (!vmaIsVirtualBlockEmpty(block)) return 15;
    vmaDestroyVirtualBlock(block);
    std::printf("block %llu %u %llu %llu %llu\n", SIZE(VmaVirtualBlockCreateInfo), unsigned(alignof(VmaVirtualBlockCreateInfo)), OFFSET(VmaVirtualBlockCreateInfo,size), OFFSET(VmaVirtualBlockCreateInfo,flags), OFFSET(VmaVirtualBlockCreateInfo,pAllocationCallbacks));
    std::printf("allocation %llu %u %llu %llu %llu %llu\n", SIZE(VmaVirtualAllocationCreateInfo), unsigned(alignof(VmaVirtualAllocationCreateInfo)), OFFSET(VmaVirtualAllocationCreateInfo,size), OFFSET(VmaVirtualAllocationCreateInfo,alignment), OFFSET(VmaVirtualAllocationCreateInfo,flags), OFFSET(VmaVirtualAllocationCreateInfo,pUserData));
    std::printf("info %llu %u %llu %llu %llu\n", SIZE(VmaVirtualAllocationInfo), unsigned(alignof(VmaVirtualAllocationInfo)), OFFSET(VmaVirtualAllocationInfo,offset), OFFSET(VmaVirtualAllocationInfo,size), OFFSET(VmaVirtualAllocationInfo,pUserData));
}
'''


@dataclass(frozen=True)
class RecordAbi:
    name: str
    size: int
    alignment: int
    offsets: tuple[int, ...]


@dataclass(frozen=True)
class FreshSourceProof:
    recipe: str
    target: str
    artifact: native.Fingerprint
    records: tuple[RecordAbi, ...]
    build_receipt: str
    probe_source_sha256: str
    kind: str = 'fresh-source-abi-proof'
    # Persisting this object cannot recreate the in-process driver authority.
    link_authority: bool = False


def parse_abi_output(text: str) -> tuple[RecordAbi, ...]:
    expected = {'block': 3, 'allocation': 4, 'info': 3}
    result = []
    for line in text.splitlines():
        words = line.split()
        if not words or words[0] not in expected or len(words) != expected[words[0]] + 3:
            raise ValueError('unexpected ABI probe output')
        name = words[0]
        if any(row.name == name for row in result):
            raise ValueError('duplicate ABI probe record')
        if any(not word.isdecimal() for word in words[1:]):
            raise ValueError('non-numeric ABI probe layout')
        size, alignment, *offsets = map(int, words[1:])
        if size > 4096 or alignment > 256 or alignment == 0 or alignment & (alignment - 1):
            raise ValueError('invalid measured ABI layout')
        if offsets != sorted(offsets) or any(value >= size for value in offsets):
            raise ValueError('invalid measured field offsets')
        result.append(RecordAbi(name, size, alignment, tuple(offsets)))
    if {row.name for row in result} != set(expected):
        raise ValueError('incomplete ABI probe output')
    return tuple(result)


def reviewed_inputs(receipt: native.BuildReceipt, root: Path) -> tuple[Path, Path]:
    sources = [Path(row.path) for row in receipt.inputs if row.sha256 == native.VMA.source_sha256]
    vulkan = [Path(row.path) for row in receipt.inputs if Path(row.path).parts[-2:] == ('vulkan', 'vulkan.h')]
    if len(sources) != 1 or len(vulkan) != 1:
        raise ValueError('receipt must contain exactly one reviewed VMA source and Vulkan root header')
    installed = tuple(map(Path, ('/usr/include', '/usr/lib', '/Library/Developer',
                                 '/Applications/Xcode.app', '/opt/homebrew', '/usr/local/Cellar')))
    for row in receipt.inputs:
        path = native.outside_inputs(Path(row.path), root)
        if path != sources[0] and not any(path.is_relative_to(base) for base in installed):
            raise ValueError(f'unreviewed SDK source root: {path}')
    native.installed_tool(Path(receipt.compiler.path), root)
    native.installed_tool(Path(receipt.archiver.path), root)
    return sources[0], vulkan[0].parent.parent


def verify_rebuilt_inputs(reviewed: native.BuildReceipt, rebuilt: native.BuildReceipt,
                          root: Path) -> None:
    # Compilation emits inert objects first. Before executing an oracle, require
    # the actual transitive headers/tools to remain the reviewed receipt inputs.
    reviewed_inputs(rebuilt, root)
    if (set(rebuilt.inputs) != set(reviewed.inputs) or rebuilt.compiler != reviewed.compiler
            or rebuilt.archiver != reviewed.archiver):
        raise ValueError('fresh dependency compilation differs from reviewed source/SDK/tool fingerprints')


def rebuild_and_prove(receipt_path: Path, target: str, output: Path,
                      root: Path = native.ROOT) -> FreshSourceProof:
    receipt_path = native.outside_inputs(receipt_path, root)
    receipt = native.parse_receipt(json.loads(receipt_path.read_text()))
    # Check all evidence, including old artifact changes, without loading it.
    native.verify_receipt(receipt_path, receipt.target, root)
    header, includes = reviewed_inputs(receipt, root)
    output = native.outside_inputs(output, root)
    if output.exists():
        raise ValueError('source ABI proof output must be fresh')
    build = native.build_vma(header, includes, Path(receipt.compiler.path),
                             Path(receipt.archiver.path), output, target, root,
                             virtual_only=True)
    verify_rebuilt_inputs(receipt, build, root)
    probe = output / 'abi-probe.cpp'
    executable = output / 'abi-probe'
    probe.write_text(ABI_PROBE)
    dependencies = output / 'abi-probe.d'
    command = [build.compiler.path, f'--target={target}', *native.VMA_VIRTUAL_FLAGS,
               '-I', str(header.parent), '-I', str(includes), str(probe),
               '-MD', '-MF', str(dependencies), build.artifact.path, '-o', str(executable)]
    with (output / 'abi-build.log').open('w') as log:
        compiled = subprocess.run(command, env=native.clean_environment(), cwd=output,
                                  timeout=300, stdout=log, stderr=subprocess.STDOUT)
    if compiled.returncode:
        raise ValueError('reviewed ABI probe failed to compile/link: ' + (output / 'abi-build.log').read_text()[-4000:])
    # The independently written probe may add installed standard-library headers,
    # but it cannot import another source implementation beside the pinned VMA.
    headers = tuple(native.fingerprint(path, root) for path in native.dependency_paths(dependencies)
                    if path.resolve() != probe.resolve())
    probe_receipt = native.BuildReceipt(build.recipe, build.target, build.compiler, build.archiver,
                    headers, build.artifact, build.wrapper_sha256, build.flags)
    reviewed_inputs(probe_receipt, root)
    measured = subprocess.run([str(executable)], env=native.clean_environment(), cwd=output,
                              timeout=10, capture_output=True, text=True)
    if measured.returncode:
        raise ValueError(f'rebuilt virtual allocator ABI/runtime oracle failed ({measured.returncode})')
    records = parse_abi_output(measured.stdout)
    proof = FreshSourceProof(RECIPE, target, build.artifact, records,
                             str(output / 'receipt.json'), native.sha256(probe))
    (output / 'abi-proof.json').write_text(json.dumps(asdict(proof), indent=2) + '\n')
    return proof


def protocol(proof: FreshSourceProof) -> str:
    fields = ["JAI_NATIVE_SOURCE_ABI_V1", proof.recipe, proof.target,
              proof.artifact.path, proof.artifact.sha256]
    for record in proof.records:
        fields.append(' '.join(map(str, (record.name, record.size, record.alignment, *record.offsets))))
    if any('\n' in field or '\r' in field for field in fields):
        raise ValueError('proof protocol fields cannot contain line breaks')
    return '\n'.join(fields) + '\n'


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    proof = commands.add_parser('rebuild-vma-virtual')
    proof.add_argument('--receipt', type=Path, required=True)
    proof.add_argument('--target', required=True)
    proof.add_argument('--output', type=Path, required=True)
    snapshot = commands.add_parser('verify-snapshot')
    snapshot.add_argument('--artifact', type=Path, required=True)
    snapshot.add_argument('--sha256', required=True)
    args = parser.parse_args()
    if args.command == 'rebuild-vma-virtual':
        print(protocol(rebuild_and_prove(args.receipt, args.target, args.output)), end='')
    else:
        actual = native.fingerprint(args.artifact)
        if actual.sha256 != args.sha256:
            raise ValueError('fresh native artifact snapshot changed')
        print(actual.sha256)


if __name__ == '__main__':
    main()
