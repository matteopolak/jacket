#!/usr/bin/env node
/** Compare actual module declaration-band CSS/outer geometry with a pinned legacy site.
 * node tools/dev/doc_declaration_bands.mjs CANDIDATE_URL BASELINE_DIRECTORY OUTPUT_JSON
 * Optional --band-css SOURCE_CSS applies only the source band rules as a diagnostic ablation.
 */
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {createHash} from 'node:crypto';
import {declarationBands} from './doc_bands.mjs';
import {createServer} from 'node:http';
import {readFile,writeFile,mkdtemp,rm,stat,mkdir} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {setTimeout as delay} from 'node:timers/promises';
import {Connection,runtimeException} from './chrome_cdp.mjs';
import {findChrome,waitForDevToolsPort} from './chrome_startup.mjs';
const [candidate,baselineDirectory,output]=process.argv.slice(2);assert(candidate&&baselineDirectory&&output);
const chrome=findChrome();
const baselineRoot=path.resolve(baselineDirectory),profile=await mkdtemp(path.join(tmpdir(),'jacket-bands-'));
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const receipt={candidate,baseline:'pinned legacy directory',page:'api/Basic',scenarios:[],exceptions:[],diagnosticAblation:process.argv.includes('--band-css')};
receipt.inputs={baselineHtmlSha256:hash(await readFile(path.join(baselineRoot,'api/Basic.html'))),baselineCssSha256:hash(await readFile(path.join(baselineRoot,'assets/site.css'))),candidateHtmlSha256:hash(Buffer.from(await(await fetch(new URL('api/Basic',candidate))).arrayBuffer()))};
const server=createServer(async(request,response)=>{try{const relative=decodeURIComponent(new URL(request.url,'http://localhost').pathname).replace(/^\//u,'');let file=path.resolve(baselineRoot,relative||'index.html');assert(file.startsWith(baselineRoot+path.sep));try{if(!(await stat(file)).isFile())throw new Error();}catch{file+='.html';}const body=await readFile(file);const ext=path.extname(file);response.writeHead(200,{'Content-Type':({'.html':'text/html; charset=utf-8','.css':'text/css','.js':'text/javascript','.json':'application/json','.wasm':'application/wasm'})[ext]??'application/octet-stream'});response.end(body);}catch{response.writeHead(404);response.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const baseline=`http://127.0.0.1:${server.address().port}/`;
const child=spawn(chrome,['--headless=new','--no-sandbox','--disable-gpu','--no-first-run','--remote-debugging-port=0',`--user-data-dir=${profile}`,'about:blank'],{stdio:['ignore','ignore','pipe']});let connection;
try{
 const port=await waitForDevToolsPort(child,profile),pages=await(await fetch(`http://127.0.0.1:${port}/json/list`)).json();connection=await new Connection(pages.find(page=>page.type==='page').webSocketDebuggerUrl).open();
 connection.listeners.push(message=>{const error=runtimeException(message);if(error!==undefined)receipt.exceptions.push(error);});const cdp=(method,params={})=>connection.cdp(method,params);
 const evaluate=async expression=>{const result=await cdp('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});assert(!result.exceptionDetails,JSON.stringify(result.exceptionDetails));return result.result.value;};
 await cdp('Page.enable');await cdp('Runtime.enable');
 if(process.argv.includes('--links')){
  const captures=output.replace(/\.json$/u,'')+'-captures';await mkdir(captures,{recursive:false});receipt.decoration=[];
  const categories=[['code','pre.code a[href]'],['card-title','.row-head a.name'],['signature','.sig-line a[href]'],['source','.row-head a.muted'],['badge','.row-head .badge'],['navigation','details.side .nav-list a[href]'],['prose','main p a[href]:not(.btn)'],['inline-prose','main p a:has(code),main p code a']];
  await cdp('DOM.enable');await cdp('CSS.enable');
  for(const theme of ['light','dark'])for(const [variant,url]of [['baseline',baseline],['candidate',candidate]]){
   await cdp('Emulation.setDeviceMetricsOverride',{width:1280,height:900,deviceScaleFactor:1,mobile:false});await cdp('Emulation.setEmulatedMedia',{features:[{name:'prefers-color-scheme',value:theme}]});
   const origin=await evaluate('performance.timeOrigin');await cdp('Page.navigate',{url:new URL('api/Basic',url).href});let ready=false;
   for(let i=0;i<600;i++){if(await evaluate(`performance.timeOrigin!==${origin}&&location.origin===${JSON.stringify(new URL(url).origin)}&&document.readyState==='complete'&&!!document.querySelector('.row-head a.name')`)){ready=true;break;}await delay(25);}assert(ready);
   await evaluate('document.fonts.ready');await delay(100);
   if(variant==='candidate'&&process.argv.includes('--link-css')){
    const source=await readFile(process.argv[process.argv.indexOf('--link-css')+1],'utf8'),rule=source.split('\n').find(line=>line.includes('text-underline-offset: 0.15em'));
    const prose=source.split('\n').find(line=>line.startsWith('main :where(')&&line.includes('text-decoration: underline'));
    assert(rule&&prose);receipt.inputs.decorationRules={base:rule.slice(rule.indexOf('{')+1,rule.lastIndexOf('}')),prose};
    const changed=await evaluate(`(()=>{let changed=0;for(const sheet of document.styleSheets)for(const rule of [...sheet.cssRules])if(rule.selectorText==='main a'){rule.style.cssText=${JSON.stringify(receipt.inputs.decorationRules.base)};sheet.insertRule(${JSON.stringify(prose)},0);changed++;}return changed;})()`);assert.equal(changed,1,'source rule ablation must replace exactly the broad rule');
   }
   for(const [category,selector]of categories){
    if(category==='prose'||category==='inline-prose'){
     const origin=await evaluate('performance.timeOrigin');await cdp('Page.navigate',{url:new URL(category==='prose'?'guides/testing':'tools/compiling',url).href});let ready=false;
     for(let i=0;i<600;i++){if(await evaluate(`performance.timeOrigin!==${origin}&&document.readyState==='complete'&&!!document.querySelector('main p a[href]')`)){ready=true;break;}await delay(25);}assert(ready,'guide prose sample absent');await evaluate('document.fonts.ready');
     if(variant==='candidate'&&process.argv.includes('--link-css'))assert.equal(await evaluate(`(()=>{let changed=0;for(const sheet of document.styleSheets)for(const rule of [...sheet.cssRules])if(rule.selectorText==='main a'){rule.style.cssText=${JSON.stringify(receipt.inputs.decorationRules.base)};sheet.insertRule(${JSON.stringify(receipt.inputs.decorationRules.prose)},0);changed++;}return changed;})()`),1);
    }
    const {root}=await cdp('DOM.getDocument'),{nodeId}=await cdp('DOM.querySelector',{nodeId:root.nodeId,selector});assert(nodeId,`${variant}: absent ${category}`);
    await evaluate(`(()=>{const node=document.querySelector(${JSON.stringify(selector)});for(let p=node.parentElement;p;p=p.parentElement){if(p.tagName==='DETAILS')p.open=true;if(p.classList.contains('fold'))p.classList.add('open');}node.scrollIntoView({block:'center'});})()`);
    const matched=await cdp('CSS.getMatchedStylesForNode',{nodeId});const cascade=(matched.matchedCSSRules??[]).filter(match=>match.rule.style.cssProperties.some(property=>property.name.startsWith('text-decoration'))).map(match=>({selector:match.rule.selectorList.text,properties:match.rule.style.cssProperties.filter(property=>property.name.startsWith('text-decoration'))}));
    for(const state of ['normal','hover','focus']){
     if(category==='badge'&&state==='focus')continue;
     await cdp('CSS.forcePseudoState',{nodeId,forcedPseudoClasses:state==='hover'?['hover']:state==='focus'?['focus','focus-visible']:[]});await delay(30);
     const actual=await evaluate(`(()=>{const node=document.querySelector(${JSON.stringify(selector)}),style=getComputedStyle(node),r=node.getBoundingClientRect();return{inlineCode:!!(node.querySelector('code')||node.closest('code')),text:node.textContent.trim(),href:node.getAttribute('href'),tag:node.tagName,line:style.textDecorationLine,weight:style.fontWeight,surroundingWeight:getComputedStyle(node.parentElement).fontWeight,style:style.textDecorationStyle,color:style.textDecorationColor,offset:style.textUnderlineOffset,hover:node.matches(':hover'),focus:node.matches(':focus-visible'),box:{x:Math.max(0,r.left+scrollX-8),y:Math.max(0,r.top+scrollY-8),width:Math.min(r.width+16,1100),height:Math.min(r.height+16,180),scale:1}};})()`);
     const file=`${variant}-${theme}-${category}-${state}.png`,capture=await cdp('Page.captureScreenshot',{format:'png',clip:actual.box,captureBeyondViewport:true});await writeFile(path.join(captures,file),Buffer.from(capture.data,'base64'));
     const record={variant,theme,category,state,...actual,cascade,file};receipt.decoration.push(record);
    }
    await cdp('CSS.forcePseudoState',{nodeId,forcedPseudoClasses:[]});
   }
  }
  receipt.decorationProblems=[];
  for(const row of receipt.decoration.filter(row=>row.variant==='candidate')){
   const baseline=receipt.decoration.find(other=>other.variant==='baseline'&&other.theme===row.theme&&other.category===row.category&&other.state===row.state);
   const expected=['code','card-title','signature','source'].includes(row.category)&&row.state!=='hover'?'none':baseline.line;
   if(row.category==='code'&&row.state==='normal'&&row.line==='none'&&row.weight===row.surroundingWeight)receipt.decorationProblems.push({category:row.category,theme:row.theme,reason:'code link has no non-color cue',weight:row.weight,surroundingWeight:row.surroundingWeight});
   if(row.line!==expected)receipt.decorationProblems.push({category:row.category,state:row.state,theme:row.theme,baseline:baseline.line,actual:row.line,expected});if(row.offset!==baseline.offset)receipt.decorationProblems.push({category:row.category,state:row.state,theme:row.theme,property:'underline-offset',expected:baseline.offset,actual:row.offset});
  }
  await writeFile(output,JSON.stringify(receipt,null,2));console.log(JSON.stringify({records:receipt.decoration.length,problems:receipt.decorationProblems,exceptions:receipt.exceptions,captures}));assert.deepEqual(receipt.exceptions,[]);assert.deepEqual(receipt.decorationProblems,[],'user-requested code/card link decoration policy violated');
 }else{
 const sample=String.raw`(()=>{
 const properties=['marginTop','marginRight','marginBottom','marginLeft','paddingTop','paddingRight','paddingBottom','paddingLeft','backgroundColor','backgroundImage','borderTopWidth','borderRightWidth','borderBottomWidth','borderLeftWidth','borderTopStyle','borderRightStyle','borderBottomStyle','borderLeftStyle','borderTopLeftRadius','borderTopRightRadius','borderBottomRightRadius','borderBottomLeftRadius'];
 const box=node=>{const r=node.getBoundingClientRect();return{x:r.x,y:r.y,width:r.width,height:r.height}};
 return ['String_Builder','Memory_Debugger','Calendar_Time'].map(id=>{const row=document.getElementById(id),members=row?.querySelector(':scope>.members'),band=members?.querySelector(':scope>pre.sig,:scope>.semantic-declaration');if(!band)throw new Error('missing real declaration band '+id);const css=getComputedStyle(band),rb=box(row),mb=box(members),bb=box(band);return{id,tag:band.tagName,fold:{open:members.classList.contains('open'),lastChild:members.matches(':last-child'),scrollHeight:members.scrollHeight,limit:Number(members.dataset.fold),button:members.nextElementSibling?.classList.contains('fold-btn')??false,variants:members.querySelectorAll(':scope>pre.sig,:scope>.semantic-declaration').length},styles:Object.fromEntries(properties.map(property=>[property,css[property]])),geometry:{leftInset:bb.x-mb.x,rightInset:mb.x+mb.width-bb.x-bb.width,topInset:bb.y-mb.y,bandWidth:bb.width,membersWidth:mb.width,rowWidth:rb.width,rowLeftInset:mb.x-rb.x,rowRightInset:rb.x+rb.width-mb.x-mb.width},formatterCode:band.matches('pre.sig.decl.semantic-declaration')&&band.querySelector(':scope>code')?.textContent.length>0};});
 })()`;
 for(const width of [1280,375])for(const theme of ['light','dark']){
  const scenario={width,theme};receipt.scenarios.push(scenario);await cdp('Emulation.setDeviceMetricsOverride',{width,height:900,deviceScaleFactor:1,mobile:false});await cdp('Emulation.setEmulatedMedia',{features:[{name:'prefers-color-scheme',value:theme}]});
  for(const [variant,url]of [['baseline',baseline],['candidate',candidate]]){
   await cdp('Page.navigate',{url:new URL('api/Basic',url).href});let ready=false;for(let i=0;i<600;i++){if(await evaluate(`location.origin===${JSON.stringify(new URL(url).origin)}&&location.pathname===${JSON.stringify(new URL("api/Basic",url).pathname)}&&document.readyState==='complete'&&!!document.querySelector('#Calendar_Time>.members')`)){ready=true;break;}await delay(25);}assert(ready,`${variant} module never loaded`);
   if(variant==='candidate'&&receipt.diagnosticAblation){const source=await readFile(process.argv[process.argv.indexOf('--band-css')+1],'utf8'),match=/\/\* A written declaration fills the band the fields table does\. \*\/([\s\S]*?)\n\.unexported/u.exec(source);assert(match,'source band rules not found');receipt.inputs.ablationRulesSha256=hash(match[1]);await evaluate(`(()=>{const style=document.createElement('style');style.textContent=${JSON.stringify(match[1])};document.head.append(style);})()`);}
   await evaluate('document.fonts.ready');await delay(150);scenario[variant]=await evaluate(sample);scenario[`${variant}Policy`]=await evaluate(`(${declarationBands.toString()})()`);
  }
  scenario.mismatches=[...scenario.baselinePolicy.problems.map(problem=>({baselinePolicy:problem})),...scenario.candidatePolicy.problems.map(problem=>({candidatePolicy:problem}))];
  for(const actual of scenario.candidate){const expected=scenario.baseline.find(row=>row.id===actual.id);assert(expected);assert.equal(actual.tag,'PRE');assert(actual.formatterCode,'formatter-authored declaration PRE/code was lost');for(const [name,value]of Object.entries(expected.styles)){const policyValue=name==='borderBottomWidth'?(actual.fold.lastChild?'0px':'1px'):value;if(name==='borderBottomWidth'&&value!==(expected.fold.lastChild?'0px':'1px'))scenario.mismatches.push({id:actual.id,variant:'baseline',property:name,expected:expected.fold.lastChild?'0px':'1px',actual:value});if(actual.styles[name]!==policyValue)scenario.mismatches.push({id:actual.id,property:name,expected:policyValue,actual:actual.styles[name]});}for(const [name,value]of Object.entries(expected.geometry))if(Math.abs(actual.geometry[name]-value)>.1)scenario.mismatches.push({id:actual.id,geometry:name,expected:value,actual:actual.geometry[name]});}
 }
 await writeFile(output,JSON.stringify(receipt,null,2));assert.deepEqual(receipt.exceptions,[]);const mismatches=receipt.scenarios.flatMap(scenario=>scenario.mismatches);console.log(JSON.stringify({scenarios:receipt.scenarios.length,bands:receipt.scenarios.length*3,mismatches}));assert.deepEqual(mismatches,[],'module declaration band differs from the legacy outer presentation');
 }
}finally{await writeFile(output,JSON.stringify(receipt,null,2));connection?.close();child.kill();await new Promise(resolve=>child.exitCode!==null||child.signalCode!==null?resolve():child.once('exit',resolve));await new Promise(resolve=>server.close(resolve));await rm(profile,{recursive:true,force:true});}
