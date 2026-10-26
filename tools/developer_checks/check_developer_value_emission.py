#!/usr/bin/env python3
"""Execute public generator value printers and generated typed Jai values."""
from developer_checks.dev_common import JAIC, JAIFMT, ROOT, run_checked, cargo_binary, output_dir



def run(command, output=""):
    return run_checked(command, label="Value emission", expected=output)


def main():
    emitter = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-value-emission.jai"
    errors = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-value-emission-errors.jai"
    consumer = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-value-emission-roundtrip.jai"
    generated = output_dir("developer-tools-value-emission-generated.jai")
    for mode in ["interp", "0", "2"]:
        for source in [emitter, errors]:
            if mode == "interp":
                run([JAIC, source, "-interp"], "ok\n")
            else:
                executable = output_dir(f"developer-tools-{source.stem}-o{mode}")
                run([JAIC, source, f"-O{mode}", "-exe", executable])
                run([executable], "ok\n")
        run([JAIFMT, "--config", ROOT / "jacket.toml", generated])
        run([JAIC, consumer, "-interp", "-I", output_dir()], "ok\n")
        for optimization in [0, 2]:
            executable = output_dir(f"developer-tools-value-consumer-{mode}-o{optimization}")
            run([JAIC, consumer, f"-O{optimization}", "-exe", executable, "-I", output_dir()])
            run([executable], "ok\n")
    paths = [emitter, errors, consumer, generated, ROOT / "stdlib/Bindings_Generator/module.jai",
             ROOT / "stdlib/internal/developer-tools/binding-value-emission.jai",
             ROOT / "stdlib/internal/developer-tools/binding-type-emission.jai",
             ROOT / "stdlib/internal/developer-tools/binding-native-model.jai"]
    run([JAIFMT, "--config", ROOT / "jacket.toml", "--check", *paths])
    run([cargo_binary("jailint"), "-D", "warnings", "-I", output_dir(), *paths])
    print("Seven public generator value/expression helpers and generated integer/float/string/enum/operator effects passed interpreter/native Mac O0/O2, formatting and strict lint")


if __name__ == "__main__":
    main()
