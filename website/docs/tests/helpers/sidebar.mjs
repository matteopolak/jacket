/** Inert DOM fixture for the exact authored production prepaint bootstrap. */
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import vm from 'node:vm';
export const script=await readFile(new URL('../../src/browser/bootstrap.js',import.meta.url),'utf8');assert.ok(script);
export class Element extends EventTarget {
  open=true;scrollTop=0;borderHeight=0;style={values:new Map(),setProperty(name,value){this.values.set(name,value);},getPropertyValue(name){return this.values.get(name)??'';}};
  /** @type {Record<string,string>} */ dataset={};
  /** @type {Record<string,string>} */ attrs={};
  children=[];nav;summary;parentSide;classList={add(){}};
  constructor(kind){super();this.kind=kind;}
  getBoundingClientRect(){return {height:this.borderHeight};}
  matches(selector){return selector===`details.${this.kind}`;}
  setAttribute(name,value){this.attrs[name]=value;}
  removeAttribute(name){delete this.attrs[name];}
  contains(node){return node===this||node===this.nav||this.children.some(child=>child.contains(node));}
  getAttribute(name){return this.attrs[name]??null;}
  closest(){return this.nav??null;}
  querySelector(selector){return selector==='summary'?this.summary??null:selector.startsWith('nav')?this.nav??null:null;}
  replaceWith(node){if(!this.parentSide)throw Error('fixture node has no parent rail');this.parentSide.nav=node;node.parentSide=this.parentSide;}
  querySelectorAll(selector){return this.children.filter(child=>child.kind==='nav-cat'&&(!selector.includes('[open]')||child.open));}
}
export class Document extends EventTarget {
  documentElement=new Element('root');side;header=new Element('top');
  /** @type {Record<string,number>} */ listeners={};
  addEventListener(type,callback,options){this.listeners[type]=(this.listeners[type]??0)+1;super.addEventListener(type,callback,options);}
  querySelector(selector){return selector==='details.side'?this.side??null:selector==='.top'?this.header:null;}
  querySelectorAll(selector){return selector==='details.side'?this.side?[this.side]:[]:selector==='details.nav-cat'?this.side?.children??[]:[];}
}
export function sidebar(key='modules'){
  const side=new Element('side'),nav=new Element('nav');nav.attrs['data-nav']=key;side.nav=nav;nav.parentSide=side;side.summary=new Element('summary');side.summary.textContent=key;
  for(const key of ['Core','Other']){const group=new Element('nav-cat');group.open=false;group.attrs['data-key']=key;group.nav=nav;side.children.push(group);}
  return side;
}
export function page(wide,key='modules',configure=context=>void context){
  const observers=[],headerObservers=[];
  class HeaderObserver {
    targets=new Set();disconnects=0;constructor(callback){this.callback=callback;headerObservers.push(this);}
    observe(target){this.targets.add(target);}disconnect(){this.targets.clear();this.disconnects++;}
  }
  class Observer {
    pending=[];disconnected=false;
    constructor(callback){this.callback=callback;observers.push(this);}
    observe(){}takeRecords(){const records=this.pending;this.pending=[];return records;}disconnect(){this.disconnected=true;}
  }
  const document=new Document(),storage=new Map([['jacket-docs-side',JSON.stringify({k:'modules',o:['Core'],t:19})]]),window=/** @type {EventTarget & {jacketFinishSidebar:()=>void,jacketFinishLayout:()=>void,jacketBootstrap:{prepare:(doc:Document)=>void}}} */(new EventTarget());
  /** @type {Record<string,number>} */
  const windowListeners={};const addWindowListener=window.addEventListener.bind(window);window.addEventListener=(type,callback,options)=>{windowListeners[type]=(windowListeners[type]??0)+1;addWindowListener(type,callback,options);};
  const context={document,window,Element,MutationObserver:Observer,ResizeObserver:HeaderObserver,AbortController,matchMedia:()=>({matches:wide}),
    sessionStorage:{getItem:key=>storage.get(key)??null,setItem:(key,value)=>storage.set(key,value)},localStorage:{getItem:()=> /** @type {string|null} */('dark'),setItem:()=>{},removeItem:()=>{}},innerHeight:800};
  configure(context);
  vm.runInNewContext(script,context);
  const side=sidebar(key);document.side=side;
  const group=name=>side.children.find(node=>node.getAttribute('data-key')===name);
  return{context,document,window,windowListeners,observers,headerObservers,observer:observers[0],side,group,storage,initialize:()=>observers[0].callback([{addedNodes:[side]}])};
}
