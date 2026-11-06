import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {test} from 'node:test';
import {diagnosticFixtureProblems,productionDiagnosticCases} from './diagnostic_fixture_checks.mjs';

test('production color fixtures contain their required real operations and both severities',async()=>{
  const covered=new Set(),severities=new Set();
  const patterns={dd:/^\s*\d+ - /mu,da:/^\s*\d+ \+ /mu,dc:/^\s*\d+ ~ /mu,dk:/^\s*\|[ \t]+-{2,}/mu};
  for(const fixture of productionDiagnosticCases){
    const rule=fixture.page.split('/').at(-1);
    const source=await readFile(new URL(`../../tests/lint/${rule}/example.snap`,import.meta.url),'utf8');
    assert(new RegExp(`^${fixture.severity}\\[${rule}\\]`,'m').test(source),`${rule}: severity`);
    severities.add(fixture.severity);
    for(const role of fixture.required){assert(patterns[role].test(source),`${rule}: missing actual ${role}`);covered.add(role);}
  }
  assert.deepEqual([...covered].sort(),['da','dc','dd','dk']);
  assert.deepEqual([...severities].sort(),['error','warning']);
});

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
