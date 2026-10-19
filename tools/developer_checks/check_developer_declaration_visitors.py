#!/usr/bin/env python3
"""Public callable omission visitor over real canonical model/closure storage."""
import os
from pathlib import Path
from dev_common import ROOT, labeled_runner


run = labeled_runner('Declaration visitor')


def main():
    source = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-omission-visitor.jai"
    jaic = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "debug/jaic"
    run([jaic, source, "-interp"], "ok\n")
    for optimization in [0, 2]:
        executable = ROOT / f"target/developer-tools-omission-visitor-o{optimization}"
        run([jaic, source, f"-O{optimization}", "-exe", executable])
        run([executable], "ok\n")
    paths = [source, ROOT / "stdlib/Bindings_Generator/module.jai",
             ROOT / "stdlib/internal/developer-tools/binding-declaration-visitors.jai"]
    run([ROOT / "target/jaifmt", "--config", ROOT / "jacket.toml", "--check", *paths])
    run([Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "debug/jailint", "-D", "warnings", *paths])
    print("Public omission visitor/closure/callback ordering/STOP effects passed interpreter/native Mac O0/O2, formatting and strict lint")


if __name__ == "__main__":
    main()
