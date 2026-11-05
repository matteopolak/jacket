#!/usr/bin/env node
/** Check real highlighted fences against their padding, preserving authored text.
 * Example: node tools/dev/doc_code_blocks.mjs http://127.0.0.1:8000/ /tmp/owned-code-blocks
 */
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {mkdir,mkdtemp,rm,writeFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {setTimeout as delay} from 'node:timers/promises';
import {Connection,runtimeException} from './chrome_cdp.mjs';
import {findChrome,waitForDevToolsPort} from './chrome_startup.mjs';
const base=new URL(process.argv[2]),out=path.resolve(process.argv[3]);
await mkdir(out,{recursive:false});
const profile=await mkdtemp(path.join(tmpdir(),'jacket-code-blocks-'));
const child=spawn(findChrome(),['--headless=new','--no-sandbox','--disable-gpu','--no-first-run','--remote-debugging-port=0',`--user-data-dir=${profile}`,'about:blank'],{stdio:['ignore','ignore','pipe']});
const receipt={base:base.href,samples:[],problems:[],exceptions:[]};let connection;
try {
 const port=await waitForDevToolsPort(child,profile),pages=await(await fetch(`http://127.0.0.1:${port}/json/list`)).json();
 connection=await new Connection(pages.find(page=>page.type==='page').webSocketDebuggerUrl).open();
 connection.listeners.push(message=>{const error=runtimeException(message);if(error!==undefined)receipt.exceptions.push(error);});
 const cdp=(method,params={})=>connection.cdp(method,params);
 const evaluate=async expression=>{const result=await cdp('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!result.exceptionDetails,JSON.stringify(result.exceptionDetails));return result.result.value;};
 await cdp('Page.enable');await cdp('Runtime.enable');
 for(const width of [375,1280,1600])for(const theme of ['light','dark'])for(const route of ['','guides/documenting']) {
  await cdp('Emulation.setDeviceMetricsOverride',{width,height:900,deviceScaleFactor:1,mobile:false});
  await cdp('Emulation.setEmulatedMedia',{features:[{name:'prefers-color-scheme',value:theme}]});
  await cdp('Page.navigate',{url:new URL(route,base).href});
  const end=Date.now()+30000;let loaded=false;
  while(Date.now()<end){if(await evaluate(`document.readyState==='complete'&&location.pathname===${JSON.stringify(new URL(route,base).pathname)}`)){loaded=true;break;}await delay(25);}assert(loaded,'actual command page did not load');
  await evaluate('document.fonts.ready.then(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))))');
  const sample=await evaluate(`(()=>{const layout=document.querySelector('body > .wrap');return{width:innerWidth,rootTheme:document.documentElement.getAttribute('data-theme'),layout:{display:getComputedStyle(layout).display,before:getComputedStyle(layout,'::before').content},blocks:[...document.querySelectorAll('main pre.code.wrap')].map(pre=>{const style=getComputedStyle(pre),box=pre.getBoundingClientRect(),walker=document.createTreeWalker(pre,NodeFilter.SHOW_TEXT);let node,first;while(node=walker.nextNode()){const range=document.createRange();range.selectNodeContents(node);const rect=[...range.getClientRects()].find(r=>r.width>0);if(rect){first={x:rect.x,y:rect.y,text:node.textContent};break;}}return{text:pre.textContent,display:style.display,before:getComputedStyle(pre,'::before').content,padding:Number.parseFloat(style.paddingLeft)+Number.parseFloat(style.borderLeftWidth),box:{x:box.x,y:box.y,width:box.width},first};})};})()`);
  assert(sample.blocks.length>=3,'representative highlighted fences missing');
  if(!route)for(const expected of ['brew install matteopolak/tap/jacket','jacket run hello.jai'])assert(sample.blocks.some(block=>block.text===expected),'home command text changed: '+expected);
  for(const [index,block]of sample.blocks.entries()) {
   if(!['none','normal'].includes(block.before))receipt.problems.push({route,width,theme,index,reason:'layout spacer generated inside code block',block});
   if(!block.first||Math.abs(block.first.x-block.box.x-block.padding)>0.25)receipt.problems.push({route,width,theme,index,reason:'first text starts outside authored code padding',block});
  }
  receipt.samples.push({route,width,theme,...sample});
  const screenshot=await cdp('Page.captureScreenshot',{format:'png'});await writeFile(path.join(out,`${route?'documenting':'home'}-${width}-${theme}.png`),Buffer.from(screenshot.data,'base64'));
 }
 receipt.status=receipt.problems.length?'fail':'pass';assert.deepEqual(receipt.exceptions,[]);assert.deepEqual(receipt.problems,[]);
} catch(error) {receipt.status='fail';receipt.error=String(error);throw error;}
finally {
 await writeFile(path.join(out,'receipt.json'),JSON.stringify(receipt,null,2)+'\n');
 connection?.close();child.kill();await new Promise(resolve=>!child.pid||child.exitCode!==null||child.signalCode!==null?resolve():child.once('exit',resolve));await rm(profile,{recursive:true,force:true});
}
