import assert from 'node:assert/strict';
import test from 'node:test';
import { fixtureURL, htmlTree, reference } from '../scripts/check-commonmark.mjs';

test('independent reference applies reviewed raw HTML, code, heading and URL policies', () => {
  const result=reference('# A & B\n\n<script>x</script>\n\n~~~rust\nx  \n~~~\n\n[a](javascript:bad "title")\n');
  assert.deepEqual(result.transformations,['code-presentation','fixture-url-map','heading-identifiers','literal-raw-html']);
  assert.match(result.html,/<h1 id="a--b">A &amp; B<\/h1>/u);
  assert.match(result.html,/&lt;script&gt;x&lt;\/script&gt;/u);
  assert.match(result.html,/<pre class="code wrap"><code>x<\/code><\/pre>/u);
  assert.ok(result.html.includes(fixtureURL('javascript:bad')));
});
test('DOM oracle ignores entity serialization and block gaps but retains inline and code whitespace', () => {
  assert.deepEqual(htmlTree('<p>&quot;</p>\n<p>x</p>'),htmlTree('<p>"</p><p>x</p>'));
  assert.notDeepEqual(htmlTree('<p>x y</p>'),htmlTree('<p>xy</p>'));
  assert.notDeepEqual(htmlTree('<pre><code>x \n</code></pre>'),htmlTree('<pre><code>x</code></pre>'));
  assert.notDeepEqual(htmlTree('<p>&lt;b&gt;</p>'),htmlTree('<p><b></b></p>'));
});
test('fixture URLs compare URI semantics without merging escaped structural separators', () => {
  assert.equal(fixtureURL('/föö'),fixtureURL('/f%C3%B6%C3%B6'));
  assert.equal(fixtureURL('/my uri'),fixtureURL('/my%20uri'));
  assert.notEqual(fixtureURL('/a/b'),fixtureURL('/a%2Fb'));
});
test('literal HTML blocks retain source line endings independently of reference AST finalization', () => {
  assert.equal(reference('<div>\n').html,'&lt;div&gt;\n');
  assert.equal(reference('<div>').html,'&lt;div&gt;');
});
