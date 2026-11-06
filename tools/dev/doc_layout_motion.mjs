/** Preserve strict layout checks while separating the declared live-main entrance from reflow.
 * Example: layoutBox(rawDocumentRect, mainFlow, validateMainMotion(facts)).
 */
export function validateMainMotion(facts){
 if(!facts.present)return 0;
 const fail=message=>{throw Error('main entrance: '+message);};
 const pure=matrix=>matrix?.is2D&&matrix.values.length===6&&matrix.values.every(Number.isFinite)&&matrix.values[0]===1&&matrix.values[1]===0&&matrix.values[2]===0&&matrix.values[3]===1&&matrix.values[4]===0;
 if(!pure(facts.matrix))fail('unexpected scale/rotation/X/perspective');
 const dy=facts.matrix.values[5];
 if(!facts.animations.length){
  if(dy!==0||facts.computed.name!=='none')fail('undeclared transform or animation');
  if(facts.enter&&!facts.reduced)fail('entrance class has no CSSAnimation');
  if(facts.pending&&!facts.reduced&&Number(facts.computed.opacity)!==0)fail('pending main was not hidden before setup');
  return 0;
 }
 if(facts.reduced||facts.pending||!facts.enter||facts.animations.length!==1)fail('unexpected entrance ownership');
 const a=facts.animations[0],t=a.timing;
 if(!a.css||!a.targetMain||a.name!=='page-in'||facts.computed.name!=='page-in'||facts.computed.duration!=='0.17s'||facts.computed.easing!=='ease-out')fail('wrong target/name/computed timing');
 if(!t||t.duration!==170||t.easing!=='linear'||t.delay!==0||t.endDelay!==0||t.iterations!==1||t.iterationStart!==0||t.direction!=='normal'||t.fill!=='both')fail('wrong effect timing');
 if(a.frames.length!==2||a.frameMatrices.length!==2)fail('wrong keyframe count');
 const allowed=new Set(['offset','computedOffset','easing','composite','opacity','transform']);
 for(let index=0;index<2;index++){
  const f=a.frames[index],m=a.frameMatrices[index];
  if(Object.keys(f).some(key=>!allowed.has(key))||f.offset!==index||f.computedOffset!==index||f.easing!=='ease-out'||!['auto','replace'].includes(f.composite)||Number(f.opacity)!==index||!pure(m)||m.values[5]!==[4,0][index])fail('unexpected keyframe properties');
 }
 if(dy<0||dy>4)fail('translateY outside declared0..4px');
 return dy;
}

/** Read the actual CSSAnimation and computed matrix; no author-name-only exemption. */
export function readMainMotion(main){
 if(!main)return{present:false};
 const matrix=value=>{const m=new DOMMatrixReadOnly(value==='none'?undefined:value);return{is2D:m.is2D,values:[m.a,m.b,m.c,m.d,m.e,m.f]};};
 const style=getComputedStyle(main);
 const facts={present:true,enter:main.classList.contains('doc-main-enter'),pending:main.classList.contains('doc-main-pending'),reduced:matchMedia('(prefers-reduced-motion: reduce)').matches,computed:{name:style.animationName,duration:style.animationDuration,easing:style.animationTimingFunction,opacity:style.opacity,transform:style.transform},matrix:matrix(style.transform),animations:main.getAnimations().map(a=>{const frames=a.effect?.getKeyframes()??[];return{css:a instanceof CSSAnimation,targetMain:a.effect?.target===main,name:a.animationName,timing:a.effect?.getTiming(),frames,frameMatrices:frames.map(f=>matrix(f.transform??'none')),currentTime:a.currentTime,playState:a.playState};})};
 return facts;
}

/** Fixed, sticky and absolute ancestry never borrows main-flow motion compensation. */
export function mainFlowEligible(inMain,positions){return inMain&&!positions.some(position=>['absolute','sticky','fixed'].includes(position));}

/** Correct only eligible main-flow boxes, before the existing tenth-pixel rounding. */
export function layoutBox(raw,mainFlow,dy){
 return raw.map((value,index)=>Math.round((value-(mainFlow&&index===1?dy:0))*10)/10);
}

/** A semantic offscreen heading remains accessible and exactly1×1 on every recorded frame. */
export function srHeading(facts,loading){
 const fail=()=>{throw Error('invalid semantic offscreen heading '+facts.id);};
 const [x,y,w,h]=facts.rect,[width,height]=facts.viewport;
 const roundedWidth=Math.round(w*10)/10,roundedHeight=Math.round(h*10)/10;
 if(!['H2','H3'].includes(facts.tag)||facts.position!=='absolute'||facts.visibility!=='visible'||!['','heading'].includes(facts.role)||facts.inert||facts.overflowX!=='hidden'||facts.overflowY!=='hidden'||facts.hidden||facts.ariaHidden||facts.usedWidth!=='1px'||facts.usedHeight!=='1px'||facts.transform!=='none'||!facts.rect.every(Number.isFinite)||roundedWidth!==1||roundedHeight!==1||!(x+w<=0||y+h<=0||x>=width||y>=height)||(!loading&&!facts.text.trim()))fail();
 return['sr-heading',facts.tag,facts.id,facts.text.trim(),roundedWidth,roundedHeight];
}

/** Same original0.6px tolerance and recursive list policy used by the frame oracle. */
export function layoutDiffers(a,b,strict=false){
 if(a===b)return false;
 if(typeof a==='number'&&typeof b==='number')return Math.abs(a-b)>0.6;
 if(Array.isArray(a)&&Array.isArray(b)){
  if(strict&&a.length!==b.length)return true;
  return a.some((v,i)=>i>=b.length||layoutDiffers(v,b[i],strict));
 }
 return true;
}
