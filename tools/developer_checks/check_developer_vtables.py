#!/usr/bin/env python3
"""Generate vtable fields and call them against a host C++ implementation."""
import os
from pathlib import Path
import platform
import re
import shutil

from dev_common import ENV, ROOT, cargo_target_dir, run_checked
from repo_output import output_dir
import build_developer_bindings


def main():
    prefix = build_developer_bindings.llvm_prefix()
    target = cargo_target_dir()
    default_jaic = target / "release/jaic"
    if not default_jaic.is_file():
        default_jaic = target / "debug/jaic"
    jaic = Path(os.environ.get("JAIC", default_jaic)).resolve()
    work = output_dir("cpp-vtables")
    work.mkdir(parents=True, exist_ok=True)
    source = ROOT / "stdlib/Bindings_Generator/tests/driven"
    system = platform.system()
    extension = {"Darwin": "dylib", "Linux": "so", "Windows": "dll"}[system]
    library = work / ("dispatch.dll" if system == "Windows" else f"libdispatch.{extension}")
    flags = ["-dynamiclib"] if system == "Darwin" else ["-shared"]
    if system == "Linux":
        flags.append("-fPIC")
    env = dict(ENV, JAIC_NATIVE_LIBS=os.pathsep.join(filter(None, [str(work), str(prefix / "lib"), str(prefix / "bin"), ENV.get("JAIC_NATIVE_LIBS")])))

    def run(command, expected=None, stderr=""):
        return run_checked(command, label="C++ vtables", env=env, expected=expected, stderr=stderr, timeout=180)

    if system == "Windows":
        # Native API tests need an import archive at their private library path.
        # Derive it from the actual SDK DLL rather than assuming an SDK archive layout.
        exports = run([prefix / "bin/llvm-readobj", "--coff-exports", prefix / "bin/libclang.dll"]).stdout
        names = sorted(set(re.findall(r"^\s*Name: (\S+)\s*$", exports, re.M)))
        if not names:
            raise RuntimeError("configured libclang DLL has no exported functions")
        definition = work / "libclang.def"
        definition.write_text("LIBRARY libclang.dll\nEXPORTS\n" + "\n".join(names) + "\n")
        machine = "arm64" if platform.machine().lower() in {"arm64", "aarch64"} else "i386:x86-64"
        run([prefix / "bin/llvm-dlltool", "-m", machine, "-D", "libclang.dll", "-d", definition,
             "-l", work / "libclang.lib"])
        # A path-qualified #library resolves beside its declaration on MSVC.
        # Match the private helper staging used by the Unix developer setup.
        private_archive = ROOT / "stdlib/internal/developer-tools/clang-bindings/clang.lib"
        shutil.copyfile(work / "libclang.lib", private_archive)
        env["PATH"] = str(prefix / "bin") + os.pathsep + env.get("PATH", "")

    # Parse foreign triples on every host; runtime dispatch remains host evidence.
    for triple in ["x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu",
                   "x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc"]:
        fixture = work / f"generate-{triple}.jai"
        text = (source / "bindings-vtable-workflow.jai").read_text()
        target_os = 'WINDOWS' if 'windows' in triple else 'LINUX'
        text = text.replace('#import "Bindings_Generator";', f'#import "Bindings_Generator"(TARGET_OS = .{target_os});')
        text = text.replace('tprint("%bindings-vtable-workflow.hpp", #filepath)',
                            '"' + (source / "bindings-vtable-workflow.hpp").as_posix() + '"')
        text = text.replace('to_c_string("c++")]',
                            'to_c_string("c++"), to_c_string("-target"), to_c_string("' + triple + '")]')
        text = text.replace('native_args.data, 2,', 'native_args.data, 4,')
        fixture.write_text(text)
        run([jaic, fixture, "-interp", "--", work / f"{triple}.jai"], "ok\n")

    for optimization in [0, 2]:
        run([prefix / "bin/clang++", "-std=c++17", f"-O{optimization}", *flags,
             source / "bindings-vtable-workflow.cpp", "-o", library])
        for wrappers, accessor in [(False, False), (True, False), (False, True), (True, True)]:
            generated = work / "generated-vtable.jai"
            run([jaic, source / "bindings-vtable-workflow.jai", "-interp", "--", generated,
                 str(wrappers).lower(), str(accessor).lower()], "ok\n")
            (work / "vtable-options.jai").write_text(
                f"expected_wrappers :: {str(wrappers).lower()};\nexpected_accessor :: {str(accessor).lower()};\n")
            consumer = work / "roundtrip.jai"
            consumer.write_text((source / "bindings-vtable-roundtrip.jai").read_text())
            run([jaic, consumer, "-interp", "-I", work], "ok\n")
            executable = work / ("roundtrip.exe" if system == "Windows" else "roundtrip")
            run([jaic, consumer, f"-O{optimization}", "-I", work, "-exe", executable])
            run([executable], "ok\n")
    check_abi(run, prefix, jaic, source, work, flags, library, system)
    check_model(run, prefix, jaic, source, work, system)
    defaults = work / "native-defaults.jai"
    # Windows compiler discovery currently warns even when libclang finds the header.
    # Keep that known warning exact so any new diagnostic still fails this fixture.
    defaults_warning = "Bindings_Generator: C compiler did not provide an include search list\n" if system == "Windows" else ""
    run([jaic, source / "bindings-native-defaults.jai", "-interp", "--", defaults], stderr=defaults_warning)
    run([jaic, defaults, "-check"])
    print("C++ vtable generation and dispatch passed host C++/Jai O0/O2 and interpreter, all helper options")


def check_abi(run, prefix, jaic, source, work, flags, library, system):
    # Force code generation: -fsyntax-only does not necessarily instantiate a vtable.
    result = run([prefix / "bin/clang++", "-std=c++17", "-Xclang", "-fdump-vtable-layouts",
                  "-c", source / "bindings-vtable-abi.cpp", "-o", work / "layout.o"])
    dump = result.stdout + result.stderr
    (work / "clang-vtable-layout.txt").write_text(dump)
    title = "VFTable for 'DispatchBase' in 'DispatchDerived'" if system == "Windows" else "Vtable for 'DispatchDerived'"
    section = dump.split(title, 1)[1].split("\n\n", 1)[0]
    entries = []
    for line in section.splitlines():
        match = re.search(r"^\s*\d+ \| (.*)", line)
        if match and "::" in match[1]:
            entries.append(match[1])
    assert entries, dump
    generated = work / "abi-table.jai"
    model = run([jaic, source / "bindings-vtable-abi.jai", "-interp", "--", generated])
    names = [line.split()[1] for line in model.stdout.splitlines()]
    expected_names = ["Destructor" if "~" in entry else entry.split("::")[-1].split("(")[0] for entry in entries]
    assert names == expected_names, (names, entries)
    # Each slot is named for its method; same-name overloads and destructors keep a positional or numbered name.
    slot_names = re.findall(r"^\s*(\w+): (?:#type|\*void).*;", generated.read_text(), re.M)
    assert len(slot_names) == len(entries), (slot_names, entries)
    consumer = work / "abi-roundtrip.jai"
    text = ('#import "Basic";\n#load "abi-table.jai";\n'
            'DispatchSmall :: struct { a: s32; }\n'
            'DispatchBig :: struct { a: s64; b: s64; c: s64; }\n'
            'DispatchFloatPair :: struct { a: float32; b: float32; }\n'
            'DispatchBase :: struct { vtable: *DispatchDerived_VTable; }\n'
            'DispatchDerived :: struct { #as using base: DispatchBase; }\n'
            'native :: #library "dispatch";\n'
            'abi_create :: () -> *DispatchDerived #foreign native;\n'
            'abi_destroy :: (object: *DispatchDerived) #foreign native;\n'
            'main :: () {\n object := abi_create();\n defer abi_destroy(object);\n')
    for index, entry in enumerate(entries):
        if "~" in entry:
            assert f"slot_{index}: *void" in generated.read_text()
            continue
        receiver = "cast(*DispatchBase) object" if "DispatchBase::" in entry else "object"
        name = expected_names[index]
        if name in {"small", "big", "pair"}:
            shape = {"small": "DispatchSmall", "big": "DispatchBig", "pair": "DispatchFloatPair"}[name]
            argument = "cast(float32) 2" if name == "pair" else "2"
            if system == "Windows":
                text += f" result_{index}: {shape};\n object.vtable.{slot_names[index]}(object, *result_{index}, {argument});\n"
            else:
                text += f" result_{index} := object.vtable.{slot_names[index]}(object, {argument});\n"
            assertion = f"result_{index}.a == 72" if name == "small" else f"result_{index}.a == 82 && result_{index}.b == 92 && result_{index}.c == 102" if name == "big" else f"result_{index}.a == 3.5 && result_{index}.b == 4.5"
            text += f" assert({assertion});\n"
            continue
        if name == "take":
            text += f" assert(object.vtable.{slot_names[index]}(object, .{{7}}, .{{11, 13, 17}}) == 48);\n"
            continue
        floating = "(float)" in entry
        value = "cast(float32) 2" if floating else "2"
        expected = 22 if "choose(float)" in entry else 42 if "choose(int)" in entry else 32 if "tail(" in entry else 62 if floating else 52
        text += f" assert(object.vtable.{slot_names[index]}({receiver}, {value}) == {expected});\n"
    text += ' print("ok\\n");\n}\n'
    consumer.write_text(text)
    for optimization in [0, 2]:
        run([prefix / "bin/clang++", "-std=c++17", f"-O{optimization}", *flags,
             source / "bindings-vtable-abi.cpp", "-o", library])
        run([jaic, consumer, "-interp"], "ok\n")
        executable = work / ("abi-roundtrip.exe" if system == "Windows" else "abi-roundtrip")
        run([jaic, consumer, f"-O{optimization}", "-exe", executable])
        run([executable], "ok\n")


def check_model(run, prefix, jaic, source, work, system):
    executable = work / ("model-layout.exe" if system == "Windows" else "model-layout")
    compiled = run([prefix / "bin/clang++", "-std=c++17", "-Xclang", "-fdump-vtable-layouts",
                    source / "generator-model-vtable-layout.cpp", "-o", executable])
    dump = compiled.stdout + compiled.stderr
    title = "VFTable for 'OriginalVirtualDestructor'" if system == "Windows" else "Vtable for 'OriginalVirtualDestructor'"
    section = dump.split(title, 1)[1].split("\n\n", 1)[0]
    slots = sum("| OriginalVirtualDestructor::~" in line for line in section.splitlines())
    assert slots > 0
    values = run([executable]).stdout
    (work / "generator-model-vtable-layout-oracle.jai").write_text(values + f"Native_Virtual_Destructor_Slots :: {slots};\n")
    run([jaic, source / "generator-model-vtables.jai", "-interp", "-I", work], "ok\n")
    program = work / ("model.exe" if system == "Windows" else "model")
    run([jaic, source / "generator-model-vtables.jai", "-O2", "-I", work, "-exe", program])
    run([program], "ok\n")


if __name__ == "__main__":
    main()
