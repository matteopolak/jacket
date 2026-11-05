/** Semantic graph validation complements JSON shape validation before any component renders. */
import type { Capability, Example, ApiModule, DocumentNode, Parameter, Payload, ResolvedText, SiteData, Target, PageMetadata } from '../generated/doc-site.js';
import { textParts, anchoredTextParts } from './render.ts';
import { resolver, externalURL } from './urls.ts';
import {validateExample} from './example-validation.ts';

export function graphValidator(site: SiteData, assets:ReadonlySet<string>=new Set(), capabilities:ReadonlySet<Capability>=new Set()): (payload: Payload, path: string) => void {
  const admitExample=(value:Example):void=>{
    validateExample(value);
    if(value.runnable&&(!capabilities.has('browser_runtime')||!capabilities.has('editable_examples')||!assets.has('runtime/engine.mjs')||!assets.has('runtime/jai_wasm.wasm')))throw new Error('runnable example requires bundle runtime capabilities and both runtime assets');
  };
  const href=resolver(site);
  const ids=new Set<string>();
  const targets=(target: Target): void => { if(target.kind==='asset'&&!assets.has(target.path))throw new Error(`unlisted asset ${target.path}`);href(target); };
  function text(value: ResolvedText): void {
    textParts(value);
    for(const span of value.spans)if(span.target)targets(span.target);
  }
  function nodes(values: DocumentNode[],depth=0): void {
    if(depth>256)throw new Error('document nesting exceeds import limit256');
    for(const node of values) {
      if('children' in node)nodes(node.children,depth+1);
      if(node.kind==='heading' && (!Number.isInteger(node.level)||node.level<1||node.level>6))throw new Error('heading level must be1..6');
      if(node.kind==='link'||node.kind==='image')targets(node.target);
      if(node.kind==='table') {nodes(node.head,depth+1);nodes(node.rows,depth+1);}
      if(node.kind==='table_row')nodes(node.cells,depth+1);
      if(node.kind==='code_block')text(node.source);
      if(node.kind==='fence') {
        text(node.example.source);
        admitExample(node.example);
      }
      if(node.kind==='example_group') {
        if(node.source.kind!=='code_block'&&node.source.kind!=='fence')throw new Error('example source must be a code fence');
        nodes([node.source],depth+1);
        if(node.result) {
          if(node.title!=='example'||node.result.body.kind!=='code_block')throw new Error('example result must belong to an example and contain static code');
          nodes([node.result.body],depth+1);
        }
      }
      if(node.kind==='compare'){text(node.before);text(node.after);}
    }
  }
  const parameter=(p: Parameter): void => {text(p.ty);text(p.default);nodes(p.docs);};
  const unique=(id:string): void=>{if(ids.has(id))throw new Error(`duplicate declaration/document ID ${id}`);ids.add(id);};
  function metadata(value:PageMetadata):void {
    for(const url of [value.url,value.canonical,value.image])if(url)externalURL(url);
    for(const data of value.structured){
      if(data.kind==='tech_article'){if(data.url)externalURL(data.url);if(data.website_url)externalURL(data.website_url);}
      else for(const entry of data.entries)if(entry.url)externalURL(entry.url);
    }
  }
  function module(m: ApiModule): void {
    metadata(m.metadata);if(m.sourceUrl)externalURL(m.sourceUrl);
    nodes(m.docs);m.parameters.forEach(parameter);
    for(const d of m.declarations) {
      metadata(d.metadata);if(d.sourceUrl)externalURL(d.sourceUrl);
      unique(d.id);targets({kind:'page',id:d.route,anchor:null});
      const definitionIds=new Set<string>(),anchorIds=new Set<string>();
      for(const definition of d.definitions){
        if(definitionIds.has(definition.id)||anchorIds.has(definition.id))throw new Error(`duplicate definition ${d.id}#${definition.id}`);definitionIds.add(definition.id);
        if(definition.sourceUrl)externalURL(definition.sourceUrl);
        text(definition.code);nodes(definition.docs);anchoredTextParts(definition.code,definition.anchors);
        for(const anchor of definition.anchors){if(anchorIds.has(anchor.id)||definitionIds.has(anchor.id))throw new Error(`duplicate definition anchor ${d.id}#${anchor.id}`);anchorIds.add(anchor.id);}
      }
      text(d.value);nodes(d.docs);
      const members=new Set<string>(),overloads=new Set<string>();
      for(const member of d.members) {if(members.has(member.id))throw new Error(`duplicate member ${d.id}#${member.id}`);members.add(member.id);text(member.ty);nodes(member.docs);}
      for(const overload of d.overloads) {if(overloads.has(overload.id))throw new Error(`duplicate overload ${d.id}#${overload.id}`);overloads.add(overload.id);text(overload.signature);nodes(overload.docs);overload.parameters.forEach(parameter);overload.results.forEach(parameter);}
    }
  }
  return (payload,path)=>{
    try {
      if(payload.payload==='module')module(payload.data);
      if(payload.payload==='example'){text(payload.data.source);admitExample(payload.data);}
      if(payload.payload==='document'){metadata(payload.data.metadata);unique(payload.data.id);nodes(payload.data.nodes);}
      if(payload.payload==='search') {
        if(payload.data.schemaVersion!==1)throw new Error('unsupported search schemaVersion');
        for(const entry of payload.data.entries)targets(entry.target);
      }
      if(payload.payload==='site') {
        const walk=(groups:SiteData['navigation']):void=>{for(const group of groups){for(const entry of group.entries)targets(entry.target);walk(group.children);}};
        for(const url of [payload.data.repository,payload.data.playgroundUrl,payload.data.socialImage,payload.data.canonicalOrigin])if(url)externalURL(url);
        if(payload.data.logo)targets({kind:'asset',path:payload.data.logo});
        walk(payload.data.navigation);for(const link of payload.data.headerLinks)targets(link.target);
      }
    }catch(error){throw new Error(`${path}: invalid documentation graph: ${error instanceof Error?error.message:String(error)}`,{cause:error});}
  };
}
