"""Strict public-context lint of the unchanged portable-data source inventory."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
PROFILES = tuple(
    (target_os, cpu)
    for target_os in ("macos", "linux", "windows")
    for cpu in ("x64", "arm64")
) + (("wasm", None),)
PUBLIC_MODULES = (
    "Ico_File",
    "Icon",
    "Simple_Package",
    "Subtitles",
    "Windows_Resources",
)
WINDOWS_FIXTURES = frozenset(
    ROOT / "stdlib/tests" / f"portable-data-{name}-windows.jai"
    for name in ("resource", "manifest", "console", "discovery")
)
# These private providers are reached through Windows_Resources on Windows.
# The portable library-owned fixtures also exercise their pure wire operations.
WINDOWS_PUBLIC_PROVIDERS = frozenset(
    ROOT / "stdlib/internal/resource-wire" / name
    for name in ("manifest.jai", "tool-versions.jai", "icon-directory.jai")
)


def public_context_source():
    return (
        "".join(
            f'Module_{index} :: #import "{name}";\n'
            for index, name in enumerate(PUBLIC_MODULES)
        )
        + "\nmain :: () {}\n"
    )


def owned_files(roots):
    result = set()
    for root in roots:
        root = root.resolve(strict=True)
        if not root.is_relative_to(ROOT):
            raise ValueError(f"source root leaves this checkout: {root}")
        result.update(root.rglob("*.jai") if root.is_dir() else [root])
    if not result:
        raise ValueError("portable-data lint source inventory is empty")
    return sorted(result)


def profile_sources(owned, target_os):
    library, fixtures = [], []
    for path in owned:
        if target_os != "windows" and (
            path in WINDOWS_FIXTURES
            or path in WINDOWS_PUBLIC_PROVIDERS
            or path.is_relative_to(ROOT / "stdlib/internal/windows-resources")
        ):
            continue
        is_library = path.is_relative_to(ROOT / "stdlib") and not path.is_relative_to(
            ROOT / "stdlib/tests"
        )
        (library if is_library else fixtures).append(path)
    return library, fixtures


def canonical(path):
    """One spelling of a file for comparing sets of paths: Windows reports them with a verbatim
    prefix, either slash and any case."""
    text = str(path)
    for prefix in ("\\\\?\\", "//?/"):
        text = text.removeprefix(prefix)
    return Path(os.path.normcase(os.path.realpath(text)))


def complete_checked_paths(report, checked):
    visited = {
        canonical(path)
        for context in report["contexts"]
        if context["complete"]
        for path in context["visited_files"]
    }
    # A file imported but omitted from lint reports does not establish coverage.
    return visited & checked


def require_coverage(owned, covered):
    missing = {canonical(path) for path in owned} - covered
    if missing:
        raise RuntimeError(
            "owned sources not completely linted: "
            + ", ".join(map(str, sorted(missing)))
        )


def lint_owned(linter, stdlib, output, roots):
    if stdlib.resolve(strict=True) != (ROOT / "stdlib").resolve():
        raise ValueError("lint the library owning this runner; selected stdlib differs")
    owned = owned_files(roots)
    output.mkdir(parents=True, exist_ok=True)
    context = output / "public-modules.jai"
    context.write_text(public_context_source())
    negative = output / "private-import-negative.jai"
    negative.write_text(
        'Private :: #import "internal/resource-wire/manifest";\n'
        "main :: () { value := Private.generate_manifest(); }\n"
    )
    env = dict(os.environ, JAIC_STDLIB=str(stdlib.resolve()))

    def digest(path):
        return hashlib.sha256(path.read_bytes()).hexdigest()

    before = {str(path): digest(path) for path in owned}
    before[str(linter)] = digest(linter)
    commands, covered = [], set()
    for target_os, cpu in PROFILES:
        label = target_os + (f"-{cpu}" if cpu else "")
        flags = ["-os", target_os] + (["-cpu", cpu] if cpu else [])
        prefix = [
            str(linter),
            "-D",
            "warnings",
            "--config",
            str(ROOT / "jailint.toml"),
            *flags,
        ]
        library, fixtures = profile_sources(owned, target_os)
        groups = [("library", context, library)] + [
            ("fixture-" + path.stem, path, [path]) for path in fixtures
        ]
        for name, enclosing, paths in groups:
            manifest = output / f"{label}-{name}.json"
            checked_file = manifest.with_suffix(".files")
            argv = [
                *prefix,
                "--context",
                str(enclosing),
                "--manifest",
                str(manifest),
                "--checked-files",
                str(checked_file),
                *map(str, paths),
            ]
            result = subprocess.run(
                argv, cwd=ROOT, env=env, capture_output=True, text=True, timeout=120
            )
            report = json.loads(manifest.read_text()) if manifest.is_file() else None
            complete = (
                report is not None
                and not report["incomplete"]
                and not report["unvisited_files"]
                and bool(report["contexts"])
                and all(context["complete"] for context in report["contexts"])
            )
            passed = (
                result.returncode == 0
                and not result.stdout
                and not result.stderr
                and complete
            )
            commands.append(
                dict(
                    argv=argv,
                    exit=result.returncode,
                    stdout=result.stdout,
                    stderr=result.stderr,
                    passed=passed,
                    kind="positive",
                    profile=label,
                )
            )
            if report is not None and checked_file.is_file():
                checked = {
                    canonical(path)
                    for path in checked_file.read_text().splitlines()
                    if path
                }
                covered.update(complete_checked_paths(report, checked))
            print(label, name, "PASS" if passed else "FAIL", flush=True)
            (output / "commands.json").write_text(json.dumps(commands, indent=2) + "\n")
        argv = [*prefix, "--context", str(negative), str(negative)]
        result = subprocess.run(
            argv, cwd=ROOT, env=env, capture_output=True, text=True, timeout=30
        )
        passed = (
            result.returncode == 1
            and not result.stdout
            and "internal sources are private" in result.stderr
        )
        commands.append(
            dict(
                argv=argv,
                exit=result.returncode,
                stdout=result.stdout,
                stderr=result.stderr,
                passed=passed,
                kind="private-guard",
                profile=label,
            )
        )
        print(label, "private-guard", "PASS" if passed else "FAIL", flush=True)
    missing = sorted({canonical(path) for path in owned} - covered)
    changed = [path for path, value in before.items() if digest(Path(path)) != value]
    passed = (
        not missing and not changed and all(command["passed"] for command in commands)
    )
    document = dict(
        passed=passed,
        owned_sha256=before,
        owned_count=len(owned),
        missing=list(map(str, missing)),
        changed=changed,
        commands=commands,
    )
    (output / "results.json").write_text(json.dumps(document, indent=2) + "\n")
    require_coverage(owned, covered)
    if not passed:
        raise RuntimeError(
            f"portable-data contextual lint failed; inspect {output / 'results.json'}"
        )
    return document


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--linter", type=Path, required=True)
    parser.add_argument("--stdlib", type=Path, required=True)
    parser.add_argument("--output-root", type=Path, required=True)
    parser.add_argument("source_roots", type=Path, nargs="+")
    args = parser.parse_args()
    lint_owned(
        args.linter.resolve(strict=True),
        args.stdlib,
        args.output_root.resolve(),
        args.source_roots,
    )


if __name__ == "__main__":
    main()
