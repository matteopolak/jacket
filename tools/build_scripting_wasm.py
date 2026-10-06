#!/usr/bin/env python3
"""Build the real Rust compiler as wasm and stage the browser bundle (module, glue, formatter driver, tour).

With --jaic (a native jaic with LLVM and wasm-ld), jaifmt compiled to a WASI module (jaifmt.wasm) is staged too.
"""
from pathlib import Path
import argparse
import hashlib
import json
import os
import shutil
import subprocess
from cargo_build_paths import checked_directory, configured_target_directory, pinned_cargo_command

ROOT = Path(__file__).resolve().parents[1]
BUNDLED_GLUE = ("engine.mjs", "README.md")


def storage_directory(path):
    while not path.exists():
        path = path.parent
    if not path.is_dir():
        raise ValueError('build storage path is not a directory')
    return path


def bundled_examples(root):
    """(bundle directory, source directory, main file) of each example the bundle ships (tests/examples.json)."""
    cases = json.loads((root / "tests/examples.json").read_text())["cases"]
    return [(case["bundle"], root / case["directory"], case["main"]) for case in cases if "bundle" in case]


def stage_example(source, output, name, main):
    """Copy an example workspace to <output>/<name>/ and describe it in <output>/<name>.json.

    Embedders (the hosted playground) fetch the index first, then each listed file. Dotfiles are left out.
    """
    destination = output / name
    if destination.is_symlink():
        raise SystemExit(f"bundle example directory is a symlink: {destination}")
    shutil.rmtree(destination, ignore_errors=True)
    files = []
    for path in sorted(source.rglob("*")):
        relative = path.relative_to(source)
        if any(part.startswith(".") for part in relative.parts):
            continue
        if path.is_symlink():
            raise SystemExit(f"example files cannot be symlinks: {path}")
        if path.is_file():
            (destination / relative).parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(path, destination / relative)
            files.append(relative.as_posix())
    if main not in files:
        raise SystemExit(f"example {source} has no {main}")
    index = {"schema_version": 1, "main": main, "files": files}
    (output / f"{name}.json").write_text(json.dumps(index, indent=2) + "\n")


def jaifmt_wasm_command(jaic, root, output):
    """Compile the WASI jaifmt (jaifmt/wasm.jai) with a native jaic; docs/tools/jaifmt.md."""
    return [str(jaic), "build", str(root / "jaifmt/wasm.jai"), "-os", "wasm", "-O2", "--no-debug-info",
            "-o", str(output)]


def build_command(cargo, target, release):
    command = [*cargo, "build", "--offline", "--locked", "-j", "1", "-p", "jai-wasm",
               "--target", "wasm32-unknown-unknown", "--target-dir", str(target)]
    if release:
        command.append("--release")
    return command


def main():
    root = ROOT
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--release", action="store_true")
    parser.add_argument("--output", type=Path, default=root / "artifacts/scripting-runtime")
    parser.add_argument("--target-dir", type=Path,
                        help="Override CARGO_TARGET_DIR or Cargo target configuration")
    parser.add_argument("--jaic", type=Path, help="native jaic that compiles jaifmt.wasm into the bundle")
    args = parser.parse_args()
    environment = {**os.environ, "CARGO_INCREMENTAL": "0"}
    cargo = pinned_cargo_command(root, environment)
    target = configured_target_directory(args.target_dir, environment, cargo, root)
    environment["CARGO_TARGET_DIR"] = str(target.path)
    # Output paths are user-selected too; they cannot replace inert original inputs.
    output = checked_directory(args.output.resolve(), root)
    for path in (root, target.path, output):
        if shutil.disk_usage(storage_directory(path)).free < 2 * 1024**3:
            raise SystemExit("build requires at least 2 GiB of free disk space on each used volume")
    command = build_command(cargo, target.path, args.release)
    subprocess.run(command, cwd=root, env=environment, check=True)
    wasm = target.path / "wasm32-unknown-unknown" / ("release" if args.release else "debug") / "jai_wasm.wasm"
    with wasm.open("rb") as compiled:
        if compiled.read(8) != b"\x00asm\x01\x00\x00\x00":
            raise SystemExit("compiler output is not a WebAssembly module")
    output.mkdir(parents=True, exist_ok=True)
    # The bundle is the module plus the glue an embedder needs; the hosted UI lives elsewhere
    # (docs/browser/playground.md).
    for name in BUNDLED_GLUE:
        shutil.copy2(root / "crates/jai-wasm/js" / name, output / name)
    # Format buttons run this driver in the engine (docs/tools/jaifmt.md).
    shutil.copy2(root / "jaifmt/playground.jai", output / "jaifmt-playground.jai")
    # Or jaifmt itself, compiled to WebAssembly: much faster than interpreting the driver.
    formatter = output / "jaifmt.wasm"
    formatter.unlink(missing_ok=True)
    formatter_sha256 = None
    if args.jaic:
        subprocess.run(jaifmt_wasm_command(args.jaic.resolve(), root, formatter), cwd=root, env=environment, check=True)
        if formatter.read_bytes()[:8] != b"\x00asm\x01\x00\x00\x00":
            raise SystemExit("jaifmt.wasm is not a WebAssembly module")
        formatter_sha256 = hashlib.sha256(formatter.read_bytes()).hexdigest()
    # Example workspaces the playground opens with (examples/tour as tour/).
    for name, source, main in bundled_examples(root):
        stage_example(source, output, name, main)
    staged = output / "jai_wasm.wasm"
    shutil.copy2(wasm, staged)
    expected = hashlib.sha256(wasm.read_bytes()).hexdigest()
    if hashlib.sha256(staged.read_bytes()).hexdigest() != expected:
        raise RuntimeError("WebAssembly output changed while staging it")
    receipt = {"build_command": command, "target_directory": target.receipt(),
               "wasm_build_path": str(wasm), "wasm_staged_path": str(staged),
               "wasm_sha256": expected}
    if formatter_sha256:
        receipt["jaifmt_wasm_sha256"] = formatter_sha256
    (output / "build-metadata.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(f"Browser bundle: {output}")


if __name__ == "__main__":
    main()
