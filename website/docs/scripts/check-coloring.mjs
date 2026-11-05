/** Compare independent runtime interpretations with reviewed UTF-8 fixture spans, not generated expectations. */
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {colorTokens} from '../src/generated/doc-coloring.ts';
const [fixturePath,producerPath]=process.argv.slice(2),cases=JSON.parse(await readFile(fixturePath,'utf8')),producer=JSON.parse(await readFile(producerPath,'utf8'));
assert.equal(new Set(cases.map(fixture=>fixture.id)).size,cases.length);assert.deepEqual(producer.map(row=>row.id),cases.map(row=>row.id));
for(const fixture of cases){
 const source=fixture.segments.map(segment=>segment[1]).join(''),expected=[];let end=0;
 for(const [role,text]of fixture.segments){const start=end;end+=Buffer.byteLength(text);expected.push({start,end,role});}
 const offsets=new Uint32Array(source.length+1);let units=0,bytes=0;
 for(const char of source){const point=char.codePointAt(0);offsets[units]=bytes;units+=char.length;bytes+=point<=0x7f?1:point<=0x7ff?2:point<=0xffff?3:4;offsets[units]=bytes;}
 const actual=[];
 for(const token of colorTokens(source)){const role=token.role??null,start=offsets[token.start],end=offsets[token.end],previous=actual.at(-1);if(previous?.role===role)previous.end=end;else actual.push({start,end,role});}
 const rust=producer.find(row=>row.id===fixture.id);assert.equal(rust.source,source,fixture.id);assert.deepEqual(rust.lexical,colorTokens(source).map(token=>({start:offsets[token.start],end:offsets[token.end],role:token.role??null,identifier:token.identifier})),`lexical partition ${fixture.id}`);assert.deepEqual(rust.spans,expected,`Rust ${fixture.id}`);assert.deepEqual(actual,expected,`browser ${fixture.id}`);
 assert.equal(colorTokens(source).map(token=>source.slice(token.start,token.end)).join(''),source);
}
console.log(`Shared coloring: ${cases.length} reviewed cases match Rust/browser UTF-8 spans and exact source`);
