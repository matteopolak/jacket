#!/usr/bin/env node
/** Check attached example/result geometry and diagnostic colors on real production fixtures.
 * node tools/dev/doc_examples.mjs OWNED_FIXTURE_ROOT_OR_PRODUCTION_URL OUTPUT_JSON
 */
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {createServer} from 'node:http';
import {readFile,writeFile,mkdtemp,rm,mkdir} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {setTimeout as delay} from 'node:timers/promises';
import {Connection,runtimeException} from './chrome_cdp.mjs';
import {findChrome,waitForDevToolsPort} from './chrome_startup.mjs';
const [fixture,output]=process.argv.slice(2);
assert(fixture&&output,'usage: doc_examples.mjs OWNED_FIXTURE_ROOT_OR_PRODUCTION_URL OUTPUT_JSON');
const captures=path.resolve(path.dirname(output),path.basename(output,'.json')+'-captures');
await mkdir(captures,{recursive:false});
const production=/^https?:\/\//u.test(fixture),root=production?undefined:path.resolve(fixture,'dist'),receipt={scope:production?'production-lint-reference':'fixture-operation-matrix',fixture:production?fixture:path.resolve(fixture),cases:[],exceptions:[],problems:[]};
const server=production?undefined:createServer(async(req,res)=>{try{const name=decodeURIComponent(new URL(req.url,'http://localhost').pathname).replace(/^\//,'');const file=path.resolve(root,name.includes('.')?name:`${name}.html`);if(!file.startsWith(root+path.sep))throw new Error('outside fixture');const bytes=await readFile(file);res.setHeader('Content-Type',({'.css':'text/css','.js':'text/javascript','.mjs':'text/javascript','.svg':'image/svg+xml','.json':'application/json'})[path.extname(file)]??'text/html');res.end(bytes);}catch{res.statusCode=404;res.end('Not found');}});
if(server)await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const base=production?fixture:`http://127.0.0.1:${server.address().port}/`,profile=await mkdtemp(path.join(tmpdir(),'jacket-examples-'));
const child=spawn(findChrome(),['--headless=new','--no-sandbox','--disable-gpu','--no-first-run','--remote-debugging-port=0',`--user-data-dir=${profile}`,'about:blank'],{stdio:['ignore','ignore','pipe']});
const closed=new Promise(resolve=>child.once('close',resolve));
let connection;
try{
 const port=await waitForDevToolsPort(child,profile),pages=await(await fetch(`http://127.0.0.1:${port}/json/list`)).json();
 connection=await new Connection(pages.find(page=>page.type==='page').webSocketDebuggerUrl).open();
 connection.listeners.push(message=>{const error=runtimeException(message);if(error!==undefined)receipt.exceptions.push(error);});
 const cdp=(method,params={})=>connection.cdp(method,params);
 const evaluate=async expression=>{const result=await cdp('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});assert(!result.exceptionDetails,JSON.stringify(result.exceptionDetails));return result.result.value;};
 await cdp('Page.enable');await cdp('Runtime.enable');
 for(const width of [375,1280])for(const theme of ['light','dark'])for(const rule of production?['self_assignment','duplicate_condition','use_after_free']:['self_assignment','duplicate_condition','use_after_free','diagnostic-operations']){
  await cdp('Emulation.setDeviceMetricsOverride',{width,height:900,deviceScaleFactor:1,mobile:false});
  await cdp('Emulation.setEmulatedMedia',{features:[{name:'prefers-color-scheme',value:theme}]});
  const previous=await evaluate('performance.timeOrigin');await cdp('Page.navigate',{url:new URL(production?`lint-rules/${rule}`:rule,base).href});
  let ready=false;for(let n=0;n<200;n++){if(await evaluate(`performance.timeOrigin!==${previous}&&document.readyState==='complete'&&!!document.querySelector('pre.code')`)){ready=true;break;}await delay(25);}assert(ready,rule+' page did not load');
  await evaluate(`document.documentElement.dataset.theme=${JSON.stringify(theme)}`);
  await evaluate('document.fonts.ready');
  const sample=await evaluate(String.raw`(()=>{
   const rect=node=>{const r=node.getBoundingClientRect();return{x:r.x,y:r.y,width:r.width,height:r.height,bottom:r.bottom};};
   const groups=[...document.querySelectorAll('.example-group')].map(group=>{
    const source=group.querySelector(':scope>.example-source'),result=group.querySelector(':scope>.example-result'),header=group.querySelector(':scope>figcaption');
    const deletions=result?[...result.querySelectorAll('.dd')].map(node=>({text:node.textContent,color:getComputedStyle(node).color})):[];
    const origins=result?[...result.querySelectorAll('.dk')].map(node=>({text:node.textContent,color:getComputedStyle(node).color})):[];
    return{header:header.textContent.trim(),resultText:result?.querySelector('code')?.textContent??null,group:rect(group),source:rect(source),result:result?rect(result):null,resultBorder:result?parseFloat(getComputedStyle(result).borderTopWidth):0,panes:[...group.querySelectorAll('pre.code')].map(node=>({padding:getComputedStyle(node).padding,radius:getComputedStyle(node).borderRadius,margin:getComputedStyle(node).margin})),deletions,origins};
   });
   const rgb=value=>(value.match(/[\d.]+/g)??[]).slice(0,3).map(Number);
   const luminance=value=>rgb(value).map(channel=>{channel/=255;return channel<=0.04045?channel/12.92:((channel+0.055)/1.055)**2.4;}).reduce((sum,value,index)=>sum+value*[0.2126,0.7152,0.0722][index],0);
   const contrast=(foreground,background)=>{const a=luminance(foreground),b=luminance(background);return(Math.max(a,b)+0.05)/(Math.min(a,b)+0.05);};
   const operations=[...document.querySelectorAll('.dd,.da,.dc,.dk')].map(node=>{
    let background=node;while(background.parentElement&&getComputedStyle(background).backgroundColor==='rgba(0, 0, 0, 0)')background=background.parentElement;
    const color=getComputedStyle(node).color,bg=getComputedStyle(background).backgroundColor;
    return{role:node.className,text:node.textContent,color,background:bg,contrast:contrast(color,bg)};
   });
   const palette=getComputedStyle(document.documentElement);const probe=document.createElement('span');document.body.append(probe);const color=name=>{probe.style.color=palette.getPropertyValue(name);return getComputedStyle(probe).color;};const colors={error:color('--err'),warning:color('--warn'),addition:color('--ok'),replacement:color('--warn'),secondary:color('--type')};probe.remove();
   return{groups,colors,operations,theme:document.documentElement.dataset.theme,colorScheme:getComputedStyle(document.documentElement).colorScheme,contrastCalibration:contrast('rgb(0, 0, 0)','rgb(255, 255, 255)'),viewport:innerWidth,bodyWidth:document.documentElement.scrollWidth,plainLabels:[...document.querySelectorAll('p')].filter(node=>['Example:','Fixed:'].includes(node.textContent.trim())).length};
  })()`);
  assert.equal(sample.theme,theme);assert.equal(sample.colorScheme,theme);
  assert(Math.abs(sample.contrastCalibration-21)<1e-9,'contrast arithmetic calibration must map white/black to21');
  const metrics=await cdp('Page.getLayoutMetrics'),captureHeight=Math.ceil(metrics.cssContentSize.height);
  const file=`${theme}-${width}-${rule}.png`;
  const screenshot=await cdp('Page.captureScreenshot',{format:'png',captureBeyondViewport:true,clip:{x:0,y:0,width,height:captureHeight,scale:1}});
  await writeFile(path.join(captures,file),Buffer.from(screenshot.data,'base64'));
  receipt.cases.push({width,theme,rule,file,captureHeight,...sample});
  if(sample.bodyWidth>width+1)receipt.problems.push({width,theme,rule,reason:'page horizontal overflow',bodyWidth:sample.bodyWidth});
  if(sample.plainLabels)receipt.problems.push({width,theme,rule,reason:'plain example labels'});
  if(rule!=='diagnostic-operations'){
   if(sample.groups.length!==2||sample.groups[0]?.header!=='Example'||!sample.groups[0]?.result||sample.groups[1]?.header!=='Fixed'||sample.groups[1]?.result)receipt.problems.push({width,theme,rule,reason:'source/result/fixed ownership missing',groups:sample.groups});
   const result=sample.groups[0]?.resultText??'',fixLines=result.split('\n').filter(line=>/^\s*\d+\s+[-+~]/u.test(line));
   if(rule==='self_assignment'&&(fixLines.length!==1||!/-\s+width = width;/u.test(fixLines[0])))receipt.problems.push({width,theme,rule,reason:'statement deletion is not minimal',fixLines});
   if(rule==='duplicate_condition'&&(fixLines.length!==1||!/-\s+else if key == 1 print/u.test(fixLines[0])))receipt.problems.push({width,theme,rule,reason:'branch deletion is not minimal',fixLines});
  }

  if(rule==='diagnostic-operations'){
   for(const role of ['dd','da','dc','dk'])if(sample.operations.filter(operation=>operation.role===role).length!==2)receipt.problems.push({width,theme,rule,reason:'operation coverage missing',role});
   for(const operation of sample.operations){
    const expected=operation.role==='dd'?sample.colors.error:operation.role==='da'?sample.colors.addition:operation.role==='dk'?sample.colors.secondary:operation.role==='dc'?sample.colors.replacement:null;
    if(expected&&operation.color!==expected)receipt.problems.push({width,theme,rule,reason:'wrong operation color',operation,expected});
    if(operation.contrast<4.5)receipt.problems.push({width,theme,rule,reason:'operation contrast below4.5',operation});
   }
  }
  for(const group of sample.groups){
   for(const pane of group.panes)if(pane.radius!=='0px'||pane.padding!=='14px 18px'||pane.margin!=='0px')receipt.problems.push({width,theme,rule,reason:'attached pane spacing or inner radius',pane});
   if(group.result&&Math.abs(group.result.y-group.source.bottom)>0.1)receipt.problems.push({width,theme,rule,reason:'source/result detached',group});
   if(group.result&&group.resultBorder!==1)receipt.problems.push({width,theme,rule,reason:'missing shared separator',group});
   for(const deletion of group.deletions)if(deletion.color!==sample.colors.error||deletion.color===sample.colors.warning)receipt.problems.push({width,theme,rule,reason:'deletion is not independently red',deletion,colors:sample.colors});
   for(const origin of group.origins)if(origin.color===sample.colors.error)receipt.problems.push({width,theme,rule,reason:'secondary origin is deletion red',origin});
  }
 }
 await writeFile(output,JSON.stringify(receipt,null,2)+'\n');
 assert.deepEqual(receipt.exceptions,[]);assert.deepEqual(receipt.problems,[]);
 console.log(JSON.stringify({cases:receipt.cases.length,problems:receipt.problems.length,exceptions:receipt.exceptions.length}));
}finally{
 connection?.close();child.kill('SIGTERM');await closed;await rm(profile,{recursive:true,force:true});if(server)await new Promise(resolve=>server.close(resolve));
}
