"""Validate pinned Mesa WGL target/install identity and native SDK dependencies."""
import ctypes
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

from window_render_wgl_ci import pe_machine

WGL_TARGETS = {"opengl32.dll": ("opengl32", "libgl-gdi"),
               "libgallium_wgl.dll": ("libgallium_wgl", "wgl")}
WGL_EXPORTS = {
    "opengl32.dll": {"wglCreateContext", "wglMakeCurrent", "wglGetCurrentContext",
                     "wglGetCurrentDC", "wglGetProcAddress", "glReadPixels", "glGetString"},
    "libgallium_wgl.dll": {"DrvCreateContext", "DrvDeleteContext", "DrvSetContext",
                           "DrvGetProcAddress", "DrvDescribePixelFormat", "DrvSetPixelFormat"}}


def sha256(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def validate_wgl_configuration(options):
    values = {option["name"]: option["value"] for option in options}
    expected = {"platforms": ["windows"], "opengl": True, "gallium-drivers": ["llvmpipe"],
                "llvm": "enabled", "shared-llvm": "disabled", "b_vscrt": "mt",
                "gallium-wgl-dll-name": "libgallium_wgl"}
    for name, value in expected.items():
        if values.get(name) != value:
            raise ValueError(f"Mesa WGL configuration mismatch: {name}={values.get(name)!r}, expected {value!r}")


def wgl_targets(source, build, targets):
    found = {}
    for filename, (name, subdir) in WGL_TARGETS.items():
        defined = source / f"src/gallium/targets/{subdir}/meson.build"
        candidates = [target for target in targets if target.get("name") == name
                      and target.get("type") == "shared library" and target.get("installed")
                      and Path(target.get("defined_in", "")).resolve() == defined.resolve()]
        if len(candidates) != 1:
            raise ValueError(f"required pinned Mesa WGL target missing/ambiguous: {name}")
        outputs = [Path(path) for path in candidates[0]["filename"] if Path(path).name.casefold() == filename]
        if len(outputs) != 1 or not outputs[0].resolve().is_relative_to(build.resolve()):
            raise ValueError(f"unexpected Mesa WGL target output: {name}")
        found[filename] = outputs[0]
    return found


def discover_wgl_install(source, build, prefix, targets, installed):
    mapping = {os.path.normcase(str(Path(origin).resolve())): destination for origin, destination in installed.items()}
    result = {}
    for name, output in wgl_targets(source, build, targets).items():
        destination = mapping.get(os.path.normcase(str(output.resolve())))
        if not isinstance(destination, str):
            raise ValueError(f"Mesa WGL install mapping missing: {name}")
        path = Path(destination)
        if not path.resolve().is_relative_to(prefix.resolve()):
            raise ValueError(f"Mesa WGL install mapping outside fresh prefix: {path}")
        if path.name.casefold() != name or path.is_symlink() or not path.is_file():
            raise ValueError(f"required installed WGL image missing/renamed: {path}")
        if output.is_symlink() or not output.is_file() or sha256(output) != sha256(path):
            raise ValueError(f"Mesa WGL build/install integrity mismatch: {name}")
        result[name] = path
    return result


def parse_dumpbin(text):
    dependencies = {match.group(1).casefold() for match in re.finditer(
        r"^\s+([A-Za-z0-9_.+-]+\.dll)\s*$", text, re.MULTILINE | re.IGNORECASE)}
    exports = {match.group(1) for match in re.finditer(
        r"^\s+\d+\s+[0-9a-f]+\s+[0-9a-f]+\s+(\S+)", text, re.MULTILINE | re.IGNORECASE)}
    return dependencies, exports


def system_dependency(name):
    # Resolve API-set contracts with the actual OS loader, not guessed physical
    # API-set filenames or an ARM64X architecture whitelist. Never use PATH.
    sdk = ctypes.WinDLL("kernel32", use_last_error=True)
    sdk.LoadLibraryExW.argtypes = [ctypes.c_wchar_p, ctypes.c_void_p, ctypes.c_uint32]
    sdk.LoadLibraryExW.restype = ctypes.c_void_p
    sdk.GetModuleFileNameW.argtypes = [ctypes.c_void_p, ctypes.c_wchar_p, ctypes.c_uint32]
    sdk.GetModuleFileNameW.restype = ctypes.c_uint32
    sdk.FreeLibrary.argtypes = [ctypes.c_void_p]
    sdk.FreeLibrary.restype = ctypes.c_int
    handle = sdk.LoadLibraryExW(name, None, 0x800)  # LOAD_LIBRARY_SEARCH_SYSTEM32
    if not handle:
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        buffer = ctypes.create_unicode_buffer(32768)
        count = sdk.GetModuleFileNameW(handle, buffer, len(buffer))
        if not count or count >= len(buffer):
            raise ctypes.WinError(ctypes.get_last_error())
        return {"requested": name, "resolved": buffer.value, "machine": pe_machine(Path(buffer.value))}
    finally:
        if not sdk.FreeLibrary(handle):
            raise ctypes.WinError(ctypes.get_last_error())


def validate_native_install(prefix, pair, dumpbin, evidence):
    images = {}
    state = {"stage": "PE images", "images": images, "system_dependencies": {},
             "validated": False, "runtime_verified": False}

    def snapshot():
        (evidence / "validation.json").write_text(json.dumps(state, indent=2) + "\n", encoding="utf-8")

    for path in sorted(prefix.rglob("*")):
        if path.suffix.casefold() != ".dll":
            continue
        name = path.name.casefold()
        if path.is_symlink() or name in images:
            raise ValueError(f"ambiguous/symbolic-link installed DLL: {path}")
        machine = pe_machine(path)
        images[name] = {"path": str(path), "machine": machine, "sha256": sha256(path)}
        state["current_image"] = name
        snapshot()
        if machine != 0xaa64:
            raise ValueError(f"non-ARM64 installed image: {path}, machine={machine:#x}")
        result = subprocess.run([str(dumpbin), "/nologo", "/dependents", "/exports", str(path)],
                                capture_output=True, text=True)
        # Native C/SDK symbol metadata only, never Jai source/log inspection.
        (evidence / f"{name}.sdk.txt").write_text(result.stdout + result.stderr, encoding="utf-8")
        result.check_returncode()
        dependencies, exports = parse_dumpbin(result.stdout)
        if not dependencies:
            raise ValueError(f"native SDK did not report DLL dependencies: {path}")
        images[name] = {"path": str(path), "machine": machine, "sha256": sha256(path),
                        "dependencies": sorted(dependencies), "exports": sorted(exports)}
        snapshot()
    state["stage"] = "WGL exports/import identity"
    snapshot()
    for name in pair:
        if name not in images or not WGL_EXPORTS[name].issubset(images[name]["exports"]):
            raise ValueError(f"installed image lacks required actual WGL exports: {name}")
    if "libgallium_wgl.dll" not in images["opengl32.dll"]["dependencies"]:
        raise ValueError("Mesa opengl32 does not import the pinned WGL ICD")
    pending, selected, system = list(pair), {}, {}
    state["stage"] = "dependency closure"
    state["system_dependencies"] = system
    while pending:
        name = pending.pop()
        if name in selected:
            continue
        selected[name] = images[name]
        for dependency in images[name]["dependencies"]:
            if dependency in images:
                pending.append(dependency)
            elif dependency not in system:
                state["current_dependency"] = dependency
                snapshot()
                system[dependency] = system_dependency(dependency)
                snapshot()
    state["stage"] = "validated; fresh native GL execution still required"
    state["validated"] = True
    snapshot()
    return {"images": selected, "system_dependencies": system, "runtime_verified": False}
