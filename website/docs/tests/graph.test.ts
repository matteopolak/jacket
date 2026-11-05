import assert from 'node:assert/strict';
import {test} from 'node:test';
import {graphValidator} from '../src/lib/graph.ts';
import {example,metadata,site} from './fixtures.ts';

test('whole graph rejects referenced assets absent from its declared inventory',()=>{
  const validate=graphValidator(site());
  assert.throws(()=>validate({payload:'document',data:{id:'guide',title:'Guide',kind:'guide',source:null,nodes:[{kind:'image',target:{kind:'asset',path:'assets/missing.png'},title:'',alternative:'Missing'}],outline:[],metadata:metadata()}},'guide.json'),/unlisted asset/);
});

test('standalone example payloads obey the same semantic rules as embedded fences',()=>{
  const validate=graphValidator(site()),missing=example();missing.program=null;
  assert.throws(()=>validate({payload:'example',data:missing},'example.json'),/runnable example/);
  const mismatch=example();mismatch.program='different program';
  assert.throws(()=>validate({payload:'example',data:mismatch},'example.json'),/template.*program/);
  graphValidator(site(),new Set(['runtime/engine.mjs','runtime/jai_wasm.wasm']),new Set(['browser_runtime','editable_examples'] as const))({payload:'example',data:example()},'valid.json');
});

test('metadata and source destinations are validated before rendering imported data',()=>{
  const invalid=site();invalid.repository='javascript:alert(1)';
  assert.throws(()=>graphValidator(invalid)({payload:'site',data:invalid},'site.json'),/unsupported external URL/);
  const head=metadata();head.canonical='javascript:alert(1)';
  assert.throws(()=>graphValidator(site())({payload:'document',data:{id:'guide',title:'Guide',kind:'guide',source:null,nodes:[],outline:[],metadata:head}},'guide.json'),/unsupported external URL/);
});

test('example groups validate nested source/results and reject forged ownership',()=>{
  const document=(nodes:import('../src/generated/doc-site.js').DocumentNode[])=>({payload:'document' as const,data:{id:'guide',title:'Guide',kind:'guide' as const,source:null,nodes,outline:[],metadata:metadata()}});
  const code={kind:'code_block' as const,language:'jai',info:'jai',source:{source:'0',spans:[]}};
  graphValidator(site())(document([{kind:'example_group',title:'example',note:null,source:code,result:{kind:'diagnostic',body:{...code,language:'diagnostic',info:'diagnostic'}}}]),'valid.json');
  assert.throws(()=>graphValidator(site())(document([{kind:'example_group',title:'example',note:null,source:{kind:'paragraph',children:[]},result:null}]),'bad-source.json'),/source must be a code fence/);
  assert.throws(()=>graphValidator(site())(document([{kind:'example_group',title:'fixed',note:null,source:code,result:{kind:'diagnostic',body:code}}]),'bad-owner.json'),/result must belong/);
  const missing=example();missing.program=null;
  assert.throws(()=>graphValidator(site())(document([{kind:'example_group',title:'example',note:null,source:{kind:'fence',example:missing},result:null}]),'bad-program.json'),/runnable example/);
});

test('runnable examples require the bundle capabilities and both declared runtime assets',()=>{
  const payload={payload:'example' as const,data:example()},runtime=new Set(['runtime/engine.mjs','runtime/jai_wasm.wasm']);
  assert.throws(()=>graphValidator(site())(payload,'example.json'),/bundle.*runtime/);
  const caps=new Set(['browser_runtime','editable_examples'] as const);
  assert.throws(()=>graphValidator(site(),runtime)(payload,'example.json'),/bundle.*runtime/);
  assert.throws(()=>graphValidator(site(),new Set(),caps)(payload,'example.json'),/bundle.*runtime/);
  graphValidator(site(),runtime,caps)(payload,'valid.json');
});
