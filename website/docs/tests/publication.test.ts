import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {mkdtemp,mkdir,readFile,readdir,realpath,rm,writeFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {dirname,join} from 'node:path';
import test from 'node:test';
import type {Manifest,Payload} from '../src/generated/doc-site.js';
import {Bundle} from '../src/lib/loader.ts';
import {writePublication} from '../src/lib/publication.ts';
import {moduleNavigation,moduleNavigationPath} from '../src/lib/navigation-path.ts';
import {apiModule,declaration,example,metadata,site} from './fixtures.ts';

async function fixture(payloads:Record<string,Payload>){
 const root=await mkdtemp(join(tmpdir(),'doc-publication-')),source=join(root,'source'),out=join(root,'public');await mkdir(source);await mkdir(out);
 const bytes=Buffer.from('<svg xmlns="http://www.w3.org/2000/svg"/>'),asset={path:'assets/logo.svg',bytes:bytes.length,sha256:createHash('sha256').update(bytes).digest('hex'),mediaType:'image/svg+xml',provenance:'configured_logo' as const};
 await mkdir(join(source,'assets'));await writeFile(join(source,asset.path),bytes);
 const shards=[];
 for(const [path,payload]of Object.entries(payloads)){const content=Buffer.from(JSON.stringify(payload));await mkdir(dirname(join(source,path)),{recursive:true});await writeFile(join(source,path),content);shards.push({path,bytes:content.length,sha256:createHash('sha256').update(content).digest('hex')});}
 const manifest:Manifest={schema:'jacket-doc-site',schemaVersion:1,producer:{jacket:'fixture',compiler:'fixture',formatter:null},source:{revision:null,dirty:false,configurationDigest:'a'.repeat(64),inputs:[]},partial:false,capabilities:[],shards,assets:[asset]};
 return{root,source,out,bytes,bundle:new Bundle(await realpath(source),manifest,{...site(),version:'v1'},new Map([...shards,asset].map(file=>[file.path,file])))};
}

test('one publication writes exact assets, navigation, search, versions and nested document examples',async()=>{
 const ex=example(),module=apiModule([declaration('call')]);module.docs=[{kind:'quote',children:[{kind:'fence',example:ex}]}];
 const payloads:Record<string,Payload>={'modules/Example.json':{payload:'module',data:module},'search.json':{payload:'search',data:{schemaVersion:1,entries:[]}},'guides/intro.json':{payload:'document',data:{id:'intro',title:'Intro',kind:'guide',metadata:metadata(),outline:[],nodes:[{kind:'example_group',title:'example',source:{kind:'fence',example:{...ex,id:'abcdef123456'}}}]}}};
 const f=await fixture(payloads);
 try{
  await writePublication(f.bundle,f.out);
  assert.deepEqual(await readFile(join(f.out,'assets/logo.svg')),f.bytes);
  assert.equal(await readFile(join(f.out,moduleNavigationPath(module.id)),'utf8'),moduleNavigation(module).json);
  assert.equal(await readFile(join(f.out,'search-index.json'),'utf8'),JSON.stringify(payloads['search.json']));
  assert.equal(await readFile(join(f.out,'versions.json'),'utf8'),'[{"version":"v1","url":"./"}]');
  assert.deepEqual((await readdir(join(f.out,'examples'))).sort(),['123456abcdef.json','abcdef123456.json']);
  assert.equal(await readFile(join(f.out,'examples/123456abcdef.json'),'utf8'),JSON.stringify({payload:'example',data:ex}));
 }finally{await rm(f.root,{recursive:true,force:true});}
});

test('publication refuses assets changed after admission and never overwrites existing output',async()=>{
 const f=await fixture({});
 try{
  await writeFile(join(f.source,'assets/logo.svg'),'changed');
  await assert.rejects(writePublication(f.bundle,f.out),/asset changed/);
  await writeFile(join(f.source,'assets/logo.svg'),f.bytes);
  await mkdir(join(f.out,'assets'));await writeFile(join(f.out,'assets/logo.svg'),'must survive');
  await assert.rejects(writePublication(f.bundle,f.out),/EEXIST|collision/);
  assert.equal(await readFile(join(f.out,'assets/logo.svg'),'utf8'),'must survive');
 }finally{await rm(f.root,{recursive:true,force:true});}
});

test('unequal complete example payloads cannot claim one published ID',async()=>{
 const ex=example(),module=apiModule();module.docs=[{kind:'fence',example:ex},{kind:'fence',example:{...ex,timeoutMs:1}}];const f=await fixture({'module.json':{payload:'module',data:module}});
 try{await assert.rejects(writePublication(f.bundle,f.out),/example ID collision/);}finally{await rm(f.root,{recursive:true,force:true});}
});
