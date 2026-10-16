"""Capture current shared verification inputs; never overlay a producer clone."""

import hashlib
import json
from pathlib import Path
import shutil
import tempfile

ROOT = Path(__file__).resolve().parents[2]


def hashes(paths):
    return {
        str(path): hashlib.sha256(path.read_bytes()).hexdigest()
        for path in sorted(paths)
    }


def capture(output_root, extra_paths=()):
    output_root = Path(output_root).resolve()
    extra_paths = tuple(extra_paths)
    input_directories = [ROOT / "stdlib"]
    input_directories.extend(
        ROOT / name for name in extra_paths if (ROOT / name).is_dir()
    )
    if output_root == ROOT.resolve() or any(
        directory.resolve() == output_root or directory.resolve() in output_root.parents
        for directory in input_directories
    ):
        raise ValueError(
            "Snapshot/output root must not overlap a selected input directory"
        )
    output_root.mkdir(parents=True, exist_ok=True)
    for attempt in range(3):
        destination = Path(tempfile.mkdtemp(prefix="bindings-inputs-", dir=output_root))

        def inputs():
            found = {p for p in (ROOT / "stdlib").rglob("*") if p.is_file()}
            for name in extra_paths:
                path = ROOT / name
                if path.is_dir():
                    found.update(
                        p
                        for p in path.rglob("*")
                        if p.is_file() and "__pycache__" not in p.parts
                    )
                else:
                    found.add(path)
            return found

        original = hashes(inputs())
        for name in original:
            source = Path(name)
            target = destination / source.relative_to(ROOT)
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target)
        current = hashes(inputs())
        copied = hashes(destination / Path(name).relative_to(ROOT) for name in original)
        matched = original == current and all(
            copied[str(destination / Path(name).relative_to(ROOT))] == digest
            for name, digest in original.items()
        )
        receipt = dict(
            source_root=str(ROOT),
            attempt=attempt,
            matched=matched,
            files={
                str(Path(name).relative_to(ROOT)): digest
                for name, digest in original.items()
            },
        )
        (destination / "input-identity.json").write_text(
            json.dumps(receipt, indent=2) + "\n"
        )
        if matched:
            # Verification must not modify even a copied native/source fixture.
            for name in copied:
                relative = Path(name).relative_to(destination)
                executable = (ROOT / relative).stat().st_mode & 0o111
                Path(name).chmod(0o444 | executable)
            return destination, receipt
    raise RuntimeError(
        "Shared inputs changed during three capture attempts; retained unsuccessful receipts"
    )


def unchanged(snapshot, receipt):
    return all(
        (snapshot / name).is_file()
        and hashlib.sha256((snapshot / name).read_bytes()).hexdigest() == digest
        for name, digest in receipt["files"].items()
    )
