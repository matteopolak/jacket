import assert from 'node:assert/strict';
import {test} from 'node:test';
import {mkdtemp,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {checkSite} from '../scripts/check-site.mjs';

test('whole output validation rejects invalid HTML content models that parsers repair or accept',async()=>{
  const root=await mkdtemp(join(tmpdir(),'jacket-html-standard-'));
  try {
    const page=(body:string)=>`<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Fixture</title></head><body>${body}</body></html>`;
    await writeFile(join(root,'index.html'),page('<main><h1>Fixture</h1><span><div>Invalid phrasing content</div></span></main>'));
    await assert.rejects(checkSite(root,'/'),/element-permitted-content/);
    await writeFile(join(root,'index.html'),page('<main><h1>Fixture</h1><p>Valid content.</p></main>'));
    assert.equal((await checkSite(root,'/')).pages,1);
  } finally {await rm(root,{recursive:true,force:true});}
});

test('production branding keeps the configured name separate from breadcrumb labels',async()=>{
  const root=await mkdtemp(join(tmpdir(),'jacket-html-brand-'));
  try {
    const {mkdir}=await import('node:fs/promises');await mkdir(join(root,'api'));
    const page=(title:string)=>`<!doctype html><html lang="en"><head><meta charset="utf-8"><title>${title}</title><meta property="og:title" content="${title}"><meta property="og:site_name" content="Jacket"></head><body><header><a href="/"><span class="wordmark">Jacket</span></a></header><main><h1>Fixture</h1></main></body></html>`;
    await writeFile(join(root,'index.html'),page('Jacket'));
    await writeFile(join(root,'api/Basic.html'),page('Basic — Jacket std'));
    await assert.rejects(checkSite(root,'/',{name:'Jacket',homeTitle:'Jacket',socialName:'Jacket'}),/branding/);
    await writeFile(join(root,'api/Basic.html'),page('Basic — Jacket'));
    assert.equal((await checkSite(root,'/',{name:'Jacket',homeTitle:'Jacket',socialName:'Jacket'})).pages,2);
  } finally {await rm(root,{recursive:true,force:true});}
});
