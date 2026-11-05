/** The compiler loads only after Run; terminating this worker enforces Stop and time budgets. */
import { playResult } from '../lib/example-program.ts';
interface Engine {play(files:Record<string,string>,main:string,options:{budget:number}):unknown}
const worker=self;
let engine:Engine|undefined;
function isEngine(value:unknown):value is Engine {return !!value&&typeof value==='object'&&'play' in value&&typeof value.play==='function';}
worker.addEventListener('message',async(event:MessageEvent<unknown>)=>{
  try {
    const data=event.data;
    if(!data||typeof data!=='object'||!('program' in data)||typeof data.program!=='string'||!('engine' in data)||typeof data.engine!=='string'||!('wasm' in data)||typeof data.wasm!=='string')throw new Error('Invalid example request');
    for(const path of [data.engine,data.wasm])if(new URL(path).origin!==worker.location.origin)throw new Error('Runtime assets must use the worker origin');
    if(!engine) {
      const factory:unknown=await import(/* @vite-ignore */ data.engine);
      if(!factory||typeof factory!=='object'||!('createEngine' in factory)||typeof factory.createEngine!=='function')throw new Error('Invalid browser compiler bundle');
      const response=await fetch(data.wasm);if(!response.ok)throw new Error(`Browser compiler Wasm is unavailable (${response.status})`);
      const loaded:unknown=await factory.createEngine(new Uint8Array(await response.arrayBuffer()));
      if(!isEngine(loaded))throw new Error('Browser compiler did not create an engine');engine=loaded;
    }
    const started=performance.now();
    const result=playResult(engine.play({'main.jai':data.program},'main.jai',{budget:200_000_000}));
    worker.postMessage({result,ms:performance.now()-started});
  } catch(error) {
    const crashed=!!error&&typeof error==='object'&&'compilerCrashed' in error&&error.compilerCrashed===true;
    if(crashed)engine=undefined;
    worker.postMessage({error:error instanceof Error?error.message:String(error),crashed});
  }
});
