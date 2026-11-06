"""Exercise the Linux sandbox with setup-node's external installation.

python3 tools/portable/image_conformance/jpeg/check_node_sandbox.py --require-external-prefix
"""

import argparse
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

from toolkit.repo import ROOT

sys.path.insert(0, str(ROOT / "tools/bench"))
from sandbox import LINUX_SYSTEM_READS, Sandbox


def main():
    """Run the resolved Node executable with only its install and an empty work mount."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--node", default="node")
    parser.add_argument("--require-external-prefix", action="store_true")
    args = parser.parse_args()
    if not sys.platform.startswith("linux"):
        raise SystemExit("This acceptance check requires the actual Linux sandbox.")
    located = shutil.which(args.node)
    if located is None:
        raise SystemExit("Node unavailable")
    executable = Path(located).resolve()
    if args.require_external_prefix and any(
        executable.is_relative_to(Path(root)) for root in LINUX_SYSTEM_READS
    ):
        raise SystemExit("Expected setup-node outside system-read mounts")
    with tempfile.TemporaryDirectory() as directory:
        work = Path(directory)
        sandbox = Sandbox(reads=[executable.parent.parent], writes=[work])
        result = subprocess.run(
            sandbox.wrap([str(executable), "--version"]),
            cwd=work,
            capture_output=True,
            text=True,
            check=True,
            timeout=30,
        )
        if not result.stdout.startswith("v24."):
            raise SystemExit("Expected the configured Node24 runtime")
        print(
            "External-prefix Node executed inside Linux sandbox: "
            + result.stdout.strip()
        )


if __name__ == "__main__":
    main()
