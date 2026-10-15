"""Run only original portable-data tests against own source-built tools.

Native stb dependencies must be built separately using the repository's reviewed
native build tool. Nothing is fetched here; no shared manifests/staging are used.
"""
from pathlib import Path
import argparse
import os
import subprocess
import sys

root = Path(__file__).resolve().parents[2]
output = root / "target/portable-data"
output.mkdir(parents=True, exist_ok=True)
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--native-libs", type=Path, default=output / "native")
parser.add_argument("--compiler", type=Path)
parser.add_argument("--formatter", type=Path)
parser.add_argument("--linter", type=Path)
parser.add_argument("--probe-unsupported-wasi-images", action="store_true")
args = parser.parse_args()
env = dict(os.environ, JAIC_NATIVE_LIBS=str(args.native_libs.resolve()), JAIC_STDLIB=str(root / "stdlib"))
if os.name == "nt":
    env["PATH"] = str(args.native_libs.resolve()) + os.pathsep + env.get("PATH", "")
compiler = (args.compiler or root / ("target/debug/jaic.exe" if os.name == "nt" else "target/debug/jaic")).resolve(strict=True)
formatter = (args.formatter or root / ("target/jaifmt.exe" if os.name == "nt" else "target/jaifmt")).resolve(strict=True)
linter = (args.linter or root / ("target/debug/jailint.exe" if os.name == "nt" else "target/debug/jailint")).resolve(strict=True)
sources = [
    "stdlib/Ico_File", "stdlib/Simple_Package", "stdlib/Subtitles",
    "stdlib/internal/resource-wire", "stdlib/internal/simple-package",
    "stdlib/internal/subtitles", "tests/portable-data",
    "stdlib/Windows_Resources", "stdlib/tests/portable-data-resource.jai",
    "stdlib/Icon", "stdlib/internal/icon-plugin",
    "stdlib/tests/portable-data-manifest.jai",
]

failures = []

def run(*args):
    print("+", *map(str, args), flush=True)
    subprocess.run(list(map(str, args)), cwd=root, env=env, check=True)

def wasm_test(source, name):
    wasm = output / f"{name}.wasm"
    try:
        run(compiler, "-os", "wasm", "-O2", source, "-exe", wasm)
        run("node", "--no-warnings", "tools/wasi_run.mjs", wasm, output)
    except subprocess.CalledProcessError:
        failures.append(f"WASI {name}")

run(sys.executable, "tests/portable-data/api_inventory.py")
run(formatter, "--check", *sources)
run(linter, "-D", "warnings", *sources)
run(sys.executable, "tests/portable-data/wire_consumer.py", output, "fixtures")
for name in ("ico-wire", "ico-filename", "package-contract", "subtitles-contract", "icon-plugin", "icon-phase"):
    source = f"tests/portable-data/{name}.jai"
    run(compiler, "-interp", source, "--", output)
    for mode in ("O0", "O2"):
        binary = output / f"{name}-{mode}"
        run(compiler, f"-{mode}", source, "-exe", binary)
        run(binary, output)
for name in ("package-contract", "subtitles-contract", "icon-plugin", "icon-phase"):
    wasm_test(f"tests/portable-data/{name}.jai", name)
resource_test = "stdlib/tests/portable-data-resource.jai"
run(compiler, "-interp", resource_test, "--", output)
for mode in ("O0", "O2"):
    binary = output / f"resource-directory-{mode}"
    run(compiler, f"-{mode}", resource_test, "-exe", binary)
    run(binary, output)
if args.probe_unsupported_wasi_images:
    # Explicit unsupported-target probe: failures stay failures and exit nonzero.
    wasm_test("tests/portable-data/ico-wire.jai", "ico-wire")
    wasm_test("tests/portable-data/ico-filename.jai", "ico-filename")
    wasm_test(resource_test, "resource-directory")
for mode in ("O0", "O2"):
    binary = output / f"manifest-{mode}"
    run(compiler, f"-{mode}", "stdlib/tests/portable-data-manifest.jai", "-exe", binary)
    with (output / f"manifests-{mode}.xml").open("w") as stream:
        subprocess.run([str(binary)], cwd=root, env=env, stdout=stream, check=True)
wasm_test("stdlib/tests/portable-data-manifest.jai", "manifest")
run(sys.executable, "tests/portable-data/sdk_resources.py", output)
run(sys.executable, "tests/portable-data/wire_consumer.py", output)
windows_sources = ["stdlib/Windows_Resources", "stdlib/internal/windows-resources",
                   "stdlib/tests/portable-data-resource-windows.jai",
                   "stdlib/tests/portable-data-manifest-windows.jai",
                   "stdlib/tests/portable-data-console-windows.jai",
                   "stdlib/tests/portable-data-discovery-windows.jai"]
run(formatter, "--check", *windows_sources)
run(linter, "-D", "warnings", "-os", "windows", "--manifest",
    output / "lint-windows-native.json", *windows_sources)
for cpu in ("x64", "arm64"):
    for source in windows_sources[2:]:
        run(compiler, "-check", "-no_dce", "-os", "windows", "-cpu", cpu, source)
contexts = [(os_name, cpu) for os_name in ("macos", "linux", "windows")
            for cpu in ("x64", "arm64")] + [("wasm", None)]
for os_name, cpu in contexts:
    target_args = ["-os", os_name] + (["-cpu", cpu] if cpu else [])
    tag = os_name + (f"-{cpu}" if cpu else "")
    run(linter, "-D", "warnings", *target_args,
        "--manifest", output / f"lint-{tag}.json", *sources)
    for name in ("ico-wire", "package-contract", "subtitles-contract", "icon-plugin", "icon-phase"):
        run(compiler, "-check", "-no_dce", *target_args,
            f"tests/portable-data/{name}.jai")
if failures:
    print("FAILED target vectors:", ", ".join(failures), file=sys.stderr)
    raise SystemExit(1)
if os.name == "nt":
    for mode in ("O0", "O2"):
        for name in ("resource", "discovery", "console"):
            source = f"stdlib/tests/portable-data-{name}-windows.jai"
            binary = output / f"{name}-windows-{mode}.exe"
            run(compiler, f"-{mode}", source, "-exe", binary)
            run(binary, output)
        run(sys.executable, "tests/portable-data/sdk_discovery.py", output)
        plugin_output = output / f"with-icon-{mode}.exe"
        run(compiler, f"-{mode}", "tests/portable-data/package-contract.jai",
            "-exe", plugin_output, "+Icon", "-icon", output / "resource.ico")
        run(sys.executable, "tests/portable-data/sdk_icon_output.py",
            plugin_output, output / "resource.ico")
    print("Scoped tests including native Windows resource/discovery/CLI Icon passed.")
else:
    print("Scoped tests passed; native Windows resource effects remain unverified.")
