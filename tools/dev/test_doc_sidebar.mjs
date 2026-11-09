import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { test } from 'node:test';
import vm from 'node:vm';

const source = await readFile(new URL('../../crates/jacket/src/doc/site/assets/early.js', import.meta.url), 'utf8');

function page(wide, key = 'modules') {
  let sidebar = null, observer;
  const nav = { getAttribute: name => name === 'data-nav' ? key : null };
  const groups = [];
  const side = {
    nodeType: 1, open: true, scrollTop: 0,
    matches: selector => selector === 'dialog.side',
    querySelector: selector => selector === 'nav' ? nav : null,
    querySelectorAll: () => groups,
  };
  const group = name => {
    const node = { nodeType: 1, open: false, matches: selector => selector === 'details.nav-cat',
      closest: () => nav, getAttribute: attribute => attribute === 'data-key' ? name : null };
    groups.push(node);
    return node;
  };
  class Observer {
    constructor(callback) { this.callback = callback; this.pending = []; this.disconnected = false; observer = this; }
    observe() {}
    takeRecords() { const records = this.pending; this.pending = []; return records; }
    disconnect() { this.disconnected = true; }
  }
  const context = { window: {}, MutationObserver: Observer,
    document: { documentElement: {}, querySelector: () => sidebar },
    matchMedia: () => ({ matches: wide }),
    sessionStorage: { getItem: () => JSON.stringify({ k: 'modules', o: ['Core'], t: 19 }) } };
  vm.runInNewContext(source, context);
  assert(observer, 'head initialization observes nodes before the sidebar exists');
  sidebar = side;
  return { side, group, window: context.window, observer };
}

test('new parser nodes receive viewport and saved group state before rendering', () => {
  for (const wide of [false, true]) {
    const p = page(wide), core = p.group('Core'), other = p.group('Other');
    p.observer.callback([{ addedNodes: [p.side, core, other] }]);
    assert.equal(p.side.open, wide);
    assert.equal(core.open, true);
    assert.equal(other.open, false);
  }
});

test('sidebar completion drains records, disconnects, and preserves reader changes', () => {
  const p = page(false), core = p.group('Core');
  p.observer.pending = [{ addedNodes: [p.side, core] }];
  p.window.jacketSide();
  assert(p.observer.disconnected);
  assert.equal(p.side.open, false);
  assert.equal(core.open, true);
  p.side.open = true;
  core.open = false;
  p.window.jacketSide({ k: 'different', o: ['Core'], t: 0 });
  assert.equal(p.side.open, true);
  assert.equal(core.open, false);
  p.window.jacketSide({ k: 'modules', o: ['Core'], t: 0 });
  assert.equal(core.open, true, 'client swaps restore matching groups synchronously');
});

test('a saved group state is not applied to a different navigation list', () => {
  const p = page(true, 'item-list'), core = p.group('Core');
  p.observer.callback([{ addedNodes: [p.side, core] }]);
  assert.equal(core.open, false);
});

test('completing parsing does not reset a menu the reader already changed', () => {
  const p = page(false), core = p.group('Core');
  p.observer.callback([{ addedNodes: [p.side, core] }]);
  p.side.open = true;
  core.open = false;
  p.window.jacketSide();
  assert.equal(p.side.open, true);
  assert.equal(core.open, false);
});

test('later parser batches do not close a menu the reader opened', () => {
  const p = page(false);
  p.observer.callback([{ addedNodes: [p.side] }]);
  p.side.open = true;
  const core = p.group('Core');
  p.observer.callback([{ addedNodes: [core] }]);
  assert.equal(p.side.open, true);
  assert.equal(core.open, true);
});

test('the early outline selects its first section when all headings are below the viewport', () => {
  const classes = () => {
    const values = new Set();
    return { contains: name => values.has(name), add: name => values.add(name) };
  };
  const button = { setAttribute() {} };
  const head = { classList: classes(), textContent: 'Types', appendChild() {}, querySelector: () => button };
  const group = { classList: classes(), previousElementSibling: head, querySelector: () => ({}) };
  head.nextElementSibling = group;
  const anchor = { getAttribute: () => '#types', closest: () => head };
  const toc = { querySelectorAll: selector => selector === 'li.items' ? [group] : [anchor] };
  const context = {
    window: { innerHeight: 900 }, matchMedia: () => ({ matches: true }),
    sessionStorage: { getItem: () => null },
    MutationObserver: class { observe() {} },
    document: { documentElement: {}, querySelector: selector => selector === '.toc' ? toc : null,
      querySelectorAll: () => [], createElement: () => button,
      getElementById: () => ({ getBoundingClientRect: () => ({ top: 1200 }) }) },
  };
  vm.runInNewContext(source, context);
  context.window.jacketPage(false);
  assert(group.classList.contains('open'), 'early and deferred outlines must select the same first section');
  toc.querySelectorAll = () => [];
  assert.doesNotThrow(() => context.window.jacketPage(false), 'an empty outline has no section to select');
});

const site = await readFile(new URL('../../crates/jacket/src/doc/site/assets/site.js', import.meta.url), 'utf8');

// `setupDrawer` of site.js with a dialog that models open, modal and the asynchronous close event.
function drawerPage() {
  const start = site.indexOf('function setupDrawer(drawer) {');
  const end = site.indexOf('\n  function sideGroups', start);
  assert(start >= 0 && end > start, 'site.js has setupDrawer before sideGroups');
  const listeners = {};
  const media = { matches: false, change: null, addEventListener: (_, f) => { media.change = f; } };
  const slide = { release: null };
  const drawer = {
    open: false, modal: false,
    classes: new Set(),
    classList: {
      add: c => drawer.classes.add(c), remove: c => drawer.classes.delete(c),
      contains: c => drawer.classes.has(c),
    },
    matches: s => s === ':modal' && drawer.modal,
    querySelector: () => null,
    contains: () => false,
    getAnimations: () => drawer.classes.has('closing')
      ? [{ finished: new Promise(resolve => { slide.release = resolve; }) }] : [],
    getBoundingClientRect: () => ({ left: 0, right: 0, top: 0, bottom: 0 }),
    showModal() { drawer.open = true; drawer.modal = true; },
    show() { drawer.open = true; drawer.modal = false; },
    close() {
      drawer.open = false; drawer.modal = false;
      queueMicrotask(() => (listeners.close || []).forEach(f => f()));
    },
    addEventListener: (name, f) => { (listeners[name] ||= []).push(f); },
  };
  const button = {
    setAttribute() {}, focus() {}, contains: () => false,
    addEventListener: (name, f) => { listeners['button:' + name] = [f]; },
  };
  const context = {
    matchMedia: () => media,
    document: { getElementById: () => button, addEventListener() {} },
    root: { classList: { toggle() {} } },
    ancestor: () => null,
    drawer,
  };
  vm.runInNewContext(`${site.slice(start, end)}\nsetupDrawer(drawer);`, context);
  return { drawer, media, slide, click: () => listeners['button:click'][0]() };
}

const settle = () => new Promise(resolve => setTimeout(resolve, 0));

test('widening the window while the drawer slides away leaves the sidebar open', async () => {
  const p = drawerPage();
  p.click();
  assert(p.drawer.open && p.drawer.modal, 'the button opens the drawer as a modal');
  p.click();
  assert(p.drawer.classes.has('closing'), 'the drawer is sliding away');
  p.media.matches = true;
  p.media.change({ matches: true });
  p.slide.release?.();
  await settle();
  assert.equal(p.drawer.open, true, 'beside the page the sidebar is open');
  assert.equal(p.drawer.modal, false, 'and is no longer a modal');
});

test('widening the window with the drawer open leaves the sidebar open', async () => {
  const p = drawerPage();
  p.click();
  p.media.matches = true;
  p.media.change({ matches: true });
  await settle();
  assert.equal(p.drawer.open, true);
  assert.equal(p.drawer.modal, false);
});
