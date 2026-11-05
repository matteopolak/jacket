#!/usr/bin/env node
/** Real production-editor regression: source preservation, safe color spans and reset links.
 * node tools/dev/doc_edit_coloring.mjs http://127.0.0.1:8000/ OUTPUT_JSON
 */
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {mkdtemp,rm,writeFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {setTimeout as delay} from 'node:timers/promises';
import {Connection,runtimeException} from './chrome_cdp.mjs';
import {findChrome,waitForDevToolsPort} from './chrome_startup.mjs';
const base=new URL(process.argv[2]),output=process.argv[3];assert(output);
const chrome=findChrome();
const profile=await mkdtemp(path.join(tmpdir(),'jacket-edit-color-'));
const child=spawn(chrome,['--headless=new','--no-sandbox','--disable-gpu','--no-first-run','--remote-debugging-port=0',`--user-data-dir=${profile}`,'about:blank'],{stdio:['ignore','ignore','pipe']});
const overlayProbe=String.raw`(()=>{
 const figure=document.querySelector('figure:has(.ed textarea)'),area=figure.querySelector('textarea'),code=figure.querySelector('pre code'),host=area.parentElement;
 const style=getComputedStyle(area),mirror=document.createElement('div');
 mirror.style.cssText='position:absolute;inset:0;visibility:hidden;pointer-events:none;overflow:hidden;';
 for(const property of ['font-family','font-size','font-weight','font-style','font-stretch','line-height','letter-spacing','word-spacing','text-indent','tab-size','padding-top','padding-right','padding-bottom','padding-left','border-top-width','border-right-width','border-bottom-width','border-left-width','border-style','box-sizing','white-space','width','height','transform','transform-origin'])mirror.style.setProperty(property,style.getPropertyValue(property));
 mirror.textContent=area.value;host.append(mirror);mirror.scrollLeft=area.scrollLeft;mirror.scrollTop=area.scrollTop;
 const leaves=root=>{const walker=document.createTreeWalker(root,NodeFilter.SHOW_TEXT),nodes=[];for(let node=walker.nextNode();node;node=walker.nextNode())nodes.push(node);return nodes;};
 const rect=(nodes,offset)=>{let start=0;for(const node of nodes){if(offset<start+node.length){const range=document.createRange();range.setStart(node,offset-start);range.setEnd(node,offset-start+1);const r=range.getBoundingClientRect();return{x:r.x,y:r.y,width:r.width,height:r.height};}start+=node.length;}throw new Error('editor offset outside rendered text');};
 const actual=leaves(code),expected=leaves(mirror),text=area.value,problems=[];let checked=0,maxX=0,maxY=0;
 try{
  if(code.textContent!==area.value&&!(area.value.endsWith('\n')&&code.textContent===area.value+' '))throw new Error('editor overlay source differs from textarea beyond the terminal newline caret spacer');
  for(let offset=0;offset<text.length;offset++){
   if(/\s/u.test(text[offset]))continue;
   const a=rect(actual,offset),b=rect(expected,offset),dx=Math.abs(a.x-b.x),dy=Math.abs(a.y-b.y);checked++;maxX=Math.max(maxX,dx);maxY=Math.max(maxY,dy);
   if((dx>0.5||dy>0.5)&&problems.length<12)problems.push({offset,char:text[offset],actual:a,expected:b,dx,dy});
  }
  return{checked,maxX,maxY,problems,textareaFont:style.font,links:[...code.querySelectorAll('a')].map(a=>({text:a.textContent,weight:getComputedStyle(a).fontWeight})),transform:style.transform};
 }finally{mirror.remove();}
})()`;
let connection;const receipt={page:'api/Basic/align_forward',themes:[],exceptions:[],requests:[]};
try{
 const port=await waitForDevToolsPort(child,profile),pages=await(await fetch(`http://127.0.0.1:${port}/json/list`)).json();connection=await new Connection(pages.find(page=>page.type==='page').webSocketDebuggerUrl).open();
 connection.listeners.push(message=>{if(message.method==='Network.requestWillBeSent')receipt.requests.push({url:message.params.request.url,type:message.params.type});const error=runtimeException(message);if(error!==undefined)receipt.exceptions.push(error);});const cdp=(method,params={})=>connection.cdp(method,params);
 const evaluate=async expression=>{const response=await cdp('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!response.exceptionDetails,JSON.stringify(response.exceptionDetails));return response.result.value;};
 await cdp('Page.enable');await cdp('Runtime.enable');await cdp('Network.enable');await cdp('Emulation.setDeviceMetricsOverride',{width:1280,height:900,deviceScaleFactor:1,mobile:false});
 for(const width of [375,1280])for(const theme of ['light','dark']){
  await cdp('Emulation.setDeviceMetricsOverride',{width,height:900,deviceScaleFactor:1,mobile:false});
  await cdp('Emulation.setEmulatedMedia',{features:[{name:'prefers-color-scheme',value:theme}]});const beforeOrigin=await evaluate('performance.timeOrigin');await cdp('Page.navigate',{url:new URL(receipt.page,base).href});let ready=false;
  for(let attempt=0;attempt<600;attempt++){if(await evaluate(`performance.timeOrigin!==${beforeOrigin}&&location.pathname===${JSON.stringify(new URL(receipt.page,base).pathname)}&&document.readyState==='complete'&&!!document.querySelector('figure .ed textarea')`)){ready=true;break;}await delay(25);}assert(ready,'production editor did not initialize');
  await evaluate("document.querySelector('figure:has(.ed textarea)').scrollIntoView({block:'center'});document.fonts.ready");
  const overlayBefore=await evaluate(overlayProbe);assert.deepEqual(overlayBefore.problems,[],'original compiler-linked code does not align with the textarea');
  const original=await evaluate("(()=>{const figure=document.querySelector('figure:has(.ed textarea)');return{source:figure.querySelector('textarea').value,markup:figure.querySelector('pre code').innerHTML,links:figure.querySelectorAll('pre code a').length};})()");
  const edited='/* outer /* nested */ remains literal */\n'+original.source+'\n// </span><script>literal Ω</script>\n';
  await evaluate("(()=>{const area=document.querySelector('figure .ed textarea');area.focus();area.select();})()");await cdp('Input.insertText',{text:edited});
  const state=await evaluate("(()=>{const figure=document.querySelector('figure:has(.ed textarea)'),code=figure.querySelector('pre code');return{source:figure.querySelector('textarea').value,text:code.textContent,comments:[...code.querySelectorAll('span.c')].map(span=>span.textContent),links:code.querySelectorAll('a').length,scripts:code.querySelectorAll('script').length,resetVisible:!figure.querySelector('[data-act=reset]').hidden};})()");const overlayEdited=await evaluate(overlayProbe);assert.deepEqual(overlayEdited.problems,[],'edited colored source does not align with the textarea');const record={theme,width,originalLinks:original.links,state,overlayBefore,overlayEdited};receipt.themes.push(record);
  assert.equal(state.source,edited);assert.equal(state.text,edited+' ');assert(state.comments.includes('/* outer /* nested */ remains literal */'),'nested comment lost coloring after edit');assert(state.comments.includes('// </span><script>literal Ω</script>'),'literal hostile comment lost coloring');assert.equal(state.links,0,'edited source retained stale compiler links');assert.equal(state.scripts,0);assert(state.resetVisible);
  await evaluate("document.querySelector('figure:has(.ed textarea) [data-act=reset]').click()");
  const reset=await evaluate("(()=>{const figure=document.querySelector('figure:has(.ed textarea)');return{source:figure.querySelector('textarea').value,markup:figure.querySelector('pre code').innerHTML,links:figure.querySelectorAll('pre code a').length};})()");assert.deepEqual(reset,original,'reset did not restore original compiler markup and links');record.overlayReset=await evaluate(overlayProbe);assert.deepEqual(record.overlayReset.problems,[],'reset compiler-linked code does not align with the textarea');
 }
 assert.deepEqual(receipt.exceptions,[]);assert(!receipt.requests.some(request=>/\.wasm(?:$|\?)/u.test(request.url)||/\/runtime\//u.test(new URL(request.url).pathname)),'editing loaded the compiler runtime');receipt.status='pass';console.log('Editor coloring: real trusted edits and reset passed in both themes');
}finally{await writeFile(output,JSON.stringify(receipt,null,2));connection?.close();child.kill();await new Promise(resolve=>child.exitCode!==null||child.signalCode!==null?resolve():child.once('exit',resolve));await rm(profile,{recursive:true,force:true});}
