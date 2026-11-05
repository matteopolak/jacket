import {recolor} from './coloring.ts';
/** Examples are isolated user actions with editable source, bounded workers and explicit cleanup. */
import { pageScope } from './lifetime.ts';
import { fetchPayload } from '../lib/client-data.ts';
import { base64url, fill, playResult } from '../lib/example-program.ts';
import type { Example } from '../generated/doc-site.js';

const scope=pageScope();
export interface RuntimeAssets {engine:string;wasm:string}
export function initExamples(base:string,parent:AbortSignal,runtime?:RuntimeAssets):void {
  const signal=scope(parent);if(signal.aborted)return;
  const timers=new Set<ReturnType<typeof setTimeout>>();
  const areas=new Set<HTMLTextAreaElement>();
  const payloads=new Map<string,Promise<Example>>();
  const load=(id:string):Promise<Example>=>{
    let pending=payloads.get(id);if(!pending){pending=fetchPayload(`${base}examples/${id}.json`,'example',signal).then(payload=>{if(payload.payload!=='example')throw new Error('expected example payload');return payload.data;});payloads.set(id,pending);}return pending;
  };
  const editors=new WeakMap<HTMLElement,HTMLTextAreaElement>();
  const originals=new WeakMap<HTMLElement,string>();
  let worker:Worker|undefined;
  let active:{token:number;button:HTMLButtonElement;output:HTMLElement;timer?:ReturnType<typeof setTimeout>}|undefined;
  let nextToken=0;
  function report(output:HTMLElement,state:string,status:string,text=''):void {
    if(signal.aborted||!output.isConnected)return;
    output.hidden=false;output.className=`ex-out${state?` ${state}`:''}`;
    const header=document.createElement('div');header.textContent=status;output.replaceChildren(header);
    if(text){const pre=document.createElement('pre');pre.textContent=text;output.append(pre);}
  }
  function finish(token:number):void {if(active?.token!==token)return;if(active.timer)clearTimeout(active.timer);active.button.textContent='Run ▸';active.button.dataset['act']='run';active=undefined;}
  function stop(message='Stopped.'):void {
    worker?.terminate();worker=undefined;
    if(active){const {token,output}=active;report(output,'',message);finish(token);}
  }
  function code(figure:HTMLElement):string {return editors.get(figure)?.value??figure.querySelector('pre.code')?.textContent??'';}
  async function run(button:HTMLButtonElement):Promise<void> {
    const figure=button.closest<HTMLElement>('figure'),id=button.dataset['program'],output=figure?.querySelector<HTMLElement>('.ex-out');
    if(!figure||!id||!output)return;
    if(active)stop('Stopped because another example was started.');
    const token=++nextToken;active={token,button,output};button.dataset['act']='stop';button.textContent='Stop ■';
    report(output,'busy',worker?'Compiling...':'Loading the compiler (the first run takes a moment)...');
    try {
      const example=await load(id);if(signal.aborted||active?.token!==token)return;
      if(!example.runnable||!example.runTemplate)throw new Error(example.reason??'This example does not support the browser target.');
      if(!runtime)throw new Error('This page has no admitted browser runtime assets.');
      worker??=new Worker(new URL('./run-worker.ts',import.meta.url),{type:'module'});
      const current=worker;
      const maximum=Math.min(example.timeoutMs,300_000);
      active.timer=setTimeout(()=>{if(active?.token===token)stop(`Stopped after ${maximum/1000} seconds.`);},maximum);
      current.onmessage=(event:MessageEvent<unknown>)=>{
        if(signal.aborted||active?.token!==token)return;
        try {
          const data=event.data;if(!data||typeof data!=='object')throw new Error('Invalid example worker response');
          if('error' in data){if('crashed' in data&&data.crashed===true){current.terminate();worker=undefined;}throw new Error(String(data.error));}
          if(!('result' in data)||!('ms' in data)||typeof data.ms!=='number')throw new Error('Invalid example worker result');
          const result=playResult(data.result),printed=result.stdout+result.stderr;
          if(result.exitCode===0&&!result.diagnostics.some(d=>d.severity==='error'))report(output,'',`✓ Ran in ${Math.max(1,Math.round(data.ms))} ms`,printed.trim()?printed:'');
          else report(output,'bad','✗ Failed',(result.rendered||printed||(result.exitCode!==null?`The example exited with status ${result.exitCode}.`:'The example did not compile.')).trim());
        }catch(error){report(output,'bad','✗ Could not run',error instanceof Error?error.message:String(error));}
        finish(token);
      };
      current.onerror=event=>{if(signal.aborted||active?.token!==token)return;current.terminate();worker=undefined;report(output,'bad','✗ Could not run',event.message||'The compiler could not start in this browser.');finish(token);};
      current.postMessage({program:fill(example.runTemplate,code(figure)),engine:new URL(runtime.engine,location.origin).href,wasm:new URL(runtime.wasm,location.origin).href});
    }catch(error){if(!signal.aborted&&active?.token===token){report(output,'bad','✗ Could not run',error instanceof Error?error.message:String(error));finish(token);}}
  }
  for(const figure of document.querySelectorAll<HTMLElement>('figure[data-example]')) {
    const pre=figure.querySelector<HTMLElement>('.ed pre.code'),runButton=figure.querySelector<HTMLButtonElement>('button[data-act="run"]');
    if(!pre||!runButton)continue;
    const source=pre.textContent??'',original=pre.querySelector('code')?.cloneNode(true),reset=figure.querySelector<HTMLButtonElement>('button[data-act="reset"]'),open=figure.querySelector<HTMLAnchorElement>('a.btn[href]'),openHref=open?.href;
    const area=document.createElement('textarea');area.value=source;area.spellcheck=false;area.wrap='off';area.autocapitalize='off';area.autocomplete='off';area.setAttribute('aria-label','Example code, editable');pre.parentElement?.append(area);areas.add(area);editors.set(figure,area);originals.set(figure,source);
    let escaped=false;
    function changed():void {
      const edited=area.value!==source,code=pre?.querySelector('code');
      if(code){if(!edited&&original)code.replaceWith(original.cloneNode(true));else recolor(code,area.value+(/\n$/u.test(area.value)?' ':''));}
      if(reset)reset.hidden=!edited;
      if(open&&openHref){if(!edited)open.href=openHref;else {const text=area.value;void load(runButton?.dataset['program']??'').then(example=>{if(!signal.aborted&&area.isConnected&&example.openTemplate&&area.value===text){const url=new URL(openHref);url.hash=`code=${base64url(fill(example.openTemplate,text))}`;open.href=url.href;}}).catch(()=>{});}}
    }
    area.addEventListener('input',changed,{signal});
    area.addEventListener('scroll',()=>{const scale=Number.parseFloat(getComputedStyle(area.parentElement??area).getPropertyValue('--k'))||1;pre.scrollLeft=area.scrollLeft*scale;pre.scrollTop=area.scrollTop*scale;},{signal});
    area.addEventListener('keydown',event=>{
      const plain=!event.ctrlKey&&!event.metaKey&&!event.altKey&&!event.shiftKey;
      if(event.key==='Tab'&&plain&&!escaped){event.preventDefault();area.setRangeText('    ',area.selectionStart,area.selectionEnd,'end');changed();}
      else if(event.key==='Enter'&&plain){const before=area.value.slice(0,area.selectionStart),indent=/^[ \t]*/u.exec(before.slice(before.lastIndexOf('\n')+1))?.[0]??'';event.preventDefault();area.setRangeText(`\n${indent}`,area.selectionStart,area.selectionEnd,'end');changed();}
      else if(event.key==='Enter'&&(event.ctrlKey||event.metaKey)){event.preventDefault();void run(runButton);}
      escaped=event.key==='Escape';
    },{signal});
    area.addEventListener('blur',()=>{escaped=false;},{signal});reset?.addEventListener('click',()=>{area.value=source;changed();area.focus();},{signal});
  }
  document.addEventListener('click',event=>{
    if(!(event.target instanceof Element))return;
    const button=event.target.closest<HTMLButtonElement>('button[data-act]');if(!button||button.disabled)return;
    const action=button.dataset['act'];
    if(action==='stop'){stop();return;}if(action==='run'){void run(button);return;}
    if(action==='copy'){
      const figure=button.closest<HTMLElement>('figure');if(!figure)return;
      const copied=():void=>{if(signal.aborted||!button.isConnected)return;const text=button.textContent;button.textContent='Copied';const timer=setTimeout(()=>{timers.delete(timer);if(!signal.aborted&&button.isConnected)button.textContent=text;},1500);timers.add(timer);};
      if(navigator.clipboard?.writeText)void navigator.clipboard.writeText(code(figure)).then(copied).catch(()=>{});
      else {const area=document.createElement('textarea');area.value=code(figure);document.body.append(area);area.select();try{if(document.execCommand('copy'))copied();}finally{area.remove();button.focus();}}
    }
  },{signal});
  window.addEventListener('pagehide',()=>stop('Stopped on navigation.'),{signal});
  signal.addEventListener('abort',()=>{stop('Stopped on navigation.');for(const timer of timers)clearTimeout(timer);timers.clear();for(const area of areas)area.remove();areas.clear();payloads.clear();},{once:true});
}
