#!/usr/bin/env python3
"""Public callable omission visitor over real canonical model/closure storage."""
from developer_checks.dev_common import JAIFMT, ROOT, labeled_runner, cargo_binary, output_dir


run = labeled_runner('Declaration visitor')


def main():
    source = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-omission-visitor.jai"
    jaic = cargo_binary("jaic")
    run([jaic, source, "-interp"], "ok\n")
    for optimization in [0, 2]:
        executable = output_dir(f"developer-tools-omission-visitor-o{optimization}")
        run([jaic, source, f"-O{optimization}", "-exe", executable])
        run([executable], "ok\n")
    paths = [source, ROOT / "stdlib/Bindings_Generator/module.jai",
             ROOT / "stdlib/internal/developer-tools/binding-declaration-visitors.jai"]
    run([JAIFMT, "--config", ROOT / "jacket.toml", "--check", *paths])
    run([cargo_binary("jailint"), "-D", "warnings", *paths])
    print("Public omission visitor/closure/callback ordering/STOP effects passed interpreter/native Mac O0/O2, formatting and strict lint")


if __name__ == "__main__":
    main()
