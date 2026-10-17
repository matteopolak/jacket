"""Native integration and explicit unsupported-contract checks; no reference tools."""

from pathlib import Path
import argparse
import json
import os
import subprocess
import sys
from target_contracts import target_contract

root = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(root / "tools"))
from repo_output import output_dir
compiler = output_dir("numerics", "current-target", "debug", "jaic")
parser = argparse.ArgumentParser()
parser.add_argument(
    "--stdlib", type=Path, default=output_dir("numerics", "canonical-reviewed")
)
parser.add_argument("--compiler", type=Path, default=compiler)
parser.add_argument("--canonical-dependencies", action="store_true", default=True)
parser.add_argument(
    "--output", type=Path, default=output_dir("numerics", "canonical-corrective-reviewed")
)
parser.add_argument(
    "--private-x11-display",
    action="store_true",
    help="Caller owns an ephemeral X11 display; required for Linux clipboard execution",
)
args = parser.parse_args()
library = args.stdlib.resolve()
compiler = args.compiler.resolve()
out = args.output.resolve()
if not out.is_relative_to(output_dir().resolve()):
    raise SystemExit("evidence must remain in the repository output directory")
out.mkdir(parents=True, exist_ok=True)
imports = []
env = dict(os.environ, JAIC_STDLIB=str(library))
records = []
host_target = {"darwin": "macos", "linux": "linux", "win32": "windows"}[sys.platform]


def unavailable(name, contract):
    records.append(
        dict(name=name, passed=None, kind="unavailable target", reason=contract.reason)
    )
    (out / "results.json").write_text(json.dumps(records, indent=2) + "\n")


def check(name, command, marker=None, diagnostic=False):
    result = subprocess.run(
        [str(x) for x in command],
        cwd=root,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    text = result.stdout.decode(errors="replace")
    (out / (name + ".log")).write_text(text)
    ok = result.returncode != 0 if diagnostic else result.returncode == 0
    if marker is not None:
        ok = ok and marker in text
    records.append(
        dict(
            name=name,
            passed=ok,
            exit=result.returncode,
            kind="expected diagnostic" if diagnostic else "supported behavior",
        )
    )
    (out / "results.json").write_text(json.dumps(records, indent=2) + "\n")
    print(("PASS " if ok else "FAIL ") + name, flush=True)
    if not ok:
        print(text)
    return ok


for source, path, marker in [
    (
        "native_clipboard",
        library / "GetRect/tests/native-clipboard.jai",
        "native clipboard ok",
    ),
    (
        "native_cursors",
        library / "GetRect/tests/native-cursors.jai",
        "native cursor lifecycle ok",
    ),
    (
        "native_ui_cursors",
        root / "stdlib/GetRect/tests/native-ui-cursors.jai",
        "native UI cursor lifecycle ok",
    ),
]:
    relative = (
        ("stdlib/" + path.relative_to(library).as_posix())
        if path.is_relative_to(library)
        else path.relative_to(root).as_posix()
    )
    contract = target_contract(relative, host_target)
    if not contract.available:
        unavailable(source, contract)
        continue
    for level in ["O0", "O2"]:
        executable = out / (source + "-" + level)
        if check(
            source + "-" + level + "-build",
            [compiler, path] + imports + ["-" + level, "-exe", executable],
        ):
            check(source + "-" + level, [executable], marker)

if host_target == "linux":
    if not args.private_x11_display or not env.get("DISPLAY"):
        raise SystemExit(
            "Linux corrective runtime requires --private-x11-display and caller-owned DISPLAY"
        )
    for source, marker in [
        (
            "stdlib/GetRect/tests/clipboard-text-actions-linux.jai",
            "GetRect canonical Linux clipboard text actions ok",
        ),
        ("stdlib/GetRect/tests/cursor-callbacks.jai", "caller cursor callbacks ok"),
        (
            "stdlib/GetRect/tests/clipboard-unavailable.jai",
            "named pasteboard provider explicitly unavailable",
        ),
        (
            "stdlib/GetRect/tests/cursors-unavailable.jai",
            "cursor provider explicitly unavailable",
        ),
    ]:
        assert target_contract(source, host_target).available
        path = (
            library / source.removeprefix("stdlib/")
            if source.startswith("stdlib/")
            else root / source
        )
        for level in ["O0", "O2"]:
            name = path.stem + "-" + level
            executable = out / name
            if check(
                name + "-build", [compiler, path, "-" + level, "-exe", executable]
            ):
                check(name, [executable, "--private-x11-display"], marker)

for source, marker in [
    ("font_required", "em requires a font"),
    ("texture_loader_required", "texture loader callback required"),
    ("text_capacity_assert", "text input capacity exceeded"),
]:
    for level in ["O0", "O2"]:
        executable = out / (source + "-" + level)
        if check(
            source + "-" + level + "-build",
            [compiler, "stdlib/GetRect/tests/" + source.replace("_", "-") + ".jai"]
            + imports
            + ["-" + level, "-exe", executable],
        ):
            check(source + "-" + level, [executable], marker, diagnostic=True)

# This is a reproduced compiler/spec integration blocker, not implemented
# augmented-keyframe functionality. Check that the exact failure is still present.
check(
    "augmented-restriction-blocker",
    [compiler, "stdlib/Math/tests/driven/augmented-generic.jai", "-check"],
    "Augmented does not satisfy the restriction `Key`",
    diagnostic=True,
)

passed = sum(record["passed"] is True for record in records)
failed = sum(record["passed"] is False for record in records)
unavailable_count = sum(record["passed"] is None for record in records)
print(f"{passed} passed; {failed} failed; {unavailable_count} unavailable; see {out}")
sys.exit(bool(failed))
