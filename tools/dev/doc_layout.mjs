#!/usr/bin/env node
// Checks that a served Jacket documentation site does not move while it loads.
//
//   node tools/dev/doc_layout.mjs http://127.0.0.1:8000/            fixed sample of page kinds
//   node tools/dev/doc_layout.mjs http://127.0.0.1:8000/ --all      every page in sitemap.xml
//
// Options: --parallel N (tabs), --only SUBSTRING (page filter), --every N (with --all, every Nth page),
// --widths 1280,375, --script-delay MS (hold site.js back so a frame paints first; 0 turns it off),
// --verbose (list every scenario), --dump (print every recorded frame).
// Time: about 3.5 s per page/width/theme with 3 tabs, so --all (about 7,600 pages) takes over a day;
// use --every 50 for a spread of about 150 pages in 40 minutes.
// No npm install: Node's built-in WebSocket speaks the Chrome DevTools Protocol to headless Chrome
// (CHROME_PATH overrides discovery).
//
// For each page, width and colour theme it runs: a fresh load; a reload after scrolling partway;
// leaving and coming Back; and in-page swaps by the site's navigator (a sidebar link, the header's
// API link, each also on its own page, then Back and Forward). Every scenario fails on
//   - any layout-shift entry with a value above zero that no input caused, and
//   - any frame whose key element boxes (title, platforms, outline, folds and their buttons,
//     header, both sidebars, scroll positions) differ from where they settle.
// A page that reaches the screen in one shape and then rearranges itself is exactly what the
// second check catches, including changes too small or too quick for the first.
import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

const args = process.argv.slice(2);
const flag = name => args.includes(name);
const option = (name, fallback) => { const i = args.indexOf(name); return i >= 0 ? args[i + 1] : fallback; };
const positional = args.filter((a, i) => !a.startsWith('--') && !(i > 0 && ['--parallel', '--only', '--widths', '--script-delay', '--every'].includes(args[i - 1])));
const base = new URL(positional[0] || 'http://127.0.0.1:8000/');
const all = flag('--all');
const verbose = flag('--verbose');
const only = option('--only', '');
const widths = option('--widths', '1280,375').split(',').map(Number);
const parallel = Math.max(1, Number(option('--parallel', all ? 3 : 2)));
const themes = ['light', 'dark'];
const SETTLE_MS = 450;
// site.js is deferred; on a real connection the page is painted before it runs. Hold it back so
// that frame exists here too (0 lets the local server run it before the first paint).
const SCRIPT_DELAY_MS = Number(option('--script-delay', 250));

// ---------------------------------------------------------------------------------------------
// Runs in every document before its own scripts. Records, once per animation frame, the boxes of
// the elements that matter (only when they changed), and every layout-shift entry.

const RECORDER = String.raw`(() => {
  if (window.__rec) return;
  const frames = [], shifts = [], errors = [];
  let prev = '', number = 0;
  const round = v => Math.round(v * 10) / 10;
  // Document coordinates for content, viewport coordinates for the sticky and fixed bars.
  const box = el => {
    if (!el) return null;
    const r = el.getBoundingClientRect();
    let viewport = false;
    for (let n = el; n && n !== document.body && !viewport; n = n.parentElement) viewport = ['sticky', 'fixed'].includes(getComputedStyle(n).position);
    if (!r.width && !r.height) return null;
    return [r.left + (viewport ? 0 : scrollX), r.top + (viewport ? 0 : scrollY), r.width, r.height].map(round);
  };
  const list = (sel, root = document, cap = 60) =>
    Array.from(root.querySelectorAll(sel)).slice(0, cap).map(box).filter(b => b !== null);
  const describe = node => {
    if (!node || !node.tagName) return String(node);
    let s = node.tagName.toLowerCase();
    if (node.id) s += '#' + node.id;
    if (node.classList && node.classList.length) s += '.' + Array.from(node.classList).join('.');
    return s;
  };
  function snap() {
    const body = document.body;
    if (!body) return null;
    const side = document.querySelector('details.side');
    const current = side && side.querySelector('nav [aria-current="page"]');
    const toc = document.querySelector('.toc');
    const main = document.getElementById('main-content');
    return {
      page: body.getAttribute('data-page') || '',
      loading: document.readyState === 'loading',
      y: Math.round(scrollY),
      // Keys under "early" exist as soon as their element is parsed; the rest only matter once the page is whole.
      early: {
        header: box(document.querySelector('header.top')),
        search: box(document.querySelector('.search-wrap')),
        siteNav: list('.site-nav > *'),
        side: box(side),
        sideOpen: side ? side.open : null,
        sideScroll: side ? Math.round(side.scrollTop) : null,
        sideGroups: side ? side.querySelectorAll('details.nav-cat[open]').length : null,
        sideCurrent: current && side ? round(current.getBoundingClientRect().top - side.getBoundingClientRect().top) : null,
        crumbs: box(document.querySelector('main .crumbs')),
        titlebar: box(document.querySelector('main .titlebar')),
        h1: box(document.querySelector('main h1')),
        platforms: box(document.querySelector('main .platforms')),
      },
      late: {
        main: box(main),
        sideHeight: side ? side.scrollHeight : null,
        toc: box(toc),
        tocOpen: toc ? toc.querySelectorAll('li.items.open').length : null,
        tocToggles: toc ? list('.tog', toc) : null,
        folds: list('main .fold'),
        examples: list('main figure.ex'),
        codeBlocks: list('main pre'),
        foldsOpen: main ? Array.from(main.querySelectorAll('.fold')).map(f => f.classList.contains('open') ? 1 : 0).join('') : null,
        foldButtons: list('main .fold-btn'),
        headings: list('main h2, main h3'),
        height: document.documentElement.scrollHeight,
      },
    };
  }
  function loop() {
    try {
      const s = snap();
      if (s) {
        const key = JSON.stringify(s);
        if (key !== prev) { prev = key; s.frame = number; s.t = Math.round(performance.now()); frames.push(s); }
      }
    } catch (e) { errors.push(String(e)); }
    number++;
    requestAnimationFrame(loop);
  }
  const take = e => {
    if (e.hadRecentInput || !(e.value > 0)) return;
    shifts.push({
      value: e.value, t: Math.round(e.startTime),
      sources: (e.sources || []).map(s => ({
        node: describe(s.node),
        from: [s.previousRect.x, s.previousRect.y, s.previousRect.width, s.previousRect.height].map(round),
        to: [s.currentRect.x, s.currentRect.y, s.currentRect.width, s.currentRect.height].map(round),
      })),
    });
  };
  let observer;
  try {
    observer = new PerformanceObserver(l => l.getEntries().forEach(take));
    observer.observe({ type: 'layout-shift', buffered: true });
  } catch (e) { errors.push('layout-shift: ' + e); }
  window.__rec = {
    frames, shifts, errors,
    frameCount: () => number,
    // Forget everything so far; what follows is one scenario.
    clear() { if (observer) observer.takeRecords(); frames.length = 0; shifts.length = 0; prev = ''; },
    read() { if (observer) observer.takeRecords().forEach(take); return { frames, shifts, errors, number }; },
  };
  requestAnimationFrame(loop);
})();`;

// ---------------------------------------------------------------------------------------------

function findChrome() {
  if (process.env.CHROME_PATH) return process.env.CHROME_PATH;
  if (process.platform === 'darwin') return '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
  return ['google-chrome', 'google-chrome-stable', 'chromium', 'chromium-browser'].find(p => spawnSync('which', [p]).status === 0);
}

class Connection {
  constructor(url) { this.url = url; this.next = 0; this.pending = new Map(); this.listeners = []; }
  async open() {
    this.socket = new WebSocket(this.url);
    await new Promise((resolve, reject) => { this.socket.onopen = resolve; this.socket.onerror = reject; });
    this.socket.onmessage = event => {
      const message = JSON.parse(event.data);
      const entry = this.pending.get(message.id);
      if (entry) {
        this.pending.delete(message.id); clearTimeout(entry.timer);
        if (message.error) entry.reject(new Error(`${entry.method}: ${JSON.stringify(message.error)}`)); else entry.resolve(message.result);
      } else if (message.method) for (const l of this.listeners) l(message);
    };
    return this;
  }
  cdp(method, params = {}) {
    return new Promise((resolve, reject) => {
      const id = ++this.next;
      const timer = setTimeout(() => { this.pending.delete(id); reject(new Error(`${method} timed out`)); }, 60_000);
      this.pending.set(id, { resolve, reject, timer, method });
      this.socket.send(JSON.stringify({ id, method, params }));
    });
  }
  close() { this.socket?.close(); }
}

class Tab {
  static async open(browser, port) {
    const { targetId } = await browser.cdp('Target.createTarget', { url: 'about:blank', newWindow: true });
    const tab = new Tab(browser, targetId, await new Connection(`ws://127.0.0.1:${port}/devtools/page/${targetId}`).open());
    tab.conn.listeners.push(m => {
      if (m.method === 'Page.loadEventFired') tab.loads++;
      if (m.method === 'Fetch.requestPaused') {
        setTimeout(() => tab.cdp('Fetch.continueRequest', { requestId: m.params.requestId }).catch(() => {}), SCRIPT_DELAY_MS);
      }
      if (m.method === 'Runtime.exceptionThrown') {
        const text = m.params.exceptionDetails.exception?.description || m.params.exceptionDetails.text;
        // Headless Chrome aborts cross-document view transitions it never shows; not the site's doing.
        if (!/ViewTransition|Transition was aborted/.test(text)) tab.exceptions.push(text);
      }
    });
    tab.loads = 0; tab.exceptions = [];
    await tab.cdp('Page.enable'); await tab.cdp('Runtime.enable');
    // Visible and focused, so requestAnimationFrame runs at full rate like it does for a reader.
    await tab.cdp('Emulation.setFocusEmulationEnabled', { enabled: true });
    // Every load goes to the server, so the held-back script below is held back on every load.
    await tab.cdp('Network.enable'); await tab.cdp('Network.setCacheDisabled', { cacheDisabled: true });
    await tab.cdp('Page.addScriptToEvaluateOnNewDocument', { source: RECORDER });
    if (SCRIPT_DELAY_MS > 0) await tab.cdp('Fetch.enable', { patterns: [{ urlPattern: '*/assets/site.js*', requestStage: 'Request' }] });
    return tab;
  }
  constructor(browser, id, conn) { this.browser = browser; this.id = id; this.conn = conn; }
  cdp(method, params) { return this.conn.cdp(method, params); }
  async evaluate(expression) {
    const result = await this.cdp('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true });
    assert(!result.exceptionDetails, JSON.stringify(result.exceptionDetails));
    return result.result.value;
  }
  async configure(width, theme) {
    await this.cdp('Emulation.setDeviceMetricsOverride', { width, height: 900, deviceScaleFactor: 1, mobile: false });
    // The OS scheme follows the theme, so nothing is caught mid-transition between the two.
    await this.cdp('Emulation.setEmulatedMedia', { features: [{ name: 'prefers-color-scheme', value: theme }] });
    await this.cdp('Storage.clearDataForOrigin', { origin: base.origin, storageTypes: 'local_storage,session_storage' });
    // A new size reaches matchMedia a moment later; a load right away can still see the old one.
    await this.evaluate('new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)))');
    await delay(100);
  }
  // Runs `trigger`, which must start a document load, and waits for the new document to be whole.
  async load(trigger) {
    const before = this.loads;
    this.exceptions = [];
    const result = await trigger();
    if (result?.errorText) throw new Error(result.errorText);
    for (let i = 0; i < 400 && this.loads === before; i++) await delay(25);
    assert(this.loads > before, 'load event never fired');
    await this.settle();
  }
  goto(url) { return this.load(() => this.cdp('Page.navigate', { url })); }
  reload() { return this.load(() => this.cdp('Page.reload')); }
  async history(offset) {
    const { currentIndex, entries } = await this.cdp('Page.getNavigationHistory');
    const entry = entries[currentIndex + offset];
    assert(entry, 'no history entry to go to');
    await this.load(() => this.cdp('Page.navigateToHistoryEntry', { entryId: entry.id }));
  }
  // The page is quiet: the sidebar's long lists have arrived and nothing has changed for a while.
  async settle() {
    for (let i = 0; i < 160; i++) {
      if (await this.evaluate(`document.readyState === 'complete' && !document.querySelector('details.side [data-source]')`)) break;
      await delay(25);
    }
    await delay(SETTLE_MS);
  }
  read() { return this.evaluate('window.__rec ? window.__rec.read() : null'); }
  close() { this.conn.close(); return this.browser.cdp('Target.closeTarget', { targetId: this.id }).catch(() => {}); }
}

// ---------------------------------------------------------------------------------------------
// Comparing frames.

const TOLERANCE = 0.6;
function differs(a, b, strict = false) {
  if (a === b) return false;
  if (typeof a === 'number' && typeof b === 'number') return Math.abs(a - b) > TOLERANCE;
  if (Array.isArray(a) && Array.isArray(b)) {
    // A list may have grown while the page was still being parsed; what was shown must not have moved.
    if (strict && a.length !== b.length) return true;
    return a.some((v, i) => i >= b.length || differs(v, b[i], strict));
  }
  return true;
}

// Every frame of a run (the same page shown) must match the last one, apart from lists that grew.
// Reports the first frame that differs, and how many others do.
function compare(run, { ignoreY = false, label }) {
  const final = run[run.length - 1];
  let first = null, more = 0;
  for (const f of run.slice(0, -1)) {
    const diffs = [];
    const groups = [['early', f.early, final.early]];
    // While the document is still being parsed, only what comes before the main text is whole.
    if (!f.loading) groups.push(['late', f.late, final.late]);
    for (const [, a, b] of groups) {
      for (const key of Object.keys(a)) {
        if (a[key] === null && f.loading) continue;
        // A sticky box sits a little differently at another scroll position; that is not a move.
        if ((key === 'side' || key === 'toc') && differs(f.y, final.y)) continue;
        if (differs(a[key], b[key], !f.loading)) diffs.push(`${key} was ${JSON.stringify(a[key])?.slice(0, 120)}, settles at ${JSON.stringify(b[key])?.slice(0, 120)}`);
      }
    }
    // A paint during parsing is before the page can put the scroll back; that is not checked.
    if (!ignoreY && !f.loading && differs(f.y, final.y)) diffs.push(`scroll was ${f.y}, settles at ${final.y}`);
    if (!diffs.length) continue;
    if (!first) first = { f, diffs }; else more++;
  }
  if (!first) return [];
  const shown = first.diffs.slice(0, 8).map(d => `${label}: frame ${first.f.frame} (${first.f.t}ms): ${d}`);
  if (first.diffs.length > 8) shown.push(`${label}: ...and ${first.diffs.length - 8} more differences in that frame`);
  if (more) shown.push(`${label}: ${more} more frame(s) before it settles differ too`);
  return shown;
}

function analyse(rec, options = {}) {
  const problems = [];
  if (!rec) return ['no recorder in the page'];
  if (rec.errors.length) problems.push(`recorder errors: ${rec.errors.join('; ')}`);
  if (!rec.frames.length) return [...problems, `no frames recorded (${rec.number} animation frames ran)`];
  // A swapped-in page of another height moves the footer, which a real click's own 500ms
  // exemption would hide; the footer alone is not counted for a swap.
  const counted = rec.shifts.filter(s => !(options.swap && s.sources.length && s.sources.every(x => x.node.startsWith('footer'))));
  for (const s of counted.slice(0, 4)) {
    const where = s.sources.slice(0, 4).map(x => `${x.node} ${x.from.join(',')} -> ${x.to.join(',')}`).join('; ');
    problems.push(`layout shift ${s.value.toFixed(4)} at ${s.t}ms: ${where}`);
  }
  if (flag('--dump')) {
    let last = null;
    for (const f of rec.frames) {
      const flat = { page: f.page, loading: f.loading, y: f.y, ...Object.fromEntries(Object.entries({ ...f.early, ...f.late }).map(([k, v]) => [k, JSON.stringify(v)])) };
      const changed = Object.keys(flat).filter(k => !last || last[k] !== flat[k]).map(k => `${k}=${last ? `${last[k]} -> ` : ''}${flat[k]}`.slice(0, 160));
      console.log(`      frame ${f.frame} @${f.t}ms: ${changed.join(' | ')}`);
      last = flat;
    }
  }
  // A run is consecutive frames showing one page.
  let run = [];
  const runs = [];
  for (const f of rec.frames) {
    if (run.length && run[0].page !== f.page) { runs.push(run); run = []; }
    run.push(f);
  }
  runs.push(run);
  for (const r of runs) problems.push(...compare(r, { ...options, label: r[0].page || '(page)' }));
  return problems;
}

// ---------------------------------------------------------------------------------------------
// Scenarios.

const click = (selectorExpression) => `(() => {
  const a = ${selectorExpression};
  if (!a) return null;
  window.__swapMarker = true;
  window.__rec.clear();
  a.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true, button: 0, detail: 1, view: window }));
  return { href: a.getAttribute('href'), same: a.href.split('#')[0] === location.href.split('#')[0] };
})()`;
const SIDE_OTHER = `(() => {
  const cur = document.querySelector('details.side nav [aria-current="page"]');
  const links = Array.from(document.querySelectorAll('details.side nav a[href]')).filter(a => /(\\/|\\.html)$/.test(a.pathname) && !a.href.includes('#'));
  const i = links.indexOf(cur);
  return links.find((a, n) => n > i && a !== cur && a.getAttribute('aria-current') !== 'page') || links.find(a => a !== cur);
})()`;
const SIDE_SELF = `document.querySelector('details.side nav [aria-current="page"]')`;
const HEADER_API = `Array.from(document.querySelectorAll('.site-nav > a')).find(a => a.textContent.trim() === 'API')`;

async function afterClick(tab, name, expression, options = {}) {
  const start = await tab.evaluate('location.href');
  const clicked = await tab.evaluate(click(expression));
  if (clicked === null) return { name, skipped: 'no such link' };
  const before = clicked.href;
  const self = options.self || clicked.same;
  // The address changes together with the swap; a link to the page itself changes nothing.
  for (let i = 0; i < 160 && !self; i++) {
    if (await tab.evaluate(`location.href !== ${JSON.stringify(start)}`).catch(() => false)) break;
    await delay(25);
  }
  await tab.settle();
  const problems = [];
  if (!await tab.evaluate('window.__swapMarker === true')) {
    problems.push(`the click on ${before} loaded a new document instead of swapping`);
    return { name, problems };
  }
  problems.push(...analyse(await tab.read(), { ignoreY: self, swap: true }));
  return { name, problems, detail: before, swapped: !self };
}

async function historyStep(tab, name, direction) {
  await tab.evaluate(`window.__rec.clear(); window.__swapMarker = true; history.${direction}()`);
  await delay(200);
  await tab.settle();
  const problems = [];
  if (!await tab.evaluate('window.__swapMarker === true')) problems.push('Back/Forward loaded a new document instead of swapping');
  problems.push(...analyse(await tab.read(), { swap: true }));
  return { name, problems };
}

async function runCombo(tab, page, width, theme) {
  const url = new URL(page, base).href;
  const results = [];
  const record = (name, problems, extra = {}) => results.push({ name, problems, ...extra });
  const guard = async (name, fn) => {
    try { await fn(); } catch (e) { record(name, [`error: ${e.message}`]); }
  };
  await tab.configure(width, theme);
  const frames = rec => rec ? rec.frames.length : 0;

  await guard('load', async () => {
    await tab.goto(url);
    const rec = await tab.read();
    const problems = analyse(rec);
    if (tab.exceptions.length) problems.push(`script errors: ${tab.exceptions.join('; ')}`);
    record('load', problems, { frames: frames(rec) });
  });

  await guard('reload', async () => {
    const max = await tab.evaluate('document.documentElement.scrollHeight - innerHeight');
    const y = Math.min(Math.floor(max / 2), 1500);
    if (max >= 40) {
      await tab.evaluate(`window.scrollTo({ top: ${y}, behavior: 'instant' })`);
      // site.js saves the position into history.state a little after scrolling stops.
      let saved = false;
      for (let i = 0; i < 60 && !saved; i++) { await delay(50); saved = await tab.evaluate(`!!(history.state && history.state.y > 0)`); }
      assert(saved, 'history.state never recorded the scroll');
    }
    await tab.reload();
    const rec = await tab.read();
    const problems = analyse(rec);
    const at = await tab.evaluate('Math.round(scrollY)');
    if (max >= 40 && Math.abs(at - y) > 2) problems.push(`reload did not restore the scroll: ${y} -> ${at}`);
    record('reload', problems, { frames: frames(rec), note: max >= 40 ? `scrolled to ${y}` : 'not scrollable' });
  });

  await guard('back', async () => {
    const other = page === '' ? 'guides/testing/' : '';
    await tab.goto(new URL(other, base).href);
    await tab.history(-1);
    const rec = await tab.read();
    const problems = analyse(rec);
    if (tab.exceptions.length) problems.push(`script errors: ${tab.exceptions.join('; ')}`);
    record('back', problems, { frames: frames(rec) });
  });

  // In-page swaps start from a fresh load of the page.
  const swaps = async (prefix, expression, selfExpression) => {
    await guard(prefix, async () => {
      await tab.goto('about:blank');
      await tab.goto(url);
      const first = await afterClick(tab, `${prefix} link`, expression);
      if (first.skipped) { record(first.name, [], { note: `skipped: ${first.skipped}` }); return; }
      record(first.name, first.problems, { note: first.detail + (first.swapped ? '' : ', the page itself') });
      const same = await afterClick(tab, `${prefix} link on its own page`, selfExpression, { self: true });
      if (same.skipped) record(same.name, [], { note: `skipped: ${same.skipped}` }); else record(same.name, same.problems);
      if (!first.swapped) return;
      const back = await historyStep(tab, `${prefix}: Back`, 'back');
      record(back.name, back.problems);
      const forward = await historyStep(tab, `${prefix}: Forward`, 'forward');
      record(forward.name, forward.problems);
    });
  };
  await swaps('sidebar', SIDE_OTHER, SIDE_SELF);
  await swaps('header API', HEADER_API, HEADER_API);
  return results;
}

// ---------------------------------------------------------------------------------------------

async function pagesToCheck() {
  const get = async relative => { const r = await fetch(new URL(relative, base)); return r; };
  const sitemap = await get('sitemap.xml');
  assert(sitemap.ok, `sitemap.xml: HTTP ${sitemap.status}`);
  const root = base.pathname;
  // The sitemap names the publication origin; only its paths matter here.
  const listed = [...(await sitemap.text()).matchAll(/<loc>([^<]+)<\/loc>/g)].map(m => {
    const p = new URL(m[1]).pathname;
    return p.startsWith(root) ? p.slice(root.length) : p.replace(/^\//, '');
  });
  const known = new Set(listed);
  if (all) {
    const every = Math.max(1, Number(option('--every', 1)));
    return { pages: [...listed, '404.html'].filter((p, i) => i % every === 0 && p.includes(only)), notes: [] };
  }
  const notes = [];
  const sample = [
    ['home', ''],
    ['guide', listed.find(p => p.startsWith('guides/'))],
    ['one-page rules (Format)', 'format-rules/'],
    ['lint rules', 'lint-rules/'],
    ['tool page', listed.find(p => p.startsWith('tools/') && p !== 'tools/')],
    ['small module', 'api/Base64/'],
    ['big module (Basic)', 'api/Basic/'],
    ['big module (Io)', 'api/Extensions/Io/'],
    ['item', 'api/Basic/print/'],
    ['private type', 'api/Extensions/Io/Waiter/'],
    ['404', '404.html'],
  ];
  const pages = [];
  for (const [kind, p] of sample) {
    if (p === undefined || (!known.has(p) && p !== '404.html' && !(await get(p)).ok)) { notes.push(`skipped ${kind}: ${p ?? '(none)'} is not in this site`); continue; }
    if (p.includes(only)) pages.push(p);
  }
  return { pages, notes };
}

const chrome = findChrome();
assert(chrome, 'Install Chrome or set CHROME_PATH');
const profile = await mkdtemp(path.join(tmpdir(), 'jacket-layout-'));
const child = spawn(chrome, ['--headless=new', '--no-sandbox', '--disable-gpu', '--no-first-run', '--remote-debugging-port=0',
  // Back and Forward must rebuild the page, as they do after the page leaves the browser's cache.
  '--disable-features=BackForwardCache', '--disable-renderer-backgrounding', '--disable-background-timer-throttling',
  '--disable-backgrounding-occluded-windows', `--user-data-dir=${profile}`, 'about:blank'], { stdio: ['ignore', 'ignore', 'pipe'] });
let launchError, chromeErrors = '';
child.on('error', e => { launchError = e; });
child.stderr.on('data', d => { chromeErrors = (chromeErrors + d).slice(-8000); });
let browser;
const started = Date.now();
try {
  let port;
  for (let i = 0; i < 200; i++) {
    assert(!launchError, String(launchError));
    assert(child.exitCode === null && child.signalCode === null, `Chrome exited: ${child.exitCode ?? child.signalCode}: ${chromeErrors}`);
    try { port = (await readFile(path.join(profile, 'DevToolsActivePort'), 'utf8')).split('\n')[0]; break; } catch { await delay(50); }
  }
  assert(port, 'Chrome did not start');
  const version = await (await fetch(`http://127.0.0.1:${port}/json/version`)).json();
  browser = await new Connection(version.webSocketDebuggerUrl).open();

  const { pages, notes } = await pagesToCheck();
  assert(pages.length, 'no pages to check');
  for (const n of notes) console.log(`note: ${n}`);
  const combos = [];
  for (const page of pages) for (const width of widths) for (const theme of themes) combos.push({ page, width, theme });
  console.log(`checking ${pages.length} pages x ${widths.length} widths x ${themes.length} themes with ${parallel} tab(s)`);

  let failures = 0, checks = 0, done = 0, skipped = 0;
  const failed = [];
  const queue = [...combos];
  const worker = async () => {
    const tab = await Tab.open(browser, port);
    try {
      while (queue.length) {
        const { page, width, theme } = queue.shift();
        let results;
        try { results = await runCombo(tab, page, width, theme); } catch (e) { results = [{ name: 'setup', problems: [`error: ${e.message}`] }]; }
        const bad = results.filter(r => r.problems.length);
        checks += results.length; failures += bad.length; skipped += results.filter(r => r.note?.startsWith('skipped')).length;
        done++;
        const head = `[${done}/${combos.length}] ${page || '/'} ${width}px ${theme}`;
        if (!bad.length && !verbose) console.log(`${head}: ok (${results.length} scenarios)`);
        else {
          console.log(`${head}: ${bad.length ? 'FAIL' : 'ok'}`);
          for (const r of results) {
            console.log(`    ${r.problems.length ? 'FAIL' : 'ok  '} ${r.name}${r.note ? ` (${r.note})` : ''}${r.frames !== undefined ? ` [${r.frames} distinct frame(s)]` : ''}`);
            for (const p of r.problems) console.log(`         ${p}`);
          }
        }
        if (bad.length) failed.push(head);
      }
    } finally { await tab.close(); }
  };
  await Promise.all(Array.from({ length: Math.min(parallel, combos.length) }, worker));
  const seconds = ((Date.now() - started) / 1000).toFixed(1);
  console.log(`layout: ${combos.length} page/width/theme runs, ${checks} scenarios, ${failures} failed, ${skipped} skipped, ${seconds}s`);
  for (const f of failed) console.log(`  failed: ${f}`);
  if (failures) process.exitCode = 1;
} finally {
  browser?.close(); child.kill();
  await new Promise(resolve => !child.pid || child.exitCode !== null || child.signalCode !== null ? resolve() : child.once('exit', resolve));
  await rm(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
}
