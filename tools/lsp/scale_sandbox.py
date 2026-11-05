"""Run the established upstream LSP scale case under the bench sandbox, without fallback.

Example: PYTHONPATH=tools python3 tools/lsp/scale_sandbox.py --worker target/debug/examples/lsp_scale_worker \
    --stdlib stdlib --source corpus/upstream/focus-editor--focus/src/main.jai
Only this case is delegated; generated scale fixtures use the ordinary Rust harness.
"""
from __future__ import annotations

import argparse
import os
import sys
import tempfile
from pathlib import Path

from bench.measure import run_measured
from bench.sandbox import Sandbox, environment


def required_sandbox(reads: list[Path], writes: list[Path]) -> Sandbox:
    """Refuse unavailable confinement on every platform, including disposable CI runners."""
    if reason := Sandbox.available():
        raise RuntimeError(f"upstream LSP scale case requires the bench sandbox: {reason}")
    return Sandbox(reads=reads, writes=writes)


def child_command(worker: Path, scratch: Path, source: Path, stdlib: Path) -> tuple[list[str], dict[str, str]]:
    """Launch only the private worker, with canonical inputs and no child-authority marker."""
    env = environment(scratch)
    return [str(worker), str(source), str(stdlib)], env


def read_roots(binary: Path, stdlib: Path, source: Path) -> list[Path]:
    """Read only the build, stdlib, upstream project, native libs and ancestor config files."""
    build = next((parent.parent for parent in binary.parents if parent.name in {"debug", "release"}), binary.parent)
    native = [Path(path) for path in os.environ.get("JAIC_NATIVE_LIBS", "").split(os.pathsep) if path]
    reads = [build, stdlib, source.parents[1], *native, Path(sys.base_prefix)]
    # Discovery may inspect enclosing configs. Grant files, not unrelated ancestor contents.
    reads.extend(parent / "jacket.toml" for parent in source.parents if (parent / "jacket.toml").is_file())
    return list(dict.fromkeys(reads))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--worker", type=Path, required=True)
    parser.add_argument("--stdlib", type=Path, required=True)
    parser.add_argument("--source", type=Path, required=True)
    args = parser.parse_args()
    worker, stdlib, source = (path.resolve(strict=True) for path in (args.worker, args.stdlib, args.source))
    with tempfile.TemporaryDirectory(prefix="lsp-scale-sandbox-") as temporary:
        scratch = Path(temporary).resolve()
        box = required_sandbox(read_roots(worker, stdlib, source), [scratch])
        try:
            command, env = child_command(worker, scratch, source, stdlib)
            result = run_measured(box.wrap(command), cwd=scratch, env=env, timeout=180)
            sys.stdout.write(result.stdout)
            sys.stderr.write(result.stderr)
            if result.timed_out:
                print("sandboxed upstream scale harness exceeded its 180-second supervisor", file=sys.stderr)
                return 1
            return 0 if result.returncode == 0 else 1
        finally:
            box.close()


if __name__ == "__main__":
    raise SystemExit(main())
