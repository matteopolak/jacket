import {nativeTransitionState} from './doc_browser_content.mjs';
/** CDP byte accounting: keep original event boundaries and never infer cached transfer sizes. */
export function category(row){
 const path=new URL(row.url).pathname;
 if(row.type==='Document')return 'document';
 if(/(?:^|\/)assets\/nav\/api\/.+\/\d+\.html$/u.test(path))return 'exportData';
 if(row.mime?.includes('text/html'))return 'document';
 if(/(?:^|\/)runtime\//u.test(path)||/\.(?:wasm)$/u.test(path))return 'runtime';
 if(/(?:^|\/)search(?:[./_-]|$)/u.test(path))return 'search';
 if(/(?:^|\/)navigation\//u.test(path)||/\.(?:json)$/u.test(path))return 'exportData';
 if(row.type==='Font'||/\.(?:woff2?|otf|ttf)$/u.test(path))return 'fonts';
 if(row.type==='Stylesheet'||/\.css$/u.test(path))return 'css';
 if(row.type==='Script'||/\.(?:m?js)$/u.test(path))return 'javascript';
 return 'other';
}
export class RequestLedger{
 rows=[];active=new Map();
 event(method,p,sessionId){
  const key=JSON.stringify([sessionId??null,p.requestId]);
  let row=this.active.get(key);
  if(method==='Network.requestWillBeSent'){
   if(p.redirectResponse&&row){this.response(row,p.redirectResponse);row.redirect=true;row.encodedTransferBytes=p.redirectResponse.encodedDataLength;row.finished=p.timestamp;}
   row={id:p.requestId,sessionId,hop:this.rows.filter(r=>r.id===p.requestId&&r.sessionId===sessionId).length,url:p.request.url,type:p.type,started:p.timestamp,documentURL:p.documentURL,loaderId:p.loaderId,initiator:p.initiator,requestHeaders:p.request.headers,decodedDataBytes:0,encodedDataEventBytes:0};
   this.rows.push(row);this.active.set(key,row);return;
  }
  if(!row)return;
  if(method==='Network.responseReceived'){this.response(row,p.response);row.responseTime=p.timestamp;}
  if(method==='Network.requestServedFromCache')row.servedFromCache=true;
  if(method==='Network.dataReceived'){row.decodedDataBytes+=p.dataLength;row.encodedDataEventBytes+=p.encodedDataLength;}
  if(method==='Network.loadingFinished'){row.finished=p.timestamp;row.encodedTransferBytes=p.encodedDataLength;}
  if(method==='Network.loadingFailed'){row.finished=p.timestamp;row.failure=p.errorText;row.canceled=p.canceled;row.blockedReason=p.blockedReason;}
 }
 response(row,response){Object.assign(row,{status:response.status,mime:response.mimeType,responseHeaders:response.headers,diskCache:response.fromDiskCache??false,prefetchCache:response.fromPrefetchCache??false,serviceWorker:response.fromServiceWorker??false,protocol:response.protocol,remoteIPAddress:response.remoteIPAddress,timing:response.timing});}
}
export function summarizeRequests(rows){
 const empty=()=>({requests:0,cacheHits:0,encodedTransferBytes:0,decodedDataBytes:0,unknownTransferSizes:0,decodedBodyBytes:0,unknownDecodedBodySizes:0,failures:0});const total=empty(),categories={};
 for(const row of rows){const key=category(row),bucket=categories[key]??=empty();for(const target of [total,bucket]){target.requests++;target.cacheHits+=Number(!!(row.diskCache||row.prefetchCache||row.servedFromCache));if(row.encodedTransferBytes===undefined)target.unknownTransferSizes++;else target.encodedTransferBytes+=row.encodedTransferBytes;target.decodedDataBytes+=row.decodedDataBytes??0;if(row.decodedBodyBytes===undefined)target.unknownDecodedBodySizes++;else target.decodedBodyBytes+=row.decodedBodyBytes;target.failures+=Number(!!row.failure);}categories[key]=bucket;}
 return {total,categories};
}

/** Installed before document scripts; buffers are retained across ClientRouter swaps. */
export const observerSource=String.raw`(()=>{
 const m=window.__docMetrics={origin:performance.timeOrigin,supported:PerformanceObserver.supportedEntryTypes,paint:[],largestContentfulPaint:[],layoutShift:[],longTask:[],longAnimationFrame:[],resource:[],events:[],frames:[],scrolling:false,transitionSamples:[]};
 for(const [type,key] of [['paint','paint'],['largest-contentful-paint','largestContentfulPaint'],['layout-shift','layoutShift'],['longtask','longTask'],['long-animation-frame','longAnimationFrame'],['resource','resource']]){
  if(!m.supported.includes(type))continue;
  new PerformanceObserver(list=>{for(const entry of list.getEntries()){const data=entry.toJSON();if(type==='layout-shift'){data.hadRecentInput=entry.hadRecentInput;data.value=entry.value;data.sources=entry.sources.map(source=>({previousRect:source.previousRect?.toJSON(),currentRect:source.currentRect?.toJSON(),node:source.node?.tagName}));}m[key].push(data);}}).observe({type,buffered:true});
 }
 for(const event of ['click','input','keydown'])document.addEventListener(event,e=>{if(e.isTrusted&&window.__docBenchmarkInput)window.__docBenchmarkInput(JSON.stringify({event,time:performance.now(),origin:performance.timeOrigin,path:location.pathname,target:e.target?.id??null}));},{capture:true});
 for(const event of ['astro:before-preparation','astro:after-preparation','astro:before-swap','astro:after-swap','astro:page-load','pageshow','pagehide','popstate','hashchange'])(event.startsWith('astro:')?document:window).addEventListener(event,e=>m.events.push({event,time:performance.now(),path:location.pathname,hash:location.hash,persisted:e.persisted}));
 const nativeTransitionState=${nativeTransitionState.toString()};let previousTransition;
 function frame(time){const native=nativeTransitionState();if(native.active!==previousTransition){m.transitionSamples.push({time,active:native.active,supported:native.supported,path:location.pathname});previousTransition=native.active;}if(m.scrolling)m.frames.push(time);requestAnimationFrame(frame);}requestAnimationFrame(frame);
})();`;
/** Raw observations remain authoritative; CLS follows the 1s-gap/5s session-window definition. */
export function summarizeObservations(observations,frameBudgetMs){
 let cls=0,score=0,start=-Infinity,last=-Infinity;
 for(const shift of observations.layoutShift??[]){if(shift.hadRecentInput)continue;if(shift.startTime-last>=1000||shift.startTime-start>=5000){score=0;start=shift.startTime;}score+=shift.value;last=shift.startTime;cls=Math.max(cls,score);}
 const frames=observations.frames??[],intervals=frames.slice(1).map((time,index)=>time-frames[index]);
 return {firstPaintMs:observations.paint?.find(entry=>entry.name==='first-paint')?.startTime??null,fcpMs:observations.paint?.find(entry=>entry.name==='first-contentful-paint')?.startTime??null,latestLcpMs:observations.largestContentfulPaint?.at(-1)?.startTime??null,cls:observations.supported?.includes('layout-shift')?cls:null,longTaskCount:observations.supported?.includes('longtask')?(observations.longTask?.length??0):null,longTaskTotalMs:observations.supported?.includes('longtask')?(observations.longTask??[]).reduce((sum,entry)=>sum+entry.duration,0):null,frameIntervalsMs:intervals,frameBudgetMs,framesOverBudget:intervals.filter(value=>value>frameBudgetMs).length};
}
