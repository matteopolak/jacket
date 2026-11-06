#!/usr/bin/env node
/** Require desktop rail geometry to stay independent of page height and page scroll.
 * node tools/dev/doc_rail_viewport.mjs BASE_URL OUTPUT_JSON [--source-css CSS]
 */
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {mkdtemp,readFile,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {setTimeout as delay} from 'node:timers/promises';
import {Connection,runtimeException} from './chrome_cdp.mjs';
import {findChrome,waitForDevToolsPort} from './chrome_startup.mjs';
const base=new URL(process.argv[2]),output=process.argv[3],cssIndex=process.argv.indexOf('--source-css');
const css=cssIndex<0?null:await readFile(path.resolve(process.argv[cssIndex+1]),'utf8');
const profile=await mkdtemp(path.join(tmpdir(),'jacket-rail-viewport-'));
const child=spawn(findChrome(),['--headless=new','--no-sandbox','--disable-gpu','--no-first-run','--remote-debugging-port=0',`--user-data-dir=${profile}`,'about:blank'],{stdio:['ignore','ignore','pipe']});
const closed=new Promise(resolve=>child.once('close',resolve)),receipt={base:base.href,cssAblation:!!css,cases:[],problems:[],exceptions:[]};let connection;
try{
 const port=await waitForDevToolsPort(child,profile),pages=await(await fetch(`http://127.0.0.1:${port}/json/list`)).json();connection=await new Connection(pages.find(page=>page.type==='page').webSocketDebuggerUrl).open();
 connection.listeners.push(message=>{const error=runtimeException(message);if(error!==undefined)receipt.exceptions.push(error);});const cdp=(method,params={})=>connection.cdp(method,params);
 const evaluate=async expression=>{const result=await cdp('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!result.exceptionDetails,JSON.stringify(result.exceptionDetails));return result.result.value;};
 await cdp('Page.enable');await cdp('Runtime.enable');
 for(const width of [899,900,1280,1600])for(const theme of ['light','dark'])for(const route of ['api/Basic','tools/compiling','lint-rules/self_assignment']){
  await cdp('Emulation.setDeviceMetricsOverride',{width,height:900,deviceScaleFactor:1,mobile:false});await cdp('Emulation.setEmulatedMedia',{features:[{name:'prefers-color-scheme',value:theme}]});
  const before=await evaluate('performance.timeOrigin'),url=new URL(route,base);url.searchParams.set('rail-viewport',`${width}-${theme}`);await cdp('Page.navigate',{url:url.href});
  let ready=false;for(let n=0;n<400;n++){if(await evaluate(`performance.timeOrigin!==${before}&&document.readyState==='complete'&&!!document.querySelector('details.side')`)){ready=true;break;}await delay(25);}assert(ready,'fixture did not load '+url);
  if(css)await evaluate(`(()=>{const style=document.createElement('style');style.textContent=${JSON.stringify(css)};document.head.append(style);})()`);
  await evaluate(`document.documentElement.dataset.theme=${JSON.stringify(theme)};document.fonts.ready`);
  await evaluate(`(()=>{for(const details of document.querySelectorAll('.side details'))details.open=true;return Promise.all(document.getAnimations().map(animation=>animation.finished.catch(()=>{})));})()`);
  const samples=[];
  for(const y of [0,100,1000000000]){
   await evaluate(`scrollTo({top:${y},behavior:'instant'});new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))`);
   samples.push(await evaluate(`(()=>{const side=document.querySelector('.side'),wrap=document.querySelector('.wrap'),main=document.querySelector('main'),header=document.querySelector('.top');const rect=node=>{const r=node.getBoundingClientRect();return{x:r.x,y:r.y,width:r.width,height:r.height,right:r.right,bottom:r.bottom};};return{y:scrollY,side:rect(side),wrap:rect(wrap),main:rect(main),header:rect(header),padding:parseFloat(getComputedStyle(wrap).paddingLeft),gap:parseFloat(getComputedStyle(wrap).columnGap),position:getComputedStyle(side).position,reservation:getComputedStyle(wrap,'::before').content};})()`));
  }
  let bottomHit=null;
  if(width>=900){
   bottomHit=await evaluate(`(async()=>{const side=document.querySelector('.side');side.scrollTop=side.scrollHeight;await new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));const links=[...side.querySelectorAll('a[href]')].filter(a=>a.getClientRects().length),last=links.at(-1),r=last.getBoundingClientRect(),x=r.left+r.width/2,y=r.top+r.height/2,hit=document.elementFromPoint(x,y);const footer=document.querySelector('.foot a'),f=footer.getBoundingClientRect(),footerHit=document.elementFromPoint(f.left+f.width/2,f.top+f.height/2);return{href:last.href,top:r.top,bottom:r.bottom,viewport:innerHeight,hit:hit?.closest('a')?.href??null,scroll:side.scrollTop,max:side.scrollHeight-side.clientHeight,footer:{href:footer.href,hit:footerHit?.closest('a')?.href??null,left:f.left,sideRight:side.getBoundingClientRect().right}};})()`);
   if(bottomHit.hit!==bottomHit.href)receipt.problems.push({width,theme,route,reason:'last sidebar link occluded at document and rail maximum scroll',bottomHit});
   if(bottomHit.footer.hit!==bottomHit.footer.href||bottomHit.footer.left<bottomHit.footer.sideRight)receipt.problems.push({width,theme,route,reason:'desktop footer credit occluded or overlaps live rail',bottomHit});
  }
  receipt.cases.push({width,theme,route,samples,bottomHit});
  for(const sample of samples){
   if(width>=900){
    if(sample.position!=='fixed'||Math.abs(sample.side.y-sample.header.bottom)>0.2)receipt.problems.push({width,theme,route,reason:'desktop rail moves with main/scroll',sample});
    if(Math.abs(sample.side.x-(sample.wrap.x+sample.padding))>0.2)receipt.problems.push({width,theme,route,reason:'reserved column horizontal alignment changed',sample});
    if(sample.main.x<sample.side.right+sample.gap-0.2)receipt.problems.push({width,theme,route,reason:'main overlaps reserved rail column',sample});
   }else if(sample.position!=='static'||sample.reservation!=='none')receipt.problems.push({width,theme,route,reason:'mobile layout changed',sample});
  }
 }
 await writeFile(output,JSON.stringify(receipt,null,2)+'\n');console.log(JSON.stringify({cases:receipt.cases.length,problems:receipt.problems.length,exceptions:receipt.exceptions.length}));assert.deepEqual(receipt.exceptions,[]);assert.deepEqual(receipt.problems,[]);
}finally{connection?.close();child.kill('SIGTERM');await closed;await rm(profile,{recursive:true,force:true});}
