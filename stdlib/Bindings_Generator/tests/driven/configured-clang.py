#!/usr/bin/env python3
"""Owned SDK test wrapper: record include probes, delegate every compiler call."""
import json
from pathlib import Path
import subprocess
import sys

configuration = json.loads((Path(sys.argv[0]).absolute().parent.parent / "compiler.json").read_text())
if "-E" in sys.argv[1:] and "-v" in sys.argv[1:]:
    Path(configuration["include_probe_marker"]).write_text("configured official SDK compiler invoked\n")
raise SystemExit(subprocess.run([configuration["compiler"], *sys.argv[1:]]).returncode)
