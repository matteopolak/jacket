#!/usr/bin/env node
// No npm install: one hash-pinned axe-core distribution and Node's built-in CDP WebSocket.
// axe-core 4.10.3 predates the dependency-age policy's 14-day minimum. No transitive packages.
import assert from 'node:assert/strict';
import { firstGuidePath, pageStatus } from './doc_samples.ts';
import { waitForDevToolsPort } from './chrome_startup.ts';
import { Connection, exceptionText } from './cdp.ts';
import { createHash } from 'node:crypto';
import { spawn, spawnSync } from 'node:child_process';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

/** One item of search-index.json: module index, name, kind, ..., flags, ..., path override. */
type SearchRow = [number, string, string, unknown, unknown, unknown, number, string?];

/** What the in-page axe call returns, as far as this script reads it. */
interface AxeRun {
  violations: { id: string; nodes: unknown[] }[];
  incomplete: number;
}

const AXE_URL = 'https://unpkg.com/axe-core@4.10.3/axe.min.js';
const AXE_SHA256 = '880970c081707360e64f34cea25ff91892f5bc95675b0776925b9709dd8a68bb';
const tags = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'];
const base = new URL(process.argv[2] || 'http://127.0.0.1:8000/');
const cache = path.join(tmpdir(), 'jacket-axe-4.10.3.min.js');
let axe: string;
try {
  axe = await readFile(cache, 'utf8');
} catch {
  axe = await (await fetch(AXE_URL)).text();
}
assert.equal(createHash('sha256').update(axe).digest('hex'), AXE_SHA256, 'axe-core integrity');
await writeFile(cache, axe);
const chrome =
  process.env.CHROME_PATH ||
  (process.platform === 'darwin'
    ? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'
    : ['google-chrome', 'google-chrome-stable', 'chromium', 'chromium-browser'].find(
        (p) => spawnSync('which', [p]).status === 0,
      ));
assert(chrome, 'Install Chrome or set CHROME_PATH');
const profile = await mkdtemp(path.join(tmpdir(), 'jacket-a11y-'));
const child = spawn(
  chrome,
  [
    '--headless=new',
    '--no-sandbox',
    '--disable-gpu',
    '--no-first-run',
    '--remote-debugging-port=0',
    `--user-data-dir=${profile}`,
    'about:blank',
  ],
  { stdio: ['ignore', 'ignore', 'pipe'] },
);
let devtools: Connection | undefined;
try {
  const port = await waitForDevToolsPort(child, profile);
  const targets: { type: string; webSocketDebuggerUrl: string }[] = await (
    await fetch(`http://127.0.0.1:${port}/json/list`)
  ).json();
  const target = targets.find((t) => t.type === 'page');
  assert(target, 'Chrome opened no page target');
  const connection = await new Connection(target.webSocketDebuggerUrl).open();
  devtools = connection;
  const cdp = <T = unknown>(method: string, params?: Record<string, unknown>) =>
    connection.cdp<T>(method, params);
  const evaluate = <T = unknown>(expression: string) => connection.evaluate<T>(expression);
  const interceptErrors: string[] = [];
  let publicationOrigin = '',
    loads = 0,
    scriptErrors: string[] = [];
  connection.listeners.push(async (message) => {
    if (message.method === 'Page.loadEventFired') {
      loads++;
      return;
    }
    if (message.method === 'Runtime.exceptionThrown') {
      scriptErrors.push(exceptionText(message));
      return;
    }
    if (message.method === 'Fetch.requestPaused') {
      const { requestId, request, resourceType } = message.params ?? {};
      assert(request, 'Fetch.requestPaused carries no request');
      try {
        // 404 links use the publication origin. Serve those requests from this build,
        // including stylesheet and search fetches, rather than testing deployed assets.
        const remote = new URL(request.url);
        const response = await fetch(new URL(remote.pathname + remote.search, base));
        let body = Buffer.from(await response.arrayBuffer());
        if (resourceType === 'Document' && publicationOrigin)
          body = Buffer.from(body.toString('utf8').replaceAll(publicationOrigin, base.origin));
        await cdp('Fetch.fulfillRequest', {
          requestId,
          responseCode: response.status,
          responseHeaders: [
            {
              name: 'Content-Type',
              value: response.headers.get('content-type') || 'application/octet-stream',
            },
            { name: 'Access-Control-Allow-Origin', value: '*' },
          ],
          body: body.toString('base64'),
        });
      } catch (error) {
        interceptErrors.push(String(error));
        await cdp('Fetch.failRequest', { requestId, errorReason: 'Failed' });
      }
      return;
    }
  });
  async function get(relative: string) {
    const response = await fetch(new URL(relative, base));
    assert(response.ok, `${relative}: HTTP ${response.status}`);
    return response;
  }
  const index: {
    items: SearchRow[];
    modules: string[];
  } = await (await get('search-index.json')).json();
  const foreign = index.items.find((row) => (row[6] & 2) !== 0 && row[2] !== 'module');
  assert(foreign, 'No foreign item in the site');
  const foreignPath = `api/${index.modules[foreign[0]]}/${foreign[7] || foreign[1].replace(/[^A-Za-z0-9_]/g, (c) => '-' + c.charCodeAt(0).toString(16).padStart(2, '0'))}`;
  const home = await (await get('')).text();
  const publication = home.match(/property="og:url" content="([^"]+)"/)?.[1];
  if (publication && new URL(publication).origin !== base.origin) {
    publicationOrigin = new URL(publication).origin;
    await cdp('Fetch.enable', {
      patterns: [
        { urlPattern: publicationOrigin + '/*', requestStage: 'Request' },
        { urlPattern: new URL('404.html', base).href, requestStage: 'Request' },
      ],
    });
  }
  const guide = firstGuidePath(home);
  const pages = ['', 'api/String', 'api/Basic/print', foreignPath, guide, '404.html'];
  await cdp('Page.enable');
  await cdp('Runtime.enable');
  let failures = 0,
    runs = 0;
  for (const page of pages) {
    assert.equal(await pageStatus(new URL(page, base)), 200, `${page}: HTTP status`);
    for (const theme of ['light', 'dark']) {
      for (const width of [1280, 320]) {
        const url = new URL(page, base).href;
        await cdp('Emulation.setDeviceMetricsOverride', {
          width,
          height: 900,
          deviceScaleFactor: 1,
          mobile: false,
        });
        // Match the OS scheme to the theme, so nothing is caught mid-transition between the two.
        await cdp('Emulation.setEmulatedMedia', {
          features: [{ name: 'prefers-color-scheme', value: theme }],
        });
        const before = loads;
        scriptErrors = [];
        const navigation = await cdp<{ errorText?: string }>('Page.navigate', { url });
        assert(!navigation.errorText, navigation.errorText);
        for (let i = 0; i < 200; i++) {
          if (
            loads > before &&
            (await evaluate(
              `location.href === ${JSON.stringify(url)} && document.readyState === 'complete' && !!document.querySelector('main')`,
            ))
          )
            break;
          await delay(50);
        }
        assert(
          loads > before &&
            (await evaluate(
              `location.href === ${JSON.stringify(url)} && document.readyState === 'complete'`,
            )),
          'Page did not load',
        );
        await evaluate(
          `document.documentElement.setAttribute('data-theme', ${JSON.stringify(theme)})`,
        );
        await delay(400); // colour transitions settle
        assert.equal(
          await evaluate(`document.querySelector('#search-q')?.labels?.[0]?.textContent.trim()`),
          'Search the docs',
          'Search input accessible name',
        );
        await evaluate(axe);
        const runAxe = () =>
          evaluate<AxeRun>(
            `axe.run(document, {runOnly:{type:'tag', values:${JSON.stringify(tags)}}}).then(r => ({violations:r.violations.map(v => ({id:v.id, nodes:v.nodes.map(n => ({target:n.target, summary:n.failureSummary}))})), incomplete:r.incomplete.length}))`,
          );
        const result = await runAxe();
        // On a phone the sidebar is a drawer; check it open too, then close it with the button.
        if (width < 900 && page !== '404.html') {
          assert(
            await evaluate(`!!document.getElementById('modules-btn')`),
            'The Modules button is missing',
          );
          await evaluate(`document.getElementById('modules-btn').click()`);
          await delay(400);
          assert(
            await evaluate(`document.querySelector('dialog.side').matches(':modal')`),
            'The drawer did not open',
          );
          const open = await runAxe();
          for (const v of open.violations)
            result.violations.push({ ...v, id: `drawer open: ${v.id}` });
          result.incomplete += open.incomplete;
          await evaluate(`document.getElementById('modules-btn').click()`);
          await delay(300);
        }
        const overflow = await evaluate('document.documentElement.scrollWidth > innerWidth');
        const h1 = await evaluate('document.querySelectorAll("h1").length');
        runs++;
        const issues =
          result.violations.length + Number(overflow) + Number(h1 !== 1) + scriptErrors.length;
        failures += issues;
        console.log(
          JSON.stringify({
            page: page || '/',
            theme,
            width,
            violations: result.violations,
            overflow,
            h1,
            scriptErrors,
            incomplete: result.incomplete,
          }),
        );
      }
    }
  }
  assert.deepEqual(interceptErrors, [], 'Local asset interception failed');
  console.log(`axe: ${runs} page/theme/width checks, ${failures} failures`);
  if (failures) process.exitCode = 1;
} finally {
  devtools?.close();
  child.kill();
  await new Promise<void>((resolve) =>
    !child.pid || child.exitCode !== null || child.signalCode !== null
      ? resolve()
      : child.once('exit', resolve),
  );
  await rm(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
}
