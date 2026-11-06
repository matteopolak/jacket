#!/usr/bin/env node
/** Capture every sidebar row/text frame and transition snapshot while the real Astro router swaps. */
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {mkdir,mkdtemp,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {setTimeout as delay} from 'node:timers/promises';
import {Connection,runtimeException} from './chrome_cdp.mjs';
import {findChrome,waitForDevToolsPort} from './chrome_startup.mjs';
const base=new URL(process.argv[2]||'http://127.0.0.1:57526/'),out=path.resolve(process.argv[3]);
const startPage=process.argv[4]||'api/Basic',ablation=process.argv.includes('--intrinsic');
const width=Number(process.argv.find(arg=>arg.startsWith('--width='))?.split('=')[1]||1280),height=Number(process.argv.find(arg=>arg.startsWith('--height='))?.split('=')[1]||900);
const scale=Number(process.argv.find(arg=>arg.startsWith('--scale='))?.split('=')[1]||1),theme=process.argv.find(arg=>arg.startsWith('--theme='))?.split('=')[1]||'dark';
const chrome=findChrome();
await mkdir(out,{recursive:false});
const profile=await mkdtemp(path.join(tmpdir(),'jacket-side-'));
const child=spawn(chrome,[...(process.argv.includes('--headed')?[]:['--headless=new']),'--no-sandbox',...(process.argv.includes('--gpu')?[]:['--disable-gpu']),'--no-first-run','--remote-debugging-port=0',`--user-data-dir=${profile}`,'about:blank'],{stdio:['ignore','ignore','pipe']});
let connection,contextId;const contexts=new Map();const exceptions=[],screens=[];let capture=false;
const probe=String.raw`(()=>{
 window.__sideTrace={origin:performance.timeOrigin,frames:[],events:[],active:false,transition:0};
 const rowIDs=new WeakMap();let rowSequence=0;const rowID=node=>{if(!rowIDs.has(node))rowIDs.set(node,++rowSequence);return rowIDs.get(node);};
 const rect=r=>[r.x,r.y,r.width,r.height];
 const style=node=>{const s=getComputedStyle(node);return{font:s.font,fontFamily:s.fontFamily,fontSize:s.fontSize,fontWeight:s.fontWeight,lineHeight:s.lineHeight,transform:s.transform,translate:s.translate,position:s.position,padding:s.padding,transition:s.transition,viewTransitionName:s.viewTransitionName};};
 const pseudo=name=>{const s=getComputedStyle(document.documentElement,name);return{width:s.width,height:s.height,inlineSize:s.inlineSize,blockSize:s.blockSize,objectFit:s.objectFit,objectPosition:s.objectPosition,transform:s.transform,animation:s.animation};};
 function sample(){
  if(window.__sideTrace.active){
   const side=document.querySelector('details.side'),main=document.querySelector('main'),mainStyle=main&&getComputedStyle(main);
   const rows=side?[...side.querySelectorAll('li a[href]')].flatMap(a=>{
    const box=a.getBoundingClientRect();if(!a.checkVisibility({checkVisibilityCSS:true})||!box.width||!box.height||box.bottom<61||box.top>innerHeight)return[];
    const range=document.createRange();const text=[...a.childNodes].find(n=>n.nodeType===Node.TEXT_NODE&&n.textContent.trim());
    let lines=[];if(text){range.selectNodeContents(text);lines=[...range.getClientRects()].map(rect);}
    return[{id:rowID(a),href:a.href,name:a.textContent.trim(),a:rect(box),li:rect(a.closest('li').getBoundingClientRect()),text:lines,css:style(a),liCSS:style(a.closest('li'))}];
   }):[];
   window.__sideTrace.frames.push({time:performance.now(),path:location.pathname,transition:window.__sideTrace.transition,bodyY:scrollY,announcers:document.querySelectorAll('.astro-route-announcer').length,headerBottom:document.querySelector('.top')?.getBoundingClientRect().bottom,side:side?rect(side.getBoundingClientRect()):null,scroll:side?.scrollTop,identity:window.__liveRail?{side:side===window.__liveRail.side,nav:side?.querySelector('nav')===window.__liveRail.nav,group:side?.contains(window.__liveRail.group),focus:document.activeElement===window.__liveRail.anchor,hit:(()=>{const anchor=window.__liveRail.anchor;if(!anchor)return null;const r=anchor.getBoundingClientRect();return document.elementFromPoint(r.x+r.width/2,r.y+r.height/2)?.closest('a')?.href??null;})()}:null,names:{html:getComputedStyle(document.documentElement).viewTransitionName,top:document.querySelector('.top')?getComputedStyle(document.querySelector('.top')).viewTransitionName:null,side:side?getComputedStyle(side).viewTransitionName:null,foot:document.querySelector('.foot')?getComputedStyle(document.querySelector('.foot')).viewTransitionName:null,main:document.querySelector('main')?getComputedStyle(document.querySelector('main')).viewTransitionName:null},rows,mainEntrance:{pending:main?.classList.contains('doc-main-pending'),name:mainStyle?.animationName,duration:mainStyle?.animationDuration,opacity:mainStyle?.opacity,transform:mainStyle?.transform,running:main?.getAnimations().some(a=>a.playState==='running'&&a.animationName==='page-in')},pseudos:{group:pseudo('::view-transition-group(side)'),image:pseudo('::view-transition-new(side)'),pair:pseudo('::view-transition-image-pair(side)')},animations:document.getAnimations().map(a=>({currentTime:a.currentTime,playState:a.playState,duration:a.effect?.getTiming().duration,pseudo:a.effect?.pseudoElement,main:a.effect?.target===main,name:a.animationName}))});
  }
  requestAnimationFrame(sample);
 }
 for(const name of ['animationstart','animationend','animationcancel'])document.addEventListener(name,event=>{if(event.target===document.querySelector('main')&&event.animationName==='page-in')window.__sideTrace.events.push({event:'live:'+name,transition:window.__sideTrace.transition,time:performance.now(),path:location.pathname});});
 for(const event of ['astro:before-swap','astro:after-swap','astro:page-load'])document.addEventListener(event,detail=>{const side=document.querySelector('details.side');if(event==='astro:before-swap'){const transition=++window.__sideTrace.transition;void detail.viewTransition.finished.then(()=>window.__sideTrace.events.push({event:'native:finished',transition,time:performance.now(),path:location.pathname})).catch(error=>{window.__sideTrace.events.push({event:'native:failed',transition,time:performance.now(),message:String(error)});reportError(error);});}window.__sideTrace.events.push({event,transition:window.__sideTrace.transition,time:performance.now(),path:location.pathname,scroll:side?.scrollTop,identity:window.__liveRail?{side:side===window.__liveRail.side,nav:side?.querySelector('nav')===window.__liveRail.nav,group:side?.contains(window.__liveRail.group),focus:document.activeElement===window.__liveRail.anchor}:null});});
 requestAnimationFrame(sample);
})();`;
captureRun: try{
 const port=await waitForDevToolsPort(child,profile);const pages=await(await fetch(`http://127.0.0.1:${port}/json/list`)).json();
 connection=await new Connection(pages.find(page=>page.type==='page').webSocketDebuggerUrl).open();
 const cdp=(method,params={})=>connection.cdp(method,params);
 connection.listeners.push(message=>{
  if(message.method==='Runtime.executionContextCreated'&&message.params.context.auxData?.isDefault)contexts.set(message.params.context.auxData.frameId,message.params.context.id);
  const error=runtimeException(message);if(error!==undefined)exceptions.push(error);
  if(message.method==='Page.screencastFrame'){
   void cdp('Page.screencastFrameAck',{sessionId:message.params.sessionId});
   if(capture){const file=`frame-${String(screens.length).padStart(4,'0')}.png`;screens.push({file,metadata:message.params.metadata});void writeFile(path.join(out,file),Buffer.from(message.params.data,'base64'));}
  }
 });
 const evaluate=async expression=>{const result=await cdp('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true,...(contextId?{contextId}:{})});assert(!result.exceptionDetails,JSON.stringify(result.exceptionDetails));return result.result.value;};
 await cdp('Page.enable');await cdp('Runtime.enable');await cdp('Emulation.setDeviceMetricsOverride',{width,height,deviceScaleFactor:scale,mobile:false});await cdp('Emulation.setEmulatedMedia',{features:[{name:'prefers-color-scheme',value:theme}]});await cdp('Emulation.setFocusEmulationEnabled',{enabled:true});await cdp('Page.bringToFront');
 await cdp('Page.addScriptToEvaluateOnNewDocument',{source:probe});
 const embedScale=Number(process.argv.find(arg=>arg.startsWith('--embed='))?.split('=')[1]||1);
 if(embedScale!==1){
  const {frameTree}=await cdp('Page.getFrameTree');
  await cdp('Page.setDocumentContent',{frameId:frameTree.frame.id,html:`<!doctype html><style>html,body{margin:0}iframe{border:0;width:${width/embedScale}px;height:${height/embedScale}px;transform:scale(${embedScale});transform-origin:top left}</style><iframe src="${new URL(startPage,base).href}"></iframe>`});
  for(let i=0;i<400;i++){const {frameTree}=await cdp('Page.getFrameTree');const id=frameTree.childFrames?.[0]?.frame.id;if(id&&contexts.has(id)){contextId=contexts.get(id);break;}await delay(25);}
  assert(contextId,'embedded preview context absent');
 }else await cdp('Page.navigate',{url:new URL(startPage,base).href});
 for(let i=0;i<600;i++){if(await evaluate("document.readyState==='complete'&&window.__sideTrace?.events.some(e=>e.event==='astro:page-load')"))break;await delay(25);}
 const pinch=Number(process.argv.find(arg=>arg.startsWith('--pinch='))?.split('=')[1]||1);
 if(pinch!==1)await cdp('Emulation.setPageScaleFactor',{pageScaleFactor:pinch});
 await delay(350);
 for(let i=0;i<200;i++){if(await evaluate("!document.querySelector('details.side [data-source]')"))break;await delay(25);}
 if(ablation)await evaluate("(()=>{const css='::view-transition-old(side),::view-transition-new(side){inline-size:auto;block-size:auto;object-fit:none;object-position:left top}';const install=doc=>{const style=doc.createElement('style');style.textContent=css;doc.head.append(style);};install(document);document.addEventListener('astro:before-swap',event=>install(event.newDocument));})()");
 if(process.argv.includes('--live-rail')){
  const liveHitAblation=process.argv.includes('--live-hit-through');
  if(liveHitAblation)await evaluate(`(()=>{const install=doc=>{const style=doc.createElement('style');style.textContent='::view-transition{pointer-events:none}::view-transition-group(main){pointer-events:auto}';doc.head.append(style);};install(document);document.addEventListener('astro:before-swap',event=>install(event.newDocument));return true;})()`);
  const requestedBodyY=Number(process.argv.find(arg=>arg.startsWith('--source-y='))?.split('=')[1]||0);
  const sourceBodyY=await evaluate(`new Promise(resolve=>{scrollTo({top:${requestedBodyY},behavior:'instant'});requestAnimationFrame(()=>requestAnimationFrame(()=>resolve(scrollY)));})`);
  const anchorAblation=process.argv.includes('--no-scroll-anchor');if(anchorAblation)await evaluate(`(()=>{document.querySelector('details.side').style.overflowAnchor='none';return true;})()`);
  const railMetrics=process.argv.find(arg=>arg.startsWith('--rail-metrics='))?.split('=')[1];
  const metricAblation=railMetrics?await evaluate(`(()=>{const side=document.querySelector('details.side'),style=getComputedStyle(side),values={};for(const name of ${JSON.stringify(railMetrics==='font'?['font-family','line-height']:railMetrics==='layout'?['--layout-max','--layout-pad','--rail-width','--layout-gap']:['font-family','line-height','--layout-max','--layout-pad','--rail-width','--layout-gap'])})values[name]=name==='line-height'?String(parseFloat(style.lineHeight)/parseFloat(style.fontSize)):style.getPropertyValue(name);for(const [name,value]of Object.entries(values))side.style.setProperty(name,value);return values;})()`):null;
  const summary=await evaluate(`(()=>{const side=document.querySelector('details.side');side.scrollTop=200;const box=side.getBoundingClientRect();const group=[...side.querySelectorAll('details.nav-cat:not([open])')].find(group=>{const r=group.querySelector('summary').getBoundingClientRect();return r.top>box.top+30&&r.bottom<Math.min(innerHeight/2,box.bottom-60);});if(!group)throw Error('no collapsed category in top-half scrolled fixture');const r=group.querySelector('summary').getBoundingClientRect();window.__liveRail={side,nav:side.querySelector('nav'),group};return{x:r.x+r.width/2,y:r.y+r.height/2,label:group.querySelector('summary').textContent};})()`);
  for(const type of ['mousePressed','mouseReleased'])await cdp('Input.dispatchMouseEvent',{type,button:'left',clickCount:1,x:summary.x,y:summary.y});
  await evaluate(`(()=>{const descriptor=Object.getOwnPropertyDescriptor(Element.prototype,'scrollTop');window.__railScrollWrites=[];Object.defineProperty(Element.prototype,'scrollTop',{...descriptor,set(value){if(this===window.__liveRail.side)window.__railScrollWrites.push({time:performance.now(),value,stack:new Error().stack});descriptor.set.call(this,value);}});return true;})()`);
  if(process.argv.includes('--trace-moves'))await evaluate(`(()=>{const sample=()=>{const side=window.__liveRail.side,s=getComputedStyle(side);return{scroll:side.scrollTop,parent:side.parentElement?.tagName,open:side.open,height:side.clientHeight,scrollHeight:side.scrollHeight,width:s.width,font:s.font,top:s.top,maxHeight:s.maxHeight,connected:side.isConnected,active:document.activeElement===window.__liveRail.anchor};};for(const [prototype,name]of [[Element.prototype,'moveBefore'],[Element.prototype,'replaceWith'],[HTMLElement.prototype,'focus']]){const original=prototype[name];if(!original)continue;prototype[name]=function(...args){const relevant=name==='moveBefore'&&args[0]===window.__liveRail.side||name==='replaceWith'&&this.tagName==='BODY'||name==='focus'&&this===window.__liveRail.anchor;if(!relevant)return original.apply(this,args);const before=sample(),result=original.apply(this,args);window.__railScrollWrites.push({operation:name,parent:this.tagName,before,after:sample(),stack:new Error().stack});return result;};}return true;})()`);
  const destination=await evaluate(`(async()=>{const {side,group}=window.__liveRail;if(!group.open)throw Error('trusted category gesture did not open it');await new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));await Promise.all(group.getAnimations({subtree:true}).map(animation=>animation.finished));const anchor=group.querySelector('a[href]');if(!anchor)throw Error('opened category has no link');const rail=side.getBoundingClientRect();side.scrollTop+=anchor.getBoundingClientRect().top-(rail.top+110);const r=anchor.getBoundingClientRect();if(r.top>=innerHeight/2||r.top<rail.top)throw Error('destination is not in rail top half');Object.assign(window.__liveRail,{anchor,scroll:side.scrollTop});return{href:anchor.href,x:r.x+r.width/2,y:r.y+r.height/2,scroll:side.scrollTop};})()`);
  await cdp('Page.startScreencast',{format:'png',everyNthFrame:1});capture=true;
  await evaluate('window.__sideTrace.active=true');await delay(80);
  for(const type of ['mousePressed','mouseReleased'])await cdp('Input.dispatchMouseEvent',{type,button:'left',clickCount:1,x:destination.x,y:destination.y});
  const settle=async(pathname,previous,expectedY)=>{for(let i=0;i<600;i++){if(await evaluate(`(()=>{const trace=window.__sideTrace,last=trace.frames.at(-1),finished=trace.events.find(event=>event.event==='native:finished'&&event.transition===trace.transition);return trace.transition>${previous}&&location.pathname===${JSON.stringify(pathname)}&&last?.path===${JSON.stringify(pathname)}&&last.transition===trace.transition&&finished&&last.time>=finished.time&&!last.mainEntrance.pending&&!last.mainEntrance.running&&trace.events.some(event=>['live:animationend','live:animationcancel'].includes(event.event)&&event.transition===trace.transition)&&Math.abs(scrollY-${expectedY})<=1&&Math.abs(last.bodyY-${expectedY})<=1&&trace.events.some(event=>event.event==='astro:page-load'&&event.transition===trace.transition);})()`))return await evaluate('window.__sideTrace.transition');await delay(25);}await writeFile(path.join(out,'failure.json'),JSON.stringify({pathname,previous,expectedY,trace:await evaluate('window.__sideTrace'),state:await evaluate('({path:location.pathname,y:scrollY,ready:document.readyState,restoration:history.scrollRestoration,folds:[...document.querySelectorAll("main .fold")].slice(0,4).map(node=>({open:node.classList.contains("open"),height:node.getBoundingClientRect().height,scrollHeight:node.scrollHeight}))})'),screens,exceptions},null,2));throw Error('live rail traversal never finished/rendered '+pathname);};
  let epoch=await settle(new URL(destination.href).pathname,0,0);
  await evaluate('history.back()');epoch=await settle(new URL(startPage,base).pathname,epoch,sourceBodyY);
  await evaluate('history.forward()');await settle(new URL(destination.href).pathname,epoch,0);
  const endState=await evaluate(`({path:location.pathname,visibility:document.visibilityState,announcers:document.querySelectorAll('.astro-route-announcer').length,side:document.querySelector('details.side')===window.__liveRail.side,nav:document.querySelector('details.side nav')===window.__liveRail.nav})`);
  await evaluate('window.__sideTrace.active=false');capture=false;await cdp('Page.stopScreencast');
  const trace=await evaluate('window.__sideTrace'),scrollWrites=await evaluate('window.__railScrollWrites'),problems=[];
  for(const event of trace.events.filter(event=>event.event==='astro:after-swap'))if(!event.identity?.side||!event.identity?.nav||!event.identity?.group)problems.push({reason:'live rail subtree replaced at swap',event});
  const first=trace.frames[0],unchanged=new Map(first.rows.filter(row=>![new URL(startPage,base).href,destination.href].includes(row.href)).map(row=>[row.id,row]));
  for(const frame of trace.frames){
   if(!frame.identity?.side||!frame.identity?.nav||!frame.identity?.group)problems.push({reason:'live rail subtree replaced',time:frame.time,identity:frame.identity});
   if(Math.abs(frame.scroll-destination.scroll)>0.1)problems.push({reason:'live rail scroll changed',time:frame.time,before:destination.scroll,after:frame.scroll});
   if(frame.path===new URL(destination.href).pathname&&!frame.identity?.focus)problems.push({reason:'clicked live anchor lost focus',time:frame.time});
   for(const name of ['html','top','side','foot','main'])if(frame.names[name]!=='none')problems.push({reason:'nonmain native capture remains',element:name,name:frame.names[name],time:frame.time});
   if(frame.announcers>1)problems.push({reason:'route live regions accumulated',time:frame.time,count:frame.announcers});
   if(frame.mainEntrance.running&&(frame.mainEntrance.name!=='page-in'||frame.mainEntrance.duration!=='0.17s'))problems.push({reason:'live main entrance changed role or duration',time:frame.time,entrance:frame.mainEntrance});
   if(frame.animations.some(animation=>animation.playState==='running'&&animation.main&&animation.name==='page-in')&&frame.identity?.hit!==destination.href)problems.push({reason:'live sidebar hit target blocked during main animation',time:frame.time,hit:frame.identity?.hit});
   for(const row of frame.rows){const previous=unchanged.get(row.id);if(!previous)continue;if(Math.abs(row.a[1]-previous.a[1])>0.1||Math.abs(row.li[1]-previous.li[1])>0.1||Math.abs(row.a[3]-previous.a[3])>0.1||row.text.some((line,index)=>previous.text[index]&&Math.abs(line[1]-previous.text[index][1])>0.1))problems.push({reason:'unaffected live row/text moved',time:frame.time,name:row.name,before:previous.a,after:row.a});}
  }
  for(const transition of new Set(trace.frames.filter(frame=>frame.transition>0).map(frame=>frame.transition)))if(!trace.frames.some(frame=>frame.transition===transition&&frame.mainEntrance.running))problems.push({reason:'no actual live main animation frames observed',transition});
  if(endState.announcers!==1)problems.push({reason:'final route announcement absent or duplicated',count:endState.announcers});
  if(trace.frames.at(-1)?.path!==new URL(destination.href).pathname)problems.push({reason:'trusted top-half link never navigated'});
  await writeFile(path.join(out,'live-rail.json'),JSON.stringify({base:base.href,startPage,theme,liveHitAblation,anchorAblation,railMetrics,metricAblation,requestedBodyY,sourceBodyY,summary,destination,endState,scrollWrites,trace,screens,problems,exceptions},null,2));
  console.log(JSON.stringify({mode:'live-rail',frames:trace.frames.length,problems:problems.length,exceptions,output:out}));
  assert(trace.frames.length>5&&screens.length>1,'missing actual live rail frame evidence');assert.deepEqual(problems,[]);assert.deepEqual(exceptions,[]);break captureRun;
 }
 if(process.argv.includes('--header-resize')){
  assert.equal(width,1280,'header-resize starts at1280px and exercises1600px then1280px');
  assert.equal(embedScale,1,'header-resize requires an ordinary top-level viewport');
  const samples=[],problems=[];
  await evaluate(String.raw`(()=>{
   const sample=label=>{const header=document.querySelector('.top'),box=header.getBoundingClientRect();return{label,time:performance.now(),width:innerWidth,scrollY,header:{height:box.height,bottom:box.bottom},offset:getComputedStyle(document.documentElement).getPropertyValue('--header-offset').trim(),inlineOffset:document.documentElement.style.getPropertyValue('--header-offset'),rails:[...document.querySelectorAll('details.side,.toc')].map(node=>{const r=node.getBoundingClientRect(),s=getComputedStyle(node),wrap=node.closest('.wrap').getBoundingClientRect();return{kind:node.matches('.side')?'side':'toc',visible:s.display!=='none',top:r.top,height:r.height,stickyTop:s.top,maxHeight:s.maxHeight,clearance:wrap.bottom-r.height};})};};
   window.__headerSample=sample;
   window.__headerReady=new Promise(resolve=>{window.__headerFirst=resolve;});
   window.__headerWatch=new ResizeObserver(()=>{if(window.__headerFirst){window.__headerFirst();window.__headerFirst=null;}if(window.__headerResolve){const resolve=window.__headerResolve;window.__headerResolve=null;resolve(sample('resize-observer-before-paint'));}});
   window.__headerWatch.observe(document.querySelector('.top'),{box:'border-box'});
   window.__headerArm=()=>{window.__headerNext=new Promise((resolve,reject)=>{const timer=setTimeout(()=>reject(Error('header did not resize')),2000);window.__headerResolve=value=>{clearTimeout(timer);resolve(value);};});};
   window.__headerFrames=label=>new Promise(resolve=>{const frames=[];const collect=()=>{frames.push(sample(label));if(frames.length===8)resolve(frames);else requestAnimationFrame(collect);};requestAnimationFrame(collect);});
   return true;
  })()`);
  await evaluate('window.__headerReady.then(()=>true)');
  const capture=async label=>{for(const y of [0,1,2,5]){await evaluate(`scrollTo({top:${y},behavior:'instant'})`);samples.push(...await evaluate(`window.__headerFrames(${JSON.stringify(label)})`));}};
  await capture('initial-1280');
  for(const nextWidth of [1600,1280]){
   await evaluate('window.__headerArm()');await cdp('Emulation.setDeviceMetricsOverride',{width:nextWidth,height,deviceScaleFactor:scale,mobile:false});
   samples.push(await evaluate('window.__headerNext'));await capture(`viewport-${nextWidth}`);
  }
  const originalStyle=await evaluate("document.querySelector('.top-in').getAttribute('style')"),initialHeight=samples.at(-1).header.height;
  await evaluate('window.__headerArm()');await evaluate("document.querySelector('.top-in').style.maxWidth='700px'");samples.push(await evaluate('window.__headerNext'));await capture('fixture-header-max-width-700');
  const wrappedHeight=samples.at(-1).header.height;
  await evaluate('window.__headerArm()');await evaluate(`(()=>{const node=document.querySelector('.top-in');${originalStyle===null?"node.removeAttribute('style')":`node.setAttribute('style',${JSON.stringify(originalStyle)})`};return true})()`);samples.push(await evaluate('window.__headerNext'));await capture('restored-header');
  for(const frame of samples){
   const offset=Number.parseFloat(frame.offset);
   if(!Number.isFinite(offset)||Math.abs(offset-frame.header.height)>0.025)problems.push({label:frame.label,issue:'measured root offset differs from actual header border box',frame});
   for(const rail of frame.rails){if(!rail.visible)continue;
    if(rail.clearance<frame.header.height-0.025){problems.push({label:frame.label,issue:'fixture rail is constrained by containing-block end',rail});continue;}
    if(Math.abs(rail.top-frame.header.bottom)>0.025||Math.abs(Number.parseFloat(rail.stickyTop)-frame.header.height)>0.025||Math.abs(Number.parseFloat(rail.maxHeight)-(height-frame.header.height))>0.025)problems.push({label:frame.label,issue:'independent rail geometry differs from actual header',frame,rail});
   }
  }
  if(wrappedHeight<=initialHeight+1)problems.push({issue:'bounded max-width fixture did not actually wrap the header',initialHeight,wrappedHeight});
  const inputChecks=[];
  const inputState=label=>evaluate(`(()=>{const input=document.querySelector('#search-q'),header=document.querySelector('.top');return{label:${JSON.stringify(label)},active:document.activeElement?.id,value:input.value,selection:[input.selectionStart,input.selectionEnd],headerHeight:header.getBoundingClientRect().height,offset:document.documentElement.style.getPropertyValue('--header-offset'),readyState:document.readyState};})()`);
  const clickAt=async point=>{await cdp('Input.dispatchMouseEvent',{type:'mouseMoved',...point});await cdp('Input.dispatchMouseEvent',{type:'mousePressed',button:'left',clickCount:1,...point});await cdp('Input.dispatchMouseEvent',{type:'mouseReleased',button:'left',clickCount:1,...point});};
  for(const selector of ['.search svg','.search kbd']){
   await evaluate("document.querySelector('#search-q').blur()");
   const point=await evaluate(`(()=>{const r=document.querySelector(${JSON.stringify(selector)}).getBoundingClientRect();return{x:r.x+r.width/2,y:r.y+r.height/2}})()`);await clickAt(point);
   const state=await inputState(`trusted-click-${selector}`);inputChecks.push(state);if(state.active!=='search-q'||Math.abs(state.headerHeight-initialHeight)>0.025)problems.push({issue:'decorative search-slot click failed to focus unchanged-height input',state});
  }
  await evaluate("document.querySelector('#search-q').blur()");await cdp('Input.dispatchKeyEvent',{type:'keyDown',key:'/',code:'Slash',windowsVirtualKeyCode:191});await cdp('Input.dispatchKeyEvent',{type:'keyUp',key:'/',code:'Slash',windowsVirtualKeyCode:191});
  const shortcut=await inputState('trusted-slash-shortcut');inputChecks.push(shortcut);if(shortcut.active!=='search-q')problems.push({issue:'trusted Slash shortcut failed to focus input',shortcut});
  await cdp('Input.insertText',{text:'Memory'});await evaluate('new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(()=>resolve(true))))');
  const filled=await inputState('trusted-filled-input');inputChecks.push(filled);if(filled.value!=='Memory'||filled.active!=='search-q'||Math.abs(filled.headerHeight-initialHeight)>0.025)problems.push({issue:'filled focused input changed header height or input behavior',filled});
  const headerScreens=[];
  for(const profile of [{theme:'dark',scale:1},{theme:'light',scale:1},{theme:'dark',scale:1.25},{theme:'light',scale:2}]){
   await cdp('Emulation.setDeviceMetricsOverride',{width:1280,height,deviceScaleFactor:profile.scale,mobile:false});await cdp('Emulation.setEmulatedMedia',{features:[{name:'prefers-color-scheme',value:profile.theme}]});await evaluate('new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(()=>resolve(true))))');
   const graphic=await evaluate(String.raw`(async()=>{const svg=document.querySelector('.search svg'),use=svg.querySelector('use'),box=r=>({x:r.x,y:r.y,width:r.width,height:r.height}),matrix=m=>m?{a:m.a,b:m.b,c:m.c,d:m.d,e:m.e,f:m.f}:null,url=new URL(use.getAttribute('href'),location.href);const xml=new DOMParser().parseFromString(await(await fetch(url.origin+url.pathname)).text(),'image/svg+xml'),symbol=xml.getElementById(url.hash.slice(1));return{svgRect:box(svg.getBoundingClientRect()),svgBBox:box(svg.getBBox()),useBBox:box(use.getBBox()),screenCTM:matrix(svg.getScreenCTM()),svgViewBox:svg.getAttribute('viewBox'),symbolViewBox:symbol?.getAttribute('viewBox'),useHref:use.getAttribute('href'),headerHeight:document.querySelector('.top').getBoundingClientRect().height,inputRect:box(document.querySelector('#search-q').getBoundingClientRect()),kbdRect:box(document.querySelector('.search kbd').getBoundingClientRect()),rootTheme:document.documentElement.dataset.theme,background:getComputedStyle(document.querySelector('.top')).backgroundColor,offset:document.documentElement.style.getPropertyValue('--header-offset')};})()`);
   const file=`header-${profile.theme}-dpr${profile.scale}.png`,shot=await cdp('Page.captureScreenshot',{format:'png',fromSurface:true,clip:{x:0,y:0,width:1280,height:Math.ceil(graphic.headerHeight+4),scale:1}});await writeFile(path.join(out,file),Buffer.from(shot.data,'base64'));headerScreens.push({...profile,file,graphic});
   if(Math.abs(graphic.headerHeight-initialHeight)>0.025||Math.abs(Number.parseFloat(graphic.offset)-graphic.headerHeight)>0.025)problems.push({issue:'header changed height with theme or device scale',profile,graphic});
  }
  await evaluate('window.__headerWatch.disconnect()');
  await writeFile(path.join(out,'header-resize.json'),JSON.stringify({base:base.href,startPage,scale,theme,initialHeight,wrappedHeight,samples,inputChecks,headerScreens,problems,exceptions,fixture:'Actual production Shell bootstrap. Only .top-in max-width changes temporarily to exercise wrapping; restored afterwards. No observer/offset CSS implementation injected.'},null,2));
  console.log(JSON.stringify({mode:'header-resize',samples:samples.length,initialHeight,wrappedHeight,problems:problems.length,exceptions,output:out}));
  assert.deepEqual(problems,[],'actual header/rail geometry changed across measured resize frames');assert.deepEqual(exceptions,[]);break captureRun;
 }
 const explicitTarget=process.argv.find(arg=>arg.startsWith('--to='))?.slice(5);
 if(explicitTarget)await evaluate(`(()=>{const a=[...document.querySelectorAll('details.side a[href]')].find(a=>new URL(a.href).pathname===${JSON.stringify(new URL(explicitTarget,base).pathname)});if(!a)throw new Error('explicit destination absent');for(let p=a.parentElement;p;p=p.parentElement)if(p.tagName==='DETAILS')p.open=true;a.scrollIntoView({block:'nearest'});})()`);
 await delay(75);
 const target=explicitTarget?new URL(explicitTarget,base).href:await evaluate("(()=>{const current=document.querySelector('details.side [aria-current=page]');const list=current?.closest('ul');const a=list?[...list.querySelectorAll('a[href]')].find(a=>a!==current&&a.getBoundingClientRect().top>=80&&a.getBoundingClientRect().bottom<innerHeight):null;if(!a)return null;return a.href;})()");assert(target,'visible peer target absent');
 await cdp('Page.startScreencast',{format:'png',everyNthFrame:1});capture=true;
 await evaluate("window.__sideTrace.active=true");await delay(75);
 const clicked=await evaluate(`(()=>{const a=[...document.querySelectorAll('details.side a[href]')].find(a=>a.href===${JSON.stringify(target)});if(!a)return null;const r=a.getBoundingClientRect();return{x:r.x+r.width/2,y:r.y+r.height/2};})()`);assert(clicked,'visible same-list sidebar destination absent');
 const point={x:clicked.x*embedScale,y:clicked.y*embedScale};
 await cdp('Input.dispatchMouseEvent',{type:'mouseMoved',...point});
 await cdp('Input.dispatchMouseEvent',{type:'mousePressed',button:'left',clickCount:1,...point});
 await cdp('Input.dispatchMouseEvent',{type:'mouseReleased',button:'left',clickCount:1,...point});
 await delay(750);await evaluate('window.__sideTrace.active=false');capture=false;await cdp('Page.stopScreencast');
 const trace=await evaluate('window.__sideTrace');await writeFile(path.join(out,'trace.json'),JSON.stringify({trace,screens,exceptions,startPage,target,ablation,scale,theme,embedScale,width,height,gpu:process.argv.includes('--gpu')},null,2));
 const reference=trace.frames[0],source=new URL(startPage,base).href;
 const unchanged=new Map(reference.rows.filter(row=>![source,target].includes(row.href)).map(row=>[row.href,row]));
 const changes=[];
 for(const frame of trace.frames)for(const row of frame.rows){const first=unchanged.get(row.href);if(!first)continue;
  const local=(r,key)=>r[key][1]+(frame.scroll??0)-(frame.side?.[1]??0);
  const initial=(key)=>first[key][1]+(reference.scroll??0)-(reference.side?.[1]??0);
  if(Math.abs(local(row,'a')-initial('a'))>0.1||Math.abs(local(row,'li')-initial('li'))>0.1||Math.abs(row.a[3]-first.a[3])>0.1||row.text.some((line,index)=>first.text[index]&&Math.abs(line[1]+(frame.scroll??0)-(frame.side?.[1]??0)-(first.text[index][1]+(reference.scroll??0)-(reference.side?.[1]??0)))>0.1))changes.push({path:frame.path,time:frame.time,name:row.name,before:first.a,after:row.a,css:row.css});
 }
 await writeFile(path.join(out,'geometry.json'),JSON.stringify({frames:trace.frames.length,screens:screens.length,changes,exceptions},null,2));
 assert(trace.frames.length>0&&screens.length>1,'missing actual frame evidence');assert.equal(trace.frames.at(-1)?.path,new URL(target).pathname,'sidebar click never navigated');
 console.log(JSON.stringify({frames:trace.frames.length,screens:screens.length,geometryChanges:changes.length,exceptions,output:out}));
 assert.equal(changes.length,0,'unaffected sidebar rows changed geometry during a rendered animation frame');assert.deepEqual(exceptions,[]);
}finally{
 connection?.close();child.kill();await new Promise(resolve=>!child.pid||child.exitCode!==null||child.signalCode!==null?resolve():child.once('exit',resolve));await rm(profile,{recursive:true,force:true});
}
