#!/usr/bin/env python3
"""Run focused supported cases with an explicitly source-built local compiler.

This is a worker regression set, not the coordinator's full conformance sweep.
Unimplemented module fixtures remain unchanged in tests/stdlib-contract.
"""
import argparse
import os
from pathlib import Path
import plistlib
import shutil
import signal
import subprocess
import sys
import time
from check_foundations import expected_output as foundation_output

ROOT = Path(__file__).resolve().parents[1]


def valid_bundle_plist(output, require_reals=False):
    expected = {"a<&\"'": "value<&\"'\r\né", "true": True, "false": False,
                "negative": -9223372036854775808, "unsigned": 18446744073709551615}
    if require_reals:
        expected.update({"real": 0.125, "large-real": 1.25e100})
    return plistlib.loads(output.encode("utf-8")) == expected


CONTRACTS = [
    "basic", "string", "memory", "pcg", "codec", "bit-array", "reflection",
    "unicode", "pool", "adpcm", "bucket-array", "file-utils", "socket", "thread", "process", "command-line",
]
EXPECTED = {
    "bundle-plist": lambda output: valid_bundle_plist(output, require_reals=True),
    "print-color": "\x1b[1;31mcolor 42\x1b[0m\x1b[4;32mscope\x1b[0mok\n",
    "basic-output": "out 7++",
    "runtime": "runtime -9223372036854775808 ff\n",
    "foundations": "-42 true text 55\n",
    "meow-dump": "state\n0: 1 2\n1: 3 4\n2: 5 6\n3: 7 8\n4: 9 10\n5: 11 12\n6: 13 14\n7: 15 16\nok\n",
    "command-help": lambda out: all(text in out for text in ["configuration comment", "last option", "--alpha", "--zeta", "--help"]) and out.index("--alpha") < out.index("--zeta") and out.endswith("ok\n"),
}
EXPECTED_STDERR = {"print-color": "\x1b[0;44merror\x1b[0m", "basic-output": "err true"}
EXPECTED_EXIT = {"basic-output": 17}
for name in ("basic-logging-output", "basic-print-bytes", "basic-stack-output"):
    EXPECTED[name], EXPECTED_STDERR[name], EXPECTED_EXIT[name] = foundation_output(name)
FAULT_EXPECTATIONS = {
    "flat-pool-capacity": ("assertion failed", "flat-pool reservation exhausted"),
    "sound-channels": ("assertion failed", "audio sample input/channel range invalid"),
    "sound-magic": ("assertion failed", "audio magic read outside input"),
    "atomic-bit-index": ("assertion failed", "atomic bit index out of range"),
    "deep-copy-fixed": ("assertion failed", "Deep_Copy does not support fixed arrays"),
    "deep-copy-any": ("assertion failed", "Deep_Copy does not support this reflected value kind"),
    "treemap-dirty": ("assertion failed", "Treemap display requires clean weights"),
    "mpeg-vlc-truncated": ("assertion failed", "truncated MPEG VLC code"),
    "mpeg-motion-bounds": ("assertion failed", "MPEG motion region out of bounds"),
    "mpeg-sequence-change": ("assertion failed", "MPEG sequence dimension/rate changes are unsupported"),
    "mpeg-picture-hole": ("assertion failed", "incomplete MPEG picture coverage"),
    "mpeg-audio-crc": ("assertion failed", "MPEG audio checksum mismatch"),
    "mpeg-audio-quantizer": ("assertion failed", "invalid MPEG grouped quantizer value"),
    "mpeg-program-pts": ("assertion failed", "invalid MPEG timestamp marker"),
    "mpeg-program-truncated": ("assertion failed", "truncated MPEG PES packet"),
    "mpeg-no-delay-b": ("assertion failed", "MPEG B picture violates no-delay assumption"),
    "meow-null": ("assertion failed", "MeowHash invalid source/length"),
    "overwriting-double-free": ("assertion failed", "invalid overwriting-allocator pointer"),
    "xar-capacity": ("assertion failed", "Xar configured capacity exceeded"),
    "crash-handler": ("crash signal 11", "native stack:"),
    "crash-memory-fault": ("crash signal ", "native stack:"),
    "zip-signature": ("assertion failed", "unexpected ZIP directory signature"),
    "zip-descriptor": ("assertion failed", "ZIP data descriptors are unsupported"),
}


def run(command, env, expected=None, cwd=ROOT, expected_stderr="", expected_code=0):
    result = subprocess.run(command, cwd=cwd, env=env, text=True,
                            capture_output=True, timeout=45)
    matches = expected(result.stdout) if callable(expected) else expected is None or result.stdout == expected
    if result.returncode != expected_code or not matches:
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
    parser.add_argument("--stdlib", type=Path, help="Production tree; no source overlays")
    parser.add_argument("--posix-root", type=Path,
                        help="new independently implemented POSIX module directory")
    parser.add_argument("--native", action="store_true")
    parser.add_argument("--keep-going", action="store_true", help="Run independent cases after failures; overall exit still fails")
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
    subprocess.run([sys.executable, "-B", str(ROOT / "tools/build_portable_runtime.py")], cwd=ROOT, check=True)
    subprocess.run([sys.executable, "-B", str(ROOT / "tools/build_portable_zip_fixtures.py")], cwd=ROOT, check=True)
    subprocess.run([sys.executable, "-B", str(ROOT / "tools/build_portable_archive_fixtures.py")], cwd=ROOT, check=True)
    subprocess.run([sys.executable, "-B", str(ROOT / "tools/check_portable_mpeg_matrix.py")], cwd=ROOT, check=True)
    if args.stdlib:
        root = args.stdlib.resolve(strict=True)
    else:
        if not args.posix_root:
            parser.error("supply --stdlib for production or --posix-root for explicit staging")
        root = ROOT / "target/portable-integration-stdlib"
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
    subprocess.run([sys.executable, "-B", str(ROOT / "tools/build_portable_mpeg_fixtures.py")], cwd=ROOT, check=True)
    subprocess.run([sys.executable, "-B", str(ROOT / "tools/build_portable_meow_fixtures.py")], cwd=ROOT, check=True)
    env = dict(os.environ, JAIC_STDLIB=str(root))
    output = ROOT / "target/portable-tests"
    output.mkdir(parents=True, exist_ok=True)
    if args.native:
        subprocess.run([sys.executable, "-B", str(ROOT / "tools/check_portable_runtime_configuration.py"),
                        "--jaic", str(compiler), "--stdlib", str(root)], cwd=ROOT, env=env, check=True)
        integrations = ["plist", "meow"]
        if sys.platform == "darwin":
            integrations.append("bundle")
        for name in integrations:
            subprocess.run([sys.executable, "-B", str(ROOT / f"tools/check_portable_{name}_integration.py"),
                            "--jaic", str(compiler), "--stdlib", str(root)], cwd=ROOT, env=env, check=True)
        if sys.platform == "darwin":
            subprocess.run([sys.executable, "-B", str(ROOT / "tools/check_portable_watcher.py"),
                            "--jaic", str(compiler), "--stdlib", str(root)], cwd=ROOT, env=env, check=True)
    cases = [(p, EXPECTED.get(p.stem, "ok\n"))
             for p in sorted((ROOT / "tests/portable-stdlib").glob("*.jai"))
             if p.stem not in {"bootstrap", "process-parent", "float-power-sweep"} and (p.stem != "windows-foundations" or sys.platform == "win32")]
    if sys.platform != "win32":
        print("Windows filesystem runtime fixture requires a Windows host; retained for Windows target checking", flush=True)
    cases += [(ROOT / f"tests/stdlib-contract/{name}/main.jai", "ok\n") for name in CONTRACTS]
    run([str(compiler), "-check", str(ROOT / "tests/portable-stdlib/bootstrap.jai")], env)
    failures = []
    # This fixture emits 2300 numerical rows, not the ordinary "ok" protocol.
    # Its independent rational oracle remains mandatory, including interpreter execution.
    sweep = [sys.executable, str(ROOT / "tools/check_float_power_sweep.py"),
             "--jaic", str(compiler), "--stdlib", str(root)]
    sweep += ["--interp", "--wasm"] if args.native else ["--interp-only"]
    try:
        subprocess.run(sweep, cwd=ROOT, env=env, check=True)
    except (subprocess.CalledProcessError, OSError) as error:
        if not args.keep_going:
            raise
        failures.append("float-power-sweep exact-rational audit")
        print("FAILED specialized float-power audit:", error, flush=True)
    for index, (source, expected) in enumerate(cases):
        try:
            run([str(compiler), "-interp", str(source), "--"], env, expected,
                expected_stderr=EXPECTED_STDERR.get(source.stem,""), expected_code=EXPECTED_EXIT.get(source.stem, 0))
            if args.native:
                executable = output / f"case-{index}"
                run([str(compiler), str(source), "-exe", str(executable)], env)
                run([str(executable)], env, expected, source.parent,
                    expected_stderr=EXPECTED_STDERR.get(source.stem,""), expected_code=EXPECTED_EXIT.get(source.stem, 0))
            print("passed", source.relative_to(ROOT), flush=True)
        except (RuntimeError, subprocess.TimeoutExpired) as error:
            if not args.keep_going: raise
            failures.append(str(source.relative_to(ROOT)))
            print("FAILED case; continuing independent tests:", source.relative_to(ROOT), str(error), flush=True)
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
    for name, diagnostic in [
        ("runtime-configuration-conflict", "conflicts with the configured runtime instance"),
        ("xar-reverse", "reverse Xar iteration is unsupported"),
        ("atomic-byte-bit", "atomic bit test requires a 2-, 4-, or 8-byte target"),
        ("buildcpp-compile", "BuildCpp tool failed"),
        ("buildcpp-tool", "BuildCpp tool failed"),
        ("buildcpp-target", "BuildCpp object target validation failed"),
        ("mpeg-scalar-store", "cannot assign to constant"),
        ("adpcm-field-visibility", "no exported member `nChannels`"),
        ("mpeg-quantizer-store", "cannot assign into the constant `M.QUANT_TAB`"),
        ("mpeg-window-store", "cannot assign into the constant `M.SYNTHESIS_WINDOW`"),
        ("unicode-offsets-store", "cannot assign into the constant `U.offsetsFromUTF8`"),
        ("unicode-first-byte-store", "cannot assign into the constant `U.firstByteMark`"),
        ("adpcm-step-store", "cannot assign into the constant `A.step_table`"),
        ("adpcm-index-store", "cannot assign into the constant `A.index_table`"),
        ("register-scalar", "expected a SIMD register operand"),
        ("meow-lane", "INDEX >= 0 && INDEX < 2"),
    ]:
        negative = ROOT / f"tests/portable-stdlib/negative/{name}.jai"
        result = subprocess.run([str(compiler), "-check", str(negative)], cwd=ROOT,
                                env=env, text=True, capture_output=True, timeout=45)
        if result.returncode == 0 or diagnostic not in result.stderr:
            raise RuntimeError(f"expected compile-time rejection: {result.stdout}{result.stderr}")
        print("passed compile-time rejection", name, flush=True)
    if (output / "cpp-failure/old").read_text() != "preserved":
        raise RuntimeError("failed C compilation replaced the existing output")
    leftovers = list(output.rglob("*.build-*")) + list(output.rglob("*.universal-*"))
    if leftovers:
        raise RuntimeError(f"C helper left intermediate directories: {leftovers}")
    mail = [sys.executable, "-B", str(ROOT / "tools/check_portable_mail.py"),
            "--jaic", str(compiler), "--stdlib", str(root)]
    if args.native:
        mail.append("--native")
    try:
        subprocess.run(mail, cwd=ROOT, env=env, check=True)
    except subprocess.CalledProcessError:
        if not args.keep_going: raise
        failures.append("SMTP/MIME integration")
    channels = [sys.executable, "-B", str(ROOT / "tools/check_portable_channels.py"),
                "--jaic", str(compiler), "--stdlib", str(root)]
    if args.native:
        channels.append("--native")
    try:
        subprocess.run(channels, cwd=ROOT, env=env, check=True)
    except subprocess.CalledProcessError:
        if not args.keep_going: raise
        failures.append("Shared-memory interprocess integration")
    if args.native:
        subprocess.run([sys.executable, "-B", str(ROOT / "tools/check_portable_crash.py"),
                        "--jaic", str(compiler), "--stdlib", str(root)], cwd=ROOT, env=env, check=True)
    dependent = {"file", "filesystem", "socket-loopback", "process", "process-io", "system", "text-handler", "zip-directory", "buildcpp", "shared-channel", "autorun"}
    posix_only = {"module-identity", "thread-lifetime", "thread-groups", "thread-condition", "thread-shutdown", "overwriting-allocator", "unmapping-allocator", "crash-handler-init", "atomic-cas-threads", "file-visitation"}
    for target in args.check_target:
        os_name, cpu = target.split("/", 1)
        before = len(failures)
        target_cases = cases + ([(ROOT / "tests/portable-stdlib/windows-foundations.jai", "ok\n")] if os_name == "windows" else [])
        for source, _ in target_cases:
            if source.stem == "default-allocator" and cpu == "x64":
                continue  # Optimized heap branch is explicitly unsupported, not fallback coverage.
            if source.parent.name != "portable-stdlib" or source.stem in dependent or (os_name == "windows" and source.stem in posix_only):
                continue
            try:
                run([str(compiler), "-check", "-os", os_name, "-cpu", cpu, str(source)], env)
            except (RuntimeError, subprocess.TimeoutExpired) as error:
                if not args.keep_going: raise
                failures.append(f"target {target}: {source.relative_to(ROOT)}")
                print("FAILED target case; continuing independent checks:", target, source.relative_to(ROOT), str(error), flush=True)
        print("checked portable cases" if len(failures) == before else "FAILED portable target cases", target, flush=True)
    print(f"{len(cases) - sum(f.startswith('tests/') for f in failures)}/{len(cases)} focused cases passed; native={args.native}")
    if failures:
        raise RuntimeError("Failed checks: " + ", ".join(failures))


if __name__ == "__main__":
    main()
