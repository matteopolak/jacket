#!/usr/bin/env node
/** Real request/lifecycle acceptance for the supported Astro router. No npm or deployed assets.
 * Example: node tools/dev/doc_spa.mjs http://127.0.0.1:8000/
 */
import assert from 'node:assert/strict';
import {spawn,spawnSync} from 'node:child_process';
import {mkdtemp,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {setTimeout as delay} from 'node:timers/promises';
import {Connection,runtimeException} from './chrome_cdp.mjs';
import {waitForDevToolsPort} from './chrome_startup.mjs';
import {productionDiagnosticCases} from './diagnostic_fixture_checks.mjs';
import {preparePointer} from '../bench/doc_browser_pointer.mjs';
const base=new URL(process.argv[2]||'http://127.0.0.1:8000/');
const development=process.argv.includes('--dev'),liveMain=process.argv.includes('--live-main');
const stateColors=process.argv.includes('--state-colors'),traceTransitions=process.argv.includes('--trace-transitions'),pointerOnly=process.argv.includes('--native-pointer');
const chrome=process.env.CHROME_PATH||(process.platform==='darwin'?'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome':['google-chrome','google-chrome-stable','chromium','chromium-browser'].find(p=>spawnSync('which',[p]).status===0));
assert(chrome,'Install Chrome or set CHROME_PATH');
const profile=await mkdtemp(path.join(tmpdir(),'jacket-spa-'));
const child=spawn(chrome,['--headless=new','--no-sandbox','--disable-gpu','--no-first-run','--remote-debugging-port=0',`--user-data-dir=${profile}`,'about:blank'],{stdio:['ignore','ignore','pipe']});
let connection,browser,readPage,scriptsDisabled=false;
const requests=[],responses=[],exceptions=[],workerTargets=new Set(),destroyedWorkers=new Set();
const receipt={mode:development?'development':'production',scope:pointerOnly?'native-pointer':stateColors?'state-colors':'full-spa',checks:[],requests:[],interactions:[],storage:[],diagnostics:[],workers:{created:0,terminated:0}};
async function wait(predicate,message,timeout=15000){const end=performance.now()+timeout;while(performance.now()<end){if(await predicate())return;await delay(25);}throw new Error(message);}
const probe=String.raw`(()=>{
 const NativeWorker=window.Worker;
 window.__spaReceipt={marker:'preserved',pages:0,swaps:0,created:0,terminated:0,live:0,paintThemes:[]};
 window.__bootstrapOrder={body:null,main:null,frame:null,domContentLoaded:null};
 const sample=()=>({installed:typeof window.jacketBootstrap?.prepare==='function',js:document.documentElement?.classList.contains('js')??false,readyState:document.readyState,time:performance.now()});
 const parser=new MutationObserver(()=>{if(document.body&&!window.__bootstrapOrder.body)window.__bootstrapOrder.body=sample();if(document.querySelector('main')&&!window.__bootstrapOrder.main)window.__bootstrapOrder.main=sample();if(window.__bootstrapOrder.body&&window.__bootstrapOrder.main)parser.disconnect();});
 parser.observe(document,{subtree:true,childList:true});
 requestAnimationFrame(function firstBodyFrame(){if(!document.body){requestAnimationFrame(firstBodyFrame);return;}window.__bootstrapOrder.frame=sample();});
 document.addEventListener('DOMContentLoaded',()=>{window.__bootstrapOrder.domContentLoaded=sample();window.__bootstrapIdentity=window.jacketBootstrap;},{once:true});
 window.Worker=class extends NativeWorker{
  constructor(...args){super(...args);window.__spaReceipt.created++;window.__spaReceipt.live++;}
  terminate(){window.__spaReceipt.terminated++;window.__spaReceipt.live--;super.terminate();}
 };
 document.addEventListener('astro:page-load',()=>window.__spaReceipt.pages++);
 document.addEventListener('astro:after-swap',()=>{window.__spaReceipt.swaps++;requestAnimationFrame(()=>window.__spaReceipt.paintThemes.push(document.documentElement.getAttribute('data-theme')));});
})();`;
const transitionProbe=String.raw`(()=>{
 const traces=window.__transitionTrace=[],labels=new WeakMap(),ids=new WeakMap();let next=0;
 const identity=value=>{if(!ids.has(value))ids.set(value,++next);return ids.get(value);};
 if(typeof ViewTransition!=='undefined'){
  for(const name of ['ready','finished','updateCallbackDone']){const descriptor=Object.getOwnPropertyDescriptor(ViewTransition.prototype,name);if(!descriptor?.get)continue;Object.defineProperty(ViewTransition.prototype,name,{...descriptor,get(){const promise=descriptor.get.call(this);labels.set(promise,{id:identity(this),stage:name});return promise;}});}
  const skip=ViewTransition.prototype.skipTransition;ViewTransition.prototype.skipTransition=function(...args){traces.push({event:'skip',id:identity(this),time:performance.now(),path:location.pathname});return skip.apply(this,args);};
 }
 const final=Promise.prototype.finally;Promise.prototype.finally=function(callback){const result=final.call(this,callback),label=labels.get(this);if(label)labels.set(result,{...label,derived:'finally'});return result;};
 window.addEventListener('unhandledrejection',event=>traces.push({event:'unhandledrejection',time:performance.now(),path:location.pathname,name:event.reason?.name,message:event.reason?.message,stack:event.reason?.stack,promise:labels.get(event.promise)??null}));
})();`;
spaRun: try{
 const port=await waitForDevToolsPort(child,profile);
 const version=await(await fetch(`http://127.0.0.1:${port}/json/version`)).json();
 browser=await new Connection(version.webSocketDebuggerUrl).open();
 browser.listeners.push(message=>{if(message.method==='Target.targetCreated'&&message.params.targetInfo.type==='worker')workerTargets.add(message.params.targetInfo.targetId);if(message.method==='Target.targetDestroyed'&&workerTargets.has(message.params.targetId))destroyedWorkers.add(message.params.targetId);});
 await browser.cdp('Target.setDiscoverTargets',{discover:true});
 const pages=await(await fetch(`http://127.0.0.1:${port}/json/list`)).json();
 connection=await new Connection(pages.find(page=>page.type==='page').webSocketDebuggerUrl).open();
 connection.listeners.push(message=>{
  const error=runtimeException(message);if(error!==undefined)exceptions.push(error);
  if(message.method==='Network.requestWillBeSent')requests.push({url:message.params.request.url,type:message.params.type,id:message.params.requestId});
  if(message.method==='Network.responseReceived')responses.push({url:message.params.response.url,mime:message.params.response.mimeType,status:message.params.response.status,prefetchCache:message.params.response.fromPrefetchCache??false,diskCache:message.params.response.fromDiskCache??false});
 });
 const cdp=(method,params={})=>connection.cdp(method,params);
 const evaluate=async expression=>{const result=await cdp('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!result.exceptionDetails,JSON.stringify(result.exceptionDetails));return result.result.value;};readPage=evaluate;
 await cdp('Page.enable');await cdp('Runtime.enable');await cdp('Network.enable');await cdp('Page.bringToFront');await cdp('Emulation.setFocusEmulationEnabled',{enabled:true});
 await cdp('Emulation.setDeviceMetricsOverride',{width:1280,height:900,deviceScaleFactor:1,mobile:false});
 await cdp('Page.addScriptToEvaluateOnNewDocument',{source:probe+(traceTransitions?transitionProbe:'')});
 await cdp('Page.navigate',{url:base.href});
 await wait(()=>evaluate("document.readyState==='complete'&&window.__spaReceipt?.pages===1&&!!document.querySelector('main')"),'initial Astro page enhancement did not finish');
 await delay(300);
 await wait(()=>evaluate('window.__bootstrapOrder?.frame!==null'),'first parsed-body frame was not observed');
 const proveLiveMain=async()=>{
  const modulePath=new URL('api/Basic',base).pathname,printPath=new URL('api/Basic/print',base).pathname;
  await cdp('Page.navigate',{url:new URL('api/Basic',base).href});
  await wait(()=>evaluate(`location.pathname===${JSON.stringify(modulePath)}&&document.readyState==='complete'&&window.__spaReceipt?.pages===1`),'live-main initial module failed');
  await evaluate('document.fonts.ready');
  await evaluate(`window.__liveMainRail=document.querySelector('.side')`);
  const click=async({x,y})=>{for(const type of ['mouseMoved','mousePressed','mouseReleased'])await cdp('Input.dispatchMouseEvent',{type,x,y,...(type==='mouseMoved'?{}:{button:'left',clickCount:1})});};
  const sample=()=>evaluate(`(()=>{const main=document.querySelector('main'),a=main?.querySelector('.crumbs a[href="${modulePath}"]'),r=a?.getBoundingClientRect(),s=document.querySelector('.side'),b=s.getBoundingClientRect(),style=getComputedStyle(main),snapshot=getComputedStyle(document.documentElement,'::view-transition-new(main)'),live=style.animationName==='page-in',x=r?r.x+r.width/2:0,y=r?r.y+r.height/2:0,hit=document.elementFromPoint(x,y);return{time:performance.now(),path:location.pathname,live,animation:live?style.animationName:snapshot.animationName,duration:live?style.animationDuration:snapshot.animationDuration,opacity:Number(live?style.opacity:snapshot.opacity),transform:live?style.transform:snapshot.transform,nativeCapture:document.activeViewTransition!==null,mainCapture:style.viewTransitionName,point:{x,y},hit:!!a&&!!hit&&(a===hit||a.contains(hit)),hitTag:hit?.tagName,railSame:s===window.__liveMainRail,railScroll:s.scrollTop,railRect:[b.x,b.y,b.width,b.height]}})()`);
  const initial=await preparePointer(evaluate,`main a[href="${printPath}"]`);await click(initial);
  let during;
  await wait(async()=>{during=await sample();return during.path===printPath&&during.animation==='page-in'&&during.opacity>.05&&during.opacity<.95;},'incoming visible mid-fade was not observed',5000);
  receipt.liveMain={during};
  // Deliberately do not wait for stationary geometry: this must be a real mid-animation gesture.
  await click(during.point);
  await wait(()=>evaluate(`location.pathname===${JSON.stringify(modulePath)}`),'trusted incoming mid-fade breadcrumb click was ignored',5000);
  receipt.liveMain.after=await sample();
  assert(during.live,'entrance must animate the live main, not a snapshot');assert.equal(during.nativeCapture,false);assert.equal(during.mainCapture,'none');assert.equal(during.duration,'0.17s');assert(during.hit,'visible incoming link is not the pointer target');assert(during.railSame&&receipt.liveMain.after.railSame,'live sidebar DOM was replaced');
  assert.deepEqual(exceptions,[]);receipt.checks.push('trusted incoming main breadcrumb remains usable during identical170ms live entrance');
 };
 if(liveMain){await proveLiveMain();console.log(JSON.stringify(receipt,null,2));break spaRun;}
 receipt.bootstrap=await evaluate(`(()=>{const scripts=[...document.querySelectorAll('script[data-doc-bootstrap]')],script=scripts[0],chrome=[['html',document.documentElement],['header',document.querySelector('.top')],['sidebar',document.querySelector('.side')],['footer',document.querySelector('.foot')]],inlineScopeRules=[];const walk=rules=>{for(const rule of rules){if(rule.selectorText?.includes('data-astro-transition-scope'))inlineScopeRules.push({selector:rule.selectorText,style:rule.style?.cssText??''});if(rule.cssRules)walk(rule.cssRules);}};for(const style of document.querySelectorAll('style'))if(style.sheet)walk(style.sheet.cssRules);return{count:scripts.length,url:script?.src??'',inlineEmpty:!(script?.textContent??'').trim(),inlineBytes:new TextEncoder().encode(script?.textContent??'').length,inHead:!!script&&document.head.contains(script),async:script?.hasAttribute('async'),defer:script?.hasAttribute('defer'),type:script?.type??'',installed:typeof window.jacketBootstrap?.prepare==='function',order:window.__bootstrapOrder,chrome:chrome.map(([name,node])=>({name,present:!!node,capture:node?getComputedStyle(node).viewTransitionName:null,transitionAttributes:node?node.getAttributeNames().filter(name=>name.startsWith('data-astro-transition')):[]})),mainCapture:getComputedStyle(document.querySelector('main')).viewTransitionName,inlineScopeRules};})()`);
 assert.equal(receipt.bootstrap.count,1,'production head must own exactly one bootstrap');
 assert(receipt.bootstrap.url,'bootstrap must be an external hashed asset rather than repeated inline page code');
 assert(receipt.bootstrap.inlineEmpty,'external bootstrap must have no inline program');
 assert(receipt.bootstrap.inHead,'bootstrap must block parsing in the head');
 assert.equal(receipt.bootstrap.async,false,'bootstrap must not be async');assert.equal(receipt.bootstrap.defer,false,'bootstrap must not be deferred');
 assert(['','text/javascript','application/javascript'].includes(receipt.bootstrap.type.toLowerCase()),'bootstrap must be a synchronous classic script');
 const bootstrapURL=new URL(receipt.bootstrap.url);assert.equal(bootstrapURL.origin,base.origin);if(development){assert(bootstrapURL.pathname.endsWith('/src/browser/bootstrap.js'),'development bootstrap must use its actual Vite source URL');assert.equal(bootstrapURL.search,'?no-inline','development must retain Vite asset-query identity');}else{assert(bootstrapURL.pathname.startsWith(new URL('_astro/',base).pathname),'bootstrap asset escaped the published base');assert.match(path.basename(bootstrapURL.pathname),/^bootstrap\.[A-Za-z0-9_-]{8,}\.js$/u,'bootstrap URL must use the compiled Vite content hash');assert.equal(bootstrapURL.search,'');}assert.equal(bootstrapURL.hash,'');
 assert(receipt.bootstrap.installed,'actual bootstrap did not execute');
 for(const stage of ['body','main','frame','domContentLoaded']){assert(receipt.bootstrap.order[stage]?.installed,`bootstrap installed after observed ${stage}`);assert(receipt.bootstrap.order[stage]?.js,`prepaint js state missing at observed ${stage}`);}
 assert.equal(receipt.bootstrap.order.body.readyState,'loading','body sample was not taken during parsing');
 for(const chrome of receipt.bootstrap.chrome){assert(chrome.present,`${chrome.name} chrome missing`);assert.equal(chrome.capture,'none',`${chrome.name} chrome acquired a native capture`);assert.deepEqual(chrome.transitionAttributes,[],`${chrome.name} retains obsolete Astro transition directives`);}
 assert.equal(receipt.bootstrap.mainCapture,'none','live main acquired a native capture');assert.deepEqual(receipt.bootstrap.inlineScopeRules,[],'per-page Astro transition scope CSS remains in static chrome');
 assert.equal(requests.filter(row=>row.url===bootstrapURL.href).length,1,'initial bootstrap was requested more or less than once');
 receipt.checks.push(`${development?'Vite source':'Vite-hashed'} synchronous classic bootstrap executes before parsed body/main and first body frame; static chrome has no generated transition scopes`);
 receipt.versionSlot=await evaluate(`(async()=>{const button=document.querySelector('#ver-button');if(!button)return null;await document.fonts.ready;const initial=button.disabled,rows=[];try{for(const disabled of [true,false]){button.disabled=disabled;await new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));rows.push({disabled,boxes:[...document.querySelectorAll('.top .search-wrap,.top .site-nav,.top .site-nav > *')].map(node=>{const r=node.getBoundingClientRect();return[r.x,r.y,r.width,r.height];})});}return rows;}finally{button.disabled=initial;}})()`);
 if(receipt.versionSlot){const [disabled,enabled]=receipt.versionSlot;assert.equal(disabled.boxes.length,enabled.boxes.length);for(let row=0;row<disabled.boxes.length;row++)for(let axis=0;axis<4;axis++)assert(Math.abs(disabled.boxes[row][axis]-enabled.boxes[row][axis])<=.1,'version chevron enable/disable moved the live header');receipt.checks.push('optional version availability keeps its chevron slot and surrounding header geometry');}

 assert.deepEqual(responses.filter(row=>row.mime==='text/html').map(row=>row.url),[base.href],'initial load eagerly fetched other catalogue pages');
 assert.equal(requests.filter(row=>/runtime\/|examples\/.*\.json|search-index\.json/u.test(new URL(row.url).pathname)).length,0,'runtime/example/search data loaded without interaction');
 receipt.checks.push('no eager catalogue, search, example or compiler requests');
 receipt.crossDocumentOptIns=await evaluate(String.raw`(()=>{const found=[];const enabled=rule=>rule.cssText.startsWith('@view-transition')&&/navigation:\s*auto/u.test(rule.cssText);if(!enabled({cssText:'@view-transition { navigation: auto; }'}))throw Error('cross-document CSS probe calibration failed');const walk=rules=>{for(const rule of rules){if(enabled(rule))found.push(rule.cssText);if(rule.cssRules)walk(rule.cssRules);}};for(const sheet of document.styleSheets)walk(sheet.cssRules);return found;})()`);
 assert.deepEqual(receipt.crossDocumentOptIns,[],'ClientRouter main transitions must not compete with legacy native cross-document opt-in');
 receipt.checks.push('ClientRouter is the sole animated document-navigation owner');
 const pathOf=relative=>new URL(relative,base).pathname;
 const target=async relative=>{const pathname=pathOf(relative);return evaluate(`(async()=>{const anchor=[...document.querySelectorAll('a[data-astro-prefetch="hover"]')].find(a=>new URL(a.href).pathname===${JSON.stringify(pathname)});if(!anchor)return null;const ancestors=[];for(let p=anchor.parentElement;p;p=p.parentElement)if(p.tagName==='DETAILS'){p.open=true;ancestors.push(p);}anchor.getBoundingClientRect();await new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));await Promise.all(ancestors.flatMap(node=>node.getAnimations({subtree:true})).map(animation=>animation.finished));anchor.scrollIntoView({block:'nearest'});const r=anchor.getBoundingClientRect();return {href:anchor.href,x:r.x+r.width/2,y:r.y+r.height/2};})()`);};
 const go=async relative=>{if(stateColors)await wait(()=>evaluate("document.getAnimations().every(animation=>animation.playState!=='running')"),'focused navigation did not settle');const destination=new URL(relative,base);const before=await evaluate('window.__spaReceipt.pages'),bootstrapRequests=requests.filter(row=>row.url===bootstrapURL.href).length;await evaluate(`(()=>{const a=document.createElement('a');a.href=${JSON.stringify(destination.href)};a.dataset.astroPrefetch='hover';document.body.append(a);a.click();a.remove();})()`);await wait(()=>evaluate(`location.pathname===${JSON.stringify(destination.pathname)}&&window.__spaReceipt?.pages>${before}&&document.readyState==='complete'`),`SPA navigation failed: ${relative}`);assert.equal(await evaluate('window.__spaReceipt.marker'),'preserved','navigation replaced the JavaScript document');const bootstrap=await evaluate(`({same:window.jacketBootstrap===window.__bootstrapIdentity,url:document.querySelector('script[data-doc-bootstrap]')?.src,count:document.querySelectorAll('script[data-doc-bootstrap]').length})`);assert.deepEqual(bootstrap,{same:true,url:bootstrapURL.href,count:1},'SPA replaced or duplicated the shared bootstrap');assert.equal(requests.filter(row=>row.url===bootstrapURL.href).length,bootstrapRequests,'SPA requested the unchanged bootstrap again');if(stateColors)await wait(()=>evaluate("document.getAnimations().every(animation=>animation.playState!=='running')"),'focused navigation did not settle');};
 if(!stateColors){
 if(!pointerOnly){
 const guide=await target('guides/testing');assert.ok(guide,'normal guide prefetch link absent');
 const htmlCount=url=>responses.filter(row=>row.url===url&&row.mime==='text/html').length;
 await cdp('Input.dispatchMouseEvent',{type:'mouseMoved',x:guide.x,y:guide.y});
 await wait(()=>htmlCount(guide.href)===1,'hover did not prefetch exactly its guide');
 await cdp('Input.dispatchMouseEvent',{type:'mouseMoved',x:0,y:0});
 await cdp('Input.dispatchMouseEvent',{type:'mouseMoved',x:guide.x,y:guide.y});await delay(150);
 assert.equal(htmlCount(guide.href),1,'repeat hover issued a duplicate prefetch');
 // End pointer intent before moving the rail for the independent keyboard-intent case.
 await cdp('Input.dispatchMouseEvent',{type:'mouseMoved',x:0,y:0});
 const module=await target('api/Basic');assert.ok(module,'module prefetch link absent');
 receipt.focusPrefetch=await evaluate(`(()=>{const a=[...document.querySelectorAll('a[data-astro-prefetch="hover"]')].find(a=>a.href===${JSON.stringify(module.href)});a.focus();return{href:a.href,active:document.activeElement?.closest('a')?.href,rect:a.getBoundingClientRect().toJSON(),visible:a.checkVisibility({checkVisibilityCSS:true}),activeTag:document.activeElement?.tagName,details:(()=>{const items=[];for(let p=a.parentElement;p;p=p.parentElement)if(p.tagName==='DETAILS')items.push({class:p.className,open:p.open,contentVisibility:getComputedStyle(p,'::details-content').contentVisibility});return items;})()};})()`);
 assert.equal(receipt.focusPrefetch.active,module.href,'keyboard-intent anchor did not acquire focus');
 await wait(()=>htmlCount(module.href)===1,'keyboard focus did not prefetch its module');
 assert.equal(responses.filter(row=>row.mime==='text/html'&&![base.href,guide.href,module.href].includes(row.url)).length,0,'targeted intent prefetched unrelated catalogue pages');
 receipt.checks.push('hover/focus targets only, repeated intent deduplicated');
 await evaluate("document.querySelector('#theme-toggle').click();document.querySelector('#theme-toggle').click()");
 assert.equal(await evaluate("document.documentElement.getAttribute('data-theme')"),'dark');
 await go('api/Basic');
 const maximum=await evaluate('document.documentElement.scrollHeight-innerHeight');assert(maximum>150,'module page is not scrollable');
 const y=Math.min(500,Math.floor(maximum/2));await evaluate(`scrollTo({top:${y},behavior:'instant'})`);await delay(150);
 await go('guides/testing');assert.equal(await evaluate("document.documentElement.getAttribute('data-theme')"),'dark');
 assert.ok((await evaluate('window.__spaReceipt.paintThemes')).every(theme=>theme==='dark'),'incoming pages flashed a different theme');
 await evaluate("document.querySelector('#theme-toggle').click()");assert.equal(await evaluate("document.documentElement.getAttribute('data-theme')"),'system','theme listener ran more or less than once after swap');
 await cdp('Input.dispatchKeyEvent',{type:'keyDown',key:'/',code:'Slash',text:'/'});await cdp('Input.dispatchKeyEvent',{type:'keyUp',key:'/',code:'Slash'});
 assert.equal(await evaluate('document.activeElement.id'),'search-q','keyboard search focus stopped working after swap');
 await evaluate("document.activeElement.blur()");
 const heading=await evaluate("document.querySelector('main h2[id]')?.id");assert(heading,'guide lacks a linked heading');
 await evaluate(`location.hash=${JSON.stringify(heading)}`);await wait(()=>evaluate(`decodeURIComponent(location.hash.slice(1))===${JSON.stringify(heading)}`),'native hash navigation failed');
 await evaluate('history.back()');await wait(()=>evaluate("location.hash===''"),'hash Back failed');
 await evaluate('history.back()');await wait(()=>evaluate(`location.pathname===${JSON.stringify(pathOf('api/Basic'))}&&Math.abs(scrollY-${y})<=2`),'Back failed to restore module scroll');
 await evaluate('history.forward()');await wait(()=>evaluate(`location.pathname===${JSON.stringify(pathOf('guides/testing'))}&&window.__spaReceipt.marker==='preserved'`),'Forward failed to preserve SPA document');
 receipt.checks.push('SPA navigation, theme before paint, single control listener, keyboard focus, hash/history/scroll');
 await go('api/Basic/align_forward');
 assert(await evaluate("!!document.querySelector('button[data-act=run]:not([disabled])')"),'runtime sample has no enabled Run button');
 await evaluate("document.querySelector('button[data-act=run]:not([disabled])').click()");
 await wait(()=>evaluate('window.__spaReceipt.created===1'),'Run did not create one worker');
 await wait(()=>workerTargets.size>=1,'CDP did not observe the real compiler worker');
 await go('');await wait(()=>evaluate('window.__spaReceipt.live===0&&window.__spaReceipt.terminated===1'),'swap did not terminate the active compiler worker');
 await wait(()=>destroyedWorkers.size>=1,'terminated compiler worker target survived the swap');
 receipt.workers=await evaluate('({created:window.__spaReceipt.created,terminated:window.__spaReceipt.terminated})');
 receipt.checks.push('compiler loads only after Run; actual worker target destroyed on swap');
 }else{await go('api/Basic');await go('');}
 for(const interaction of [{button:'middle',modifiers:0,label:'middle-click'},{button:'left',modifiers:process.platform==='darwin'?4:2,label:'modified click'}]) {
  const before=await evaluate('location.href'),existing=new Set((await browser.cdp('Target.getTargets')).targetInfos.map(row=>row.targetId));
  const anchor=await target('guides/testing');assert(anchor);
  receipt.interactions.push(await evaluate(`(()=>{const hit=document.elementFromPoint(${anchor.x},${anchor.y});return{label:${JSON.stringify(interaction.label)},target:${JSON.stringify(anchor)},viewport:[innerWidth,innerHeight],scrollY,hitTag:hit?.tagName,hitAnchor:hit?.closest('a')?.href,animations:document.getAnimations().map(animation=>({playState:animation.playState,pseudo:animation.effect?.pseudoElement}))};})()`));
  // Page-load reports DOM enhancement, not native transition interactivity. One trusted
  // gesture is issued only when its intended anchor is actually the browser hit target.
  await wait(()=>evaluate(`document.elementFromPoint(${anchor.x},${anchor.y})?.closest('a')?.href===${JSON.stringify(anchor.href)}`),'native gesture target never became hit-testable');
  receipt.interactions.at(-1).readyHit=await evaluate(`({tag:document.elementFromPoint(${anchor.x},${anchor.y})?.tagName,href:document.elementFromPoint(${anchor.x},${anchor.y})?.closest('a')?.href})`);
  assert.equal(receipt.interactions.at(-1).readyHit.href,anchor.href);
  await cdp('Input.dispatchMouseEvent',{type:'mousePressed',button:interaction.button,modifiers:interaction.modifiers,clickCount:1,x:anchor.x,y:anchor.y});
  await cdp('Input.dispatchMouseEvent',{type:'mouseReleased',button:interaction.button,modifiers:interaction.modifiers,clickCount:1,x:anchor.x,y:anchor.y});
  let opened;
  await wait(async()=>{opened=(await browser.cdp('Target.getTargets')).targetInfos.find(row=>row.type==='page'&&!existing.has(row.targetId));return !!opened;},`native ${interaction.label} did not open another tab`);
  await wait(async()=>{opened=(await browser.cdp('Target.getTargetInfo',{targetId:opened.targetId})).targetInfo;return opened.url===anchor.href;},`native ${interaction.label} opened the wrong destination`);
  receipt.interactions.at(-1).openedURL=opened.url;
  assert.equal(await evaluate('location.href'),before,`${interaction.label} changed the current page`);
  await browser.cdp('Target.closeTarget',{targetId:opened.targetId});await cdp('Page.bringToFront');
  receipt.checks.push(`native ${interaction.label} opens a separate tab`);
 }
 }
 if(pointerOnly){if(traceTransitions)receipt.transitionTrace=await evaluate('window.__transitionTrace');receipt.requests=responses.filter(row=>row.mime==='text/html');console.log(JSON.stringify({...receipt,exceptions},null,2));assert.deepEqual(exceptions,[],'browser runtime exceptions');break spaRun;}
 await proveLiveMain();
 await cdp('Emulation.setDeviceMetricsOverride',{width:1280,height:900,deviceScaleFactor:1,mobile:false});
 // Exercise the actual production bootstrap with failed stores, not a replacement implementation.
 for(const blockedRead of [true,false]){
  const shim=`(()=>{
   const get=Storage.prototype.getItem,set=Storage.prototype.setItem,remove=Storage.prototype.removeItem;
   const side='jacket-docs-side',theme='jacket-docs-theme';
   set.call(sessionStorage,side,JSON.stringify({k:'modules',o:[],t:0}));set.call(localStorage,theme,'dark');
   window.__storageFailures={reads:0,writes:0};
   window.__freshStorage=(state)=>{set.call(sessionStorage,side,JSON.stringify(state));remove.call(localStorage,theme);};
   Storage.prototype.getItem=function(key){if(${blockedRead}&&this===sessionStorage&&key===side){window.__storageFailures.reads++;throw new DOMException('blocked','SecurityError');}return get.call(this,key);};
   Storage.prototype.setItem=function(key,value){if(this===sessionStorage&&key===side||this===localStorage&&key===theme){window.__storageFailures.writes++;throw new DOMException('full','QuotaExceededError');}return set.call(this,key,value);};
   Storage.prototype.removeItem=function(key){if(this===localStorage&&key===theme){window.__storageFailures.writes++;throw new DOMException('full','QuotaExceededError');}return remove.call(this,key);};
  })()`;
  const {identifier}=await cdp('Page.addScriptToEvaluateOnNewDocument',{source:shim});
  try{
   const origin=await evaluate('performance.timeOrigin');await cdp('Page.navigate',{url:new URL('api/Basic',base).href});
   await wait(()=>evaluate(`performance.timeOrigin!==${origin}&&window.__spaReceipt?.pages===1&&document.readyState==='complete'`),'storage fixture did not load a fresh production document');
   const expected=await evaluate(`(async()=>{
    const side=document.querySelector('details.side'),groups=[...side.querySelectorAll('details.nav-cat')];for(const group of groups)group.open=true;side.getBoundingClientRect();await new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));await Promise.all(groups.flatMap(group=>group.getAnimations({subtree:true})).map(animation=>animation.finished));side.scrollTop=37;
    const read=()=>({groups:[...document.querySelectorAll('details.side details.nav-cat[open]')].map(node=>node.dataset.key).sort(),scroll:document.querySelector('details.side').scrollTop,theme:document.documentElement.dataset.theme});
    window.__storagePaints=[];document.addEventListener('astro:after-swap',()=>window.__storagePaints.push(read()));
    // Keep both destinations inside the restored rail at scroll37 and scroll19.
    // Navigating to an offscreen current entry legitimately reveals it before paint.
    const box=side.getBoundingClientRect();
    const destinations=[...side.querySelectorAll('nav[data-nav="modules"] a[href]')].map(anchor=>{
     const rect=anchor.getBoundingClientRect(),url=new URL(anchor.href);
     return{href:url.href,label:anchor.textContent,top:rect.top-box.top,bottom:rect.bottom-box.top};
    }).filter(anchor=>new URL(anchor.href).pathname!==location.pathname&&anchor.top>=40&&anchor.bottom<=side.clientHeight-40).slice(0,2);
    return{...read(),destinations,viewport:innerWidth,scrollHeight:side.scrollHeight,clientHeight:side.clientHeight};
   })()`);receipt.storage.push({blockedRead,initial:expected});assert(expected.groups.length>1&&expected.scroll===37&&expected.destinations.length===2,'storage fixture lacks a scrolled/open catalogue with two visible destinations');
   await evaluate("document.querySelector('#theme-toggle').click()");assert.equal(await evaluate('document.documentElement.dataset.theme'),'system');
   for(const destination of expected.destinations.map(anchor=>anchor.href)){
    await go(destination);const actual=await evaluate('window.__storagePaints.at(-1)');
    assert.deepEqual(actual.groups,expected.groups,'failed sidebar write lost opened groups');assert.equal(actual.scroll,37,'failed sidebar write lost before-paint scroll');assert.equal(actual.theme,'system','failed theme removal lost manual preference');
   }
   await evaluate("document.querySelector('#theme-toggle').click()");assert.equal(await evaluate('document.documentElement.dataset.theme'),'light','theme control listener ran more than once');
   await go(expected.destinations[0].href);assert.equal(await evaluate('window.__storagePaints.at(-1).theme'),'light','failed theme set lost manual preference');
   const failures=await evaluate('window.__storageFailures');assert(failures.writes>0);if(blockedRead)assert(failures.reads>0);
   if(!blockedRead){await evaluate(`window.__freshStorage({k:'modules',o:${JSON.stringify(expected.groups)},t:19})`);await go(expected.destinations[1].href);const fresh=await evaluate('window.__storagePaints.at(-1)');assert.equal(fresh.theme,'system','fresh stored theme was ignored');assert.equal(fresh.scroll,37,'fresh stored data reset the live persistent sidebar');}
   Object.assign(receipt.storage.at(-1),{openedGroups:expected.groups.length,restoredScroll:37,failures,beforePaint:await evaluate('window.__storagePaints')});
  }finally{await cdp('Page.removeScriptToEvaluateOnNewDocument',{identifier});}
 }
 receipt.checks.push('actual blocked/quota storage preserves groups, scroll and manual themes before paint; fresh stored changes win');
 // These are real regenerated CLI diagnostic examples, including both severities and fix operations.
 const diagnosticCases=productionDiagnosticCases;
 for(const theme of ['light','dark'])for(const fixture of diagnosticCases){
  await go(fixture.page);await evaluate(`document.documentElement.setAttribute('data-theme',${JSON.stringify(theme)})`);
  const colors=await evaluate(String.raw`(()=>{
   const code=[...document.querySelectorAll('pre.code>code')].find(node=>node.querySelector('span.de,span.dw'));
   if(!code)throw new Error('production diagnostic block absent');
   const variables={de:'--err',dw:'--warn',dd:'--err',da:'--ok',dc:'--warn',dk:'--type',dm:'--muted'};
   const rgb=value=>{const match=/^rgba?\(([^)]+)\)$/.exec(value);if(!match)throw new Error('unexpected computed color '+value);return match[1].split(/[, ]+/).filter(Boolean).map(Number);};
   const luminance=value=>{const c=rgb(value).slice(0,3).map(channel=>{const s=channel/255;return s<=.04045?s/12.92:((s+.055)/1.055)**2.4;});return c[0]*.2126+c[1]*.7152+c[2]*.0722;};
   const records=[];
   for(const [name,variable]of Object.entries(variables))for(const span of code.querySelectorAll('span.'+name)){
    const color=getComputedStyle(span).color;let background;for(let node=span;node;node=node.parentElement){const value=getComputedStyle(node).backgroundColor,channels=rgb(value);if(channels.length<4||channels[3]===1){background=value;break;}if(channels[3]!==0)throw new Error('diagnostic contrast requires alpha composition');}
    if(!background)throw new Error('diagnostic background unresolved');const probe=document.createElement('span');probe.style.color='var('+variable+')';document.body.append(probe);const expected=getComputedStyle(probe).color;probe.remove();
    const a=luminance(color),b=luminance(background);records.push({role:name,text:span.textContent,color,expected,background,contrast:(Math.max(a,b)+.05)/(Math.min(a,b)+.05)});
   }
   return{theme:document.documentElement.dataset.theme,source:code.textContent,records,calibration:(luminance('rgb(255, 255, 255)')+.05)/(luminance('rgb(0, 0, 0)')+.05)};
  })()`);assert.equal(colors.calibration,21,'contrast probe must resolve the black/white reference');assert.equal(colors.theme,theme);assert(colors.records.some(record=>record.role===(fixture.severity==='error'?'de':'dw')));
  for(const required of fixture.required)assert(colors.records.some(record=>record.role===required),`${fixture.page}: missing ${required}`);
  for(const record of colors.records){assert.equal(record.color,record.expected,`${fixture.page}: ${record.role} palette mismatch`);assert(record.contrast>=4.5,`${fixture.page}: ${record.role} contrast ${record.contrast}`);if(record.role==='dd')assert.equal(record.text,'-');if(record.role==='da')assert.equal(record.text,'+');if(record.role==='dc')assert.equal(record.text,'~');}
  if(fixture.severity==='warning'&&fixture.required.includes('dd'))assert.notEqual(colors.records.find(record=>record.role==='dd').color,colors.records.find(record=>record.role==='dw').color,'warning deletion inherited severity yellow');
  receipt.diagnostics.push({page:fixture.page,severity:fixture.severity,...colors});
 }
 receipt.checks.push('actual diagnostic error/warning deletions red, additions green, replacements and origins distinct; computed light/dark contrast');
 // CDP Runtime evaluation is itself script execution. Keep it disabled and inspect native DOM.
 let noScriptLoads=0;connection.listeners.push(message=>{if(message.method==='Page.loadEventFired')noScriptLoads++;});
 if(traceTransitions)receipt.transitionTrace=await evaluate('window.__transitionTrace');
 await cdp('DOM.enable');await cdp('Emulation.setScriptExecutionDisabled',{value:true});scriptsDisabled=true;
 const nativeQuery=async selector=>{const {root}=await cdp('DOM.getDocument',{depth:0});return(await cdp('DOM.querySelector',{nodeId:root.nodeId,selector})).nodeId;};
 const nativeUrl=async()=>{const history=await cdp('Page.getNavigationHistory');return history.entries[history.currentIndex].url;};
 const assertStatic=async()=>{
  const {root}=await cdp('DOM.getDocument',{depth:0});
  assert.equal((await cdp('DOM.querySelectorAll',{nodeId:root.nodeId,selector:'h1'})).nodeIds.length,1,'no-JavaScript page lost its main heading');
  const html=await nativeQuery('html'),{attributes}=await cdp('DOM.getAttributes',{nodeId:html});
  const classIndex=attributes.indexOf('class'),classes=classIndex<0?'':attributes[classIndex+1];
  assert(!classes.split(/\s+/u).includes('js'),'page scripts ran while JavaScript was disabled');
 };
 const beforeStatic=noScriptLoads;
 await cdp('Page.navigate',{url:new URL('api/Basic',base).href});
 await wait(async()=>noScriptLoads>beforeStatic&&new URL(await nativeUrl()).pathname===pathOf('api/Basic'),'no-JavaScript module page failed');
 await assertStatic();
 const tree=(await cdp('DOM.getDocument',{depth:-1})).root,nodes=new Map(),parents=new Map();
 const visit=(node,parent)=>{nodes.set(node.nodeId,node);if(parent)parents.set(node.nodeId,parent);for(const child of node.children??[])visit(child,node.nodeId);};visit(tree,null);
 const plain=await nativeQuery(`a[href="${pathOf('guides/testing')}"]`);assert(plain,'ordinary guide link absent without JavaScript');
 const nativeClick=async nodeId=>{await cdp('DOM.scrollIntoViewIfNeeded',{nodeId});const {model}=await cdp('DOM.getBoxModel',{nodeId}),q=model.content,x=(q[0]+q[2]+q[4]+q[6])/4,y=(q[1]+q[3]+q[5]+q[7])/4;await cdp('Input.dispatchMouseEvent',{type:'mousePressed',button:'left',clickCount:1,x,y});await cdp('Input.dispatchMouseEvent',{type:'mouseReleased',button:'left',clickCount:1,x,y});};
 const ancestors=[];for(let id=parents.get(plain);id;id=parents.get(id))ancestors.unshift(nodes.get(id));
 for(const node of ancestors)if(node.nodeName==='DETAILS'&&!(node.attributes??[]).includes('open')){const summary=node.children.find(child=>child.nodeName==='SUMMARY');assert(summary,'native disclosure lacks summary');await nativeClick(summary.nodeId);}
 const documents=requests.filter(row=>row.type==='Document').length,beforeClick=noScriptLoads;
 await nativeClick(plain);
 await wait(async()=>noScriptLoads>beforeClick&&new URL(await nativeUrl()).pathname===pathOf('guides/testing'),'ordinary no-JavaScript guide link failed');
 assert.equal(requests.filter(row=>row.type==='Document').length,documents+1,'no-JavaScript link did not perform ordinary document navigation');
 await assertStatic();
 receipt.checks.push('static/no-JavaScript main content and ordinary guide navigation');
 assert.deepEqual(exceptions,[],'browser runtime exceptions');
 if(traceTransitions&&!scriptsDisabled)receipt.transitionTrace=await evaluate('window.__transitionTrace');
 receipt.requests=responses.filter(row=>row.mime==='text/html');
 console.log(JSON.stringify(receipt,null,2));
}catch(error){
 if(traceTransitions&&readPage&&!scriptsDisabled)receipt.transitionTrace=await readPage('window.__transitionTrace').catch(reason=>({readError:String(reason)}));
 console.error(JSON.stringify({status:'fail',...receipt,exceptions,requests:responses.filter(row=>row.mime==='text/html')},null,2));
 throw error;
}finally{
 connection?.close();browser?.close();child.kill();await new Promise(resolve=>!child.pid||child.exitCode!==null||child.signalCode!==null?resolve():child.once('exit',resolve));await rm(profile,{recursive:true,force:true});
}
