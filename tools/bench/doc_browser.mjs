#!/usr/bin/env node
/** Matched production-browser measurements. Validate only by default; execute inside exclusive gate.
 * node tools/bench/doc_browser.mjs --plan /tmp/doc-browser.json [--execute --out /tmp/owned-browser]
 */
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {createHash} from 'node:crypto';
import {mkdir,mkdtemp,readFile,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {setTimeout as delay} from 'node:timers/promises';
import {Connection,runtimeException} from '../dev/chrome_cdp.mjs';
import {waitForDevToolsPort} from '../dev/chrome_startup.mjs';
import {RequestLedger,summarizeRequests,summarizeObservations,observerSource} from './doc_browser_accounting.mjs';
import {contentProbeSource} from './doc_browser_content.mjs';
const option=name=>process.argv[process.argv.indexOf(name)+1];
assert(process.argv.includes('--plan'),'provide --plan PATH');const plan=JSON.parse(await readFile(option('--plan'),'utf8'));
const validName=value=>typeof value==='string'&&/^[a-z0-9][a-z0-9_-]*$/u.test(value);
assert.equal(plan.schema,'jacket-doc-browser/1');assert(plan.repetitions>=3);assert(plan.identities&&plan.equivalence&&plan.chrome);
assert(plan.variants?.length===2&&plan.scenarios?.length>0&&plan.profiles?.length>0);
for(const list of [plan.variants,plan.profiles,plan.scenarios])assert.equal(new Set(list.map(item=>item.name)).size,list.length,'duplicate plan name');
for(const variant of plan.variants){assert(validName(variant.name));new URL(variant.base);}
for(const profile of plan.profiles){assert(validName(profile.name)&&profile.width>0&&profile.height>0&&profile.scale>0&&profile.cpuRate>=1&&profile.frameBudgetMs>0);assert(profile.network&&Number.isFinite(profile.network.latency)&&Number.isFinite(profile.network.downloadThroughput)&&Number.isFinite(profile.network.uploadThroughput));}
const actionTypes=new Set(['goto','hover','focus','click','search','scroll','back','forward','wait','key']);
for(const scenario of plan.scenarios){assert(validName(scenario.name)&&scenario.cacheState&&scenario.actions?.length>0);assert.equal(scenario.actions[0].type,'goto','fresh sessions must begin with an explicit document visit');for(const action of scenario.actions){assert(actionTypes.has(action.type));if(action.type==='wait')assert(Number.isFinite(action.durationMs)&&action.durationMs>=0);if(['goto','click','back','forward','search'].includes(action.type))assert(action.expect?.selector&&action.expect?.path,'timed actions require an exact destination and useful-content predicate');}}
if(!process.argv.includes('--execute')){console.log(`Validated ${plan.scenarios.length} scenarios; no browser launched or measured`);process.exit(0);}
assert(process.argv.includes('--out'),'execution requires an absent owned --out');const out=path.resolve(option('--out'));await mkdir(out,{recursive:false});await writeFile(path.join(out,'plan.json'),JSON.stringify(plan,null,2));
const results=[];
async function session(variant,profile,scenario,repetition){
 const directory=path.join(out,`${profile.name}-${scenario.name}-${repetition}-${variant.name}`);await mkdir(directory);
 const browserProfile=await mkdtemp(path.join(tmpdir(),'jacket-doc-bench-'));
 const child=spawn(plan.chrome,['--headless=new','--no-sandbox','--no-first-run','--remote-debugging-port=0',`--user-data-dir=${browserProfile}`,'about:blank'],{stdio:['ignore','ignore','pipe']});
 let connection;const ledger=new RequestLedger(),exceptions=[],actions=[],inputs=[],workerSetups=[];let currentAction=-1;
 const receipt={variant:variant.name,profile,scenario:scenario.name,cacheState:scenario.cacheState,repetition,actions,inputs,requests:ledger.rows,exceptions,workerTargets:[],notes:['CDP/getResponseBody hashes collected after actions. PerformanceObserver and polling add matched measurement overhead.','Each session starts with a fresh profile; warm/new/prefetch/history preparation must be explicit preceding actions.']};
 try{
  const port=await waitForDevToolsPort(child,browserProfile),version=await(await fetch(`http://127.0.0.1:${port}/json/version`)).json();receipt.browser={product:version.Browser,userAgent:version['User-Agent'],'protocol-version':version['Protocol-Version']};
  const pages=await(await fetch(`http://127.0.0.1:${port}/json/list`)).json();connection=await new Connection(pages.find(page=>page.type==='page').webSocketDebuggerUrl).open();
  connection.listeners.push(message=>{const error=runtimeException(message);if(error!==undefined)exceptions.push(error);if(message.method==='Runtime.bindingCalled'&&message.params.name==='__docBenchmarkInput')inputs.push({...JSON.parse(message.params.payload),hostReceived:performance.now(),action:currentAction});const previous=ledger.rows.length;ledger.event(message.method,message.params??{},message.sessionId);if(ledger.rows.length>previous)ledger.rows.at(-1).action=currentAction;});
  const cdp=(method,params={},sessionId)=>connection.cdp(method,params,sessionId);
  // Pause owned dedicated workers until their network domain is observable, including imports/Wasm.
  connection.listeners.push(message=>{
   if(message.method!=='Target.attachedToTarget')return;
   const {sessionId,targetInfo}=message.params;
   receipt.workerTargets.push({sessionId,type:targetInfo.type,url:targetInfo.url});
   const setup=(async()=>{
    await cdp('Network.enable',{maxTotalBufferSize:100_000_000,maxResourceBufferSize:20_000_000},sessionId);
    await cdp('Runtime.enable',{},sessionId);
    await cdp('Network.setCacheDisabled',{cacheDisabled:false},sessionId);
    assert(profile.cpuRate===1&&profile.network.latency===0&&profile.network.downloadThroughput===-1&&profile.network.uploadThroughput===-1,'worker first-use ledger requires the explicit unthrottled profile; this Chrome worker target does not support Network.emulateNetworkConditions');
    await cdp('Runtime.runIfWaitingForDebugger',{},sessionId);
   })().catch(error=>{exceptions.push(`worker network setup: ${error}`);});
   workerSetups.push(setup);
  });
  await cdp('Target.setAutoAttach',{autoAttach:true,waitForDebuggerOnStart:true,flatten:true});
  const evaluate=async expression=>{const response=await cdp('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!response.exceptionDetails,JSON.stringify(response.exceptionDetails));return response.result.value;};
  await cdp('Page.enable');await cdp('Runtime.enable');await cdp('Runtime.addBinding',{name:'__docBenchmarkInput'});await cdp('Network.enable',{maxTotalBufferSize:100_000_000,maxResourceBufferSize:20_000_000});
  await cdp('Network.clearBrowserCache');await cdp('Network.setCacheDisabled',{cacheDisabled:false});await cdp('Network.setBypassServiceWorker',{bypass:profile.bypassServiceWorker??false});
  await cdp('Emulation.setDeviceMetricsOverride',{width:profile.width,height:profile.height,deviceScaleFactor:profile.scale,mobile:false});await cdp('Emulation.setCPUThrottlingRate',{rate:profile.cpuRate});
  await cdp('Network.emulateNetworkConditions',{offline:false,...profile.network});await cdp('Emulation.setEmulatedMedia',{features:[{name:'prefers-color-scheme',value:profile.theme??'light'},{name:'prefers-reduced-motion',value:profile.motion??'no-preference'}]});
  await cdp('Page.addScriptToEvaluateOnNewDocument',{source:observerSource});
  async function waitContent(expected){
   const url=new URL(expected.path,variant.base),deadline=performance.now()+(expected.timeoutMs??20_000);
   const expression=`(${contentProbeSource})(${JSON.stringify({...expected,path:url.pathname,hash:url.hash})})`;
   let domReady;
   while(performance.now()<deadline){
    assert.deepEqual(exceptions,[],'browser/worker instrumentation failed');
    const state=await evaluate(expression);if(state.domReady&&!domReady)domReady=state;
    if(state.ready){
     const settled=await evaluate(`new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(()=>resolve(${expression}))))`);
     if(settled.ready)return{domReady,contentReady:settled};
    }
    await delay(10);
   }
   throw new Error(`content-ready deadline: ${expected.path} ${expected.selector}`);
  }
  async function point(selector){const value=await evaluate(`(()=>{const node=document.querySelector(${JSON.stringify(selector)});if(!node)return null;node.scrollIntoView({block:'center'});const r=node.getBoundingClientRect();return{x:r.x+r.width/2,y:r.y+r.height/2};})()`);assert(value,`missing action target ${selector}`);return value;}
  for(const [index,action]of scenario.actions.entries()){
   currentAction=index;const row={action,index,startRequest:ledger.rows.length};actions.push(row);const hostStart=performance.now();
   row.before=await evaluate('({origin:performance.timeOrigin,time:performance.now(),path:location.pathname,hash:location.hash,scrollY})');
   if(action.type==='goto')await cdp('Page.navigate',{url:new URL(action.path,variant.base).href});
   if(action.type==='hover'){await cdp('Input.dispatchMouseEvent',{type:'mouseMoved',...await point(action.selector)});await delay(action.durationMs??250);}
   if(action.type==='focus'){await evaluate(`document.querySelector(${JSON.stringify(action.selector)}).focus()`);await delay(action.durationMs??250);}
   if(action.type==='click'){const position=await point(action.selector);await cdp('Input.dispatchMouseEvent',{type:'mouseMoved',...position});await cdp('Input.dispatchMouseEvent',{type:'mousePressed',button:'left',clickCount:1,...position});await cdp('Input.dispatchMouseEvent',{type:'mouseReleased',button:'left',clickCount:1,...position});}
   if(action.type==='search'){await evaluate(`document.querySelector(${JSON.stringify(action.selector)}).focus()`);await cdp('Input.insertText',{text:action.text});}
   if(action.type==='key'){await cdp('Input.dispatchKeyEvent',{type:'keyDown',key:action.key,code:action.code??action.key});await cdp('Input.dispatchKeyEvent',{type:'keyUp',key:action.key,code:action.code??action.key});}
   if(action.type==='back'||action.type==='forward'){const history=await cdp('Page.getNavigationHistory'),entry=history.entries[history.currentIndex+(action.type==='back'?-1:1)];assert(entry,'missing history destination');await cdp('Page.navigateToHistoryEntry',{entryId:entry.id});}
   if(action.type==='wait')await delay(action.durationMs);
   if(action.type==='scroll'){await evaluate('window.__docMetrics.frames=[];window.__docMetrics.scrolling=true');for(let step=0;step<(action.steps??30);step++){await cdp('Input.dispatchMouseEvent',{type:'mouseWheel',x:profile.width/2,y:profile.height/2,deltaX:0,deltaY:action.deltaY??120});await delay(action.intervalMs??16);}await delay(200);await evaluate('window.__docMetrics.scrolling=false');}
   if(action.expect){const readiness=await waitContent(action.expect);row.domReady=readiness.domReady;row.contentReady=readiness.contentReady;if(action.expect.activeId!==undefined)assert.equal(row.contentReady.active.id,action.expect.activeId,'focus mismatch');if(action.expect.scrollY!==undefined)assert(Math.abs(row.contentReady.scrollY-action.expect.scrollY)<=(action.expect.scrollTolerance??2),'history/anchor scroll mismatch');if(action.expect.minimumDomNodes!==undefined)assert(row.contentReady.domNodes>=action.expect.minimumDomNodes,'required content missing');}
   row.hostActionToContentReadyMs=row.contentReady?performance.now()-hostStart:null;
   const input=inputs.find(value=>value.action===index&&value.event===({click:'click',search:'input',key:'keydown'}[action.type]));
   row.trustedInput=input??null;
   row.browserInputToDomReadyMs=input&&row.domReady?.origin===input.origin?row.domReady.time-input.time:null;
   row.browserActionToDomReadyMs=row.domReady?.origin===row.before.origin?row.domReady.time-row.before.time:null;
   row.browserInputToContentReadyMs=input&&row.contentReady?.origin===input.origin?row.contentReady.time-input.time:null;
   row.hostInputReceivedToContentReadyMs=input?performance.now()-input.hostReceived:null;
   row.browserActionToContentReadyMs=row.contentReady?.origin===row.before.origin?row.contentReady.time-row.before.time:null;
   row.metrics=await evaluate("({observations:window.__docMetrics,resources:performance.getEntriesByType('resource').map(entry=>entry.toJSON()),navigation:performance.getEntriesByType('navigation').map(entry=>entry.toJSON()),fontStatus:document.fonts.status,fonts:[...document.fonts].map(font=>({family:font.family,status:font.status})),computedFonts:{body:getComputedStyle(document.body).fontFamily,code:document.querySelector('pre')?getComputedStyle(document.querySelector('pre')).fontFamily:null},serviceWorker:{controller:navigator.serviceWorker?.controller?.scriptURL??null},domNodes:document.getElementsByTagName('*').length})");
   row.observationSummary=summarizeObservations(row.metrics.observations,profile.frameBudgetMs);
   row.endRequest=ledger.rows.length;
   await writeFile(path.join(directory,'receipt.json'),JSON.stringify(receipt,null,2));
  }
  await Promise.all(workerSetups);
  await cdp('DOM.enable');await cdp('CSS.enable');
  const {root}=await cdp('DOM.getDocument');receipt.platformFonts={};
  for(const selector of ['body','main h1','pre']){const {nodeId}=await cdp('DOM.querySelector',{nodeId:root.nodeId,selector});if(nodeId)receipt.platformFonts[selector]=await cdp('CSS.getPlatformFontsForNode',{nodeId});}
  for(const row of ledger.rows){if(row.redirect||row.failure||!row.finished||!/^https?:/u.test(row.url))continue;try{const body=await cdp('Network.getResponseBody',{requestId:row.id},row.sessionId);const decoded=Buffer.from(body.body,body.base64Encoded?'base64':'utf8');row.decodedBodyBytes=decoded.length;row.bodySha256=createHash('sha256').update(decoded).digest('hex');}catch(error){row.bodyHashUnavailable=String(error);}}
  receipt.summary=summarizeRequests(ledger.rows);
  const bodies=new Map();for(const request of ledger.rows){if(request.bodySha256){const group=bodies.get(request.bodySha256)??[];group.push({url:request.url,action:request.action,encodedTransferBytes:request.encodedTransferBytes,cache:!!(request.diskCache||request.prefetchCache||request.servedFromCache)});bodies.set(request.bodySha256,group);}}
  receipt.repeatedBodies=[...bodies].filter(([,rows])=>rows.length>1).map(([sha256,requests])=>({sha256,requests}));
  receipt.refetchedHashedAssets=ledger.rows.filter(row=>/\/_astro\//u.test(new URL(row.url).pathname)&&row.encodedTransferBytes>0&&ledger.rows.some(other=>other!==row&&other.url===row.url&&other.started<row.started));
  receipt.unconsumedSpeculationCandidates=ledger.rows.filter(request=>{
   const action=actions[request.action]?.action;
   const purpose=Object.entries(request.requestHeaders??{}).find(([key])=>/^(?:sec-)?purpose$/iu.test(key))?.[1];
   const speculative=purpose?.includes('prefetch')||(['hover','focus'].includes(action?.type)&&request.mime?.includes('text/html'));
   if(!speculative)return false;
   const destination=new URL(request.url);
   return !actions.some(later=>later.index>request.action&&later.contentReady&&later.contentReady.path===destination.pathname);
  });
  assert.deepEqual(exceptions,[],'browser exceptions occurred');receipt.status='pass';return receipt;
 }catch(error){receipt.status='fail';receipt.error=String(error);receipt.summary=summarizeRequests(ledger.rows);throw error;}
 finally{await writeFile(path.join(directory,'receipt.json'),JSON.stringify(receipt,null,2));connection?.close();child.kill();await new Promise(resolve=>!child.pid||child.exitCode!==null||child.signalCode!==null?resolve():child.once('exit',resolve));await rm(browserProfile,{recursive:true,force:true});}
}
for(let repetition=0;repetition<plan.repetitions;repetition++)for(const profile of plan.profiles)for(const scenario of plan.scenarios)for(const variant of repetition%2?[...plan.variants].reverse():plan.variants){results.push(await session(variant,profile,scenario,repetition));await writeFile(path.join(out,'results.json'),JSON.stringify({schema:plan.schema,results},null,2));}
console.log(`Recorded ${results.length} matched production-browser sessions; inspect raw receipts before interpreting differences`);
