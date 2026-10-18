"""Select caller-assigned test scratch; never silently use the host temp default."""

import os
from pathlib import Path


def assigned_scratch(environment=None):
    env = os.environ if environment is None else environment
    base = env.get("CARGO_TARGET_DIR")
    if not base and env.get("CI") == "true":
        base = env.get("RUNNER_TEMP")
        if not base and env.get("GITHUB_WORKSPACE"):
            base = str(Path(env["GITHUB_WORKSPACE"]) / "target/test-scratch")
    if not base:
        base = env.get("TMPDIR")
    if not base:
        raise RuntimeError(
            "assign CARGO_TARGET_DIR or TMPDIR locally, or CI runner/workspace scratch"
        )
    directory = Path(base)
    directory.mkdir(parents=True, exist_ok=True)
    return directory
