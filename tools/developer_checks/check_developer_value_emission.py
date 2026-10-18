#!/usr/bin/env python3
"""Execute public generator value printers and generated typed Jai values."""
from dev_common import run_checked, ROOT, JAIC



def run(command, output=""):
    return run_checked(command, label="Value emission", expected=output)


def main():
    emitter = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-value-emission.jai"
    errors = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-value-emission-errors.jai"
    consumer = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-value-emission-roundtrip.jai"
    generated = ROOT / "target/developer-tools-value-emission-generated.jai"
    for mode in ["interp", "0", "2"]:
        for source in [emitter, errors]:
            if mode == "interp":
                run([JAIC, source, "-interp"], "ok\n")
            else:
                executable = ROOT / f"target/developer-tools-{source.stem}-o{mode}"
                run([JAIC, source, f"-O{mode}", "-exe", executable])
                run([executable], "ok\n")
        run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", generated])
        run([JAIC, consumer, "-interp", "-I", ROOT / "target"], "ok\n")
        for optimization in [0, 2]:
            executable = ROOT / f"target/developer-tools-value-consumer-{mode}-o{optimization}"
            run([JAIC, consumer, f"-O{optimization}", "-exe", executable, "-I", ROOT / "target"])
            run([executable], "ok\n")
    paths = [emitter, errors, consumer, generated, ROOT / "stdlib/Bindings_Generator/module.jai",
             ROOT / "stdlib/internal/developer-tools/binding-value-emission.jai",
             ROOT / "stdlib/internal/developer-tools/binding-type-emission.jai",
             ROOT / "stdlib/internal/developer-tools/binding-native-model.jai"]
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", *paths])
    run([ROOT / "target/debug/jailint", "-D", "warnings", "-I", ROOT / "target", *paths])
    print("Seven public generator value/expression helpers and generated integer/float/string/enum/operator effects passed interpreter/native Mac O0/O2, formatting and strict lint")


if __name__ == "__main__":
    main()
