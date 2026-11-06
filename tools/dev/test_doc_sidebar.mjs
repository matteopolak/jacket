import assert from 'node:assert/strict';
import { test } from 'node:test';
import { page } from '../../website/docs/tests/helpers/sidebar.mjs';

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
  p.window.jacketFinishSidebar();
  assert(p.observer.disconnected);
  assert.equal(p.side.open, false);
  assert.equal(core.open, true);
  p.side.open = true;
  core.open = false;
  p.storage.set('jacket-docs-side', JSON.stringify({ k: 'different', o: ['Core'], t: 0 }));
  p.window.jacketFinishSidebar();
  assert.equal(p.side.open, true);
  assert.equal(core.open, false);
  p.storage.set('jacket-docs-side', JSON.stringify({ k: 'modules', o: ['Core'], t: 0 }));
  p.window.jacketBootstrap.prepare(p.document);
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
  p.window.jacketFinishSidebar();
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

// Model the attached completed main and its streamed outline, as emitted by ModulePage.
function outlinePage(marks) {
  const classes = () => {
    const values = new Set();
    return { contains: name => values.has(name), add: name => values.add(name),
      toggle(name, enabled) { if (enabled) values.add(name); else values.delete(name); } };
  };
  const attrs = new Map();
  const button = { setAttribute: (name, value) => attrs.set(name, value), getAttribute: name => attrs.get(name) ?? null };
  const head = { classList: classes(), querySelector: () => button };
  head.classList.add('has-list');
  const group = { classList: classes(), previousElementSibling: head };
  head.nextElementSibling = group;
  const anchor = { getAttribute: () => '#types', closest: () => head };
  const toc = { querySelectorAll: selector => selector === 'li.items' ? [group] : [anchor] };
  const wrap = { querySelector: selector => selector === ':scope > .toc' ? toc : null, querySelectorAll: () => marks };
  const p = page(true);
  const original = p.document.querySelector.bind(p.document);
  p.document.querySelector = selector => selector === '.wrap' ? wrap : original(selector);
  return { ...p, group, button, toc };
}

test('the early outline selects its first section when all headings are below the viewport', () => {
  const p = outlinePage([{ id: 'types', getBoundingClientRect: () => ({ top: 1200 }) }]);
  p.window.jacketFinishLayout();
  assert(p.group.classList.contains('open'), 'early and deferred outlines must select the same first section');
  assert.equal(p.button.getAttribute('aria-expanded'), 'true');
  p.toc.querySelectorAll = () => [];
  assert.doesNotThrow(() => p.window.jacketFinishLayout(), 'an empty outline has no section to select');
});

test('an outline without completed main marks cannot select a provisional parser prefix', () => {
  const p = outlinePage([]);
  p.window.jacketFinishLayout();
  assert.equal(p.group.classList.contains('open'), false);
  assert.equal(p.button.getAttribute('aria-expanded'), null);
});
