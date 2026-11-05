import assert from 'node:assert/strict';
import test from 'node:test';
import * as lifetime from '../src/browser/lifetime.ts';
import { pageScope, frame } from '../src/browser/lifetime.ts';

test('replacing a page scope removes old listeners and parent abort removes current listeners', () => {
  const scope=pageScope(),parent=new AbortController(),target=new EventTarget();
  let old=0,current=0;
  const previous=scope(parent.signal);target.addEventListener('change',()=>old++,{signal:previous});
  const next=scope(parent.signal);target.addEventListener('change',()=>current++,{signal:next});
  target.dispatchEvent(new Event('change'));assert.equal(old,0);assert.equal(current,1);
  parent.abort();target.dispatchEvent(new Event('change'));assert.equal(current,1);assert.ok(next.aborted);
});
test('an already aborted page never attaches live enhancements', () => {
  const parent=new AbortController();parent.abort();assert.ok(pageScope()(parent.signal).aborted);
});

test('page teardown cancels pending animation work and stale callbacks cannot touch replacements', () => {
  const previousFrame=globalThis.requestAnimationFrame,previousCancel=globalThis.cancelAnimationFrame;
  let callback: FrameRequestCallback | undefined,cancelled=0,calls=0;
  globalThis.requestAnimationFrame=next=>{callback=next;return 17;};
  globalThis.cancelAnimationFrame=id=>{cancelled=id;};
  try {
    const scope=new AbortController();frame(scope.signal,()=>calls++);scope.abort();
    assert.equal(cancelled,17);callback?.(0);assert.equal(calls,0);
  } finally {
    if(previousFrame)globalThis.requestAnimationFrame=previousFrame;else Reflect.deleteProperty(globalThis,'requestAnimationFrame');
    if(previousCancel)globalThis.cancelAnimationFrame=previousCancel;else Reflect.deleteProperty(globalThis,'cancelAnimationFrame');
  }
});


test('native visual cancellation is observed only after the DOM swap succeeds', async()=>{
  assert.equal(typeof lifetime.observeTransition,'function');
  const reported:unknown[]=[];
  await lifetime.observeTransition({ready:Promise.reject(new DOMException('skipped','AbortError')),updateCallbackDone:Promise.resolve()},error=>{reported.push(error);});
  assert.deepEqual(reported,[]);
});
test('a cancelled visual transition cannot hide a failed DOM update', async()=>{
  assert.equal(typeof lifetime.observeTransition,'function');
  const reported:unknown[]=[],failure=new Error('swap failed');
  await lifetime.observeTransition({ready:Promise.reject(new DOMException('skipped','AbortError')),updateCallbackDone:Promise.reject(failure)},error=>{reported.push(error);});
  assert.deepEqual(reported,[failure]);
});
test('unexpected visual errors and ordinary errors named AbortError remain visible', async()=>{
  assert.equal(typeof lifetime.observeTransition,'function');
  for(const failure of [new DOMException('duplicate name','InvalidStateError'),Object.assign(new Error('application failure'),{name:'AbortError'})]){
    const reported:unknown[]=[];
    await lifetime.observeTransition({ready:Promise.reject(failure),updateCallbackDone:Promise.resolve()},error=>{reported.push(error);});
    assert.deepEqual(reported,[failure]);
  }
});
test('successful visual readiness needs no cancellation handling', async()=>{
  assert.equal(typeof lifetime.observeTransition,'function');
  const reported:unknown[]=[];
  await lifetime.observeTransition({ready:Promise.resolve(),updateCallbackDone:Promise.resolve()},error=>{reported.push(error);});
  assert.deepEqual(reported,[]);
});


test('live shell compatibility refuses another protocol, base, version or missing main',()=>{
  const shell=(values:Record<string,string>={},main=true)=>shellDocument({data:values,main});
  assert.equal(lifetime.compatibleShell(shell(),shell()),true);
  for(const values of [{docShell:'other'},{base:'/other/'},{docVersion:'other'}])assert.equal(lifetime.compatibleShell(shell(),shell(values)),false);
  assert.equal(lifetime.compatibleShell(shell(),shell({},false)),false);
});


function shellDocument(options:{main?:boolean;data?:Record<string,string|undefined>;bootstrap?:string|null;scripts?:string[];styles?:string[]}={}):Document {
  return {body:{dataset:{docShell:'jacket-docs/1',base:'/',docVersion:'test',docIdentity:'same-export',docState:'fresh',docIcons:'/_astro/icons-a.svg',...options.data}},
    querySelector:(selector:string)=>selector==='script[data-doc-bootstrap]'?options.bootstrap===null?null:{textContent:options.bootstrap??'',getAttribute:(name:string)=>name==='src'?'/_astro/bootstrap-a.js':null,hasAttribute:()=>false}:selector==='.wrap > main'&&options.main!==false?{}:null,
    querySelectorAll:(selector:string)=>(selector.startsWith('script')?options.scripts??['/_astro/runtime-a.js']:options.styles??['/_astro/site-a.css']).map(value=>({getAttribute:()=>value,closest:()=>null})),
  } as unknown as Document;
}
test('retained shell refuses stale export, changed compiled runtime/styles or bootstrap code',()=>{
  const initial=shellDocument();assert.equal(lifetime.compatibleShell(initial,shellDocument()),true);
  for(const changed of [shellDocument({data:{docIdentity:'changed-module-or-config'}}),shellDocument({data:{docState:'stale'}}),shellDocument({data:{docIcons:'/_astro/icons-b.svg'}}),shellDocument({data:{docIcons:undefined}}),shellDocument({data:{docIdentity:undefined}}),shellDocument({scripts:['/_astro/runtime-b.js']}),shellDocument({styles:['/_astro/site-b.css']}),shellDocument({bootstrap:'changed bootstrap'}),shellDocument({bootstrap:null})])assert.equal(lifetime.compatibleShell(initial,changed),false);
});

test('live shell compatibility uses the external prepaint asset identity',()=>{
 const shell=(src:string|null,options:{inline?:string;defer?:boolean;async?:boolean;type?:string}={}):Document=>{
  const document=shellDocument(),query=document.querySelector.bind(document);
  document.querySelector=((selector:string)=>selector==='script[data-doc-bootstrap]'?{textContent:options.inline??'',getAttribute:(name:string)=>name==='src'?src:name==='type'?options.type??null:null,hasAttribute:(name:string)=>name==='defer'?!!options.defer:name==='async'?!!options.async:false}:query(selector))as typeof document.querySelector;
  return document;
 };
 assert.equal(lifetime.compatibleShell(shell('/_astro/bootstrap-a.js'),shell('/_astro/bootstrap-a.js')),true);
 for(const incoming of [shell('/_astro/bootstrap-b.js'),shell(null),shell('/_astro/bootstrap-a.js',{inline:'unexpected inline code'}),shell('/_astro/bootstrap-a.js',{defer:true}),shell('/_astro/bootstrap-a.js',{async:true}),shell('/_astro/bootstrap-a.js',{type:'module'})])assert.equal(lifetime.compatibleShell(shell('/_astro/bootstrap-a.js'),incoming),false);
});
