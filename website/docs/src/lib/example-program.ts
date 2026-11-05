import type { ProgramTemplate } from '../generated/doc-site.js';

/** Fill the producer's wrapper exactly as Jacket's doctest Template::fill does. */
export function fill(template:ProgramTemplate,code:string):string {
  const lines=code.replace(/\r/gu,'').split('\n');if(lines.at(-1)==='')lines.pop();
  let output=template.head;
  if(template.wrapped) {
    const body:string[]=[];
    for(const line of lines)if(/^\s*#(?:import|load)/u.test(line))output+=`${line.trimStart()}\n`;else body.push(line);
    output+=template.middle;
    for(const line of body)output+=`${line!==''||template.pad?'    ':''}${line}\n`;
  } else {for(const line of lines)output+=`${line}\n`;output+=template.middle;}
  return output+template.tail;
}

/** UTF-8 encoding preserves non-ASCII examples in playground links. */
export function base64url(text:string):string {
  const bytes=new TextEncoder().encode(text);let binary='';
  for(let i=0;i<bytes.length;i+=0x8000)binary+=String.fromCharCode(...bytes.subarray(i,i+0x8000));
  return btoa(binary).replaceAll('+','-').replaceAll('/','_').replace(/=+$/u,'');
}

export interface PlayResult {exitCode:number|null;stdout:string;stderr:string;rendered:string;diagnostics:{severity:string}[]}
export interface RunResponse {result:PlayResult;ms:number}
export interface RunRequest {program:string;engine:string;wasm:string}

export function playResult(value:unknown):PlayResult {
  if(!value||typeof value!=='object'||!('exitCode' in value)||!(value.exitCode===null||typeof value.exitCode==='number')||!('stdout' in value)||typeof value.stdout!=='string'||!('stderr' in value)||typeof value.stderr!=='string'||!('rendered' in value)||typeof value.rendered!=='string'||!('diagnostics' in value)||!Array.isArray(value.diagnostics))throw new Error('Invalid browser compiler response');
  const diagnostics=value.diagnostics.map((diagnostic:unknown)=>{if(!diagnostic||typeof diagnostic!=='object'||!('severity' in diagnostic)||typeof diagnostic.severity!=='string')throw new Error('Invalid compiler diagnostic');return {severity:diagnostic.severity};});
  return {exitCode:value.exitCode,stdout:value.stdout,stderr:value.stderr,rendered:value.rendered,diagnostics};
}
