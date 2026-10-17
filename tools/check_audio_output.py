#!/usr/bin/env python3
"""Check what the stdlib runner's rows cannot say: real devices and multichannel formats.

`stdlib_runtime.py` runs every Sound_Player case and accepts either outcome of a case that opens
a sound device ("opened" or "unavailable"). This script is for a host that must have a device:

* `--require-device` fails when any device case reports it could not open one;
* `--require-multichannel` also requires real logical 6- and 8-channel streams from the
  controlled-output case, run in the interpreter and as `-O0` and `-O2` builds.

Without either flag it still runs the 6- and 8-channel variants and prints how many opened.
"""
import argparse
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
TESTS = ROOT / "stdlib/Sound_Player/tests"
DEVICE_CASES = [
    "device-lifecycle", "streams", "compressed-stream", "entity-phase", "entity-lifecycle",
    "concurrent-render-stop", "concurrent-factories", "region-lifetime",
]
UNAVAILABLE = "native-device-unavailable\nok\n"


def run(command, env, cwd=ROOT):
    result = subprocess.run(list(map(str, command)), cwd=cwd, env=env, capture_output=True,
                            text=True, timeout=120)
    if result.returncode or result.stderr:
        raise RuntimeError(f"{Path(str(command[0])).name} {command[1:3]}: exit {result.returncode}\n"
                           f"{result.stderr[-800:]}")
    return result.stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--jaic", type=Path, default=ROOT / "target/debug/jaic")
    parser.add_argument("--native-libs", type=Path, default=ROOT / "target/audio-native")
    parser.add_argument("--require-device", action="store_true")
    parser.add_argument("--require-multichannel", action="store_true")
    args = parser.parse_args()
    compiler = args.jaic.resolve(strict=True)
    env = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"),
               JAIC_NATIVE_LIBS=str(args.native_libs.resolve(strict=True)))
    if args.require_device:
        for name in DEVICE_CASES:
            if run([compiler, TESTS / (name + ".jai"), "-interp"], env) == UNAVAILABLE:
                raise RuntimeError(f"{name}: a sound device is required but none could be opened")
            print(name + ": device opened", flush=True)
    controlled = TESTS / "controlled-output.jai"
    with tempfile.TemporaryDirectory(prefix="audio-output-") as scratch:
        for channels in (6, 8):
            channel_env = dict(env, JAI_AUDIO_CHANNELS=str(channels))
            opened = f"native-{channels}-channel-output-opened\nok\n"
            extra = ["-add", f"EXPECTED_CHANNELS :: {channels};"]
            seen = [run([compiler, controlled, "-interp", *extra], channel_env)]
            for level in ("O0", "O2"):
                executable = Path(scratch) / f"controlled-{channels}-{level}{'.exe' if os.name == 'nt' else ''}"
                run([compiler, controlled, "-" + level, *extra, "-exe", executable], channel_env)
                seen.append(run([executable], channel_env, cwd=controlled.parent))
            for output in seen:
                if output not in (opened, UNAVAILABLE):
                    raise RuntimeError(f"{channels}-channel controlled output printed {output!r}")
            count = seen.count(opened)
            if args.require_multichannel and count != len(seen):
                raise RuntimeError(f"{channels}-channel format unavailable in {len(seen) - count} of "
                                   f"{len(seen)} runs")
            print(f"{channels}-channel controlled output: {count} of {len(seen)} runs opened", flush=True)


if __name__ == "__main__":
    main()
