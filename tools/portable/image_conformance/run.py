"""Run complete pinned PNG/BMP/PNM/QOI/ICO corpora and independent encoder checks.

    PYTHONPATH=tools python3 tools/portable/image_conformance/run.py --jaic target/debug/jaic --output target/image-conformance

The default runs native ASan/UBSan, optimized native, and actual Node WASI. A focused
--modes invocation is diagnostic evidence only for the modes it names.
"""

import argparse
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
import sys

from bench.measure import run_measured
from fetch import fetch_all
from toolkit.hashing import sha256_file
from toolkit.repo import ROOT, cargo_binary

HERE = Path(__file__).resolve().parent
MODES = ("native-asan", "native-o2", "wasi-o2")


def run(command, *, environment, log, allow_failure=False, timeout=None):
    """Capture one required command; e.g. run([probe, inputs], environment=env, log=stream)."""
    start = log.tell()
    log.write("COMMAND " + " ".join(map(str, command)) + "\n")
    log.flush()
    result = run_measured(
        list(map(str, command)),
        cwd=ROOT,
        env=environment,
        timeout=timeout or math.inf,
        output=log,
    )
    code = "timeout" if result.timed_out else result.returncode
    if result.timed_out:
        log.write("TIMEOUT owned process group terminated\n")
    log.flush()
    log.seek(start)
    transcript = log.read()
    log.seek(0, 2)
    if code != 0 and not allow_failure:
        raise subprocess.CalledProcessError(code, command, output=transcript)
    return transcript, code


def observed_result(codec, text):
    """Decode one probe result; e.g. observed_result('ico_corpus', 'false unsupported ICO DIB')."""
    if text is None:
        return {"executed": False}
    fields = text.split(" ")
    result = {"executed": True, "accepted": fields[0] in ("1", "true"), "raw": text}
    if codec == "core_png":
        result.update(
            direct_accepted=fields[1] == "true",
            width=int(fields[2]),
            height=int(fields[3]),
            error=" ".join(fields[4:]),
        )
    elif codec == "decode_corpus":
        result.update(
            width=int(fields[1]), height=int(fields[2]), error=" ".join(fields[3:])
        )
    else:
        result["error"] = " ".join(fields[1:])
    return result


def independent_outputs(root, mode, cases):
    """Decode every encoder result independently, including all ten PNG levels."""
    from PIL import Image

    failures = []
    checked = 0
    for case in cases:
        if not case["valid"]:
            continue
        original = (root / "oracle" / (case["name"] + ".rgba")).read_bytes()
        opaque = bytearray(original)
        opaque[3::4] = bytes([255]) * (len(opaque) // 4)
        for variant in ("direct", "bgra"):
            suffixes = [f"{variant}.{level}.png" for level in range(10)]
            suffixes += [f"{variant}.{fmt}" for fmt in ("bmp", "qoi", "pnm")]
            for suffix in suffixes:
                name = case["name"] + "." + suffix
                try:
                    with Image.open(root / ("core-" + mode) / name) as image:
                        expected = (
                            bytes(opaque) if suffix.endswith(".pnm") else original
                        )
                        if (
                            image.size != (case["width"], case["height"])
                            or image.convert("RGBA").tobytes() != expected
                        ):
                            failures.append(name + ": independent pixels")
                        checked += 1
                except Exception as error:
                    failures.append(name + ": " + str(error))
    return {"outputs": checked, "failures": failures}


def main():
    """Prepare verified corpora and run every named mode; see the module command example."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--modes", default=",".join(MODES))
    parser.add_argument("--expected-source")
    args = parser.parse_args()
    modes = args.modes.split(",")
    if (
        not modes
        or len(set(modes)) != len(modes)
        or any(mode not in MODES for mode in modes)
    ):
        parser.error("--modes must be distinct members of " + ",".join(MODES))
    root = args.output.resolve()
    root.mkdir(parents=True, exist_ok=True)
    if (root / "result.json").exists():
        parser.error(
            "--output already contains a result; preserve its ledger and choose a fresh directory"
        )
    compiler = (args.jaic or cargo_binary("jaic")).resolve(strict=True)
    head = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
    ).strip()
    dirty = bool(
        subprocess.check_output(
            ["git", "status", "--porcelain"], cwd=ROOT, text=True
        ).strip()
    )
    if args.expected_source and (head != args.expected_source or dirty):
        parser.error("source must be clean at --expected-source")
    environment = dict(
        os.environ,
        JAIC_STDLIB=str(ROOT / "stdlib"),
        ASAN_OPTIONS="detect_leaks=0:abort_on_error=1",
        UBSAN_OPTIONS="halt_on_error=1",
    )
    manifest = fetch_all(root)
    for template in ("core_png.jai", "decode_corpus.jai", "ico_corpus.jai"):
        shutil.copyfile(HERE / template, root / template)
    receipt = {
        "source": head,
        "dirty": dirty,
        "image_source_hashes": {
            str(path.relative_to(ROOT)): sha256_file(path)
            for path in (ROOT / "stdlib/Extensions/Image").rglob("*.jai")
        },
        "harness_hashes": {
            str(path.relative_to(HERE)): sha256_file(path)
            for path in HERE.rglob("*")
            if path.is_file()
            and path.suffix in (".py", ".jai", ".ts", ".json", ".c", ".txt")
        },
        "compiler_sha256": sha256_file(compiler),
        "manifest_sha256": sha256_file(HERE / "manifest.json"),
        "modes": {},
        "coverage": "FULL" if set(modes) == set(MODES) else "PARTIAL",
        "leak_detection": False,
        "passed": False,
    }
    try:
        with (root / "commands.log").open("w+") as log:
            for script in (
                "prepare_png.py",
                "prepare_decode.py",
                "prepare_qoi.py",
                "prepare_ico.py",
            ):
                run(
                    [sys.executable, HERE / script, root],
                    environment=environment,
                    log=log,
                )
            for mode in modes:
                mode_result = {"passed": False, "cases": {}, "process_exits": {}}
                receipt["modes"][mode] = mode_result
                for stem, arguments in (
                    (
                        "core_png",
                        (root / "inputs", root / ("core-" + mode), root / "oracle"),
                    ),
                    (
                        "decode_corpus",
                        (
                            root / "decode-inputs",
                            root / ("decode-" + mode),
                            root / "decode-oracle",
                        ),
                    ),
                    ("ico_corpus", (root / "ico-inputs", root / "ico-oracle")),
                ):
                    for directory in arguments:
                        directory.mkdir(exist_ok=True)
                    binary = root / (
                        stem + "-" + mode + (".wasm" if mode == "wasi-o2" else "")
                    )
                    flags = (
                        ["-O0", "-sanitize", "address,undefined"]
                        if mode == "native-asan"
                        else ["-O2"]
                    )
                    if mode == "wasi-o2":
                        flags += ["-os", "wasm"]
                    run(
                        [compiler, *flags, root / (stem + ".jai"), "-exe", binary],
                        environment=environment,
                        log=log,
                    )
                    command = [binary, *arguments]
                    if mode == "wasi-o2":
                        command = [
                            "node",
                            "--no-warnings",
                            HERE / "wasi_corpus.ts",
                            *command,
                        ]
                    transcript, exit_code = run(
                        command,
                        environment=environment,
                        log=log,
                        allow_failure=True,
                        timeout=600,
                    )
                    mode_result["process_exits"][stem] = exit_code
                    filename = {
                        "core_png": "png-cases.json",
                        "decode_corpus": "decode-cases.json",
                        "ico_corpus": "ico-cases.json",
                    }[stem]
                    expected_cases = json.loads((root / filename).read_text())
                    actual = {}
                    completed = []
                    diagnostics = []
                    for line in transcript.splitlines():
                        if line.startswith("DONE "):
                            completed.append(line.removeprefix("DONE "))
                        if line.startswith("FAIL "):
                            diagnostics.append(line)
                        if line.startswith("RESULT "):
                            fields = line.split(" ", 2)
                            if fields[1] in actual:
                                raise AssertionError("duplicate result: " + fields[1])
                            actual[fields[1]] = fields[2]
                    mode_result["cases"][stem] = [
                        dict(
                            case,
                            observed=dict(
                                observed_result(stem, actual.get(case["name"])),
                                completed=case["name"] in completed,
                                diagnostics=[
                                    line
                                    for line in diagnostics
                                    if line.startswith("FAIL " + case["name"] + ":")
                                ],
                            ),
                        )
                        for case in expected_cases
                    ]
                    if (
                        exit_code
                        or set(actual) != {case["name"] for case in expected_cases}
                        or set(completed) != set(actual)
                        or len(completed) != len(actual)
                    ):
                        raise AssertionError(
                            f"{stem}: exit {exit_code}, incomplete or failed probe; inspect case ledger and commands.log"
                        )
                mode_result.update(
                    independent_outputs(root, mode, manifest["png_cases"])
                )
                if mode_result["failures"]:
                    raise AssertionError(mode_result["failures"])
                mode_result["passed"] = True
            for script in ("check_framing.py", "check_qoi.py"):
                run(
                    [sys.executable, HERE / script, root, *modes],
                    environment=environment,
                    log=log,
                )
        receipt["passed"] = True
    except BaseException as error:
        receipt["error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        (root / "result.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(
        json.dumps(
            {
                "source": head,
                "dirty": dirty,
                "passed": receipt["passed"],
                "coverage": receipt["coverage"],
                "modes": {
                    mode: {
                        "passed": item["passed"],
                        "independent_encoder_outputs": item["outputs"],
                        "cases": {
                            codec: len(cases) for codec, cases in item["cases"].items()
                        },
                    }
                    for mode, item in receipt["modes"].items()
                },
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
