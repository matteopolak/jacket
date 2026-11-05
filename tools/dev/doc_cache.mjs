#!/usr/bin/env node
/** Exercise real compiled docs under controlled synthetic valid JSON deployments and browser caching.
 * Example: node tools/dev/doc_cache.mjs /owned/production-site /owned/cache-receipt.json
 * Run through the shared heavy gate. This never changes or publishes the supplied site.
 */
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawn} from 'node:child_process';
import {createServer} from 'node:http';
import {readFile,mkdtemp,rm,writeFile,mkdir} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {setTimeout as delay} from 'node:timers/promises';
import {Connection,runtimeException} from './chrome_cdp.mjs';
import {findChrome,waitForDevToolsPort} from './chrome_startup.mjs';
import {peerWindow} from '../../website/docs/src/lib/render.ts';

const [directory,receiptPath]=process.argv.slice(2);assert(directory&&receiptPath,'supply production site directory and receipt JSON');
const root=path.resolve(directory),output=path.resolve(receiptPath),sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const api='api/Basic/align_forward',apiHTML=await readFile(path.join(root,`${api}.html`),'utf8');
const navURL=/data-source="([^"]+)"/u.exec(apiHTML)?.[1];assert(navURL,'real API peer navigation source absent');
const navPath=new URL(navURL,'http://fixture.invalid').pathname;
const sourceNav=JSON.parse(await readFile(path.join(root,navPath.slice(1)),'utf8'));
const current=sourceNav.data.entries.find(item=>item.id===api);assert(current);
const range=peerWindow(sourceNav.data.entries,current);
const peer=range.peers.find((item,index)=>index>=range.start&&index<range.end&&(index<range.first||index>=range.last));assert(peer,'fixture needs a hydrated peer outside the five SSR rows');
const sourceSearch=JSON.parse(await readFile(path.join(root,'search-index.json'),'utf8'));
const searchTarget=sourceSearch.data.entries.find(entry=>entry.id==='api/Basic');assert(searchTarget&&!searchTarget.preview,'fixture needs an existing summary-rendering search row');
function generation(marker,experimental,identity){
 const search=structuredClone(sourceSearch),navigation=structuredClone(sourceNav);
 search.data.entries.find(entry=>entry.id===searchTarget.id).summary+=` ${marker}`;
 navigation.data.entries.find(entry=>entry.id===peer.id).experimental=experimental;
 const searchBytes=Buffer.from(JSON.stringify(search)),navBytes=Buffer.from(JSON.stringify(navigation));
 return {marker,experimental,identity:sha(identity),searchBytes,navBytes,searchSHA:sha(searchBytes),navSHA:sha(navBytes)};
}
const first=generation('jacketcachefirstunique',false,'fixture-export-one');
const second=generation('jacketcachesecondunique',true,'fixture-export-two');
const third={...second,identity:sha('unrelated-fixture-html-export-change')};
const fourth=generation('jacketcacheabortunique',true,'fixture-abort-generation');
let active=first,holdNextSearch=false,held;
const serverRequests=[],exceptions=[],requests=[],responses=[],cacheIDs=new Set(),failed=[];
const types={'.html':'text/html; charset=utf-8','.json':'application/json','.js':'text/javascript','.mjs':'text/javascript','.css':'text/css','.svg':'image/svg+xml','.wasm':'application/wasm','.png':'image/png'};
const server=createServer(async(req,res)=>{
 const url=new URL(req.url,'http://fixture.invalid'),record={url:req.url,status:null,kind:null};serverRequests.push(record);
 try{
  let bytes,kind,contentType;
  if(url.pathname==='/__empty'){bytes=Buffer.from('<!doctype html><title>Owned empty cache context</title>');kind='html';contentType=types['.html'];}
  else if(url.pathname==='/search-index.json'){bytes=active.searchBytes;kind='search';contentType=types['.json'];}
  else if(url.pathname===navPath){bytes=active.navBytes;kind='navigation';contentType=types['.json'];}
  else{
   let relative=decodeURIComponent(url.pathname).replace(/^\/+|\/+$/gu,'')||'index.html';
   if(!path.extname(relative))relative+='.html';
   const file=path.resolve(root,relative);assert(file.startsWith(root+path.sep),'request escapes owned site');
   bytes=await readFile(file);kind=relative.endsWith('.html')?'html':'asset';contentType=types[path.extname(file)]??'application/octet-stream';
   if(kind==='html')bytes=Buffer.from(bytes.toString('utf8').replace(/data-search-source="[^"]*"/u,`data-search-source="/search-index.json?v=${active.searchSHA}"`).replace(/data-doc-identity="[^"]*"/u,`data-doc-identity="${active.identity}"`).replaceAll(navURL,`${navPath}?v=${active.navSHA}`));
  }
  record.kind=kind;record.sha256=sha(bytes);
  const send=()=>{if(res.destroyed){record.aborted=true;return;}record.status=200;res.writeHead(200,{'Content-Type':contentType,'Content-Length':bytes.length,'Cache-Control':kind==='html'?'no-store':'public, max-age=3600',ETag:`"${record.sha256}"`});res.end(bytes);};
  if(kind==='search'&&holdNextSearch){holdNextSearch=false;held={send,record};res.on('close',()=>{if(!res.writableFinished)record.aborted=true;});}
  else send();
 }catch(error){record.status=404;record.error=String(error);res.writeHead(404,{'Cache-Control':'no-store'});res.end('Owned fixture missing file');}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin=`http://127.0.0.1:${server.address().port}`;
const profile=await mkdtemp(path.join(tmpdir(),'jacket-doc-cache-'));
const child=spawn(findChrome(),['--headless=new','--no-sandbox','--disable-gpu','--no-first-run','--remote-debugging-port=0',`--user-data-dir=${profile}`,'about:blank'],{stdio:['ignore','ignore','pipe']});
let browser,connection,contextConnection,contextID;
const receipt={schemaVersion:1,input:{api,htmlSHA256:sha(apiHTML),originalSearchSHA256:sha(JSON.stringify(sourceSearch)),originalNavigationSHA256:sha(JSON.stringify(sourceNav)),hydratedPeer:peer.id},scope:'real compiled client; synthetic valid payload/HTML deployment overlays',checks:[],generations:[],serverRequests,requests,responses,exceptions,limits:['loopback HTTP with max-age3600 JSON/assets, no-store HTML, no compression; no service worker','root page base; genuine nested installed builds are checked separately','cache correctness only, no performance score or historical snapshot/integrity promise']};
async function wait(predicate,message,timeout=20000){const end=performance.now()+timeout;while(performance.now()<end){if(await predicate())return;await delay(20);}throw new Error(message);}
try{
 const port=await waitForDevToolsPort(child,profile),version=await(await fetch(`http://127.0.0.1:${port}/json/version`)).json();browser=await new Connection(version.webSocketDebuggerUrl).open();
 const pages=await(await fetch(`http://127.0.0.1:${port}/json/list`)).json();connection=await new Connection(pages.find(page=>page.type==='page').webSocketDebuggerUrl).open();
 connection.listeners.push(message=>{
  const exception=runtimeException(message);if(exception!==undefined)exceptions.push(exception);
  if(message.method==='Network.requestWillBeSent')requests.push({id:message.params.requestId,url:message.params.request.url,type:message.params.type});
  if(message.method==='Network.requestServedFromCache')cacheIDs.add(message.params.requestId);
  if(message.method==='Network.loadingFailed')failed.push({id:message.params.requestId,canceled:message.params.canceled??false,error:message.params.errorText});
  if(message.method==='Network.responseReceived'){const {response,requestId}=message.params;responses.push({id:requestId,url:response.url,status:response.status,diskCache:response.fromDiskCache??false,prefetchCache:response.fromPrefetchCache??false});}
 });
 const cdp=(method,params={})=>connection.cdp(method,params);
 const evaluate=async expression=>{const result=await cdp('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!result.exceptionDetails,JSON.stringify(result.exceptionDetails));return result.result.value;};
 await cdp('Page.enable');await cdp('Runtime.enable');await cdp('Network.enable');await cdp('Emulation.setDeviceMetricsOverride',{width:1280,height:900,deviceScaleFactor:1,mobile:false});
 const jsonCount=()=>serverRequests.filter(row=>['search','navigation'].includes(row.kind)).length;
 async function visit(label,data,expectedServerDelta){
  active=data;const before=jsonCount(),requestStart=requests.length;
  await cdp('Page.navigate',{url:`${origin}/${api}?cache-stage=${encodeURIComponent(label)}`});
  await wait(()=>evaluate(`document.readyState==='complete'&&location.search===${JSON.stringify('?cache-stage='+encodeURIComponent(label))}&&document.body.dataset.docIdentity===${JSON.stringify(data.identity)}&&document.querySelector('ul[data-loaded]')?.dataset.loaded===${JSON.stringify(navPath+'?v='+data.navSHA)}`),`${label} actual committed generation did not hydrate`);
  const state=await evaluate(`(()=>{const row=[...document.querySelectorAll('ul[data-loaded] a')].find(anchor=>new URL(anchor.href).pathname===${JSON.stringify('/'+peer.route)});return{found:!!row,source:document.body.dataset.searchSource,nav:document.querySelector('ul[data-loaded]').dataset.loaded,experimental:!!row?.querySelector('.dot.exp')};})()`);
  assert.equal(state.found,true,`${label} must inspect the actual hydrated peer row`);
  assert.equal(state.experimental,data.experimental,`${label} hydrated peer has stale experimental fact`);
  assert.equal(state.source,`/search-index.json?v=${data.searchSHA}`);assert.equal(state.nav,`${navPath}?v=${data.navSHA}`);
  assert(!requests.slice(requestStart).some(row=>new URL(row.url).pathname==='/search-index.json'),`${label} fetched search before input intent`);
  await evaluate(`(()=>{const input=document.querySelector('#search-q');input.value=${JSON.stringify(data.marker)};input.focus();input.dispatchEvent(new Event('input',{bubbles:true}));})()`);
  await wait(()=>evaluate(`(()=>{const row=document.querySelector('#search-results a[href="/api/Basic"]');return !!row&&row.textContent.includes(${JSON.stringify(data.marker)});})()`),`${label} actual search result did not reflect the generation marker`);
  assert.equal(jsonCount()-before,expectedServerDelta,`${label} unexpectedly refetched or reused server JSON`);
  const observed=responses.filter(row=>requests.slice(requestStart).some(request=>request.id===row.id)&&['/search-index.json',navPath].includes(new URL(row.url).pathname)).map(row=>({...row,servedFromCache:cacheIDs.has(row.id)}));
  if(expectedServerDelta===0){for(const pathname of ['/search-index.json',navPath])assert(observed.some(row=>new URL(row.url).pathname===pathname&&(row.diskCache||row.servedFromCache)),`${label} lacks CDP cache-hit evidence for ${pathname}`);}
  receipt.generations.push({label,...state,serverJSONRequests:jsonCount()-before,responses:observed});
 }
 await visit('v1 cold first intent',first,2);
 await visit('v1 warm revisit',first,0);
 await visit('v2 changed relevant payloads',second,2);
 await visit('v3 unrelated HTML/export identity only',third,0);
 assert.equal(second.searchSHA,third.searchSHA);assert.equal(second.navSHA,third.navSHA);assert.notEqual(second.identity,third.identity);
 receipt.checks.push('no eager search, actual summary-query result, hydrated peer outside five SSR rows','relevant payload changes use distinct SHA URLs and fresh JSON','unrelated HTML identity retains keys and actual CDP browser-cache hits without server JSON requests');
 // A fresh isolated cache demonstrates that digest queries do not retain historical server bytes.
 contextID=(await browser.cdp('Target.createBrowserContext')).browserContextId;
 const target=(await browser.cdp('Target.createTarget',{url:`${origin}/__empty`,browserContextId:contextID})).targetId;
 let targetPage;await wait(async()=>{targetPage=(await(await fetch(`http://127.0.0.1:${port}/json/list`)).json()).find(page=>page.id===target);return !!targetPage;},'fresh cache-context target missing');
 contextConnection=await new Connection(targetPage.webSocketDebuggerUrl).open();await contextConnection.cdp('Runtime.enable');
 await wait(async()=>{const state=await contextConnection.cdp('Runtime.evaluate',{expression:'location.origin',returnByValue:true});return state.result.value===origin;},'fresh cache context did not reach the owned HTTP origin');
 const oldQuery=`${origin}${navPath}?v=${first.navSHA}`;
 const uncached=await contextConnection.cdp('Runtime.evaluate',{expression:`(async()=>{const response=await fetch(${JSON.stringify(oldQuery)});return{status:response.status,text:await response.text()};})()`,awaitPromise:true,returnByValue:true});assert(!uncached.exceptionDetails,JSON.stringify(uncached.exceptionDetails));
 assert.equal(sha(uncached.result.value.text),second.navSHA);assert.notEqual(sha(uncached.result.value.text),first.navSHA);
 receipt.oldUncachedQuery={requestedDigest:first.navSHA,receivedDigest:sha(uncached.result.value.text),status:uncached.result.value.status,context:'new isolated empty browser cache',conclusion:'server ignores query when locating current bytes; cache identity is not immutable historical storage or integrity verification'};
 contextConnection.close();contextConnection=undefined;await browser.cdp('Target.disposeBrowserContext',{browserContextId:contextID});contextID=undefined;
 // Delay an otherwise valid search, then use the supported real SPA to discard its controller.
 active=fourth;await cdp('Page.navigate',{url:`${origin}/${api}`});await wait(()=>evaluate(`document.readyState==='complete'&&document.body.dataset.docIdentity===${JSON.stringify(fourth.identity)}&&!!document.querySelector('ul[data-loaded]')`),'abort fixture initial hydration failed');
 await evaluate("window.__cacheDocument='same-live-document';window.__cachePageLoads=0;document.addEventListener('astro:page-load',()=>window.__cachePageLoads++)");holdNextSearch=true;
 await evaluate("(()=>{const q=document.querySelector('#search-q');q.value='obsoletequerywithoutresults';q.focus();q.dispatchEvent(new Event('input',{bubbles:true}));})()");await wait(()=>!!held,'old search request did not reach controlled delay');
 const pendingURL=`${origin}/search-index.json?v=${fourth.searchSHA}`;
 const oldRequest=requests.findLast(row=>row.url===pendingURL);assert(oldRequest);
 const hit=await evaluate("(()=>{const a=document.querySelector('.side a[href=\"/api/Basic\"]');if(!a)return null;a.scrollIntoView({block:'nearest'});const r=a.getBoundingClientRect();return{x:r.x+r.width/2,y:r.y+r.height/2,hit:document.elementFromPoint(r.x+r.width/2,r.y+r.height/2)?.closest('a')?.href};})()");assert(hit&&hit.hit===`${origin}/api/Basic`,'SPA abort fixture requires one actual hit-testable module link');
 await cdp('Input.dispatchMouseEvent',{type:'mousePressed',x:hit.x,y:hit.y,button:'left',clickCount:1});await cdp('Input.dispatchMouseEvent',{type:'mouseReleased',x:hit.x,y:hit.y,button:'left',clickCount:1});
 await wait(()=>evaluate("location.pathname==='/api/Basic'&&window.__cachePageLoads>0&&document.readyState==='complete'"),'supported SPA navigation failed');assert.equal(await evaluate('window.__cacheDocument'),'same-live-document');
 await wait(()=>failed.some(row=>row.id===oldRequest.id&&row.canceled),'old search request was not canceled by page teardown');
 await evaluate(`(()=>{const q=document.querySelector('#search-q');q.value=${JSON.stringify(fourth.marker)};q.focus();q.dispatchEvent(new Event('input',{bubbles:true}));})()`);
 await wait(()=>evaluate(`document.querySelector('#search-results a[href="/api/Basic"]')?.textContent.includes(${JSON.stringify(fourth.marker)})`),'fresh page search did not render');
 held.send();await delay(100);assert(await evaluate(`!document.querySelector('#search-results').hidden&&document.querySelector('#search-results a[href="/api/Basic"]')?.textContent.includes(${JSON.stringify(fourth.marker)})`),'aborted old response replaced fresh page search');
 receipt.abortedSearch={request:oldRequest,networkFailure:failed.find(row=>row.id===oldRequest.id),server:held.record,documentRetained:true,freshResultPreserved:true};
 receipt.serviceWorkers=await evaluate('navigator.serviceWorker.getRegistrations().then(rows=>rows.length)');assert.equal(receipt.serviceWorkers,0);
 assert.deepEqual(exceptions,[],'actual browser runtime exceptions');receipt.status='pass';
}catch(error){receipt.status='fail';receipt.error=String(error);throw error;}
finally{
 for(const response of responses)response.servedFromCache=cacheIDs.has(response.id);
 await mkdir(path.dirname(output),{recursive:true});await writeFile(output,JSON.stringify(receipt,null,2)+'\n');
 contextConnection?.close();if(contextID)await browser?.cdp('Target.disposeBrowserContext',{browserContextId:contextID}).catch(()=>{});
 connection?.close();browser?.close();held?.send();child.kill();await new Promise(resolve=>child.exitCode!==null||child.signalCode!==null?resolve():child.once('exit',resolve));await rm(profile,{recursive:true,force:true});server.closeAllConnections();await new Promise(resolve=>server.close(resolve));
}
console.log(JSON.stringify({status:receipt.status,generations:receipt.generations.map(({label,serverJSONRequests,responses})=>({label,serverJSONRequests,cacheHits:responses.filter(row=>row.diskCache||row.servedFromCache).length})),oldUncachedQuery:receipt.oldUncachedQuery,abortedSearch:{canceled:receipt.abortedSearch.networkFailure.canceled,documentRetained:receipt.abortedSearch.documentRetained,freshResultPreserved:receipt.abortedSearch.freshResultPreserved},runtimeExceptions:exceptions.length,serviceWorkers:receipt.serviceWorkers},null,2));
console.log('Real compiled warm-cache deployment checks passed; see '+output);
