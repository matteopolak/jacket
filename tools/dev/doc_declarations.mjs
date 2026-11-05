#!/usr/bin/env node
/** Verify real formatted declaration geometry, hash highlighting and narrow readability.
 * node tools/dev/doc_declarations.mjs BASE_URL OUTPUT_JSON
 */
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {mkdir,mkdtemp,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {setTimeout as delay} from 'node:timers/promises';
import {Connection,runtimeException} from './chrome_cdp.mjs';
import {findChrome,waitForDevToolsPort} from './chrome_startup.mjs';
const [url,output]=process.argv.slice(2);assert(url&&output,'pass production URL and absent receipt path');
const base=new URL(url),captures=path.resolve(path.dirname(output),path.basename(output,'.json')+'-captures');await mkdir(captures,{recursive:false});
const profile=await mkdtemp(path.join(tmpdir(),'jacket-declarations-'));
const child=spawn(findChrome(),['--headless=new','--no-sandbox','--disable-gpu','--no-first-run','--remote-debugging-port=0',`--user-data-dir=${profile}`,'about:blank'],{stdio:['ignore','ignore','pipe']});
const closed=new Promise(resolve=>child.once('close',resolve)),receipt={url:base.href,cases:[],exceptions:[],problems:[]};let connection;
try{
 const port=await waitForDevToolsPort(child,profile),pages=await(await fetch(`http://127.0.0.1:${port}/json/list`)).json();connection=await new Connection(pages.find(page=>page.type==='page').webSocketDebuggerUrl).open();
 const cdp=(method,params={})=>connection.cdp(method,params);
 receipt.browser=await cdp('Browser.getVersion');
 connection.listeners.push(message=>{const error=runtimeException(message);if(error!==undefined)receipt.exceptions.push(error);});
 const evaluate=async expression=>{const result=await cdp('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});assert(!result.exceptionDetails,JSON.stringify(result.exceptionDetails));return result.result.value;};
 await cdp('Page.enable');await cdp('Runtime.enable');let serial=0;
 for(const width of [375,1280])for(const theme of ['light','dark'])for(const [route,targets] of [['api/File_Watcher/File_Watcher',['m-callback','m-event_streams','d2-m-callback','m-watch_paths']],['api/Bindings_Generator/Library_Info',['m-handle','d1-m-handle-2']]]){
  await cdp('Emulation.setDeviceMetricsOverride',{width,height:900,deviceScaleFactor:1,mobile:false});await cdp('Emulation.setEmulatedMedia',{features:[{name:'prefers-color-scheme',value:theme}]});
  for(const target of targets){
   const destination=new URL(route,base);destination.searchParams.set('declaration-proof',String(serial++));destination.hash=target;
   const before=await evaluate('performance.timeOrigin');await cdp('Page.navigate',{url:destination.href});
   let ready=false;for(let n=0;n<400;n++){if(await evaluate(`performance.timeOrigin!==${before}&&document.readyState==='complete'&&!!document.getElementById(${JSON.stringify(target)})`)){ready=true;break;}await delay(25);}assert(ready,`${route}#${target} absent`);
   await evaluate(`document.documentElement.dataset.theme=${JSON.stringify(theme)};document.fonts.ready`);
   await evaluate(`new Promise(resolve=>{let n=0;const frame=()=>{if(++n>30)resolve();else requestAnimationFrame(frame);};requestAnimationFrame(frame);})`);
   const sample=await evaluate(`(()=>{
    const target=document.getElementById(${JSON.stringify(target)}),line=target.closest('.formatted-line'),pre=target.closest('pre'),header=document.querySelector('.top');
    const rect=node=>{const r=node.getBoundingClientRect();return{x:r.x,y:r.y,width:r.width,height:r.height,bottom:r.bottom};};
    const before=pre.getBoundingClientRect(),reference=pre.cloneNode(false),code=pre.querySelector('code').cloneNode(false);reference.removeAttribute('id');reference.style.cssText='position:absolute;visibility:hidden;top:-100000px;left:0;width:'+before.width+'px';code.textContent=pre.querySelector('code').textContent;reference.append(code);pre.parentElement.append(reference);const referenceHeight=reference.getBoundingClientRect().height;reference.remove();
    return{theme:document.documentElement.dataset.theme,colorScheme:getComputedStyle(document.documentElement).colorScheme,bodyWidth:document.documentElement.scrollWidth,pre:rect(pre),plainHeight:referenceHeight,line:line?rect(line):null,highlight:line?getComputedStyle(line).backgroundColor:null,headerBottom:header.getBoundingClientRect().bottom,target:rect(target),hash:location.hash,font:getComputedStyle(pre).font,lineHeight:getComputedStyle(pre).lineHeight,definitionCount:document.querySelectorAll('pre.semantic-declaration').length};
   })()`);
   receipt.cases.push({width,theme,route,target,...sample});
   if(sample.theme!==theme||sample.colorScheme!==theme)receipt.problems.push({width,theme,route,target,reason:'wrong theme'});
   if(!sample.line||sample.highlight==='rgba(0, 0, 0, 0)')receipt.problems.push({width,theme,route,target,reason:'field line highlight absent'});
   if(sample.line&&(sample.line.y<sample.headerBottom-1||sample.line.y>900))receipt.problems.push({width,theme,route,target,reason:'hash field obscured or outside viewport',sample});
   if(Math.abs(sample.pre.height-sample.plainHeight)>1)receipt.problems.push({width,theme,route,target,reason:'line wrappers changed formatter text height',sample});
   if(sample.bodyWidth>width+1||sample.pre.x<0||sample.pre.x+sample.pre.width>width+1)receipt.problems.push({width,theme,route,target,reason:'declaration/page horizontal overflow',sample});
  }
  const file=`${theme}-${width}-${route.split('/')[1]}.png`,metrics=await cdp('Page.getLayoutMetrics'),height=Math.ceil(metrics.cssContentSize.height);
  const screenshot=await cdp('Page.captureScreenshot',{format:'png',captureBeyondViewport:true,clip:{x:0,y:0,width,height,scale:1}});await writeFile(path.join(captures,file),Buffer.from(screenshot.data,'base64'));
 }
 await writeFile(output,JSON.stringify(receipt,null,2)+'\n');console.log(JSON.stringify({cases:receipt.cases.length,problems:receipt.problems.length,exceptions:receipt.exceptions.length,captures}));
 assert.deepEqual(receipt.exceptions,[]);assert.deepEqual(receipt.problems,[]);
}finally{connection?.close();child.kill('SIGTERM');await closed;await rm(profile,{recursive:true,force:true});}
