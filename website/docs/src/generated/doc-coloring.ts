// Spec SHA-256: e08c56930a79357e37cee027d05a4c33e4919a0e32ab8521fd3589c322ae8392
// Generated from crates/jacket/src/doc/coloring/spec.json; do not edit.
export type ColorRole = 'k' | 't' | 's' | 'n' | 'c' | 'd';
interface Rule {kind:string;role?:ColorRole;open?:string;close?:string;escape?:string;horizontal?:string;lineBounded?:boolean;paint?:string;semantic?:boolean;hereName?:string;hereRole?:ColorRole;radixPrefixes?:string[]}
interface Spec {schema:string;identifier:{nonAscii:boolean;excluded:string[]};keywords:string[];types:string[];rules:Rule[]}
const SPEC:Spec = {"schema":"jai-doc-coloring/1","identifier":{"nonAscii":true,"excluded":["\u00a0","\ufeff"]},"keywords":["if","else","then","for","while","return","defer","using","struct","enum","enum_flags","union","cast","xx","inline","break","continue","case","remove","true","false","null","ifx","push_context","context","operator","interface"],"types":["int","float","float32","float64","bool","string","void","Type","Any","s8","s16","s32","s64","u8","u16","u32","u64","u128","s128","assert"],"rules":[{"kind":"line_comment","open":"//","role":"c"},{"kind":"nested_comment","open":"/*","close":"*/","role":"c"},{"kind":"quoted","open":"\"","escape":"\\","role":"s"},{"kind":"note","open":"@","lineBounded":true,"paint":"lexical","semantic":false},{"kind":"directive","open":"#","horizontal":" \t","hereName":"string","hereRole":"s","role":"d"},{"kind":"range","open":".."},{"kind":"number","role":"n","radixPrefixes":["0x","0b","0h"]},{"kind":"identifier"},{"kind":"plain"}]};
export interface ColorToken {start:number;end:number;role:ColorRole|undefined;identifier:boolean}
const ident=(source:string,at:number,first:boolean):boolean=>{const c=source.charCodeAt(at);return c>=65&&c<=90||c>=97&&c<=122||c===95||!first&&c>=48&&c<=57||c>=128&&SPEC.identifier.nonAscii&&!SPEC.identifier.excluded.some(word=>source.startsWith(word,at));};
const digit=(source:string,at:number):boolean=>{const c=source.charCodeAt(at);return c>=48&&c<=57;};
const alnum=(source:string,at:number):boolean=>{const c=source.charCodeAt(at);return c>=65&&c<=90||c>=97&&c<=122||c>=48&&c<=57;};
function word(source:string,at:number):number{while(ident(source,at,false))at++;return at;}
function blanks(source:string,at:number,characters:string):number{while(at<source.length&&characters.includes(source[at]!))at++;return at;}
function quoted(source:string,start:number,quote:string,escape:string):number{let at=start+1;while(at<source.length){const c=source[at++];if(c===quote)break;if(c===escape&&at<source.length)at++;}return at;}
function note(source:string,start:number,lineBounded:boolean):number{let at=start+1;if(source[at]==='"')return quoted(source,at,'"','\\');let depth=0;while(at<source.length){const c=source[at]!;if(lineBounded&&c==='\n'||depth===0&&' \t\r\n\v\f'.includes(c))break;if(c==='"'){at=quoted(source,at,'"','\\');continue;}if(c==='(')depth++;else if(c===')'&&depth>0)depth--;at++;}return at;}
function here(source:string,afterName:number,horizontal:string):number|undefined{
 let at=afterName;
 for(;;){at=blanks(source,at,horizontal);if(source[at]!==',')break;at=blanks(source,at+1,horizontal);if(source[at]==='\\'){at++;if(at<source.length)at+=(source.codePointAt(at)??0)>0xffff?2:1;}else{const end=word(source,at);if(end===at)return;at=end;}}
 const end=word(source,at);if(end===at)return;const tag=source.slice(at,end);at=end;
 while(at<source.length&&source[at]!=='\n')at++;at=Math.min(at+1,source.length);
 while(at<source.length){const start=blanks(source,at,horizontal);if(source.startsWith(tag,start)&&!ident(source,start+tag.length,false))return start+tag.length;while(at<source.length&&source[at]!=='\n')at++;at=Math.min(at+1,source.length);}
 return source.length;
}
/** Presentation only: UTF-16 slice offsets preserve the exact browser source, including partial edits.
 * Fixed ordered operations consume scanned input; nesting is counter-based, with no regex backtracking.
 */
export function visitColorTokens(source:string,visit:(token:ColorToken)=>void):void{visitRegion(source,visit,false);}
function visitRegion(source:string,visit:(token:ColorToken)=>void,insideNote:boolean):void{
 let at=0;
 while(at<source.length){const start=at;let matched=false;
  for(const rule of SPEC.rules){const open=rule.open??'',prefix=source.startsWith(open,at);let end=at,role=rule.role,identifier=false,notePaint=false;
   switch(rule.kind){
    case 'line_comment':if(prefix){end=at+open.length;while(end<source.length&&source[end]!=='\n')end++;}break;
    case 'nested_comment':if(prefix){const close=rule.close!;end=at+open.length;let depth=1;while(end<source.length&&depth>0){if(source.startsWith(open,end)){depth++;end+=open.length;}else if(source.startsWith(close,end)){depth--;end+=close.length;}else end++;}}break;
    case 'quoted':if(prefix)end=quoted(source,at,open,rule.escape!);break;
    case 'note':if(prefix&&!insideNote){end=note(source,at,rule.lineBounded!);notePaint=rule.paint==='lexical';}break;
    case 'directive':if(prefix){const horizontal=rule.horizontal!,name=blanks(source,at+open.length,horizontal);if(ident(source,name,true)){end=word(source,name);if(source.slice(name,end)===rule.hereName){const last=here(source,end,horizontal);if(last!==undefined){end=last;role=rule.hereRole;}}}}break;
    case 'range':if(prefix)end=at+open.length;break;
    case 'number':if(digit(source,at)||source[at]==='.'&&digit(source,at+1)){end=at+1;const radix=rule.radixPrefixes!.some(prefix=>source.startsWith(prefix,at));while(end<source.length){const c=source[end];if(alnum(source,end)||c==='_'||(c==='.'&&source[end+1]!=='.'&&!ident(source,end+1,true))||(!radix&&(c==='+'||c==='-')&&(source[end-1]==='e'||source[end-1]==='E')&&digit(source,end+1)))end++;else break;}}break;
    case 'identifier':if(ident(source,at,true)){end=word(source,at);identifier=true;const name=source.slice(at,end);role=SPEC.keywords.includes(name)?'k':SPEC.types.includes(name)?'t':undefined;}break;
    case 'plain':end=at+((source.codePointAt(at)??0)>0xffff?2:1);break;
    default:throw new Error('unsupported checked coloring operation');
   }
   if(end>at){if(notePaint)visitRegion(source.slice(start,end),token=>visit({...token,start:token.start+start,end:token.end+start,identifier:rule.semantic===false?false:token.identifier}),true);else visit({start,end,role,identifier});at=end;matched=true;break;}
  }
  if(!matched)throw new Error('checked coloring spec has no consuming fallback');
 }
}
/** Array adapter for fixture comparisons; the editor streams tokens without retaining this array. */
export function colorTokens(source:string):ColorToken[]{const result:ColorToken[]=[];visitColorTokens(source,token=>result.push(token));return result;}
