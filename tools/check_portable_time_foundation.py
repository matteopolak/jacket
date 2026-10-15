#!/usr/bin/env python3
"""Verify portable time with the NEW parameterized foundations producer.

Only target/ is composed. Peer sources/checkouts are read-only. Public facade and
load-list overlap is explicit staging, not production coordinator integration.
"""
import os
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
BASELINE = ROOT / "target/portable-integration-stdlib"
PEER = ROOT.parent / "foundations/stdlib"
LIBRARY = ROOT / "target/portable-time-foundation/stdlib"
OWN_FILES = ["apollo.jai", "time-errors.jai", "clock-calendar.jai", "filetime.jai",
             "windows-clock.jai", "windows-calendar.jai", "elapsed-time.jai", "calendar-text.jai"]


def compose():
    shutil.copytree(BASELINE, LIBRARY, dirs_exist_ok=True)
    for relative in ["Basic", "String", "File_Utilities", "internal/basic",
                     "internal/string", "internal/files", "internal/basic-interface",
                     "internal/basic-context", "internal/native-path"]:
        shutil.copytree(PEER / relative, LIBRARY / relative, dirs_exist_ok=True)
    for filename in OWN_FILES:
        shutil.copyfile(ROOT / "stdlib/internal/basic" / filename,
                        LIBRARY / "internal/basic" / filename)
    shutil.copytree(ROOT / "stdlib/internal/time", LIBRARY / "internal/time", dirs_exist_ok=True)
    load_root = LIBRARY / "internal/basic/module.jai"
    text = load_root.read_text()
    anchor = '#load "apollo.jai";'
    if text.count(anchor) != 1:
        raise RuntimeError("NEW foundation Apollo load-list contract changed")
    additions = [f'#load "{name}";' for name in OWN_FILES[1:]
                 if f'#load "{name}";' not in text]
    load_root.write_text(text.replace(anchor, anchor + '\n' + '\n'.join(additions)))
    facade = LIBRARY / "Basic/module.jai"
    text = facade.read_text()
    public = ["to_apollo", "seconds_since_init", "timespec_to_apollo", "to_timespec",
              "current_time_consensus", "current_time_monotonic", "to_calendar",
              "calendar_to_apollo", "filetime_to_apollo", "to_filetime"]
    portable = ["to_apollo", "calendar_to_string", "calendar_to_iso_string", "ConvertToApollo"]
    unix = ["timespec_to_apollo", "to_timespec"]
    windows = ["filetime_to_apollo", "to_filetime"]
    common = [name for name in public if name not in portable + unix + windows]
    additions = []
    for names, condition in [(portable, None), (unix, "OS == .MACOS || OS == .LINUX || OS == .ANDROID"),
                             (windows, "OS == .WINDOWS"),
                             (common, "OS == .MACOS || OS == .LINUX || OS == .ANDROID || OS == .WINDOWS")]:
        lines = [f"{name} :: Impl.{name};" for name in names
                 if not any(line.lstrip().startswith(f"{name} ::") for line in text.splitlines())]
        if lines:
            additions.append((f"#if {condition} {{\n" + '\n'.join(lines) + "\n}")
                             if condition else '\n'.join(lines))
    anchor = '#scope_file\nImpl :: #import "internal/basic"'
    if text.count(anchor) != 1:
        raise RuntimeError("NEW foundation private facade contract changed")
    facade.write_text(text.replace(anchor, '\n'.join(additions) + '\n\n' + anchor))
    formatter = str(ROOT / "target/jaifmt")
    config = str(ROOT / "jaifmt.toml")
    for mode in [[], ["--check"]]:
        subprocess.run([formatter, "--config", config, *mode, str(facade), str(load_root)], check=True)
    return facade, load_root


def main():
    initial = tuple(path.read_bytes() for path in compose())
    repeated = tuple(path.read_bytes() for path in compose())
    if initial != repeated:
        raise RuntimeError("normalized foundation/time composition is not regeneration-stable")
    print("NEW foundation facade/load-list regeneration and formatting are stable", flush=True)
    env = dict(os.environ, JAIC_STDLIB=str(LIBRARY))
    compiler = str(ROOT / "target/compiler/debug/jaic")
    fixture = ROOT / "tests/portable-stdlib/time-configured/main.jai"
    for cpu in ["x64", "arm64"]:
        subprocess.run([compiler, "-check", "-os", "windows", "-cpu", cpu, str(fixture)],
                       cwd=ROOT, env=env, check=True)
    for name in ["time-configured/main", "time-conversion", "elapsed-time", "calendar-text", "calendar-native", "apollo-caller-return"]:
        source = ROOT / "tests/portable-stdlib" / (name + ".jai")
        commands = [[compiler, "-interp", str(source)]]
        for optimization in ["-O0", "-O2"]:
            executable = ROOT / "target/portable-time-foundation" / (name.replace('/', '-') + optimization)
            subprocess.run([compiler, str(source), optimization, "-exe", str(executable)], cwd=ROOT, env=env, check=True)
            commands.append([str(executable)])
        for command in commands:
            result = subprocess.run(command, cwd=ROOT, env=env, text=True,
                                    capture_output=True, check=True)
            if result.stdout != "ok\n" or result.stderr:
                raise RuntimeError(f"{command!r}\n{result.stdout}{result.stderr}")
        print(f"NEW foundation time integration passed {name}", flush=True)


if __name__ == "__main__":
    main()
