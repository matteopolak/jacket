#!/usr/bin/env python3
"""Immutable shared-source checks for allocator domains and live config."""

import argparse
import hashlib
import importlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from shared_snapshot import capture, unchanged


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in (
        "compiler",
        "native_compiler",
        "jailint",
        "formatter",
        "vendor",
        "output_root",
    ):
        parser.add_argument("--" + name.replace("_", "-"), type=Path, required=True)
    args = parser.parse_args()
    names = (
        "rpmalloc-independent-domains",
        "rpmalloc-live-config",
        "rpmalloc-live-config-raw",
    )
    native_names = ("rpmalloc-metadata-failure", "rpmalloc-metadata-diagnostics")
    snapshot, identity = capture(
        args.output_root,
        (
            "tools/native_bindings",
            "tests/native-bindings/generated",
            *(f"tests/native-bindings/{n}.jai" for n in names),
            *(f"tests/native-bindings/{n}.jai" for n in native_names),
            "tests/native-bindings/rpmalloc_metadata_fault.c",
            "tests/native-bindings/rpmalloc_domains_probe.c",
            "tests/native-bindings/rpmalloc_live_config_probe.c",
            "tests/native-bindings/rpmalloc_metadata_probe.c",
            "docs/specs/stdlib/modules/rpmalloc.md",
            "jaifmt.toml",
            "jailint.toml",
        ),
    )
    work = Path(tempfile.mkdtemp(prefix="domains-", dir=args.output_root))
    sys.path.insert(0, str(snapshot / "tools/native_bindings"))
    prepare = importlib.import_module("prepare_rpmalloc").prepare
    builder = importlib.import_module("build_rpmalloc")
    profiles = importlib.import_module("rpmalloc_profiles")
    prepared = prepare(args.vendor, work / "prepared")
    repeated = prepare(args.vendor, work / "reproduced")
    hashes = {p.name: digest(p) for p in prepared.iterdir() if p.is_file()}
    report = dict(
        snapshot=str(snapshot),
        input_identity=identity,
        checks=[],
        tools={
            str(p.resolve()): digest(p.resolve())
            for p in (args.compiler, args.native_compiler, args.jailint, args.formatter)
        },
        native_sdk_provenance=json.loads((prepared / "provenance.json").read_text()),
        scope="Mac ARM64 native; six desktop source checks; other native platforms unverified",
    )
    report["checks"].append(
        dict(
            label="independently reproduced SDK patches",
            status="passed"
            if hashes == {p.name: digest(p) for p in repeated.iterdir() if p.is_file()}
            else "failed",
        )
    )
    env = dict(
        os.environ,
        JAIC_STDLIB=str(snapshot / "stdlib"),
        TMPDIR=str(work),
        PYTHONDONTWRITEBYTECODE="1",
        ASAN_OPTIONS="allocator_may_return_null=1",
    )

    def invoke(label, command, expected=0, diagnostic=None):
        result = subprocess.run(
            [str(x) for x in command],
            cwd=snapshot,
            env=env,
            capture_output=True,
            text=True,
            timeout=180,
        )
        row = dict(
            label=label,
            command=[str(x) for x in command],
            exit=result.returncode,
            stdout=result.stdout,
            stderr=result.stderr,
            expected=expected,
            status="passed"
            if result.returncode == expected
            and (diagnostic is None or diagnostic in result.stderr)
            else "failed",
        )
        report["checks"].append(row)
        (work / "verification.json").write_text(json.dumps(report, indent=2) + "\n")
        print(label, result.returncode, result.stderr[-900:], flush=True)
        return row["status"] == "passed"

    api = json.loads(
        (snapshot / "stdlib/internal/bindings/rpmalloc-sdk/provenance.json").read_text()
    )
    invoke(
        "complete SDK domain regeneration",
        [
            sys.executable,
            snapshot / "tools/native_bindings/generate_rpmalloc_domains.py",
            "--check",
            "--vendor",
            args.vendor,
            "--work",
            work / "domain-regeneration",
        ],
    )
    invoke(
        "SDK metadata hook regeneration",
        [
            sys.executable,
            snapshot / "tools/native_bindings/generate_rpmalloc_metadata.py",
            "--check",
            "--vendor",
            args.vendor,
            "--work",
            work / "metadata-regeneration",
        ],
    )
    invoke(
        "SDK ABI/facade regeneration and six native header targets",
        [
            sys.executable,
            snapshot / "tools/native_bindings/vendor_rpmalloc.py",
            "--check",
            "--vendor",
            args.vendor,
            "--work",
            work / "sdk-regeneration",
            "--formatter",
            args.formatter,
        ],
    )
    native_formatter = args.native_compiler.with_name("clang-format")
    report["tools"][str(native_formatter.resolve())] = digest(
        native_formatter.resolve()
    )
    invoke(
        "owned native formatting",
        [
            native_formatter,
            "--dry-run",
            "--Werror",
            "--style={BasedOnStyle: LLVM, SortIncludes: Never}",
            *(
                snapshot / ("stdlib/internal/bindings/rpmalloc-heaps/" + name)
                for name in (
                    "bridge.c",
                    "runtime-domain-post.h",
                    "runtime-domain.inc",
                    "runtime-callback-state.h",
                    "runtime-live.h",
                    "runtime-metadata.h",
                )
            ),
            snapshot / "stdlib/internal/bindings/rpmalloc-callbacks/bridge.c",
            snapshot / "tests/native-bindings/rpmalloc_domains_probe.c",
            snapshot / "tests/native-bindings/rpmalloc_live_config_probe.c",
            snapshot / "tests/native-bindings/rpmalloc_metadata_probe.c",
            snapshot / "tests/native-bindings/rpmalloc_metadata_fault.c",
        ],
    )
    selected_profiles = [
        p
        for p in profiles.PROFILES
        if p["statistics"]
        and p["global_cache"]
        and p["thread_cache"]
        and p["assertions"]
        and p["validation"]
        and not p["adaptive"]
        and (not p["raw"] or not p["public_heaps"])
    ]
    selected = {p[k] for p in selected_profiles for k in ("sdk", "callbacks")}
    host_profile = next(p for p in selected_profiles if not p["raw"])
    for source in (
        "stdlib/internal/bindings/rpmalloc-heaps/bridge.c",
        "tests/native-bindings/rpmalloc_metadata_probe.c",
    ):
        invoke(
            "Mac x64 C source/ABI " + source,
            [
                args.native_compiler,
                "-arch",
                "x86_64",
                "-std=c11",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-fno-builtin-calloc",
                "-isystem",
                prepared,
                "-I",
                snapshot / "stdlib/internal/bindings/rpmalloc-heaps",
                *profiles.c_definitions(host_profile, api["functions"]),
                "-fsyntax-only",
                snapshot / source,
            ],
        )
    for instrumented in (False, True):
        folder = work / ("native-sanitized" if instrumented else "native")
        sanitize = ("address", "undefined") if instrumented else ()
        flags = ["-fsanitize=" + ",".join(sanitize)] if sanitize else []
        builder.build(
            folder,
            selected=selected,
            source=prepared,
            sanitize=sanitize,
            cc=args.native_compiler,
        )
        env.update(JAIC_NATIVE_LIBS=str(folder), LIBRARY_PATH=str(folder))
        invoke(
            "callback context dependency " + str(instrumented),
            [
                args.native_compiler,
                "-dynamiclib",
                "-Wall",
                "-Wextra",
                "-Werror",
                *flags,
                snapshot / "stdlib/internal/bindings/callback-context/bridge.c",
                "-o",
                folder / "libbinding_callback_context.dylib",
            ],
        )
        if not instrumented:
            invoke(
                "native-only real metadata fault observer",
                [
                    args.native_compiler,
                    "-dynamiclib",
                    "-O2",
                    "-fno-builtin-calloc",
                    "-pthread",
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                    snapshot / "tests/native-bindings/rpmalloc_metadata_fault.c",
                    "-o",
                    folder / "librpmalloc_metadata_fault.dylib",
                ],
            )
            for name in native_names:
                for mode in ("O0", "O2"):
                    exe = work / (name + "-" + mode)
                    if invoke(
                        "public OOM build " + exe.name,
                        [
                            args.compiler,
                            snapshot / f"tests/native-bindings/{name}.jai",
                            "-" + mode,
                            "-exe",
                            exe,
                        ],
                    ):
                        invoke("public OOM behavior " + exe.name, [exe])
        objects = []
        for profile in selected_profiles:
            raw = profile["raw"]
            obj = folder / ("live-raw.o" if raw else "live.o")
            objects.append(obj)
            aliases = (
                [
                    "-Dnative_error_mode=raw_native_error_mode",
                    "-Dnative_error=raw_native_error",
                    "-Dnative_name=raw_native_name",
                    "-Dnative_size=raw_native_size",
                    "-DLIVE_NAME_COPY=" + profile["facade_prefix"] + "name_copy",
                ]
                if raw
                else []
            )
            invoke(
                "SDK live probe object " + str(raw) + " " + str(instrumented),
                [
                    args.native_compiler,
                    "-std=c11",
                    "-fPIC",
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                    "-isystem",
                    prepared,
                    *profiles.c_definitions(profile, api["functions"]),
                    *aliases,
                    *flags,
                    "-c",
                    snapshot / "tests/native-bindings/rpmalloc_live_config_probe.c",
                    "-o",
                    obj,
                ],
            )
        invoke(
            "SDK live probe archive " + str(instrumented),
            ["ar", "rcs", folder / "librpmalloc_live_config_probe.a", *objects],
        )
        invoke(
            "SDK live probe dynamic " + str(instrumented),
            [
                args.native_compiler,
                "-dynamiclib",
                *objects,
                *flags,
                "-L",
                folder,
                "-lbinding_rpmalloc_sdk",
                "-lbinding_rpmalloc_sdk_raw_base",
                "-Wl,-rpath,@loader_path",
                "-o",
                folder / "librpmalloc_live_config_probe.dylib",
            ],
        )
        profile = next(p for p in selected_profiles if not p["raw"])
        probe = folder / "domains-probe"
        if invoke(
            "independent SDK domains build " + str(instrumented),
            [
                args.native_compiler,
                "-std=c11",
                "-O2",
                "-fno-builtin-calloc",
                "-pthread",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-isystem",
                prepared,
                "-I",
                snapshot / "stdlib/internal/bindings/rpmalloc-heaps",
                *profiles.c_definitions(profile, api["functions"]),
                *flags,
                snapshot / "tests/native-bindings/rpmalloc_domains_probe.c",
                "-o",
                probe,
            ],
        ):
            invoke(
                "independent SDK domains/thread lifetime/reserves " + str(instrumented),
                [probe],
            )
        metadata_probe = folder / "metadata-probe"
        if invoke(
            "transactional metadata build " + str(instrumented),
            [
                args.native_compiler,
                "-std=c11",
                "-O2",
                "-fno-builtin-calloc",
                "-pthread",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-isystem",
                prepared,
                "-I",
                snapshot / "stdlib/internal/bindings/rpmalloc-heaps",
                *profiles.c_definitions(profile, api["functions"]),
                *flags,
                snapshot / "tests/native-bindings/rpmalloc_metadata_probe.c",
                "-o",
                metadata_probe,
            ],
        ):
            invoke(
                "metadata OOM/retry/realloc preservation/thread handoff "
                + str(instrumented),
                [metadata_probe],
            )
        for name in names:
            fixture = snapshot / f"tests/native-bindings/{name}.jai"
            if not instrumented:
                invoke(
                    "public interpreter " + name, [args.compiler, fixture, "-interp"]
                )
            for mode in ("O0",) if instrumented else ("O0", "O2"):
                exe = work / (
                    name + "-" + mode + ("-sanitized" if instrumented else "")
                )
                if invoke(
                    "public build " + exe.name,
                    [
                        args.compiler,
                        fixture,
                        "-" + mode,
                        *(["-sanitize", "address,undefined"] if instrumented else []),
                        "-exe",
                        exe,
                    ],
                ):
                    invoke("public runtime " + exe.name, [exe])
                    if name == "rpmalloc-live-config":
                        env["RPMALLOC_TEST_CLEAR_ERROR"] = "1"
                        try:
                            invoke(
                                "cleared error callback default " + exe.name,
                                [exe],
                                -6,
                                "rpmalloc assertion failed without an error callback",
                            )
                        finally:
                            del env["RPMALLOC_TEST_CLEAR_ERROR"]
    for name in (*names, *native_names):
        fixture = snapshot / f"tests/native-bindings/{name}.jai"
        for os_name in ("macos", "linux", "windows"):
            for cpu in ("x64", "arm64"):
                invoke(
                    "source " + name + " " + os_name + "-" + cpu,
                    [
                        args.compiler,
                        fixture,
                        "-check",
                        "-no_dce",
                        "-os",
                        os_name,
                        "-cpu",
                        cpu,
                    ],
                )
        invoke(
            "strict lint " + name,
            [
                args.jailint,
                "-D",
                "warnings",
                "--config",
                snapshot / "jailint.toml",
                "--context",
                fixture,
                snapshot / "stdlib/rpmalloc",
                snapshot / "stdlib/internal/bindings/rpmalloc-domain",
                fixture,
            ],
        )
        invoke("fixture format " + name, [args.formatter, "--check", fixture])
    invoke(
        "production format",
        [
            args.formatter,
            "--check",
            snapshot / "stdlib/rpmalloc",
            snapshot / "stdlib/internal/bindings/rpmalloc-domain",
        ],
    )
    report["privacy_inputs"] = []
    for member in (
        "Domain",
        "Domain_Handle",
        "default_map",
        "default_unmap",
        "callback_name",
        "Scope",
        "scope_enter",
        "scope_leave",
        "binding_metadata_map",
    ):
        negative = work / ("private-" + member + ".jai")
        negative.write_text(
            'M :: #import "rpmalloc"; main :: () { value := M.' + member + "; }\n"
        )
        negative.chmod(0o444)
        report["privacy_inputs"].append(
            dict(path=str(negative), sha256=digest(negative))
        )
        invoke(
            "public privacy " + member, [args.compiler, negative, "-check", "-no_dce"]
        )
        row = report["checks"][-1]
        exact = "module `rpmalloc` has no exported member `" + member + "`"
        row["status"] = (
            "passed" if row["exit"] != 0 and exact in row["stderr"] else "failed"
        )
    report["checks"].append(
        dict(
            label="immutable source and SDK inputs",
            status="passed"
            if unchanged(snapshot, identity)
            and hashes == {p.name: digest(p) for p in prepared.iterdir() if p.is_file()}
            else "failed",
        )
    )
    (work / "verification.json").write_text(json.dumps(report, indent=2) + "\n")
    failures = sum(row["status"] != "passed" for row in report["checks"])
    print(f"{failures} failures; {work / 'verification.json'}")
    return int(bool(failures))


if __name__ == "__main__":
    raise SystemExit(main())
