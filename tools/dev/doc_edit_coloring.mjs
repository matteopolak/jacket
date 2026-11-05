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
let connection;const receipt={page:'api/Basic/align_forward',themes:[],exceptions:[],requests:[]};
try{
 const port=await waitForDevToolsPort(child,profile),pages=await(await fetch(`http://127.0.0.1:${port}/json/list`)).json();connection=await new Connection(pages.find(page=>page.type==='page').webSocketDebuggerUrl).open();
 connection.listeners.push(message=>{if(message.method==='Network.requestWillBeSent')receipt.requests.push({url:message.params.request.url,type:message.params.type});const error=runtimeException(message);if(error!==undefined)receipt.exceptions.push(error);});const cdp=(method,params={})=>connection.cdp(method,params);
 const evaluate=async expression=>{const response=await cdp('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!response.exceptionDetails,JSON.stringify(response.exceptionDetails));return response.result.value;};
 await cdp('Page.enable');await cdp('Runtime.enable');await cdp('Network.enable');await cdp('Emulation.setDeviceMetricsOverride',{width:1280,height:900,deviceScaleFactor:1,mobile:false});
 for(const theme of ['light','dark']){
  await cdp('Emulation.setEmulatedMedia',{features:[{name:'prefers-color-scheme',value:theme}]});const beforeOrigin=await evaluate('performance.timeOrigin');await cdp('Page.navigate',{url:new URL(receipt.page,base).href});let ready=false;
  for(let attempt=0;attempt<600;attempt++){if(await evaluate(`performance.timeOrigin!==${beforeOrigin}&&location.pathname===${JSON.stringify(new URL(receipt.page,base).pathname)}&&document.readyState==='complete'&&!!document.querySelector('figure .ed textarea')`)){ready=true;break;}await delay(25);}assert(ready,'production editor did not initialize');
  const original=await evaluate("(()=>{const figure=document.querySelector('figure:has(.ed textarea)');return{source:figure.querySelector('textarea').value,markup:figure.querySelector('pre code').innerHTML,links:figure.querySelectorAll('pre code a').length};})()");
  const edited='/* outer /* nested */ remains literal */\n'+original.source+'\n// </span><script>literal Ω</script>\n';
  await evaluate("(()=>{const area=document.querySelector('figure .ed textarea');area.focus();area.select();})()");await cdp('Input.insertText',{text:edited});
  const state=await evaluate("(()=>{const figure=document.querySelector('figure:has(.ed textarea)'),code=figure.querySelector('pre code');return{source:figure.querySelector('textarea').value,text:code.textContent,comments:[...code.querySelectorAll('span.c')].map(span=>span.textContent),links:code.querySelectorAll('a').length,scripts:code.querySelectorAll('script').length,resetVisible:!figure.querySelector('[data-act=reset]').hidden};})()");receipt.themes.push({theme,originalLinks:original.links,state});
  assert.equal(state.source,edited);assert.equal(state.text,edited+' ');assert(state.comments.includes('/* outer /* nested */ remains literal */'),'nested comment lost coloring after edit');assert(state.comments.includes('// </span><script>literal Ω</script>'),'literal hostile comment lost coloring');assert.equal(state.links,0,'edited source retained stale compiler links');assert.equal(state.scripts,0);assert(state.resetVisible);
  await evaluate("document.querySelector('figure:has(.ed textarea) [data-act=reset]').click()");
  const reset=await evaluate("(()=>{const figure=document.querySelector('figure:has(.ed textarea)');return{source:figure.querySelector('textarea').value,markup:figure.querySelector('pre code').innerHTML,links:figure.querySelectorAll('pre code a').length};})()");assert.deepEqual(reset,original,'reset did not restore original compiler markup and links');
 }
 assert.deepEqual(receipt.exceptions,[]);assert(!receipt.requests.some(request=>/\.wasm(?:$|\?)/u.test(request.url)||/\/runtime\//u.test(new URL(request.url).pathname)),'editing loaded the compiler runtime');receipt.status='pass';console.log('Editor coloring: real trusted edits and reset passed in both themes');
}finally{await writeFile(output,JSON.stringify(receipt,null,2));connection?.close();child.kill();await new Promise(resolve=>child.exitCode!==null||child.signalCode!==null?resolve():child.once('exit',resolve));await rm(profile,{recursive:true,force:true});}
