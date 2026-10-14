"""Normalize generated Jai with the actual source-built repository formatter."""
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_FORMATTER = ROOT / "target/jaifmt"


def emit_generated(path, source, formatter=DEFAULT_FORMATTER, check=False):
    formatter = Path(formatter).resolve(strict=True)
    command = [str(formatter), "--stdin", "--config", str(ROOT / "jaifmt.toml")]
    formatted = subprocess.run(command, input=source, text=True, capture_output=True, check=True).stdout
    repeated = subprocess.run(command, input=formatted, text=True, capture_output=True, check=True).stdout
    if repeated != formatted:
        raise ValueError(f"repository formatter is not idempotent for {path}")
    path = Path(path)
    if check:
        if not path.is_file() or path.read_text() != formatted:
            raise ValueError(f"stale generated source: {path}")
    else:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(formatted)
