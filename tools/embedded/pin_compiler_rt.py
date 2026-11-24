"""Pins the compiler-rt release and the source files of its builtins for each freestanding CPU.

`jacket toolchain add` compiles these files with the installed clang. The list is what
compiler-rt's own `lib/builtins/CMakeLists.txt` builds for a bare-metal target of that CPU, read
from the pinned release, so the archive has the symbols a CMake build would give it.

    PYTHONPATH=tools python3 tools/embedded/pin_compiler_rt.py            # rewrite the pins
    PYTHONPATH=tools python3 tools/embedded/pin_compiler_rt.py --check    # fail if they are stale

The rules this applies, from CMakeLists.txt, for `COMPILER_RT_BAREMETAL_BUILD`:

- `GENERIC_SOURCES` without `emutls.c`, `enable_execute_stack.c` and `eprintf.c`, and without the
  atomic library (`atomic.c`): the C library provides those.
- `__bf16` conversions (`BF16_SOURCES`) only for aarch64: clang has no `__bf16` for Cortex-M or AVR.
- A generic file is dropped when the CPU has its own file of the same name (`arm/udivsi3.S`
  replaces `udivsi3.c`).
- aarch64: no `emupac.cpp` (it emulates PAC from a SIGILL handler and needs LLVM's third-party
  SipHash, neither of which exists on bare metal); the 128-bit float routines, `lse.S` once per
  outline atomic (operation, size, memory model), and no SME routines.
- Cortex-M is built soft-float (compiler-rt's `armv7em`, `armv8m.main`): the VFP, SjLj and
  Thumb-1 `restore_vfp` routines are left out, as CMake leaves them out without `__ARM_FP`.
"""

import argparse
import hashlib
import re
import sys
import tempfile
from pathlib import Path

from toolkit.repo import ROOT

OUTPUT = ROOT / "crates/jaic-toolchain/data/compiler-rt.txt"

VERSION = "21.1.8"
URL = f"https://github.com/llvm/llvm-project/releases/download/llvmorg-{VERSION}/compiler-rt-{VERSION}.src.tar.xz"
SHA256 = "dd54ae21aee1780fac59445b51ebff601ad016b31ac3a7de3b21126fd3ccb229"

OUTLINE_OPERATIONS = ["cas", "swp", "ldadd", "ldclr", "ldeor", "ldset"]
OUTLINE_SIZES = [1, 2, 4, 8, 16]
OUTLINE_MODELS = [1, 2, 3, 4, 5]

BAREMETAL_EXCLUDED = {"emutls.c", "enable_execute_stack.c", "eprintf.c"}


def fetch_tree(directory: Path) -> Path:
    """The pinned release, unpacked under `directory`."""
    from toolkit.download import fetch

    data = fetch(URL)
    digest = hashlib.sha256(data).hexdigest()
    if digest != SHA256:
        raise SystemExit(f"{URL} has the sha256 {digest}, not {SHA256}")
    archive = directory / "compiler-rt.tar.xz"
    archive.write_bytes(data)
    import subprocess

    subprocess.run(["tar", "-xf", str(archive), "-C", str(directory)], check=True)
    return directory / f"compiler-rt-{VERSION}.src/lib/builtins"


def named_lists(cmake: str) -> dict[str, list[str]]:
    """Every `set(NAME ...)` list of the file, the first definition of each name, with `${NAME}`
    references expanded."""
    text = re.sub(r"#[^\n]*", "", cmake)
    lists: dict[str, list[str]] = {}
    for match in re.finditer(r"\bset\(\s*([A-Za-z0-9_.]+)((?:[^()]|\([^()]*\))*)\)", text):
        name, body = match.group(1), match.group(2)
        if name in lists:
            continue
        words: list[str] = []
        for word in body.split():
            reference = re.fullmatch(r"\$\{([A-Za-z0-9_.]+)\}", word)
            if reference:
                words.extend(lists.get(reference.group(1), []))
            else:
                words.append(word)
        lists[name] = words
    return lists


def without_replaced(generic: list[str], own: list[str]) -> list[str]:
    """`generic` without the files `own` replaces: same stem, in the CPU's folder."""
    replaced = {Path(file).stem for file in own if "/" in file}
    return [file for file in generic if "/" in file or Path(file).stem not in replaced]


def families(lists: dict[str, list[str]]) -> dict[str, list[tuple[str, list[str]]]]:
    generic = [f for f in lists["GENERIC_SOURCES"] if f not in BAREMETAL_EXCLUDED]
    bf16 = lists["BF16_SOURCES"]

    def plain(files: list[str]) -> list[tuple[str, list[str]]]:
        seen: list[str] = []
        for file in files:
            if file not in seen:
                seen.append(file)
        return [(file, []) for file in seen]

    aarch64_own = ["cpu_model/aarch64.c", "aarch64/fp_mode.c"]
    aarch64 = plain(
        lists["GENERIC_TF_SOURCES"] + without_replaced(generic, aarch64_own) + aarch64_own + bf16
    )
    for operation in OUTLINE_OPERATIONS:
        for size in OUTLINE_SIZES:
            if operation != "cas" and size == 16:
                continue
            for model in OUTLINE_MODELS:
                aarch64.append(
                    (
                        "aarch64/lse.S",
                        [f"L_{operation}", f"SIZE={size}", f"MODEL={model}"],
                    )
                )

    arm_own = [f for f in lists["arm_or_thumb2_base_SOURCES"] if f.startswith("arm/")]
    arm_own += lists["arm_sync_SOURCES"] + lists["arm_EABI_RT_SOURCES"] + lists["arm_EABI_CLIB_SOURCES"]
    arm_own += lists["arm_Thumb1_JT_SOURCES"] + lists["arm_Thumb1_icache_SOURCES"]
    cortexm = plain(without_replaced(generic, arm_own) + arm_own)

    avr_own = [f for f in lists["avr_SOURCES"] if f.startswith("avr/")]
    avr = plain(without_replaced(generic, avr_own) + avr_own)
    return {"aarch64": aarch64, "cortexm": cortexm, "avr": avr}


def render(builtins: Path) -> str:
    lists = named_lists((builtins / "CMakeLists.txt").read_text())
    lines = [
        f"# compiler-rt {VERSION}: the builtins of each freestanding CPU. Written by",
        "# tools/embedded/pin_compiler_rt.py; see its docstring for the rules.",
        f"release {VERSION} {SHA256} {URL}",
        "# family source [definition...]",
    ]
    for family, sources in families(lists).items():
        for source, definitions in sources:
            if not (builtins / source).is_file():
                raise SystemExit(f"{source} is not in compiler-rt {VERSION}")
            lines.append(" ".join([family, source, *definitions]))
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--check", action="store_true", help="fail when the file differs")
    args = parser.parse_args()
    with tempfile.TemporaryDirectory() as directory:
        text = render(fetch_tree(Path(directory)))
    if args.check:
        if OUTPUT.read_text() != text:
            print(f"{OUTPUT} is stale: run tools/embedded/pin_compiler_rt.py", file=sys.stderr)
            return 1
        return 0
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(text)
    print(f"wrote {OUTPUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
