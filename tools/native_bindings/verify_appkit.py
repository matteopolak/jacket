"""Capture and verify the reviewed AppKit facade without launching NSApplication."""

import argparse
import ctypes
import hashlib
import json
import os
import re
from pathlib import Path
import subprocess
import sys
import tempfile

from shared_snapshot import ROOT, capture, unchanged


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("compiler", "jailint", "formatter", "output_root", "native_libs"):
        parser.add_argument("--" + name.replace("_", "-"), type=Path, required=True)
    args = parser.parse_args()
    reductions = [
        "tests/native-bindings/objc-appkit-public-classes.jai",
        "tests/native-bindings/objc-appkit-opengl-class.jai",
        "tests/native-bindings/appkit-window-inheritance.jai",
        "tests/native-bindings/appkit.jai",
    ]
    extra = [
        "tests/native-bindings/appkit-public.jai",
        "tests/native-bindings/appkit_probe.m",
        "tests/native-bindings/appkit-gl-identity.jai",
        "docs/specs/stdlib/modules/appkit.md",
        "tools/native_bindings/generate_appkit.py",
        "tools/native_bindings/verify_appkit.py",
        "tools/native_bindings/shared_snapshot.py",
        "jailint.toml",
        "jaifmt.toml",
        "docs/specs/stdlib/modules/objective-c.md",
        "tests/native-bindings/appkit-ui.jai",
        "tests/native-bindings/appkit_ui_probe.m",
    ] + reductions
    snapshot, identity = capture(args.output_root, extra)
    work = Path(tempfile.mkdtemp(prefix="appkit-", dir=args.output_root))
    env = dict(
        os.environ,
        JAIC_STDLIB=str(snapshot / "stdlib"),
        TMPDIR=str(work),
        JAIC_NATIVE_LIBS=str(args.native_libs),
        LIBRARY_PATH=str(work) + ":" + str(args.native_libs),
        DYLD_LIBRARY_PATH=str(work) + ":" + str(args.native_libs),
    )
    checks = []

    def condition(label, value):
        checks.append(dict(label=label, passed=bool(value)))

    def run(label, argv, expected=None):
        result = subprocess.run(
            [str(a) for a in argv],
            cwd=snapshot,
            env=env,
            capture_output=True,
            text=True,
            timeout=180,
        )
        passed = result.returncode == 0
        if expected:
            passed = result.returncode != 0 and expected in result.stderr
        checks.append(
            dict(
                label=label,
                command=[str(a) for a in argv],
                exit=result.returncode,
                passed=passed,
                stdout=result.stdout,
                stderr=result.stderr,
            )
        )
        return passed

    fixture = snapshot / extra[0]
    run(
        "UI SDK controls build only",
        [
            "cc",
            "-dynamiclib",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-framework",
            "AppKit",
            snapshot / "tests/native-bindings/appkit_ui_probe.m",
            "-o",
            work / "libappkit_ui_probe.dylib",
        ],
    )
    run(
        "SDK probe build",
        [
            "cc",
            "-dynamiclib",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-framework",
            "AppKit",
            snapshot / extra[1],
            "-o",
            work / "libappkit_probe.dylib",
        ],
    )
    probe = ctypes.CDLL(str(work / "libappkit_probe.dylib"))
    has_method = probe.appkit_probe_has_method
    has_method.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_int]
    has_method.restype = ctypes.c_int
    selectors = []
    owner = member = None
    for line in (
        (snapshot / "stdlib/Objective_C/AppKit/generated.jai")
        .read_text()
        .split("#scope_file", 1)[0]
        .splitlines()
    ):
        nominal = re.match(r"(\w+) :: struct", line)
        if nominal:
            owner = nominal[1]
        elif line.startswith("}"):
            owner = member = None
        elif owner:
            method = re.match(r"    (\w+) ::", line)
            if method:
                member = method[1]
            selector = re.search(r'binding_selector\("([^"]+)"\)', line)
            if selector:
                receiver = re.search(r'binding_class\("([^"]+)"\)', line)
                cls = receiver[1] if receiver else owner
                available = bool(
                    has_method(cls.encode(), selector[1].encode(), bool(receiver))
                )
                selectors.append(
                    dict(
                        api=owner + "." + member,
                        native_class=cls,
                        selector=selector[1],
                        class_method=bool(receiver),
                        available=available,
                    )
                )
    absent = {row["api"] for row in selectors if not row["available"]}
    known_boundaries = {
        "NSView.convertPoint",
        "NSEvent.keyRepeatDelay",
        "NSEvent.keyRepeatInterval",
        "NSEvent.pressedMouseButtons",
    }
    condition(
        "native selector availability; exact reviewed receiver boundaries",
        absent == known_boundaries,
    )
    sdk = Path(subprocess.check_output(["xcrun", "--show-sdk-path"], text=True).strip())
    headers = ["NSApplication.h", "NSView.h", "NSEvent.h", "NSOpenGL.h", "NSScreen.h"]
    sdk_headers = {
        name: hashlib.sha256(
            (
                sdk / "System/Library/Frameworks/AppKit.framework/Headers" / name
            ).read_bytes()
        ).hexdigest()
        for name in headers
    }
    for cpu in ("arm64", "x64"):
        ui = snapshot / "tests/native-bindings/appkit-ui.jai"
        run(
            "UI source only " + cpu,
            [args.compiler, ui, "-check", "-no_dce", "-os", "macos", "-cpu", cpu],
        )
        run(
            "UI strict lint " + cpu,
            [
                args.jailint,
                "-D",
                "warnings",
                "-os",
                "macos",
                "-cpu",
                cpu,
                "--manifest",
                work / (cpu + "-ui-lint.json"),
                ui,
            ],
        )
        manifest = json.loads((work / (cpu + "-ui-lint.json")).read_text())
        condition(
            "UI complete source coverage " + cpu,
            not manifest["incomplete"] and not manifest["unvisited_files"],
        )
        for name in reductions:
            run(
                "original missing-export reduction " + cpu + " " + name,
                [
                    args.compiler,
                    snapshot / name,
                    "-check",
                    "-no_dce",
                    "-os",
                    "macos",
                    "-cpu",
                    cpu,
                ],
            )
            manifest_path = work / (cpu + "-" + Path(name).stem + "-lint.json")
            run(
                "reduction strict lint " + cpu + " " + name,
                [
                    args.jailint,
                    "-D",
                    "warnings",
                    "-os",
                    "macos",
                    "-cpu",
                    cpu,
                    "--manifest",
                    manifest_path,
                    snapshot / name,
                ],
            )
            manifest = json.loads(manifest_path.read_text())
            condition(
                "reduction complete coverage " + cpu + " " + name,
                not manifest["incomplete"] and not manifest["unvisited_files"],
            )
        run(
            "SDK ABI compile " + cpu,
            [
                "cc",
                "-arch",
                "arm64" if cpu == "arm64" else "x86_64",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-c",
                snapshot / extra[1],
                "-o",
                work / (cpu + "-sdk.o"),
            ],
        )
        run(
            "GL canonical identity " + cpu,
            [
                args.compiler,
                snapshot / extra[2],
                "-check",
                "-no_dce",
                "-os",
                "macos",
                "-cpu",
                cpu,
            ],
        )
        run(
            "Mac source " + cpu,
            [args.compiler, fixture, "-check", "-no_dce", "-os", "macos", "-cpu", cpu],
        )
        run(
            "strict lint " + cpu,
            [
                args.jailint,
                "-D",
                "warnings",
                "-os",
                "macos",
                "-cpu",
                cpu,
                "--context",
                fixture,
                "--manifest",
                work / (cpu + "-lint.json"),
                snapshot / "stdlib/Objective_C/AppKit",
                fixture,
            ],
        )
        run(
            "NSGL strict lint " + cpu,
            [
                args.jailint,
                "-D",
                "warnings",
                "-os",
                "macos",
                "-cpu",
                cpu,
                "--context",
                snapshot / extra[2],
                "--manifest",
                work / (cpu + "-nsgl-lint.json"),
                snapshot / "stdlib/internal/bindings/nsgl",
                snapshot / extra[2],
            ],
        )
        for suffix in ("-lint.json", "-nsgl-lint.json"):
            manifest_path = work / (cpu + suffix)
            manifest = json.loads(manifest_path.read_text())
            condition(
                "complete lint coverage " + cpu + suffix,
                not manifest["incomplete"]
                and not manifest["unvisited_files"]
                and all(c["complete"] for c in manifest["contexts"]),
            )
    run("interpreter", [args.compiler, fixture, "-interp"])
    for mode in ("O0", "O2"):
        exe = work / mode
        if run("build " + mode, [args.compiler, fixture, "-" + mode, "-exe", exe]):
            run("execute " + mode, [exe])
    existing = snapshot / "tests/native-bindings/appkit.jai"
    run("retained ABI interpreter", [args.compiler, existing, "-interp"])
    for mode in ("O0", "O2"):
        exe = work / ("retained-abi-" + mode)
        if run(
            "retained ABI build " + mode,
            [args.compiler, existing, "-" + mode, "-exe", exe],
        ):
            run("retained ABI execute " + mode, [exe])
    exe = work / "sanitized"
    if run(
        "sanitized build",
        [args.compiler, fixture, "-O1", "-sanitize", "address,undefined", "-exe", exe],
    ):
        run("sanitized execution", [exe])
    for member in (
        "NSApplication",
        "NSOpenGLContext",
        "NSView",
        "NSEvent",
        "NSResponder",
        "NSWindow",
        "NSOpenGLPixelFormat",
        "NSMenu",
        "NSMenuItem",
        "NSScreen",
        "NSApplicationDelegate",
    ):
        private = work / ("privacy-" + member + ".jai")
        private.write_text(
            'O :: #import "Objective_C"; leak :: O.' + member + "; main :: () {}\n"
        )
        run(
            "root privacy " + member,
            [args.compiler, private, "-check"],
            "has no exported member `" + member + "`",
        )
    for module, member in [
        ("Objective_C/AppKit", name)
        for name in (
            "binding_send",
            "binding_class",
            "Selectors",
            "initialize_selectors",
        )
    ] + [("GL", "NSOpenGLContext")]:
        private = work / ("private-helper-" + member + ".jai")
        private.write_text(
            'A :: #import "' + module + '"; leak :: A.' + member + "; main :: () {}\n"
        )
        run(
            "helper privacy " + member,
            [args.compiler, private, "-check"],
            "has no exported member `" + member + "`",
        )
    for os_ in ("windows", "linux"):
        for cpu in ("arm64", "x64"):
            private = work / "availability.jai"
            private.write_text('A :: #import "Objective_C/AppKit"; main :: () {}\n')
            run(
                "unsupported availability " + os_ + "-" + cpu,
                [args.compiler, private, "-check", "-os", os_, "-cpu", cpu],
                "Objective_C/AppKit requires macOS",
            )
    run(
        "format",
        [
            args.formatter,
            "--check",
            snapshot / "stdlib/Objective_C/AppKit",
            snapshot / "stdlib/internal/bindings/nsgl",
            fixture,
            snapshot / extra[2],
            snapshot / "tests/native-bindings/appkit-ui.jai",
            *[snapshot / name for name in reductions],
        ],
    )
    generated = work / "generated.jai"
    run(
        "reviewed regeneration",
        [
            sys.executable,
            snapshot / "tools/native_bindings/generate_appkit.py",
            "--output",
            generated,
        ],
    )
    run(
        "format regeneration",
        [args.formatter, "--config", snapshot / "jaifmt.toml", generated],
    )
    condition(
        "deterministic generated facade",
        generated.read_bytes()
        == (snapshot / "stdlib/Objective_C/AppKit/generated.jai").read_bytes(),
    )
    condition("immutable captured inputs", unchanged(snapshot, identity))
    owned = extra + [
        "stdlib/Objective_C/AppKit/module.jai",
        "stdlib/Objective_C/AppKit/generated.jai",
        "stdlib/internal/bindings/nsgl/module.jai",
    ]
    condition(
        "owned source stable",
        all(
            hashlib.sha256((ROOT / name).read_bytes()).hexdigest()
            == identity["files"][name]
            for name in owned
        ),
    )
    report = dict(
        selectors=selectors,
        official_sdk=dict(path=str(sdk), headers=sdk_headers),
        snapshot=str(snapshot),
        work=str(work),
        input_identity=identity,
        tools={
            str(p): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in (args.compiler, args.jailint, args.formatter)
        },
        checks=checks,
        limits=[
            "No UI lease/run; no application or GL context creation",
            "x64 source only; other platforms unavailable for AppKit",
        ],
    )
    (work / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(str(work / "report.json"))
    raise SystemExit(any(not c["passed"] for c in checks))


if __name__ == "__main__":
    main()
