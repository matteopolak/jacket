"""Build-only, hash-checked fixes to official pinned C sources. Never edit vendor."""

import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
PATCHES = (
    (
        "sdk-page-granularity.patch",
        "4ee8af6bc941251310da5ccea8810c158eeb8152b6960923a9d5a9e5a05394ff",
        "Round heap-pool mappings to whole configured pages",
    ),
    (
        "sdk-runtime-dimensions.patch",
        "f2b12d6b3ea053e0422fa7f6369aebdd8add5f1bfe0955296c77867f9e0af259",
        "Conditional private runtime storage; original SDK layout otherwise unchanged",
    ),
    (
        "sdk-runtime-policies.patch",
        "cd14c428c4a5dcf46cc76a24e87662a85cddf105414b670336a1ec60a181158d",
        "Private unlimited/unmap policies and reviewed Linux ordinary mapping/huge-page discovery",
    ),
    (
        "sdk-live-config.patch",
        "c617393d4773d848995548b09620ddb6aa5fcb6e6596e747df61676e481d364d",
        "Read live name provider at each native naming use; thread-local copy lifetime",
    ),
    (
        "sdk-domains.patch",
        "28ec6483d0d4c132e0429fbd76694f0d2ed65dfb96742cb2dcbe8d4121f2291d",
        "Complete private mutable SDK state in typed module-owned domains and per-domain thread slots",
    ),
    (
        "sdk-metadata.patch",
        "3e83a44c76f33c079df075dc357d50607a8a1068056195c23f0e35e1d6ff0a74",
        "Reserve typed heap/span publication capacity before native ownership mutation; retire at actual unmap",
    ),
)


def patch_provenance():
    result = []
    for name, expected, reason in PATCHES:
        path = ROOT / "stdlib/internal/bindings/rpmalloc-heaps" / name
        data = path.read_bytes()
        if hashlib.sha256(data).hexdigest() != expected:
            raise ValueError("Reviewed rpmalloc patch mismatch: " + str(path))
        result.append(
            dict(path=str(path.relative_to(ROOT)), sha256=expected, purpose=reason)
        )
    return result


def prepare(source, destination, stop_before=None):
    pin = json.loads((Path(__file__).with_name("dependencies.json")).read_text())[
        "rpmalloc"
    ]
    destination.mkdir(parents=True, exist_ok=True)
    hashes = {}
    for name, expected in pin["files"].items():
        original = source / name
        data = original.read_bytes()
        actual = hashlib.sha256(data).hexdigest()
        if actual != expected:
            raise ValueError("Pinned rpmalloc source mismatch: " + str(original))
        hashes[name] = actual
        (destination / Path(name).name).write_bytes(data)
    patches = patch_provenance()
    if stop_before and stop_before not in {Path(p["path"]).name for p in patches}:
        raise ValueError("Unknown SDK preparation boundary: " + stop_before)
    applied = []
    for patch in patches:
        if stop_before and Path(patch["path"]).name == stop_before:
            break
        data = (ROOT / patch["path"]).read_bytes()
        if hashlib.sha256(data).hexdigest() != patch["sha256"]:
            raise ValueError("rpmalloc patch changed during preparation")
        subprocess.run(
            ["patch", "--batch", "--forward", "--fuzz=0", "-p1"],
            input=data,
            cwd=destination,
            check=True,
            capture_output=True,
        )
        applied.append(patch)
    manifest = dict(
        repository=pin["repository"],
        revision=pin["revision"],
        original_sha256=hashes,
        patches=applied,
        patched_source_sha256=hashlib.sha256(
            (destination / "rpmalloc.c").read_bytes()
        ).hexdigest(),
        reason="Reviewed private SDK fixes; see patches and native configuration design",
    )
    (destination / "provenance.json").write_text(json.dumps(manifest, indent=2) + "\n")
    return destination


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=ROOT / ".work/vendor/rpmalloc")
    parser.add_argument("--output", type=Path, default=ROOT / ".work/patched-rpmalloc")
    args = parser.parse_args()
    print(prepare(args.source, args.output))
