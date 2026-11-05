/** Check production component output against real generated lint source/result nodes. */
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {join} from 'node:path';
import {parse} from 'parse5';
import {displayText} from '../src/lib/render.ts';
import {diagnosticFixtureProblems} from '../../../tools/dev/diagnostic_fixture_checks.mjs';
const root=process.argv[2];
if(!root)throw new Error('usage: node check-examples.mjs OWNED_FIXTURE_ROOT');
const text=node=>node.nodeName==='#text'?node.value:(node.childNodes??[]).map(text).join('');
const attr=(node,name)=>node.attrs?.find(value=>value.name===name)?.value;
const has=(node,name)=>(attr(node,'class')??'').split(/\s+/).includes(name);
function find(node,predicate){return [node,...(node.childNodes??[]).flatMap(child=>find(child,predicate))].filter(predicate);}
for(const record of JSON.parse(await readFile(join(root,'nodes.json'),'utf8'))){
  const html=parse(await readFile(join(root,'dist',`${record.id}.html`),'utf8'));
  const expected=record.nodes.filter(node=>node.kind==='example_group');
  const groups=find(html,node=>has(node,'example-group'));
  assert.equal(groups.length,expected.length,`${record.id}: explicit group count`);
  assert.equal(find(html,node=>node.tagName==='p'&&['Example:','Fixed:'].includes(text(node).trim())).length,0,`${record.id}: ghost prose labels`);
  groups.forEach((group,index)=>{
    const node=expected[index];
    assert.equal(text(find(group,child=>child.tagName==='figcaption')[0]).trim(),node.title==='fixed'?'Fixed':`Example${node.note??''}`);
    const source=find(group,child=>has(child,'example-source'));
    assert.equal(source.length,1);
    assert.equal(text(source[0]),displayText(node.source.source).source,`${record.id}: source display bytes`);
    const result=find(group,child=>has(child,'example-result'));
    assert.equal(result.length,node.result?1:0,`${record.id}: result ownership`);
    if(node.result){assert.equal(attr(result[0],'aria-label'),'Diagnostic');assert.equal(text(result[0]),displayText(node.result.body.source).source,`${record.id}: diagnostic display bytes`);assert.deepEqual(diagnosticFixtureProblems(record.id,text(result[0])),[],`${record.id}: span/context expectations`);}
  });
}
console.log('Real lint references: semantic headers, exact source/result bytes, span/context expectations and ownership pass');
