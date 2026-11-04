#!/usr/bin/env node
// No npm install: one hash-pinned axe-core distribution and Node's built-in CDP WebSocket.
// axe-core 4.10.3 predates the dependency-age policy's 14-day minimum. No transitive packages.
import assert from 'node:assert/strict';
import { waitForDevToolsPort } from './chrome_startup.mjs';
import { createHash } from 'node:crypto';
import { spawn, spawnSync } from 'node:child_process';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

const AXE_URL = 'https://unpkg.com/axe-core@4.10.3/axe.min.js';
const AXE_SHA256 = '880970c081707360e64f34cea25ff91892f5bc95675b0776925b9709dd8a68bb';
const tags = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'];
const base = new URL(process.argv[2] || 'http://127.0.0.1:8000/');
const cache = path.join(tmpdir(), 'jacket-axe-4.10.3.min.js');
let axe;
try { axe = await readFile(cache, 'utf8'); } catch { axe = await (await fetch(AXE_URL)).text(); }
assert.equal(createHash('sha256').update(axe).digest('hex'), AXE_SHA256, 'axe-core integrity');
await writeFile(cache, axe);
const chrome = process.env.CHROME_PATH || (process.platform === 'darwin'
  ? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'
  : ['google-chrome', 'google-chrome-stable', 'chromium', 'chromium-browser'].find(p => spawnSync('which', [p]).status === 0));
assert(chrome, 'Install Chrome or set CHROME_PATH');
const profile = await mkdtemp(path.join(tmpdir(), 'jacket-a11y-'));
const child = spawn(chrome, ['--headless=new', '--no-sandbox', '--disable-gpu', '--no-first-run',
  '--remote-debugging-port=0', `--user-data-dir=${profile}`, 'about:blank'], { stdio: ['ignore', 'ignore', 'pipe'] });
let socket;
try {
  const port = await waitForDevToolsPort(child, profile);
  const targets = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
  socket = new WebSocket(targets.find(t => t.type === 'page').webSocketDebuggerUrl);
  await new Promise((resolve, reject) => { socket.onopen = resolve; socket.onerror = reject; });
  let next = 0;
  const pending = new Map();
  const interceptErrors = [];
  let publicationOrigin = '', loads = 0, scriptErrors = [];
  socket.onmessage = async event => {
    const message = JSON.parse(event.data);
    if (message.method === 'Page.loadEventFired') { loads++; return; }
    if (message.method === 'Runtime.exceptionThrown') { scriptErrors.push(message.params.exceptionDetails.exception?.description || message.params.exceptionDetails.text); return; }
    if (message.method === 'Fetch.requestPaused') {
      const { requestId, request } = message.params;
      try {
        // 404 links use the publication origin. Serve those requests from this build,
        // including stylesheet and search fetches, rather than testing deployed assets.
        const remote = new URL(request.url);
        const response = await fetch(new URL(remote.pathname + remote.search, base));
        let body = Buffer.from(await response.arrayBuffer());
        if (message.params.resourceType === 'Document' && publicationOrigin) body = Buffer.from(body.toString('utf8').replaceAll(publicationOrigin, base.origin));
        await cdp('Fetch.fulfillRequest', { requestId, responseCode: response.status,
          responseHeaders: [{ name: 'Content-Type', value: response.headers.get('content-type') || 'application/octet-stream' },
            { name: 'Access-Control-Allow-Origin', value: '*' }],
          body: body.toString('base64') });
      } catch (error) { interceptErrors.push(String(error)); await cdp('Fetch.failRequest', { requestId, errorReason: 'Failed' }); }
      return;
    }
    const entry = pending.get(message.id);
    if (!entry) return;
    pending.delete(message.id); clearTimeout(entry.timer);
    if (message.error) entry.reject(new Error(JSON.stringify(message.error))); else entry.resolve(message.result);
  };
  function cdp(method, params = {}) {
    return new Promise((resolve, reject) => {
      const id = ++next;
      const timer = setTimeout(() => { pending.delete(id); reject(new Error(`${method} timed out`)); }, 60_000);
      pending.set(id, { resolve, reject, timer }); socket.send(JSON.stringify({ id, method, params }));
    });
  }
  async function evaluate(expression) {
    const result = await cdp('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true });
    assert(!result.exceptionDetails, JSON.stringify(result.exceptionDetails));
    return result.result.value;
  }
  async function get(relative) {
    const response = await fetch(new URL(relative, base));
    assert(response.ok, `${relative}: HTTP ${response.status}`);
    return response;
  }
  const index = await (await get('search-index.json')).json();
  const foreign = index.items.find(row => (row[6] & 2) !== 0 && row[2] !== 'module');
  assert(foreign, 'No foreign item in the site');
  const foreignPath = `api/${index.modules[foreign[0]]}/${foreign[7] || foreign[1].replace(/[^A-Za-z0-9_]/g, c => '-' + c.charCodeAt(0).toString(16).padStart(2, '0'))}`;
  const home = await (await get('')).text();
  const publication = home.match(/property="og:url" content="([^"]+)"/)?.[1];
  if (publication && new URL(publication).origin !== base.origin) {
    publicationOrigin = new URL(publication).origin;
    await cdp('Fetch.enable', { patterns: [{ urlPattern: publicationOrigin + '/*', requestStage: 'Request' }, { urlPattern: new URL('404.html', base).href, requestStage: 'Request' }] });
  }
  const guidesPath = home.match(/>API<\/a><a href="([^"]+)">/)?.[1];
  assert(guidesPath, 'No guides link');
  const guides = await (await get(guidesPath)).text();
  const guide = guides.match(/<a class="card" href="([^"]+)"/)?.[1];
  assert(guide, 'No guide page');
  const pages = ['', 'api/String', 'api/Basic/print', foreignPath, guide, '404.html'];
  await cdp('Page.enable');
  await cdp('Runtime.enable');
  let failures = 0, runs = 0;
  for (const page of pages) {
    await get(page);
    for (const theme of ['light', 'dark']) {
      for (const width of [1280, 320]) {
        const url = new URL(page, base).href;
        await cdp('Emulation.setDeviceMetricsOverride', { width, height: 900, deviceScaleFactor: 1, mobile: false });
        // Match the OS scheme to the theme, so nothing is caught mid-transition between the two.
        await cdp('Emulation.setEmulatedMedia', { features: [{ name: 'prefers-color-scheme', value: theme }] });
        const before = loads; scriptErrors = [];
        const navigation = await cdp('Page.navigate', { url });
        assert(!navigation.errorText, navigation.errorText);
        for (let i = 0; i < 200; i++) {
          if (loads > before && await evaluate(`location.href === ${JSON.stringify(url)} && document.readyState === 'complete' && !!document.querySelector('main')`)) break;
          await delay(50);
        }
        assert(loads > before && await evaluate(`location.href === ${JSON.stringify(url)} && document.readyState === 'complete'`), 'Page did not load');
        await evaluate(`document.documentElement.setAttribute('data-theme', ${JSON.stringify(theme)})`);
        await delay(400); // colour transitions settle
        await evaluate(axe);
        const result = await evaluate(`axe.run(document, {runOnly:{type:'tag', values:${JSON.stringify(tags)}}}).then(r => ({violations:r.violations.map(v => ({id:v.id, nodes:v.nodes.map(n => ({target:n.target, summary:n.failureSummary}))})), incomplete:r.incomplete.length}))`);
        const overflow = await evaluate('document.documentElement.scrollWidth > innerWidth');
        const h1 = await evaluate('document.querySelectorAll("h1").length');
        runs++;
        const issues = result.violations.length + Number(overflow) + Number(h1 !== 1) + scriptErrors.length;
        failures += issues;
        console.log(JSON.stringify({ page: page || '/', theme, width, violations: result.violations, overflow, h1, scriptErrors, incomplete: result.incomplete }));
      }
    }
  }
  assert.deepEqual(interceptErrors, [], "Local asset interception failed");
  console.log(`axe: ${runs} page/theme/width checks, ${failures} failures`);
  if (failures) process.exitCode = 1;
} finally {
  socket?.close(); child.kill();
  await new Promise(resolve => !child.pid || child.exitCode !== null || child.signalCode !== null ? resolve() : child.once('exit', resolve));
  await rm(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
}
