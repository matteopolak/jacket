#!/usr/bin/env python3
"""Fetch pinned Khronos glTF fixtures. Run with PYTHONPATH=tools; pass --samples for the 1.1 GB sample models."""
import shutil
import sys
import tarfile
from repo_output import output_dir
from toolkit.download import pinned

SAMPLES = "8f492390ddc7a2ae221a1369ef560589cf8c70fb"
VALIDATOR = "602c54043ba66856b2e68c40fdf2bced975cdb8a"


def fetch(samples=False):
    """Stage the validator's core test data and, with `samples`, the two core sample variants.
    The samples are a 1.1 GB download, so only the CI leg that tests them fetches them."""
    root = output_dir("gltf-conformance")
    suites = [("glTF-Validator", VALIDATOR, "dfdb6e200ee411a173e1d0a6738d5bd319c1e93dbf7db222b61fe33ae9c82366", "validator")]
    if samples:
        suites.append(("glTF-Sample-Assets", SAMPLES, "192e679b411954fb43cd17967ec4ba5847abc9eca1d12847d5d4e4e0272316f1", "samples"))
    for repo, commit, digest, folder in suites:
        target = root / folder
        stamp = target / ".complete"
        if stamp.exists() and stamp.read_text() == commit:
            continue
        archive = pinned(f"https://codeload.github.com/KhronosGroup/{repo}/tar.gz/{commit}", digest, f"gltf/{commit}.tar.gz")
        if target.exists():
            shutil.rmtree(target)
        target.mkdir(parents=True)
        with tarfile.open(archive) as tar:
            members = []
            for member in tar.getmembers():
                parts = member.name.split("/")[1:]
                selected = (len(parts) >= 4 and parts[0] == "Models" and parts[2] in ("glTF", "glTF-Binary")) if folder == "samples" else parts[:3] == ["test", "base", "data"]
                if selected:
                    member.name = "/".join(parts)
                    members.append(member)
            tar.extractall(target, members=members, filter="data")
        stamp.write_text(commit)
    return root


if __name__ == "__main__":
    print(fetch(samples="--samples" in sys.argv[1:]))
