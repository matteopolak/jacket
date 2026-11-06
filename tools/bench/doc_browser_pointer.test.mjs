import assert from 'node:assert/strict';
import {test} from 'node:test';
import vm from 'node:vm';
import {pointerProbe} from './doc_browser_pointer.mjs';

function environment(frames){
 let index=0,time=0,scrollCalls=0;
 const node={isConnected:true,scrollIntoView(options){assert.deepEqual(JSON.parse(JSON.stringify(options)),{block:'center'});scrollCalls++;},contains(hit){return hit===this;},getBoundingClientRect(){return frames[Math.min(index,frames.length-1)].rect;}};
 const context={document:{querySelector:()=>node,elementFromPoint:()=>frames[Math.min(index,frames.length-1)].hit?node:null},innerWidth:1280,innerHeight:900,performance:{now:()=>time},requestAnimationFrame(callback){index++;time+=16;queueMicrotask(callback);}};
 return{run:()=>vm.runInNewContext(`(${pointerProbe.toString()})('a',100)`,context),calls:()=>scrollCalls};
}
const rect=y=>({x:100,y,width:40,height:20});
test('waits for smooth scroll and stable hit-testable geometry before dispatch point',async()=>{
 const fixture=environment([{rect:rect(6000),hit:false},{rect:rect(500),hit:true},{rect:rect(460),hit:true},{rect:rect(450),hit:true},{rect:rect(450),hit:true},{rect:rect(450),hit:true}]);
 const value=await fixture.run();assert.equal(value.y,460);assert.equal(value.x,120);assert.equal(fixture.calls(),1);assert(value.frames>=5);
});
test('does not accept a stable covered or clipped target',async()=>{
 const fixture=environment([{rect:rect(450),hit:false}]);await assert.rejects(fixture.run(),/pointer-ready deadline/);assert.equal(fixture.calls(),1);
});
test('does not accept offscreen target even if a fabricated hit claims it',async()=>{
 const fixture=environment([{rect:rect(6000),hit:true}]);await assert.rejects(fixture.run(),/pointer-ready deadline/);
});
test('propagates genuine DOM failures',async()=>{
 const context={document:{querySelector(){throw Error('actual DOM failure');}}};await assert.rejects(vm.runInNewContext(`(${pointerProbe.toString()})('a',100)`,context),/actual DOM failure/);
});
