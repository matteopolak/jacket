/** Assert real Astro output suppresses empty optional sections and preserves authored zeroes. */
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {parse} from 'parse5';
const root=process.argv[2];
assert(root,'pass the rendered conditional fixture directory');
const all=node=>[node,...(node.childNodes??[]).flatMap(all)];
const text=node=>node.nodeName==='#text'?node.value:(node.childNodes??[]).map(text).join('');
const id=(tree,value)=>all(tree).find(node=>node.attrs?.some(attribute=>attribute.name==='id'&&attribute.value===value));
const cls=(tree,value)=>all(tree).find(node=>node.attrs?.some(attribute=>attribute.name==='class'&&attribute.value.split(/\s+/u).includes(value)));
for(const name of ['module','empty','zero','record']) {
 const tree=parse(await readFile(resolve(root,`${name}.html`),'utf8'));
 const main=id(tree,'fixture');assert(main,`${name}: fixture missing`);
 const scope=name==='module'?id(main,'empty'):main;
 assert(!(scope.childNodes??[]).some(node=>node.nodeName==='#text'&&node.value.trim()==='0'),`${name}: empty declaration emitted a numeric placeholder`);
 if(name==='empty')assert(!id(main,'declaration'),'empty declaration section should be absent');
 if(name==='record') {
  const snippet=cls(main,'semantic-declaration');assert(id(main,'declaration')&&snippet,'populated declaration should render');
  assert.equal(text(snippet),'record :: struct {\n    field: int;\n}','formatter text must not change when lines are wrapped');
  const anchor=id(snippet,'m-field');assert(anchor,'field identity missing');
  let parent=anchor.parentNode;while(parent&&parent!==snippet&&!parent.attrs?.some(value=>value.name==='class'&&value.value.split(/\s+/).includes('formatted-line')))parent=parent.parentNode;
  assert(parent&&parent!==snippet,'field target needs its formatter-owned line wrapper');
 }
 if(name==='zero') {
  assert.equal(text(cls(main,'lead')),'0','authored zero description must remain');
  assert(all(main).some(node=>node.tagName==='code'&&text(node).includes(':: 0')),'zero default/value must remain');
 }
 if(name==='module') {
  assert.equal(text(cls(id(main,'zero'),'summary')),'0','module card must preserve authored zero description');
  assert(cls(id(main,'record'),'semantic-declaration'),'module card must render populated declaration');
 }
}
process.stdout.write('Verified four production component cases: empty, populated and meaningful zero content.\n');
