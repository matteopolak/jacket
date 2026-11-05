import assert from 'node:assert/strict';
import { test } from 'node:test';
import { firstGuidePath, pageStatus, waitForFrames } from './doc_samples.mjs';
import { createServer } from 'node:http';
import { once } from 'node:events';
import vm from 'node:vm';

test('the guide navigation target is already a page, including under a base path', () => {
  for (const base of ['/', '/reference/']) {
    const html = `<a class="card" href="wrong"></a><nav class="site-nav" aria-label="Site">
      <a href="${base}">API</a><a href="${base}guides/testing">Guides</a><a href="${base}tools">Tools</a></nav>`;
    assert.equal(firstGuidePath(html), `${base}guides/testing`);
  }
});

test('a missing guide fails instead of silently reducing browser coverage', () => {
  assert.throws(() => firstGuidePath('<nav class="site-nav"><a href="/">API</a></nav>'), /No guide page/);
});

test('status probes request no response body and preserve missing-page status', async () => {
  const methods = [];
  const server = createServer((request, response) => {
    methods.push(request.method);
    response.writeHead(request.url === '/missing' ? 404 : 200, { 'Content-Length': 262144, Connection: 'close' });
    response.end(request.method === 'HEAD' ? undefined : Buffer.alloc(262144));
  });
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
  try {
    const base = `http://127.0.0.1:${server.address().port}`;
    assert.equal(await pageStatus(`${base}/guide`), 200);
    assert.equal(await pageStatus(`${base}/missing`), 404);
    assert.deepEqual(methods, ['HEAD', 'HEAD']);
  } finally {
    server.closeAllConnections();
    await new Promise(resolve => server.close(resolve));
  }
});

test('frame synchronization cannot finish on elapsed time before two browser frames', async () => {
  const callbacks = [];
  let finished = false;
  const waiting = waitForFrames(expression => vm.runInNewContext(expression, {
    requestAnimationFrame: callback => callbacks.push(callback),
  })).then(() => { finished = true; });
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(finished, false);
  assert.equal(callbacks.length, 1);
  callbacks.shift()();
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(finished, false);
  assert.equal(callbacks.length, 1);
  callbacks.shift()();
  await waiting;
  assert.equal(finished, true);
});

test('foreign accessibility sample uses the versioned producer target without a display-name slugger', async () => {
  const { foreignItemPath } = await import('./doc_samples.mjs');
  assert.equal(foreignItemPath({payload:'search',data:{schemaVersion:1,entries:[
    {kind:'module',binding:true,target:{kind:'page',id:'api/Ignore'}},
    {kind:'declaration',binding:true,name:'operator /',target:{kind:'page',id:'api/Native/canonical-2f',anchor:null}},
  ]}}),'api/Native/canonical-2f');
  for(const input of [{items:[]},{payload:'search',data:{schemaVersion:2,entries:[]}},
    {payload:'search',data:{schemaVersion:1,entries:[]}},
    {payload:'search',data:{schemaVersion:1,entries:[{kind:'declaration',binding:true,target:{kind:'page',id:'../escape'}}]}}])
    assert.throws(()=>foreignItemPath(input));
});
