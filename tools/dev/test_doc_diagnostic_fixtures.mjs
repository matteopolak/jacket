import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {test} from 'node:test';
import {diagnosticFixtureProblems} from './diagnostic_fixture_checks.mjs';

test('real owning snapshots satisfy the four diagnostic span and context expectations',async()=>{
  for(const rule of ['neg_cmp_float','manual_array_find','almost_swapped','manual_index_counter']){
    const source=await readFile(new URL(`../../tests/lint/${rule}/example.snap`,import.meta.url),'utf8');
    assert.deepEqual(diagnosticFixtureProblems(rule,source),[],`${rule}: actual snapshot`);
    // Only constructed variants change terminators; the actual snapshot is checked above.
    const fixture=source.replaceAll(/\r\n|\r/gu,'\n');
    for(const ending of ['\n','\r\n','\r'])assert.deepEqual(diagnosticFixtureProblems(rule,fixture.replaceAll('\n',ending)),[],`${rule}: ${JSON.stringify(ending)}`);
  }
});

test('missing closing parenthesis and trailing header whitespace remain visible failures',()=>{
  const neg=' 5 |     if !(x < limit) {\n   |        ^^^^^^^^^^^\n';
  const array=' 6 |     for names {\n   |     ^^^^^^^^^^\n';
  assert(diagnosticFixtureProblems('neg_cmp_float',neg).length>0);
  assert(diagnosticFixtureProblems('manual_array_find',array).length>0);
});
