#!/usr/bin/env python3
"""Original strict-lint regressions using only an explicitly independent stdlib."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jailint", type=Path, required=True)
    parser.add_argument("--stdlib", type=Path, required=True)
    args = parser.parse_args()
    directory = ROOT / "target/platform-io-lint-regressions"
    stdlib = directory / "stdlib"
    shutil.copytree(args.stdlib.resolve(strict=True), stdlib, dirs_exist_ok=True)
    private = stdlib / "internal/lint-probe.jai"
    private.write_text('probe :: (value: int) -> int { return value + 0; }\n')
    env = dict(os.environ, JAIC_STDLIB=str(stdlib))
    binary = args.jailint.resolve(strict=True)

    def check(path, extra, expected, text):
        result = subprocess.run([str(binary), "-D", "warnings", "--config", str(ROOT / "jailint.toml"),
                                 *extra, str(path)], cwd=ROOT, env=env, text=True, capture_output=True, timeout=60)
        assert result.returncode == expected and text in result.stdout + result.stderr, result

    check(private, ["--os", "windows", "--cpu", "x64"], 1, "identity_op")
    for cpu in ["x64", "arm64"]:
        private.write_text('#if OS == .WINDOWS probe :: (value: int) -> int { return value + 0; }\n'
                           'else probe :: (value: int) -> int { return value; }\n')
        check(private, ["--os", "windows", "--cpu", cpu], 1, "identity_op")
    check(private, ["--os", "macos", "--cpu", "arm64"], 0, "")
    app = directory / "application.jai"
    app.write_text('#import "internal/lint-probe";\nmain :: () {}\n')
    check(app, [], 1, "internal sources are private")
    app.write_text('main :: () -> s32 { return REQUIRED; }\n')
    check(app, [], 1, "REQUIRED")
    check(app, ["--add", "REQUIRED :: 17;"], 0, "")
    private.write_text('probe :: () -> int { return missing_value; }\n')
    check(private, [], 1, "missing_value")
    assert not (stdlib / ".jailint-module-root.jai").exists()
    print("passed private module, target selection, privacy and failed-body lint regressions")


if __name__ == "__main__":
    main()
