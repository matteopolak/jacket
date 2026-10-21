#!/usr/bin/env python3
"""Pieces of the post-release smoke test (.github/workflows/release-smoke.yml, docs/tools/release-smoke.md).

Usage:
  release_smoke.py fetch   --tag vX.Y.Z --dest DIR     download this platform's archive, verify it, unpack it; prints the folder
  release_smoke.py project --bin DIR                           `jacket new`, `build`, `run` and a language-server hover with the tools in DIR
  release_smoke.py vsix    --tag vX.Y.Z --dest DIR     download the VSIX, verify it against SHA256SUMS and check its contents
  release_smoke.py vsix-store --tag vX.Y.Z --dest DIR  the same extension version from Open VSX
  release_smoke.py version --tag vX.Y.Z --bin DIR      every tool in DIR reports the release's version
  release_smoke.py bindings --bin DIR                  `generate_bindings` on a C header and on a C++ class with the tools in DIR

`fetch` and `vsix` use `gh release download` (GH_TOKEN in the environment).
"""

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

import argparse
import hashlib
import json
import platform
import re
import subprocess
import sys
import tarfile
import tempfile
import time
import zipfile
from pathlib import Path

from toolkit.download import fetch as http_fetch
from toolkit.lsp import Lsp

TOOLS = ("jaic", "jailsp", "jailint", "jaifmt", "jacket")
OPEN_VSX = "https://open-vsx.org/api/matteopolak/jacket"


def platform_name() -> str:
    system = {"Darwin": "macos", "Linux": "linux", "Windows": "windows"}[
        platform.system()
    ]
    machine = platform.machine().lower()
    arch = "arm64" if machine in ("arm64", "aarch64") else "x64"
    return f"{system}-{arch}"


def asset_name(name: str) -> str:
    return f"jacket-{name}." + ("zip" if name.startswith("windows") else "tar.gz")


def exe(tool: str) -> str:
    return tool + (".exe" if platform.system() == "Windows" else "")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def parse_sums(text: str) -> dict[str, str]:
    """{asset: lower-case hash} from `sha256sum` output."""
    sums = {}
    for line in text.splitlines():
        m = re.fullmatch(r"([0-9a-fA-F]{64}) [ *](.+)", line.strip())
        if m:
            sums[m.group(2)] = m.group(1).lower()
    return sums


def download(tag: str, patterns: list[str], dest: Path) -> None:
    dest.mkdir(parents=True, exist_ok=True)
    command = [
        "gh",
        "release",
        "download",
        tag,
        "--repo",
        "matteopolak/jacket",
        "--clobber",
        "-D",
        str(dest),
    ]
    for pattern in patterns:
        command += ["-p", pattern]
    subprocess.run(command, check=True)


def verified(tag: str, pattern: str, dest: Path) -> Path:
    """The release asset matching `pattern`, downloaded into `dest` and checked against SHA256SUMS."""
    download(tag, [pattern, "SHA256SUMS"], dest)
    sums = parse_sums((dest / "SHA256SUMS").read_text())
    found = [
        p
        for p in dest.iterdir()
        if p.name != "SHA256SUMS"
        and re.fullmatch(pattern.replace(".", r"\.").replace("*", ".*"), p.name)
    ]
    if len(found) != 1:
        raise SystemExit(
            f"expected one asset matching {pattern}, found {[p.name for p in found]}"
        )
    path = found[0]
    expected = sums.get(path.name)
    if expected is None:
        raise SystemExit(f"SHA256SUMS has no line for {path.name}")
    actual = sha256(path)
    if actual != expected:
        raise SystemExit(
            f"{path.name}: sha256 {actual} does not match SHA256SUMS ({expected})"
        )
    print(f"{path.name}: sha256 {actual} matches SHA256SUMS", file=sys.stderr)
    return path


def fetch(tag: str, dest: Path) -> Path:
    archive = verified(tag, asset_name(platform_name()), dest)
    unpacked = dest / "unpacked"
    unpacked.mkdir()
    if archive.suffix == ".zip":
        with zipfile.ZipFile(archive) as z:
            z.extractall(unpacked)
    else:
        with tarfile.open(archive) as t:
            t.extractall(
                unpacked, filter="tar"
            )  # keeps the executable bits and the stdlib's symlinks; our own release
    (root,) = [p for p in unpacked.iterdir() if p.is_dir()]
    return root


def run(command, **kwargs) -> subprocess.CompletedProcess:
    print("+", " ".join(str(c) for c in command), flush=True)
    return subprocess.run(
        [str(c) for c in command], text=True, capture_output=True, **kwargs
    )


def need(result: subprocess.CompletedProcess, what: str) -> str:
    if result.returncode:
        raise SystemExit(
            f"{what} failed ({result.returncode}):\n{result.stdout}\n{result.stderr}"
        )
    return result.stdout


def check_versions(tag: str, bin_dir: Path) -> None:
    """`jacket version` lists jacket and every tool it fronts; each line carries the release's version."""
    version = tag.removeprefix("v")
    if not (bin_dir / exe("jacket")).exists():
        # Releases before 0.20 do not ship jacket; check the compiler alone.
        print("::warning::this release has no jacket; checking jaic only")
        text = need(run([bin_dir / exe("jaic"), "-version"]), "jaic -version")
        print(text.strip())
        if version not in text:
            raise SystemExit(f"jaic does not report {version}")
        return
    text = need(run([bin_dir / exe("jacket"), "version"]), "jacket version")
    print(text.strip())
    lines = [line for line in text.splitlines() if line.strip()]
    stale = [line for line in lines if version not in line]
    if len(lines) < 4 or stale:
        raise SystemExit(
            f"expected every tool to report {version}; lines without it: {stale}"
        )


def project(bin_dir: Path) -> None:
    """New project, build, run, and a hover from the language server in the same install."""
    jacket = bin_dir / exe("jacket")
    with tempfile.TemporaryDirectory(prefix="smoke-") as work:
        work = Path(work)
        if jacket.exists():
            need(run([jacket, "new", "smoke"], cwd=work), "jacket new")
            app = work / "smoke"
            need(run([jacket, "build"], cwd=app), "jacket build")
            # The template's build program names the executable after the project.
            built = need(
                run([app / exe("smoke")], cwd=app), "the program `jacket build` made"
            )
            if "Hello from smoke!" not in built:
                raise SystemExit(f"the built program printed {built!r}")
            out = need(run([jacket, "run"], cwd=app), "jacket run")
        else:
            app = work / "smoke"
            app.mkdir()
            (app / "main.jai").write_text(
                '#import "Basic";\n\nmain :: () {\n    print("Hello from smoke!\\n");\n}\n'
            )
            need(run([bin_dir / exe("jaic"), "main.jai"], cwd=app), "jaic main.jai")
            out = need(run([app / exe("main")], cwd=app), "the built program")
        if "Hello from smoke!" not in out:
            raise SystemExit(f"the program printed {out!r}")
        print("new, build, run: ok")
        hover(bin_dir / exe("jailsp"), app / "main.jai", None)


C_HEADER = "int add(int a, int b);\n"
C_SOURCE = '#include "add.h"\n#if defined(_WIN32)\n__declspec(dllexport)\n#endif\nint add(int a, int b) { return a + b; }\n'
# Issue 26's program: a C++ class, no language option.
CPP_HEADER = "#if defined(_WIN32)\n#define DEMO_EXPORT __declspec(dllexport)\n#else\n#define DEMO_EXPORT\n#endif\nclass Demo {\npublic:\n    DEMO_EXPORT int add(int a, int b);\n};\n"
CPP_SOURCE = '#include "add.h"\nint Demo::add(int a, int b) { return a + b; }\n'
BUILD = """main :: () {
    options: B.Generate_Bindings_Options;
    array_add(*options.source_files, "add.h");
    array_add(*options.libraries, .{ filename = "libadd" });
    array_add(*options.library_search_paths, #filepath);
    EXTRA
    ok := B.generate_bindings(options, "add.jai", log_results = true);
    if !ok exit(1);
}

#import "Basic";
#import "System";
B :: #import "Bindings_Generator";
"""


def bindings(bin_dir: Path) -> None:
    """`generate_bindings` needs the helper library, libclang and a compiler, all found from the install.

    A C header must produce a binding for `add` (the helper library and libclang loaded and
    parsed it). Issue 26's C++ class, with no language option, must be parsed as C++ and bind
    `Demo::add` by its mangled name. An explicit `-x c` keeps a header C: the same class then
    fails with libclang's own error.
    """
    jaic = bin_dir / exe("jaic")
    shared = {"Darwin": "dylib", "Windows": "dll"}.get(platform.system(), "so")
    flag = "-dynamiclib" if platform.system() == "Darwin" else "-shared"

    def generate(work: Path, extra: str = "") -> subprocess.CompletedProcess:
        (work / "build.jai").write_text(BUILD.replace("EXTRA", extra))
        (work / "add.jai").unlink(missing_ok=True)
        return run([jaic, "-interp", "build.jai"], cwd=work)

    with tempfile.TemporaryDirectory(prefix="bindings-") as work:
        work = Path(work)
        (work / "add.c").write_text(C_SOURCE)
        (work / "add.h").write_text(C_HEADER)
        need(run(["clang", flag, "-o", f"libadd.{shared}", "add.c"], cwd=work), "building the C library")
        need(generate(work), "generate_bindings on a C header")
        generated = (work / "add.jai").read_text()
        if "add :: " not in generated or "#cpp_method" in generated:
            raise SystemExit(f"add.jai is not the C binding of `add`:\n{generated}")
        print("generate_bindings, C header: ok")

        (work / "add.cpp").write_text(CPP_SOURCE)
        (work / "add.h").write_text(CPP_HEADER)
        need(run(["clang++", flag, "-o", f"libadd.{shared}", "add.cpp"], cwd=work), "building the C++ library")
        need(generate(work), "generate_bindings on the issue's C++ class")
        generated = (work / "add.jai").read_text()
        if "Demo :: struct" not in generated:
            raise SystemExit(f"add.jai has no Demo struct:\n{generated}")
        symbol = "?add@Demo@@QEAAHHH@Z" if platform.system() == "Windows" else "_ZN4Demo3addEii"
        if symbol not in generated:
            raise SystemExit(f"add.jai does not bind Demo::add:\n{generated}")
        (work / "consumer.jai").write_text(
            '#import "Basic";\n#load "add.jai";\n'
            'main :: () { object: Demo; assert(Demo.add(*object, 18, 24) == 42); }\n')
        need(run([jaic, "consumer.jai", "-interp"], cwd=work), "calling the bound C++ method in the interpreter")
        consumer = work / exe("consumer")
        need(run([jaic, "consumer.jai", "-O2", "-exe", consumer], cwd=work), "building the C++ method consumer")
        need(run([consumer], cwd=work), "calling the bound C++ method natively")
        print("generate_bindings, C++ class: native mangled symbol and interpreter/native calls: ok")

        result = generate(work, 'array_add(*options.extra_clang_arguments, "-x", "c");')
        text = result.stdout + result.stderr
        if result.returncode == 0 or "unknown type name 'class'" not in text:
            raise SystemExit(f"an explicit -x c must keep the header C ({result.returncode}):\n{text}")
        print("generate_bindings, explicit -x c: stays C")


def hover(jailsp: Path, source: Path, env) -> None:
    text = source.read_text()
    line = next(i for i, row in enumerate(text.splitlines()) if row.startswith("main"))
    client = Lsp(jailsp, source.parent, env)
    try:
        client.request(
            "initialize",
            {
                "processId": None,
                "rootUri": source.parent.resolve().as_uri(),
                "capabilities": {},
            },
            60,
        )
        client.notify("initialized", {})
        uri = source.resolve().as_uri()
        client.notify(
            "textDocument/didOpen",
            {
                "textDocument": {
                    "uri": uri,
                    "languageId": "jai",
                    "version": 1,
                    "text": text,
                }
            },
        )
        # The server answers once the document is analysed; ask a few times.
        answer = None
        for _ in range(10):
            answer = client.request(
                "textDocument/hover",
                {
                    "textDocument": {"uri": uri},
                    "position": {"line": line, "character": 1},
                },
                60,
            )
            if answer:
                break
            time.sleep(1)
        if not answer or not answer.get("contents"):
            raise SystemExit(f"jailsp gave no hover for `main` in {source}")
        print(f"jailsp initialize + hover: ok ({json.dumps(answer['contents'])[:80]})")
    finally:
        client.close()


def vsix(tag: str, dest: Path) -> Path:
    """The release's VSIX: the checksum matches, the version is the tag's or the editor's, the bundle is there."""
    path = verified(tag, "jacket-vscode-*.vsix", dest)
    editor_version = re.fullmatch(r"jacket-vscode-(.+)\.vsix", path.name).group(1)
    with zipfile.ZipFile(path) as z:
        package = json.loads(z.read("extension/package.json"))
        bundle = z.read("extension/dist/extension.cjs").decode(errors="replace")
        names = z.namelist()
    if package["version"] != editor_version:
        raise SystemExit(f"{path.name}: package.json says {package['version']}")
    for needed in (
        "extension/syntaxes/jai.tmLanguage.json",
        "extension/language-configuration.json",
    ):
        if needed not in names:
            raise SystemExit(f"{path.name} has no {needed}")
    sums = parse_sums((dest / "SHA256SUMS").read_text())
    pinned = [
        name
        for name, digest in sums.items()
        if name.startswith("jacket-")
        and not name.endswith(".vsix")
        and digest in bundle
    ]
    archives = [
        name
        for name in sums
        if re.fullmatch(
            r"jacket-(macos|linux|windows)-(x64|arm64)\.(tar\.gz|zip)", name
        )
    ]
    if pinned and sorted(pinned) != sorted(archives):
        raise SystemExit(
            f"{path.name} pins {sorted(pinned)} but the release has {sorted(archives)}"
        )
    print(
        f"{path.name}: version {editor_version}, bundle ok, {len(pinned)}/{len(archives)} archive checksums pinned"
    )
    return path


def vsix_store(tag: str, dest: Path) -> None:
    """The VSIX at Open VSX for the same editor version carries the same package.json version."""
    path = vsix(tag, dest)
    editor_version = re.fullmatch(r"jacket-vscode-(.+)\.vsix", path.name).group(1)
    meta = json.loads(http_fetch(f"{OPEN_VSX}/{editor_version}", timeout=60))
    if meta.get("version") != editor_version:
        raise SystemExit(
            f"Open VSX reports version {meta.get('version')}, expected {editor_version}"
        )
    download_url = meta["files"]["download"]
    fetched = dest / "open-vsx.vsix"
    fetched.write_bytes(http_fetch(download_url))
    with zipfile.ZipFile(fetched) as z:
        version = json.loads(z.read("extension/package.json"))["version"]
    if version != editor_version:
        raise SystemExit(
            f"the Open VSX package.json says {version}, expected {editor_version}"
        )
    same = sha256(fetched) == sha256(path)
    print(
        f"Open VSX {editor_version}: package.json ok; identical to the release asset: {same}"
    )


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    sub = parser.add_subparsers(dest="command", required=True)
    for name in ("fetch", "vsix", "vsix-store"):
        p = sub.add_parser(name)
        p.add_argument("--tag", required=True)
        p.add_argument("--dest", type=Path, required=True)
    for name in ("project", "bindings"):
        p = sub.add_parser(name)
        p.add_argument("--bin", type=Path, required=True)
    p = sub.add_parser("version")
    p.add_argument("--tag", required=True)
    p.add_argument("--bin", type=Path, required=True)
    args = parser.parse_args()
    if args.command == "fetch":
        print(fetch(args.tag, args.dest))
    elif args.command == "project":
        project(args.bin)
    elif args.command == "bindings":
        bindings(args.bin)
    elif args.command == "vsix":
        vsix(args.tag, args.dest)
    elif args.command == "vsix-store":
        vsix_store(args.tag, args.dest)
    else:
        check_versions(args.tag, args.bin)
    return 0


if __name__ == "__main__":
    sys.exit(main())
