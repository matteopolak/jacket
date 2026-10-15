#!/usr/bin/env python3
"""Verify this audio worker's explicit scope using its own source-built tools."""
import os
import argparse
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--jaic", type=Path, default=ROOT / "target/debug/jaic")
parser.add_argument("--jailint", type=Path, default=ROOT / "target/debug/jailint")
parser.add_argument("--jaifmt", type=Path, default=ROOT / "target/jaifmt")
parser.add_argument("--stdlib", type=Path, default=ROOT / "stdlib")
parser.add_argument("--native-libs", type=Path, default=ROOT / "target/audio-native")
parser.add_argument("--require-device", action="store_true", help="fail when a base native device is unavailable")
parser.add_argument("--require-multichannel", action="store_true", help="also require real logical 6/8-channel streams")
args = parser.parse_args()
COMPILER = args.jaic.resolve(strict=True)
ENV = dict(os.environ, JAIC_STDLIB=str(args.stdlib.resolve(strict=True)),
           JAIC_NATIVE_LIBS=str(args.native_libs.resolve(strict=True)))
CASES = sorted((ROOT / "tests/audio").glob("*.jai"))
NEGATIVE = sorted((ROOT / "tests/audio/negative").glob("*.jai"))
OWNED = [ROOT / "stdlib/Sound_Player/module.jai"]
OWNED += sorted(p for p in (ROOT / "stdlib/internal/audio").glob("*.jai")
                if p.name != "wav.jai")
OWNED += CASES


def run(command, expected=None, cwd=ROOT, env=ENV):
    result = subprocess.run(list(map(str, command)), cwd=cwd, env=env,
                            capture_output=True, text=True, timeout=60)
    matches = expected(result.stdout) if callable(expected) else expected is None or result.stdout == expected
    if result.returncode or result.stderr or not matches:
        headlines = [line for line in (result.stderr + result.stdout).splitlines()
                     if "error:" in line or line.startswith("error[") or line.lstrip().startswith("-->")
                     or line.startswith("jailint:")]
        raise RuntimeError(f"{Path(str(command[0])).name}: exit {result.returncode}; "
                           + "; ".join(headlines[:8]) + ("; unexpected stdout" if not matches else ""))
    return result.stdout


def main():
    manifest = json.loads((ROOT / "tests/audio/manifest.json").read_text())
    declared = {ROOT / entry["path"] for entry in manifest["cases"]}
    if declared != set(CASES):
        raise RuntimeError("Audio fixture/provenance manifest differs from discovered NEW tests")
    if {ROOT / entry["path"] for entry in manifest["negative"]} != set(NEGATIVE):
        raise RuntimeError("Audio negative fixture/provenance manifest differs from discovered tests")
    output = ROOT / "target/audio-tests"
    output.mkdir(parents=True, exist_ok=True)
    run([args.jaifmt.resolve(strict=True), "--check", *OWNED, *NEGATIVE])
    windows_paths = [p for p in OWNED if p.stem == "windows-abi"]
    common_paths = [p for p in OWNED if p not in windows_paths]
    run([args.jailint.resolve(strict=True), "-D", "warnings", *common_paths])
    for cpu in ("x64", "arm64"):
        run([args.jailint.resolve(strict=True), "-D", "warnings", "--os", "windows", "--cpu", cpu,
             ROOT / "stdlib/Sound_Player/module.jai", ROOT / "tests/audio/signatures.jai", *windows_paths])
    print("source-built jaifmt --check and warnings-denied jailint passed", flush=True)
    for case in CASES:
        if case.stem == "windows-abi":
            for cpu in ("x64", "arm64"):
                run([COMPILER, case, "-check", "-os", "windows", "-cpu", cpu], "")
            print("windows-abi.jai: Windows x64/ARM64 static ABI checks; native execution unavailable", flush=True)
            continue
        expected = "ok\n"
        if case.stem in {"device-lifecycle", "streams", "compressed-stream", "entity-phase", "entity-lifecycle", "concurrent-render-stop", "concurrent-factories", "region-lifetime"}:
            expected = lambda out: out in {"native-device-opened\nok\n", "native-device-unavailable\nok\n"}
        if case.stem == "controlled-output":
            expected = lambda out: out in {"native-2-channel-output-opened\nok\n", "native-6-channel-output-opened\nok\n", "native-8-channel-output-opened\nok\n", "native-device-unavailable\nok\n"}
        interpreted = run([COMPILER, case, "-interp"], expected)
        if args.require_device and "native-device-unavailable" in interpreted:
            raise RuntimeError(f"{case.name}: required native device unavailable; no playback pass recorded")
        for level in ("O0", "O2"):
            executable = output / (f"{case.stem}-{level}" + (".exe" if os.name == "nt" else ""))
            run([COMPILER, case, "-" + level, "-exe", executable], "")
            native_evidence = run([executable], expected, cwd=case.parent)
            if args.require_device and "native-device-unavailable" in native_evidence:
                raise RuntimeError(f"{case.name}: required native device unavailable; no playback pass recorded")
            if callable(expected):
                print(f"  {level}: {native_evidence.splitlines()[0]}", flush=True)
        for target_os in ("linux", "windows", "macos"):
            for cpu in ("x64", "arm64"):
                run([COMPILER, case, "-check", "-os", target_os, "-cpu", cpu], "")
        label = {"signatures": "signature/link probes",
                 "history": "public call smoke test; private extrema have no accessor"}.get(case.stem, "behavior test")
        print(f"{case.name}: {label}, interpreter/native O0/O2 and six target checks passed", flush=True)
    controlled = ROOT / "tests/audio/controlled-output.jai"
    for channels in (6, 8):
        env = dict(ENV, JAI_AUDIO_CHANNELS=str(channels))
        opened = f"native-{channels}-channel-output-opened\nok\n"
        expected = lambda out: out in (opened, "native-device-unavailable\nok\n")
        extra = ["-add", f"EXPECTED_CHANNELS :: {channels};"]
        observed = [run([COMPILER, controlled, "-interp", *extra], expected, env=env)]
        for level in ("O0", "O2"):
            executable = output / (f"controlled-{channels}-{level}" + (".exe" if os.name == "nt" else ""))
            run([COMPILER, controlled, "-" + level, *extra, "-exe", executable], "")
            observed.append(run([executable], expected, cwd=controlled.parent, env=env))
        count = observed.count(opened)
        if args.require_multichannel and count != 3:
            raise RuntimeError(f"{channels}-channel SDK format unavailable in {3-count} execution mode(s); no multichannel playback pass recorded")
        print(f"{channels}-channel SDK/controlled-buffer fixture: {count}/3 opened, {3-count}/3 unavailable; physical multichannel unverified", flush=True)
    for case in NEGATIVE:
        result = subprocess.run([COMPILER, case, "-check"], cwd=ROOT, env=ENV,
                                capture_output=True, text=True, timeout=60)
        expected = {"private-mutex": "sound_mutex", "private-player-helper": "lock_player",
                    "private-producer": "lock_producer"}[case.stem]
        if result.returncode != 1 or f"unknown identifier `{expected}`" not in result.stderr:
            raise RuntimeError(f"{case.name}: expected private API rejection was not observed")
        print(f"{case.name}: expected private API rejection passed", flush=True)


if __name__ == "__main__":
    main()
