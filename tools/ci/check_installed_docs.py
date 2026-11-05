#!/usr/bin/env python3
"""Build real docs through a relocated installed CLI, with no checkout builder override.

Example (inside the heavy-work gate):
  python3 tools/ci/check_installed_docs.py --jacket target/debug/jacket \
    --jaic target/debug/jaic --formatter target/jaifmt --work /tmp/owned-installed-docs
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

from check_doc_resolver import fixture
from package_docs_builder import ROOT, package

CACHES = frozenset({".astro", ".vite", ".vite-temp", ".cache"})


def link_or_copy(source, target):
    """Reuse explicit installed bytes without sharing writable cache directories."""
    try:
        os.link(source, target)
    except OSError:
        shutil.copy2(source, target)
    return target


def stage_dependencies(source, destination):
    """Copy an installed dependency tree, retaining only internal relative symlinks."""
    if not (source / "astro/package.json").is_file():
        raise ValueError("install the pinned builder dependencies explicitly with npm ci first")
    for path in source.rglob("*"):
        if path.is_symlink():
            target = os.readlink(path)
            if os.path.isabs(target) or not path.resolve().is_relative_to(source.resolve()):
                raise ValueError(f"dependency symlink escapes installed closure: {path.relative_to(source)}")
    shutil.copytree(source, destination, symlinks=True, copy_function=link_or_copy,
                    ignore=lambda _directory, names: [name for name in names if name in CACHES])


def installed_environment(bin_directory, preload):
    """Remove checkout overrides; Node records which installed build script actually runs."""
    environment = dict(os.environ)
    for key in ("JACKET_DOCS_BUILDER", "JACKET_DOC_EXPORT", "JACKET_DOC_OUT", "JACKET_DOC_BASE", "JACKET_DOC_EXPORT_STALE", "NODE_PATH"):
        environment.pop(key, None)
    environment["PATH"] = str(bin_directory) + os.pathsep + environment.get("PATH", "")
    environment["NODE_OPTIONS"] = "--import=" + preload.as_uri()
    return environment


def run(jacket, jaic, formatter, work):
    """Require relocated sibling selection and real root/nested/symlink site builds."""
    if work.exists():
        raise ValueError("--work must name an absent owned directory")
    if work.is_relative_to(ROOT.resolve()):
        raise ValueError("--work must be outside the source checkout")
    for binary in (jacket, jaic, formatter):
        if not binary.is_file():
            raise ValueError(f"required already-built tool is missing: {binary}")
    work.mkdir(parents=True)
    bin_directory = work / "relocated toolchain/bin"
    bin_directory.mkdir(parents=True)
    suffix = ".exe" if os.name == "nt" else ""
    tools = {}
    for name, source in (("jacket", jacket), ("jaic", jaic), ("jaifmt", formatter)):
        destination = bin_directory / (name + suffix)
        link_or_copy(source, destination)
        with source.open("rb") as stream:
            tools[name] = {"sha256": hashlib.file_digest(stream, "sha256").hexdigest()}
    builder = bin_directory / "docs-builder"
    manifest = package(ROOT / "website/docs", builder)
    stage_dependencies(ROOT / "website/docs/node_modules", builder / "node_modules")
    trace = work / "node-processes.jsonl"
    preload = work / "record-node.mjs"
    preload.write_text("import {appendFileSync} from 'node:fs';\n"
                       + "appendFileSync(" + json.dumps(str(trace))
                       + ",JSON.stringify({script:process.argv[1]??null,cwd:process.cwd()})+'\\n');\n", encoding="utf-8")
    environment = installed_environment(bin_directory, preload)
    cases = [("root", "/", bin_directory / ("jacket" + suffix)),
             ("nested", "/nested/docs/", bin_directory / ("jacket" + suffix))]
    symlink = work / ("linked-jacket" + suffix)
    symlink_status = "supported"
    try:
        symlink.symlink_to(bin_directory / ("jacket" + suffix))
    except (NotImplementedError, OSError) as error:
        symlink_status = "unavailable: " + type(error).__name__
    else:
        cases.append(("symlink", "/linked/docs/", symlink))
    ledger = []
    for label, base, executable in cases:
        project = work / ("authored " + label)
        fixture(project, "# Installed fixture\n\n[Fixture guide](guide.md#destination)\n\n[Fixture API](library/module.jai)\n")
        output, export = project / "site", project / "export"
        before = len(trace.read_text(encoding="utf-8").splitlines()) if trace.exists() else 0
        command = [str(executable), "doc", "library", "--out", str(output), "--export", str(export), "--base-url", base]
        result = subprocess.run(command, cwd=project, env=environment, text=True,
                                stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        (project / "build.log").write_text(result.stdout, encoding="utf-8")
        if result.returncode:
            raise RuntimeError(f"installed {label} build failed; see {project / 'build.log'}\n{result.stdout}")
        processes = [json.loads(line) for line in trace.read_text(encoding="utf-8").splitlines()[before:]]
        build_processes = [entry for entry in processes if entry["script"] and Path(entry["script"]).name == "build.mjs"]
        if len(build_processes) != 1 or Path(build_processes[0]["script"]).resolve() != builder / "scripts/build.mjs" or Path(build_processes[0]["cwd"]).resolve() != builder:
            raise RuntimeError(f"installed sibling builder was not exclusively selected: {build_processes}")
        for entry in processes:
            if entry["script"] and Path(entry["script"]).resolve().is_relative_to(ROOT / "website/docs"):
                raise RuntimeError("installed render executed a checkout builder script")
        site = json.loads((export / "site.json").read_text(encoding="utf-8"))["data"]
        if site["pageBase"] != base:
            raise RuntimeError("installed builder lost the requested page base")
        home = (output / "index.html").read_text(encoding="utf-8")
        if "Installed fixture" not in home or f'href="{base}guides/guide#destination"' not in home:
            raise RuntimeError("installed home page lacks actual authored content or guide destination")
        module = next(route for route in site["routes"] if route["kind"] == "module")
        rendered = (output / module["output"]).read_text(encoding="utf-8")
        if "fixture_record" not in rendered or "semantic-declaration" not in rendered:
            raise RuntimeError("installed package did not render the native formatted declaration")
        if not list((output / "_astro").glob("icons.*.svg")) or not list((output / "_astro").glob("favicon.*.svg")):
            raise RuntimeError("installed package did not emit its content-hashed UI assets")
        ledger.append({"id": label, "base": base, "status": "pass", "builder": "relocated toolchain/bin/docs-builder/scripts/build.mjs",
                       "pages": len(list(output.rglob("*.html"))), "nodeProcesses": len(processes)})
    receipt = {"schemaVersion": 1, "cases": ledger, "symlink": symlink_status,
               "packageSourceFiles": len(manifest["files"]), "tools": tools,
               "dependencySetup": "explicit reuse of this platform's already npm-ci-installed locked dependencies; independent staged directories, regular files hardlinked or copied, internal symlinks retained, caches excluded; no npm/network invocation",
               "limits": ["does not perform a fresh registry installation", "compiler standard-library resolution uses the explicitly configured test environment"]}
    (work / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    print(f"Installed docs closure: {len(ledger)} real CLI builds passed")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("jacket", "jaic", "formatter", "work"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    run(args.jacket.resolve(), args.jaic.resolve(), args.formatter.resolve(), args.work.resolve())


if __name__ == "__main__":
    main()
