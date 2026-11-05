/** Compare every formatter-authored API definition, anchor and resolved link with actual HTML.
 * node --experimental-strip-types scripts/check-definitions.mjs EXPORT_DIR SITE_DIR OUTPUT_JSON
 */
import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import {resolve,sep} from 'node:path';
import {createHash} from 'node:crypto';
import {parse} from 'parse5';
import {resolver} from '../src/lib/urls.ts';
const [exportDirectory,siteDirectory,output]=process.argv.slice(2);
assert(exportDirectory&&siteDirectory&&output,'pass export, production output and receipt paths');
const exported=resolve(exportDirectory),built=resolve(siteDirectory),encoder=new TextEncoder();
const manifest=JSON.parse(await readFile(resolve(exported,'manifest.json'),'utf8')).data;
const site=JSON.parse(await readFile(resolve(exported,'site.json'),'utf8')).data,href=resolver(site);
const receipt={definitions:0,declarations:0,anchors:0,resolvedLinks:0,codeBytes:0,formatter:manifest.producer.formatter,focused:[],failures:[]};
const attrs=(node,key)=>node.attrs?.find(value=>value.name===key)?.value;
const all=node=>[node,...(node.childNodes??[]).flatMap(all)];
const text=node=>node.nodeName==='#text'?node.value:(node.childNodes??[]).map(text).join('');
const hash=value=>createHash('sha256').update(value).digest('hex');
for(const shard of manifest.shards.filter(entry=>entry.path.startsWith('modules/'))){
 const module=JSON.parse(await readFile(resolve(exported,shard.path),'utf8')).data;
 for(const declaration of module.declarations.filter(item=>item.definitions.length>0)){
  receipt.declarations++;
  try{
   const file=resolve(built,`${declaration.route}.html`);assert(file.startsWith(built+sep));
   const html=await readFile(file),tree=parse(html.toString('utf8'));
   const snippets=all(tree).filter(node=>node.tagName==='pre'&&(attrs(node,'class')??'').split(/\s+/).includes('semantic-declaration'));
   assert.equal(snippets.length,declaration.definitions.length,'definition count');
   declaration.definitions.forEach((definition,index)=>{
    const snippet=snippets[index],content=text(snippet),bytes=encoder.encode(content);
    assert.equal(content,definition.code.source,'formatter output bytes/content');
    assert.equal(attrs(snippet,'id'),definition.id,'definition identity');
    const ids=all(snippet).map(node=>attrs(node,'id')).filter(Boolean);
    const links=[];let cursor=0;
    function walk(node){const start=cursor;if(node.nodeName==='#text')cursor+=encoder.encode(node.value).length;else for(const child of node.childNodes??[])walk(child);if(node.tagName==='a')links.push({start,end:cursor,href:attrs(node,'href')});}
    walk(snippet);
    const expected=definition.code.spans.filter(span=>span.target).map(span=>({start:span.start,end:span.end,href:href(span.target)}));
    assert.deepEqual(links,expected,'compiler resolved links');
    for(const anchor of definition.anchors){
     const element=all(snippet).find(node=>attrs(node,'id')===anchor.id);
     let line=element?.parentNode;while(line&&line!==snippet&&!(attrs(line,'class')??'').split(/\s+/).includes('formatted-line'))line=line.parentNode;
     assert(line&&line!==snippet,'field target lacks its formatter-line presentation wrapper');
     assert.equal(ids.filter(id=>id===anchor.id).length,1,`unique field anchor ${anchor.id}`);
     assert.equal(new TextDecoder('utf8',{fatal:true}).decode(bytes.subarray(anchor.start,anchor.end)),anchor.name,'anchor name/source position');
     assert(!links.some(link=>link.start<anchor.end&&link.end>anchor.start),'declared field identifier must not become a type link');
    }
    receipt.definitions++;receipt.anchors+=definition.anchors.length;receipt.resolvedLinks+=links.length;receipt.codeBytes+=bytes.length;
    if(['api/File_Watcher/File_Watcher','api/Bindings_Generator/Library_Info'].includes(declaration.route))receipt.focused.push({route:declaration.route,definition:definition.id,platforms:definition.platforms,source:definition.source,codeSha256:hash(bytes),htmlSha256:hash(html),fields:definition.anchors.map(anchor=>({id:anchor.id,name:anchor.name,start:anchor.start,end:anchor.end}))});
   });
  }catch(error){receipt.failures.push({route:declaration.route,error:error.stack??String(error)});}
 }
}
await writeFile(output,JSON.stringify(receipt,null,2)+'\n');
assert.deepEqual(receipt.failures,[],'formatted API content differs from the admitted semantic export');
console.log(JSON.stringify({declarations:receipt.declarations,definitions:receipt.definitions,anchors:receipt.anchors,resolvedLinks:receipt.resolvedLinks,codeBytes:receipt.codeBytes,failures:receipt.failures.length}));
