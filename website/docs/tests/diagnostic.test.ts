import assert from 'node:assert/strict';
import {test} from 'node:test';
import {diagnosticParts} from '../src/lib/diagnostic.ts';

test('fix operation colors are independent of error/warning severity and keep origin markers distinct',()=>{
  for(const severity of ['error','warning']) {
    const source=`${severity}[fixture]: finding Ω\n  --> example.jai:5:2\n  5 | -original\n    | ^^^ primary\n    | --- secondary origin\n  5 - removed();\n  5 + added();\n  5 ~ replaced();\n  = help: keep <verbatim> content\n`;
    const parts=diagnosticParts(source);
    assert.equal(parts.map(part=>part.text).join(''),source);
    assert.deepEqual(parts.filter(part=>part.role==='deletion').map(part=>part.text),['-']);
    assert.deepEqual(parts.filter(part=>part.role==='addition').map(part=>part.text),['+']);
    assert.deepEqual(parts.filter(part=>part.role==='replacement').map(part=>part.text),['~']);
    assert(parts.some(part=>part.role==='secondary'&&part.text==='--- secondary origin'));
    assert(parts.some(part=>part.role==='frame'&&part.text==='-->'));
    assert(parts.some(part=>part.role===severity&&part.text===`${severity}[fixture]`));
    assert(!parts.some(part=>part.role==='deletion'&&part.text.includes('original')));
  }
});
