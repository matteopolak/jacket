import assert from 'node:assert/strict';
import test from 'node:test';
import {readFile} from 'node:fs/promises';
import {colorTokens} from '../src/generated/doc-coloring.ts';
const fixtures=JSON.parse(await readFile(new URL('../../../crates/jacket/src/doc/coloring/fixtures.json',import.meta.url),'utf8')) as {id:string;segments:[string|null,string][]}[];
for(const fixture of fixtures)test(`shared coloring preserves ${fixture.id}`,()=>{
 const source=fixture.segments.map(segment=>segment[1]).join('');const actual:[string|null,string][]=[];
 for(const token of colorTokens(source)){const role=token.role??null,text=source.slice(token.start,token.end),previous=actual.at(-1);if(previous?.[0]===role)previous[1]+=text;else actual.push([role,text]);}
 assert.deepEqual(actual,fixture.segments);assert.equal(actual.map(segment=>segment[1]).join(''),source);
});
test('browser source offsets do not rewrite astral or unpaired UTF-16 input',()=>{
 for(const source of ['😀 "雪"','\ud800','"\ud800\\"','/* \udfff */'])assert.equal(colorTokens(source).map(token=>source.slice(token.start,token.end)).join(''),source);
});
test('partial Unicode edits partition source and deep comments need no recursive stack',()=>{
 for(const scalar of ['😀','雪','\u00a0','\ufeff','"','\\','\n',',','@','#'])for(const source of [`#string,\\${scalar} TAG\nbody\nTAG; return`,`/*${scalar}/*nested*/remaining*/`,`@note(${scalar}\nreturn`,`"\\${scalar}`]){
  const tokens=colorTokens(source);assert.equal(tokens.map(token=>source.slice(token.start,token.end)).join(''),source);assert(tokens.every(token=>token.end>token.start));
 }
 const source='/*'.repeat(8192)+'*/'.repeat(8192),tokens=colorTokens(source);assert.equal(tokens.length,1);assert.equal(tokens[0]?.role,'c');assert.equal(tokens[0]?.end,source.length);
});
