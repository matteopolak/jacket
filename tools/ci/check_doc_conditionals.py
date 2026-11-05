#!/usr/bin/env python3
"""Render production API components with empty, populated and meaningful-zero content.

Example (inside the heavy-work gate):
  PYTHONPATH=tools python3 tools/ci/check_doc_conditionals.py --work /tmp/owned-doc-conditionals
"""
import argparse
import json
from pathlib import Path
import subprocess

from toolkit.repo import ROOT


def run(work):
    """Require empty optional sections to disappear without hiding authored zero text."""
    if work.exists():
        raise ValueError("--work must name an absent owned directory")
    work.mkdir(parents=True)
    builder = ROOT / "website/docs"
    source = work / "src/pages"
    source.mkdir(parents=True)
    fixtures = builder / "tests/fixtures.ts"
    page = "---\n" + f"import {{site,metadata}} from {json.dumps(str(fixtures))};\n"
    page += f"import ModulePage from {json.dumps(str(builder / 'src/components/ModulePage.astro'))};\n"
    page += f"import ApiPage from {json.dumps(str(builder / 'src/components/ApiPage.astro'))};\n"
    page += r"""export function getStaticPaths(){return ['module','empty','zero','record'].map(id=>({params:{id}}));}
const empty={source:'',spans:[]};
const declaration=(name,kind)=>({id:`api/Fixture/${name}`,name,kind,route:`api/Fixture/${name}`,source:{file:'Fixture/module.jai',line:1},sourceUrl:null,private:false,binding:false,experimental:false,since:'',platforms:[],group:'',docs:[],definitions:[],value:empty,members:[],overloads:[],metadata:metadata()});
const plain=declaration('empty','procedure');
plain.overloads=[{id:'sig',label:null,source:{file:'Fixture/module.jai',line:1},signature:{source:'()',spans:[]},docs:[],parameters:[],results:[],flags:[],platforms:[]}];
const zero={...declaration('zero','constant'),docs:[{kind:'paragraph',children:[{kind:'text',text:'0'}]}],value:{source:'0',spans:[]}};
const record={...declaration('record','struct'),definitions:[{id:'d1',kind:'struct',source:{file:'Fixture/module.jai',line:1},sourceUrl:null,platforms:[],scope:[],docs:[],code:{source:'record :: struct {\n    field: int;\n}',spans:[]},anchors:[{id:'m-field',name:'field',start:23,end:28}]}]};
const module={id:'Fixture',title:'Fixture',source:{file:'Fixture/module.jai',line:1},sourceUrl:null,docs:[],implicit:false,experimental:false,wasm:false,since:'',platforms:[],category:null,group:'',parameters:[],declarations:[plain,zero,record],dependencies:[],metadata:metadata()};
const selected=module.declarations.find(d=>d.name===Astro.params.id);
const href=target=>'/' +(target.id??target.path??'')+(target.anchor?'#'+target.anchor:'');
---
<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Conditional fixture</title></head><body><main id="fixture">{Astro.params.id==='module'?<ModulePage site={site()} module={module} href={href}/>:<ApiPage site={site()} module={module} declaration={selected} href={href}/>}</main></body></html>
"""
    (source / "[id].astro").write_text(page)
    (work / "package.json").write_text('{"type":"module","private":true}')
    (work / "node_modules").symlink_to(builder / "node_modules", target_is_directory=True)
    (work / "astro.config.mjs").write_text("export default {output:'static',trailingSlash:'never',build:{format:'file'},compressHTML:false,devToolbar:{enabled:false}};\n")
    package = json.loads((builder / "node_modules/astro/package.json").read_text())
    cli = builder / "node_modules/astro" / package["bin"]["astro"]
    subprocess.run(["node", str(cli), "build", "--root", str(work)], cwd=work, check=True)
    subprocess.run(["node", str(builder / "scripts/check-conditionals.mjs"), str(work / "dist")], cwd=builder, check=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work", required=True, type=Path)
    run(parser.parse_args().work.resolve())
