import assert from 'node:assert/strict';
import test from 'node:test';
import { initExamples } from '../src/browser/examples.ts';
import type { Example } from '../src/generated/doc-site.js';

class Element extends EventTarget {
  dataset:Record<string,string>={};textContent:string|null='';className='';hidden=false;isConnected=true;disabled=false;
  figure:Element|undefined;output:Element|undefined;children:Element[]=[];
  closest(selector:string):Element|null{return selector==='figure'?this.figure??null:this;}
  querySelector(selector:string):Element|null{return selector==='.ex-out'?this.output??null:selector==='pre.code'?Object.assign(new Element(),{textContent:'main :: () {}'}):null;}
  replaceChildren(...children:Element[]):void{this.children=children;}
  append(child:Element):void{this.children.push(child);}
}
class Document extends EventTarget {
  querySelectorAll():Element[]{return [];}
  createElement():Element{return new Element();}
}
function install(values:Record<string,unknown>):()=>void{
  const originals=new Map(Object.keys(values).map(key=>[key,Object.getOwnPropertyDescriptor(globalThis,key)]));
  for(const [key,value]of Object.entries(values))Object.defineProperty(globalThis,key,{value,writable:true,configurable:true});
  return()=>{for(const [key,descriptor]of originals)if(descriptor)Object.defineProperty(globalThis,key,descriptor);else Reflect.deleteProperty(globalThis,key);};
}
const example:Example={id:'0123456789ab',language:'jai',source:{source:'main :: () {}',spans:[]},fence:'jai',location:{file:'fixture.jai',line:1},runnable:true,reason:null,program:'main :: () {}\n',runTemplate:{head:'',middle:'',tail:'',wrapped:false,pad:false},openTemplate:null,playgroundUrl:null,timeoutMs:300000,capabilities:['browser_runtime','editable_examples']};

test('discarding a page cancels a pending example load before a worker can start',async()=>{
  const document=new Document(),window=new EventTarget(),controller=new AbortController();
  const figure=new Element(),output=new Element(),button=new Element();figure.output=output;button.figure=figure;button.dataset={'act':'run','program':example.id};
  let requestSignal:AbortSignal|undefined;
  const restore=install({document,window,Element,fetch:(_url:unknown,options?:RequestInit)=>{requestSignal=options?.signal??undefined;return new Promise(()=>{});}});
  try{
    initExamples('/',controller.signal);
    document.dispatchEvent(Object.defineProperty(new Event('click'),'target',{value:button}));
    assert.ok(requestSignal,'the real example loader receives its page abort signal');
    controller.abort();assert.ok(requestSignal.aborted);assert.equal(button.dataset['act'],'run');
  }finally{controller.abort();restore();}
});
test('discarding a page terminates an active compiler worker and clears its time budget',async()=>{
  const document=new Document(),window=new EventTarget(),controller=new AbortController();
  const figure=new Element(),output=new Element(),button=new Element();figure.output=output;button.figure=figure;button.dataset={'act':'run','program':example.id};
  let terminated=0,posted=0; let message:unknown;
  class Worker{onmessage:unknown;onerror:unknown;terminate():void{terminated++;}postMessage(value:unknown):void{posted++;message=value;}}
  const restore=install({document,window,Element,Worker,location:{origin:'https://docs.invalid'},fetch:async()=>new Response(JSON.stringify({payload:'example',data:example}),{status:200})});
  try{
    initExamples('/',controller.signal,{engine:'/runtime/engine.mjs?v='+ 'a'.repeat(64),wasm:'/runtime/jai_wasm.wasm?v='+ 'b'.repeat(64)});document.dispatchEvent(Object.defineProperty(new Event('click'),'target',{value:button}));
    for(let i=0;i<50&&!posted;i++)await new Promise(resolve=>setImmediate(resolve));
    assert.equal(posted,1,'validated example starts one compiler worker');assert.deepEqual(message,{program:example.program,engine:'https://docs.invalid/runtime/engine.mjs?v='+ 'a'.repeat(64),wasm:'https://docs.invalid/runtime/jai_wasm.wasm?v='+ 'b'.repeat(64)});controller.abort();
    assert.equal(terminated,1);assert.equal(button.dataset['act'],'run');
  }finally{controller.abort();restore();}
});
