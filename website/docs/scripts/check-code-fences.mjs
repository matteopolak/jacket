/** Independent DOM assertions for the production component's real non-Jai code fences. */
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {parse} from 'parse5';
const [casesPath,pagePath]=process.argv.slice(2);
const cases=JSON.parse(await readFile(casesPath,'utf8')),page=parse(await readFile(pagePath,'utf8'));
const attr=(node,name)=>node.attrs?.find(attribute=>attribute.name===name)?.value;
const all=(node,predicate)=>[...(predicate(node)?[node]:[]),...(node.childNodes??[]).flatMap(child=>all(child,predicate))];
const text=node=>node.nodeName==='#text'?node.value:(node.childNodes??[]).map(text).join('');
for(const fixture of cases){
 const section=all(page,node=>node.tagName==='section'&&attr(node,'id')===fixture.language)[0];assert(section);
 const pre=all(section,node=>node.tagName==='pre');assert.equal(pre.length,1);assert.equal(attr(pre[0],'style'),undefined,'highlighter overrides existing fence background/layout');
 assert.equal(text(pre[0]),fixture.source.trimEnd(),`${fixture.language}: displayed text changed`);
 assert.equal(all(section,node=>node.tagName==='script').length,0,'author source became executable markup');
 const colored=all(pre[0],node=>node.tagName==='span'&&attr(node,'style')?.includes('color:'));
 if(fixture.language==='unknown-fixture-language')assert.equal(colored.length,0);
 else{
  assert(colored.length>0,`${fixture.language}: supported fence lost highlighting`);
  assert(colored.some(node=>attr(node,'style').includes('var(--str)')),`${fixture.language}: string palette missing`);
  assert(colored.some(node=>attr(node,'style').includes('var(--muted)')),`${fixture.language}: comment palette missing`);
  assert(colored.every(node=>/^color:var\(--(?:fg|kw|str|type|muted)\)(?:;font-style:italic)?$/u.test(attr(node,'style'))),`${fixture.language}: unrelated color or layout style`);
 }
}
console.log(`Code fences: ${cases.length} production rendered fixtures passed`);
