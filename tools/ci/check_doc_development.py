#!/usr/bin/env python3
"""Exercise the actual owned foreground Astro development publication.

Run inside the heavy gate, with PYTHONPATH=tools and the pinned Node on PATH:
  python3 tools/ci/check_doc_development.py --work /tmp/owned-doc-dev \
    --root-export /owned/root/export --root-site /owned/root/site \
    --nested-export /owned/nested/export --nested-site /owned/nested/site \
    --full-export /owned/full/export --full-site /owned/full/site

Without supplied tiny exports, --jacket and --formatter generate authored fixtures
through the real CLI. --tiny-only explicitly limits coverage to root/nested setup.
"""
import argparse
import hashlib
from html.parser import HTMLParser
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import time
import urllib.error
import urllib.request
from urllib.parse import urljoin

from check_doc_resolver import fixture
from check_installed_docs import stage_dependencies
from package_docs_builder import ROOT, package

ANSI = re.compile(r"\x1b\[[0-?]*[ -/]*[@-~]")


class PageResources(HTMLParser):
    """Collect framework-owned script/stylesheet entry URLs from actual dev HTML."""

    def __init__(self):
        super().__init__()
        self.resources = []

    def handle_starttag(self, tag, attrs):
        attributes = dict(attrs)
        if tag == "script" and attributes.get("src"):
            self.resources.append((attributes["src"], {"text/javascript", "application/javascript"}))
        if tag == "link" and attributes.get("rel") == "stylesheet" and attributes.get("href"):
            self.resources.append((attributes["href"], {"text/css"}))


def execute(command, cwd, env, log):
    """Retain actual probe exits, e.g. execute([node, probe, url], root, env, log)."""
    with log.open("w", encoding="utf-8") as stream:
        result = subprocess.run(command, cwd=cwd, env=env, stdout=stream, stderr=subprocess.STDOUT)
    if result.returncode:
        raise RuntimeError(f"command exited {result.returncode}; see {log}")


def environment(export=None):
    """Discard stale checkout overrides, preserving only explicit source/tool inputs."""
    env = dict(os.environ)
    for key in ("JACKET_DOC_BASE", "JACKET_DOC_OUT", "JACKET_DOC_EXPORT_STALE", "NODE_OPTIONS", "NODE_PATH"):
        env.pop(key, None)
    if export is not None:
        env["JACKET_DOC_EXPORT"] = str(export)
    return env


def start(builder, export, log):
    """Start the declared CLI in supported foreground mode; port zero is OS-assigned."""
    metadata = json.loads((builder / "node_modules/astro/package.json").read_text())
    cli = builder / "node_modules/astro" / metadata["bin"]["astro"]
    node = shutil.which("node")
    if not node:
        raise RuntimeError("install pinned Node explicitly before this check")
    stream = log.open("w", encoding="utf-8")
    child = subprocess.Popen([node, str(cli), "dev", "--ignore-lock", "--host", "127.0.0.1", "--port", "0"],
                             cwd=builder, env=environment(export), stdout=stream, stderr=subprocess.STDOUT,
                             start_new_session=os.name != "nt")
    try:
        deadline = time.monotonic() + 90
        while time.monotonic() < deadline:
            if child.poll() is not None:
                raise RuntimeError(f"foreground Astro launcher exited {child.returncode} before readiness; see {log}")
            text = ANSI.sub("", log.read_text(encoding="utf-8"))
            match = re.search(r"http://127\.0\.0\.1:(\d+)", text)
            if match:
                return child, stream, f"http://127.0.0.1:{match[1]}/"
            time.sleep(0.05)
        raise RuntimeError(f"owned Astro did not report a listening URL; see {log}")
    except BaseException:
        stop(child, stream)
        raise


def stop(child, stream, termination=None):
    """Signal and reap only the process/group created by this driver, never a lockfile PID."""
    forced = False
    if child.poll() is None:
        child.send_signal(termination if termination is not None else (signal.SIGINT if os.name != "nt" else signal.SIGTERM))
        try:
            child.wait(timeout=15)
        except subprocess.TimeoutExpired:
            forced = True
            if os.name != "nt":
                os.killpg(child.pid, signal.SIGKILL)
            else:
                child.kill()
            child.wait()
    stream.close()
    return forced


def request(url):
    """Read one actual HTTP response; policy failures are not retried."""
    try:
        response = urllib.request.urlopen(url, timeout=30)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        return response.status, response.headers.get_content_type(), response.read()


def check_case(label, export, oracle, builder, work, browsers):
    """Check exact generated public files, HTTP data/MIME, process ownership and shutdown."""
    case = work / label
    case.mkdir()
    manifest = json.loads((export / "manifest.json").read_text())["data"]
    site = json.loads((export / "site.json").read_text())["data"]
    expected = {path.relative_to(oracle).as_posix(): "application/json" for path in oracle.rglob("*.json")}
    expected.update({asset["path"]: asset["mediaType"] for asset in manifest["assets"]})
    if not {"search-index.json", "versions.json"}.issubset(expected) or not any(name.startswith("navigation/") for name in expected):
        raise RuntimeError("production oracle is not a complete derived publication")
    before = set((builder / ".astro").glob("jacket-doc-public-*"))
    child, stream, origin = start(builder, export, case / "server.log")
    base = origin.rstrip("/") + site["pageBase"]
    receipt = {"case": label, "base": base, "launcherPID": child.pid, "foregroundAliveAtStart": child.poll() is None,
               "http": [], "publicationFiles": len(expected), "browsers": browsers,
               "exportManifestSHA256": hashlib.sha256((export / "manifest.json").read_bytes()).hexdigest(),
               "builderManifestSHA256": hashlib.sha256((builder / "builder-manifest.json").read_bytes()).hexdigest()}
    try:
        status, mime, home = request(base)
        receipt["home"] = {"status": status, "mime": mime, "bytes": len(home), "sha256": hashlib.sha256(home).hexdigest()}
        if status != 200 or mime != "text/html" or b"<main" not in home:
            raise RuntimeError("actual Astro development home did not render")
        page = PageResources()
        page.feed(home.decode("utf-8"))
        receipt["frameworkResources"] = []
        if not page.resources:
            raise RuntimeError("development page has no framework scripts/styles")
        for resource, allowed_mime in page.resources:
            url = urljoin(base, resource)
            status, mime, data = request(url)
            receipt["frameworkResources"].append({"url": url, "status": status, "mime": mime, "bytes": len(data)})
            if status != 200 or mime not in allowed_mime:
                raise RuntimeError(f"development framework resource failed: {url} {status} {mime}")
        owned = set((builder / ".astro").glob("jacket-doc-public-*")) - before
        if len(owned) != 1:
            raise RuntimeError(f"expected one owned development public directory, got {len(owned)}")
        public = next(iter(owned))
        actual = {path.relative_to(public).as_posix() for path in public.rglob("*") if path.is_file()}
        if actual != set(expected):
            raise RuntimeError(f"generated publication differs: missing={sorted(set(expected)-actual)}, extra={sorted(actual-set(expected))}")
        for name, expected_mime in sorted(expected.items()):
            status, mime, data = request(base + name)
            wanted = (oracle / name).read_bytes()
            entry = {"path": name, "status": status, "mime": mime, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}
            receipt["http"].append(entry)
            if status != 200 or mime != expected_mime or data != wanted:
                raise RuntimeError(f"actual development publication mismatch: {entry}, expected MIME {expected_mime}")
        if manifest["assets"]:
            asset = manifest["assets"][0]
            with urllib.request.urlopen(urllib.request.Request(base + asset["path"], method="HEAD"), timeout=30) as response:
                receipt["assetHead"] = {"path": asset["path"], "status": response.status,
                                        "mime": response.headers.get_content_type(),
                                        "contentLength": response.headers.get("Content-Length"),
                                        "bodyBytes": len(response.read())}
            if receipt["assetHead"] != {"path": asset["path"], "status": 200, "mime": asset["mediaType"],
                                        "contentLength": str(asset["bytes"]), "bodyBytes": 0}:
                raise RuntimeError("declared development asset HEAD differs from its admitted metadata")
        if child.poll() is not None:
            raise RuntimeError("launcher exited while another process kept serving development")
        receipt["foregroundAliveAfterRequests"] = True
        if browsers:
            node = shutil.which("node")
            execute([node, str(ROOT / "tools/dev/doc_spa.mjs"), base, "--dev"], ROOT, environment(), case / "spa.log")
            execute([node, str(ROOT / "tools/dev/doc_runtime_smoke.mjs"), base, str(case / "runtime-search.json"), "--dev"], ROOT, environment(), case / "runtime-search.log")
            if child.poll() is not None:
                raise RuntimeError("foreground launcher did not survive actual browser/worker interactions")
        receipt["status"] = "pass"
    except BaseException as error:
        receipt["status"] = "fail"
        receipt["error"] = str(error)
        raise
    finally:
        termination = signal.SIGTERM if label == "nested" or os.name == "nt" else signal.SIGINT
        receipt["shutdownSignal"] = signal.Signals(termination).name
        receipt["shutdownForced"] = stop(child, stream, termination)
        receipt["shutdownExit"] = child.returncode
        remaining = set((builder / ".astro").glob("jacket-doc-public-*")) - before
        receipt["publicDirectoriesAfterStop"] = [path.name for path in sorted(remaining)]
        receipt["portClosed"] = False
        try:
            request(base)
        except (OSError, urllib.error.URLError):
            receipt["portClosed"] = True
        if remaining or not receipt["portClosed"] or receipt["shutdownForced"]:
            receipt["status"] = "fail"
            receipt["cleanupError"] = "owned Astro public directory or listener survived shutdown"
        (case / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    if receipt["status"] != "pass":
        raise RuntimeError(f"{label} shutdown/publication failure; see {case / 'receipt.json'}")
    return receipt


def run(args):
    """Stage real sources/dependencies and exercise explicit root/nested/full export inputs."""
    work = args.work.resolve()
    if work.exists() or work.is_relative_to(ROOT.resolve()):
        raise ValueError("--work must be absent and outside the checkout")
    work.mkdir(parents=True)
    builder = work / "builder"
    package(ROOT / "website/docs", builder)
    stage_dependencies(ROOT / "website/docs/node_modules", builder / "node_modules")
    cases = []
    for label, base in (("root", "/"), ("nested", "/nested/docs/")):
        export, oracle = getattr(args, label + "_export"), getattr(args, label + "_site")
        if export is None:
            if args.jacket is None or args.formatter is None:
                raise ValueError("provide root/nested exports+sites or matching --jacket and --formatter")
            project = work / (label + "-project")
            fixture(project, "# Development fixture\n\n[Guide](guide.md#destination)\n")
            export, oracle = project / "export", project / "site"
            env = environment()
            env["JACKET_DOCS_BUILDER"] = str(builder)
            env["PATH"] = str(args.formatter.resolve().parent) + os.pathsep + env.get("PATH", "")
            execute([str(args.jacket.resolve()), "doc", "library", "--export", str(export), "--out", str(oracle), "--base-url", base], project, env, project / "production.log")
        elif oracle is None:
            raise ValueError("each supplied export requires its production site oracle")
        cases.append((label, export.resolve(), oracle.resolve(), False))
    if not args.tiny_only:
        if args.full_export is None or args.full_site is None:
            raise ValueError("full acceptance requires --full-export/--full-site; --tiny-only explicitly limits coverage")
        cases.append(("full-runtime", args.full_export.resolve(), args.full_site.resolve(), True))
    ledger = []
    for label, export, oracle, browsers in cases:
        try:
            ledger.append(check_case(label, export, oracle, builder, work, browsers))
        except BaseException as error:
            failed = work / label / "receipt.json"
            if failed.is_file():
                ledger.append(json.loads(failed.read_text()))
            (work / "receipt.json").write_text(json.dumps({"status": "fail", "error": str(error), "cases": ledger}, indent=2) + "\n")
            raise
        (work / "receipt.json").write_text(json.dumps({"status": "running", "cases": ledger}, indent=2) + "\n")
        print(f"Actual Astro dev {label}: HTTP publication, foreground ownership and cleanup passed", flush=True)
    (work / "receipt.json").write_text(json.dumps({"status": "pass", "cases": ledger, "dependencySetup": "explicit reuse of installed locked dependencies; no npm/network install"}, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work", required=True, type=Path)
    for name in ("root-export", "root-site", "nested-export", "nested-site", "full-export", "full-site", "jacket", "formatter"):
        parser.add_argument("--" + name, type=Path)
    parser.add_argument("--tiny-only", action="store_true")
    run(parser.parse_args())


if __name__ == "__main__":
    main()
