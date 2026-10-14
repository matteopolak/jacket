"""Normalize generated owned sources with the repository formatter engine."""
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]

def normalize(source):
    default = ROOT/'artifacts/jaifmt-current'
    formatter = Path(os.environ.get('NUMERICS_JAIFMT', default))
    if not formatter.is_file():
        raise SystemExit('Build repository jaifmt/main.jai from canonical producers before regenerating numerics')
    result = subprocess.run([str(formatter), '--stdin', '--config', str(ROOT/'jaifmt.toml')], input=source, text=True,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE, cwd=ROOT)
    if result.returncode:
        raise SystemExit(result.stderr or 'repository formatter failed')
    return result.stdout

def write_generated(path, source):
    Path(path).write_text(normalize(source))
