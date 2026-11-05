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
