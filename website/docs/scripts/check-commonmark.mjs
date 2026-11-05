/** Independent CommonMark AST oracle and complete-case ledger. Never accept actual output as expected. */
import { Parser } from 'commonmark';
import { parse, parseFragment } from 'parse5';
import { readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const escape = value => value.replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;').replaceAll('"','&quot;');
export const fixtureURL = value => {
  const canonical=value.split(/(%[0-9a-f]{2})/iu).map(part=>/^%[0-9a-f]{2}$/iu.test(part)?part.toUpperCase():encodeURI(part)).join('');
  return `https://fixtures.invalid/${Buffer.from(canonical,'utf8').toString('hex')}`;
};
const attrs = values => Object.entries(values).filter(([,v])=>v!==undefined && v!==null).map(([k,v])=>` ${k}="${escape(String(v))}"`).join('');
function children(node, render) {let html='';for(let n=node.firstChild;n;n=n.next)html+=render(n);return html;}
function plain(node,heading=false) {
  if(node.type==='text'||node.type==='code')return node.literal;
  if(node.type==='softbreak'||node.type==='linebreak')return heading?'':'\n';
  if(node.type==='html_inline'||node.type==='html_block')return heading?'':node.literal;
  return children(node,n=>plain(n,heading));
}

/** Transform only declared documentation policies; an unknown reference AST node is a failure. */
export function reference(markdown) {
  const root=new Parser().parse(markdown), transformations=new Set(),used=new Set(),codeSources=[];
  function render(node) {
    const inner=()=>children(node,render);
    switch(node.type) {
      case 'document':return inner();
      case 'text':return escape(node.literal);
      case 'softbreak':return '\n';
      case 'linebreak':return '<br>';
      case 'paragraph':return node.parent.type==='item' && node.parent.parent.listTight?inner():`<p>${inner()}</p>`;
      case 'emph':return `<em>${inner()}</em>`;
      case 'strong':return `<strong>${inner()}</strong>`;
      case 'code':return `<code>${escape(node.literal)}</code>`;
      case 'thematic_break':return '<hr>';
      case 'block_quote':return `<blockquote>${inner()}</blockquote>`;
      case 'heading': {
        transformations.add('heading-identifiers');
        const base=plain(node,true).trim().toLowerCase().replaceAll(' ','-').replace(/[^\p{L}\p{N}_-]/gu,'');
        let id=base,n=1;while(used.has(id))id=`${base}-${n++}`;used.add(id);
        return `<h${node.level}${attrs({id})}>${inner()}</h${node.level}>`;
      }
      case 'html_inline':transformations.add('literal-raw-html');return escape(node.literal);
      case 'html_block': {
        transformations.add('literal-raw-html');
        const endLine=node.sourcepos[1][0];
        const sourceLines=markdown.match(/[^\n]*(?:\n|$)/gu);
        const newline=sourceLines[endLine-1]?.endsWith('\n')?'\n':'';
        return escape(node.literal+newline);
      }
      case 'code_block': {
        transformations.add('code-presentation');codeSources.push(node.literal);
        const lang=(node.info??'').split(/[ ,]/u)[0];
        return `<pre class="code${['jai','diagnostic'].includes(lang)?'':' wrap'}"><code>${escape(node.literal.trimEnd())}</code></pre>`;
      }
      case 'link':transformations.add('fixture-url-map');return `<a${attrs({href:fixtureURL(node.destination),title:node.title||undefined})}>${inner()}</a>`;
      case 'image':transformations.add('fixture-url-map');transformations.add('image-alternative');return `<img${attrs({src:fixtureURL(node.destination),title:node.title||undefined,alt:plain(node)})}>`;
      case 'list':return node.listType==='bullet'?`<ul>${inner()}</ul>`:`<ol${attrs({start:node.listStart===1?undefined:node.listStart})}>${inner()}</ol>`;
      case 'item':return `<li>${inner()}</li>`;
      default:throw new Error(`unreviewed reference AST node ${node.type}`);
    }
  }
  return {html:render(root),transformations:[...transformations].sort(),codeSources};
}


/** Published extension HTML is an independent fixture; only declared presentation attributes change. */
export function extensionReference(html) {
  const root=parseFragment(html),transformations=new Set(['enabled-markdown-extensions']),used=new Set();
  function visit(node) {
    if(node.attrs)for(const attribute of node.attrs)if(attribute.name==='href'||attribute.name==='src') {
      transformations.add('fixture-url-map');attribute.value=fixtureURL(attribute.value);
    }
    if(/^h[1-6]$/u.test(node.tagName??'')) {
      transformations.add('heading-identifiers');
      const content=n=>n.nodeName==='#text'?n.value:(n.childNodes??[]).map(content).join('');
      const base=content(node).trim().toLowerCase().replaceAll(' ','-').replace(/[^\p{L}\p{N}_-]/gu,'');
      let id=base,n=1;while(used.has(id))id=`${base}-${n++}`;used.add(id);node.attrs.push({name:'id',value:id});
    }
    if(node.tagName==='pre')throw new Error('unreviewed extension code presentation');
    if(node.tagName==='script'||node.tagName==='style')throw new Error('unreviewed extension raw HTML');
    for(const child of node.childNodes??[])visit(child);
  }
  visit(root);
  return {expectedTree:tree(root),transformations:[...transformations].sort(),codeSources:[]};
}

/** Preserve all inline/code whitespace, coalesce adjacent text, ignore serialization-only block gaps. */
export function tree(root) {
  const blocks=new Set(['#document-fragment','main','ul','ol','li','blockquote','table','thead','tbody','tr']);
  function canonical(node,parent) {
    if(node.nodeName==='#comment')return undefined;
    if(node.nodeName==='#text')return /^\s*$/u.test(node.value)&&blocks.has(parent)?undefined:node.value;
    const result=[];
    for(const child of node.childNodes??[]) {
      const value=canonical(child,node.tagName??node.nodeName);
      if(value===undefined)continue;
      if(typeof value==='string' && typeof result.at(-1)==='string')result[result.length-1]+=value;else result.push(value);
    }
    return {tag:node.tagName??node.nodeName,attrs:(node.attrs??[]).map(a=>[a.name,a.value]).sort((a,b)=>a[0].localeCompare(b[0])),children:result};
  }
  return canonical(root,'');
}
export const htmlTree = html => tree(parseFragment(html));
function find(node,predicate) {if(predicate(node))return node;for(const child of node.childNodes??[]){const found=find(child,predicate);if(found)return found;}return undefined;}
export async function check(casesPath,producerPath,output,ledgerPath) {
  const cases=JSON.parse(await readFile(casesPath,'utf8')),producer=JSON.parse(await readFile(producerPath,'utf8'));
  const ids=producer.map(row=>row.id);
  if(new Set(ids).size!==cases.length || ids.length!==cases.length || cases.some(row=>!ids.includes(row.example)))throw new Error('incomplete/duplicate producer inventory');
  const records=new Map(producer.map(row=>[row.id,row])),ledger=[];
  const reviewed=JSON.parse(await readFile(new URL('../../../tools/ci/fixtures/commonmark/transformations.json',import.meta.url),'utf8'));
  const actualGroups={};
  for(const item of cases) {
    const errors=[],record=records.get(item.example),expected=item.profile==='extensions'?extensionReference(item.html):reference(item.markdown);
    for(const name of expected.transformations)(actualGroups[name]??=[]).push(item.example);
    const actualCode=[];
    function codes(value) {if(!value||typeof value!=='object')return;if(value.kind==='code_block')actualCode.push(value.source.source);for(const child of Object.values(value))if(Array.isArray(child))child.forEach(codes);else if(child&&typeof child==='object')codes(child);}
    codes(record.nodes);
    if(JSON.stringify(expected.codeSources)!==JSON.stringify(actualCode))errors.push('raw-code-source-preservation');
    if(JSON.stringify(htmlTree(item.html))!==JSON.stringify(htmlTree(item.profile==='extensions'?record.effectiveHtml:record.upstreamHtml)))errors.push('base-provider-tree');
    const page=parse(await readFile(resolve(output,`${item.example}.html`),'utf8'));
    const main=find(page,node=>node.tagName==='main'&&node.attrs.some(a=>a.name==='id'&&a.value==='fixture'));
    if(!main)errors.push('missing-production-content');
    else {
      const actual=tree({nodeName:'#document-fragment',childNodes:main.childNodes});
      if(JSON.stringify((expected.expectedTree??htmlTree(expected.html)))!==JSON.stringify(actual))errors.push('transformed-document-tree');
    }
    ledger.push({id:item.example,section:item.section,status:errors.length?'fail':'pass',transformations:expected.transformations,errors});
  }
  for(const name of new Set([...Object.keys(reviewed.groups),...Object.keys(actualGroups)]))if(JSON.stringify(reviewed.groups[name])!==JSON.stringify(actualGroups[name]))throw new Error(`unreviewed transformation group ${name}`);
  await writeFile(ledgerPath,JSON.stringify(ledger,null,2)+'\n');
  const failures=ledger.filter(row=>row.status!=='pass');
  if(failures.length)throw new Error(`${failures.length}/${cases.length} CommonMark cases failed: ${failures.map(row=>`${row.id}:${row.errors.join(',')}`).join(' ')}`);
  return ledger;
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href)await check(...process.argv.slice(2));
