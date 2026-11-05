/** Standards-aware, whole-tree HTML and local fragment validation, independent of components. */
import { parse } from 'parse5';
import {HtmlValidate} from 'html-validate';

const standards=new HtmlValidate({root:true,extends:['html-validate:standard']});
import { readFile, readdir } from 'node:fs/promises';
import { resolve, relative } from 'node:path';
import { pathToFileURL } from 'node:url';

/** The optional branding facts come from admitted configuration, never template constants.
 * @param {{name?:string,homeTitle?:string,socialName?:string}} [branding]
 */
export async function checkSite(root,base,branding={}) {
  const files = new Set();
  const pages = new Map();
  const errors = [];
  let errorCount=0;
  const record=message=>{errorCount++;if(errors.length<100)errors.push(message);};
  async function inventory(directory) {
    for(const entry of await readdir(directory,{withFileTypes:true})) {
      const path=resolve(directory,entry.name);
      if(entry.isSymbolicLink())throw new Error(`symlink in built output ${path}`);
      if(entry.isDirectory())await inventory(path);else if(entry.isFile())files.add(relative(root,path).replaceAll('\\','/'));
    }
  }
  await inventory(root);
  for(const file of files) {
    if(!file.endsWith('.html'))continue;
    const ids=new Set(),links=[];
    let title,wordmark,ogTitle,ogSite;
    const source=await readFile(resolve(root,file),'utf8');
    const report=await standards.validateString(source,file);
    for(const result of report.results)for(const message of result.messages)if(message.severity===2)record(`${file}:${message.line}:${message.column}: ${message.ruleId}: ${message.message}`);
    const document=parse(source,{onParseError:error=>record(`${file}: HTML ${error.code}`)});
    const content=node=>node.nodeName==='#text'?node.value:(node.childNodes??[]).map(content).join('');
    function walk(node) {
      if(branding.name){
        if(node.tagName==='title')title=content(node);
        if(node.attrs?.some(attribute=>attribute.name==='class'&&attribute.value.split(/\s+/u).includes('wordmark')))wordmark=content(node);
        if(node.tagName==='meta'){
          const property=node.attrs?.find(attribute=>attribute.name==='property')?.value;
          const value=node.attrs?.find(attribute=>attribute.name==='content')?.value;
          if(property==='og:title')ogTitle=value;
          if(property==='og:site_name')ogSite=value;
        }
      }
      if(node.attrs)for(const attribute of node.attrs) {
        if(attribute.name==='id') {if(ids.has(attribute.value))record(`${file}: duplicate id ${attribute.value}`);ids.add(attribute.value);}
        if(attribute.name==='href'||attribute.name==='src')links.push(attribute.value);
      }
      for(const child of node.childNodes??[])walk(child);
      if(node.content)walk(node.content);
    }
    walk(document);
    if(branding.name){
      const suffix=` — ${branding.name}`;
      if(file==='index.html'?title!==(branding.homeTitle??branding.name):!title?.endsWith(suffix)||title.length===suffix.length)record(`${file}: branding: page title does not use the configured site name`);
      if(wordmark!==branding.name)record(`${file}: branding: wordmark does not use the configured site name`);
      if(ogTitle!==title)record(`${file}: branding: Open Graph title differs from document title`);
      if(branding.socialName&&ogSite!==branding.socialName)record(`${file}: branding: Open Graph site name differs from configuration`);
    }
    pages.set(file,{ids,links});
  }
  for(const [file,page] of pages)for(const href of page.links) {
    if(!href)continue;
    if(/^(?:https?:|mailto:|data:)/iu.test(href)||href.startsWith('//'))continue;
    if(/^[a-z][a-z0-9+.-]*:/iu.test(href)) {record(`${file}: unsafe URL ${href}`);continue;}
    let url;
    try {url=new URL(href,`https://docs.invalid${base}${file}`);}catch{record(`${file}: invalid URL ${href}`);continue;}
    let path;
    try {path=decodeURIComponent(url.pathname);}catch{record(`${file}: invalid URL encoding ${href}`);continue;}
    if(!path.startsWith(base)) {record(`${file}: URL escapes site base ${href}`);continue;}
    path=path.slice(base.length);
    const destination=files.has(path)?path:files.has(`${path}.html`)?`${path}.html`:path===''?'index.html':undefined;
    if(!destination||!files.has(destination)) {record(`${file}: missing local destination ${href}`);continue;}
    if(url.hash && pages.has(destination)) {
      let fragment;try{fragment=decodeURIComponent(url.hash.slice(1));}catch{record(`${file}: invalid fragment ${href}`);continue;}
      if(!pages.get(destination).ids.has(fragment))record(`${file}: missing fragment ${href}`);
    }
  }
  if(errorCount)throw new Error(`${errorCount} static output error(s):\n${errors.slice(0,100).join('\n')}`);
  process.stdout.write(`Validated ${pages.size} HTML pages and their local links/fragments.\n`);
  return {pages:pages.size,files:files.size};
}
if(process.argv[1] && import.meta.url===pathToFileURL(resolve(process.argv[1])).href) {
  const root=process.argv[2],base=process.argv[3]??'/';
  if(!root)throw new Error('usage: node check-site.mjs DIR [BASE]');
  await checkSite(resolve(root),base);
}
