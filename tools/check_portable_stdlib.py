#!/usr/bin/env python3
"""Run focused supported cases with an explicitly source-built local compiler.

This is a worker regression set, not the coordinator's full conformance sweep.
Unimplemented module fixtures remain unchanged in tests/stdlib-contract.
"""
import argparse
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = [
    "basic", "string", "memory", "pcg", "codec", "bit-array", "reflection",
    "unicode", "pool", "adpcm", "bucket-array", "file-utils", "socket", "thread", "process", "command-line",
]
EXPECTED = {
    "print-color": "\x1b[1;31mcolor 42\x1b[0m\x1b[4;32mscope\x1b[0mok\n",
    "runtime": "runtime -9223372036854775808 ff\n",
    "foundations": "-42 true text 55\n",
    "command-help": lambda out: all(text in out for text in ["configuration comment", "last option", "--alpha", "--zeta", "--help"]) and out.index("--alpha") < out.index("--zeta") and out.endswith("ok\n"),
}
EXPECTED_STDERR = {"print-color": "\x1b[0;44merror\x1b[0m"}
FAULT_EXPECTATIONS = {
    "overwriting-double-free": ("assertion failed", "invalid overwriting-allocator pointer"),
    "xar-capacity": ("assertion failed", "Xar configured capacity exceeded"),
    "crash-handler": ("crash signal 11", "native stack:"),
    "crash-memory-fault": ("crash signal ", "native stack:"),
}


def run(command, env, expected=None, cwd=ROOT, expected_stderr=""):
    result = subprocess.run(command, cwd=cwd, env=env, text=True,
                            capture_output=True, timeout=45)
    matches = expected(result.stdout) if callable(expected) else expected is None or result.stdout == expected
    if result.returncode or not matches:
        print("FAILED:", " ".join(map(str, command)), file=sys.stderr)
        print(result.stdout, end="", file=sys.stderr)
        print(result.stderr, end="", file=sys.stderr)
        raise RuntimeError(f"exit={result.returncode}, expected stdout={expected!r}")
    if result.stderr != expected_stderr:
        print(result.stderr, end="", file=sys.stderr)
        raise RuntimeError("unexpected compiler/runtime diagnostics")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", type=Path, default=ROOT / "target/compiler/debug/jaic")
    parser.add_argument("--posix-root", required=True, type=Path,
                        help="new independently implemented POSIX module directory")
    parser.add_argument("--native", action="store_true")
    parser.add_argument("--hash-root", type=Path,
                        help="new independently implemented Hash module directory for allocator integration")
    parser.add_argument("--curl-root", type=Path,
                        help="new Curl module directory; defaults beside the supplied POSIX module")
    parser.add_argument("--check-target", action="append", default=[],
                        help="OS/CPU pair, e.g. linux/x64; checks portable cases only")
    args = parser.parse_args()
    compiler = args.jaic.resolve(strict=True)
    if not compiler.is_relative_to(ROOT / "target"):
        parser.error("compiler must be a source build under this checkout's target directory")
    root = ROOT / "target/portable-integration-stdlib"
    subprocess.run([sys.executable, "-B", str(ROOT / "tools/build_portable_runtime.py")], cwd=ROOT, check=True)
    shutil.copytree(ROOT / "stdlib", root, dirs_exist_ok=True)
    shutil.copytree(args.posix_root, root / "POSIX", dirs_exist_ok=True)
    if args.hash_root:
        shutil.copytree(args.hash_root, root / "Hash", dirs_exist_ok=True)
        shutil.copytree(args.hash_root.parent / "internal/hash", root / "internal/hash", dirs_exist_ok=True)
    curl_root = args.curl_root or args.posix_root.parent / "Curl"
    if not curl_root.is_dir():
        parser.error("supply the new Curl dependency with --curl-root")
    shutil.copytree(curl_root, root / "Curl", dirs_exist_ok=True)
    shutil.copytree(curl_root.parent / "internal/bindings", root / "internal/bindings", dirs_exist_ok=True)
    env = dict(os.environ, JAIC_STDLIB=str(root))
    output = ROOT / "target/portable-tests"
    output.mkdir(parents=True, exist_ok=True)
    cases = [(p, EXPECTED.get(p.stem, "ok\n"))
             for p in sorted((ROOT / "tests/portable-stdlib").glob("*.jai"))
             if p.stem not in {"bootstrap", "process-parent"}]
    cases += [(ROOT / f"tests/stdlib-contract/{name}/main.jai", "ok\n") for name in CONTRACTS]
    run([str(compiler), "-check", str(ROOT / "tests/portable-stdlib/bootstrap.jai")], env)
    for index, (source, expected) in enumerate(cases):
        run([str(compiler), "-interp", str(source), "--"], env, expected,
            expected_stderr=EXPECTED_STDERR.get(source.stem,""))
        if args.native:
            executable = output / f"case-{index}"
            run([str(compiler), str(source), "-exe", str(executable)], env)
            run([str(executable)], env, expected, source.parent,
                expected_stderr=EXPECTED_STDERR.get(source.stem,""))
        print("passed", source.relative_to(ROOT), flush=True)
    parent_source = ROOT / "tests/portable-stdlib/process-parent.jai"
    for native in ([False, True] if args.native else [False]):
        for keep in [False, True]:
            marker = parent_source.parent / ("process-retained-marker.tmp" if keep else "process-killed-marker.tmp")
            marker.unlink(missing_ok=True)
            extra = ["-add", "PROCESS_KEEP_ALIVE :: true;"] if keep else []
            try:
                if native:
                    executable = output / f"parent-{keep}"
                    run([str(compiler), str(parent_source), *extra, "-exe", str(executable)], env)
                    run([str(executable)], env, "ok\n", parent_source.parent)
                else:
                    run([str(compiler), "-interp", str(parent_source), *extra, "--"], env, "ok\n")
                deadline = time.monotonic() + (2 if keep else 0.5)
                while time.monotonic() < deadline:
                    if marker.exists():
                        break
                    time.sleep(0.01)
                if marker.exists() != keep:
                    raise RuntimeError(f"parent-exit cleanup mismatch: native={native}, keep={keep}")
            finally:
                marker.unlink(missing_ok=True)
    print("passed parent-exit cancellation and opt-out", flush=True)
    for source in sorted((ROOT / "tests/portable-stdlib/faults").glob("*.jai")):
        commands = [[str(compiler), "-interp", str(source), "--"]]
        if args.native:
            executable = output / source.stem
            run([str(compiler), str(source), "-exe", str(executable)], env)
            commands.append([str(executable)])
        for command in commands:
            result = subprocess.run(command, cwd=ROOT, env=env, text=True,
                                    capture_output=True, timeout=45)
            if result.returncode == 0 or not all(text in result.stderr for text in FAULT_EXPECTATIONS[source.stem]):
                raise RuntimeError(f"expected checked runtime fault: {result.returncode}: {result.stdout}{result.stderr}")
            if source.stem in {"crash-handler", "crash-memory-fault"}:
                if result.returncode not in {-signal.SIGSEGV, -signal.SIGBUS}:
                    raise RuntimeError("crash handler failed to preserve fatal signal termination")
        print("passed expected runtime fault", source.name, flush=True)
    negative = ROOT / "tests/portable-stdlib/negative/xar-reverse.jai"
    result = subprocess.run([str(compiler), "-check", str(negative)], cwd=ROOT,
                            env=env, text=True, capture_output=True, timeout=45)
    if result.returncode == 0 or "reverse Xar iteration is unsupported" not in result.stderr:
        raise RuntimeError(f"expected compile-time reverse iteration rejection: {result.stdout}{result.stderr}")
    print("passed compile-time Xar reverse rejection", flush=True)
    mail = [sys.executable, "-B", str(ROOT / "tools/check_portable_mail.py"),
            "--jaic", str(compiler), "--stdlib", str(root)]
    if args.native:
        mail.append("--native")
    subprocess.run(mail, cwd=ROOT, env=env, check=True)
    if args.native:
        subprocess.run([sys.executable, "-B", str(ROOT / "tools/check_portable_crash.py"),
                        "--jaic", str(compiler), "--stdlib", str(root)], cwd=ROOT, env=env, check=True)
    dependent = {"file", "filesystem", "socket-loopback", "process", "process-io", "system", "text-handler"}
    posix_only = {"module-identity", "thread-lifetime", "thread-groups", "thread-condition", "thread-shutdown", "overwriting-allocator", "unmapping-allocator", "crash-handler-init"}
    for target in args.check_target:
        os_name, cpu = target.split("/", 1)
        for source, _ in cases:
            if source.stem == "default-allocator" and cpu == "x64":
                continue  # Optimized heap branch is explicitly unsupported, not fallback coverage.
            if source.parent.name != "portable-stdlib" or source.stem in dependent or (os_name == "windows" and source.stem in posix_only):
                continue
            run([str(compiler), "-check", "-os", os_name, "-cpu", cpu, str(source)], env)
        print("checked portable cases", target, flush=True)
    print(f"{len(cases)} focused cases passed; native={args.native}")


if __name__ == "__main__":
    main()
