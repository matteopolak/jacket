"""Makes the helpers one directory up (benchgen, jaistats, repo_output) importable.

Import this before them. It appends rather than prepends, so the modules of this directory keep
priority over same-named ones in tools/."""
import sys
from pathlib import Path

TOOLS = str(Path(__file__).resolve().parent.parent)
if TOOLS not in sys.path:
    sys.path.append(TOOLS)
