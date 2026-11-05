#!/usr/bin/env python3
"""Build selected real production routes with the complete catalogue for navigation regression.

This is a test fixture, not a publication build. Run through the heavy gate.
Example: PYTHONPATH=tools python3 tools/ci/build_doc_navigation_fixture.py --export /tmp/export --assets /tmp/site --out /tmp/absent-fixture
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess

from toolkit.repo import ROOT
ROUTES = ("api/Basic", "api/Hash_Table", "api/Bucket_Array", "api/Memory", "guides/testing", "tools/compiling", "lint-rules/self_assignment", "lint-rules/duplicate_condition", "api/Hash_Table/Table", "api/Hash_Table/init")


def build(export, assets, work):
    """Use actual Page/Shell, full admitted data and actual public assets; select only output paths."""
    if work.exists():
        raise ValueError("--out must name an absent owned fixture directory")
    site = json.loads((export / "site.json").read_text())["data"]
    ids = {route["id"] for route in site["routes"]}
    if not set(ROUTES).issubset(ids):
        raise ValueError("export lacks required real navigation fixture routes")
    builder = ROOT / "website/docs"
    pages = work / "src/pages"
    pages.mkdir(parents=True)
    (work / "package.json").write_text('{"type":"module","private":true}')
    (work / "node_modules").symlink_to(builder / "node_modules", target_is_directory=True)
    imports = "---\n" + f"import Page from {json.dumps(str(builder / 'src/components/Page.astro'))};\n"
    page = imports + "export function getStaticPaths(){return " + json.dumps(ROUTES) + ".map(id=>({params:{route:id},props:{id}}));}\nconst {id}=Astro.props;\n---\n<Page id={id}/>\n"
    (pages / "[...route].astro").write_text(page)
    (pages / "index.astro").write_text(imports + '---\n<Page id=""/>\n')
    (work / "astro.config.mjs").write_text("export default {output:'static',prefetch:{prefetchAll:false,defaultStrategy:'hover'},trailingSlash:'never',build:{format:'file'},compressHTML:false,devToolbar:{enabled:false}};\n")
    public = work / "public"
    public.mkdir()
    for name in ("assets", "navigation", "runtime", "examples"):
        if (assets / name).is_dir():
            shutil.copytree(assets / name, public / name, copy_function=os.link)
    for name in ("search-index.json", "versions.json"):
        if (assets / name).is_file():
            os.link(assets / name, public / name)
    package = json.loads((builder / "node_modules/astro/package.json").read_text())
    env = os.environ.copy()
    env["JACKET_DOC_EXPORT"] = str(export)
    env["JACKET_DOC_BASE"] = site["pageBase"]
    subprocess.run(["node", str(builder / "node_modules/astro" / package["bin"]["astro"]), "build", "--root", str(work)], cwd=work, env=env, check=True)
    (work / "fixture.json").write_text(json.dumps({"purpose":"Selected real-route regression fixture; not a complete publication", "routes":["", *ROUTES], "export":str(export), "assets":str(assets)}, indent=2) + "\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--export", required=True, type=Path)
    parser.add_argument("--assets", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    build(args.export.resolve(), args.assets.resolve(), args.out.resolve())
