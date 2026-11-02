"""What each Protocol Buffers library costs to build: compile time and executable size.

One minimal program per library defines the Spans schema (`build_cost/spans.proto`) and encodes and decodes a
batch of eight spans. Each is built with `jaic -O2 -exe` and the wall time and the executable's size are
reported next to an empty program's, so the library's own share can be read off.

- `jacket`: Extensions/Protobuf, the schema turned into Jai at compile time.
- `jai_protobuf`: segcore/jai-protobuf the same way (`#insert #run generate_jai(...)`).
- `jai_protobuf_pregenerated`: jai-protobuf's separate step, `tools/protoc-jai.jai`, writes `generated.jai`
  first. The step is timed on its own (running the built tool) and so is building the tool (once per checkout,
  not per schema change); the program then `#load`s the file.

The community sources are third-party and unreviewed: every build, the generator and every program run
inside `bench.sandbox.Sandbox` (no network, writes only to a scratch copy of `build_cost/`), as in
tools/bench/library_compare.py.

    PYTHONPATH=tools python3 benchmarks/compare/protobuf/build_cost.py --out target/compare/protobuf-build.md
"""
from __future__ import annotations

import argparse
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'tools'))

from bench.library_compare import machine, native_libraries, sandbox_for  # noqa: E402
from bench.sandbox import environment  # noqa: E402
from toolkit.repo import cargo_binary  # noqa: E402

SOURCE = Path(__file__).resolve().parent / 'build_cost'
GENERATOR = 'segcore--jai-protobuf/tools/protoc-jai.jai'


def timed(sandbox, command: list[str], cwd: Path, env: dict[str, str]) -> tuple[float, str]:
    """Run `command` in the sandbox; return the wall time in seconds and its standard output."""
    start = time.perf_counter()
    result = subprocess.run(sandbox.wrap(command), cwd=cwd, env=env, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, text=True)
    elapsed = time.perf_counter() - start
    if result.returncode != 0:
        sys.exit(f'build_cost: {" ".join(command[:4])} ... failed ({result.returncode})\n{result.stderr}{result.stdout}')
    return elapsed, result.stdout


def kib(size: int) -> str:
    return f'{size / 1024:,.0f} KiB'


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--jacket', default=str(cargo_binary('jacket', 'release')), help='the jacket binary (jaic is next to it)')
    parser.add_argument('--upstream', default=str(ROOT / 'corpus' / 'upstream'), help='the community sources')
    parser.add_argument('--repeat', type=int, default=5, help='builds per program (the median is reported)')
    parser.add_argument('--threads', type=int, default=3, help='jaic -threads')
    parser.add_argument('--out', help='write the Markdown table here')
    args = parser.parse_args(argv)
    jacket = Path(args.jacket).resolve()
    jaic = jacket.with_name('jaic')
    upstream = Path(args.upstream).resolve()
    flags = ['-O2', '-threads', str(args.threads)]

    with tempfile.TemporaryDirectory(prefix='protobuf-build-cost-') as folder:
        scratch = Path(folder).resolve()
        project = scratch / 'project'
        shutil.copytree(SOURCE, project)
        sandbox = sandbox_for(jacket, upstream, scratch, [SOURCE], loopback=False)
        env = native_libraries(environment(scratch))
        import_dir = ['-import_dir', str(upstream)]

        def build(name: str, source: str) -> tuple[list[float], Path, str]:
            exe = scratch / name
            times = []
            for _ in range(args.repeat):
                exe.unlink(missing_ok=True)
                seconds, _ = timed(sandbox, [str(jaic), source, *flags, '-exe', str(exe), *import_dir], project, env)
                times.append(seconds)
            _, output = timed(sandbox, [str(exe)], project, env)
            return times, exe, output.strip()

        # One build that is not counted warms the file cache and the stdlib's parse.
        timed(sandbox, [str(jaic), 'baseline.jai', *flags, '-exe', str(scratch / 'warmup'), *import_dir], project, env)

        rows = []
        for name, source in (('baseline', 'baseline.jai'), ('jacket', 'jacket.jai'), ('jai_protobuf', 'jai_protobuf.jai')):
            times, exe, output = build(name, source)
            rows.append((name, 0.0, times, exe.stat().st_size, output))

        # jai-protobuf with a separate generation step.
        tool = scratch / 'protoc-jai'
        tool_seconds, _ = timed(sandbox, [str(jaic), str(upstream / GENERATOR), *flags, '-exe', str(tool), *import_dir], project, env)
        generation = []
        for _ in range(args.repeat):
            seconds, _ = timed(sandbox, [str(tool), '-o', str(project / 'generated.jai'), 'spans.proto'], project, env)
            generation.append(seconds)
        times, exe, output = build('jai_protobuf_pregenerated', 'jai_protobuf_pregenerated.jai')
        rows.append(('jai_protobuf_pregenerated', statistics.median(generation), times, exe.stat().st_size, output))

        base_time = statistics.median(rows[0][2])
        base_size = rows[0][3]
        lines = ['| Program | Generation step | Compile (median, min) | Compile minus baseline | Executable | Minus baseline | Output |',
                 '| --- | --- | --- | --- | --- | --- | --- |']
        for name, step, times, size, output in rows:
            median = statistics.median(times)
            lines.append(f"| {name} | {f'{step:.2f} s' if step else 'none'} | {median:.2f} s, {min(times):.2f} s | "
                         f"{median - base_time:+.2f} s | {kib(size)} | {kib(size - base_size)} | `{output}` |")
        text = '\n'.join(lines)
        text += f'\n\nBuilding jai-protobuf\'s generator tool (`tools/protoc-jai.jai`, once): {tool_seconds:.2f} s.'
        text += f'\n\nEach program is built {args.repeat} times with `jaic {" ".join(flags)} -exe`.'
        header = '\n'.join(f'- {k}: {v}' for k, v in machine().items())
        output = f'{header}\n\n{text}\n'
        if args.out:
            Path(args.out).write_text(output, encoding='utf-8')
        print(output)
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
