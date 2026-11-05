import assert from 'node:assert/strict';
import { test } from 'node:test';
import { anchoredTextParts, formattedTextLines, textParts, jsonLd } from '../src/lib/render.ts';
import { apiModule, declaration } from './fixtures.ts';
import type { Peer } from '../src/lib/render.ts';

test('resolved UTF-8 byte spans preserve exact Unicode text and target identity', () => {
  const target = { kind: 'page', id: 'Example/Entry', anchor: null } as const;
  const parts = textParts({ source: 'é Entry(雪)', spans: [
    { start: 0, end: 2, role: 'declared_name', target: null },
    { start: 3, end: 8, role: 'type', target },
  ] });
  assert.equal(parts.map(part => part.text).join(''), 'é Entry(雪)');
  assert.equal(parts[0]?.role, 'declared_name');
  assert.equal(parts[2]?.text, 'Entry');
  assert.deepEqual(parts[2]?.target, target);
});

test('invalid Unicode boundaries and overlapping spans are rejected before markup', () => {
  for (const spans of [
    [{ start: 1, end: 2, role: 'plain', target: null }],
    [{ start: 0, end: 2, role: 'plain', target: null }, { start: 1, end: 2, role: 'plain', target: null }],
    [{ start: 0, end: 100, role: 'plain', target: null }],
    [{ start: -1, end: 0, role: 'plain', target: null }],
  ] as const) {
    assert.throws(() => textParts({ source: 'é', spans: [...spans] }));
  }
});

test('structured metadata serializes only supported facts and cannot terminate its script', () => {
  const hostile = '</script><script>alert("owned")</script>&\u2028';
  const serialized = jsonLd([
    { kind: 'tech_article', headline: hostile, description: 'Literal <text>', url: null,
      website_name: 'Owned docs', website_url: null, module_name: 'Example' },
    { kind: 'breadcrumb_list', entries: [{ position: 1, name: hostile, url: null }] },
  ]);
  assert(!/[<>&\u2028\u2029]/u.test(serialized));
  const data: unknown = JSON.parse(serialized);
  assert.deepEqual(data, [
    { '@context': 'https://schema.org', '@type': 'TechArticle', headline: hostile,
      description: 'Literal <text>', isPartOf: { '@type': 'WebSite', name: 'Owned docs' },
      about: { '@type': 'SoftwareSourceCode', name: 'Example' } },
    { '@context': 'https://schema.org', '@type': 'BreadcrumbList', itemListElement: [
      { '@type': 'ListItem', position: 1, name: hostile },
    ] },
  ]);
  assert.equal(jsonLd([]), '[]');
});

test('item peer windows preserve kind, binding, group and bounded fallback geometry', async () => {
  const { peerWindow } = await import('../src/lib/render.ts');
  const peer = (id: string, extra: Partial<Peer> = {}): Peer => ({ id, kind: 'procedure', binding: false, group: '', private: false, ...extra });
  const peers = Array.from({ length: 50 }, (_, i) => peer(`p${i}`));
  const all = [...peers, peer('foreign', { binding: true }), peer('type', { kind: 'struct' }), peer('grouped', { group: 'Other' }), peer('hidden', { private: true })];
  const window = peerWindow(all, peers[25]!);
  assert.equal(window.peers.length, 50);
  assert.deepEqual([window.start, window.end, window.first, window.last, window.before, window.after], [5, 45, 23, 28, 18, 17]);
  assert.deepEqual(window.visible.map(item => item.id), ['p23', 'p24', 'p25', 'p26', 'p27']);
  const small = peerWindow(peers.slice(0, 4), peers[0]!);
  assert.equal(small.visible.length, 4);
  const hidden = peerWindow(all, all.at(-1)!);
  assert.equal(hidden.visible.length, 0);
  assert.equal(hidden.before + hidden.after, 0);
});

test('category CSS preserves theme channels without accepting executable CSS input', async () => {
  const { categoryCSS } = await import('../src/lib/render.ts');
  const category = { id: 'category-0', name: 'Library', badge: 'owned', tip: '', color: '#123456', darkColor: '#abcdef' };
  const css = categoryCSS([category]);
  assert(css.includes('.cat-0{--c:#123456}'));
  assert(css.includes(':root[data-theme="dark"] .cat-0{--c:#abcdef}'));
  assert(css.includes('@media (prefers-color-scheme: dark)'));
  assert.throws(() => categoryCSS([{ ...category, color: '#123456}</style><script>alert(1)</script>' }]));
  assert.throws(() => categoryCSS([{ ...category, darkColor: 'red' }]));
  assert.equal(categoryCSS([{ ...category, badge: '' }]), '');
});

test('module outlines keep grouped item lists and omit a lone section outline', async () => {
  const { moduleOutline } = await import('../src/lib/api.ts');
  const module=apiModule([declaration('run','procedure')]);
  assert.deepEqual(moduleOutline(module), []);
  module.declarations.push(declaration('Entry', 'struct'));
  assert.deepEqual(moduleOutline(module), [
    { kind: 'heading', id: 'procedures', label: 'Procedures', sub: false },
    { kind: 'items', items: [{ id: 'run', label: 'run' }] },
    { kind: 'heading', id: 'types', label: 'Types', sub: false },
    { kind: 'items', items: [{ id: 'Entry', label: 'Entry' }] },
  ]);
});


test('formatter text receives exact UTF-8 member anchors without syntax reflow',()=>{
  const source='Pair :: struct {\n    café, second: string = "keep  spaces";\n}\n';
  const before=new TextEncoder().encode(source.slice(0,source.indexOf('café'))).length;
  const value={source,spans:[]};
  const parts=anchoredTextParts(value,[{id:'m-café',start:before,end:before+5}]);
  assert.equal(parts.map(part=>part.text).join(''),source);
  assert.equal(parts.find(part=>part.anchors.includes('m-café'))?.text.startsWith('café'),true);
  assert.throws(()=>anchoredTextParts(value,[{id:'bad',start:before+4,end:before+5}]));
  assert.throws(()=>anchoredTextParts(value,[{id:'same',start:before,end:before+5},{id:'same',start:before,end:before+5}]),/invalid definition anchor/);
});


test('formatter line presentation preserves detail bytes and literal blank lines in compact overviews',()=>{
  const source='Record :: struct {\n    // field prose\n\n    text := #string END\nfirst\n\nlast\nEND; // trailing prose\n}\n';
  const comment=source.indexOf('// field'),trailing=source.indexOf('// trailing'),literal=source.indexOf('#string'),literalEnd=source.indexOf('END;')+3;
  const value={source,spans:[{start:comment,end:source.indexOf('\n',comment),role:'comment' as const,target:null},{start:literal,end:literalEnd,role:'string' as const,target:null},{start:trailing,end:source.indexOf('\n',trailing),role:'comment' as const,target:null}]};
  const start=source.indexOf('text');
  const full=formattedTextLines(value,[{id:'m-text',start,end:start+4}]);
  const joined=(lines:ReturnType<typeof formattedTextLines>)=>lines.map(line=>line.parts.map(part=>part.text).join('')+(line.newline?'\n':'')).join('');
  assert.equal(joined(full),source);
  assert.equal(full.findIndex(line=>line.parts.some(part=>part.anchors.includes('m-text'))),3);
  const compact=joined(formattedTextLines(value,[],false));
  assert(!compact.includes('field prose')&&!compact.includes('trailing prose'));
  assert(compact.includes('#string END\nfirst\n\nlast\nEND; '),'literal-contained blank lines must remain');
  assert(compact.startsWith('Record :: struct {\n    text'));
});


test('actual platform badge policy preserves the retired model platform assertions',async()=>{
  const api=await import('../src/lib/api.ts');assert.equal(typeof api.platformRows,'function');
  const open=declaration('open'),map=declaration('map'),all=declaration('all');
  map.platforms=['WINDOWS'];all.platforms=['MACOS','LINUX','WINDOWS','WASM'];const module=apiModule([open,map,all]);
  const supported=()=>api.platformRows(module).filter(row=>row.state!=='none').map(row=>row.name);
  const missing=(item:typeof map)=>api.platformRows(module,item).filter((row,index)=>api.platformRows(module)[index]!.state!=='none'&&row.state==='none').map(row=>row.name);
  assert.deepEqual(supported(),['macOS','Linux','Windows']);assert.deepEqual(missing(open),[]);assert.deepEqual(missing(map),['macOS','Linux']);
  module.wasm=true;assert.deepEqual(supported(),['macOS','Linux','Windows','Wasm']);assert.deepEqual(missing(all),[]);assert.deepEqual(missing(map),['macOS','Linux','Wasm']);
  const rows=api.platformRows(module,map);assert.equal(rows[2]!.state,'full');assert.equal(rows[0]!.state,'none');
  assert.equal(rows[2]!.tip,'map works on Windows');assert.equal(rows[0]!.tip,'map is not available on macOS');
  assert.equal(api.platformRows(module)[0]!.tip,'1 item is not available on macOS');
  map.private=true;assert(api.platformRows(module).every(row=>row.state==='full'),'private helpers changed public module support');
});

test('platform badges keep explicit target order and unknown wasm status',async()=>{
  const api=await import('../src/lib/api.ts');assert.equal(typeof api.platformRows,'function');const module=apiModule();module.platforms=['IOS','WINDOWS','ANDROID'];
  assert.deepEqual(api.platformRows(module).map(row=>[row.name,row.state]),[['macOS','none'],['Linux','none'],['Windows','full'],['Android','full'],['iOS','full'],['Wasm','none']]);
  module.wasm=true;assert.equal(api.platformRows(module).at(-1)!.state,'full');
});


test('navigation request identity follows emitted facts rather than module docs or source',async()=>{
  const {moduleNavigation}=await import('../src/lib/navigation-path.ts');
  const module=apiModule([declaration('Entry','struct')]),original=moduleNavigation(module);
  assert.equal(moduleNavigation(module),original,'one imported module shares one serialized projection');
  const docsOnly=structuredClone(module);docsOnly.metadata.title='Different title';docsOnly.sourceUrl='https://source.test/new';docsOnly.declarations[0]!.metadata.description='New prose';
  assert.equal(moduleNavigation(docsOnly).sha256,original.sha256);
  const changed=structuredClone(docsOnly);changed.declarations[0]!.experimental=true;
  assert.notEqual(moduleNavigation(changed).sha256,original.sha256);
  assert.equal(JSON.parse(original.json).data.entries.length,1);
});
