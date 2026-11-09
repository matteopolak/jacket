import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { test } from 'node:test';
import vm from 'node:vm';

const site = await readFile(
  new URL('../../crates/jacket/src/doc/site/assets/site.js', import.meta.url),
  'utf8',
);

/** A search-index row: module index, name, kind, summary, signature, shapes, flags. */
type Row = [number, string, string, string, string, unknown[], number];

/** Rows for `item`s: the index `jacket doc` writes, with the native flag set where asked. */
function rows(entries: [string, string, string?, string?, boolean?][]) {
  const modules = [...new Set(entries.map((e) => e[0]))];
  const items: Row[] = entries.map(([module, name, kind = 'type', summary = '', native = false]) => [
    modules.indexOf(module),
    name,
    kind,
    summary,
    '',
    [],
    native ? 2 : 0,
  ]);
  return { modules, items };
}

/** `load` then `byName` of site.js over `index`, as the paths `Module.name` found, in order. */
async function search(index: ReturnType<typeof rows>, query: string): Promise<string[]> {
  const piece = (from: string, to: string) => {
    const start = site.indexOf(from);
    const end = site.indexOf(to, start + from.length);
    assert(start >= 0 && end > start, `site.js has ${from.trim()}`);
    return site.slice(start, end);
  };
  const context = {
    base: '',
    fetch: async () => ({ json: async () => index }),
    query,
    found: null as Promise<{ title: string; name: string }[]> | null,
  };
  vm.runInNewContext(
    `var items, cats, loading;\n${piece('  function load()', '  /** @param {string} type */')}\n` +
      `${piece('  function byName(', '  /**\n   * The nearest element')}\nfound = load().then(function () { return byName(query); });`,
    context,
  );
  return Array.from(await context.found!, (it) => (it.name ? `${it.title}.${it.name}` : it.title));
}

const index = rows([
  ['Extensions/Postgres', 'Time'],
  ['Socket', 'timeval32'],
  ['Extensions/Time', 'time_of_day', 'proc'],
  ['Extensions/Time', 'Time_Of_Day'],
  ['Extensions/Time', '', 'module', 'Exact instants, civil values, explicit zones and calendar-aware spans.'],
  ['Basic', 'current_time_monotonic', 'proc'],
  ['Socket', 'Timeval', 'type', '', true],
]);

test('a module whose name is the query comes first', async () => {
  assert.equal((await search(index, 'time'))[0], 'Time');
});

test('a name that starts with the query as a whole word beats one that only shares the letters', async () => {
  const order = await search(index, 'time');
  assert(order.indexOf('Time.time_of_day') < order.indexOf('Socket.timeval32'), order.join(', '));
});

test('an exact name still beats a word prefix', async () => {
  const order = await search(index, 'time');
  assert(order.indexOf('Postgres.Time') < order.indexOf('Time.time_of_day'), order.join(', '));
});

test('a word inside a name beats letters inside a word', async () => {
  const words = rows([['A', 'runtime'], ['B', 'current_time']]);
  assert.deepEqual(await search(words, 'time'), ['B.current_time', 'A.runtime']);
});

test('the module name narrows a query of two terms', async () => {
  assert.equal((await search(index, 'time of_day'))[0], 'Time.time_of_day');
});
