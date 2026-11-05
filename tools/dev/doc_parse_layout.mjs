#!/usr/bin/env node
// Hold real production HTML at deterministic rail/footer parser boundaries, then compare
// live geometry before and after each release. No timer determines which frame is sampled.
// Example: node tools/dev/doc_parse_layout.mjs target/doc-site /tmp/parser-layout.json
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {readFile,mkdtemp,rm,writeFile} from 'node:fs/promises';
import {resolve,join,extname,basename} from 'node:path';
import {tmpdir} from 'node:os';
import {spawn} from 'node:child_process';
import {setTimeout as delay} from 'node:timers/promises';
import {Connection,runtimeException} from './chrome_cdp.mjs';
import {findChrome,waitForDevToolsPort} from './chrome_startup.mjs';
assert(process.argv[2]&&process.argv[3],'usage: doc_parse_layout.mjs SITE RECEIPT');
const site=resolve(process.argv[2]),output=process.argv[3],profile=await mkdtemp(join(tmpdir(),'docs-parser-'));
const mime={'.js':'text/javascript','.mjs':'text/javascript','.css':'text/css','.svg':'image/svg+xml','.json':'application/json','.html':'text/html','.wasm':'application/wasm'};
let pending,active;
const gate=()=>{let release;const promise=new Promise(done=>{release=done;});return {promise,release};};
const server=createServer(async(request,response)=>{
 try{
  const pathname=decodeURIComponent(new URL(request.url,'http://localhost').pathname);
  let file=resolve(site,'.'+pathname);assert(file===site||file.startsWith(site+'/'));
  if(pathname==='/')file=join(site,'index.html');else if(!extname(file))file+='.html';
  const body=await readFile(file);response.writeHead(200,{'Content-Type':mime[extname(file)]||'application/octet-stream','Cache-Control':'no-store'});
  if(pending&&pending.pathname===pathname){
   const state=pending;active=state;pending=null;const text=body.toString(),category=text.indexOf('<details class="nav-cat"'),railEnd=text.indexOf('</summary>',category)+10,footer=text.indexOf('<footer');
   assert(category>=0&&railEnd>category&&footer>railEnd,'streaming fixture needs actual rail and footer');
   response.write(text.slice(0,railEnd));state.started=true;await state.rail.promise;
   response.write(text.slice(railEnd,footer));await state.footer.promise;response.end(text.slice(footer));
  }else response.end(body);
 }catch(error){if(!response.headersSent)response.writeHead(500);response.end(String(error));}
});
await new Promise(done=>server.listen(0,'127.0.0.1',done));
const base=`http://127.0.0.1:${server.address().port}`;
const child=spawn(findChrome(),['--headless=new','--no-sandbox','--no-first-run','--remote-debugging-port=0',`--user-data-dir=${profile}`,'about:blank'],{stdio:['ignore','ignore','pipe']});
const receipt={purpose:'deterministic real production HTML parser boundaries, no latency measurement',cases:[],exceptions:[]};let connection;
try{
 const port=await waitForDevToolsPort(child,profile),pages=await(await fetch(`http://127.0.0.1:${port}/json/list`)).json();connection=await new Connection(pages.find(page=>page.type==='page').webSocketDebuggerUrl).open();
 connection.listeners.push(message=>{const error=runtimeException(message);if(error!==undefined)receipt.exceptions.push(error);});
 const cdp=(method,parameters={})=>connection.cdp(method,parameters),evaluate=async expression=>{const result=await cdp('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});assert(!result.exceptionDetails,JSON.stringify(result.exceptionDetails));return result.result.value;};
 const wait=async(expression)=>{const deadline=Date.now()+30000;while(Date.now()<deadline){if(await evaluate(expression))return;await delay(20);}throw Error('parser fixture condition: '+expression);};
 const frames=()=>evaluate('new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))');
 const geometry=()=>evaluate(`(()=>{const rect=selector=>{const element=document.querySelector(selector);if(!element)return null;const r=element.getBoundingClientRect();return{x:r.x,y:r.y,width:r.width,height:r.height};};return{readyState:document.readyState,header:rect('.top'),side:rect('.side'),main:rect('main'),h1:rect('main h1'),footer:rect('footer'),wrap:getComputedStyle(document.querySelector('.wrap')).alignContent,sideHeight:getComputedStyle(document.querySelector('.side')).height};})()`);
 await cdp('Page.enable');await cdp('Runtime.enable');
 for(const width of [375,1280])for(const theme of ['light','dark'])for(const pathname of ['/format-rules/max_width','/404.html']){
  await cdp('Emulation.setDeviceMetricsOverride',{width,height:900,deviceScaleFactor:1,mobile:false});
  await cdp('Page.addScriptToEvaluateOnNewDocument',{source:`localStorage.setItem('jacket-docs-theme',${JSON.stringify(theme)})`});
  const state={pathname,rail:gate(),footer:gate()},caseId=receipt.cases.length+1;pending=state;
  await cdp('Page.addScriptToEvaluateOnNewDocument',{source:`window.__parserFixtureCase=${caseId};sessionStorage.clear()`});
  await cdp('Page.navigate',{url:base+pathname});
  await wait(`window.__parserFixtureCase===${caseId}&&document.readyState==='loading'&&!!document.querySelector('details.nav-cat summary')&&getComputedStyle(document.querySelector('.side')).paddingTop!=='0px'`);assert(state.started,'expected parser barrier request');await frames();
  const railPartial=await geometry();state.rail.release();
  await wait(`!!document.querySelector('main h1')`);await frames();const footerPending=await geometry();assert.equal(footerPending.readyState,'loading');assert.equal(footerPending.footer,null);
  state.footer.release();await wait(`document.readyState==='complete'`);await evaluate('document.fonts.ready');await frames();const complete=await geometry();
  let screenshot;if(width===375){const file=output.replace(/\.json$/u,'')+`-${theme}-${pathname.replace(/[^a-z0-9_-]+/giu,'-')}.png`;const capture=await cdp('Page.captureScreenshot',{format:'png'});await writeFile(file,Buffer.from(capture.data,'base64'));screenshot=basename(file);}
  const problems=[];if(Math.abs(footerPending.main.y-complete.main.y)>.1)problems.push('main moves after footer parsing');if(width>=900&&Math.abs(railPartial.side.height-complete.side.height)>.1)problems.push('desktop rail grows while parsing');
  receipt.cases.push({width,theme,pathname,railPartial,footerPending,complete,problems,screenshot});
 }
 receipt.status=receipt.cases.every(item=>item.problems.length===0)&&receipt.exceptions.length===0?'pass':'fail';
 assert.equal(receipt.status,'pass',JSON.stringify(receipt.cases.filter(item=>item.problems.length).map(({width,theme,pathname,problems})=>({width,theme,pathname,problems}))));
}catch(error){receipt.status='fail';receipt.error=String(error);throw error;}finally{
 await writeFile(output,JSON.stringify(receipt,null,2)+'\n');pending?.rail.release();pending?.footer.release();active?.rail.release();active?.footer.release();connection?.close();child.kill();await new Promise(done=>child.exitCode!==null||child.signalCode!==null?done():child.once('exit',done));server.closeAllConnections();await new Promise(done=>server.close(done));await rm(profile,{recursive:true,force:true});
}
