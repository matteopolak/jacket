#!/usr/bin/env node
/** Trusted actual Run and retained search/SPA smoke. BASE_URL OUTPUT_JSON [--dev] [--server-log FILE]. */
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawn} from 'node:child_process';
import {mkdtemp,rm,writeFile,readFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {setTimeout as delay} from 'node:timers/promises';
import {Connection,runtimeException} from './chrome_cdp.mjs';
import {findChrome,waitForDevToolsPort} from './chrome_startup.mjs';
const dev=process.argv.includes('--dev'),logFlag=process.argv.indexOf('--server-log');
const base=new URL(process.argv[2]),out=process.argv[3],profile=await mkdtemp(path.join(tmpdir(),'docs-runtime-'));
const child=spawn(findChrome(),['--headless=new','--no-sandbox','--disable-gpu','--no-first-run','--remote-debugging-port=0',`--user-data-dir=${profile}`,'about:blank'],{stdio:['ignore','ignore','pipe']});
let connection;const receipt={base:base.href,mode:dev?'development':'production',requests:[],exceptions:[],checks:[]};
try{
 const port=await waitForDevToolsPort(child,profile),pages=await(await fetch(`http://127.0.0.1:${port}/json/list`)).json();connection=await new Connection(pages.find(p=>p.type==='page').webSocketDebuggerUrl).open();
 connection.listeners.push(m=>{const e=runtimeException(m);if(e!==undefined)receipt.exceptions.push(e);if(m.method==='Network.requestWillBeSent')receipt.requests.push({url:m.params.request.url,type:m.params.type});});
 const cdp=(method,params={})=>connection.cdp(method,params),evaluate=async expression=>{const r=await cdp('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!r.exceptionDetails,JSON.stringify(r.exceptionDetails));return r.result.value;};
 const wait=async(fn,label,ms=60000)=>{const end=Date.now()+ms;while(Date.now()<end){if(await fn())return;await delay(50);}throw Error(label);};
 await cdp('Page.enable');await cdp('Runtime.enable');await cdp('Network.enable');await cdp('Emulation.setDeviceMetricsOverride',{width:1280,height:900,deviceScaleFactor:1,mobile:false});
 await cdp('Page.navigate',{url:new URL('api/Basic/align_forward',base).href});await wait(()=>evaluate(`document.readyState==='complete'&&!!document.querySelector('button[data-act=run]')`),'enabled Run missing');
 assert(!receipt.requests.some(r=>/\.wasm(?:$|\?)/u.test(r.url)),'compiler fetched before user Run');
 receipt.checks.push('runtime lazy before Run');
 receipt.runtimeUrls=await evaluate(`({engine:document.body.dataset.runtimeEngine,wasm:document.body.dataset.runtimeWasm,icons:document.body.dataset.docIcons})`);for(const key of ['engine','wasm'])assert.match(new URL(receipt.runtimeUrls[key],base).searchParams.get('v'),/^[a-f0-9]{64}$/u);if(dev)assert(new URL(receipt.runtimeUrls.icons,base).pathname.endsWith('/src/assets/icons.svg'),'dev sprite must use the actual Vite source URL');else assert.match(new URL(receipt.runtimeUrls.icons,base).pathname,/\/_astro\/icons\.[^/]+\.svg$/u);
 const logPath=logFlag<0?undefined:process.argv[logFlag+1],logBefore=logPath?(await readFile(logPath,'utf8')).length:0;
 await evaluate(`(()=>{const b=document.querySelector('button[data-act=run]');b.scrollIntoView({block:'center'});return document.fonts.ready;})()`);
 const point=await evaluate(`(()=>{const r=document.querySelector('button[data-act=run]').getBoundingClientRect();return{x:r.x+r.width/2,y:r.y+r.height/2};})()`);
 await cdp('Input.dispatchMouseEvent',{type:'mousePressed',button:'left',clickCount:1,...point});await cdp('Input.dispatchMouseEvent',{type:'mouseReleased',button:'left',clickCount:1,...point});
 await wait(()=>evaluate(`document.querySelector('.ex-out')?.textContent.includes('✓ Ran')||document.querySelector('.ex-out')?.classList.contains('bad')`),'actual Run never completed',120000);
 receipt.result=await evaluate(`document.querySelector('.ex-out').textContent`);assert(receipt.result.includes('✓ Ran'),receipt.result);if(logPath){receipt.workerHttp=(await readFile(logPath,'utf8')).slice(logBefore).split('\n').filter(line=>line.includes('/runtime/'));for(const key of ['engine','wasm']){const url=new URL(receipt.runtimeUrls[key],base);assert(receipt.workerHttp.some(line=>line.includes('GET '+url.pathname+url.search+' HTTP/1.1\" 200')),'server did not serve the exact admitted '+key+' URL');}}
 receipt.runtimeHttp=[];for(const key of ['engine','wasm']){const url=new URL(receipt.runtimeUrls[key],base),response=await fetch(url),bytes=Buffer.from(await response.arrayBuffer()),digest=createHash('sha256').update(bytes).digest('hex');assert.equal(response.status,200);assert.equal(digest,url.searchParams.get('v'));receipt.runtimeHttp.push({key,url:url.href,status:response.status,sha256:digest,mime:response.headers.get('content-type')});}receipt.checks.push('trusted Run completed successfully');
 await evaluate(`(()=>{window.__sameBody=document.body;const a=[...document.querySelectorAll('a[href]')].find(a=>new URL(a.href).pathname===${JSON.stringify(new URL('api/Basic',base).pathname)});a.click();})()`);
 await wait(()=>evaluate(`location.pathname===${JSON.stringify(new URL('api/Basic',base).pathname)}&&document.querySelector('h1')?.textContent.includes('Basic')`),'navigation away failed');assert(await evaluate('document.body===window.__sameBody'),'SPA swapped retained body');receipt.checks.push('main-only SPA after actual Run');
 await evaluate(`(()=>{const q=document.querySelector('#search-q');window.__sameSearch=q;q.focus();q.value='Bucket_Array';q.dispatchEvent(new Event('input',{bubbles:true}));})()`);
 await wait(()=>evaluate(`document.querySelector('#search-results')?.hidden===false&&!!document.querySelector('#search-results a.hit')`),'search did not show actual index results');
 const expected=await evaluate(`document.querySelector('#search-results a.hit').href`);
 await cdp('Input.dispatchKeyEvent',{type:'keyDown',key:'Enter',code:'Enter'});await cdp('Input.dispatchKeyEvent',{type:'keyUp',key:'Enter',code:'Enter'});
 await wait(()=>evaluate(`location.href===${JSON.stringify(expected)}&&document.querySelector('#search-results').hidden`),'search navigation left stale visible results');
 receipt.search=await evaluate(`({same:document.querySelector('#search-q')===window.__sameSearch,value:document.querySelector('#search-q').value,rows:document.querySelector('#search-results').childElementCount,expanded:document.querySelector('#search-q').getAttribute('aria-expanded'),active:document.querySelector('#search-q').getAttribute('aria-activedescendant'),sameBody:document.body===window.__sameBody})`);
 assert.deepEqual(receipt.search,{same:true,value:'Bucket_Array',rows:0,expanded:'false',active:null,sameBody:true});receipt.checks.push('retained search input, real index and Enter navigation clear stale results');

 assert.deepEqual(receipt.exceptions,[]);receipt.status='pass';
}catch(e){receipt.status='fail';receipt.error=String(e);throw e;}finally{await writeFile(out,JSON.stringify(receipt,null,2)+'\n');connection?.close();child.kill();await new Promise(r=>child.exitCode!==null||child.signalCode!==null?r():child.once('exit',r));await rm(profile,{recursive:true,force:true});}
