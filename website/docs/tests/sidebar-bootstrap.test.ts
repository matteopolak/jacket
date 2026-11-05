import assert from 'node:assert/strict';
import vm from 'node:vm';
import test from 'node:test';
import {page,Document,Element,sidebar,script} from './helpers/sidebar.mjs';
import {setupControls} from '../src/browser/controls.ts';
test('initial parser restoration never resets controls the reader already changed',()=>{
  const p=page(false);p.initialize();assert.equal(p.side.open,false);assert.equal(p.side.children[0]!.open,true);
  p.side.open=true;p.side.children[0]!.open=false;
  p.observers[0]!.pending=[{addedNodes:[p.side]}];p.window.jacketFinishSidebar();
  assert.equal(p.side.open,true);assert.equal(p.side.children[0]!.open,false);assert.ok(p.observers[0]!.disconnected);
});
test('swap prepares incoming theme/groups from freshly saved state and restores before paint',()=>{
  const p=page(true);p.initialize();p.window.jacketFinishSidebar();p.side.scrollTop=73;
  p.side.children[0]!.open=false;p.side.children[1]!.open=true;
  const incoming=new Document();incoming.side=sidebar();
  p.document.dispatchEvent(Object.assign(new Event('astro:before-swap'),{newDocument:incoming}));
  assert.equal(incoming.documentElement.getAttribute('data-theme'),'dark');assert.equal(incoming.side.children[0]!.open,false);assert.equal(incoming.side.children[1]!.open,true);
  p.document.side=incoming.side;p.document.dispatchEvent(new Event('astro:after-swap'));
  assert.equal(incoming.side.scrollTop,73);
  p.side.dispatchEvent(new Event('wheel'));assert.equal(p.side.dataset['readerMoved'],undefined,'old page reader listeners were aborted');
});
test('rerunning the authored bootstrap does not accumulate lifecycle handlers or observers',()=>{
  const p=page(true);p.initialize();vm.runInNewContext(script!,p.context);
  assert.equal(p.observers.length,1);assert.equal(p.document.listeners['astro:before-swap'],1);assert.equal(p.document.listeners['astro:after-swap'],1);
});

for(const blockedRead of [true,false])test(`sidebar survives failed storage writes${blockedRead?' and blocked reads':' with stale readable data'}`,()=>{
  const p=page(true,'modules',context=>{
    if(blockedRead)context.sessionStorage.getItem=()=>{throw new Error('blocked');};
    context.sessionStorage.setItem=()=>{throw new Error('quota');};
  });
  p.initialize();p.window.jacketFinishSidebar();p.side.scrollTop=117;p.side.children[0]!.open=false;p.side.children[1]!.open=true;
  const incoming=new Document();incoming.side=sidebar();
  p.document.dispatchEvent(Object.assign(new Event('astro:before-swap'),{newDocument:incoming}));
  assert.equal(incoming.side.children[0]!.open,false);assert.equal(incoming.side.children[1]!.open,true);
  p.document.side=incoming.side;p.document.dispatchEvent(new Event('astro:after-swap'));assert.equal(incoming.side.scrollTop,117);
  p.side.dispatchEvent(new Event('wheel'));assert.equal(p.side.dataset['readerMoved'],undefined);
  vm.runInNewContext(script!,p.context);assert.equal(p.document.listeners['astro:before-swap'],1);assert.equal(p.document.listeners['astro:after-swap'],1);
});
test('fresh stored sidebar data remains authoritative after ordinary successful storage',()=>{
  const p=page(true);p.initialize();p.window.jacketFinishSidebar();
  p.storage.set('jacket-docs-side',JSON.stringify({k:'modules',o:['Other'],t:88}));
  const incoming=new Document();incoming.side=sidebar();p.window.jacketBootstrap.prepare(incoming);
  assert.equal(incoming.side.children[0]!.open,false);assert.equal(incoming.side.children[1]!.open,true);
  p.document.side=incoming.side;p.window.jacketFinishSidebar();assert.equal(incoming.side.scrollTop,88);
});
test('failed theme writes preserve the actual control choice across swaps but fresh stored changes still win',()=>{
  const p=page(true,'modules',context=>{context.localStorage.setItem=()=>{throw new Error('quota');};context.localStorage.removeItem=()=>{throw new Error('quota');};});
  p.initialize();p.window.jacketFinishSidebar();const button=new Element('button'),query=p.document.querySelector.bind(p.document);
  p.document.querySelector=selector=>selector==='#theme-toggle'?button:query(selector);
  const values={document:p.document,localStorage:p.context.localStorage},descriptors=new Map(Object.keys(values).map(key=>[key,Object.getOwnPropertyDescriptor(globalThis,key)])),controller=new AbortController();
  for(const [key,value]of Object.entries(values))Object.defineProperty(globalThis,key,{value,writable:true,configurable:true});
  try{
    setupControls(controller.signal);setupControls(controller.signal);button.dispatchEvent(new Event('click'));
    assert.equal(p.document.documentElement.getAttribute('data-theme'),'system','control choice advanced more than once');
    for(let swap=0;swap<2;swap++){const incoming=new Document();incoming.side=sidebar();p.document.dispatchEvent(Object.assign(new Event('astro:before-swap'),{newDocument:incoming}));assert.equal(incoming.documentElement.getAttribute('data-theme'),'system','stale readable storage erased manual choice');p.document.side=incoming.side;p.document.dispatchEvent(new Event('astro:after-swap'));}
    button.dispatchEvent(new Event('click'));assert.equal(p.document.documentElement.getAttribute('data-theme'),'light');
    const chosen=new Document();chosen.side=sidebar();p.document.dispatchEvent(Object.assign(new Event('astro:before-swap'),{newDocument:chosen}));assert.equal(chosen.documentElement.getAttribute('data-theme'),'light','failed setItem erased manual light choice');
    p.context.localStorage.getItem=()=>null;const fresh=new Document();fresh.side=sidebar();p.window.jacketBootstrap.prepare(fresh);assert.equal(fresh.documentElement.getAttribute('data-theme'),'system','fresh stored theme change was ignored');
  }finally{controller.abort();for(const [key,descriptor]of descriptors)if(descriptor)Object.defineProperty(globalThis,key,descriptor);else Reflect.deleteProperty(globalThis,key);}
});

test('pending sidebar memory yields to changed stored data and successful writes recover normally',()=>{
  const p=page(true,'modules',context=>{context.sessionStorage.setItem=()=>{throw new Error('quota');};});p.initialize();p.window.jacketFinishSidebar();
  p.side.scrollTop=117;p.side.children[0]!.open=false;p.side.children[1]!.open=true;
  const first=new Document();first.side=sidebar();p.document.dispatchEvent(Object.assign(new Event('astro:before-swap'),{newDocument:first}));
  p.storage.set('jacket-docs-side',JSON.stringify({k:'modules',o:['Core'],t:224}));const fresh=new Document();fresh.side=sidebar();p.window.jacketBootstrap.prepare(fresh);
  assert.equal(fresh.side.children[0]!.open,true);assert.equal(fresh.side.children[1]!.open,false);p.document.side=fresh.side;p.window.jacketFinishSidebar();assert.equal(fresh.side.scrollTop,224);
  p.context.sessionStorage.setItem=(key,value)=>p.storage.set(key,value);fresh.side.scrollTop=301;fresh.side.children[0]!.open=false;fresh.side.children[1]!.open=true;
  const recovered=new Document();recovered.side=sidebar();p.document.dispatchEvent(Object.assign(new Event('astro:before-swap'),{newDocument:recovered}));p.document.side=recovered.side;p.document.dispatchEvent(new Event('astro:after-swap'));
  assert.equal(recovered.side.scrollTop,301);assert.equal(recovered.side.children[1]!.open,true);assert.equal(JSON.parse(p.storage.get('jacket-docs-side')!).t,301);
});


test('actual fractional and wrapped header border boxes become the synchronous rail offset',()=>{
  const p=page(true);p.document.header.borderHeight=62.800003;p.window.jacketFinishSidebar();
  assert.equal(p.document.documentElement.style.getPropertyValue('--header-offset'),'62.800003px');
  p.document.header.borderHeight=107.25;p.window.jacketFinishLayout();
  assert.equal(p.document.documentElement.style.getPropertyValue('--header-offset'),'107.25px','wrapped header height must replace the nominal fallback');
});
test('incoming documents carry the measured offset and retarget one observer before paint',()=>{
  const p=page(true);p.document.header.borderHeight=62.8;p.window.jacketFinishLayout();
  const incoming=new Document();incoming.side=sidebar();incoming.header.borderHeight=114.5;
  p.document.dispatchEvent(Object.assign(new Event('astro:before-swap'),{newDocument:incoming}));
  assert.equal(incoming.documentElement.style.getPropertyValue('--header-offset'),'62.8px');
  const old=p.document.header;p.document.header=incoming.header;p.document.documentElement=incoming.documentElement;p.document.side=incoming.side;
  p.document.dispatchEvent(new Event('astro:after-swap'));
  assert.equal(p.document.documentElement.style.getPropertyValue('--header-offset'),'114.5px');
  assert.equal(p.headerObservers.length,1);assert.deepEqual([...p.headerObservers[0]!.targets],[incoming.header]);assert(!p.headerObservers[0]!.targets.has(old));
  vm.runInNewContext(script!,p.context);assert.equal(p.headerObservers.length,1);assert.equal(p.windowListeners['resize'],1);
});
test('window resize and header resize update the same measured authority without scroll listeners',()=>{
  const p=page(true);p.document.header.borderHeight=62.8;p.window.jacketFinishLayout();
  p.document.header.borderHeight=89.5;p.window.dispatchEvent(new Event('resize'));
  assert.equal(p.document.documentElement.style.getPropertyValue('--header-offset'),'89.5px');
  p.document.header.borderHeight=121.75;p.headerObservers[0]!.callback([]);
  assert.equal(p.document.documentElement.style.getPropertyValue('--header-offset'),'121.75px');
  assert.equal(p.headerObservers.length,1);assert.equal(p.document.listeners['scroll'],undefined);assert.equal(p.windowListeners['scroll'],undefined);
  p.document.header.borderHeight=0;p.headerObservers[0]!.callback([]);assert.equal(p.document.documentElement.style.getPropertyValue('--header-offset'),'121.75px','a detached/hidden zero box cannot erase the last attached measurement');
});


test('same global Astro persist swap retains live rail subtree and reader geometry without restoring it',()=>{
  const p=page(true);p.initialize();p.window.jacketFinishSidebar();
  p.side.scrollTop=187;p.side.children[0]!.open=false;p.side.children[1]!.open=true;
  const nav=p.side.nav,groups=[...p.side.children];
  const incoming=new Document();incoming.side=sidebar();
  p.document.dispatchEvent(Object.assign(new Event('astro:before-swap'),{newDocument:incoming}));
  // Astro's supported persist transfers the original element, not the incoming placeholder.
  p.document.dispatchEvent(new Event('astro:after-swap'));
  assert.equal(p.document.side,p.side);assert.equal(p.side.nav,nav);assert.deepEqual(p.side.children,groups);
  assert.equal(p.side.scrollTop,187);assert.equal(groups[0]!.open,false);assert.equal(groups[1]!.open,true);
  const state=JSON.parse(p.storage.get('jacket-docs-side')!);assert.equal(state.t,187);
});


test('global and latest peer context return to the same cached navigation nodes and scroll positions',()=>{
  const p=page(true);p.initialize();p.window.jacketFinishSidebar();p.side.scrollTop=187;
  const global=p.side.nav;
  const swap=(key:string)=>{const incoming=new Document();incoming.side=sidebar(key);p.document.dispatchEvent(Object.assign(new Event('astro:before-swap'),{newDocument:incoming}));p.document.dispatchEvent(new Event('astro:after-swap'));};
  swap('Basic module');const peer=p.side.nav;assert.notEqual(peer,global);p.side.scrollTop=63;
  swap('modules');assert.equal(p.side.nav,global);assert.equal(p.side.scrollTop,187);
  swap('Basic module');assert.equal(p.side.nav,peer);assert.equal(p.side.scrollTop,63);
});


test('a retained mobile rail exposes desktop navigation and remembers the mobile disclosure choice',()=>{
  const p=page(false);p.document.header.borderHeight=62.8;p.initialize();p.window.jacketFinishSidebar();assert.equal(p.side.open,false);
  p.context.matchMedia=()=>({matches:true});p.window.dispatchEvent(new Event('resize'));assert.equal(p.side.open,true,'desktop navigation remained inside a closed details');
  p.context.matchMedia=()=>({matches:false});p.window.dispatchEvent(new Event('resize'));assert.equal(p.side.open,false);
  p.side.open=true;p.context.matchMedia=()=>({matches:true});p.window.dispatchEvent(new Event('resize'));
  p.context.matchMedia=()=>({matches:false});p.window.dispatchEvent(new Event('resize'));assert.equal(p.side.open,true,'mobile reader disclosure choice was discarded');
  assert.equal(p.windowListeners['resize'],1);
});

test('full-document restoration is native and instant before deferred router, then returns to framework ownership',()=>{
  const p=page(true,'modules',context=>{
    Object.assign(context.window,{history:{scrollRestoration:'manual'}});
  });
  const history=(p.window as typeof p.window & {history:{scrollRestoration:string}}).history;
  assert.equal(history.scrollRestoration,'auto');
  assert.equal(p.document.documentElement.getAttribute('data-doc-restoring'),'');
  p.document.dispatchEvent(new Event('astro:page-load'));
  assert.equal(history.scrollRestoration,'manual');
  assert.equal(p.document.documentElement.getAttribute('data-doc-restoring'),null);
  const traverse=new Document();
  p.document.dispatchEvent(Object.assign(new Event('astro:before-swap'),{newDocument:traverse,navigationType:'traverse'}));
  assert.equal(traverse.documentElement.getAttribute('data-doc-restoring'),'','instant history CSS survives root attribute swap');
  const push=new Document();
  p.document.dispatchEvent(Object.assign(new Event('astro:before-swap'),{newDocument:push,navigationType:'push'}));
  assert.equal(push.documentElement.getAttribute('data-doc-restoring'),null,'ordinary links retain smooth hash behavior');
  p.window.dispatchEvent(new Event('pagehide'));
  assert.equal(history.scrollRestoration,'auto','departing document lets native browser restore its history entry');
  p.document.documentElement.setAttribute('data-doc-restoring','');
  p.window.dispatchEvent(Object.assign(new Event('pageshow'),{persisted:true}));
  assert.equal(history.scrollRestoration,'manual');
  assert.equal(p.document.documentElement.getAttribute('data-doc-restoring'),null);
});
