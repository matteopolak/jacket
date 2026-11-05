import assert from 'node:assert/strict';
import test from 'node:test';
import {contentReadiness} from './doc_browser_content.mjs';

function fixture(run){
 const saved=new Map(),node={textContent:'Destination',checkVisibility:()=>true,getBoundingClientRect:()=>({left:10,top:10,right:210,bottom:50,width:200,height:40}),getAnimations:()=>[],parentElement:null,contains:other=>other===node};
 const values={performance:{now:()=>10,timeOrigin:1,getEntriesByType:()=>[{name:'first-contentful-paint'}]},document:{activeElement:node,activeViewTransition:null,querySelector:()=>node,getElementsByTagName:()=>[node],elementFromPoint:()=>node,fonts:{status:'loaded'}},location:{pathname:'/destination',hash:''},scrollY:0,innerHeight:900,innerWidth:1280,getComputedStyle:()=>({opacity:'1',visibility:'visible'})};
 for(const [key,value]of Object.entries(values)){saved.set(key,Object.getOwnPropertyDescriptor(globalThis,key));Object.defineProperty(globalThis,key,{value,writable:true,configurable:true});}
 try{return run(node,values.document);}finally{for(const [key,value]of saved)if(value)Object.defineProperty(globalThis,key,value);else Reflect.deleteProperty(globalThis,key);}
}
const expected={selector:'main h1',path:'/destination',hash:'',text:'Destination'};
test('new DOM behind an active native capture is not useful content yet',()=>fixture((node,document)=>{
 document.activeViewTransition={};const state=contentReadiness(expected);assert.equal(state.domReady,true);assert.equal(state.ready,false);
}));
test('uncovered destination is useful only when its visible hit target is live',()=>fixture((node,document)=>{
 document.elementFromPoint=()=>({});assert.equal(contentReadiness(expected).ready,false);
 document.elementFromPoint=()=>node;assert.equal(contentReadiness(expected).ready,true);
}));
test('ongoing ancestor animation is distinct from DOM arrival',()=>fixture((node)=>{
 node.parentElement={getAnimations:()=>[{playState:'running',effect:{getComputedTiming:()=>({endTime:170})}}],parentElement:null};
 const state=contentReadiness(expected);assert.equal(state.domReady,true);assert.equal(state.ready,false);
}));

test('an initial DOM predicate cannot precede the browser first contentful paint',()=>fixture(()=>{
 performance.getEntriesByType=()=>[];assert.equal(contentReadiness(expected).ready,false);
}));
