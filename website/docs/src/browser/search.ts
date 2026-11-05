/** Name and signature search use compiler facts; presentation never reconstructs item slugs. */
import type { SearchEntry, Target } from '../generated/doc-site.js';
import { fetchPayload } from '../lib/client-data.ts';

const normal=(text:string):string=>text.replace(/^[a-z_][a-z0-9_]*\s*:\s*/iu,'').replace(/\s+/gu,'').toLowerCase();
/** Split only outer signature commas; nested procedure/generic/array types remain whole. */
const types=(text:string):string[]=>{
  let source=text.trim();if(source.startsWith('(')&&source.endsWith(')'))source=source.slice(1,-1);
  const parts:string[]=[];let start=0,depth=0;
  for(let i=0;i<source.length;i++){const c=source[i];if(c==='('||c==='['||c==='{')depth++;else if(c===')'||c===']'||c==='}')depth--;else if(c===','&&depth===0){parts.push(source.slice(start,i));start=i+1;}}
  parts.push(source.slice(start));return parts.map(normal).filter(Boolean);
};
function contains(have:string[],wanted:string[]):boolean {
  const rest=[...have];
  return wanted.every(value=>{const index=rest.indexOf(value);if(index<0)return false;rest.splice(index,1);return true;});
}
const compare=(a:string,b:string):number=>{const left=a.toLowerCase(),right=b.toLowerCase();return left<right?-1:left>right?1:0;};
/** Exact matches, prefix/substring and qualified names preserve native-binding demotion. */
export function search(entries:SearchEntry[],query:string):SearchEntry[] {
  if(query.includes('->')) {
    const cut=query.indexOf('->'),parameters=types(query.slice(0,cut)),results=types(query.slice(cut+2));
    return entries.flatMap(entry=>{
      if(entry.binding)return [];
      const scores=entry.overloads.filter(overload=>contains(overload.parameters.map(normal),parameters)&&contains(overload.results.map(normal),results)).map(overload=>Number(overload.parameters.length===parameters.length)+Number(overload.results.length===results.length));
      return scores.length?[{entry,score:Math.max(...scores)}]:[];
    }).sort((a,b)=>b.score-a.score||a.entry.name.length-b.entry.name.length||compare(a.entry.qualifiedName,b.entry.qualifiedName)).map(hit=>hit.entry);
  }
  const terms=query.toLowerCase().split(/\s+/u).filter(Boolean);
  return entries.flatMap(entry=>{
    const name=entry.kind==='module'?'':entry.name.toLowerCase(),path=entry.qualifiedName.toLowerCase(),summary=entry.summary.toLowerCase();
    let score=0;
    for(const term of terms) {
      const part=name===term?1000:name.startsWith(term)?800:name.includes(term)?600:path.includes(term)?400:summary.includes(term)?100:0;
      if(!part)return [];
      score+=part;
    }
    if(entry.kind==='module')score+=50;
    if(entry.binding)score=Math.floor(score/4);
    return [{entry,score}];
  }).sort((a,b)=>b.score-a.score||a.entry.name.length-b.entry.name.length||compare(a.entry.qualifiedName,b.entry.qualifiedName)).map(hit=>hit.entry);
}

/** Search rows display declaration facts, with the same compact signature policy as before. */
export function searchPreview(entry:SearchEntry):{text:string;className:string}[] {
  const result:{text:string;className:string}[]=[];
  const add=(text:string,className=''):void=>{if(text)result.push({text,className});};
  const preview=entry.preview;
  if(!preview){add(entry.summary);return result;}
  if(preview.kind==='value'){add(`:: ${preview.value}`);return result;}
  if(preview.kind==='declaration_kind'){add(`:: ${preview.declaration_kind}`);return result;}
  const parameter=(value:typeof preview.parameters[number]):void=>{
    add(value.name);
    if(value.ty){if(value.name)add(': ');add(value.ty,entry.binding?'':'ty');if(value.default)add(` = ${value.default}`);}
    else if(value.default)add(` := ${value.default}`);
  };
  const list=(values:typeof preview.parameters):void=>{values.forEach((value,index)=>{if(index)add(', ');parameter(value);});};
  add('(');list(preview.parameters);add(')');
  if(preview.results.length){add(' -> ');if(preview.results.length>1)add('(');list(preview.results);if(preview.results.length>1)add(')');}
  return result;
}

export function initSearch(source:string,href:(target:Target)=>string,signal:AbortSignal,navigate:(url:string)=>void):void {
  const foundInput=document.querySelector<HTMLInputElement>('#search-q'),foundBox=document.querySelector<HTMLElement>('#search-results');
  if(!foundInput||!foundBox)return;
  const input:HTMLInputElement=foundInput,box:HTMLElement=foundBox;
  let entries:Promise<SearchEntry[]>|undefined,hits:SearchEntry[]=[],picked=-1,epoch=0,wanted=false;
  const load=():Promise<SearchEntry[]>=>entries??=fetchPayload(source,'search',signal).then(payload=>{
    if(payload.payload!=='search')throw new Error('expected search payload');return payload.data.entries;
  });
  const close=():void=>{box.hidden=true;input.setAttribute('aria-expanded','false');input.removeAttribute('aria-activedescendant');};
  const hide=():void=>{epoch++;wanted=false;hits=[];picked=-1;close();};
  // The header survives route swaps; release page-owned results without moving its focus.
  const dispose=():void=>{hits=[];picked=-1;hide();box.replaceChildren();};
  if(signal.aborted){dispose();return;}
  signal.addEventListener('abort',dispose,{once:true});
  const element=(tag:string,className:string,text:string):HTMLElement=>{const item=document.createElement(tag);item.className=className;item.textContent=text;return item;};
  function show():void {
    box.replaceChildren();
    if(hits.length===0){const empty=element('div','empty','No matches.');empty.setAttribute('role','option');empty.setAttribute('aria-disabled','true');box.append(empty);}
    hits.forEach((entry,index)=>{
      const row=document.createElement('a');row.className='hit';row.href=href(entry.target);row.tabIndex=-1;row.dataset['astroPrefetch']='hover';row.id=`hit-${index}`;row.setAttribute('role','option');row.setAttribute('aria-selected',String(index===picked));row.setAttribute('aria-label',`${entry.qualifiedName}: ${entry.summary}`);
      const code=element('code','','');code.append(element('span','path',entry.module?`${entry.module.split('/').at(-1)}${entry.kind==='module'?'':`.${entry.name}`}`:entry.name));row.append(code);
      if(entry.binding){const dot=element('span','dot foreign','');dot.title='Foreign: a binding to a native library';dot.setAttribute('role','img');dot.setAttribute('aria-label',dot.title);dot.append(element('span','sr',dot.title));row.append(dot);}
      if(entry.category){const dot=element('span',`dot ${entry.category.replace('category-','cat-')}`,'');dot.setAttribute('role','img');dot.setAttribute('aria-label','API category');row.append(dot);}
      if(entry.experimental){const dot=element('span','dot exp','');dot.setAttribute('role','img');dot.setAttribute('aria-label','Experimental: may change or go away');row.append(dot);}
      if(entry.kind!=='declaration')row.append(element('span','what',entry.kind));
      const signature=element('span','sig','');for(const part of searchPreview(entry))signature.append(element('span',part.className,part.text));row.append(signature);box.append(row);
    });
    const tips=element('div','tips','');for(const tip of ['↑↓ to move','↵ to open','type -> bool to search by return type'])tips.append(element('span','',tip));tips.setAttribute('role','option');tips.setAttribute('aria-disabled','true');box.append(tips);
    box.hidden=false;input.setAttribute('aria-expanded','true');
    if(picked>=0)input.setAttribute('aria-activedescendant',`hit-${picked}`);else input.removeAttribute('aria-activedescendant');
  }
  async function update(last=false):Promise<void> {
    const query=input.value.trim();if(!query){hide();return;}
    const intent=++epoch;wanted=true;hits=[];picked=-1;close();
    const current=():boolean=>!signal.aborted&&wanted&&epoch===intent&&input.value.trim()===query;
    try{const all=await load();if(!current())return;hits=search(all,query).slice(0,12);picked=hits.length?(last?hits.length-1:0):-1;show();}
    catch{if(!current())return;box.textContent='Search could not load. Reload the page to try again.';box.hidden=false;input.setAttribute('aria-expanded','true');}
  }
  input.addEventListener('focus',()=>{void load().catch(()=>{});},{signal});input.addEventListener('input',()=>{void update();},{signal});
  input.addEventListener('keydown',event=>{
    if(event.key==='ArrowDown'||event.key==='ArrowUp'){if(box.hidden){if(input.value.trim()){event.preventDefault();void update(event.key==='ArrowUp');}return;}if(!hits.length)return;event.preventDefault();picked=(picked+(event.key==='ArrowDown'?1:hits.length-1))%hits.length;show();}
    else if(event.key==='Enter'){const hit=wanted&&!box.hidden?hits[picked]:undefined;if(hit){event.preventDefault();navigate(href(hit.target));}}
    else if(event.key==='Escape'){event.preventDefault();hide();}
  },{signal});
  const leave=(event:FocusEvent):void=>{const next=event.relatedTarget;if(next instanceof Node&&(next===input||box.contains(next)))return;hide();};
  input.addEventListener('focusout',leave,{signal});box.addEventListener('focusout',leave,{signal});
  document.addEventListener('click',event=>{if(event.target instanceof Node&&!box.contains(event.target)&&event.target!==input)hide();},{signal});
  document.addEventListener('keydown',event=>{if(event.key!=='/'||event.ctrlKey||event.metaKey||event.altKey)return;const target=event.target;if(target instanceof HTMLElement&&(target.matches('input,textarea,select')||target.isContentEditable))return;event.preventDefault();input.focus();input.select();},{signal});
}
