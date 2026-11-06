/** Prepare one trusted pointer gesture without bypassing native scrolling or animation.
 * Example: await preparePointer(evaluate, 'main a.name'); then dispatch CDP mouse input.
 */
export async function preparePointer(evaluate,selector,{timeoutMs=20_000}={}){
 return evaluate(`(${pointerProbe.toString()})(${JSON.stringify(selector)},${JSON.stringify(timeoutMs)})`);
}

/** Browser-side probe: request one normal scroll, then require an exposed, stationary target. */
export async function pointerProbe(selector,timeoutMs){
 const node=document.querySelector(selector);
 if(!node)throw new Error(`missing action target ${selector}`);
 node.scrollIntoView({block:'center'});
 const deadline=performance.now()+timeoutMs;
 let previous,stable=0,frames=0,last;
 while(performance.now()<deadline){
  const rect=node.getBoundingClientRect(),x=rect.x+rect.width/2,y=rect.y+rect.height/2;
  const hit=document.elementFromPoint(x,y);
  const visible=node.isConnected&&rect.width>0&&rect.height>0&&x>=0&&y>=0&&x<innerWidth&&y<innerHeight;
  const exposed=visible&&!!hit&&(hit===node||node.contains(hit));
  const geometry=[rect.x,rect.y,rect.width,rect.height];
  stable=exposed&&previous&&geometry.every((value,index)=>value===previous[index])?stable+1:0;
  last={x,y,width:rect.width,height:rect.height,exposed,frames};
  if(stable>=2)return{x,y,frames};
  previous=exposed?geometry:undefined;
  await new Promise(resolve=>requestAnimationFrame(resolve));frames++;
 }
 throw new Error(`pointer-ready deadline: ${selector} ${JSON.stringify(last)}`);
}
