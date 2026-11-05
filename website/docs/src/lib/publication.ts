/** One publication authority for static builds and Astro's development public directory. */
import {mkdir,writeFile} from 'node:fs/promises';
import {dirname,resolve} from 'node:path';
import type {ApiModule,DocumentNode,Example} from '../generated/doc-site.js';
import type {Bundle} from './loader.ts';
import {safeRelativePath} from './loader.ts';
import {moduleNavigation,moduleNavigationPath} from './navigation-path.ts';

/** Visit typed document content, including table cells and attached source/result examples. */
function* documentExamples(nodes:readonly DocumentNode[]):Generator<Example>{
 for(const node of nodes){
  if(node.kind==='fence')yield node.example;
  else if('children' in node)yield* documentExamples(node.children);
  else if(node.kind==='table')yield* documentExamples([...node.head,...node.rows]);
  else if(node.kind==='table_row')yield* documentExamples(node.cells);
  else if(node.kind==='example_group'){yield* documentExamples([node.source]);if(node.result)yield* documentExamples([node.result.body]);}
 }
}

/** The compiler DTO owns every documentation slot; publication never reparses source text. */
function* moduleExamples(module:ApiModule):Generator<Example>{
 yield* documentExamples(module.docs);
 for(const parameter of module.parameters)yield* documentExamples(parameter.docs);
 for(const declaration of module.declarations){
  yield* documentExamples(declaration.docs);
  for(const definition of declaration.definitions)yield* documentExamples(definition.docs);
  for(const member of declaration.members)yield* documentExamples(member.docs);
  for(const overload of declaration.overloads){
   yield* documentExamples(overload.docs);
   for(const parameter of [...overload.parameters,...overload.results])yield* documentExamples(parameter.docs);
  }
 }
}

/** Write only admitted assets and derived browser facts; e.g. writePublication(bundle, staging). */
export async function writePublication(bundle:Bundle,directory:string):Promise<void>{
 const examples=new Map<string,string>();
 async function write(path:string,bytes:string|Uint8Array):Promise<void>{
  safeRelativePath(path);const output=resolve(directory,path);
  await mkdir(dirname(output),{recursive:true});await writeFile(output,bytes,{flag:'wx'});
 }
 async function example(example:Example):Promise<void>{
  if(!example.runnable)return;
  if(!/^[a-f0-9]{12}$/u.test(example.id))throw new Error('invalid example asset ID');
  const bytes=JSON.stringify({payload:'example',data:example}),previous=examples.get(example.id);
  if(previous!==undefined){if(previous!==bytes)throw new Error(`example ID collision ${example.id}`);return;}
  examples.set(example.id,bytes);await write(`examples/${example.id}.json`,bytes);
 }
 for(const asset of bundle.manifest.assets)await write(asset.path,await bundle.asset(asset.path));
 for(const shard of bundle.manifest.shards){
  const payload=await bundle.shard(shard.path);
  if(payload.payload==='module'){
   for(const item of moduleExamples(payload.data))await example(item);
   await write(moduleNavigationPath(payload.data.id),moduleNavigation(payload.data).json);
  }else if(payload.payload==='document'){
   for(const item of documentExamples(payload.data.nodes))await example(item);
  }else if(payload.payload==='search')await write('search-index.json',JSON.stringify(payload));
 }
 await write('versions.json',JSON.stringify(bundle.site.version?[{version:bundle.site.version,url:'./'}]:[]));
}
