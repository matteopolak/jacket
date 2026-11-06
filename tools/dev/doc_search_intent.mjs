#!/usr/bin/env node
/** Trusted browser search-intent checks against the actual source module and real validators.
 * node tools/dev/doc_search_intent.mjs FULL_SITE_DIRECTORY RECEIPT_JSON [--search-source PATH]
 * Synthetic delayed responses reuse two valid published search entries; no compiler/perf claim.
 */
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawn} from 'node:child_process';
import {createServer} from 'node:http';
import {readFile,writeFile,mkdtemp,rm,mkdir} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {setTimeout as delay} from 'node:timers/promises';
import {Connection,runtimeException} from './chrome_cdp.mjs';
import {findChrome,waitForDevToolsPort} from './chrome_startup.mjs';
const [site,output]=process.argv.slice(2);assert(site&&output,'supply full production site and receipt');
const repository=path.resolve(fileURLToPath(new URL('../../',import.meta.url))),builder=path.join(repository,'website/docs');
const option=process.argv.indexOf('--search-source'),source=path.resolve(option<0?path.join(builder,'src/browser/search.ts'):process.argv[option+1]);
const original=JSON.parse(await readFile(path.join(site,'search-index.json'),'utf8'));
const entries=['api/Basic','api/Bucket_Array'].map(id=>{const entry=original.data.entries.find(entry=>entry.id===id);assert(entry,`missing actual fixture entry ${id}`);return entry;});
const payload=Buffer.from(JSON.stringify({payload:'search',data:{schemaVersion:1,entries}})),digest=createHash('sha256').update(payload).digest('hex');
const work=await mkdtemp(path.join(tmpdir(),'jacket-search-intent-')),dist=path.join(work,'dist');
const profile=await mkdtemp(path.join(tmpdir(),'jacket-search-browser-'));
let child,connection,server;
const receipt={sourceSHA256:createHash('sha256').update(await readFile(source)).digest('hex'),payloadSHA256:digest,scope:'actual source module and validator in authored focused DOM; synthetic delayed valid JSON or empty HTTP503; not full Shell or performance measurement',cases:[],exceptions:[],requests:[]};
const pending=new Map(),inflight=new Map();let activeLoader;
try{
 await writeFile(path.join(work,'package.json'),'{"type":"module","private":true}');
 await writeFile(path.join(work,'index.html'),'<!doctype html><html lang="en"><head><meta charset="utf-8"><link rel="icon" href="data:,"><title>Search intent fixture</title><style>body{margin:24px}input,button{font:16px sans-serif;padding:12px}#search-results{border:1px solid;max-width:500px}.hit{display:block}</style></head><body><input id="search-q" aria-expanded="false" aria-controls="search-results"><div id="search-results" hidden></div><button id="outside">Outside</button><button id="abort">Abort page</button><script type="module" src="/entry.ts"></script></body></html>');
 await writeFile(path.join(work,'entry.ts'),`import {initSearch} from ${JSON.stringify(source.replaceAll('\\','/'))};\nconst scope=new AbortController();window.__nav=[];window.__source='/search-index.json?v=${digest}&case='+encodeURIComponent(new URL(location.href).searchParams.get('case'));initSearch(window.__source,target=>'/'+target.id,scope.signal,url=>window.__nav.push(url));document.querySelector('#abort').addEventListener('click',()=>scope.abort());window.__ready=true;`);
 const {build}=await import(pathToFileURL(path.join(builder,'node_modules/vite/dist/node/index.js')).href);
 await build({root:work,configFile:false,cacheDir:path.join(work,'.cache'),build:{outDir:dist,emptyOutDir:false,minify:false}});
 server=createServer(async(req,res)=>{try{
  const url=new URL(req.url,'http://fixture.invalid');
  if(url.pathname==='/search-index.json'){const id=url.searchParams.get('case');receipt.requests.push({id,url:req.url});pending.set(id,{response:res});return;}
  const relative=url.pathname==='/'?'index.html':decodeURIComponent(url.pathname).slice(1),file=path.resolve(dist,relative);assert(file.startsWith(dist+path.sep));const bytes=await readFile(file);res.writeHead(200,{'Content-Type':file.endsWith('.html')?'text/html; charset=utf-8':'text/javascript','Cache-Control':'no-store'});res.end(bytes);
 }catch(error){res.writeHead(404);res.end(String(error));}});
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const base=`http://127.0.0.1:${server.address().port}/`;
 child=spawn(findChrome(),['--headless=new','--no-sandbox','--disable-gpu','--no-first-run','--remote-debugging-port=0',`--user-data-dir=${profile}`,'about:blank'],{stdio:['ignore','ignore','pipe']});
 const port=await waitForDevToolsPort(child,profile),pages=await(await fetch(`http://127.0.0.1:${port}/json/list`)).json();connection=await new Connection(pages.find(page=>page.type==='page').webSocketDebuggerUrl).open();
 connection.listeners.push(message=>{const exception=runtimeException(message);if(exception!==undefined)receipt.exceptions.push(exception);if(message.method==='Page.frameNavigated'&&!message.params.frame.parentId)activeLoader=message.params.frame.loaderId;if(message.method==='Network.requestWillBeSent')inflight.set(message.params.requestId,{url:message.params.request.url,loader:message.params.loaderId});if(['Network.loadingFinished','Network.loadingFailed'].includes(message.method))inflight.delete(message.params.requestId);});
 const cdp=(method,params={})=>connection.cdp(method,params),evaluate=async expression=>{const result=await cdp('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});assert(!result.exceptionDetails,JSON.stringify(result.exceptionDetails));return result.result.value;};
 const wait=async(predicate,label)=>{const end=performance.now()+15000;while(performance.now()<end){if(await predicate())return;await delay(15);}throw Error(label);};
 const settle=async()=>{await wait(()=>[...inflight.values()].every(request=>request.loader!==activeLoader),'current-document browser resources did not finish');await evaluate('new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))');};
 const click=async selector=>{const point=await evaluate(`(()=>{const r=document.querySelector(${JSON.stringify(selector)}).getBoundingClientRect();return{x:r.x+r.width/2,y:r.y+r.height/2};})()`);await cdp('Input.dispatchMouseEvent',{type:'mousePressed',button:'left',clickCount:1,...point});await cdp('Input.dispatchMouseEvent',{type:'mouseReleased',button:'left',clickCount:1,...point});};
 const key=async value=>{const codes={Escape:27,Tab:9,Backspace:8,Enter:13,ArrowDown:40,ArrowUp:38},event={key:value,code:value,windowsVirtualKeyCode:codes[value]};await cdp('Input.dispatchKeyEvent',{type:'keyDown',...event});await cdp('Input.dispatchKeyEvent',{type:'keyUp',...event});};
 const start=async id=>{const before=await evaluate('performance.timeOrigin');await cdp('Page.navigate',{url:base+'?case='+encodeURIComponent(id)});await wait(()=>evaluate(`performance.timeOrigin!==${before}&&document.readyState==='complete'&&window.__ready===true`),'focused search fixture not ready');assert.equal(receipt.requests.filter(row=>row.id===id).length,0,'search fetched before intent');await click('#search-q');await cdp('Input.insertText',{text:'a'});await wait(()=>pending.has(id),'lazy request absent');};
 const release=async(id,outcome)=>{const response=pending.get(id).response;if(!response.destroyed){response.writeHead(outcome==='success'?200:503,{'Content-Type':'application/json','Cache-Control':'no-store','Content-Length':outcome==='success'?payload.length:0});response.end(outcome==='success'?payload:undefined);}await settle();};
 const state=()=>evaluate("({hidden:document.querySelector('#search-results').hidden,expanded:document.querySelector('#search-q').getAttribute('aria-expanded'),value:document.querySelector('#search-q').value,focus:document.activeElement.id,rows:document.querySelector('#search-results').childElementCount,nav:window.__nav,text:document.querySelector('#search-results').textContent})");
 await cdp('Page.enable');await cdp('Runtime.enable');await cdp('Network.enable');
 for(const outcome of ['success','error'])for(const dismissal of ['escape','outside','focus','empty','abort']){
  const id=outcome+'-'+dismissal;await start(id);
  if(dismissal==='escape')await key('Escape');else if(dismissal==='outside')await click('#outside');else if(dismissal==='focus')await key('Tab');else if(dismissal==='empty'){await evaluate("document.querySelector('#search-q').select()");await key('Backspace');assert.equal(await evaluate("document.querySelector('#search-q').value"),'','trusted Backspace must clear the selected query');}else await click('#abort');
  await release(id,outcome);const actual=await state();receipt.cases.push({id,...actual,requests:receipt.requests.filter(row=>row.id===id).length});assert.equal(actual.hidden,true,id+' late result reopened');assert.equal(actual.expanded,'false');assert.equal(actual.value,dismissal==='empty'?'':'a','dismissal changed the input value');assert.equal(actual.focus,['escape','empty'].includes(dismissal)?'search-q':dismissal==='abort'?'abort':'outside','dismissal stole focus');assert.deepEqual(actual.nav,[]);assert.equal(receipt.cases.at(-1).requests,1,'shared lazy load retried');
 }
 await start('keyboard');await release('keyboard','success');assert.equal((await state()).hidden,false);await key('Escape');await key('Enter');assert.deepEqual((await state()).nav,[],'hidden Enter navigated');await key('ArrowDown');await settle();assert.equal((await state()).hidden,false);await key('Enter');assert.equal((await state()).nav.at(-1),'/api/Basic');await key('Escape');await key('ArrowUp');await settle();await key('Enter');assert.equal((await state()).nav.at(-1),'/api/Bucket_Array');assert.equal(receipt.requests.filter(row=>row.id==='keyboard').length,1);receipt.cases.push({id:'keyboard',...await state()});
 await start('current-error');await release('current-error','error');const error=await state();assert.equal(error.hidden,false);assert.match(error.text,/Search could not load/);receipt.cases.push({id:'current-error',...error});assert.deepEqual(receipt.exceptions,[]);receipt.status='pass';console.log('Actual browser delayed search intent: '+receipt.cases.length+' cases passed');
}catch(error){receipt.status='fail';receipt.error=String(error);receipt.outstanding=[...inflight.values()];receipt.activeLoader=activeLoader;throw error;}
finally{
 await mkdir(path.dirname(path.resolve(output)),{recursive:true});await writeFile(output,JSON.stringify(receipt,null,2)+'\n');connection?.close();
 if(child){child.kill();await new Promise(resolve=>child.exitCode!==null||child.signalCode!==null?resolve():child.once('exit',resolve));}
 if(server){server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
 await rm(work,{recursive:true,force:true});await rm(profile,{recursive:true,force:true});
}
