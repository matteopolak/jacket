"""Linux packaging of an explicit installed OpenSSL SDK; no crypto reimplementation."""

import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import tempfile


SYSTEM_LIBRARIES = frozenset(
    {"libc.so.6", "libdl.so.2", "libpthread.so.0", "librt.so.1", "libm.so.6"}
)


def run(command, *, cwd=None, env=None):
    return subprocess.run(
        list(map(str, command)),
        cwd=cwd,
        env=env,
        check=True,
        capture_output=True,
        text=True,
        timeout=180,
    ).stdout


def sha256(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def elf_dynamic(text):
    sonames = re.findall(r"\(SONAME\).*?\[([^\]]+)\]", text)
    if (
        len(sonames) != 1
        or Path(sonames[0]).name != sonames[0]
        or sonames[0] in (".", "..")
    ):
        raise ValueError("OpenSSL SDK ELF must have one basename SONAME")
    needed = re.findall(r"\(NEEDED\).*?\[([^\]]+)\]", text)
    foreign = sorted(set(needed) - SYSTEM_LIBRARIES)
    if foreign:
        raise ValueError(f"undeclared third-party dependency closure: {foreign}")
    return sonames[0], needed


def runtime_paths(text, needed):
    result = {}
    for line in text.splitlines():
        if "=> not found" in line:
            raise ValueError(
                f"unresolved OpenSSL SDK runtime dependency: {line.strip()}"
            )
        match = re.match(r"\s*(\S+)\s+=>\s+(/.*?)\s+\(0x[0-9a-fA-F]+\)\s*$", line)
        if match:
            result[match[1]] = Path(match[2])
            continue
        loader = re.match(r"\s*(/\S+)\s+\(0x[0-9a-fA-F]+\)\s*$", line)
        if loader:
            path = Path(loader[1])
            result[path.name] = path
    missing = set(needed) - result.keys()
    if missing:
        raise ValueError(
            f"runtime dependency paths missing from native loader report: {sorted(missing)}"
        )
    for name, path in result.items():
        if not path.is_file():
            raise ValueError(f"native loader dependency is absent: {name}: {path}")
        if name not in SYSTEM_LIBRARIES and not name.startswith(
            ("ld-linux-", "ld-musl-")
        ):
            raise ValueError(f"undeclared transitive runtime dependency: {name}")
    return result


def public_header_paths(text):
    _, separator, payload = text.replace("\\\n", "").partition(":")
    if not separator:
        raise ValueError(
            "C compiler did not produce a dependency rule for public SDK headers"
        )
    paths = [Path(name) for name in shlex.split(payload)]
    return sorted(
        {path for path in paths if path.suffix == ".h" and "openssl" in path.parts}
    )


def license_sources(sdk, inputs):
    """Use actual SDK notices or package-manager-owned installed copyright files."""
    direct = [
        sdk.prefix / name
        for name in ("LICENSE.txt", "LICENSE", "AUTHORS.md")
        if (sdk.prefix / name).is_file()
    ]
    if any(path.name.startswith("LICENSE") for path in direct):
        return [("sdk", path) for path in direct]
    result = set()
    if shutil.which("dpkg-query"):
        packages = set()
        for path in inputs:
            owners = run(["dpkg-query", "--search", path]).splitlines()
            attributable = set()
            for owner in owners:
                package, separator, recorded = owner.partition(": ")
                if separator and Path(recorded).resolve() == path.resolve():
                    attributable.update(package.split(", "))
            if not attributable:
                raise ValueError(
                    f"OpenSSL SDK input has no installed package owner: {path}"
                )
            packages.update(attributable)
        for package in sorted(packages):
            notices = [
                Path(name)
                for name in run(["dpkg-query", "--listfiles", package]).splitlines()
                if name.endswith("/copyright") and "/share/doc/" in name
            ]
            if not any(notice.is_file() for notice in notices):
                raise ValueError(
                    f"OpenSSL SDK package has no installed copyright notice: {package}"
                )
            for notice in notices:
                if notice.is_file():
                    result.add((package, notice))
    elif shutil.which("rpm"):
        packages = {
            run(["rpm", "-qf", "--qf", "%{NAME}\n", path]).strip() for path in inputs
        }
        for package in sorted(packages):
            package_notices = set()
            for name in run(["rpm", "-ql", package]).splitlines():
                path = Path(name)
                if "/share/licenses/" in name and path.is_file():
                    package_notices.add((package, path))
            if not package_notices:
                raise ValueError(
                    f"OpenSSL SDK package has no installed license notice: {package}"
                )
            result.update(package_notices)
    if not result:
        raise ValueError(
            "OpenSSL SDK has no attributable installed license; provide its actual SDK license or package-manager development package"
        )
    return sorted(result, key=lambda item: (item[0], str(item[1])))


def version_header(sdk):
    path = sdk.include_dir / "openssl/opensslv.h"
    text = path.read_text()
    major = re.search(r"^\s*#\s*define\s+OPENSSL_VERSION_MAJOR\s+(\d+)\s*$", text, re.M)
    if not major or major[1] != "3":
        raise ValueError("OpenSSL SDK must declare OPENSSL_VERSION_MAJOR 3")
    version = re.search(
        r'^\s*#\s*define\s+OPENSSL_VERSION_TEXT\s+"([^"]+)"', text, re.M
    )
    if not version:
        raise ValueError("OpenSSL SDK has no public version text")
    return path, version[1]


def build_linux(sdk, stdlib, spec, root, manifest):
    """Package real archive+ELF runtime and execute both unchanged C consumers."""
    header, version = version_header(sdk)
    archive_source = sdk.library.parent / "libcrypto.a"
    shared_source = sdk.library.resolve()
    if not archive_source.is_file() or shared_source.suffix == ".a":
        raise ValueError(
            "OpenSSL SDK requires its actual static archive and shared ELF library"
        )
    identity, needed = elf_dynamic(run(["readelf", "--dynamic", shared_source]))
    dependencies = runtime_paths(run(["ldd", shared_source]), needed)
    abi = root / "tests/native/android-crypto/abi.c"
    version_source = root / "tests/native/android-crypto/package-version.c"
    headers = {header}
    for source in (abi, version_source):
        headers.update(
            public_header_paths(
                run(["cc", "-M", "-MT", "OPENSSL_HEADERS", *sdk.cflags, source])
            )
        )
    if not headers or any(not path.is_file() for path in headers):
        raise ValueError("OpenSSL SDK public header dependency closure is incomplete")
    notices = license_sources(sdk, [*sorted(headers), archive_source, shared_source])
    output = stdlib.absolute() / spec["provider_relative_directory"]
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(
        prefix="android-crypto-build-", dir=output.parent
    ) as temporary:
        stage = Path(temporary)
        work = stage / ".probe"
        work.mkdir()
        env = sdk.runtime_environment()
        env.update(TMPDIR=str(work), TEMP=str(work), TMP=str(work))
        archive = stage / "libcrypto.a"
        runtime = stage / identity
        shared = stage / "libcrypto.so"
        shutil.copyfile(archive_source, archive)
        shutil.copyfile(shared_source, runtime)
        if runtime != shared:
            shared.symlink_to(identity)
        staged_identity, staged_needed = elf_dynamic(
            run(["readelf", "--dynamic", shared])
        )
        if staged_identity != identity or staged_needed != needed:
            raise ValueError(
                "packaged ELF identity/dependency closure differs from SDK"
            )
        runtime_versions = {}
        for artifact in (archive, shared):
            executable = work / ("abi-" + artifact.suffix[1:])
            common = ["cc", "-std=c11", "-Wall", "-Wextra", "-Werror", *sdk.cflags]
            link = [
                artifact,
                "-ldl",
                "-pthread",
                "-Wl,-rpath,$ORIGIN/..",
                "-o",
                executable,
            ]
            run([*common, abi, *link], cwd=work, env=env)
            run([executable], cwd=work, env=env)
            run([*common, version_source, *link], cwd=work, env=env)
            actual = run([executable], cwd=work, env=env).strip()
            if actual != version:
                raise ValueError(
                    f"SDK header/archive version mismatch: {version} / {actual}"
                )
            runtime_versions[artifact.name] = actual
        origin = work / "runtime-origin.c"
        origin.write_text(
            "#define _GNU_SOURCE\n#include <dlfcn.h>\n#include <openssl/crypto.h>\n#include <stdio.h>\n"
            "int main(void) { Dl_info info; if (!dladdr((void *)OpenSSL_version, &info)) return 1; puts(info.dli_fname); return 0; }\n"
        )
        executable = work / "runtime-origin"
        run(
            [
                "cc",
                "-std=c11",
                "-Wall",
                "-Wextra",
                "-Werror",
                *sdk.cflags,
                origin,
                shared,
                "-ldl",
                "-Wl,-rpath,$ORIGIN/..",
                "-o",
                executable,
            ],
            cwd=work,
            env=env,
        )
        loaded = Path(run([executable], cwd=work, env=env).strip())
        if loaded.resolve() != runtime.resolve():
            raise ValueError(
                f"native probe loaded a library outside the package: {loaded}"
            )
        license_records = []
        for owner, source in notices:
            relative = Path("licenses/openssl") / (
                source.name
                if owner == "sdk"
                else owner.replace(":", "_") + "/" + source.name
            )
            destination = stage / relative
            if destination.exists() and sha256(destination) != sha256(source):
                raise ValueError(f"ambiguous installed license destination: {relative}")
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, destination)
            license_records.append(
                dict(
                    package_owner=owner,
                    source=str(source),
                    path=relative.as_posix(),
                    sha256=sha256(source),
                )
            )
        shutil.rmtree(work)
        files = sorted(
            {
                "libcrypto.a",
                "libcrypto.so",
                identity,
                *(record["path"] for record in license_records),
            }
        )
        machine = re.search(
            r"^\s*Machine:\s*(.+)$", run(["readelf", "--file-header", shared]), re.M
        )
        if not machine:
            raise ValueError("packaged runtime has no ELF machine identity")
        provenance = {
            "format": 1,
            "implementation": version,
            "openssl_major": 3,
            "sdk_prefix": str(sdk.prefix),
            "sdk_resolved_prefix": str(sdk.prefix.resolve()),
            "sdk_descriptor": dict(
                include_dir=str(sdk.include_dir),
                library=str(sdk.library),
                cflags=list(sdk.cflags),
                ldflags=list(sdk.ldflags),
                source=sdk.source,
            ),
            "sdk_archive_sha256": sha256(archive_source),
            "sdk_archive": str(archive_source),
            "sdk_shared": str(shared_source),
            "sdk_shared_sha256": sha256(shared_source),
            "sdk_public_headers": [
                {
                    "path": str(path),
                    "resolved_path": str(path.resolve()),
                    "sha256": sha256(path),
                }
                for path in sorted(headers)
            ],
            "shared_identity": identity,
            "runtime_dependencies": needed,
            "runtime_dependency_provenance": [
                {
                    "name": name,
                    "path": str(path),
                    "resolved_path": str(path.resolve()),
                    "sha256": sha256(path),
                    "distribution": "required host system runtime; not copied",
                }
                for name, path in sorted(dependencies.items())
            ],
            "third_party_runtime_dependencies": [],
            "architectures": machine[1].strip(),
            "build": "actual installed SDK archive and SONAME ELF copied; no compatibility shim",
            "license": "actual installed SDK/distribution notices",
            "license_inputs": license_records,
            "sha256": {name: sha256(stage / name) for name in files},
            "runtime_versions": runtime_versions,
            "runtime_origin_verified": "packaged SONAME via executable $ORIGIN/.. RUNPATH",
            "manifest_sha256": sha256(manifest),
            "native_platforms_verified_by_build": ["linux-" + os.uname().machine],
            "linux_native_verified": True,
            "windows_native_verified": False,
            "linux_windows_native_verified": False,
        }
        (stage / "provenance.json").write_text(json.dumps(provenance, indent=2) + "\n")
        if output.exists():
            shutil.rmtree(output)
        shutil.copytree(
            stage, output, symlinks=True, ignore=shutil.ignore_patterns("._*")
        )
    return output
