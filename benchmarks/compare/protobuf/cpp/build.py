"""Builds and runs Google's C++ protobuf on the shapes of benches/protobuf.jai (`bench.cc`).

`protoc` writes the C++ for `../proto/shapes.proto` (proto3, default options, no arena), `/usr/bin/clang++ -O2`
compiles it with `bench.cc` and links `libprotobuf.a` and the Abseil libraries the protobuf build used. Everything,
protoc, the compiler and the program, runs inside `bench.sandbox.Sandbox` (no network, writes only to a scratch
folder): Google's code is upstream code, like the Jai libraries.

Needs a CMake build of protobuf (its `protoc` and `libprotobuf.a`, with Abseil from Homebrew) and its sources:

    PYTHONPATH=tools python3 benchmarks/compare/protobuf/cpp/build.py \\
        --protobuf-source /path/to/protobuf --protobuf-build /path/to/protobuf/build [--out target/compare/protobuf-cpp.md]

Use `/usr/bin/clang++`, not Homebrew's clang, which does not agree with the SDK. The result is a table of
`name  bytes  ns/iter  MB/s  allocations` per workload, named like the Jai benchmarks (`decode_spans__cpp`).
"""
from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / 'tools'))

from bench.sandbox import Sandbox, environment  # noqa: E402

CLANG = '/usr/bin/clang++'
SHAPES = HERE.parent / 'proto' / 'shapes.proto'


def link_libraries(build: Path) -> list[str]:
    """The libraries the protobuf build itself links a program with (read from its build.ninja): the static
    archives and the Abseil dylibs."""
    for line in (build / 'build.ninja').read_text(encoding='utf-8').splitlines():
        if line.strip().startswith('LINK_LIBRARIES') and 'libprotobuf.a' in line:
            words = line.split('=', 1)[1].split()
            libraries = []
            for word in words:
                if word.endswith('.dylib') or word.endswith('.so'):
                    libraries.append(word)
                elif word.endswith('.a'):
                    libraries.append(str(word if Path(word).is_absolute() else build / word))
            return libraries
    sys.exit('build.py: no link line with libprotobuf.a in build.ninja')


def run(sandbox: Sandbox, command: list[str], cwd: Path, env: dict[str, str], what: str) -> subprocess.CompletedProcess:
    start = time.perf_counter()
    result = subprocess.run(sandbox.wrap(command), cwd=cwd, env=env, capture_output=True, text=True)
    if result.returncode != 0:
        sys.exit(f'build.py: {what} failed ({result.returncode})\n{result.stderr}{result.stdout}')
    print(f'{what}: {time.perf_counter() - start:.1f} s', file=sys.stderr)
    return result


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--protobuf-source', required=True, help='a protobuf checkout (src/ and third_party/utf8_range)')
    parser.add_argument('--protobuf-build', required=True, help='its CMake build folder (protoc, libprotobuf.a)')
    parser.add_argument('--only', default='', help='one workload: mixed, packed, deep, logs or spans')
    parser.add_argument('--out', help='write the table here')
    args = parser.parse_args(argv)
    source = Path(args.protobuf_source).resolve()
    build = Path(args.protobuf_build).resolve()
    protoc = build / 'protoc'

    with tempfile.TemporaryDirectory(prefix='protobuf-cpp-') as folder:
        scratch = Path(folder).resolve()
        project = scratch / 'project'
        project.mkdir()
        for name in ('bench.cc',):
            shutil.copy(HERE / name, project / name)
        shutil.copy(SHAPES, project / 'shapes.proto')
        sandbox = Sandbox(reads=[source, build, Path('/opt/homebrew'), HERE.parent], writes=[scratch])
        env = environment(scratch)

        run(sandbox, [str(protoc), '--cpp_out=.', 'shapes.proto'], project, env, 'protoc')
        flags = ['-O2', '-DNDEBUG', '-std=gnu++17', '-arch', 'arm64' if sys.platform == 'darwin' else '-m64']
        flags = [flag for flag in flags if flag not in ('-m64',)]
        includes = [f'-I{source / "src"}', f'-I{source}', f'-I{source / "third_party" / "utf8_range"}', f'-I{build}',
                    f'-I{build / "src"}', '-isystem', '/opt/homebrew/include', '-I.']
        run(sandbox, [CLANG, *flags, *includes, '-c', 'shapes.pb.cc', '-o', 'shapes.pb.o'], project, env, 'compile shapes.pb.cc')
        run(sandbox, [CLANG, *flags, *includes, '-c', 'bench.cc', '-o', 'bench.o'], project, env, 'compile bench.cc')
        libraries = link_libraries(build)
        run(sandbox, [CLANG, *flags, 'bench.o', 'shapes.pb.o', '-o', 'bench', *libraries,
                      '-Wl,-rpath,/opt/homebrew/lib'], project, env, 'link')
        result = run(sandbox, ['./bench', *([args.only] if args.only else [])], project, env, 'run')
        sys.stderr.write(result.stderr)
        rows = [line.split('\t') for line in result.stdout.splitlines() if line.count('\t') == 7]

    lines = ['| Workload | Bytes | Time per call | MB/s | Allocations |', '| --- | --- | --- | --- | --- |']
    for name, size, median, low, high, rate, count, total in rows:
        lines.append(f'| {name} | {int(size):,} | {float(median):,.0f} ns ({float(low):,.0f} to {float(high):,.0f}) | '
                     f'{int(rate):,} | {int(count):,} ({int(total):,} B) |')
    text = '\n'.join(lines)
    print(text)
    if args.out:
        Path(args.out).write_text(text + '\n', encoding='utf-8')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
