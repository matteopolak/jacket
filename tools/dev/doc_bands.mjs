/** Executed in the real page after settling; checks presentation without comparing compact source heights. */
export function declarationBands(){
 const problems=[],bands=[...document.querySelectorAll('.row > .members > pre.sig,.row > .members > .semantic-declaration')];
 for(const band of bands){
  const members=band.parentElement,row=members.parentElement,css=getComputedStyle(band),px=getComputedStyle(row).getPropertyValue('--px').trim();
  const expected={marginTop:'0px',marginRight:'0px',marginBottom:'0px',marginLeft:'0px',paddingTop:'12px',paddingBottom:'12px',paddingLeft:px,paddingRight:px,backgroundColor:'rgba(0, 0, 0, 0)',backgroundImage:'none',borderTopWidth:'1px',borderRightWidth:'0px',borderBottomWidth:members.matches(':last-child')?'0px':'1px',borderLeftWidth:'0px',borderTopStyle:'solid',borderBottomStyle:'solid',borderTopLeftRadius:'0px',borderTopRightRadius:'0px',borderBottomLeftRadius:'0px',borderBottomRightRadius:'0px'};
  for(const [property,value]of Object.entries(expected))if(css[property]!==value)problems.push(`${row.id}: declaration band ${property}=${css[property]}, expected ${value}`);
  const box=band.getBoundingClientRect(),container=members.getBoundingClientRect(),previous=band.previousElementSibling,expectedTop=previous?.matches('pre.sig,.semantic-declaration')?previous.getBoundingClientRect().bottom:container.top;
  if(Math.abs(box.left-container.left)>.1||Math.abs(box.right-container.right)>.1||Math.abs(box.top-expectedTop)>.1)problems.push(`${row.id}: declaration band does not fill its member container or contiguous variant stack`);
 }
 return{count:bands.length,problems};
}
