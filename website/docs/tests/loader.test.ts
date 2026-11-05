import assert from 'node:assert/strict';
import { mkdtemp, writeFile, symlink, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { Bundle, loadBundle, safeRelativePath } from '../src/lib/loader.ts';

import type { Manifest } from '../src/generated/doc-site.js';
import { site } from './fixtures.ts';

/** Import validation rejects native and encoded traversal before touching the filesystem. */
test('portable inventory paths reject traversal, roots and platform ambiguity', () => {
  for (const path of ['', '/etc/passwd', '../site.json', 'a/../b', 'a//b', 'C:/site.json', '\\server\\share', 'a\0b', 'a/./b', 'data/CON.json', 'data/trailing.', 'a?b']) {
    assert.throws(() => safeRelativePath(path), /unsafe relative path/);
  }
  assert.equal(safeRelativePath('modules/Basic.json'), 'modules/Basic.json');
});

test('unsupported contracts fail before opening inventoried shards', async () => {
  const root = await mkdtemp(join(tmpdir(), 'jacket-export-test-'));
  try {
    await writeFile(join(root, 'manifest.json'), JSON.stringify({ payload: 'manifest', data: {
      schema: 'jacket-doc-site', schemaVersion: 2,
      producer: {jacket: 'test', compiler: 'test', formatter:null},
      source: {revision: null, dirty: false, configurationDigest: 'a'.repeat(64), inputs: []},
      partial: false, capabilities: [], shards: [], assets: [],
    }}));
    await assert.rejects(loadBundle(root), /schemaVersion.*unsupported/);
    await rm(join(root, 'manifest.json'));
    const other = join(root, 'outside.json');
    await writeFile(other, '{}');
    await symlink(other, join(root, 'manifest.json'));
    await assert.rejects(loadBundle(root), /symlink/);
  } finally {
    await rm(root, {recursive: true, force: true});
  }
});

test('formatter identity and profile are admitted before opening shards',async()=>{
  const root=await mkdtemp(join(tmpdir(),'jacket-formatter-manifest-'));
  try {
    const formatter={version:'owned formatter',sha256:'a'.repeat(64),profile:'documentation-declarations/1',maxWidth:88,indentWidth:4};
    for(const broken of [{...formatter,profile:'unknown/1'},{...formatter,sha256:'bad'},{...formatter,maxWidth:0},{...formatter,indentWidth:0},{...formatter,version:''}]){
      await writeFile(join(root,'manifest.json'),JSON.stringify({payload:'manifest',data:{schema:'jacket-doc-site',schemaVersion:1,producer:{jacket:'test',compiler:'test',formatter:broken},source:{revision:null,dirty:false,configurationDigest:'a'.repeat(64),inputs:[]},partial:false,capabilities:[],shards:[],assets:[]}}));
      await assert.rejects(loadBundle(root),/formatter|SHA-256/,`invalid formatter identity accepted: ${JSON.stringify(broken)}`);
    }
  }finally{await rm(root,{recursive:true,force:true});}
});


test('bundle identity invalidates retained chrome for module, source, config and runtime changes',()=>{
  const manifest:Manifest={schema:'jacket-doc-site',schemaVersion:1,producer:{jacket:'owned',compiler:'owned',formatter:null},source:{dirty:false,revision:null,configurationDigest:'a'.repeat(64),inputs:[{path:'module.jai',sha256:'b'.repeat(64)}]},partial:false,capabilities:[],shards:[{path:'modules/Module.json',bytes:1,sha256:'c'.repeat(64)}],assets:[{path:'runtime/interp.wasm',bytes:1,sha256:'d'.repeat(64),mediaType:'application/wasm',provenance:'browser_runtime'}]};
  const make=(value:Manifest,root='/owned')=>new Bundle(root,value,site(),new Map());
  const initial=make(manifest).identity;assert.match(initial,/^[a-f0-9]{64}$/u);
  assert.equal(make(structuredClone(manifest),'/another-owned-root').identity,initial,'host path changed logical export identity');
  for(const change of [(value:Manifest)=>{value.shards[0]!.sha256='e'.repeat(64);},(value:Manifest)=>{value.source.inputs[0]!.sha256='e'.repeat(64);},(value:Manifest)=>{value.source.configurationDigest='e'.repeat(64);},(value:Manifest)=>{value.assets[0]!.sha256='e'.repeat(64);}]){
    const changed=structuredClone(manifest);change(changed);assert.notEqual(make(changed).identity,initial);
  }
});


test('unknown asset provenance is rejected before any shard is opened',async()=>{
  const root=await mkdtemp(join(tmpdir(),'jacket-asset-origin-'));
  try {
    await writeFile(join(root,'manifest.json'),JSON.stringify({payload:'manifest',data:{schema:'jacket-doc-site',schemaVersion:1,producer:{jacket:'owned',compiler:'owned',formatter:null},source:{dirty:false,revision:null,configurationDigest:'a'.repeat(64),inputs:[]},partial:false,capabilities:[],shards:[],assets:[{path:'assets/logo.svg',mediaType:'image/svg+xml',bytes:1,sha256:'a'.repeat(64),provenance:'unknown-origin'}]}}));
    await assert.rejects(loadBundle(root),/provenance/);
  }finally{await rm(root,{recursive:true,force:true});}
});

test('runtime capability without its complete declared asset pair is rejected before shards',async()=>{
  const root=await mkdtemp(join(tmpdir(),'jacket-runtime-manifest-'));
  try {
    await writeFile(join(root,'manifest.json'),JSON.stringify({payload:'manifest',data:{schema:'jacket-doc-site',schemaVersion:1,producer:{jacket:'owned',compiler:'owned',formatter:null},source:{dirty:false,revision:null,configurationDigest:'a'.repeat(64),inputs:[]},partial:false,capabilities:['browser_runtime','editable_examples'],shards:[],assets:[]}}));
    await assert.rejects(loadBundle(root),/runtime.*asset|asset.*runtime/);
  }finally{await rm(root,{recursive:true,force:true});}
});


test('request identities follow individual admitted files, not unrelated export changes',async()=>{
  const {assetURL,publicationImageURL,versionedURL}=await import('../src/lib/urls.ts');
  const manifest:Manifest={schema:'jacket-doc-site',schemaVersion:1,producer:{jacket:'owned',compiler:'owned',formatter:null},source:{dirty:false,revision:null,configurationDigest:'a'.repeat(64),inputs:[]},partial:false,capabilities:[],shards:[{path:'search.json',bytes:1,sha256:'b'.repeat(64)},{path:'modules/Example.json',bytes:1,sha256:'c'.repeat(64)}],assets:[{path:'assets/logo.png',bytes:1,sha256:'d'.repeat(64),mediaType:'image/png',provenance:'configured_logo'}]};
  const make=(value:Manifest)=>new Bundle('/owned',value,{...site(),pageBase:'/docs/',canonicalOrigin:'https://docs.test'},new Map([...value.shards,...value.assets].map(file=>[file.path,file])));
  const old=make(manifest),changed=structuredClone(manifest);changed.source.configurationDigest='f'.repeat(64);
  const current=make(changed);assert.notEqual(old.identity,current.identity);
  assert.equal(old.fileDigest('search.json'),current.fileDigest('search.json'));
  assert.equal(old.fileDigest('modules/Example.json'),current.fileDigest('modules/Example.json'));
  assert.throws(()=>old.fileDigest('missing.json'),/missing/);
  assert.equal(assetURL(old.site,'assets/logo.png'),'/docs/assets/logo.png?v='+ 'd'.repeat(64));
  assert.equal(publicationImageURL(old.site,'https://docs.test/docs/assets/logo.png'),'https://docs.test/docs/assets/logo.png?v='+ 'd'.repeat(64));
  assert.equal(publicationImageURL(old.site,'https://external.test/docs/assets/logo.png'),'https://external.test/docs/assets/logo.png');
  assert.equal(publicationImageURL(old.site,'https://docs.test/not-owned.png'),'https://docs.test/not-owned.png');
  const oldURL=versionedURL('/docs/search-index.json',old.fileDigest('search.json'));
  const cache=new Map([[oldURL,{name:'Old valid search'}]]);
  changed.shards[0]!.sha256='e'.repeat(64);const deployed=make(changed);
  const newURL=versionedURL('/docs/search-index.json',deployed.fileDigest('search.json'));
  assert.notEqual(oldURL,newURL);assert.equal(cache.has(newURL),false,'a warm valid old response must not satisfy a changed shard request');
  assert.equal(deployed.fileDigest('modules/Example.json'),old.fileDigest('modules/Example.json'));
  assert.throws(()=>versionedURL('/docs/search-index.json','bad'),/digest/);
});
