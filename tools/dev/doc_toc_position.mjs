#!/usr/bin/env node
/** Check desktop outline geometry before and during sticky admission.
 * Example: node tools/dev/doc_toc_position.mjs http://127.0.0.1:57526/ /tmp/toc-position.json
 */
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {createHash} from 'node:crypto';
import {mkdtemp,readFile,rm,writeFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {setTimeout as delay} from 'node:timers/promises';
import {Connection,runtimeException} from './chrome_cdp.mjs';
import {findChrome,waitForDevToolsPort} from './chrome_startup.mjs';
const base=new URL(process.argv[2]),output=path.resolve(process.argv[3]);
const sourceCssIndex=process.argv.indexOf('--source-css');
const sourceCss=sourceCssIndex<0?undefined:await readFile(path.resolve(process.argv[sourceCssIndex+1]),'utf8');
const profile=await mkdtemp(path.join(tmpdir(),'jacket-toc-'));
const chrome=findChrome();
const child=spawn(chrome,['--headless=new','--no-sandbox','--disable-gpu','--no-first-run','--remote-debugging-port=0',`--user-data-dir=${profile}`,'about:blank'],{stdio:['ignore','ignore','pipe']});
const receipt={base:base.href,cases:[],exceptions:[],problems:[]};let connection;
try {
 const port=await waitForDevToolsPort(child,profile);const pages=await(await fetch(`http://127.0.0.1:${port}/json/list`)).json();
 connection=await new Connection(pages.find(page=>page.type==='page').webSocketDebuggerUrl).open();
 connection.listeners.push(message=>{const error=runtimeException(message);if(error!==undefined)receipt.exceptions.push(error);});
 const cdp=(method,params={})=>connection.cdp(method,params);
 const evaluate=async expression=>{const value=await cdp('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!value.exceptionDetails,JSON.stringify(value.exceptionDetails));return value.result.value;};
 await cdp('Page.enable');await cdp('Runtime.enable');
 for(const width of [1280,1600])for(const theme of ['light','dark'])for(const scale of [1,1.25,2])for(const page of ['api/Basic','guides/testing','api/Hash_Table']) {
  await cdp('Emulation.setDeviceMetricsOverride',{width,height:900,deviceScaleFactor:scale,mobile:false});await cdp('Emulation.setEmulatedMedia',{features:[{name:'prefers-color-scheme',value:theme}]});
  const origin=await evaluate('performance.timeOrigin');await cdp('Page.navigate',{url:new URL(page,base).href});let ready=false;
  for(let attempt=0;attempt<400;attempt++){if(await evaluate(`performance.timeOrigin!==${origin}&&document.readyState==='complete'&&!!document.querySelector('.toc-h')`)){ready=true;break;}await delay(25);}assert(ready,`${page} did not load a visible outline`);
  await evaluate('document.fonts.ready');
  if(sourceCss!==undefined) {await evaluate(`(()=>{const style=document.createElement('style');style.textContent=${JSON.stringify(sourceCss)};document.head.append(style);})()`);receipt.sourceCssAblation=true;receipt.sourceCssSha256=createHash('sha256').update(sourceCss).digest('hex');}
  const header=await evaluate(`(()=>{const input=document.querySelector('#search-q'),icon=document.querySelector('.search>svg'),key=document.querySelector('.search>kbd');const rect=node=>{const box=node.getBoundingClientRect();return{x:box.x,y:box.y,width:box.width,height:box.height,center:box.y+box.height/2};};return['empty-blurred','empty-focused','filled-focused','filled-blurred'].map(state=>{input.value=state.startsWith('filled')?'Fixture':'';if(state.endsWith('blurred'))input.blur();else input.focus({preventScroll:true});return{state,input:rect(input),icon:rect(icon),key:rect(key),dpr:devicePixelRatio,inputLineHeight:getComputedStyle(input).lineHeight,keyLineHeight:getComputedStyle(key).lineHeight};});})()`);
  for(const state of header)for(const control of ['icon','key'])if(Math.abs(state[control].center-state.input.center)>.1)receipt.problems.push(`${width}/${theme}/${scale}/${page}/${state.state}: search ${control} not centered`);
  const samples=[];
  for(const scroll of [0,1,2,5,10,80,240]) {
   samples.push(await evaluate(`(async()=>{scrollTo({top:${scroll},behavior:'instant'});await new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));const toc=document.querySelector('.toc'),heading=document.querySelector('.toc-h'),header=document.querySelector('.top'),style=getComputedStyle(toc);return{requested:${scroll},scrollY,y:toc.getBoundingClientRect().y,headingY:heading.getBoundingClientRect().y,headerBottom:header.getBoundingClientRect().bottom,stickyTop:style.top,height:toc.clientHeight,scrollHeight:toc.scrollHeight,display:style.display,position:style.position};})()`));
  }
  const first=samples[0];
  if(samples.some(sample=>Math.abs(sample.y-first.y)>.05||Math.abs(sample.headingY-first.headingY)>.05))receipt.problems.push(`${width}/${theme}/${page}: outline drifts before sticking`);
  if(samples.some(sample=>sample.y+.05<sample.headerBottom))receipt.problems.push(`${width}/${theme}/${page}: outline overlaps sticky header`);
  const independent=await evaluate(`(()=>{const toc=document.querySelector('.toc'),before=scrollY;toc.scrollTop=Math.min(19,Math.max(0,toc.scrollHeight-toc.clientHeight));return{documentBefore:before,documentAfter:scrollY,tocScroll:toc.scrollTop,canScroll:toc.scrollHeight>toc.clientHeight};})()`);
  if(independent.documentAfter!==independent.documentBefore||independent.canScroll&&independent.tocScroll<=0)receipt.problems.push(`${width}/${theme}/${page}: outline does not scroll independently`);
  receipt.cases.push({width,theme,scale,page,header,samples,independent});
 }
 await cdp('Emulation.setDeviceMetricsOverride',{width:375,height:812,deviceScaleFactor:1,mobile:true});await evaluate('scrollTo(0,0)');
 receipt.mobile=await evaluate(`({display:getComputedStyle(document.querySelector('.toc')).display})`);assert.equal(receipt.mobile.display,'none');
 assert.equal(receipt.exceptions.length,0,receipt.exceptions.join('\n'));
 assert.equal(receipt.problems.length,0,receipt.problems.join('\n'));
} finally {
 await writeFile(output,JSON.stringify(receipt,null,2)+'\n');connection?.close();const closed=child.exitCode!==null||child.signalCode!==null?Promise.resolve():new Promise(resolve=>child.once('close',resolve));child.kill('SIGTERM');await closed;await rm(profile,{recursive:true,force:true});
}
