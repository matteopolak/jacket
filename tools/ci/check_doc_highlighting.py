#!/usr/bin/env python3
"""Render production Document fences and verify escaped, byte-preserving language highlighting.

Run within the shared heavy-job gate:
  PYTHONPATH=tools python3 tools/ci/check_doc_highlighting.py --work /tmp/owned-doc-highlighting
"""
import argparse
import json
from pathlib import Path
import subprocess

from toolkit.repo import ROOT


def run(work):
    """Use real static Astro rendering, not a string imitation of the production component."""
    if work.exists():
        raise ValueError("--work must name an absent owned directory")
    builder = ROOT / "website/docs"
    source = work / "src/pages"
    source.mkdir(parents=True)
    cases = []
    for language in ["sh", "bash", "shell", "zsh", "console", "powershell", "ps1", "pwsh", "toml"]:
        code = 'name = "<script>雪 & literal</script>"\n# comment\n' if language == "toml" else 'echo "<script>雪 & literal</script>"\n# comment\n'
        cases.append({"language": language, "source": code})
    cases.append({"language": "unknown-fixture-language", "source": '<script>alert("literal")</script>\n'})
    data = work / "cases.json"
    data.write_text(json.dumps(cases))
    (source / "index.astro").write_text("---\n"
        + f"import Document from {json.dumps(str(builder / 'src/components/Document.astro'))};\n"
        + f"import cases from {json.dumps(str(data))};\n---\n"
        + '<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Code fence fixture</title></head><body>'
        + '{cases.map(item=><section id={item.language}><Document nodes={[{kind:"code_block",language:item.language,source:{source:item.source,spans:[]}}]} href={()=>{throw new Error("unexpected fixture link")}}/></section>)}'
        + '</body></html>\n')
    (work / "package.json").write_text('{"type":"module","private":true}')
    (work / "node_modules").symlink_to(builder / "node_modules", target_is_directory=True)
    (work / "astro.config.mjs").write_text("export default {output:'static',compressHTML:false,devToolbar:{enabled:false}};\n")
    package = json.loads((builder / "node_modules/astro/package.json").read_text())
    cli = builder / "node_modules/astro" / package["bin"]["astro"]
    subprocess.run(["node", str(cli), "build", "--root", str(work)], cwd=work, check=True)
    subprocess.run(["node", str(builder / "scripts/check-code-fences.mjs"), str(data), str(work / "dist/index.html")], cwd=builder, check=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work", type=Path, required=True)
    run(parser.parse_args().work.resolve())
