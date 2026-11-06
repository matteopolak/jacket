/** Presentation roles for the toolchain's verbatim diagnostic output, independent of Jai syntax. */
export type DiagnosticRole='error'|'warning'|'frame'|'secondary'|'help'|'replacement'|'addition'|'deletion';
export interface DiagnosticPart {text:string;role:DiagnosticRole|null}

/** Classify gutters by operation, so a warning's deletion remains red and an origin stays distinct. */
export function diagnosticParts(source:string):DiagnosticPart[] {
  const parts:DiagnosticPart[]=[];
  let severity:DiagnosticRole='error';
  const add=(text:string,role:DiagnosticRole|null=null):void=>{
    if(!text)return;
    const previous=parts.at(-1);
    if(previous&&previous.role===role)previous.text+=text;else parts.push({text,role});
  };
  const lines=source.split('\n');
  lines.forEach((line,index)=>{
    const trimmed=line.trimStart(),indent=line.slice(0,line.length-trimmed.length);
    const kind=(['error','warning'] as const).find(kind=>trimmed.startsWith(`${kind}[`)||trimmed.startsWith(`${kind}:`));
    if(kind){
      severity=kind;const colon=trimmed.indexOf(':');
      add(indent);add(colon<0?trimmed:trimmed.slice(0,colon),severity);if(colon>=0)add(trimmed.slice(colon));
    } else if(trimmed.startsWith('--> ')){
      add(indent);add('-->','frame');add(trimmed.slice(3));
    } else {
      const label=['= note:','= help:','help:','note:'].find(label=>trimmed.startsWith(label));
      if(label){add(indent);add(label,'help');add(trimmed.slice(label.length));}
      else {
        // The renderer writes exactly one separator before its marker cell. Consuming
        // arbitrary gaps would mistake a neutral context line's unary operator for it.
        const gutter=/^([ \t]*)(\d+|\.{3})? ([|~+-])(?= |$)(.*)$/u.exec(line);
        if(!gutter||(!gutter[2]&&gutter[3]!=='|'))add(line);
        else {
          const [,leading='',number='',sign='',rest='']=gutter;
          add(leading+number+' ');
          const role:DiagnosticRole=sign==='|'?'frame':sign==='~'?'replacement':sign==='+'?'addition':'deletion';
          add(sign,role);
          const markers=number===''&&sign==='|'?/^(\s*)([\^-]+)(.*)$/u.exec(rest):null;
          if(markers){add(markers[1]??'');add((markers[2]??'')+(markers[3]??''),markers[2]?.startsWith('^')?severity:'secondary');}
          else add(rest);
        }
      }
    }
    if(index<lines.length-1)add('\n');
  });
  return parts;
}
