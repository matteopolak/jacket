#!/usr/bin/env python3
"""Check literal Rust includes against public checkout inputs before compiling."""
import re
import subprocess
from pathlib import Path

INCLUDE = re.compile(r'include_(?:str|bytes)!\s*\(\s*"([^"\n]+)"\s*,?\s*\)')


def inspect_includes(repository, public_files):
    repository = repository.resolve()
    public = {path.resolve() for path in public_files if path.is_file()}
    errors = []
    count = 0
    for source in sorted(public):
        if source.suffix != ".rs":
            continue
        if not source.is_relative_to(repository):
            errors.append(f"Rust source is outside the checkout: {source}")
            continue
        text = source.read_text()
        for match in INCLUDE.finditer(text):
            count += 1
            target = (source.parent / match.group(1)).resolve()
            line = text.count("\n", 0, match.start()) + 1
            location = f"{source.relative_to(repository)}:{line}"
            if not target.is_relative_to(repository):
                errors.append(f"{location}: include escapes the public checkout")
                continue
            relative = target.relative_to(repository)
            if relative.parts[0] in {"reference", "vendor", "artifacts", ".git"} or relative.is_relative_to("corpus/upstream"):
                errors.append(f"{location}: include uses excluded input {relative}")
            elif target not in public or not target.is_file():
                errors.append(f"{location}: include is absent from public checkout inputs: {relative}")
    return count, errors


def public_files(repository):
    output = subprocess.check_output(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=repository,
    )
    return [repository / path.decode() for path in output.split(b"\0") if path]


def main():
    repository = Path(__file__).resolve().parents[1]
    count, errors = inspect_includes(repository, public_files(repository))
    if errors:
        raise SystemExit("\n".join(errors))
    print(f"Public checkout literal Rust include check passed ({count} includes)")


if __name__ == "__main__":
    main()
