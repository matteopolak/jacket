"""Third-party reference measurements for `measure.py`: protoc and segcore/jai-protobuf.

Both are untrusted upstream code. They only ever run inside `bench.sandbox.Sandbox` (no network,
writes only to the variant's scratch folder), never directly.
"""
from __future__ import annotations

import json
import re
import subprocess
import sys
import time
from pathlib import Path

import measure

HERE = Path(__file__).resolve().parent
PROTOC = "/Volumes/LodestoneScratch/protobuf-compare/build-conf2/protoc-38.0.0"
PROTOBUF_SRC = "/Volumes/LodestoneScratch/protobuf-compare/protobuf/src"
JAI_PROTOBUF = HERE.parents[1] / "corpus" / "upstream" / "segcore--jai-protobuf"
if not JAI_PROTOBUF.is_dir():  # a worktree does not carry the corpus: use the main checkout's
    JAI_PROTOBUF = Path("/Users/matthew/projects/jai/corpus/upstream/segcore--jai-protobuf")

GEN_JP = """#import,dir "%LIB%";
#import "Basic";
#import "File";

files :: string.[%FILES%];
dirs :: string.[%DIRS%];

main :: () {
    start := current_time_monotonic();
    code, failed := generate_jai(..files, dirs);
    ms := to_float64_seconds(current_time_monotonic() - start) * 1000.0;
    write_entire_file("%OUT%", code);
    print("gen_ms=% bytes=% failed=%\\n", ms, code.count, failed.count);
}
"""

PROG_JP = """#import,dir "%LIB%";
#import "Basic";

gen :: () -> string {
    files := string.[%FILES%];
    dirs := string.[%DIRS%];
    code, failed := generate_jai(..files, dirs);
    return code;
}
#insert #run gen();

main :: () {
%BODY%
    print("OK\\n");
}
"""


def sandboxed(arguments, scratch: Path, reads: list[Path]):
    """(wrap, env) for running untrusted code: reads limited to `reads` and the compiler's install."""
    sys.path.insert(0, str(HERE.parent))
    from bench.sandbox import Sandbox, environment
    from bench.dialects import parse_compiler
    from bench.cmd_compile import install_reads
    scratch.mkdir(parents=True, exist_ok=True)
    box = Sandbox(reads=[*install_reads(parse_compiler(f"jaic={arguments.jaic}")), *reads], writes=[scratch])
    return box.wrap, environment(scratch)


def proto_files(variant: dict, set_root: Path) -> list[str]:
    """The entry files of a variant relative to the set root: its path, or every file of its dir."""
    if "dir" in variant:
        return sorted(p.relative_to(set_root).as_posix() for p in (set_root / variant["dir"]).rglob("*.proto"))
    return [variant["path"]]


IMPORT = re.compile(r'^\s*import\s+(?:public\s+|weak\s+)?"([^"]+)"', re.M)


def closure(files: list[str], roots: list[Path]) -> list[str]:
    """`files` and everything they import (found under `roots`), except google/protobuf/*, which the
    protobuf runtime already ships generated: protoc only writes code for the files it is given."""
    seen: dict[str, None] = {}
    todo = list(files)
    while todo:
        name = todo.pop()
        if name in seen or name.startswith("google/protobuf/"):
            continue
        seen[name] = None
        for root in roots:
            if (root / name).exists():
                todo += IMPORT.findall((root / name).read_text(errors="replace"))
                break
    return sorted(seen)


def run_boxed(wrap, command, lock, cwd, env, timeout=measure.TIMEOUT):
    with measure.heavy(lock):
        start = time.perf_counter()
        try:
            done = subprocess.run(wrap(command), cwd=cwd, env=env, capture_output=True, text=True, timeout=timeout)
        except subprocess.TimeoutExpired:
            done = subprocess.CompletedProcess(command, 124, "", f"timeout after {timeout}s")
        return time.perf_counter() - start, done


def measure_protoc(variant: dict, arguments, cache: Path, work: Path) -> dict:
    set_root = cache / variant["set"]
    out = work / "cpp"
    subprocess.run(["rm", "-rf", str(out)])
    out.mkdir(parents=True)
    extra = cache / "descriptor"
    files = closure(proto_files(variant, set_root), [set_root, extra])
    wrap, env = sandboxed(arguments, work, [Path(PROTOC).parent, set_root, Path(PROTOBUF_SRC)])
    command = [PROTOC, f"--cpp_out={out}", "-I", str(set_root), "-I", PROTOBUF_SRC, *files]
    wall, done = run_boxed(wrap, command, None, work, env)
    result: dict = {"protoc_s": round(wall, 3), "protoc_ok": done.returncode == 0}
    if done.returncode != 0:
        result["protoc_error"] = measure.tail(done, 6)
        return result
    generated = [p for p in out.rglob("*") if p.suffix in (".h", ".cc")]
    result["cpp_bytes"] = sum(p.stat().st_size for p in generated)
    if arguments.cxx:
        include = ["-I", str(out), "-I", PROTOBUF_SRC, "-I", "/opt/homebrew/include"]
        total = 0
        spent = 0.0
        for source in [p for p in generated if p.suffix == ".cc"]:
            obj = source.with_suffix(".o")
            command = ["/usr/bin/clang++", "-std=c++17", "-O2", "-c", *include, str(source), "-o", str(obj)]
            took, built = run_boxed(wrap, command, arguments.lock, work, env, timeout=measure.TIMEOUT * 4)
            if built.returncode != 0:
                result["cxx_error"] = measure.tail(built, 4)
                return result
            total += obj.stat().st_size
            spent += took
        result["cxx_s"] = round(spent, 1)
        result["cxx_object_bytes"] = total
    return result


def measure_jai_protobuf(variant: dict, arguments, cache: Path, work: Path) -> dict:
    set_root = cache / variant["set"]
    files = proto_files(variant, set_root)
    dirs = [str(set_root), str(cache / "descriptor"), PROTOBUF_SRC]

    def quote(items):
        return ", ".join(json.dumps(i) for i in items)

    jp = work / "jp"
    jp.mkdir(parents=True, exist_ok=True)
    out = jp / "generated.jai"
    (jp / "gen.jai").write_text(GEN_JP.replace("%LIB%", str(JAI_PROTOBUF)).replace("%FILES%", quote(files))
                                .replace("%DIRS%", quote(dirs)).replace("%OUT%", str(out)))
    # The library puts each package in a namespace struct, so the dotted proto name works as written.
    body = "\n".join(f"    value_{i}: {t};" for i, t in enumerate(variant["types"]))
    (jp / "prog.jai").write_text(PROG_JP.replace("%LIB%", str(JAI_PROTOBUF)).replace("%FILES%", quote(files))
                                 .replace("%DIRS%", quote(dirs)).replace("%BODY%", body))
    wrap, env = sandboxed(arguments, jp, [JAI_PROTOBUF, set_root, cache / "descriptor", Path(PROTOBUF_SRC)])
    jaic = arguments.jaic
    threads = ["-threads", str(arguments.threads)]
    result: dict = {}
    _, done = run_boxed(wrap, [jaic, *threads, "-interp", str(jp / "gen.jai")], arguments.lock, jp, env)
    match = re.search(r"gen_ms=(\S+) bytes=(\d+) failed=(\d+)", done.stdout)
    if not match:
        result["jp_error"] = measure.tail(done, 30)
        return result
    result["jp_gen_ms"] = round(float(match.group(1)), 1)
    result["jp_emitted_bytes"] = int(match.group(2))
    result["jp_failed_imports"] = int(match.group(3))
    wall, done = run_boxed(wrap, [jaic, *threads, "-check", str(jp / "prog.jai")], arguments.lock, jp, env)
    result["jp_check_s"] = round(wall, 2)
    (jp / "check.log").write_text(done.stdout + done.stderr)
    if done.returncode != 0:
        errors = [line.split("error[", 1)[1] for line in (done.stdout + done.stderr).splitlines() if "error[" in line]
        result["jp_error"] = "; ".join(dict.fromkeys(errors))[:400] or measure.tail(done, 6)
        return result
    exe = jp / "prog"
    wall, done = run_boxed(wrap, [jaic, *threads, "-O2", str(jp / "prog.jai"), "-exe", str(exe)], arguments.lock, jp, env)
    result["jp_o2_s"] = round(wall, 2)
    if done.returncode != 0 or not exe.exists():
        result["jp_error"] = measure.tail(done, 30)
        return result
    result["jp_binary_bytes"] = exe.stat().st_size
    _, ran = run_boxed(wrap, [str(exe)], None, jp, env)
    result["jp_run_ok"] = ran.returncode == 0 and ran.stdout.strip().endswith("OK")
    return result
