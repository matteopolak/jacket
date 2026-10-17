"""Owned Mac helper runtime packaging from an explicitly configured native SDK.

Only copies SDK runtime images into this worker's artifact directories. Never
edits the SDK. System OS linkage remains unchanged. Shared distribution manifest
integration is owned by the coordinator.
"""

import os
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import re


def output(command):
    return subprocess.check_output([str(item) for item in command], text=True)


def dependencies(image):
    lines = output(["otool", "-L", image]).splitlines()[1:]
    identity = output(["otool", "-D", image]).splitlines()[1:]
    return [
        line.strip().split(" (", 1)[0]
        for line in lines
        if line.strip().split(" (", 1)[0] not in identity
    ]


def system_library(name):
    return name.startswith(("/usr/lib/", "/System/Library/"))


def runtime_names(image):
    """Basenames of the non-system libraries an image links, in link order.

    SDKs differ in which spelling the linker records (`libclang.dylib` on
    arm64 builds, the versioned `libclang.13.dylib` from conda), so staging
    follows what the image actually asks for rather than a fixed name.
    """
    names = []
    for dependency in dependencies(image):
        name = Path(dependency).name
        if not system_library(dependency) and name not in names:
            names.append(name)
    return names


def sign(image):
    subprocess.run(
        ["codesign", "--force", "--sign", "-", str(image)],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
    )


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def sdk_metadata_sidecar(path):
    if not path.name.startswith("._"):
        return False
    if (
        path.read_bytes()[:8] == bytes.fromhex("0005160700020000")
        and path.with_name(path.name[2:]).exists()
    ):
        return True
    raise RuntimeError("Unrecognized SDK metadata sidecar: " + str(path))


def conda_notices(root, source):
    """Find real notices in the cache of the package owning this installed image."""
    metadata_root = root / "conda-meta"
    if not metadata_root.is_dir():
        return None
    relative = source.relative_to(root).as_posix()
    owners = []
    for metadata_path in sorted(metadata_root.glob("*.json")):
        if sdk_metadata_sidecar(metadata_path):
            continue
        metadata = json.loads(metadata_path.read_text())
        files = set(metadata.get("files", []))
        files.update(
            p["_path"] for p in metadata.get("paths_data", {}).get("paths", [])
        )
        if relative in files:
            owners.append((metadata_path, metadata))
    if len(owners) != 1:
        raise RuntimeError(
            f"Conda runtime must have exactly one package owner: {source}"
        )
    metadata_path, metadata = owners[0]
    identity = {key: metadata[key] for key in ("name", "version", "build")}
    if not all(
        isinstance(value, str) and re.fullmatch(r"[A-Za-z0-9_.-]+", value)
        for value in identity.values()
    ):
        raise RuntimeError("Invalid conda package identity")
    cache = Path(metadata["link"]["source"])
    if not cache.is_absolute():
        raise RuntimeError("Conda package cache must be an absolute provenance path")
    cache = cache.resolve(strict=True)
    index_path = cache / "info/index.json"
    index = json.loads(index_path.read_text())
    if any(index.get(key) != value for key, value in identity.items()):
        raise RuntimeError("Conda cache identity differs from installed package")
    paths_path = cache / "info/paths.json"
    paths = json.loads(paths_path.read_text())
    if relative not in {p["_path"] for p in paths["paths"]}:
        raise RuntimeError("Conda cache does not own the installed runtime image")
    licenses = cache / "info/licenses"
    if not licenses.is_dir() or not licenses.resolve().is_relative_to(cache):
        raise RuntimeError(
            f"Conda runtime package lacks contained license files: {source}"
        )
    component = "conda-" + "-".join(identity.values())
    provenance = {
        "conda_package": identity,
        "installed_metadata_sha256": sha256(metadata_path),
        "package_index_sha256": sha256(index_path),
        "package_paths_sha256": sha256(paths_path),
        "package_archive_sha256": metadata.get("sha256"),
    }
    result = []
    for notice in sorted(licenses.rglob("*")):
        if not notice.is_file():
            continue
        if sdk_metadata_sidecar(notice):
            continue
        if not notice.resolve().is_relative_to(licenses.resolve()):
            raise RuntimeError("Conda license escapes its owning package")
        result.append(
            (
                notice,
                Path("notices") / component / notice.relative_to(licenses),
                provenance,
            )
        )
    if not result:
        raise RuntimeError(f"Conda runtime package contains no license files: {source}")
    return result


def stage_notices(sources, destination):
    """Retain installed SDK notices for every non-system runtime component.

    The configured SDK provides top-level notices or conda package provenance.
    Missing notices are a packaging failure, never a fabricated license choice.
    """
    filenames = (
        "LICENSE",
        "LICENSE.TXT",
        "LICENSE.txt",
        "COPYING",
        "COPYING.txt",
        "COPYRIGHT",
        "NOTICE",
        "NOTICE.txt",
    )
    planned = {}
    for name, source in sources.items():
        root = source.parent.parent
        package_notices = conda_notices(root, source)
        if package_notices is not None:
            planned[name] = package_notices
            continue
        notices = [
            root / filename for filename in filenames if (root / filename).is_file()
        ]
        if not notices:
            raise RuntimeError(
                f"SDK runtime lacks top-level license/notice files: {source}"
            )
        component = root.parent.name + "-" + root.name
        planned[name] = [
            (notice, Path("notices") / component / notice.name, {})
            for notice in notices
        ]
    result = {}
    for name, notices in planned.items():
        result[name] = []
        for source, relative, provenance in notices:
            target = destination / relative
            digest = sha256(source)
            if target.exists() and sha256(target) != digest:
                raise RuntimeError(f"Conflicting staged SDK notice: {target}")
            target.parent.mkdir(parents=True, exist_ok=True)
            if not target.exists():
                shutil.copy2(source, target)
            result[name].append(
                dict(
                    sdk_notice=str(source),
                    staged_path=str(relative),
                    sha256=digest,
                    **provenance,
                )
            )
    return result


def stage_macos_sdk(prefix, destination, names):
    """Copy and relocate the complete non-system dependency closure.

    Name collisions fail instead of selecting an arbitrary runtime ABI. The
    copied images use loader-relative dependencies, which take precedence over
    ambient rpaths. The existing compiler JAIC_NATIVE_LIBS resolver is untouched.
    """
    destination.mkdir(parents=True, exist_ok=True)
    pending = [(prefix / "lib" / name, name) for name in names]
    sources, links = {}, {}
    while pending:
        source, name = pending.pop()
        resolved = source.resolve(strict=True)
        if name in sources:
            if sources[name] != resolved:
                raise RuntimeError(f"Conflicting native SDK runtime basename: {name}")
            continue
        sources[name] = resolved
        links[name] = dependencies(resolved)
        for dependency in links[name]:
            if system_library(dependency):
                continue
            if dependency.startswith("@rpath/"):
                candidate = prefix / "lib" / dependency.removeprefix("@rpath/")
            elif dependency.startswith("@loader_path/"):
                candidate = resolved.parent / dependency.removeprefix("@loader_path/")
            elif dependency.startswith("/"):
                candidate = Path(dependency)
            else:
                raise RuntimeError(
                    f"Unsupported SDK runtime dependency spelling: {dependency}"
                )
            pending.append((candidate, Path(dependency).name))
    notices = stage_notices(sources, destination)
    for name, source in sources.items():
        target = destination / name
        if target.is_symlink():
            target.unlink()
        shutil.copy2(source, target)
        target.chmod(target.stat().st_mode | 0o200)
    manifest_path = destination / "runtime-sdk.json"
    manifest = json.loads(manifest_path.read_text()) if manifest_path.exists() else {}
    for name, image_links in links.items():
        target = destination / name
        command = ["install_name_tool", "-id", "@rpath/" + name]
        for dependency in image_links:
            if not system_library(dependency):
                command.extend(
                    ["-change", dependency, "@loader_path/" + Path(dependency).name]
                )
        subprocess.run([*command, str(target)], check=True)
        sign(target)
        manifest[name] = {
            "sdk_image": str(sources[name]),
            "sdk_sha256": sha256(sources[name]),
            "staged_sha256": sha256(target),
            "staged_dependencies": dependencies(target),
            "notices": notices[name],
        }
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
    return set(sources)


def relocate_macos_helper(image, runtime, *, library=False):
    command = ["install_name_tool"]
    if library:
        command.extend(["-id", "@rpath/" + image.name])
    for dependency in dependencies(image):
        if system_library(dependency):
            continue
        bundled = runtime / Path(dependency).name
        if not bundled.is_file():
            raise RuntimeError(f"Missing helper runtime dependency: {bundled.name}")
        relative = os.path.relpath(bundled, image.parent)
        command.extend(["-change", dependency, "@loader_path/" + relative])
    if len(command) > 1:
        subprocess.run([*command, str(image)], check=True)
        sign(image)
