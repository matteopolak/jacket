/** Inspect actual producer/consumer output with the independent standards HTML parser. */
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { parse } from 'parse5';
const [bundle,output,base]=process.argv.slice(2);
const site=JSON.parse(await readFile(resolve(bundle,'site.json'),'utf8')).data;
const manifest=JSON.parse(await readFile(resolve(bundle,'manifest.json'),'utf8')).data;
assert.equal(manifest.producer.formatter?.profile,'documentation-declarations/1');
assert.match(manifest.producer.formatter.sha256,/^[0-9a-f]{64}$/u);
assert.equal(manifest.producer.formatter.maxWidth,88);
const page=parse(await readFile(resolve(output,'index.html'),'utf8'));
const anchors=[],proseAnchors=[],elements=[],text=[];
function contents(node){return node.nodeName==='#text'?node.value:(node.childNodes??[]).map(contents).join('');}
function walk(node,inProse=false) {
  const prose=inProse||(node.attrs??[]).some(attr=>attr.name==='class'&&attr.value.split(/\s+/u).includes('prose'));
  if(node.tagName)elements.push(node);
  if(node.nodeName==='#text')text.push(node.value);
  if(node.tagName==='a'){anchors.push(node);if(prose)proseAnchors.push(node);}
  for(const child of node.childNodes??[])walk(child,prose);
}
walk(page);
const attr=(node,name)=>node.attrs.find(a=>a.name===name)?.value;
const link=label=>proseAnchors.find(node=>contents(node)===label);
assert.equal(attr(link('Fixture guide'),'href'),`${base}guides/guide#destination`);
const module=site.routes.find(route=>route.kind==='module');assert.ok(module);
assert.equal(attr(link('Fixture API'),'href'),`${base}${module.route}`);
const modulePage=parse(await readFile(resolve(output,`${module.output}`),'utf8'));
assert.ok(contents(modulePage).includes('fixture_record :: struct {\n    first: int;\n}'),'real installed extraction must use the native declaration formatter');
assert.equal(attr(link('External'),'href'),'https://example.com/q?a=1&b=2');
assert.equal(attr(link('External'),'title'),'A & B');
assert.equal(link('Unsafe'),undefined);
assert.ok(text.join('').includes('<script>window.fixtureInjected=1</script>'));
assert.ok(text.join('').includes('<img src=x onerror=fixtureInjected()>'));
assert.ok(text.join('').includes('Text & "quotes" <not-a-template> Ω.'));
assert.ok(!elements.some(node=>node.attrs.some(a=>a.name==='onerror'||a.name==='onmouseover')));
assert.ok(!elements.some(node=>node.tagName==='script'&&contents(node).includes('fixtureInjected')));
for(const node of anchors)assert.ok(!/^(javascript|data):/iu.test(attr(node,'href')??''));
console.log(`Verified actual guide URL/escaping policies under ${base}`);
