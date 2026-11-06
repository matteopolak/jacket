import type { ResolvedText, StructuredData, Target, TokenRole } from '../generated/doc-site.js';

export interface TextPart {
  text: string;
  role: TokenRole;
  target?: Target;
}

/** Split compiler text at UTF-8 byte offsets, e.g. a two-byte `é` remains one character. */
export function textParts(value: ResolvedText): TextPart[] {
  const bytes = new TextEncoder().encode(value.source);
  const decoder = new TextDecoder('utf-8', { fatal: true });
  const parts: TextPart[] = [];
  let cursor = 0;
  for (const span of value.spans) {
    if (!Number.isSafeInteger(span.start) || !Number.isSafeInteger(span.end) || span.start < cursor || span.end < span.start || span.end > bytes.length) {
      throw new Error('invalid or overlapping resolved-text byte span');
    }
    if (span.start > cursor) parts.push({ text: decoder.decode(bytes.subarray(cursor, span.start)), role: 'plain' });
    const text = decoder.decode(bytes.subarray(span.start, span.end));
    parts.push(span.target ? { text, role: span.role, target: span.target } : { text, role: span.role });
    cursor = span.end;
  }
  if (cursor < bytes.length) parts.push({ text: decoder.decode(bytes.subarray(cursor)), role: 'plain' });
  return parts;
}

/** Insert semantic anchors at UTF-8 boundaries without changing formatter-authored text. */
export function anchoredTextParts(value:ResolvedText,anchors:readonly {id:string;start:number;end:number}[]):(TextPart&{anchors:string[]})[] {
  const bytes=new TextEncoder().encode(value.source),decoder=new TextDecoder('utf-8',{fatal:true});
  const positions=new Map<number,string[]>(),ids=new Set<string>();
  for(const anchor of anchors){
    if(!anchor.id||ids.has(anchor.id)||!Number.isSafeInteger(anchor.start)||!Number.isSafeInteger(anchor.end)||anchor.start<0||anchor.end<=anchor.start||anchor.end>bytes.length)throw new Error('invalid definition anchor');
    decoder.decode(bytes.subarray(0,anchor.start));decoder.decode(bytes.subarray(anchor.start,anchor.end));
    ids.add(anchor.id);positions.set(anchor.start,[...(positions.get(anchor.start)??[]),anchor.id]);
  }
  let offset=0;const out:(TextPart&{anchors:string[]})[]=[];
  for(const part of textParts(value)){
    const size=new TextEncoder().encode(part.text).length,end=offset+size;
    const cuts=[offset,...[...positions.keys()].filter(at=>at>offset&&at<end).sort((a,b)=>a-b),end];
    for(let i=0;i<cuts.length-1;i++){const start=cuts[i]!,finish=cuts[i+1]!;out.push({...part,text:decoder.decode(bytes.subarray(start,finish)),anchors:positions.get(start)??[]});}
    offset=end;
  }
  return out;
}

export interface FormattedTextLine {parts:(TextPart&{anchors:string[]})[];newline:boolean}

/** Wrap existing formatter lines for target highlighting, never reconstruct Jai statements.
 * The catalogue can hide comment/blank lines; lexical string roles protect literal contents.
 */
export function formattedTextLines(value:ResolvedText,anchors:readonly {id:string;start:number;end:number}[]=[],comments=true):FormattedTextLine[] {
  const lines:FormattedTextLine[]=[];let parts:FormattedTextLine['parts']=[];
  for(const part of anchoredTextParts(value,anchors)){
    const chunks=part.text.split('\n');
    chunks.forEach((text,index)=>{
      parts.push({...part,text,anchors:index===0?part.anchors:[]});
      if(index<chunks.length-1){lines.push({parts,newline:true});parts=[];}
    });
  }
  if(parts.some(part=>part.text.length||part.anchors.length))lines.push({parts,newline:false});
  if(comments)return lines;
  return lines.flatMap(line=>{
    const parts=line.parts.filter(part=>part.role!=='comment');
    return parts.every(part=>part.role==='plain'&&!part.text.trim())?[]:[{...line,parts}];
  });
}

/** Build JSON-LD from supported semantic facts, escaping HTML script delimiters after JSON encoding. */
export function jsonLd(facts: StructuredData[]): string {
  const data = facts.map(fact => {
    switch (fact.kind) {
      case 'tech_article': return {
        '@context': 'https://schema.org', '@type': 'TechArticle', headline: fact.headline,
        description: fact.description, url: fact.url ?? undefined,
        isPartOf: { '@type': 'WebSite', name: fact.website_name, url: fact.website_url ?? undefined },
        about: { '@type': 'SoftwareSourceCode', name: fact.module_name },
      };
      case 'breadcrumb_list': return {
        '@context': 'https://schema.org', '@type': 'BreadcrumbList',
        itemListElement: fact.entries.map(entry => ({
          '@type': 'ListItem', position: entry.position, name: entry.name, item: entry.url ?? undefined,
        })),
      };
    }
  });
  return JSON.stringify(data).replace(/[<>&\u2028\u2029]/gu, character => `\\u${character.charCodeAt(0).toString(16).padStart(4, '0')}`);
}

export type Peer = Pick<import('../generated/doc-site.js').Declaration, 'id' | 'kind' | 'binding' | 'group' | 'private'>;

/** Keep the prior forty-peer window and five-row fallback without exporting layout indices. */
export function peerWindow<T extends Peer>(declarations: readonly T[], declaration: T): {
  peers: T[]; current: number; start: number; end: number; first: number; last: number;
  before: number; after: number; visible: T[];
} {
  if (declaration.private) return { peers: [], current: 0, start: 0, end: 0, first: 0, last: 0, before: 0, after: 0, visible: [] };
  const type = (value: Peer): boolean => ['struct', 'union', 'enum', 'enum_flags', 'type'].includes(value.kind);
  const peers = declarations.filter(item => !item.private && item.binding === declaration.binding && item.group === declaration.group
    && type(item) === type(declaration) && (item.kind === 'procedure') === (declaration.kind === 'procedure'));
  const current = peers.findIndex(item => item.id === declaration.id);
  if (current < 0) throw new Error('current declaration is absent from its module peers');
  const start = peers.length > 40 ? Math.max(0, current - 20) : 0;
  const end = peers.length > 40 ? Math.min(start + 40, peers.length) : peers.length;
  const first = end - start > 5 ? Math.max(start, current - 2) : start;
  const last = end - start > 5 ? Math.min(end, current + 3) : end;
  return { peers, current, start, end, first, last, before: first - start, after: end - last, visible: peers.slice(first, last) };
}

/** Emit only component-authored category CSS after checking every configurable color channel. */
export function categoryCSS(categories: import('../generated/doc-site.js').Category[]): string {
  const hex = /^#[0-9a-f]{6}(?:[0-9a-f]{2})?$/iu;
  let light = '', dark = '', system = '';
  for (const [index, category] of categories.entries()) {
    for (const color of [category.color, category.darkColor]) {
      if (color && !hex.test(color)) throw new Error(`invalid category color ${JSON.stringify(color)}`);
    }
    if (!category.badge || !category.color) continue;
    light += `.cat-${index}{--c:${category.color}}`;
    const alternate = category.darkColor || category.color;
    if (alternate !== category.color) {
      dark += `:root[data-theme="dark"] .cat-${index}{--c:${alternate}}`;
      system += `:root[data-theme="system"] .cat-${index}{--c:${alternate}}`;
    }
  }
  return light + dark + (system ? `@media (prefers-color-scheme: dark){${system}}` : '');
}

/** Keep exact parsed code in data; the established display policy omits trailing whitespace. */
export function displayText(value:ResolvedText):ResolvedText {
  const source=value.source.trimEnd(),end=new TextEncoder().encode(source).length;
  return {source,spans:value.spans.filter(span=>span.start<end).map(span=>({...span,end:Math.min(span.end,end)}))};
}
