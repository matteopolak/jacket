import assert from 'node:assert/strict';
import test from 'node:test';
import vm from 'node:vm';
import {RequestLedger,category,summarizeRequests,observerSource} from './doc_browser_accounting.mjs';
test('CDP accounting retains redirect hops, actual transfer fields and failures',()=>{
 const ledger=new RequestLedger();
 ledger.event('Network.requestWillBeSent',{requestId:'a',timestamp:1,type:'Document',request:{url:'https://example.test/a'},initiator:{type:'other'}});
 ledger.event('Network.requestWillBeSent',{requestId:'a',timestamp:2,type:'Document',request:{url:'https://example.test/b'},redirectResponse:{url:'https://example.test/a',status:301,encodedDataLength:120,headers:{Location:'/b'}},initiator:{type:'other'}});
 ledger.event('Network.responseReceived',{requestId:'a',timestamp:3,response:{url:'https://example.test/b',mimeType:'text/html',status:200,headers:{'Content-Encoding':'gzip'},fromDiskCache:false}});
 ledger.event('Network.dataReceived',{requestId:'a',dataLength:900,encodedDataLength:210});
 ledger.event('Network.loadingFinished',{requestId:'a',timestamp:4,encodedDataLength:250});
 ledger.event('Network.requestWillBeSent',{requestId:'fail',timestamp:4,type:'Fetch',request:{url:'https://example.test/search/index.json'},initiator:{type:'script'}});
 ledger.event('Network.loadingFailed',{requestId:'fail',timestamp:5,errorText:'net::ERR_ABORTED',canceled:true});
 assert.equal(ledger.rows.length,3);assert.equal(ledger.rows[0].redirect,true);assert.equal(ledger.rows[0].encodedTransferBytes,120);
 assert.equal(ledger.rows[1].encodedTransferBytes,250);assert.equal(ledger.rows[1].decodedDataBytes,900);assert.equal(ledger.rows[2].failure,'net::ERR_ABORTED');
});
test('cache and prefetch rows remain in session totals without inventing unknown byte sizes',()=>{
 const rows=[{url:'https://a/_astro/a.js',type:'Script',encodedTransferBytes:0,decodedDataBytes:300,diskCache:true},{url:'https://a/api/X',type:'Fetch',mime:'text/html',encodedTransferBytes:700,decodedDataBytes:1200,prefetchCache:false,initiator:{type:'script'}},{url:'https://a/no-size',type:'Other'}];
 const summary=summarizeRequests(rows);assert.equal(summary.total.requests,3);assert.equal(summary.total.cacheHits,1);assert.equal(summary.total.encodedTransferBytes,700);assert.equal(summary.total.unknownTransferSizes,1);assert.equal(summary.categories.document.requests,1);
 assert.equal(category({url:'https://a/navigation/abcd.json'}),'exportData');assert.equal(category({url:'https://a/assets/runtime/engine.js'}),'runtime');assert.equal(category({url:'https://a/search/index.json'}),'search');assert.equal(category({url:'https://a/search-index.json'}),'search');assert.equal(category({url:'https://a/api/search-index',type:'Document',mime:'text/html'}),'document');
});
test('CLS uses maximum session window and excludes recent-input shifts',async()=>{
 const {summarizeObservations}=await import('./doc_browser_accounting.mjs');
 const result=summarizeObservations({supported:['paint','largest-contentful-paint','layout-shift','longtask'],paint:[{name:'first-contentful-paint',startTime:123}],largestContentfulPaint:[{startTime:300}],layoutShift:[{startTime:0,value:.1,hadRecentInput:false},{startTime:900,value:.2,hadRecentInput:false},{startTime:2100,value:.25,hadRecentInput:false},{startTime:2300,value:9,hadRecentInput:true}],longTask:[{duration:80}],frames:[10,26,58]},16);
 assert.equal(result.cls,.30000000000000004);assert.equal(result.fcpMs,123);assert.equal(result.latestLcpMs,300);assert.deepEqual(result.frameIntervalsMs,[16,32]);assert.equal(result.framesOverBudget,1);assert.equal(result.longTaskTotalMs,80);
});
test('browser plan validation never launches the configured browser',async()=>{
 const {mkdtemp,writeFile,rm}=await import('node:fs/promises');const {tmpdir}=await import('node:os');const {join}=await import('node:path');const {spawnSync}=await import('node:child_process');
 const directory=await mkdtemp(join(tmpdir(),'jacket-browser-plan-'));
 try{
  const file=join(directory,'plan.json');const plan={schema:'jacket-doc-browser/1',repetitions:3,identities:{fixture:'test'},equivalence:{fixture:'test'},chrome:'/must/not/be/launched',variants:[{name:'baseline',base:'http://127.0.0.1:1/'},{name:'candidate',base:'http://127.0.0.1:2/'}],profiles:[{name:'fixture',width:1000,height:800,scale:1,cpuRate:1,frameBudgetMs:17,network:{latency:0,downloadThroughput:-1,uploadThroughput:-1}}],scenarios:[{name:'cold',cacheState:'empty profile',actions:[{type:'goto',path:'/',expect:{path:'/',selector:'main'}}]}]};
  await writeFile(file,JSON.stringify(plan));const script=new URL('./doc_browser.mjs',import.meta.url);
  const valid=spawnSync(process.execPath,[script.pathname,'--plan',file],{encoding:'utf8'});assert.equal(valid.status,0,valid.stderr);assert.match(valid.stdout,/no browser launched or measured/u);
  plan.scenarios[0].actions[0].expect={};await writeFile(file,JSON.stringify(plan));const invalid=spawnSync(process.execPath,[script.pathname,'--plan',file],{encoding:'utf8'});assert.notEqual(invalid.status,0);assert.match(invalid.stderr,/useful-content predicate/u);
 }finally{await rm(directory,{recursive:true,force:true});}
});

test('document-dispatched Astro lifecycle is observed without requiring event bubbling',()=>{
 const window=new EventTarget(),document=new EventTarget();document.activeViewTransition=null;
 vm.runInNewContext(observerSource,{window,document,location:{pathname:'/',hash:''},performance:{timeOrigin:1,now:()=>10},PerformanceObserver:class {static supportedEntryTypes=[];},requestAnimationFrame:()=>0});
 document.dispatchEvent(new Event('astro:after-swap'));
 assert.equal(window.__docMetrics.events.length,1);
 assert.equal(window.__docMetrics.events[0].event,'astro:after-swap');
});

test('unsupported browser observers remain unavailable instead of becoming zero-cost results',async()=>{
 const {summarizeObservations}=await import('./doc_browser_accounting.mjs');
 const result=summarizeObservations({supported:[],frames:[]},16);
 assert.equal(result.cls,null);assert.equal(result.longTaskCount,null);assert.equal(result.longTaskTotalMs,null);
});

test('dedicated-worker requests retain a separate CDP namespace and actual runtime bytes',()=>{
 const ledger=new RequestLedger();
 const request=url=>({requestId:'same-id',timestamp:1,request:{url,headers:{}},type:'Script'});
 ledger.event('Network.requestWillBeSent',request('https://docs.test/page.js'));
 ledger.event('Network.requestWillBeSent',request('https://docs.test/runtime/engine.mjs?v=hash'),'worker-session');
 ledger.event('Network.loadingFinished',{requestId:'same-id',timestamp:2,encodedDataLength:30});
 ledger.event('Network.loadingFinished',{requestId:'same-id',timestamp:3,encodedDataLength:70},'worker-session');
 assert.equal(ledger.rows.length,2);
 assert.equal(ledger.rows[0].encodedTransferBytes,30);
 assert.equal(ledger.rows[1].encodedTransferBytes,70);
 assert.equal(ledger.rows[1].sessionId,'worker-session');
 assert.equal(ledger.rows[1].id,'same-id','raw request ID remains usable for that session getResponseBody');
 assert.equal(summarizeRequests(ledger.rows).categories.runtime.encodedTransferBytes,70);
});
