"""Build and test the exact libjpeg-turbo release used by the independent JPEG oracle.

    python3 build_reference.py --output target/jpeg-reference

The corpus revision is separate from this reference implementation pin. No system
library is substituted when the pinned source/version is unavailable.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import urllib.request

HERE = Path(__file__).resolve().parent


def extract_source(archive, destination):
    """Extract regular source files only; e.g. extract_source(archive, output / 'source')."""
    destination.mkdir()
    with tarfile.open(archive) as bundle:
        for member in bundle.getmembers():
            path = (destination / member.name).resolve()
            if not path.is_relative_to(destination.resolve()):
                raise ValueError("unsafe reference archive path")
            if member.isdir():
                path.mkdir(parents=True, exist_ok=True)
            elif member.isfile():
                path.parent.mkdir(parents=True, exist_ok=True)
                with bundle.extractfile(member) as source, path.open("wb") as output:
                    shutil.copyfileobj(source, output)
                path.chmod(member.mode & 0o777)
            else:
                raise ValueError("unsupported reference archive member type")
    roots = list(destination.iterdir())
    if len(roots) != 1 or not roots[0].is_dir():
        raise ValueError("unexpected reference archive root")
    return roots[0]


def main():
    """Verify pinned bytes, build the shared reference API, and run all its configured tests."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument(
        "--archive",
        type=Path,
        help="Optional local copy; its hash must match the manifest.",
    )
    args = parser.parse_args()
    pin = json.loads((HERE / "manifest.json").read_text())["reference_build"]
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    archive = output / "source.tar.gz"
    if args.archive:
        data = args.archive.read_bytes()
    else:
        request = urllib.request.Request(
            pin["url"], headers={"User-Agent": "JPEG-reference-build"}
        )
        with urllib.request.urlopen(request, timeout=60) as response:
            data = response.read()
    actual = hashlib.sha256(data).hexdigest()
    if actual != pin["sha256"]:
        raise ValueError("reference archive hash mismatch: " + actual)
    archive.write_bytes(data)
    source = extract_source(archive, output / "source")
    if (
        "set(VERSION " + pin["version"] + ")"
        not in (source / "CMakeLists.txt").read_text()
    ):
        raise ValueError("reference version differs from manifest")
    parallel = int(
        os.environ.get(
            "CMAKE_BUILD_PARALLEL_LEVEL", os.environ.get("CARGO_BUILD_JOBS", "2")
        )
    )
    if parallel < 1 or parallel > 2:
        parser.error("reference build parallelism must be between 1 and 2")
    build, install = output / "build", output / "install"
    commands = [
        [
            "cmake",
            "-S",
            source,
            "-B",
            build,
            "-DCMAKE_BUILD_TYPE=Release",
            f"-DCMAKE_INSTALL_PREFIX={install}",
            "-DCMAKE_INSTALL_LIBDIR=lib",
            "-DENABLE_SHARED=ON",
            "-DENABLE_STATIC=OFF",
            "-DWITH_TESTS=ON",
            "-DWITH_TURBOJPEG=OFF",
        ],
        ["cmake", "--build", build, "--parallel", str(parallel)],
        [
            "ctest",
            "--test-dir",
            build,
            "--output-on-failure",
            "--no-tests=error",
            "--parallel",
            str(parallel),
        ],
        ["cmake", "--install", build],
    ]
    receipt = {
        "pin": pin,
        "archive_sha256": actual,
        "passed": False,
        "commands_completed": 0,
    }
    try:
        with (output / "build.log").open("w") as log:
            for command in commands:
                subprocess.run(
                    list(map(str, command)),
                    check=True,
                    stdout=log,
                    stderr=subprocess.STDOUT,
                )
                receipt["commands_completed"] += 1
        shutil.copyfile(source / "LICENSE.md", output / "LICENSE.md")
        receipt["installed_sha256"] = {
            str(path.relative_to(install)): hashlib.sha256(
                path.read_bytes()
            ).hexdigest()
            for folder in ("include", "lib")
            for path in (install / folder).rglob("*")
            if path.is_file()
        }
        receipt["passed"] = True
    finally:
        (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(
        json.dumps(
            {"version": pin["version"], "revision": pin["revision"], "passed": True}
        )
    )


if __name__ == "__main__":
    main()
