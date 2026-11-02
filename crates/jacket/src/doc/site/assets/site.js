// Jacket API docs. The pages read fine without this; it adds the theme button, search, the
// version menu, Copy, and Run (which compiles the example in your browser).
(function () {
  'use strict';
  var root = document.documentElement;
  var base = document.body.getAttribute('data-base') || '/';


  // Theme: system, light or dark, remembered in this browser.
  var KEY = 'jacket-docs-theme';
  var ORDER = ['system', 'light', 'dark'];
  var themeButton = document.getElementById('theme-toggle');
  function theme() { return root.getAttribute('data-theme') || 'system'; }
  function labelTheme() {
    if (!themeButton) return;
    var text = 'Theme: ' + theme() + ' (click to change)';
    themeButton.setAttribute('aria-label', text);
    themeButton.title = text;
    var icon = document.querySelector('#theme-icon use');
    if (icon) icon.setAttribute('href', icon.getAttribute('href').split('#')[0] + '#' + theme());
  }
  if (themeButton) {
    labelTheme();
    themeButton.addEventListener('click', function () {
      var next = ORDER[(ORDER.indexOf(theme()) + 1) % ORDER.length];
      root.setAttribute('data-theme', next);
      try {
        if (next === 'system') localStorage.removeItem(KEY); else localStorage.setItem(KEY, next);
      } catch (e) { /* private window or blocked storage: the choice lasts for this page only */ }
      labelTheme();
    });
  }

  // The side navigation starts closed on a phone. On a wide screen the blocking script right after
  // it has already put it back where it was. Its place and open groups are kept for the next page.
  var side = document.querySelector('details.side');
  if (side) {
    if (window.matchMedia('(max-width: 899px)').matches) side.open = false;
    window.addEventListener('pagehide', function () {
      try {
        var nav = side.querySelector('nav');
        var open = Array.prototype.map.call(side.querySelectorAll('details.nav-cat[open]'), function (d) { return d.getAttribute('data-key'); });
        sessionStorage.setItem('jacket-docs-side', JSON.stringify({ k: nav && nav.getAttribute('data-nav'), t: side.scrollTop, o: open }));
      } catch (e) { /* storage is blocked: the sidebar starts fresh */ }
    });
  }

  function sideGroups() {
    Array.prototype.forEach.call(document.querySelectorAll('.nav-cat'), function (d, i) {
      var summary = d.querySelector('summary'), list = d.querySelector('.nav-list');
      list.id = 'nav-group-' + i; summary.setAttribute('aria-controls', list.id);
      function state() { summary.setAttribute('aria-expanded', String(d.open)); }
      d.addEventListener('toggle', state); state();
    });
  }
  sideGroups();

  // The outline on the right marks every item on screen. Its item lists fold like the
  // sidebar's groups: lists with a section on screen are open, the rest closed, and a list
  // the reader opened stays open. Without JS every list is open. `signal` ends its listeners
  // when another page is swapped in.
  function outline(signal) {
  var toc = document.querySelectorAll('.toc a[href^="#"]');
  if (toc.length) {
    var marks = Array.prototype.map.call(toc, function (a) {
      return { a: a, h: document.getElementById(a.getAttribute('href').slice(1)) };
    }).filter(function (m) { return m.h; });
    // Each mark knows its group, so a scroll touches only what is on screen, never the whole
    // outline: big modules have thousands of entries.
    var groups = [];
    var setOpen = function (g) {
      var open = g.user === null ? g.seen : g.user;
      if (open === g.open) return;
      g.open = open;
      g.l.classList.toggle('open', open);
      g.b.setAttribute('aria-expanded', String(open));
      moved = true;
    };
    var groupOf = new Map();
    Array.prototype.forEach.call(document.querySelectorAll('.toc li.items'), function (l) {
      var head = l.previousElementSibling;
      if (!head || head.classList.contains('items')) return;
      var list = l.querySelector('ul');
      // early.js added the button and opened the lists on screen.
      var b = head.querySelector('.tog');
      if (!b) return;
      var open = l.classList.contains('open');
      var g = { l: l, b: b, user: null, seen: open, open: open };
      b.addEventListener('click', function () {
        g.user = !g.open;
        setOpen(g);
      });
      groupOf.set(head, g);
      Array.prototype.forEach.call(list.children, function (li) { groupOf.set(li, g); });
      groups.push(g);
    });
    marks.forEach(function (m) {
      var li = m.a.closest('li');
      m.g = groupOf.get(li) || null;
      m.item = !!(m.g && li.parentNode.parentNode === m.g.l);
      if (m.item) m.a.setAttribute('data-t', m.a.textContent);
    });
    var rail = document.querySelector('.toc');
    var from = -1, to = -1, first = true, moved = false;
    var still = window.matchMedia('(prefers-reduced-motion: reduce)');
    var pending = false;
    // The first mark whose heading is below y (marks follow the page order).
    var below = function (y) {
      var lo = 0, hi = marks.length;
      while (lo < hi) {
        var mid = (lo + hi) >> 1;
        if (marks[mid].h.getBoundingClientRect().top > y) hi = mid; else lo = mid + 1;
      }
      return lo;
    };
    // The rail scroll that puts entries a..b in its middle.
    var centre = function (a, b) {
      var top = marks[a].a.getBoundingClientRect().top, end = marks[b].a.getBoundingClientRect().bottom;
      var r = rail.getBoundingClientRect();
      return Math.round(rail.scrollTop + (top + end) / 2 - (r.top + r.bottom) / 2);
    };
    // Keep the marked entries in the middle of the rail, gliding there as the page scrolls.
    var glide = function (want) {
      if (Math.abs(want - rail.scrollTop) > 1) rail.scrollTo({ top: want, behavior: first || still.matches ? 'instant' : 'smooth' });
      first = false;
    };
    var mark = function () {
      pending = false;
      if (!marks.length) return;
      // Every section with some part on screen is marked. A section runs from its heading to
      // the next one in the outline; the sticky header covers the top 70px.
      var a = Math.max(0, below(70) - 1);
      var b = Math.max(a, below(window.innerHeight - 1) - 1);
      if (a === from && b === to) return;
      // Read where the rail should scroll before changing any class. Marking an entry never
      // moves the others, so these positions stay right, and the frame lays out only once.
      var want = centre(a, b);
      moved = false;
      var seen = [];
      var i;
      for (i = from; i >= 0 && i <= to; i++) {
        if (i < a || i > b) {
          marks[i].a.classList.remove('active');
          if (marks[i].g) marks[i].g.seen = false;
        }
      }
      if (from >= 0) marks[from].a.removeAttribute('aria-current');
      for (i = a; i <= b; i++) {
        // Only item names are coloured; headings just open their lists.
        if (marks[i].item) marks[i].a.classList.add('active');
        if (marks[i].g) { marks[i].g.seen = true; seen.push(marks[i].g); }
      }
      marks[a].a.setAttribute('aria-current', 'location');
      for (i = from; i >= 0 && i <= to; i++) {
        var g = marks[i].g;
        // A list the reader closed opens again once they have moved on and come back.
        if (g && !g.seen && g.user === false) g.user = null;
        if (g) setOpen(g);
      }
      seen.forEach(setOpen);
      from = a;
      to = b;
      // Animate folding only after the first layout, so a page load doesn't show lists closing.
      if (!rail.classList.contains('ready')) requestAnimationFrame(function () { rail.classList.add('ready'); });
      // A list opening or closing does move entries, so then the rail is placed a frame later.
      if (moved) requestAnimationFrame(function () { glide(centre(from, to)); });
      else glide(want);
    };
    window.addEventListener('scroll', function () {
      if (!pending) { pending = true; requestAnimationFrame(mark); }
    }, { passive: true, signal: signal });
    mark();
  }
  }

  // Search.
  var input = document.getElementById('search-q');
  var box = document.getElementById('search-results');
  var items = null;
  var cats = [];
  var loading = null;
  var hits = [];
  var picked = -1;
  var BUILTIN = {};
  'int float float32 float64 bool string void Type Any s8 s16 s32 s64 u8 u16 u32 u64 s128 u128 code Code'.split(' ').forEach(function (n) { BUILTIN[n] = true; });

  function dot(cls, label) {
    var d = el('span', 'dot ' + cls, '');
    d.title = label;
    d.setAttribute('role', 'img');
    d.setAttribute('aria-label', label); d.appendChild(el('span', 'sr', label));
    return d;
  }
  function hrefOf(it) {
    if (it.kind === 'module') return base + 'api/' + it.module + '/';
    if (it.folder) return base + 'api/' + it.module + '/' + it.folder + '/';
    return base + 'api/' + it.module + '/' + it.name.replace(/[^A-Za-z0-9_]/g, function (c) { return '-' + ('0' + c.charCodeAt(0).toString(16)).slice(-2); }) + '/';
  }
  function load() {
    if (!loading) {
      loading = fetch(base + 'search-index.json').then(function (r) { return r.json(); }).then(function (d) {
        cats = d.cats || [];
        items = d.items.map(function (r) {
          var module = d.modules[r[0]];
          var title = module.split('/').pop();
          return {
            module: module, title: title, name: r[1], kind: r[2], summary: r[3], sig: r[4], shapes: r[5], experimental: (r[6] & 1) === 1, native: (r[6] & 2) === 2, cat: (r[6] >> 2) - 1, folder: r[7],
            lname: r[1].toLowerCase(), lpath: (title + '.' + r[1]).toLowerCase(), lsum: r[3].toLowerCase()
          };
        });
        return items;
      });
    }
    return loading;
  }
  function norm(type) {
    return type.replace(/^[a-z_][a-z0-9_]*\s*:\s*/i, '').replace(/\s+/g, '').toLowerCase();
  }
  function typesOf(text) {
    text = text.trim().replace(/^\(/, '').replace(/\)$/, '');
    if (!text) return [];
    return text.split(',').map(norm).filter(Boolean);
  }
  function contains(have, wanted) {
    var rest = have.slice();
    return wanted.every(function (w) {
      var at = rest.indexOf(w);
      if (at < 0) return false;
      rest.splice(at, 1);
      return true;
    });
  }
  function bySignature(query) {
    var cut = query.indexOf('->');
    var want = typesOf(query.slice(0, cut));
    var ret = typesOf(query.slice(cut + 2));
    var out = [];
    items.forEach(function (it) {
      if (it.kind !== 'proc') return;
      var best = -1;
      it.shapes.forEach(function (s) {
        if (!contains(s[0], want) || !contains(s[1], ret)) return;
        var exact = (want.length === 0 || s[0].length === want.length ? 1 : 0) + (ret.length === 0 || s[1].length === ret.length ? 1 : 0);
        best = Math.max(best, exact);
      });
      if (best >= 0) out.push({ it: it, score: best });
    });
    out.sort(function (a, b) { return (a.it.native ? 1 : 0) - (b.it.native ? 1 : 0) || b.score - a.score || a.it.name.length - b.it.name.length || (a.it.lpath < b.it.lpath ? -1 : 1); });
    return out.map(function (o) { return o.it; });
  }
  function byName(query) {
    var terms = query.toLowerCase().split(/\s+/).filter(Boolean);
    var out = [];
    items.forEach(function (it) {
      var score = 0;
      for (var i = 0; i < terms.length; i++) {
        var t = terms[i];
        var s = 0;
        if (it.lname === t) s = 1000;
        else if (it.lname.indexOf(t) === 0) s = 800;
        else if (it.lname.indexOf(t) > 0) s = 600;
        else if (it.lpath.indexOf(t) >= 0) s = 400;
        else if (it.lsum.indexOf(t) >= 0) s = 100;
        if (!s) return;
        score += s;
      }
      if (it.kind === 'module') score += 50;
      // Native bindings come after the library's own API that matches as well.
      if (it.native) score = Math.floor(score / 4);
      out.push({ it: it, score: score });
    });
    out.sort(function (a, b) { return b.score - a.score || a.it.name.length - b.it.name.length || (a.it.lpath < b.it.lpath ? -1 : 1); });
    return out.map(function (o) { return o.it; });
  }
  function el(tag, cls, text) {
    var e = document.createElement(tag);
    if (cls) e.className = cls;
    if (text !== undefined) e.textContent = text;
    return e;
  }
  // A procedure's signature coloured like the one on its page: parameter names, builtin types,
  // and the library's types as links to the declarations the compiler resolved them to. The index
  // writes a link as \u0001Module/item\u0002Name\u0003.
  function sigNodes(it) {
    var out = document.createDocumentFragment();
    var text = it.sig;
    if (it.kind !== 'proc') { out.appendChild(document.createTextNode(text)); return out; }
    var re = /\u0001([^\u0002]*)\u0002([^\u0003]*)\u0003|"(?:[^"\\]|\\.)*"|[A-Za-z_][A-Za-z0-9_]*|\s+|[\s\S]/g;
    // Polymorphic names (`$T`) are types wherever they appear in the signature.
    var poly = {};
    text.replace(/\$([A-Za-z_][A-Za-z0-9_]*)/g, function (_, n) { poly[n] = true; return _; });
    var m, prev = '', plain = '';
    function flush() { if (plain) { out.appendChild(document.createTextNode(plain)); plain = ''; } }
    while ((m = re.exec(text))) {
      var t = m[0];
      if (m[1] !== undefined) {
        flush();
        var a = el('span', 'ty', m[2]);
        out.appendChild(a);
        prev = m[2];
        continue;
      }
      if (/^[A-Za-z_]/.test(t) && prev !== '.') {
        var named = /^\s*:/.test(text.slice(re.lastIndex)) && (prev === '(' || prev === ',' || prev === '$');
        var cls = named ? 'pname' : (BUILTIN[t] || poly[t]) ? 'ty' : '';
        if (cls) { flush(); out.appendChild(el('span', cls, t)); } else plain += t;
      } else {
        plain += t;
      }
      if (!/^\s+$/.test(t)) prev = t;
    }
    flush();
    return out;
  }
  function show() {
    box.textContent = '';
    if (!hits.length) {
      var empty = el('div', 'empty', 'No matches.'); empty.setAttribute('role', 'option'); empty.setAttribute('aria-disabled', 'true'); box.appendChild(empty);
    }
    hits.forEach(function (it, i) {
      var a = el('div', 'hit');
      a.id = 'hit-' + i;
      a.setAttribute('role', 'option');
      a.setAttribute('aria-selected', i === picked ? 'true' : 'false');
      var name = el('span', 'path', it.kind === 'module' ? it.title : it.title + '.' + it.name);
      a.setAttribute('aria-label', (it.kind === 'module' ? it.title : it.title + '.' + it.name) + ': ' + it.summary);
      a.appendChild(el('code', '', '')).appendChild(name);
      // Flags are dots here, coloured like the badges on the pages; the word is the tooltip.
      if (it.native) a.appendChild(dot('foreign', 'Foreign: a binding to a native library'));
      if (it.cat >= 0 && cats[it.cat]) a.appendChild(dot('cat-' + it.cat, cats[it.cat][1]));
      if (it.experimental) a.appendChild(dot('exp', 'Experimental: may change or go away'));
      // The signature column already says what procedures, types and constants are.
      if (it.kind === 'module') a.appendChild(el('span', 'what', 'module'));
      var sig = el('span', 'sig');
      sig.appendChild(sigNodes(it));
      a.appendChild(sig);
      box.appendChild(a);
    });
    var tips = el('div', 'tips');
    ['↑↓ to move', '↵ to open', 'type -> bool to search by return type'].forEach(function (t) { tips.appendChild(el('span', '', t)); });
    tips.setAttribute('role', 'option'); tips.setAttribute('aria-disabled', 'true'); box.appendChild(tips);
    box.hidden = false;
    input.setAttribute('aria-expanded', 'true');
    if (picked >= 0) input.setAttribute('aria-activedescendant', 'hit-' + picked); else input.removeAttribute('aria-activedescendant');
  }
  function hide() {
    box.hidden = true;
    input.setAttribute('aria-expanded', 'false');
    input.removeAttribute('aria-activedescendant');
  }
  function update() {
    var q = input.value.trim();
    if (!q) { hits = []; picked = -1; hide(); return; }
    load().then(function () {
      if (input.value.trim() !== q) return;
      hits = (q.indexOf('->') >= 0 ? bySignature(q) : byName(q)).slice(0, 12);
      picked = hits.length ? 0 : -1;
      show();
    });
  }
  if (input && box) {
    input.addEventListener('focus', load);
    input.addEventListener('input', update);
    input.addEventListener('keydown', function (e) {
      if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
        if (!hits.length) return;
        e.preventDefault();
        picked = (picked + (e.key === 'ArrowDown' ? 1 : hits.length - 1)) % hits.length;
        show();
      } else if (e.key === 'Enter') {
        if (picked >= 0 && hits[picked]) { e.preventDefault(); window.location.href = hrefOf(hits[picked]); }
      } else if (e.key === 'Escape') {
        e.preventDefault(); hide();
      }
    });
    box.addEventListener('click', function (e) {
      var row = e.target.closest ? e.target.closest('.hit') : null;
      if (row && !e.target.closest('a') && hits[+row.id.slice(4)]) window.location.href = hrefOf(hits[+row.id.slice(4)]);
    });
    document.addEventListener('click', function (e) {
      if (!box.contains(e.target) && e.target !== input) hide();
    });
    document.addEventListener('keydown', function (e) {
      if (e.key !== '/' || e.ctrlKey || e.metaKey || e.altKey) return;
      var t = e.target;
      if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.tagName === 'SELECT' || t.isContentEditable)) return;
      e.preventDefault();
      input.focus();
      input.select();
    });
  }

  // Copy and Run.
  var worker = null;
  var runtime = null;
  var texts = {};
  function runtimeUrls() {
    if (!runtime) {
      runtime = fetch(base + 'runtime.json').then(function (r) { return r.json(); }).then(function (c) {
        return { engine: new URL(c.engine, new URL(base, location.href).href).href, wasm: new URL(c.wasm, new URL(base, location.href).href).href };
      });
    }
    return runtime;
  }
  // An example's program is stored as the parts around its text (examples/<id>.json), so the
  // text a reader edited can be put back in. `fill` is `Template::fill` of doctest.rs.
  function parts(id) {
    if (!texts[id]) texts[id] = fetch(base + 'examples/' + id + '.json').then(function (r) { return r.json(); });
    return texts[id];
  }
  function fill(t, code) {
    var lines = code.replace(/\r/g, '').split('\n');
    if (lines[lines.length - 1] === '') lines.pop();
    var out = t.head;
    if (t.wrapped) {
      var body = [];
      lines.forEach(function (l) {
        if (/^\s*#(import|load)/.test(l)) out += l.replace(/^\s+/, '') + '\n'; else body.push(l);
      });
      out += t.middle;
      body.forEach(function (l) { out += (l !== '' || t.pad ? '    ' : '') + l + '\n'; });
    } else {
      lines.forEach(function (l) { out += l + '\n'; });
      out += t.middle;
    }
    return out + t.tail;
  }
  function base64url(text) {
    var bytes = new TextEncoder().encode(text), bin = '';
    for (var i = 0; i < bytes.length; i += 0x8000) bin += String.fromCharCode.apply(null, bytes.subarray(i, i + 0x8000));
    return btoa(bin).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
  }

  // Editable examples: a textarea sits over the highlighted code and shows through it. The code
  // is coloured again as you type, by the same rules as highlight.rs (minus the links).
  var KW = {}, TY = {};
  'if else then for while return defer using struct enum enum_flags union cast xx inline break continue case remove true false null ifx push_context context operator interface'.split(' ').forEach(function (w) { KW[w] = 'k'; });
  'int float float32 float64 bool string void Type Any s8 s16 s32 s64 u8 u16 u32 u64 u128 s128 assert'.split(' ').forEach(function (w) { TY[w] = 't'; });
  function escapeHtml(s) { return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;'); }
  function highlight(code) {
    var re = /\/\/[^\n]*|"(?:[^"\\]|\\[\s\S])*"?|#[A-Za-z_][A-Za-z0-9_]*|\d(?:[A-Za-z0-9_]|\.(?!\.))*|[A-Za-z_][A-Za-z0-9_]*|[\s\S]/g;
    return code.replace(re, function (m) {
      var cls = m.charAt(0) === '/' && m.charAt(1) === '/' ? 'c' : m.charAt(0) === '"' ? 's' : m.charAt(0) === '#' && m.length > 1 ? 'd' : /^\d/.test(m) ? 'n' : KW[m] || TY[m];
      return cls ? '<span class="' + cls + '">' + escapeHtml(m) + '</span>' : escapeHtml(m);
    });
  }
  function editable(figure) {
    var runButton = figure.querySelector('button[data-act="run"][data-program]');
    var pre = figure.querySelector('pre.code');
    if (!runButton || !pre || figure.querySelector('.ed')) return;
    var code = pre.querySelector('code');
    var original = pre.textContent;
    var originalHtml = code.innerHTML;
    var reset = figure.querySelector('button[data-act="reset"]');
    var open = figure.querySelector('a.btn[href]');
    var openHref = open && open.getAttribute('href');
    var wrap = el('div', 'ed');
    pre.parentNode.insertBefore(wrap, pre);
    wrap.appendChild(pre);
    var area = el('textarea');
    area.value = original;
    area.spellcheck = false;
    area.wrap = 'off';
    area.setAttribute('autocapitalize', 'off');
    area.setAttribute('autocomplete', 'off');
    area.setAttribute('aria-label', 'Example code, editable');
    wrap.appendChild(area);
    var escaped = false;
    function changed() {
      var text = area.value;
      var edited = text !== original;
      code.innerHTML = edited ? highlight(text) + (/\n$/.test(text) ? ' ' : '') : originalHtml;
      if (reset) reset.hidden = !edited;
      if (open) {
        if (!edited) open.setAttribute('href', openHref);
        else parts(runButton.getAttribute('data-program')).then(function (p) {
          if (p.open && area.value === text) open.setAttribute('href', p.playground + '#code=' + base64url(fill(p.open, text)));
        });
      }
    }
    function insert(text) {
      area.focus();
      if (!document.execCommand || !document.execCommand('insertText', false, text)) {
        area.setRangeText(text, area.selectionStart, area.selectionEnd, 'end');
        changed();
      }
    }
    area.addEventListener('input', changed);
    // On phones the textarea is drawn scaled (site.css, --k), so its scroll offsets are too.
    area.addEventListener('scroll', function () {
      var k = parseFloat(getComputedStyle(wrap).getPropertyValue('--k')) || 1;
      pre.scrollLeft = area.scrollLeft * k; pre.scrollTop = area.scrollTop * k;
    });
    area.addEventListener('keydown', function (e) {
      var plain = !e.ctrlKey && !e.metaKey && !e.altKey && !e.shiftKey;
      if (e.key === 'Tab' && plain && !escaped) {
        e.preventDefault();
        insert('    ');
      } else if (e.key === 'Enter' && plain) {
        // Keep the indentation of the line you were on.
        var before = area.value.slice(0, area.selectionStart);
        var indent = /^[ \t]*/.exec(before.slice(before.lastIndexOf('\n') + 1))[0];
        e.preventDefault();
        insert('\n' + indent);
      } else if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        if (!runButton.disabled) run(runButton);
      }
      // Escape lets the next Tab leave the box, so the keyboard is never stuck in it.
      escaped = e.key === 'Escape';
    });
    area.addEventListener('blur', function () { escaped = false; });
    if (reset) reset.addEventListener('click', function () {
      area.value = original;
      changed();
      area.focus();
    });
    figure.__text = function () { return area.value; };
  }
  function ask(request, limit) {
    return new Promise(function (resolve, reject) {
      if (!worker) worker = new Worker(base + 'assets/run-worker.js', { type: 'module' });
      var w = worker;
      var timer = setTimeout(function () {
        w.terminate();
        if (worker === w) worker = null;
        reject(new Error('Stopped after ' + limit / 1000 + ' seconds.'));
      }, limit);
      w.onmessage = function (e) {
        clearTimeout(timer);
        if (e.data.error) {
          if (e.data.crashed && worker === w) { w.terminate(); worker = null; }
          reject(new Error(e.data.error));
        } else {
          resolve(e.data);
        }
      };
      w.onerror = function (e) {
        clearTimeout(timer);
        if (worker === w) worker = null;
        reject(new Error(e.message || 'The compiler could not start in this browser.'));
      };
      w.postMessage(request);
    });
  }
  function report(out, state, status, text) {
    out.hidden = false;
    out.className = 'ex-out' + (state ? ' ' + state : '');
    out.textContent = '';
    out.appendChild(el('div', '', status));
    if (text) out.appendChild(el('pre', '', text));
  }
  function run(button) {
    var figure = button.closest('figure');
    var out = figure.querySelector('.ex-out');
    var id = button.getAttribute('data-program');
    button.disabled = true;
    report(out, 'busy', worker ? 'Compiling...' : 'Loading the compiler (the first run takes a moment)...');
    var text = figure.__text ? figure.__text() : figure.querySelector('pre.code').textContent;
    Promise.all([parts(id), runtimeUrls()]).then(function (got) {
      return ask({ program: fill(got[0].run, text), engine: got[1].engine, wasm: got[1].wasm }, 60000);
    }).then(function (done) {
      var r = done.result;
      var ms = Math.max(1, Math.round(done.ms));
      var printed = (r.stdout || '') + (r.stderr || '');
      if (r.exitCode === 0 && !(r.diagnostics || []).some(function (d) { return d.severity === 'error'; })) {
        report(out, '', '✓ Ran in ' + ms + ' ms', printed.trim() ? printed : '');
      } else {
        var why = r.rendered || printed || (r.exitCode !== null ? 'The example exited with status ' + r.exitCode + '.' : 'The example did not compile.');
        report(out, 'bad', '✗ Failed', why.trim());
      }
    }).catch(function (error) {
      report(out, 'bad', '✗ Could not run', String(error.message || error));
    }).then(function () { button.disabled = false; });
  }
  function copy(button) {
    var figure = button.closest('figure');
    var text = figure.__text ? figure.__text() : figure.querySelector('pre.code').textContent;
    var done = function () {
      var old = button.textContent;
      button.textContent = 'Copied';
      setTimeout(function () { button.textContent = old; }, 1500);
    };
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(done, function () {});
    } else {
      var area = document.createElement('textarea');
      area.value = text;
      document.body.appendChild(area);
      area.select();
      try { document.execCommand('copy'); done(); } catch (e) { /* nothing more to try */ }
      document.body.removeChild(area);
    }
  }
  document.addEventListener('click', function (e) {
    var button = e.target.closest && e.target.closest('button[data-act]');
    if (!button || button.disabled) return;
    if (button.getAttribute('data-act') === 'run') run(button); else if (button.getAttribute('data-act') === 'copy') copy(button);
  });

  // Clicking anywhere on an item card opens the item, except on links, buttons, examples and when selecting text.
  function cardLink(e) {
    var t = e.target;
    if (!t.closest || t.closest('a, button, summary, input, textarea, .ex')) return null;
    var row = t.closest('.row');
    var name = row && row.querySelector('.row-head a.name');
    if (!name) return null;
    var sel = window.getSelection && window.getSelection();
    if (sel && !sel.isCollapsed && row.contains(sel.anchorNode)) return null;
    return name;
  }
  document.addEventListener('click', function (e) {
    if (e.button !== 0) return;
    var name = cardLink(e);
    if (!name) return;
    if (e.metaKey || e.ctrlKey || e.shiftKey) window.open(name.href, '_blank'); else name.click();
  });
  document.addEventListener('auxclick', function (e) {
    var name = e.button === 1 && cardLink(e);
    if (name) window.open(name.href, '_blank');
  });

  // A link into folded members opens them and scrolls there.
  function openTarget() {
    var id = decodeURIComponent(location.hash.slice(1));
    var target = id ? document.getElementById(id) : null;
    if (!target) return;
    var opened = false;
    for (var node = target; node; node = node.parentElement) {
      if (node.tagName === 'DETAILS' && !node.open) { node.open = true; opened = true; }
      if (node.classList.contains('fold') && !node.classList.contains('open')) {
        node.classList.add('open');
        if (node.nextElementSibling && node.nextElementSibling.classList.contains('fold-btn')) node.nextElementSibling.remove();
        opened = true;
      }
    }
    if (opened) target.scrollIntoView({ behavior: matchMedia('(prefers-reduced-motion: no-preference)').matches ? 'smooth' : 'instant' });
  }
  window.addEventListener('hashchange', openTarget);

  // Version menu. versions.json is [{"version": "0.31.0", "url": "../0.31.0/"}, ...]; a relative url is read from versions.json's folder.
  (function () {
    var btn = document.getElementById('ver-button'), menu = document.getElementById('ver-menu');
    if (!btn || !menu) return;
    var listUrl = new URL(base + 'versions.json', location.href);
    fetch(listUrl).then(function (r) { return r.ok ? r.json() : []; }).then(function (list) {
      if (!Array.isArray(list) || list.length < 2) return;
      var current = btn.textContent.trim();
      list.forEach(function (v) {
        var a = document.createElement('a');
        a.href = new URL(v.url, listUrl).href; a.textContent = v.version; a.setAttribute('role', 'menuitem'); a.tabIndex = -1;
        if (v.version === current) a.setAttribute('aria-current', 'true');
        menu.appendChild(a);
      });
      btn.disabled = false;
    }).catch(function () {});
    function set(open) { menu.hidden = !open; btn.setAttribute('aria-expanded', open ? 'true' : 'false'); }
    function openAt(last) { set(true); var links = menu.querySelectorAll('a'); if (links.length) links[last ? links.length - 1 : 0].focus(); }
    btn.addEventListener('click', function (e) { e.stopPropagation(); if (menu.hidden) openAt(false); else set(false); });
    btn.addEventListener('keydown', function (e) {
      if (['ArrowDown', 'ArrowUp', 'Enter', ' '].indexOf(e.key) < 0) return;
      e.preventDefault(); openAt(e.key === 'ArrowUp');
    });
    menu.addEventListener('keydown', function (e) {
      if (e.key === ' ') { e.preventDefault(); document.activeElement.click(); return; }
      var links = Array.prototype.slice.call(menu.querySelectorAll('a'));
      var i = links.indexOf(document.activeElement);
      if (e.key === 'Tab') { set(false); btn.focus(); return; }
      if (['ArrowDown', 'ArrowUp', 'Home', 'End'].indexOf(e.key) < 0) return;
      e.preventDefault();
      var n = e.key === 'Home' ? 0 : e.key === 'End' ? links.length - 1 : (i + (e.key === 'ArrowUp' ? -1 : 1) + links.length) % links.length;
      links[n].focus();
    });
    document.addEventListener('click', function (e) { if (!menu.contains(e.target)) set(false); });
    document.addEventListener('keydown', function (e) { if (e.key === 'Escape' && !menu.hidden) { set(false); btn.focus(); } });
  })();

  // A code block wider than its column scrolls on its own; give it a tab stop so the keyboard
  // can scroll it too.
  // All widths are read before any tab stop changes, so this lays the page out once.
  var pres = [], sizing = false;
  function scrollable() {
    sizing = false;
    var wide = Array.prototype.map.call(pres, function (pre) { return pre.scrollWidth > pre.clientWidth; });
    Array.prototype.forEach.call(pres, function (pre, i) {
      if (wide[i]) pre.tabIndex = 0; else pre.removeAttribute('tabindex');
    });
  }
  window.addEventListener('resize', function () {
    if (!sizing) { sizing = true; requestAnimationFrame(scrollable); }
  });

  // A long block shows its top and a Show more button. early.js has already opened the blocks
  // that fit; the button is in the HTML, so nothing moves when this runs. Focus inside a block,
  // from Tab or a search result, opens it too, so a focused link is never hidden.
  function folds() {
  Array.prototype.forEach.call(document.querySelectorAll('main .fold'), function (fold, i) {
    var btn = fold.nextElementSibling;
    if (!btn || !btn.classList.contains('fold-btn')) return;
    if (fold.classList.contains('open')) { btn.remove(); return; }
    if (!fold.id) fold.id = 'fold-' + i;
    btn.setAttribute('aria-controls', fold.id);
    function open() { fold.classList.add('open'); btn.remove(); }
    btn.addEventListener('click', function () {
      // Keep keyboard focus in place of the removed button.
      fold.tabIndex = -1;
      open();
      fold.focus({ preventScroll: true });
    });
    fold.addEventListener('focusin', function () {
      if (!fold.classList.contains('open')) open();
    });
  });
  }

  // What each page sets up for its own content. It runs again when another page is swapped in.
  var page = null;
  function setupPage() {
    if (page) page.abort();
    page = new AbortController();
    outline(page.signal);
    Array.prototype.forEach.call(document.querySelectorAll('figure.ex'), editable);
    pres = document.querySelectorAll('main pre');
    scrollable();
    folds();
    openTarget();
  }


  setupPage();

  // Moving between pages of the site fetches the next page and swaps in its content, so the
  // header and sidebar stay where they are. A link is fetched once the pointer rests on it, so
  // most clicks find the page ready. Anything unexpected falls back to a normal page load.
  var pages = new Map();
  function jump(y) { window.scrollTo({ top: y, behavior: 'instant' }); }
  var siteRoot = new URL(base, location.href);
  function bare(href) { var u = new URL(href, location.href); u.hash = ''; return u.href; }
  function inSite(a) {
    if (!a || !a.href || (a.target && a.target !== '_self') || a.hasAttribute('download')) return null;
    var u = new URL(a.href, location.href);
    if (u.origin !== siteRoot.origin || u.pathname.indexOf(siteRoot.pathname) !== 0) return null;
    if (!/(\/|\.html)$/.test(u.pathname) || u.pathname.indexOf('/assets/') >= 0) return null;
    return u;
  }
  function fetchPage(href) {
    var key = bare(href);
    var got = pages.get(key);
    if (!got) {
      got = fetch(key).then(function (r) {
        if (!r.ok || !/html/.test(r.headers.get('content-type') || '')) throw new Error('not a page');
        return r.text();
      }).then(function (text) { return new DOMParser().parseFromString(text, 'text/html'); });
      got.catch(function () { pages.delete(key); });
      pages.set(key, got);
      // Keep the few most recent pages.
      if (pages.size > 40) pages.delete(pages.keys().next().value);
    }
    return got;
  }
  if (window.fetch && window.DOMParser && history.pushState) {
    history.scrollRestoration = 'manual';
    var rest = null;
    var hover = function (e) {
      var u = inSite(e.target.closest && e.target.closest('a[href]'));
      if (!u || bare(u.href) === bare(location.href)) return;
      clearTimeout(rest);
      rest = setTimeout(function () { fetchPage(u.href); }, e.type === 'pointerover' ? 65 : 0);
    };
    document.addEventListener('pointerover', hover, { passive: true });
    document.addEventListener('pointerout', function () { clearTimeout(rest); }, { passive: true });
    document.addEventListener('touchstart', hover, { passive: true });
    document.addEventListener('focusin', hover);

    var remember = function () {
      try { history.replaceState({ y: window.scrollY }, ''); } catch (e) { /* too many calls: skip one */ }
    };
    var saving = null;
    window.addEventListener('scroll', function () {
      clearTimeout(saving);
      saving = setTimeout(remember, 150);
    }, { passive: true });
    window.addEventListener('pagehide', remember);

    var going = 0;
    var go = function (href, push, keyboard) {
      var n = ++going;
      var slow = setTimeout(function () { root.classList.add('loading'); }, 150);
      fetchPage(href).then(function (doc) {
        if (n !== going) return;
        if (doc.body.getAttribute('data-base') !== document.body.getAttribute('data-base')) throw new Error('another site');
        if (push) {
          remember();
          history.pushState({ y: 0 }, '', href);
        }
        swap(doc);
        var id = decodeURIComponent(new URL(href).hash.slice(1));
        var target = id && document.getElementById(id);
        // A new page starts where it should be, without the smooth scroll the page has.
        if (!push && history.state) jump(history.state.y || 0);
        else if (target) target.scrollIntoView({ behavior: 'instant' });
        else jump(0);
        if (keyboard) document.getElementById('main-content').focus({ preventScroll: true });
      }).catch(function () {
        if (n === going) location.assign(href);
      }).then(function () {
        clearTimeout(slow);
        if (n === going) root.classList.remove('loading');
      });
    };
    document.addEventListener('click', function (e) {
      if (e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
      var u = inSite(e.target.closest && e.target.closest('a[href]'));
      if (!u) return;
      if (bare(u.href) === bare(location.href)) {
        // The page that is already open: a heading link scrolls as usual, the page itself goes
        // back to the top without loading again.
        if (u.hash) return;
        e.preventDefault();
        if (u.href !== location.href) history.pushState({ y: 0 }, '', u.href);
        window.scrollTo(0, 0);
        return;
      }
      e.preventDefault();
      go(u.href, true, e.detail === 0);
    });
    window.addEventListener('popstate', function () {
      // Back or forward within the page that is shown: only the scroll changes.
      if (document.body.getAttribute('data-shown') === bare(location.href)) {
        if (history.state) jump(history.state.y || 0);
        return;
      }
      go(location.href, false, false);
    });
    document.body.setAttribute('data-shown', bare(location.href));
  }

  // Puts `doc`'s page in place of this one, keeping the header and, where it is the same list,
  // the sidebar.
  function swap(doc) {
    document.title = doc.title;
    ['meta[name="description"]', 'link[rel="canonical"]'].forEach(function (sel) {
      var old = document.head.querySelector(sel), next = doc.head.querySelector(sel);
      if (old && next) old.replaceWith(document.importNode(next, true));
    });
    document.body.setAttribute('data-page', doc.body.getAttribute('data-page') || '');
    document.body.setAttribute('data-shown', bare(location.href));
    // The header's links stay; only which one is current changes.
    var here = doc.querySelectorAll('.site-nav > a');
    Array.prototype.forEach.call(document.querySelectorAll('.site-nav > a'), function (a, i) {
      var current = here[i] && here[i].getAttribute('aria-current');
      if (current) a.setAttribute('aria-current', current); else a.removeAttribute('aria-current');
    });
    swapSide(doc);
    var main = document.getElementById('main-content'), next = doc.getElementById('main-content');
    main.replaceWith(document.importNode(next, true));
    var toc = document.querySelector('.wrap > .toc'), nextToc = doc.querySelector('.wrap > .toc');
    if (toc) toc.remove();
    if (nextToc) document.getElementById('main-content').after(document.importNode(nextToc, true));
    // Inline scripts in the new content don't run; this does their work.
    window.jacketPage(false);
    setupPage();
  }

  // The next page's sidebar replaces this one in the same frame, keeping the groups the reader
  // opened and its scroll. A long group this one already has in full is moved across, so it never
  // shrinks back to the rows around the current link.
  function swapSide(doc) {
    var side = document.querySelector('details.side'), nav = side && side.querySelector('nav');
    var next = doc.querySelector('details.side nav');
    if (!nav || !next) return;
    var open = Array.prototype.map.call(nav.querySelectorAll('details.nav-cat[open]'), function (d) { return d.getAttribute('data-key'); });
    var state = { k: nav.getAttribute('data-nav'), t: side.scrollTop, o: open };
    next = document.importNode(next, true);
    Array.prototype.forEach.call(next.querySelectorAll('[data-source]'), function (list) {
      var source = list.getAttribute('data-source');
      var full = nav.querySelector('[data-loaded="' + source + '"]');
      if (!full) return;
      var href = bare(location.href);
      Array.prototype.forEach.call(full.querySelectorAll('a'), function (a) {
        if (bare(a.href) === href) a.setAttribute('aria-current', 'page'); else a.removeAttribute('aria-current');
      });
      list.replaceWith(full);
    });
    var label = doc.querySelector('details.side > summary');
    if (label) side.querySelector('summary').textContent = label.textContent;
    nav.replaceWith(next);
    if (window.jacketSide) window.jacketSide(state);
    sideGroups();
  }
})();
