// The playground page's address and option rules (crates/jacket/src/doc/site/assets/playground-state.mjs).
import test from 'node:test';
import assert from 'node:assert/strict';
import {
  DEFAULTS,
  decode,
  encode,
  formatConfig,
  fromBase64,
  toBase64,
  words,
} from '../../crates/jacket/src/doc/site/assets/playground-state.mjs';

const one = { files: { 'main.jai': 'main :: () {}\n' }, active: 'main.jai', example: '' };

test('code survives the address, including text outside ASCII', () => {
  const text = 'print("héllo ✓ 😀\\n");\n';
  assert.equal(fromBase64(toBase64(text)), text);
  assert.doesNotMatch(toBase64(text), /[+/=]/);
});

test('one main.jai is the #code= form the docs link to, with no options at their defaults', () => {
  const fragment = encode(one, DEFAULTS);
  assert.equal(fragment, 'code=' + toBase64('main :: () {}\n'));
  assert.deepEqual(decode('#' + fragment, DEFAULTS), { work: one, options: DEFAULTS });
});

test('options that differ travel with the code', () => {
  const options = { ...DEFAULTS, limit: '2b', args: 'a "b c"', show: 'errors', indent: '2' };
  const fragment = encode(one, options);
  assert.match(fragment, /&limit=2b&args=a%20%22b%20c%22&show=errors&indent=2$/);
  assert.doesNotMatch(fragment, /braces|stdin/);
  assert.deepEqual(decode(fragment, DEFAULTS).options, options);
});

test('several files travel as files= and keep the open one', () => {
  const work = {
    files: { 'main.jai': '#load "a.jai";\n', 'a.jai': 'x :: 1;\n' },
    active: 'a.jai',
    example: 'hello',
  };
  const fragment = encode(work, DEFAULTS);
  assert.match(fragment, /^files=/);
  assert.deepEqual(decode(fragment, DEFAULTS).work, work);
});

test('a fragment with no code, or broken code, leaves the page alone', () => {
  assert.equal(decode('', DEFAULTS).work, null);
  assert.deepEqual(decode('#limit=20m', DEFAULTS), {
    work: null,
    options: { ...DEFAULTS, limit: '20m' },
  });
  assert.deepEqual(decode('#code=%%%', DEFAULTS), { work: null, options: DEFAULTS });
  assert.equal(decode('#files=' + toBase64('[]'), DEFAULTS).work, null);
});

test('arguments split on spaces, with quotes grouping', () => {
  assert.deepEqual(words(''), []);
  assert.deepEqual(words('a  b'), ['a', 'b']);
  assert.deepEqual(words(`"x y" 'z w' q`), ['x y', 'z w', 'q']);
});

test('format settings are written as a [format] table, and only known values are', () => {
  assert.equal(
    formatConfig({ ...DEFAULTS, indent: '2', braces: 'preserve' }),
    '[format]\nindent-width = 2\nbrace-style = "preserve"\n',
  );
  assert.match(
    formatConfig({ ...DEFAULTS, indent: '99', braces: 'x"' }),
    /= 4\nbrace-style = "same_line"/,
  );
});
