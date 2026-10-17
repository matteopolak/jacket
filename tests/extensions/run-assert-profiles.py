"""Execute real Io imports under both assertion profiles without source overlays."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from repo_output import output_dir

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, default=ROOT / "target/debug/jaic")
    parser.add_argument("--output", type=Path, default=output_dir("extensions-assert-profiles"))
    args = parser.parse_args()
    compiler = args.compiler.resolve(strict=True)
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))

    def fingerprints():
        return {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
                for p in (ROOT / "stdlib").rglob("*.jai")}

    before = fingerprints()
    checks = []

    def run(command, expected):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, env=env,
                                capture_output=True, text=True, timeout=90)
        row = dict(command=list(map(str, command)), exit=result.returncode,
                   stdout=result.stdout, stderr=result.stderr)
        checks.append(row)
        (output / "results.json").write_text(json.dumps(checks, indent=2) + "\n")
        if result.returncode or result.stdout != expected or result.stderr:
            raise RuntimeError(json.dumps(row, indent=2))

    host = "arm64" if platform.machine().lower() in ("arm64", "aarch64") else "x64"
    cpus = ["arm64", "x64"] if platform.system() == "Darwin" else [host]
    parent = ROOT / "tests/extensions/io-configured-production.jai"
    source = ROOT / "tests/extensions/io-configured-workers.jai"
    executions = 0
    for setting in ("enabled", "disabled"):
        exe = output / (setting + "-interp")
        run([compiler, "-interp", parent, "-", setting, source, exe, "interp", host], "ok\n")
        executions += 1
        for cpu in cpus:
            for mode in ("O0", "O2"):
                exe = output / (setting + "-" + cpu + "-" + mode)
                run([compiler, "-interp", parent, "-", setting, source, exe, mode, cpu], "")
                run([exe], "ok\n")
                executions += 1
    if fingerprints() != before:
        raise RuntimeError("production Jai source changed during configured checks")
    print(f"PASS {executions} actual Io worker executions; both assertion profiles; production sources unchanged")


if __name__ == "__main__":
    main()
