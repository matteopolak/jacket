#!/usr/bin/env node
/** Validate frozen legacy output without mistaking its published list fragments for documents.
 * node tools/bench/doc_validate.mjs /tmp/legacy-site /
 */
import {createHash} from 'node:crypto';
import {link,mkdir,mkdtemp,readFile,readdir,rm,writeFile} from 'node:fs/promises';
import {createRequire} from 'node:module';
import {dirname,join,resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {checkSite} from '../../website/docs/scripts/check-site.mjs';
const require=createRequire(new URL('../../website/docs/package.json',import.meta.url));
const {parseFragment}=await import(pathToFileURL(require.resolve('parse5')).href);
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');

/** The exact retired protocol emits li rows into ul.nav-list; the shadow never changes served files.
 * Example: await validateLegacySite('/tmp/legacy-site','/').
 */
export async function validateLegacySite(directory,base){
 const root=resolve(directory),shadow=await mkdtemp(join(dirname(root),'.jacket-legacy-validation-')),fragments=[];
 try{
  async function visit(relative=''){
   for(const entry of await readdir(join(root,relative),{withFileTypes:true})){
    const file=relative?`${relative}/${entry.name}`:entry.name;
    if(entry.isSymbolicLink())throw new Error(`symlink in frozen legacy output: ${file}`);
    if(entry.isDirectory()){await mkdir(join(shadow,file));await visit(file);continue;}
    if(!entry.isFile())throw new Error(`non-file in frozen legacy output: ${file}`);
    if(/^assets\/nav\/api\/.+\/\d+\.html$/u.test(file)){
     const source=await readFile(join(root,file)),fragment=parseFragment(source.toString('utf8'));
     if(fragment.childNodes.some(node=>node.tagName!=='li'&&!(node.nodeName==='#text'&&/^\s*$/u.test(node.value))))throw new Error(`${file}: non-list node in legacy navigation fragment`);
     fragments.push({file,sha256:sha(source),bytes:source.length});
     await writeFile(join(shadow,file),`<!doctype html><html lang="en"><head><title>Navigation fragment validation</title></head><body><ul class="nav-list">${source.toString('utf8')}</ul></body></html>`);
    }else await link(join(root,file),join(shadow,file));
   }
  }
  await visit();
  const result=await checkSite(shadow,base);
  for(const fragment of fragments)if(sha(await readFile(join(root,fragment.file)))!==fragment.sha256)throw new Error(`served legacy fragment changed during validation: ${fragment.file}`);
  return {...result,documents:result.pages-fragments.length,fragmentCount:fragments.length,fragments,context:'ul.nav-list; validation-only shadow, original served bytes unchanged'};
 }finally{await rm(shadow,{recursive:true,force:true});}
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){
 if(!process.argv[2])throw new Error('usage: node tools/bench/doc_validate.mjs DIR [BASE]');
 console.log(JSON.stringify(await validateLegacySite(process.argv[2],process.argv[3]??'/'),null,2));
}
