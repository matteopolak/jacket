import assert from 'node:assert/strict';
import {mkdtemp,mkdir,readFile,rm,writeFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import {checkSite} from '../../website/docs/scripts/check-site.mjs';
import {validateLegacySite} from './doc_validate.mjs';

test('validate legacy list fragments in their actual list context without changing served bytes',async()=>{
 const root=await mkdtemp(join(tmpdir(),'jacket-legacy-validation-'));
 try{
  await mkdir(join(root,'assets/nav/api/Basic'),{recursive:true});
  await writeFile(join(root,'index.html'),'<!doctype html><html lang="en"><head><title>Home</title></head><body><main id="target"><h1>Home</h1></main></body></html>');
  const path=join(root,'assets/nav/api/Basic/0.html'),source='<li><a href="/#target">Home</a></li>\n';
  await writeFile(path,source);
  await assert.rejects(checkSite(root,'/'),/doctype|Element <li>/u);
  const result=await validateLegacySite(root,'/');
  assert.equal(result.fragmentCount,1);assert.equal(result.documents,1);
  assert.equal(await readFile(path,'utf8'),source);
  await writeFile(path,'<li><a href="/missing">Missing</a></li>');
  await assert.rejects(validateLegacySite(root,'/'),/missing local destination/u);
  await writeFile(path,'<div>not a list item</div>');
  await assert.rejects(validateLegacySite(root,'/'),/non-list node/u);
 }finally{await rm(root,{recursive:true,force:true});}
});
