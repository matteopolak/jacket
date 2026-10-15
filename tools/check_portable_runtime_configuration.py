#!/usr/bin/env python3
"""Execute the adopted runtime against settled root and child workspace sizes.

Uses the explicitly selected production library without source substitution.
"""
from pathlib import Path
import argparse
import os
import subprocess

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--jaic", type=Path, default=ROOT / "target/debug/jaic")
parser.add_argument("--stdlib", type=Path, default=ROOT / "stdlib")
args = parser.parse_args()
LIBRARY = args.stdlib.resolve(strict=True)
COMPILER = args.jaic.resolve(strict=True)
OUTPUT = ROOT / "target/portable-runtime-configuration"


def run(arguments, expected=None):
    result = subprocess.run(arguments, cwd=ROOT,
                            env=dict(os.environ, JAIC_STDLIB=str(LIBRARY)),
                            capture_output=True, text=True, timeout=120)
    if result.returncode or result.stderr or (expected is not None and result.stdout != expected):
        raise RuntimeError(f"{arguments!r}\nexit={result.returncode}\n{result.stdout}{result.stderr}")


def main():
    OUTPUT.mkdir(parents=True, exist_ok=True)
    source = ROOT / "tests/portable-stdlib/runtime-configuration.jai"
    for size in [0, 65536, 131072]:
        options = ["-add", f"REQUESTED_SIZE :: {size};"]
        run([str(COMPILER), "-interp", str(source), *options], "ok\n")
        for optimization in ["-O0", "-O2"]:
            executable = OUTPUT / f"root-{size}{optimization}"
            run([str(COMPILER), str(source), *options, optimization, "-exe", str(executable)])
            run([str(executable)], "ok\n")
        print(f"actual root storage {size}: interpreter/native O0/O2 passed", flush=True)

    # Each child owns its configuration and executable. Source strings are emitted
    # as Jai literals with explicit escaping, not interpreted by a shell.
    build = ['C :: #import "Compiler"; #run {']
    # The parent sets each child's build options. Child source only observes
    # storage; it must not run the root fixture's default-workspace setter.
    child_body = '#import "Basic";\nmain ::' + source.read_text().split('main ::', 1)[1]
    for size in [0, 131072]:
        child = f"REQUESTED_SIZE :: {size};\n" + child_body
        literal = '"' + child.replace('\\', '\\\\').replace('"', '\\"').replace('\n', '\\n') + '"'
        directory = '"' + str(OUTPUT).replace('\\', '\\\\').replace('"', '\\"') + '"'
        build.append(f'''{{
    workspace := C.compiler_create_workspace("child-{size}");
    options := C.get_build_options(workspace);
    options.output_type = .EXECUTABLE;
    options.output_path = {directory};
    options.output_executable_name = "child-{size}";
    options.temporary_storage_size = {size};
    C.set_build_options(options, workspace);
    C.add_build_string({literal}, workspace);
}}''')
    build.append('} main :: () {}')
    builder = OUTPUT / "children.jai"
    builder.write_text('\n'.join(build) + '\n')
    run([str(COMPILER), "-interp", str(builder)])
    for size in [0, 131072]:
        run([str(OUTPUT / f"child-{size}")], "ok\n")
    print("independent child storage 0/131072: native executions passed", flush=True)


if __name__ == "__main__":
    main()
