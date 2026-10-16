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


def output(command):
    return subprocess.check_output([str(item) for item in command], text=True)


def dependencies(image):
    lines = output(["otool", "-L", image]).splitlines()[1:]
    identity = output(["otool", "-D", image]).splitlines()[1:]
    return [line.strip().split(" (", 1)[0] for line in lines
            if line.strip().split(" (", 1)[0] not in identity]


def system_library(name):
    return name.startswith(("/usr/lib/", "/System/Library/"))


def sign(image):
    subprocess.run(["codesign", "--force", "--sign", "-", str(image)], check=True,
                   stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)


def sha256(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def stage_notices(sources, destination):
    """Retain installed SDK notices for every non-system runtime component.

    The configured native SDK must provide its top-level license/notice files.
    Missing notices are a packaging failure, never a fabricated license choice.
    """
    filenames = ('LICENSE', 'LICENSE.TXT', 'LICENSE.txt', 'COPYING',
                 'COPYING.txt', 'COPYRIGHT', 'NOTICE', 'NOTICE.txt')
    planned = {}
    for name, source in sources.items():
        root = source.parent.parent
        notices = [root / filename for filename in filenames if (root / filename).is_file()]
        if not notices:
            raise RuntimeError(f"SDK runtime lacks top-level license/notice files: {source}")
        component = root.parent.name + '-' + root.name
        planned[name] = [(notice, Path('notices') / component / notice.name) for notice in notices]
    result = {}
    for name, notices in planned.items():
        result[name] = []
        for source, relative in notices:
            target = destination / relative
            digest = sha256(source)
            if target.exists() and sha256(target) != digest:
                raise RuntimeError(f"Conflicting staged SDK notice: {target}")
            target.parent.mkdir(parents=True, exist_ok=True)
            if not target.exists():
                shutil.copy2(source, target)
            result[name].append(dict(sdk_notice=str(source), staged_path=str(relative), sha256=digest))
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
                raise RuntimeError(f"Unsupported SDK runtime dependency spelling: {dependency}")
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
                command.extend(["-change", dependency, "@loader_path/" + Path(dependency).name])
        subprocess.run([*command, str(target)], check=True)
        sign(target)
        manifest[name] = {"sdk_image": str(sources[name]),
                          "sdk_sha256": sha256(sources[name]),
                          "staged_sha256": sha256(target),
                          "staged_dependencies": dependencies(target),
                          "notices": notices[name]}
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
