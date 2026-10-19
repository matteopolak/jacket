#!/usr/bin/env python3
"""Run every generator of checked-in files with --check: each must reproduce its output exactly.

    python3 tools/generators/check_generated.py [--formatter target/jaifmt] [--offline]

The SDL3 and native bindings generators are checked by their own CI steps, since they need
fetched sources or their own working directory.
"""

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

import argparse
from pathlib import Path
import subprocess
import sys
from toolkit.repo import ROOT


# Generators that format their output with the repository jaifmt.
FORMATTED = [
    "tools/generators/audio_generate_signatures.py",
    "tools/generators/generate_jai_lexer_contract.py",
    "tools/generators/generate_machine_x64.py",
    "tools/generators/generate_mpeg_tables.py",
    "tools/generators/generate_simd_lanes.py",
    "tools/generators/generate_socket_abi.py",
    "tools/generators/window_render_evdev.py",
    "tools/generators/window_render_gl_formats.py",
    "tools/developer_checks/generate_developer_binding_model.py",
]
# Generators that write their output verbatim.
VERBATIM = [
    "tools/generators/window_render_feature_font.py",
    "tools/corpus/gen_json_corpus.py",
    "tools/corpus/gen_http_corpus.py",
]
# Generators that download their pinned, hash-checked input.
DOWNLOADING = [
    "tools/generators/gen_io_uring.py",
    "tools/generators/gen_regex_unicode.py",
]


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--formatter", type=Path, default=ROOT / "target/jaifmt",
                        help="repository jaifmt built from source (default: target/jaifmt)")
    parser.add_argument("--offline", action="store_true", help="skip the generators that download their input")
    args = parser.parse_args()
    formatter = str(args.formatter.resolve(strict=True))
    commands = [[script, "--formatter", formatter] for script in FORMATTED]
    commands += [[script] for script in VERBATIM + ([] if args.offline else DOWNLOADING)]
    failed = []
    for script, *options in commands:
        result = subprocess.run([sys.executable, script, "--check", *options], cwd=ROOT)
        if result.returncode != 0:
            failed.append(script)
    if failed:
        print("stale generated files; rerun without --check: " + ", ".join(failed), file=sys.stderr)
        return 1
    print(f"all {len(commands)} generators reproduce their checked-in files")
    return 0


if __name__ == "__main__":
    sys.exit(main())
