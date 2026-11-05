/** Coloring uses the generated shared spec; edited source cannot reuse stale compiler links. */
import {visitColorTokens,type ColorRole} from '../generated/doc-coloring.ts';

export function recolor(code:HTMLElement,source:string):void{
 const document=code.ownerDocument,fragment=document.createDocumentFragment();
 let start=0,end=0,role:ColorRole|undefined;
 const append=():void=>{
  if(start===end)return;
  const text=source.slice(start,end);
  if(role){const span=document.createElement('span');span.className=role;span.textContent=text;fragment.append(span);}
  else fragment.append(document.createTextNode(text));
 };
 visitColorTokens(source,token=>{if(token.role===role&&token.start===end)end=token.end;else{append();start=token.start;end=token.end;role=token.role;}});
 append();code.replaceChildren(fragment);
}
