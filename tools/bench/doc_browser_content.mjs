/** Observe native transition state without handling application promises or changing routing. */
export function nativeTransitionState(){
 if('activeViewTransition' in document)return{supported:true,active:document.activeViewTransition!==null};
 if(globalThis.CSS?.supports('selector(:active-view-transition)'))return{supported:true,active:document.documentElement.matches(':active-view-transition')};
 if(typeof document.startViewTransition==='function')throw new Error('browser cannot expose reliable native transition activity');
 return{supported:false,active:false};
}

/** Browser-side content observation; serialized into the same document as the production UI. */
export function contentReadiness(expected){
 const node=document.querySelector(expected.selector),time=performance.now();
 const base={time,origin:performance.timeOrigin,path:location.pathname,hash:location.hash,scrollY,active:{tag:document.activeElement?.tagName,id:document.activeElement?.id},domNodes:document.getElementsByTagName('*').length};
 if(location.pathname!==expected.path||location.hash!==expected.hash||!node||expected.text&&!node.textContent.includes(expected.text))return{...base,domReady:false,ready:false};
 const r=node.getBoundingClientRect(),domReady=node.checkVisibility({checkVisibilityCSS:true})&&r.width>0&&r.height>0&&r.bottom>0&&r.top<innerHeight&&r.right>0&&r.left<innerWidth;
 const native=nativeTransitionState(),blockers=[];
 if(!domReady)blockers.push('destination');
 if(native.active)blockers.push('native-transition');
 if(!performance.getEntriesByType('paint').some(entry=>entry.name==='first-contentful-paint'))blockers.push('first-contentful-paint');
 for(let ancestor=node;ancestor;ancestor=ancestor.parentElement){
  const style=getComputedStyle(ancestor);
  if(Number(style.opacity)<1||style.visibility==='hidden')blockers.push('opacity-or-visibility');
  if(ancestor.getAnimations().some(animation=>animation.playState==='running'&&animation.effect&&Number.isFinite(animation.effect.getComputedTiming().endTime)))blockers.push('ancestor-animation');
 }
 if(document.fonts&&document.fonts.status!=='loaded')blockers.push('fonts');
 const x=(Math.max(0,r.left)+Math.min(innerWidth,r.right))/2,y=(Math.max(0,r.top)+Math.min(innerHeight,r.bottom))/2;
 if(!node.contains(document.elementFromPoint(x,y)))blockers.push('hit-target');
 return{...base,domReady,ready:blockers.length===0,blockers,nativeTransition:native};
}

/** The probe carries its readonly native-state dependency into the browser realm. */
export const contentProbeSource=`(()=>{const nativeTransitionState=${nativeTransitionState.toString()};return ${contentReadiness.toString()};})()`;
