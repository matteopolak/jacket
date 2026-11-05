import {routeIndex} from './urls.ts';
import type { ApiModule, Declaration, DocumentNode, ResolvedText } from '../generated/doc-site.js';

export const isType = (kind: Declaration['kind']): boolean => ['struct','union','enum','enum_flags','type'].includes(kind);
export const docsFor = (d: Declaration): DocumentNode[] => d.docs.length > 0 ? d.docs : d.overloads[0]?.docs ?? [];
export const lead = (nodes: DocumentNode[]): DocumentNode[] => nodes.find(n => n.kind === 'paragraph')?.kind === 'paragraph' ? nodes.filter(n => n.kind === 'paragraph')[0]?.children ?? [] : [];
export function plain(nodes: DocumentNode[]): string {
  return nodes.map(n => n.kind === 'text' || n.kind === 'code' ? n.text : 'children' in n ? plain(n.children) : n.kind === 'break' ? '\n' : '').join('');
}
export function headingId(label: string): string {
  return label.replace(/[^\p{L}\p{N}]/gu, '-').replace(/[A-Z]/gu, c => c.toLowerCase()).split('-').filter(Boolean).join('-');
}
export interface Group { heading?: { id: string; label: string }; items: Declaration[] }
export interface Section { id: string; label: string; groups: Group[] }
/** Grouping is presentation, using compiler kinds and corrected binding facts directly. */
export function sections(module: ApiModule): Section[] {
  const output: Section[] = [];
  for (const [id,label] of [['procedures','Procedures'],['types','Types'],['constants','Constants']] as const) {
    const items = module.declarations.filter(d => !d.private && (id === 'procedures' ? d.kind === 'procedure' : id === 'types' ? isType(d.kind) : d.kind === 'constant')).sort((a,b) => Number(a.binding)-Number(b.binding));
    if (items.length === 0) continue;
    const names = id === 'procedures' ? [...new Set(items.filter(d => !d.binding).map(d => d.group))] : [];
    const bindings = items.some(d => d.binding);
    const groups: Group[] = [];
    if (names.length > 1 || bindings && names.some(Boolean)) {
      for (const name of names) {
        const label = name || 'Other';
        groups.push({heading:{id:`g-${headingId(label)}`,label},items:items.filter(d => !d.binding && d.group === name)});
      }
      if (bindings) groups.push({heading:{id:'g-foreign',label:'Foreign'},items:items.filter(d => d.binding)});
    } else groups.push({items});
    output.push({id,label,groups});
  }
  return output;
}
/** Slice source and intersect byte spans without parsing HTML or losing Unicode boundaries. */
export function sourceLines(text: ResolvedText): ResolvedText[] {
  let start = 0;
  return text.source.split('\n').map(source => {
    const length = new TextEncoder().encode(source).length;
    const spans = text.spans.filter(span => span.end > start && span.start < start + length).map(span => ({...span,start:Math.max(span.start,start)-start,end:Math.min(span.end,start+length)-start}));
    start += length + 1;
    return {source,spans};
  });
}

export type OutlineEntry =
  | { kind: 'heading'; id: string; label: string; sub: boolean }
  | { kind: 'items'; items: { id: string; label: string }[] };

/** The module outline shares page section grouping and retains expandable item-list siblings. */
export function moduleOutline(module: ApiModule): OutlineEntry[] {
  const outline: OutlineEntry[] = [];
  if (module.parameters.length > 0) outline.push({ kind: 'heading', id: 'parameters', label: 'Module parameters', sub: false });
  for (const section of sections(module)) {
    outline.push({ kind: 'heading', id: section.id, label: section.label, sub: false });
    for (const group of section.groups) {
      if (group.heading) outline.push({ kind: 'heading', id: group.heading.id, label: group.heading.label, sub: true });
      if (group.items.length > 0) outline.push({ kind: 'items', items: group.items.map(item => ({ id: item.name, label: item.name })) });
    }
  }
  return outline.filter(item => item.kind === 'heading').length < 2 ? [] : outline;
}

/** Local API destinations match the visible overloads, declaration and remaining member table. */
export function declarationOutline(declaration: Declaration): OutlineEntry[] {
  const outline: OutlineEntry[] = [];
  if (declaration.kind === 'procedure' && declaration.overloads.length > 1) {
    for (const overload of declaration.overloads) outline.push({ kind: 'heading', id: overload.id, label: overload.label ?? overload.id, sub: false });
  }
  if (declaration.definitions.length) outline.push({ kind: 'heading', id: 'declaration', label: 'Declaration', sub: false });
  const members = memberParameters(declaration).length > 0;
  if (members) outline.push({ kind: 'heading', id: 'members', label: ['enum', 'enum_flags'].includes(declaration.kind) ? 'Members' : 'Fields', sub: false });
  return outline;
}

/** The member table retains type information independently of snippet presentation. */
export function memberParameters(d: Declaration): (import('../generated/doc-site.js').Parameter & {id?:string})[] {
  const inline=new Set(d.definitions.flatMap(definition=>definition.anchors.map(anchor=>anchor.id)));
  return d.members.filter(member=>!inline.has(member.id)).map(member=>({...member,ty:['enum','enum_flags'].includes(d.kind)?{source:'',spans:[]}:member.ty,default:{source:'',spans:[]}}));
}

export function missingPlatforms(module:ApiModule,declaration:Declaration):string[] {
  if(declaration.platforms.length===0)return [];
  const systems=[['MACOS','macOS'],['LINUX','Linux'],['WINDOWS','Windows'],['ANDROID','Android'],['IOS','iOS'],['WASM','Wasm']] as const;
  return systems.filter(([code])=>(module.platforms.length===0?['MACOS','LINUX','WINDOWS'].includes(code):module.platforms.includes(code))||code==='WASM'&&module.wasm).filter(([code])=>!declaration.platforms.includes(code)).map(([,name])=>name);
}

/** Related API destinations come from resolved links, followed by nearby compiler-classified peers. */
export function seeAlso(site:import('../generated/doc-site.js').SiteData,module:ApiModule,d:Declaration):{id:string;label:string}[] {
  const routes=routeIndex(site);
  const output:{id:string;label:string}[]=[];
  const add=(id:string):void=>{const route=routes?.get(id);if(route?.kind==='declaration'&&id!==d.id&&!output.some(item=>item.id===id))output.push({id,label:route.label});};
  function walk(nodes:DocumentNode[]):void {for(const node of nodes){if(node.kind==='link'&&node.target.kind==='page')add(node.target.id);if('children' in node)walk(node.children);if(node.kind==='table'){walk(node.head);walk(node.rows);}if(node.kind==='table_row')walk(node.cells);}}
  walk(docsFor(d));for(const overload of d.overloads)walk(overload.docs);
  const peers=module.declarations.filter(other=>!other.private&&other.binding===d.binding&&other.group===d.group&&isType(other.kind)===isType(d.kind)&&(other.kind==='procedure')===(d.kind==='procedure'));
  if(!d.private){const at=peers.findIndex(other=>other.id===d.id);for(const index of [at-2,at-1,at+1,at+2]){const peer=peers[index];if(peer&&output.length<6)add(peer.id);}}
  return output.slice(0,6);
}


export interface PlatformRow { name:string; state:'none'|'partial'|'full'; tip:string; }
/** Present checked target facts; e.g. a Windows-only declaration is unavailable on macOS. */
export function platformRows(module:ApiModule,declaration?:Declaration):PlatformRow[] {
  const systems=[['MACOS','macOS'],['LINUX','Linux'],['WINDOWS','Windows'],['ANDROID','Android'],['IOS','iOS'],['WASM','Wasm']] as const;
  const desktop=['MACOS','LINUX','WINDOWS'];
  const supports=(code:string):boolean=>module.platforms.length===0?desktop.includes(code)||code==='WASM'&&module.wasm:module.platforms.includes(code)||code==='WASM'&&module.wasm;
  return systems.filter(([code])=>desktop.includes(code)||code==='WASM'||module.platforms.includes(code)).map(([code,name])=>{
    const missing=declaration?supports(code)&&declaration.platforms.length>0&&!declaration.platforms.includes(code)?1:0:module.declarations.filter(item=>!item.private&&item.platforms.length>0&&!item.platforms.includes(code)).length;
    const state=!supports(code)||declaration&&missing>0?'none':missing>0?'partial':'full';
    const tip=declaration?state==='none'?`${declaration.name} is not available on ${name}`:`${declaration.name} works on ${name}`:state==='none'?`Not available on ${name}`:state==='partial'?`${missing} item${missing===1?' is':'s are'} not available on ${name}`:`Everything works on ${name}`;
    return{name,state,tip};
  });
}
