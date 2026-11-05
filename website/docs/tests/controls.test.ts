import assert from 'node:assert/strict';
import { test } from 'node:test';
import { nextTheme, setupControls, versionEntries } from '../src/browser/controls.ts';

test('theme control cycles system, light, dark and recovers an unknown stored state', () => {
  assert.equal(nextTheme('system'), 'light');
  assert.equal(nextTheme('light'), 'dark');
  assert.equal(nextTheme('dark'), 'system');
  assert.equal(nextTheme('unknown'), 'system');
});

test('version destinations resolve relative to the manifest and reject executable URLs', () => {
  const source = new URL('https://example.com/docs/versions.json');
  assert.deepEqual(versionEntries([{ version: 'current', url: './' }, { version: 'archive', url: 'versions/archive/' }], source), [
    { version: 'current', url: 'https://example.com/docs/' },
    { version: 'archive', url: 'https://example.com/docs/versions/archive/' },
  ]);
  for (const value of [null, {}, [{ version: 'bad', url: 'javascript:alert(1)' }], [{ version: 'bad', url: '//name:secret@example.com/' }], [{ version: 1, url: '/' }]]) {
    assert.throws(() => versionEntries(value, source));
  }
});


test('real version menu anchors request native navigation across documentation versions',async()=>{
  class Element extends EventTarget {
    isConnected=true;attrs=new Map<string,string>();textContent='';href='';tabIndex=0;hidden=false;disabled=false;children:Element[]=[];
    setAttribute(name:string,value:string):void{this.attrs.set(name,value);}
    getAttribute(name:string):string|null{return this.attrs.get(name)??null;}
    replaceChildren():void{this.children=[];}
    querySelectorAll():Element[]{return this.children;}
    focus():void{}
    contains(node:unknown):boolean{return node===this||this.children.includes(node as Element);}
    appendChild(node:Element):Element{this.children.push(node);return node;}
  }
  const button=new Element(),menu=new Element();button.textContent='current';
  const document=Object.assign(new EventTarget(),{body:{dataset:{base:'/docs/'}},querySelector:(selector:string)=>selector==='#ver-button'?button:selector==='#ver-menu'?menu:null,createElement:()=>new Element()});
  let requests=0;let requested:RequestInit|undefined;
  const values={document,Node:Element,location:new URL('https://example.test/docs/'),fetch:async(_url:unknown,options?:RequestInit)=>{requests++;requested=options;return {ok:true,json:async()=>[{version:'current',url:'./'},{version:'archive',url:'archive/'}]};}};
  const descriptors=new Map(Object.keys(values).map(key=>[key,Object.getOwnPropertyDescriptor(globalThis,key)])),scope=new AbortController();
  for(const [key,value]of Object.entries(values))Object.defineProperty(globalThis,key,{value,writable:true,configurable:true});
  try {
    setupControls(scope.signal);await new Promise<void>(resolve=>setImmediate(resolve));
    assert.equal(requests,0,'the version catalogue must not load before intent');
    button.dispatchEvent(new Event('pointerenter'));await new Promise<void>(resolve=>setImmediate(resolve));
    assert.equal(requests,1);assert.equal(requested?.cache,'no-cache');assert.ok(requested?.signal);
    button.dispatchEvent(new Event('focus'));assert.equal(requests,1,'repeat intent must share the first request');
    assert.equal(menu.children.length,2);for(const anchor of menu.children)assert.equal(anchor.getAttribute('data-astro-reload'),'');
    assert.deepEqual(menu.children.map(anchor=>anchor.href),['https://example.test/docs/','https://example.test/docs/archive/']);
  }finally{scope.abort();for(const [key,descriptor]of descriptors)if(descriptor)Object.defineProperty(globalThis,key,descriptor);else Reflect.deleteProperty(globalThis,key);}
});


for(const outcome of ['offline','http','single','aborted']as const)test(`archive intent remains disabled and closed after ${outcome}`,async()=>{
  class Element extends EventTarget {
    isConnected=true;textContent='current';hidden=true;disabled=false;children:Element[]=[];attributes=new Map<string,string>();
    setAttribute(key:string,value:string):void{this.attributes.set(key,value);}replaceChildren():void{this.children=[];}
    appendChild(node:Element):void{this.children.push(node);}querySelectorAll():Element[]{return this.children;}focus():void{}contains(node:unknown):boolean{return node===this||this.children.includes(node as Element);}
  }
  const button=new Element(),menu=new Element(),controller=new AbortController();let requests=0;let requestSignal:AbortSignal|undefined;
  const document=Object.assign(new EventTarget(),{body:{dataset:{base:'/docs/'}},querySelector:(selector:string)=>selector==='#ver-button'?button:selector==='#ver-menu'?menu:null,createElement:()=>new Element()});
  const values={document,Node:Element,location:new URL('https://docs.test/'),fetch:async(_url:unknown,options?:RequestInit)=>{requests++;requestSignal=options?.signal??undefined;if(outcome==='offline')throw new TypeError('offline');if(outcome==='aborted')controller.abort();return {ok:outcome!=='http',json:async()=>[{version:'current',url:'./'}]};}};
  const descriptors=new Map(Object.keys(values).map(key=>[key,Object.getOwnPropertyDescriptor(globalThis,key)]));
  for(const [key,value]of Object.entries(values))Object.defineProperty(globalThis,key,{configurable:true,value});
  try{setupControls(controller.signal);assert.equal(requests,0);button.dispatchEvent(new Event('focus'));await new Promise<void>(resolve=>setImmediate(resolve));assert.equal(requests,1);assert.equal(button.disabled,outcome!=='aborted');assert.equal(menu.hidden,true);assert.equal(menu.children.length,0);if(outcome==='aborted')assert.equal(requestSignal?.aborted,true);}
  finally{controller.abort();for(const [key,descriptor]of descriptors)if(descriptor)Object.defineProperty(globalThis,key,descriptor);else Reflect.deleteProperty(globalThis,key);}
});


test('first archive activation remains available while an intent request is pending',async()=>{
  class Element extends EventTarget {
    isConnected=true;textContent='current';hidden=true;disabled=false;children:Element[]=[];attrs=new Map<string,string>();
    setAttribute(key:string,value:string):void{this.attrs.set(key,value);}replaceChildren():void{this.children=[];}
    appendChild(node:Element):void{this.children.push(node);}querySelectorAll():Element[]{return this.children;}focus():void{}contains(node:unknown):boolean{return node===this||this.children.includes(node as Element);}
  }
  const button=new Element(),menu=new Element(),controller=new AbortController();let resolveResponse:(value:unknown)=>void=()=>{};
  const document=Object.assign(new EventTarget(),{body:{dataset:{base:'/docs/'}},querySelector:(selector:string)=>selector==='#ver-button'?button:selector==='#ver-menu'?menu:null,createElement:()=>new Element()});
  const values={document,Node:Element,location:new URL('https://docs.test/'),fetch:()=>new Promise(resolve=>{resolveResponse=resolve;})};
  const descriptors=new Map(Object.keys(values).map(key=>[key,Object.getOwnPropertyDescriptor(globalThis,key)]));
  for(const [key,value]of Object.entries(values))Object.defineProperty(globalThis,key,{configurable:true,value});
  try{setupControls(controller.signal);button.dispatchEvent(new Event('pointerenter'));assert.equal(button.disabled,false,'hover must not disable the first trusted click while the archive request is pending');button.dispatchEvent(new Event('click'));resolveResponse({ok:true,json:async()=>[{version:'current',url:'./'},{version:'archive',url:'archive/'}]});await new Promise<void>(resolve=>setImmediate(resolve));assert.equal(menu.hidden,false);assert.equal(menu.children.length,2);button.dispatchEvent(Object.assign(new Event('focusout'),{relatedTarget:menu.children[0]}));assert.equal(menu.hidden,false,'focus entering the menu is still allowed');menu.dispatchEvent(Object.assign(new Event('focusout'),{relatedTarget:button}));assert.equal(menu.hidden,false,'focus returning to the menu button is still allowed');}
  finally{controller.abort();for(const [key,descriptor]of descriptors)if(descriptor)Object.defineProperty(globalThis,key,descriptor);else Reflect.deleteProperty(globalThis,key);}
});


test('an aborted archive completion cannot disable the retained replacement control',async()=>{
  class Element extends EventTarget {isConnected=true;textContent='current';hidden=true;disabled=false;setAttribute():void{}replaceChildren():void{}}
  const button=new Element(),menu=new Element(),first=new AbortController(),second=new AbortController();let resolveResponse:(value:unknown)=>void=()=>{};
  const document=Object.assign(new EventTarget(),{body:{dataset:{base:'/docs/'}},querySelector:(selector:string)=>selector==='#ver-button'?button:selector==='#ver-menu'?menu:null});
  const values={document,Node:Element,location:new URL('https://docs.test/'),fetch:()=>new Promise(resolve=>{resolveResponse=resolve;})};
  const descriptors=new Map(Object.keys(values).map(key=>[key,Object.getOwnPropertyDescriptor(globalThis,key)]));
  for(const [key,value]of Object.entries(values))Object.defineProperty(globalThis,key,{configurable:true,value});
  try{setupControls(first.signal);button.dispatchEvent(new Event('pointerenter'));first.abort();setupControls(second.signal);resolveResponse({ok:false});await new Promise<void>(resolve=>setImmediate(resolve));assert.equal(button.disabled,false,'discarded page completion changed the retained fresh version control');}
  finally{first.abort();second.abort();for(const [key,descriptor]of descriptors)if(descriptor)Object.defineProperty(globalThis,key,descriptor);else Reflect.deleteProperty(globalThis,key);}
});

for(const dismissal of ['escape','outside','button','keyboard','space','focus','pageabort']as const)test(`dismissal by ${dismissal} cancels a pending archive opening`,async()=>{
  let focused=0;
  class Element extends EventTarget {
    isConnected=true;textContent='current';hidden=true;disabled=false;children:Element[]=[];attrs=new Map<string,string>();
    setAttribute(key:string,value:string):void{this.attrs.set(key,value);}replaceChildren():void{this.children=[];}
    appendChild(node:Element):void{this.children.push(node);}querySelectorAll():Element[]{return this.children;}focus():void{focused++;}
    contains(node:unknown):boolean{return node===this||this.children.includes(node as Element);}
  }
  const button=new Element(),menu=new Element(),controller=new AbortController();let resolveResponse:(value:unknown)=>void=()=>{};
  const document=Object.assign(new EventTarget(),{body:{dataset:{base:'/docs/'}},querySelector:(selector:string)=>selector==='#ver-button'?button:selector==='#ver-menu'?menu:null,createElement:()=>new Element()});
  const values={document,Node:Element,location:new URL('https://docs.test/'),fetch:()=>new Promise(resolve=>{resolveResponse=resolve;})};
  const descriptors=new Map(Object.keys(values).map(key=>[key,Object.getOwnPropertyDescriptor(globalThis,key)]));
  for(const [key,value]of Object.entries(values))Object.defineProperty(globalThis,key,{configurable:true,value});
  try{
    setupControls(controller.signal);button.dispatchEvent(new Event('click'));
    if(dismissal==='button')button.dispatchEvent(new Event('click'));
    else if(dismissal==='keyboard'||dismissal==='space')button.dispatchEvent(Object.assign(new Event('keydown'),{key:dismissal==='space'?' ':'Enter'}));
    else if(dismissal==='focus')button.dispatchEvent(Object.assign(new Event('focusout'),{relatedTarget:new Element()}));
    else if(dismissal==='pageabort')controller.abort();
    else document.dispatchEvent(dismissal==='escape'?Object.assign(new Event('keydown'),{key:'Escape'}):Object.defineProperty(new Event('click'),'target',{value:new Element()}));
    resolveResponse({ok:true,json:async()=>[{version:'current',url:'./'},{version:'archive',url:'archive/'}]});await new Promise<void>(resolve=>setImmediate(resolve));
    assert.equal(menu.hidden,true,'an obsolete opening intent must not reopen the menu');assert.equal(button.attrs.get('aria-expanded'),'false');assert.equal(focused,0,'an obsolete request must not steal focus after dismissal');
  }finally{controller.abort();for(const [key,descriptor]of descriptors)if(descriptor)Object.defineProperty(globalThis,key,descriptor);else Reflect.deleteProperty(globalThis,key);}
});
