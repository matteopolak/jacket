#!/usr/bin/env python3
"""Author client setup variants without a compile-time run or bootstrap edit."""
from itertools import product
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    output = ["// Independently authored client setup; reviewed purpose, no literal-text compatibility promise.", ""]
    for manual, graphics, history in product([False, True], repeat=3):
        flags = [("MANUAL_MODE", manual), ("USE_GRAPHICS", graphics), ("DO_HISTORY", history)]
        condition = " && ".join(name if enabled else "!" + name for name, enabled in flags)
        arguments = ", ".join(f"{name} = {str(enabled).lower()}" for name, enabled in flags)
        output += [f"#if {condition} {{", "    TO_INSERT :: #string PROFILE_CLIENT_SETUP", "#scope_file",
                   f'Independent_Profile_Client :: #import "Iprof"(IMPORT_MODE = .CLIENT, {arguments});',
                   "#add_context _Iprof :: Independent_Profile_Client;", "PROFILE_CLIENT_SETUP", "}", ""]
    destination = ROOT / "stdlib/internal/developer-tools/profiling/client-setup-text.jai"
    destination.write_text("\n".join(output))
    subprocess.run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", destination], check=True)


if __name__ == "__main__":
    main()
