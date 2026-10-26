#!/usr/bin/env python3
"""Build the real Rust compiler as wasm and stage the browser bundle (module, glue, formatter driver, examples).

With --jaic (a native jaic with LLVM and wasm-ld), jaifmt compiled to a WASI module (jaifmt.wasm) is staged too.
"""

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

from pathlib import Path
import argparse
import json
import os
import shutil
import subprocess
from repo_output import output_dir
from cargo_build_paths import checked_directory, configured_target_directory, pinned_cargo_command
from toolkit.hashing import sha256_file
from toolkit.repo import ROOT

BUNDLED_GLUE = ("engine.mjs", "webgpu_host.mjs", "webgpu_bindings.generated.mjs", "README.md")


MAX_RELEASE_WASM_BYTES = 24 * 1024 * 1024


def storage_directory(path):
    while not path.exists():
        path = path.parent
    if not path.is_dir():
        raise ValueError('build storage path is not a directory')
    return path


def bundled_examples(root):
    """The cases of tests/corpus/examples.json the bundle ships, in listing order (the first is the default)."""
    cases = json.loads((root / "tests/corpus/examples.json").read_text())["cases"]
    return [case for case in cases if case.get("bundle")]


# Binary files an example loads at run time with Extensions/Assets. They live in its `assets/`
# folder, are listed under "assets" in the index (not opened as source) and are capped in size.
ASSET_SUFFIXES = {".png"}
MAX_ASSET_BYTES = 256 * 1024

# Version of the bundle's examples.json.
EXAMPLES_SCHEMA_VERSION = 2

# The settings every example shares. jaifmt and jailint find them natively in the folder above a
# workspace; a bundled workspace carries its own copy so it is self-contained.
SHARED_CONFIG = "examples/jacket.toml"


def stage_example(source, output, name, main, config=None):
    """Copy an example workspace to <output>/<name>/ and return its file lists for examples.json.

    Embedders (the hosted playground) fetch examples.json first, then each listed file; `assets`
    are fetched by the program itself (load_asset, with the engine's asset base at <name>/).
    Dotfiles are left out. `config`, the shared jacket.toml, is copied in as <name>/jacket.toml.
    """
    destination = output / name
    if destination.is_symlink():
        raise SystemExit(f"bundle example directory is a symlink: {destination}")
    shutil.rmtree(destination, ignore_errors=True)
    files = []
    assets = []
    for path in sorted(source.rglob("*")):
        relative = path.relative_to(source)
        if any(part.startswith(".") for part in relative.parts):
            continue
        if path.is_symlink():
            raise SystemExit(f"example files cannot be symlinks: {path}")
        if path.is_file():
            if config and relative.as_posix() == "jacket.toml":
                raise SystemExit(f"example {source} has its own jacket.toml; every example shares {SHARED_CONFIG}")
            if relative.parts[0] == "assets":
                if path.suffix not in ASSET_SUFFIXES or path.stat().st_size > MAX_ASSET_BYTES:
                    raise SystemExit(f"example asset must be {sorted(ASSET_SUFFIXES)} of at most {MAX_ASSET_BYTES} bytes: {path}")
                assets.append(relative.as_posix())
            else:
                files.append(relative.as_posix())
            (destination / relative).parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(path, destination / relative)
    if main not in files:
        raise SystemExit(f"example {source} has no {main}")
    if config:
        shutil.copy2(config, destination / "jacket.toml")
        files.append("jacket.toml")
    files.sort(key=lambda item: (item != main, item))
    return files, assets


def stage_examples(root, output):
    """Stage every bundled example and write <output>/examples.json, the one index of them."""
    config = root / SHARED_CONFIG
    if not config.is_file():
        raise SystemExit(f"missing {SHARED_CONFIG}")
    entries = []
    for case in bundled_examples(root):
        name = case["id"]
        if any(entry["name"] == name for entry in entries):
            raise SystemExit(f"duplicate bundled example: {name}")
        files, assets = stage_example(root / case["directory"], output, name, case["main"], config)
        entries.append({"name": name, "title": case["title"], "summary": case["summary"],
                        "main": case["main"], "files": files, "assets": assets})
    index = {"schema_version": EXAMPLES_SCHEMA_VERSION, "examples": entries}
    (output / "examples.json").write_text(json.dumps(index, indent=2) + "\n")


def jaifmt_wasm_command(jaic, root, output):
    """Compile the WASI jaifmt (jaifmt/wasm.jai) with a native jaic; docs/tools/jaifmt.md."""
    return [str(jaic), str(root / "jaifmt/wasm.jai"), "-os", "wasm", "-O2", "-no_debug_info",
            "-exe", str(output)]


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
    parser.add_argument("--output", type=Path, default=output_dir("scripting-runtime"))
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
    # Static hosts cap file sizes (Cloudflare Pages at 25 MiB); keep the release module under that.
    if args.release and wasm.stat().st_size > MAX_RELEASE_WASM_BYTES:
        raise SystemExit(f"jai_wasm.wasm is {wasm.stat().st_size} bytes, over the {MAX_RELEASE_WASM_BYTES}-byte limit; "
                         "check what crates/jai-wasm/build.rs bundles")
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
        formatter_sha256 = sha256_file(formatter)
    # The example workspaces the playground lists, and their index.
    stage_examples(root, output)
    staged = output / "jai_wasm.wasm"
    shutil.copy2(wasm, staged)
    expected = sha256_file(wasm)
    if sha256_file(staged) != expected:
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
