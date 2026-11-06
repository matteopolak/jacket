import assert from 'node:assert/strict';
import { test } from 'node:test';
import { initSearch, search } from '../src/browser/search.ts';
import type { SearchEntry } from '../src/generated/doc-site.js';
const entry=(name:string,binding=false):SearchEntry=>({id:`api/Fuzz/${name}`,name,qualifiedName:`Fuzz.${name}`,module:'Fuzz',target:{kind:'page',id:`api/Fuzz/${name}`,anchor:null},summary:'Fuzz input',kind:'declaration',binding,experimental:false,category:null,overloads:[{parameters:['string'],results:['bool']}]});
test('corrected compiler binding fact controls rank and signature eligibility',()=>{
  const own=entry('Case'),foreign=entry('Case_Native',true);
  assert.equal(search([foreign,own],'Case')[0],own);
  assert.deepEqual(search([foreign,own],'string -> bool'),[own]);
  assert.deepEqual(search([own],'Fuzz.Case'),[own]);
});

test('signature results prefer exact arity and preserve nested type argument commas',()=>{
  const extra=entry('A_extra'),exact=entry('Z_exact'),nested=entry('Nested');
  extra.overloads=[{parameters:['string','int'],results:['bool']}];
  exact.overloads=[{parameters:['string'],results:['bool']}];
  nested.overloads=[{parameters:['Map(string, int)'],results:['bool']}];
  assert.deepEqual(search([extra,exact],'string -> bool'),[exact,extra]);
  assert.deepEqual(search([nested],'Map(string, int) -> bool'),[nested]);
});

test('search previews preserve semantic signature details and highlight only owned API types',async()=>{
  const {searchPreview}=await import('../src/browser/search.ts');
  const own=entry('Case');own.preview={kind:'procedure',parameters:[{name:'text',ty:'string',default:'"keep  spaces"'}],results:[{name:'',ty:'bool',default:''}]};
  const parts=searchPreview(own);
  assert.equal(parts.map(part=>part.text).join(''),'(text: string = "keep  spaces") -> bool');
  assert.deepEqual(parts.filter(part=>part.className==='ty').map(part=>part.text),['string','bool']);
  own.binding=true;assert.equal(searchPreview(own).some(part=>part.className==='ty'),false);
});


test('navigation cleanup clears retained search results while preserving the input and focus',()=>{
  class Element extends EventTarget {
    value='Bucket';hidden=false;children=['stale result'];
    attributes=new Map([['aria-expanded','true'],['aria-activedescendant','hit-0']]);
    setAttribute(name:string,value:string):void{this.attributes.set(name,value);}
    removeAttribute(name:string):void{this.attributes.delete(name);}
    replaceChildren():void{this.children=[];}
  }
  const input=new Element(),box=new Element(),document=new EventTarget();
  Object.assign(document,{activeElement:input,querySelector:(selector:string)=>selector==='#search-q'?input:box});
  const previous=Object.getOwnPropertyDescriptor(globalThis,'document');
  Object.defineProperty(globalThis,'document',{configurable:true,value:document});
  try {
    const controller=new AbortController();
    initSearch('/',()=>'/unused',controller.signal,()=>assert.fail('cleanup must not navigate'));
    controller.abort();
    assert.equal(box.hidden,true,'old rows must not remain visible under a fresh controller');
    assert.deepEqual(box.children,[],'detached search controller must release its result rows');
    assert.equal(input.attributes.get('aria-expanded'),'false');
    assert.equal(input.attributes.has('aria-activedescendant'),false);
    assert.equal(input.value,'Bucket');
    assert.equal((document as EventTarget&{activeElement:Element}).activeElement,input);
  } finally {
    if(previous)Object.defineProperty(globalThis,'document',previous);else Reflect.deleteProperty(globalThis,'document');
  }
});


test('lazy search requests the admitted per-file URL exactly',()=>{
  const input=Object.assign(new EventTarget(),{setAttribute:()=>{},removeAttribute:()=>{}}),box=Object.assign(new EventTarget(),{hidden:true,replaceChildren:()=>{}});
  const doc=Object.assign(new EventTarget(),{querySelector:(selector:string)=>selector==='#search-q'?input:box});
  const source='/docs/search-index.json?v='+ 'a'.repeat(64),requests:string[]=[];
  const values={document:doc,fetch:(url:string)=>{requests.push(url);return new Promise(()=>{});}};
  const descriptors=new Map(Object.keys(values).map(key=>[key,Object.getOwnPropertyDescriptor(globalThis,key)])),scope=new AbortController();
  for(const [key,value]of Object.entries(values))Object.defineProperty(globalThis,key,{configurable:true,value});
  try{initSearch(source,()=>'/unused',scope.signal,()=>{});assert.deepEqual(requests,[]);input.dispatchEvent(new Event('focus'));assert.deepEqual(requests,[source]);}
  finally{scope.abort();for(const [key,descriptor]of descriptors)if(descriptor)Object.defineProperty(globalThis,key,descriptor);else Reflect.deleteProperty(globalThis,key);}
});

/** Real lazy loader with a controlled response; only the DOM shell is fabricated. */
async function lazySearchFixture(){
  await import('../src/generated/validate-search.js');
  class Element extends EventTarget {
    value='Case';hidden=true;children:Element[]=[];attrs=new Map<string,string>();className='';textContent='';href='';tabIndex=0;id='';dataset:Record<string,string>={};
    setAttribute(key:string,value:string):void{this.attrs.set(key,value);}removeAttribute(key:string):void{this.attrs.delete(key);}
    replaceChildren():void{this.children=[];this.textContent='';}append(...nodes:Element[]):void{this.children.push(...nodes);}
    contains(value:unknown):boolean{return value===this||this.children.some(child=>child.contains(value));}
    focus():void{document.activeElement=this;this.dispatchEvent(new Event('focus'));}select():void{}
    matches():boolean{return this===input;}isContentEditable=false;
  }
  const input=new Element(),box=new Element(),document=Object.assign(new EventTarget(),{activeElement:null as Element|null,querySelector:(selector:string)=>selector==='#search-q'?input:box,createElement:()=>new Element()});
  const controller=new AbortController(),requested:string[]=[],navigated:string[]=[],source='/search-index.json?v='+ 'a'.repeat(64);
  let parses=0;
  let resolveResponse:(value:Response)=>void=()=>{};
  const pending=new Promise<Response>(resolve=>{resolveResponse=resolve;});
  const values={document,Node:Element,HTMLElement:Element,fetch:(url:string)=>{requested.push(url);return pending.then(response=>{const copy=response.clone(),json=copy.json.bind(copy);copy.json=()=>{parses++;return json();};return copy;});}};
  const descriptors=new Map(Object.keys(values).map(key=>[key,Object.getOwnPropertyDescriptor(globalThis,key)]));
  for(const[key,value]of Object.entries(values))Object.defineProperty(globalThis,key,{configurable:true,value});
  initSearch(source,target=>target.kind==='page'?'/'+target.id:'/unused',controller.signal,url=>navigated.push(url));
  const key=(value:string):void=>{input.dispatchEvent(Object.assign(new Event('keydown',{cancelable:true}),{key:value}));};
  const flush=async():Promise<void>=>{await new Promise<void>(resolve=>setImmediate(resolve));await new Promise<void>(resolve=>setImmediate(resolve));};
  return {input,box,document,controller,requested,navigated,key,flush,source,get parses(){return parses;},
    type:(value:string):void=>{input.value=value;input.dispatchEvent(new Event('input'));},
    release:async(outcome:'success'|'error'|'invalid'):Promise<void>=>{resolveResponse(outcome!=='error'?new Response(JSON.stringify({payload:'search',data:{schemaVersion:outcome==='invalid'?2:1,entries:[{...entry('Case_A'),preview:null},{...entry('Case_B'),preview:null}]}})):new Response('unavailable',{status:503}));await flush();},
    dismiss:(kind:string):void=>{if(kind==='escape')key('Escape');else if(kind==='outside')document.dispatchEvent(Object.defineProperty(new Event('click'),'target',{value:new Element()}));else if(kind==='focus')input.dispatchEvent(Object.assign(new Event('focusout'),{relatedTarget:new Element()}));else if(kind==='empty'){input.value='';input.dispatchEvent(new Event('input'));}else controller.abort();},
    restore:():void=>{controller.abort();for(const[key,descriptor]of descriptors)if(descriptor)Object.defineProperty(globalThis,key,descriptor);else Reflect.deleteProperty(globalThis,key);},
  };
}
for(const outcome of ['success','error']as const)for(const dismissal of ['escape','outside','focus','empty','abort'])test(`pending search ${outcome} respects ${dismissal} dismissal`,async()=>{
  const f=await lazySearchFixture();try{assert.equal(f.requested.length,0);f.input.focus();f.type('Case');f.dismiss(dismissal);await f.release(outcome);assert.equal(f.box.hidden,true,'late search response reopened dismissed results');assert.equal(f.input.attrs.get('aria-expanded'),'false');assert.equal(f.requested.length,1,'lazy promise must remain shared');f.key('Enter');assert.deepEqual(f.navigated,[],'hidden results must not navigate');}finally{f.restore();}
});
test('hidden Enter cannot select stale hits; explicit arrows reopen with current selection',async()=>{
  const f=await lazySearchFixture();try{f.type('Case');await f.release('success');assert.equal(f.box.hidden,false);f.key('Escape');f.key('Enter');assert.deepEqual(f.navigated,[]);f.key('ArrowDown');await f.flush();assert.equal(f.box.hidden,false);f.key('Enter');assert.equal(f.navigated.at(-1),'/api/Fuzz/Case_A');f.key('Escape');f.key('ArrowUp');await f.flush();f.key('Enter');assert.equal(f.navigated.at(-1),'/api/Fuzz/Case_B');assert.equal(f.requested.length,1);}finally{f.restore();}
});
test('current query errors remain visible and a cleared query does not inherit an old failure',async()=>{
  const f=await lazySearchFixture();try{f.type('Case');await f.release('error');assert.equal(f.box.hidden,false);assert.match(f.box.textContent,/Search could not load/);f.type('');await f.flush();assert.equal(f.box.hidden,true);assert.deepEqual(f.navigated,[]);}finally{f.restore();}
});


test('visible arrows retain their cyclic selection semantics',async()=>{
  const f=await lazySearchFixture();try{f.type('Case');await f.release('success');f.key('ArrowDown');f.key('Enter');assert.equal(f.navigated.at(-1),'/api/Fuzz/Case_B');f.key('ArrowUp');f.key('Enter');assert.equal(f.navigated.at(-1),'/api/Fuzz/Case_A');assert.equal(f.requested.length,1);}finally{f.restore();}
});


test('completed search admission is reused by a new page in the same document',async()=>{
  const f=await lazySearchFixture(),next=new AbortController();
  try {
    f.type('Case');await f.release('success');assert.equal(f.requested.length,1);assert.equal(f.parses,1);
    f.controller.abort();
    initSearch(f.source,target=>target.kind==='page'?'/'+target.id:'/unused',next.signal,url=>f.navigated.push(url));
    f.input.focus();f.type('Case');await f.flush();
    assert.equal(f.requested.length,1,'unchanged admitted search must not fetch again after navigation');
    assert.equal(f.parses,1,'unchanged admitted search must not decode again after navigation');
    assert.equal(f.box.hidden,false);f.key('Enter');assert.equal(f.navigated.at(-1),'/api/Fuzz/Case_A');
  } finally {next.abort();f.restore();}
});

test('aborted search admission cannot populate the next page cache',async()=>{
  const f=await lazySearchFixture(),next=new AbortController();
  try {
    f.type('Case');f.controller.abort();await f.release('success');
    initSearch(f.source,()=>'/unused',next.signal,()=>{});f.input.focus();f.type('Case');await f.flush();
    assert.equal(f.requested.length,2,'aborted response must not become a successful shared admission');
    assert.equal(f.parses,2);assert.equal(f.box.hidden,false);
  } finally {next.abort();f.restore();}
});

test('a changed search digest replaces the one admitted document entry',async()=>{
  const f=await lazySearchFixture(),scopes:AbortController[]=[];
  try {
    f.type('Case');await f.release('success');f.controller.abort();
    const changed=f.source.replace('a'.repeat(64),'b'.repeat(64));
    for(const source of [changed,f.source]) {
      const scope=new AbortController();scopes.push(scope);
      initSearch(source,()=>'/unused',scope.signal,()=>{});f.input.focus();f.type('Case');await f.flush();
      assert.equal(f.box.hidden,false);scope.abort();
    }
    assert.deepEqual(f.requested,[f.source,changed,f.source],'cache must retain one source identity, not every visited digest');
    assert.equal(f.parses,3);
  } finally {for(const scope of scopes)scope.abort();f.restore();}
});


for(const failure of ['error','invalid'] as const)test('failed search admission '+failure+' is not retained for a new page',async()=>{
  const f=await lazySearchFixture(),next=new AbortController();
  try {
    f.type('Case');await f.release(failure);assert.equal(f.parses,failure==='error'?0:1);f.controller.abort();
    Object.defineProperty(globalThis,'fetch',{configurable:true,value:(url:string)=>{f.requested.push(url);return Promise.resolve(new Response(JSON.stringify({payload:'search',data:{schemaVersion:1,entries:[{...entry('Recovered'),preview:null}]}})));}});
    initSearch(f.source,target=>target.kind==='page'?'/'+target.id:'/unused',next.signal,url=>f.navigated.push(url));
    f.input.focus();f.type('Recovered');await f.flush();
    assert.deepEqual(f.requested,[f.source,f.source]);assert.equal(f.box.hidden,false);f.key('Enter');assert.equal(f.navigated.at(-1),'/api/Fuzz/Recovered');
  } finally {next.abort();f.restore();}
});
