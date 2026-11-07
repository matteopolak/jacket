#!/usr/bin/env python3
"""Full pinned JPEG differential gate. Fetch first; run only against a reviewed candidate."""

import argparse
import collections
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import shutil
import subprocess
import sys
import time

from toolkit.repo import cargo_target_dir

MODES = ("native-asan", "native-o2", "wasi-o2")
POLICY = {
    "maximum_absolute_error": 2,
    "maximum_mean_error": 0.5,
    "idct": "JDCT_ISLOW",
    "fancy_upsampling": True,
    "cmyk": "unprofiled conventional; Adobe marker selects inverted components",
    "official_T83": "unavailable paid document/data; no certification claim",
}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save(path, value):
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, indent=2) + "\n")
    temporary.replace(path)


def parse_args():
    ap = argparse.ArgumentParser(description=__doc__)
    for name in (
        "repo",
        "jaic",
        "corpus",
        "output",
        "reference-include",
        "reference-library",
    ):
        ap.add_argument("--" + name, type=Path, required=True)
    ap.add_argument(
        "--manifest", type=Path, default=Path(__file__).with_name("manifest.json")
    )
    ap.add_argument(
        "--heavy",
        type=Path,
        help="Optional local admission wrapper; CI may wrap externally or omit.",
    )
    ap.add_argument(
        "--expected-source",
        help="Optional exact Git HEAD receipt; dirty state is always recorded.",
    )
    ap.add_argument("--expected-jaic-sha256")
    ap.add_argument(
        "--expected-reference-library-sha256",
        help="Requires --reference-library to name the actual linked file.",
    )
    ap.add_argument(
        "--reference-version",
        default="3.2.0",
        help="Pinned expected libjpeg-turbo API/header version.",
    )
    ap.add_argument("--cc", default="cc")
    ap.add_argument("--node", default="node")
    ap.add_argument(
        "--modes",
        default=",".join(MODES),
        help="Focused runs remain PARTIAL unless all three modes execute.",
    )
    ap.add_argument("--cpu-seconds", type=int, default=180)
    ap.add_argument("--wall-seconds", type=int, default=240)
    ap.add_argument("--file-bytes", type=int, default=400_000_000)
    ap.add_argument(
        "--reviewed-dispositions",
        type=Path,
        help="Optional owner/root-reviewed exact-hash classifications; no unreviewed weakening.",
    )
    return ap.parse_args()


def process(command, *, cwd, env, timeout, sandbox=None, cpu=0, file_bytes=0):
    def bounds():
        import resource

        resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
        if cpu:
            resource.setrlimit(resource.RLIMIT_CPU, (cpu, cpu))
        if file_bytes:
            resource.setrlimit(resource.RLIMIT_FSIZE, (file_bytes, file_bytes))

    wrapped = (
        sandbox.wrap(list(map(str, command))) if sandbox else list(map(str, command))
    )
    started = time.monotonic()
    p = subprocess.Popen(
        wrapped,
        cwd=cwd,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        preexec_fn=bounds if os.name == "posix" else None,
        start_new_session=True,
    )
    try:
        stdout, stderr = p.communicate(timeout=timeout)
        code = p.returncode
    except subprocess.TimeoutExpired:
        try:
            os.killpg(p.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        stdout, stderr = p.communicate()
        code = "timeout"
    except BaseException:
        try:
            os.killpg(p.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        p.wait()
        raise
    result = {
        "exit": code,
        "stdout": stdout.decode(errors="replace"),
        "stderr": stderr.decode(errors="replace"),
    }
    result["elapsed_seconds"] = time.monotonic() - started
    return result


def pixel_compare(actual, reference, reference_header, decoded):
    width, height, channels, adobe, color_space = reference_header
    expected = reference.read_bytes()
    if channels == 4:
        rgb = bytearray()
        for start in range(0, len(expected), 4):
            c, m, y, k = expected[start : start + 4]
            rgb.extend(
                ((v * k + 127) // 255 for v in (c, m, y))
                if adobe
                else (((255 - v) * (255 - k) + 127) // 255 for v in (c, m, y))
            )
        expected = bytes(rgb)
        channels = 3
    if (width, height, channels) != (
        decoded["width"],
        decoded["height"],
        decoded["channels"],
    ):
        return {
            "pass": False,
            "reason": "shape mismatch",
            "reference_shape": [width, height, channels],
        }
    if actual.stat().st_size != len(expected):
        return {"pass": False, "reason": "byte length mismatch"}
    total = 0
    maximum = 0
    with actual.open("rb") as stream:
        for start in range(0, len(expected), 65536):
            data = stream.read(65536)
            delta = [abs(a - b) for a, b in zip(data, expected[start : start + 65536])]
            total += sum(delta)
            maximum = max(maximum, max(delta, default=0))
    return {
        "samples": len(expected),
        "max_error": maximum,
        "mean_error": total / max(1, len(expected)),
        "pass": maximum <= 2 and total * 2 <= len(expected),
    }


def review_map(path, cases):
    if path is None:
        return {}
    content = json.loads(path.read_text())
    if not content.get("reviewed_by") or not content.get("review_receipt"):
        raise ValueError("dispositions require reviewed_by and review_receipt")
    known = {c["id"]: c for c in cases}
    result = {}
    for entry in content["cases"]:
        if entry["id"] in result or entry["id"] not in known:
            raise ValueError("duplicate or unknown disposition case")
        if entry["sha256"] != known[entry["id"]]["sha256"] or not entry.get("reason"):
            raise ValueError("disposition hash/reason mismatch")
        if entry["expectation"] not in (
            "reject",
            "accept",
            "metadata-outside-core",
            "documented-recovery",
            "suite-label-inconsistency",
            "retained-compatibility-extension",
        ):
            raise ValueError("unknown disposition expectation")
        result[entry["id"]] = entry
    return result


def classify(case, result, reference, disposition):
    failures, review = [], []
    if result["process"]["exit"] != 0:
        return ["decoder-process-error"], []
    if not decision_known(result):
        failures.append("decoder-result-missing-or-ambiguous")
    if result["sanitizer_diagnostic"]:
        failures.append("sanitizer-diagnostic")
    if result.get("parity") is not True:
        failures.append("direct-detected-parity")
    if result["accepted"]:
        for key in ("invariant", "cap_minus_one", "cap_exact"):
            if result.get(key) is not True:
                failures.append(key)
    elif result.get("reject_empty") is not True:
        failures.append("rejection-returned-pixels")
    if case["id"] in result["above_cap_ids"] and result["accepted"]:
        failures.append("above-production-cap-accepted")
    if (
        case["id"] in result["above_cap_ids"]
        and case["class"] != "invalid"
        and not result.get("cap_rejection")
    ):
        failures.append("above-production-cap-not-rejected-by-cap")
    expectation = disposition["expectation"] if disposition else case["expectation"]
    if expectation == "reject" and result["accepted"]:
        failures.append("strict-required-rejection-accepted")
    if (
        expectation in ("accept", "retained-compatibility-extension")
        and not result["accepted"]
    ):
        failures.append("required-acceptance-rejected")
    if (
        not case["unsupported_process"]
        and not result["accepted"]
        and reference["clean"]
        and reference["exit"] == 0
    ):
        if expectation not in ("reject",):
            failures.append("clean-reference-supported-image-rejected")
    comparison = result.get("pixels")
    if result["accepted"] and reference["clean"] and comparison is None:
        failures.append("clean-reference-pixels-unavailable")
    if comparison and not comparison["pass"]:
        if reference["clean"]:
            failures.append("clean-reference-pixel-mismatch")
        elif not disposition or expectation != "documented-recovery":
            review.append("damaged-recovery-pixel-mismatch")
    if result["accepted"] and reference["exit"] != 0:
        if not disposition or expectation not in (
            "documented-recovery",
            "suite-label-inconsistency",
        ):
            review.append("accepted-without-successful-independent-pixels")
    if reference.get("warning_regression"):
        failures.append("independent-reference-warning-change")
    if case["class"] == "invalid" and result["accepted"] and disposition:
        # A reviewed exception remains visible; it never becomes a strict rejection pass.
        review.append("reviewed-invalid-class-acceptance:" + expectation)
    return failures, review


def result_match(transcript):
    """Require one complete result line; duplicate or malformed evidence is unknown."""
    lines = [line for line in transcript.splitlines() if line.startswith("RESULT")]
    return (
        re.fullmatch(r"RESULT (true|false) (\d+) (\d+) (\d+) (.*)", lines[0])
        if len(lines) == 1
        else None
    )


def decision_known(row):
    """Only a completed probe with RESULT establishes acceptance; a launch error is unknown."""
    return row["process"]["exit"] == 0 and row.get("result_observed", False)


def main():
    args = parse_args()
    if args.heavy and os.environ.get("JPEG_FINAL_GATE_ADMITTED") != "1":
        env = dict(os.environ, JPEG_FINAL_GATE_ADMITTED="1")
        return subprocess.run(
            [
                sys.executable,
                str(args.heavy),
                "--",
                sys.executable,
                str(Path(__file__).resolve()),
                *sys.argv[1:],
            ],
            env=env,
        ).returncode
    if os.name != "posix":
        raise SystemExit(
            "This sanitizer gate requires a POSIX host; no silent Windows skip."
        )
    for key in (
        "repo",
        "jaic",
        "corpus",
        "output",
        "reference_include",
        "reference_library",
        "manifest",
    ):
        setattr(args, key, getattr(args, key).resolve())
    modes = args.modes.split(",")
    if len(modes) != len(set(modes)) or any(mode not in MODES for mode in modes):
        raise SystemExit("Invalid/duplicate mode; use native-asan,native-o2,wasi-o2.")
    manifest = json.loads(args.manifest.read_text())
    cases = manifest["cases"]
    if len(cases) != 258 or len({c["id"] for c in cases}) != 258:
        raise SystemExit("Pinned full inventory must contain 258 unique cases.")
    for item in manifest["sources"]:
        path = args.corpus / item["local_path"]
        if (
            not path.is_file()
            or path.stat().st_size != item["bytes"]
            or digest(path) != item["sha256"]
        ):
            raise SystemExit("Corpus/source integrity failure: " + item["local_path"])
    for case in cases:
        if digest(args.corpus / case["id"]) != case["sha256"]:
            raise SystemExit("Case integrity failure: " + case["id"])
    dispositions = review_map(args.reviewed_dispositions, cases)
    head = subprocess.check_output(
        ["git", "-C", str(args.repo), "rev-parse", "HEAD"], text=True
    ).strip()
    if args.expected_source and head != args.expected_source:
        raise SystemExit("Source HEAD does not match reviewed receipt.")
    if args.expected_jaic_sha256 and digest(args.jaic) != args.expected_jaic_sha256:
        raise SystemExit("Compiler hash does not match reviewed receipt.")
    if args.expected_reference_library_sha256 and (
        not args.reference_library.is_file()
        or digest(args.reference_library) != args.expected_reference_library_sha256
    ):
        raise SystemExit("Reference library file does not match the pinned hash.")
    # Refuse overwrite, including a failed earlier run. Every attempt receives a new directory.
    args.output.mkdir(parents=True, exist_ok=False)
    work = args.output / "work"
    work.mkdir()
    receipt = {
        "source_head": head,
        "compiler_sha256": digest(args.jaic),
        "manifest_sha256": digest(args.manifest),
        "policy": POLICY,
        "modes": modes,
        "source_status": subprocess.check_output(
            ["git", "-C", str(args.repo), "status", "--porcelain"], text=True
        ),
        "image_source_hashes": {
            str(p.relative_to(args.repo)): digest(p)
            for p in (args.repo / "stdlib/Extensions/Image").rglob("*.jai")
        },
        "harness_hashes": {
            p.name: digest(p)
            for p in Path(__file__).parent.iterdir()
            if p.is_file() and p.suffix in (".py", ".jai", ".c", ".json", ".mjs")
        },
        "bounds": {
            "cpu_seconds": args.cpu_seconds,
            "wall_seconds": args.wall_seconds,
            "file_bytes": args.file_bytes,
        },
        "reviewed_dispositions_sha256": digest(args.reviewed_dispositions)
        if args.reviewed_dispositions
        else None,
    }
    save(args.output / "receipt.json", receipt)
    sys.path.insert(0, str(args.repo / "tools/bench"))
    from sandbox import Sandbox

    node = shutil.which(args.node)
    if node is None:
        raise SystemExit("Node executable unavailable: " + args.node)
    args.node = str(Path(node).resolve())
    node_install = Path(args.node).parent.parent
    receipt["node_executable_sha256"] = digest(Path(args.node))
    runtime_sandbox = Sandbox(
        reads=[
            node_install,
            args.corpus,
            args.output,
            Path(__file__).with_name("wasi_probe.mjs"),
            args.reference_include,
            args.reference_library,
        ],
        writes=[work],
    )
    env = dict(
        os.environ,
        JAIC_STDLIB=str(args.repo / "stdlib"),
        CARGO_TARGET_DIR=str(cargo_target_dir(resolve=True)),
    )
    # Address/bounds checking stays enabled. Leak checking is the repository's existing policy.
    env["ASAN_OPTIONS"] = "detect_leaks=0:abort_on_error=1"
    env["UBSAN_OPTIONS"] = "halt_on_error=1"
    here = Path(__file__).resolve().parent
    reference = args.output / "reference"
    reference_link = (
        [str(args.reference_library)]
        if args.reference_library.is_file()
        else ["-L" + str(args.reference_library), "-ljpeg"]
    )
    reference_directory = (
        args.reference_library.parent
        if args.reference_library.is_file()
        else args.reference_library
    )
    reference_env = dict(env)
    if sys.platform.startswith("linux"):
        reference_env["LD_LIBRARY_PATH"] = (
            str(reference_directory) + os.pathsep + env.get("LD_LIBRARY_PATH", "")
        )
    reference_link += ["-Wl,-rpath," + str(reference_directory)]
    if sys.platform.startswith("linux"):
        reference_link += ["-ldl"]
    commands = [
        (
            "reference",
            [
                args.cc,
                "-I" + str(args.reference_include),
                str(here / "reference.c"),
                *reference_link,
                "-o",
                str(reference),
            ],
        )
    ]
    binaries = {}
    for mode in modes:
        binary = args.output / (
            "probe-" + mode + (".wasm" if mode == "wasi-o2" else "")
        )
        binaries[mode] = binary
        flags = (
            ["-O0", "-sanitize", "address,undefined"]
            if mode == "native-asan"
            else ["-O2"]
        )
        if mode == "wasi-o2":
            flags += ["-os", "wasm"]
        commands.append(
            (
                mode,
                [str(args.jaic), *flags, str(here / "probe.jai"), "-exe", str(binary)],
            )
        )
    builds = []
    for name, command in commands:
        # Compilation can spawn linkers: its ordinary bound is the whole CI job, not a parent-only timer.
        result = process(command, cwd=args.repo, env=env, timeout=None)
        builds.append({"mode": name, "command": command, **result})
        save(args.output / "builds.json", builds)
        if result["exit"] != 0:
            save(
                args.output / "summary.json",
                {"status": "ERROR", "reason": "build failed", "mode": name},
            )
            return 2
    version = process(
        [reference, "--version"],
        cwd=work,
        env=reference_env,
        timeout=10,
        sandbox=runtime_sandbox,
    )
    receipt["reference_version"] = version
    receipt["reference_header_hashes"] = {
        p.name: digest(p) for p in args.reference_include.glob("j*.h")
    }
    reference_files = (
        [args.reference_library]
        if args.reference_library.is_file()
        else list(args.reference_library.glob("libjpeg*"))
    )
    receipt["reference_library_hashes"] = {
        p.name: digest(p) for p in reference_files if p.is_file()
    }
    loaded_match = re.search(r"^LIBRARY (.+)$", version["stdout"], re.M)
    loaded_library = Path(loaded_match[1]).resolve() if loaded_match else None
    allowed_libraries = {path.resolve() for path in reference_files if path.is_file()}
    runtime_library_matches = loaded_library in allowed_libraries
    receipt["runtime_reference_library_matches"] = runtime_library_matches
    if runtime_library_matches:
        receipt["runtime_reference_library_sha256"] = digest(loaded_library)
    receipt["binary_hashes"] = {name: digest(path) for name, path in binaries.items()}
    save(args.output / "receipt.json", receipt)
    if (
        version["exit"] != 0
        or not runtime_library_matches
        or ("libjpeg-turbo " + args.reference_version + " ") not in version["stdout"]
    ):
        save(
            args.output / "summary.json",
            {"status": "ERROR", "reason": "reference version mismatch"},
        )
        return 2
    rows = []
    above_cap_ids = manifest["above_production_cap_cases"]
    reference_default_limit = manifest["production_default_pixels"]
    for number, case in enumerate(cases, 1):
        input_path = args.corpus / case["id"]
        ref_path = work / "reference.raw"
        ref_path.unlink(missing_ok=True)
        ref_process = process(
            [reference, input_path, ref_path, str(reference_default_limit)],
            cwd=work,
            env=reference_env,
            timeout=args.wall_seconds,
            sandbox=runtime_sandbox,
            cpu=args.cpu_seconds,
            file_bytes=args.file_bytes,
        )
        match = re.search(
            r"^REF (\d+) (\d+) (\d+) (\d+) (\d+)$", ref_process["stdout"], re.M
        )
        warning_match = re.search(r"^WARN (\d+)$", ref_process["stdout"], re.M)
        warning_count = int(warning_match[1]) if warning_match else None
        ref = {
            "exit": ref_process["exit"],
            "warnings": warning_count,
            "clean": ref_process["exit"] == 0
            and warning_count == 0
            and not ref_process["stderr"],
            "process": ref_process,
        }
        baseline = case["baseline_reference"]
        ref["warning_regression"] = (
            ref_process["exit"] != baseline["exit"]
            or warning_count != baseline["warnings"]
            or ref_process["stderr"] != baseline["stderr"]
        )
        for mode in modes:
            actual_path = work / "actual.raw"
            actual_path.unlink(missing_ok=True)
            command = [binaries[mode], input_path, actual_path]
            if mode == "wasi-o2":
                command = [
                    args.node,
                    "--no-warnings",
                    here / "wasi_probe.mjs",
                    binaries[mode],
                    args.corpus,
                    work,
                    case["id"],
                    actual_path.name,
                ]
            p = process(
                command,
                cwd=work,
                env=env,
                timeout=args.wall_seconds,
                sandbox=runtime_sandbox,
                cpu=args.cpu_seconds,
                file_bytes=args.file_bytes,
            )
            match_actual = result_match(p["stdout"])
            row = {
                "case": case["id"],
                "sha256": case["sha256"],
                "class": case["class"],
                "mode": mode,
                "process": p,
                "reference": ref,
                "accepted": False,
                "result_observed": match_actual is not None,
                "sanitizer_diagnostic": bool(
                    re.search(
                        r"AddressSanitizer|UndefinedBehaviorSanitizer|runtime error:",
                        p["stderr"],
                    )
                ),
                "above_cap_ids": above_cap_ids,
            }
            for field, token in [
                ("parity", "PARITY"),
                ("invariant", "INVARIANT"),
                ("cap_minus_one", "CAP_MINUS_ONE"),
                ("cap_exact", "CAP_EXACT"),
                ("reject_empty", "REJECT_EMPTY"),
            ]:
                row[field] = bool(re.search("^" + token + r" true$", p["stdout"], re.M))
            limit = re.search(r"^LIMIT (\d+)$", p["stdout"], re.M)
            row["default_limit"] = int(limit[1]) if limit else None
            if match_actual:
                row["accepted"] = match_actual[1] == "true"
                row["decoded"] = dict(
                    zip(
                        ("width", "height", "channels"),
                        map(int, match_actual.group(2, 3, 4)),
                    )
                )
                row["error"] = match_actual[5]
                row["cap_rejection"] = (
                    not row["accepted"] and "pixel limit" in row["error"]
                )
            if (
                row["accepted"]
                and match
                and ref_process["exit"] == 0
                and actual_path.exists()
                and ref_path.exists()
            ):
                row["pixels"] = pixel_compare(
                    actual_path,
                    ref_path,
                    list(map(int, match.groups())),
                    row["decoded"],
                )
            failures, review = classify(case, row, ref, dispositions.get(case["id"]))
            if row["default_limit"] is None:
                failures.append("production-default-not-observed")
            elif row["default_limit"] != reference_default_limit:
                failures.append("production-default-changed-review-required")
            row.pop("above_cap_ids")
            row["failures"], row["review"] = failures, review
            row["disposition"] = dispositions.get(case["id"])
            row["pixel_sha256"] = digest(actual_path) if actual_path.exists() else None
            row["pixel_reference_class"] = (
                "clean" if ref["clean"] else "warning-bearing-or-unavailable"
            )
            blocking = [
                item
                for item in review
                if not item.startswith("reviewed-invalid-class-acceptance:")
            ]
            row["outcome"] = (
                "error"
                if p["exit"] != 0 or row["sanitizer_diagnostic"]
                else "fail"
                if failures
                else "review"
                if blocking
                else "accepted-clean-reference"
                if row["accepted"] and ref["clean"]
                else "accepted-damaged-or-unverified"
                if row["accepted"]
                else "rejected-above-cap"
                if case["id"] in above_cap_ids
                else "rejected-unsupported-process"
                if case["unsupported_process"]
                else "rejected-external-context-required"
                if (row.get("disposition") or {}).get("classification")
                == "external-context-required"
                else "rejected-structural-or-malformed"
            )
            row["recovery_pixel_mismatch"] = bool(
                row.get("pixels") and not row["pixels"]["pass"] and not ref["clean"]
            )
            rows.append(row)
            save(args.output / "cases.json", rows)
            actual_path.unlink(missing_ok=True)
            print(
                f"{number}/{len(cases)} {mode} {case['id']}: {'ERROR' if not decision_known(row) else 'ACCEPT' if row['accepted'] else 'REJECT'}; "
                f"failures={len(failures)} review={len(review)}",
                flush=True,
            )
        ref_path.unlink(missing_ok=True)
    expected = {(mode, c["id"], c["sha256"]) for mode in modes for c in cases}
    observed = {(r["mode"], r["case"], r["sha256"]) for r in rows}
    summary = {
        "inventory_exact": expected == observed and len(rows) == len(expected),
        "complete_mode_matrix": set(modes) == set(MODES),
        "official_T83": POLICY["official_T83"],
        "modes": {},
        "policy": POLICY,
    }
    unresolved = 0
    for mode in modes:
        selected = [r for r in rows if r["mode"] == mode]
        blocking_review = [
            r
            for r in selected
            if any(
                not item.startswith("reviewed-invalid-class-acceptance:")
                for item in r["review"]
            )
        ]
        failures = sum(bool(r["failures"]) for r in selected)
        unresolved += failures + len(blocking_review)
        summary["modes"][mode] = {
            "cases": len(selected),
            "skipped_cases": 0,
            "outcomes": dict(collections.Counter(r["outcome"] for r in selected)),
            "accepted": sum(decision_known(r) and r["accepted"] for r in selected),
            "rejected": sum(decision_known(r) and not r["accepted"] for r in selected),
            "decision_unverified": sum(not decision_known(r) for r in selected),
            "failed_cases": failures,
            "review_required_cases": len(blocking_review),
            "reviewed_invalid_acceptances": sum(
                r["class"] == "invalid" and r["accepted"] and bool(r["disposition"])
                for r in selected
            ),
            "compatibility_extension_acceptances": sum(
                decision_known(r)
                and r["accepted"]
                and r.get("disposition", {}).get("expectation")
                == "retained-compatibility-extension"
                for r in selected
                if r.get("disposition")
            ),
            "process_errors": sum(r["process"]["exit"] != 0 for r in selected),
            "sanitizer_errors": sum(r["sanitizer_diagnostic"] for r in selected),
            "pixel_pass": sum(r.get("pixels", {}).get("pass", False) for r in selected),
            "pixel_fail": sum(
                r.get("pixels", {}).get("pass") is False for r in selected
            ),
            "damaged_recovery_mismatches": sum(
                r["recovery_pixel_mismatch"] for r in selected
            ),
            "strict_invalid_rejections": sum(
                r["class"] == "invalid" and decision_known(r) and not r["accepted"]
                for r in selected
            ),
        }
    summary["cross_mode_status_differences"] = [
        case["id"]
        for case in cases
        if len(
            {
                r["accepted"]
                for r in rows
                if r["case"] == case["id"] and decision_known(r)
            }
        )
        > 1
    ]
    summary["execution_complete"] = all(decision_known(row) for row in rows)
    summary["cross_mode_unverified_cases"] = sorted(
        {row["case"] for row in rows if not decision_known(row)}
    )
    if not summary["execution_complete"]:
        unresolved += 1
    if summary["cross_mode_status_differences"]:
        unresolved += len(summary["cross_mode_status_differences"])
    summary["status"] = (
        "FAIL" if unresolved else "PASS_WITH_REVIEWED_SCOPE" if dispositions else "PASS"
    )
    if not summary["inventory_exact"] or not summary["complete_mode_matrix"]:
        summary["status"] = "PARTIAL"
    save(args.output / "summary.json", summary)
    print(json.dumps(summary, indent=2))
    return 0 if summary["status"] in ("PASS", "PASS_WITH_REVIEWED_SCOPE") else 1


if __name__ == "__main__":
    raise SystemExit(main())
